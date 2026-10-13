// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Per-window settings over the party's exact retained native disclosures.

use cambium::{el,text,graph_atlas_swatch,GraphAtlasEvent};
use isometry_campaign::native::overmap::{self,ViewSettings};
use super::{model::{App,Defaults},view::{Child,button}};
use crate::products::Mode;

impl App {
    fn vtt_map_settings(&mut self,key:u64,settings:Option<ViewSettings>)->Result<(),String> {
        let local=self.windows.get_mut(&key).ok_or("window is closed")?;
        let mut defaults:Defaults=serde_json::from_slice(&local.native.binding().view_defaults)
            .map_err(|error|error.to_string())?;
        defaults.overmap=settings;
        local.native.set_view_defaults(serde_json::to_vec(&defaults).map_err(|error|error.to_string())?)?;
        local.preferences.modes.insert("vtt".into(),defaults);
        self.refresh();
        Ok(())
    }
}

pub(super) fn panel(app:&App,key:u64)->Child {
    let mut rows:Vec<Child>=vec![Box::new(el("h3",text("Disclosed overmap")))];
    let read=(|| {
        let local=app.windows.get(&key).ok_or("window is closed")?;
        let access=local.native.projection()?.access;
        let defaults:Defaults=serde_json::from_slice(&local.native.binding().view_defaults)
            .map_err(|error|error.to_string())?;
        let party=defaults.party.ok_or("Choose an authored party")?;
        app.world.read_game(Mode::Vtt,|session,bytes|overmap::read_overmap(session,bytes,access,&party,defaults.overmap))
    })();
    let read=match read {Ok(read)=>read,Err(reason)=>{
        rows.push(Box::new(el("p",text(reason))));
        return Box::new(el("section",rows).attr("class","panel"));
    }};
    rows.push(Box::new(el("p",text(match &read.disclosure {
        Some(source)=>format!("Disclosures through {source}. Access and membership are current."),
        None=>"All retained disclosures. Access and membership are current.".into(),
    }))));
    if let Some(settings)=&read.settings {
        for (label,field,value) in [("Biome layer",0,settings.biomes),("Faction layer",1,settings.factions),
            ("Thumbnails",2,settings.thumbnails==Some(true))] {
            let mut selected=settings.clone();
            match field {0=>selected.biomes=!value,1=>selected.factions=!value,_=>selected.thumbnails=Some(!value)};
            rows.push(button(format!("map-layer-{field}"),format!("{label}: {}",if value {"on"} else {"off"}),key,
                move |app|app.vtt_map_settings(key,Some(selected.clone()))));
        }
        for source in &read.available_disclosures {
            let mut selected=settings.clone(); selected.disclosure=Some(source.clone());
            rows.push(button(format!("map-history-{source}"),format!("Read through {source}"),key,
                move |app|app.vtt_map_settings(key,Some(selected.clone()))));
        }
        let mut selected=settings.clone(); selected.disclosure=None;
        rows.push(button("map-history-live".into(),"Read all retained disclosures".into(),key,
            move |app|app.vtt_map_settings(key,Some(selected.clone()))));
        rows.push(button("map-inherit".into(),"Use world map settings".into(),key,
            move |app|app.vtt_map_settings(key,None)));
    }
    if let Some(swatch)=isometry_views::native_overmap_swatch(&read) {
        rows.push(Box::new(graph_atlas_swatch(&swatch,move |app:&mut App,event| {
            if let GraphAtlasEvent::Activate(id)=event
                && let Some(site)=id.strip_prefix("site:").and_then(|id|id.parse().ok()) {
                // Shared map facts do not grant close-up awareness. Examine
                // goes through this window's existing native seat cut.
                let result=app.examine(key,Some(site)); app.record(key,result);
            }
        })));
    }
    for limitation in read.limitations {rows.push(Box::new(el("p",text(limitation))));}
    Box::new(el("section",rows).attr("class","panel"))
}
