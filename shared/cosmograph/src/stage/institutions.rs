// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;

impl Simulation {
    pub(crate) fn finish_staged_institution(&self,stage:&mut Stage,risky:bool) -> Result<()> {
        if stage.institution_producer.is_none() && self.genesis.rules.processes.get(&stage.process)
            .is_some_and(|p|p.requires.iter().any(|q|matches!(q,crate::rules::Query::Institution {..}))) {
            return Err("selected institutional process omitted its actual staged operation marker".into());
        }
        if let Some(input)=&mut stage.institution_producer {
            let (event,_)=stage.event.as_ref().ok_or("institution producer needs the same stage's actual native event")?;
            input.finish(event,stage.institution_decision.as_ref(),risky)?;
        }
        Ok(())
    }
}
