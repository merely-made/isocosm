use super::*;
use isocosm::schema::BodyDocument;

/// An explicit rational mapping into the caller's quantized geometry. D&D
/// specifies double/half dimensions. PF2e specifies footprint categories.
/// Ars specifies Size; an authored geometry mapping is required for it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BodyChange {
    pub numerator: u32,
    pub denominator: u32,
}

impl BodyChange {
    pub fn validate(self) -> Result<(), String> {
        if self.numerator == 0
            || self.denominator == 0
            || self.numerator > 1024
            || self.denominator > 1024
        {
            return Err("invalid body projection scale".into());
        }
        Ok(())
    }
    fn scale(self, value: i32) -> Result<i32, String> {
        self.validate()?;
        let n = i64::from(value)
            .checked_mul(i64::from(self.numerator))
            .ok_or("body scale overflow")?;
        let d = i64::from(self.denominator);
        if n % d != 0 {
            return Err(
                "body projection needs a finer geometry unit; rounding would move attachments"
                    .into(),
            );
        }
        let scaled = n / d;
        i32::try_from(scaled).map_err(|_| "body scale overflow".into())
    }
}

/// A derived document, not a new incarnation or a write to native tissue.
/// Attachment offsets, pivots and extents follow one scale; loss/origin/part
/// addresses remain those of the current base document. Fit/support consumers
/// use the returned document and its caller-owned effect revision.
pub fn project_body(
    map: &MapDocument,
    token: TokenId,
    base: &BodyDocument,
    ars_scale: Option<BodyChange>,
) -> Result<BodyDocument, String> {
    let Some(records) = Records::read(map)? else {
        return Ok(base.clone());
    };
    let mut scale = None;
    for effect in projection::selected(&records, &Target::Creature(token)) {
        if effect.definition.kind != Kind::Size {
            continue;
        }
        scale = Some(match effect.definition.edition {
            Edition::Dnd5 => {
                if effect.size_change > 0 {
                    BodyChange {
                        numerator: 2,
                        denominator: 1,
                    }
                } else {
                    BodyChange {
                        numerator: 1,
                        denominator: 2,
                    }
                }
            },
            Edition::Pathfinder2 => {
                let current = map
                    .sheet(token)
                    .and_then(|s| s.int("size_category"))
                    .ok_or("base Size is undeclared")?;
                let footprint = |size| match size {
                    0 => 1,
                    1 | 2 => 2,
                    3 => 4,
                    4 => 6,
                    _ => 8,
                }; // half-footprint units on a 5-foot board.
                BodyChange {
                    numerator: footprint(if effect.rank >= 4 { 4 } else { 3 }),
                    denominator: footprint(current),
                }
            },
            Edition::Ars5 => ars_scale
                .ok_or("Ars Size needs an authored physical scale; no voxel ratio is implied")?,
        });
    }
    let Some(scale) = scale else {
        return Ok(base.clone());
    };
    scale.validate()?;
    let mut body = base.clone();
    for part in &mut body.parts {
        for k in 0..3 {
            // Native declared planar shapes may carry zero/negative signed
            // extents. Preserve that classifier rather than making a box.
            if part.half_extent[k] > 0 {
                part.half_extent[k] = scale.scale(part.half_extent[k])?.max(1);
            }
            part.pivot[k] = scale.scale(part.pivot[k])?;
            if let Some(attachment) = &mut part.attachment {
                attachment.offset[k] = scale.scale(attachment.offset[k])?;
            }
        }
    }
    Ok(body)
}
