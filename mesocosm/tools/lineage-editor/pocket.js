// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0
const $ = id => document.getElementById(id);
const colors = {host:"#996327", resident:"#306b55"};
const sides = ["stay","leave"];
let data, playing, sequence=0, applied;
const stop = () => { clearInterval(playing); playing=null; $("play").textContent="Play"; };
async function post(url, payload) {
    const r=await fetch(url,{method:"POST",headers:{"Content-Type":"application/json"},body:JSON.stringify(payload)});
    if (!r.ok) throw Error(await r.text());
    return r;
}
function sections() {
    $("paths").innerHTML=sides.map((s,i)=>`<article class="panel" id="${s}">
        <div class="padded branch-head"><h2>${i?"Departure trial":"Resident stays"}</h2><span class="contact"></span></div>
        <svg class="viewbox" viewBox="0 0 360 120" role="img" aria-label="Front views of the two grown bodies"></svg>
        <div class="padded"><div class="key"><span><i style="background:${colors.host}"></i> Host</span><span><i style="background:${colors.resident}"></i> Resident</span><strong>Body tissue over time</strong></div>
        <svg class="tissue" viewBox="0 0 360 68" preserveAspectRatio="none" role="img" aria-label="Host and resident tissue on a shared scale"></svg><div class="live"></div>
        <div class="ledger"></div><div class="tools"><button class="save">Save world at this tick</button><button class="enter">Enter world</button></div></div></article>`).join("");
    for(const side of sides) for(const action of ["save","enter"]) $(side).querySelector("."+action).onclick=()=>choose(side,action);
}
// Draw front elevations from native part centres and half-extents. Each body has its own origin.
function elevation(body,x,color) {
    const bounds=body.parts.flatMap(p=>[p.center[1]-p.half[1],p.center[1]+p.half[1]]);
    const low=Math.min(...bounds),high=Math.max(...bounds),width=Math.max(1,...body.parts.map(p=>Math.abs(p.center[0])+p.half[0]));
    const scale=Math.min(5.5,62/width,69/Math.max(1,high-low));
    const parts=[...body.parts].sort((a,b)=>a.center[2]-b.center[2]);
    return parts.map(p=>{
        const [cx,cy]=p.center,[w,h]=p.half;
        const marked=body.port.includes(p.id)||body.frond===p.id;
        return `<rect x="${x+(cx-w)*scale}" y="${85-(cy+h-low)*scale}" width="${2*w*scale}" height="${2*h*scale}" fill="${marked?color:"#c6cbbc"}" stroke="#415348" stroke-width=".9"><title>Part ${p.id}</title></rect>`;
    }).join("");
}
function anatomy(f,previous) {
    const exchanging=f.contact&&f.host.alive&&f.resident.alive;
    const soil=f.flows.soil-previous.soil,sugar=f.flows.sugar-previous.sugar;
    const link=exchanging?`<g fill="none" stroke-linecap="round">
        <path d="M148 46H211m-5 -4 5 4 -5 4" stroke="${colors.host}" stroke-width="${soil?2.7:1}"/>
        <path d="M211 67H148m5 -4 -5 4 5 4" stroke="${colors.resident}" stroke-width="${sugar?2.7:1}"/></g>
        <g font-size="10" text-anchor="middle" fill="#344639"><text x="180" y="39">${soil||""} soil</text><text x="180" y="82">${sugar||""} sugar</text></g>`:"";
    return link+elevation(f.host,88,colors.host)+elevation(f.resident,273,colors.resident)+
        `<g font-size="12" text-anchor="middle" fill="#344639"><text x="88" y="107">Fungal host #${f.host.id}</text><text x="273" y="107">Fixing resident #${f.resident.id}</text></g>`;
}
function chart(frames, tick, peak) {
    const x = index => 22 + 334 * index / (frames.length - 1);
    const y = amount => 57 - 48 * amount / peak;
    const departure = x(applied.leave_at);
    const axes = `<g fill="#536256" font-size="9">
        <text x="1" y="11">${peak}</text><text x="7" y="58">0</text>
        <text x="${departure}" y="67" text-anchor="middle">leave ${applied.leave_at}</text></g>
        <path d="M22 57H356" stroke="#b9c5b6"/>
        <path d="M${departure} 7V57" stroke="#879889" stroke-dasharray="3 3"/>`;
    return axes + ["host", "resident"].map(role => {
        const points = frames.slice(0, tick + 1)
            .map((frame, index) => x(index) + "," + y(frame[role].tissue)).join(" ");
        return `<polyline points="${points}" fill="none" stroke="${colors[role]}" stroke-width="2.4"/>
            <circle cx="${x(tick)}" cy="${y(frames[tick][role].tissue)}" r="3" fill="${colors[role]}"/>`;
    }).join("");
}
const partList = ids => ids.map(id=>"#"+id).join(", ");
function readings(f) {
    const {host:h,resident:r,site}=f, terms=data.terms;
    return [
        ["Open exchange pocket","Host · floor and two working walls",h.opening&&h.alive?"Parts "+partList(h.opening):"Missing open, living lining"],
        ["Deliver and receive","Resident · exchange organ",r.port.length&&r.alive?"Part "+partList(r.port):"Missing working organ"],
        ["Fix tissue","Resident · exposed fixing frond",r.frond!=null&&r.alive?"Part #"+r.frond:"Missing exposed fixing tissue"],
        ["Fixing conditions","Patch · light and soil",`${site.light} light · ${site.soil} soil · ${site.habitable?"habitable":"uninhabitable"}`],
        ["Nutrient delivery","Host → resident",`${r.soil} soil held by resident · ${f.contact&&h.alive&&r.alive?"connected":"disconnected"}`],
        ["Sugar delivery","Resident → host",`${h.sugar} sugar held by host · ${f.contact&&h.alive&&r.alive?"connected":"disconnected"}`],
        ["Build resident tissue","Resident · fixes soil using patch light",`${f.fixed} tissue made so far`],
        ["Build host tissue","Host · digests sugar from resident",`${f.digested} tissue made from sugar`],
        ["Carrying cost","Host · own tissue",`${terms.fee} per exchange · ${f.flows.fee} paid`]
    ];
}
function paint() {
    if(!data)return;
    const t=Number($("tick").value), rows=[], histories=[data.frames,data.leave];
    $("time").textContent=t+" / "+(data.frames.length-1);
    const peak=Math.max(1,...histories.flatMap(fs=>fs.flatMap(f=>[f.host.tissue,f.resident.tissue])));
    sides.forEach((side,i)=>{
        const root=$(side), trail=histories[i], f=trail[t], before=trail[Math.max(0,t-1)].flows, flows=f.flows;
        const shared = side === "leave" && t < applied.leave_at;
        let contact = "No resident in pocket";
        if (f.contact) contact = f.host.alive && f.resident.alive ? "Living together" : "Contact ended by death";
        else if (i && t >= applied.leave_at && data.frames[0].contact) contact = "Separated";
        root.querySelector(".contact").textContent = shared ? "Shared history · " + contact : contact;
        root.querySelector(".save").textContent = shared ? "Save shared world at this tick" : "Save world at this tick";
        root.querySelector(".viewbox").innerHTML=anatomy(f,before);
        root.querySelector(".tissue").innerHTML=chart(trail,t,peak);
        root.querySelector(".live").textContent=`Host ${f.host.tissue} (${f.host.alive?"alive":"dead"}) · Resident ${f.resident.tissue} (${f.resident.alive?"alive":"dead"}) · Offspring ${f.host.offspring} host / ${f.resident.offspring} resident`;
        root.querySelector(".ledger").innerHTML=`<strong>Exchange this tick</strong> · ${flows.soil-before.soil} soil → resident · ${flows.sugar-before.sugar} sugar → host<br><small>Since founding · ${flows.soil} soil delivered · ${flows.sugar} sugar delivered · ${flows.fee} host tissue spent</small>`;
        root.querySelector(".enter").hidden=!data.launch_key;
        rows.push(readings(f));
    });
    $("requires").innerHTML=rows[0].map(([name,supplier,v],i)=>`<tr><td>${name}</td><td>${supplier}</td><td>${v}</td><td>${rows[1][i][2]}</td></tr>`).join("");
}
async function grow(e) {
    e?.preventDefault(); stop(); const n=++sequence;
    data=null; $("paths").replaceChildren(); $("requires").replaceChildren();
    $("play").disabled=$("tick").disabled=true; $("status").textContent="Growing the founders…";
    const options={study:"pocket",seed:Number($("seed").value),pocket:$("pocket").checked,port:$("port").checked,leave_at:Number($("departure").value)};
    try {
        const next=await (await post("/preview",{edits:options})).json(); if(n!==sequence)return;
        applied=options; data=next; sections(); $("tick").max=data.frames.length-1; $("tick").value=0;
        $("play").disabled=$("tick").disabled=false;
        const {nutrient, sugar, fee, period} = data.terms;
        $("contract").textContent = `Every ${period} sim minutes the pair can exchange ${nutrient} soil for ${sugar} sugar. Carrying it costs the host ${fee} tissue.`;
        const atDeparture = data.frames[options.leave_at];
        if (!data.frames[0].contact) {
            $("status").textContent = "The two founders live on the same patch. Their grown structures cannot establish pocket contact.";
        } else if (!atDeparture.resident.alive || !atDeparture.host.alive) {
            const lost = !atDeparture.resident.alive ? "resident" : "host";
            $("status").textContent = `The ${lost} has already died by tick ${options.leave_at}. Choose an earlier departure to separate the living pair.`;
        } else {
            $("status").textContent = `Same founders and history until the departure at tick ${options.leave_at}. Scrub to compare.`;
        }
        paint();
    }catch(err){if(n===sequence)$("status").textContent=err.message;}
}
async function choose(side,action) {
    if(!data)return; stop(); const t=Number($("tick").value),f=(side==="stay"?data.frames:data.leave)[t], seed=applied.seed;
    try{
        const r=await post("/"+action,{edits:{...applied,branch:side},tick:t,hash:f.hash,launch_key:data.launch_key});
        if(action==="enter"){const result=await r.json();$("status").textContent="Opened world from "+result.folder;return;}
        const url=URL.createObjectURL(await r.blob()), a=document.createElement("a");a.href=url;a.download=`pocket-${seed}-${side}-tick-${t}.json`;a.click();setTimeout(()=>URL.revokeObjectURL(url),1500);
        $("status").textContent=`Saved the ${side} world at tick ${t}.`;
    }catch(err){$("status").textContent=err.message;}
}
$("settings").onsubmit=grow; $("tick").oninput=()=>{stop();paint();};
$("play").onclick=()=>{if(playing){stop();return;} if(+$("tick").value>=+$("tick").max)$("tick").value=0;
    $("play").textContent="Pause"; playing=setInterval(()=>{$("tick").stepUp();paint();if(+$("tick").value>=+$("tick").max)stop();},550);};
grow();
