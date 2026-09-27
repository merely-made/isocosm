//! Local display settings and the last terrain report, separate from campaigns.

pub const MIB: u64 = 1024 * 1024;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TerrainSettings {
    pub budget_mib: u32,
    pub device_budget: Option<u64>,
    pub omitted: usize,
}

impl Default for TerrainSettings {
    fn default() -> Self {
        Self {
            budget_mib: 8,
            device_budget: None,
            omitted: 0,
        }
    }
}

impl TerrainSettings {
    pub fn budget_bytes(&self) -> u64 {
        u64::from(self.budget_mib) * MIB
    }

    pub fn max_mib(&self) -> u32 {
        self.device_budget
            .map_or(8, |bytes| (bytes / MIB) as u32)
            .max(1)
    }

    pub fn step(&mut self, delta: i32) {
        if self.device_budget.is_none() {
            return;
        }
        let effective = self.budget_mib.min(self.max_mib());
        if (delta > 0 && effective == self.max_mib()) || (delta < 0 && effective == 1) {
            return;
        }
        self.budget_mib = effective
            .saturating_add_signed(delta)
            .clamp(1, self.max_mib());
    }

    /// Reports device limits without turning a clamped default into a new
    /// preference. The source bounds the requested bytes on that device.
    pub fn report(&mut self, device_budget: u64, omitted: usize) {
        self.device_budget = Some(device_budget);
        self.omitted = omitted;
    }
}
