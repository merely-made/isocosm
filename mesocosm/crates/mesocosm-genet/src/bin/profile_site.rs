// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Cold terrain/place preparation and camera residency over an exact saved site.
use std::{path::Path,time::Instant};
use cosmograph::{operations::Window,Simulation,Execution};
use isometer::{BrickSource,ColumnTerrain,SlabCamera,framed_bricks};

fn main() -> Result<(),String> {
    let path=std::env::args().nth(1).ok_or("profile_site <native world trace>")?;
    let saved=mesocosm_genet::played::read_trace(Path::new(&path))?;
    let sim=Simulation::new(saved.genesis.clone(),Execution::Grouped)?;
    let t=sim.genesis().rules.spatial_templates.as_ref().ok_or("saved world has no terrain windows")?;
    let site=t.stands.iter().max_by_key(|(_,s)|s.len()).map(|(id,_)|*id).ok_or("no sites")?;
    let side=sim.genesis().world.footprint.ok_or("no site outline")?.side as i64;
    if std::env::args().any(|s|s=="--probe") { return probe(&sim,t,site,side); }
    let view=cosmograph::terrain::View::of(sim.genesis())?;
    let centre=side/2;
    let mut cold=Vec::new();
    let mut local=Vec::new();
    for width in [64i64,128,256,512].into_iter().filter(|w|*w<=side) {
        let w=Window {min:[centre-width/2;2],max:[centre+width/2;2],depth:8};
        let start=Instant::now();let v=sim.volume(site,w.min,w.max,w.depth)?;
        let volume_us=start.elapsed().as_micros();
        let start=Instant::now();
        let potential_cells=(v.ceiling()-v.datum) as u128*width as u128*width as u128;
        let places = if potential_cells<=8_000_000 {
            Some(isometer::space::places::Places::derive(&v,t.places)?)
        } else {None};
        cold.push(serde_json::json!({"width":width,"columns":width*width,"volume_us":volume_us,
            "potential_cells":potential_cells,"places_us":places.as_ref().map(|_|start.elapsed().as_micros()),
            "bricks":v.ground().brick_count(),"places":places.as_ref().map(|p|p.places.len()),
            "places_limit":8_000_000}));
        println!("cold {}",cold.last().unwrap());
        if width<=256 {
            let start=Instant::now();
            let v=isometer::space::volume::Volume::paged(&view,site,w.min,w.max,w.depth,&[])?;
            let volume_us=start.elapsed().as_micros();let start=Instant::now();
            let places=isometer::space::places::Places::derive(&v,t.places)?;
            local.push(serde_json::json!({"width":width,"columns":width*width,
                "volume_us":volume_us,"places_us":start.elapsed().as_micros(),"places":places.places.len()}));
            println!("local {}",local.last().unwrap());
        }
    }
    let mut frames=Vec::new();
    for half in [28f32,56.,112.,224.,448.,896.] {
        let start=Instant::now();
        let v=isometer::space::volume::Volume::paged(&view,site,[0;2],[side;2],8,&[])?;
        let datum=v.datum;
        let y=v.surface(centre,centre).ok_or("site centre has no terrain")?;
        let mut source=ColumnTerrain::new(v,mesocosm_genet::section::terrain_cell(half));
        let camera=SlabCamera::dimetric_2_1([0.,(y+1-datum) as f32,0.],half,
            1.5,half*6.).unwrap();
        let framed=source.frame(camera,8.,16383);
        let cell=source.cell;
        let frame_us=start.elapsed().as_micros();
        let mut raw=vec![0;512];let start=Instant::now();
        for key in &framed.keys {source.fill(*key,&mut raw);}
        let fill_us=start.elapsed().as_micros();let start=Instant::now();
        let warm=source.frame(camera,8.,16383);
        frames.push(serde_json::json!({"half_height":half,"cell":cell,"bricks":warm.keys.len(),
            "overflow":warm.overflow,"cold_framing_us":frame_us,"fill_us":fill_us,
            "warm_framing_us":start.elapsed().as_micros()}));
        println!("frame {}",frames.last().unwrap());
    }
    println!("summary {}",serde_json::json!({"seed":saved.genesis.seed,"site":site,"side":side,
        "dense_reference":cold,"local_physics":local,"terrain_frames":frames}));
    Ok(())
}

fn probe(sim:&Simulation,t:&cosmograph::spatial::Templates,site:u64,side:i64)->Result<(),String> {
    use isometer::{TerrainSource,PagedTerrain,Residency,ResidencySettings};
    let stand=t.stands[&site].iter().find(|s|s.actor==Some(38)).unwrap_or(&t.stands[&site][0]);
    let pose=cosmograph::spatial::Pose {at:stand.at,yaw:stand.yaw,feet:stand.feet.clone()};
    let doc=sim.state().population.get(stand.actor.unwrap()).unwrap().body.as_ref().unwrap();
    let view=cosmograph::terrain::View::of(sim.genesis())?;
    let v=isometer::space::volume::Volume::paged(&view,site,[0;2],[side;2],8,&[])?;
    let camera=SlabCamera {centre:v.to_ground(stand.at).map(|x|x as f32),
        forward:mesocosm_genet::section::CameraMode::DimetricNorth.forward(),
        half_height:28.,aspect:1.5,depth:168.,cutaway:None};
    let contacts=cosmograph::spatial::embodiment::contacts(&v,doc,&pose)?;
    let source=ColumnTerrain::new(v.clone(),1);
    let framed=framed_bricks(camera,8.,&source,16383);
    let limits=isometer::lens::AtlasLimits {max_atlas_bytes:8*1024*1024,
        ..isometer::lens::AtlasLimits::DEFAULT};
    let residency=std::cell::RefCell::new(Residency::new(ResidencySettings::default(),limits));
    let map=PagedTerrain::new(&residency,&source,&framed,0).brick_map()?;
    let samples:Vec<_>=contacts.iter().map(|c| {
        let native=[c.at[0],c.at[1]-1,c.at[2]];let grid=v.to_ground(native);
        serde_json::json!({"native":native,"grid":grid,"native_material":v.material(native),
            "map_material":map.material_at(grid),"framed":framed.keys.contains(&grid.map(|v|v.div_euclid(8) as i16)),
            "ray":map.trace_ray([grid[0] as f32+0.5,grid[1] as f32+10.,grid[2] as f32+0.5],[0.,-1.,0.],32.).map(|h|h.map(|h|(h.point,h.material))).map_err(|e|e.to_string())})
    }).collect();
    println!("probe {}",serde_json::json!({"datum":v.datum,"ceiling":v.ceiling(),"side":side,
        "origin":map.origin(),"root":stand.at,"camera":format!("{camera:?}"),
        "bricks":framed.keys.len(),"overflow":framed.overflow,"samples":samples}));
    Ok(())
}
