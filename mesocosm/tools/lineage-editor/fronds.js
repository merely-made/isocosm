// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0
import {drawBody, svg} from "./drawing.js";

const $ = id => document.getElementById(id);
const colors = ["#315f45", "#9a4d25"];
let worlds = [], timer, generation = 0;
const elem = (tag, text, cls) => {
    const node = document.createElement(tag);
    if (text !== undefined) node.textContent = text;
    if (cls) node.className = cls;
    return node;
};
async function request(path, body) {
    const res = await fetch(path, {method:"POST", headers:{"Content-Type":"application/json"}, body:JSON.stringify(body)});
    if (!res.ok) throw new Error(await res.text());
    return res;
}
function stop() { clearInterval(timer); timer = null; $("play").textContent = "Play"; }
function anatomy(world) {
    const box = document.querySelector(".forms"); box.replaceChildren();
    for (const [i,title] of ["One broad frond","Three small fronds"].entries()) {
        const panel=elem("article",undefined,"panel study"), text=elem("div");
        const body=world.founders.find(p => p.form === i);
        text.append(elem("h2",title,i ? "split":"broad"));
        if (body) {
            const image=svg("svg",{role:"img","aria-label":title+" grown from the founding genome"});
            const leaves=body.parts.filter(p => p.cells["function:fix"]);
            const cells=leaves.reduce((n,p)=>n+p.cells["function:fix"],0);
            const volume=leaves.reduce((n,p)=>n+p.half.reduce((a,h)=>a*(2*h+1),1),0);
            drawBody(image,body,world.map,null,null,0.36);
            panel.append(image);
            text.append(elem("p",cells+" fixing cells · "+volume+" frond voxels"));
        } else text.append(elem("p","No founder drew this form."));
        panel.append(text); box.append(panel);
    }
}
function chart(world,tick,max) {
    const W=400,H=132,L=31,B=111,R=389,T=12;
    const root=svg("svg",{viewBox:"0 0 400 132",class:"plot",role:"img","aria-label":"Living plants by tick. Solid green is broad, dashed brown is split."});
    const x=t=>L+t*(R-L)/(world.frames.length-1), y=n=>B-n*(B-T)/max;
    for (const n of [0,max]) {
        root.append(svg("line",{x1:L,x2:R,y1:y(n),y2:y(n),stroke:"#c2c9b9"}),svg("text",{x:L-4,y:y(n)+4,"text-anchor":"end","font-size":11},String(n)));
    }
    for(let i=0;i<2;i++) root.append(svg("polyline",{points:world.frames.map(f=>[x(f.tick),y(f.living[i])].join(",")).join(" "),
        fill:"none",stroke:colors[i],"stroke-width":2.5,"stroke-dasharray":i?"6 3":"none"}));
    root.append(svg("line",{x1:x(tick),x2:x(tick),y1:T,y2:B,stroke:"#4c514a","stroke-dasharray":"2 3"}));
    for(const t of [0,world.frames.length-1]) root.append(svg("text",{x:x(t),y:H-4,"font-size":11,"text-anchor":"middle"},String(t)));
    return root;
}
async function save(world, tick, enter) {
    try {
        const res=await request(enter?"/enter":"/save",{edits:world.edits,tick,hash:world.frames[tick].hash,launch_key:world.launch_key});
        if(enter){const result=await res.json(); $("study-status").textContent="Opened tick "+tick+" in Mesocosm. "+result.folder;return;}
        const url=URL.createObjectURL(await res.blob()), a=elem("a");
        a.href=url; a.download="fronds-"+world.seed+"-"+(world.edits.grazers?"grazed":"ungrazed")+"-tick-"+tick+".json"; a.click(); setTimeout(()=>URL.revokeObjectURL(url),1000);
    } catch(e){$("study-status").textContent=e.message;}
}
function render() {
    const tick=+$("tick").value; $("time").value=tick;
    const box=document.querySelector(".worlds"); box.replaceChildren();
    const max=Math.max(1,...worlds.flatMap(w=>w.frames.flatMap(f=>f.living)));
    for(const world of worlds) {
        const f=world.frames[tick], panel=elem("section",undefined,"panel study"), table=elem("table"), head=elem("tr");
        for(const [i,title] of ["","Broad","Split"].entries()) head.append(elem("th",title,["","broad","split"][i]));
        table.append(head);
        for(const [title,values] of [["Living plants",f.living],["Active / exposed cells",f.active.map((n,i)=>n+" / "+f.exposed[i])],
            ["Births so far",f.totals.births],["Soil fixed",f.totals.soil_fixed],["Biomass grazed",f.totals.grazed]]) {
            const row=elem("tr"); row.append(elem("th",title),...values.map(n=>elem("td",n))); table.append(row);
        }
        panel.append(elem("h2",world.edits.grazers?"With grazers":"Without grazers"),elem("p",f.grazers+" living grazers · plant population through tick "+(world.frames.length-1)),chart(world,tick,max),table);
        const button=elem("button","Save at tick "+tick);button.onclick=()=>save(world,tick,false);panel.append(button);
        if(world.launch_key){const enter=elem("button","Enter world");enter.onclick=()=>save(world,tick,true);panel.append(enter);}
        box.append(panel);
    }
}
function dimensions(id) {
    const sides=$(id).value.trim().split(/\s*[x×,]\s*/i);
    if(sides.length!==3 || !sides.every(n=>/^\d+$/.test(n) && +n>=3 && +n<=129 && +n%2===1))
        throw new Error("Use three odd voxel dimensions between 3 and 129 for each frond.");
    return sides.map(n=>(+n-1)/2);
}
async function load() {
    if(!$("seed").reportValidity()) return;
    const gen=++generation, seed=+$("seed").value, socket=$("facing").value;
    stop(); $("play").disabled=$("tick").disabled=true;
    $("study-status").textContent="Growing the founders and running the paired worlds…";
    document.querySelector(".worlds").replaceChildren(); document.querySelector(".forms").replaceChildren(); worlds=[];
    try {
        const whole=dimensions("whole"), piece=dimensions("piece");
        const pair=await Promise.all([false,true].map(async grazers=>{
            const edits={study:"fronds",seed,frond_socket:socket,whole,piece,grazers};
            return {...await (await request("/preview",{edits})).json(),edits};
        }));
        if(gen!==generation) return;
        worlds=pair; anatomy(pair[0]); $("tick").max=pair[0].frames.length-1; $("tick").value=0;
        $("play").disabled=$("tick").disabled=false; render(); $("study-status").textContent="Seed "+seed+" · "+pair[0].founders.length+" plant founders in each world";
    } catch(e) {if(gen===generation) $("study-status").textContent=e.message;}
}
$("play").onclick=()=>{if(timer){stop();return;}if(+$("tick").value===+$("tick").max)$("tick").value=0;render();$("play").textContent="Pause";
    timer=setInterval(()=>{$("tick").value=+$("tick").value+1;render();if(+$("tick").value===+$("tick").max)stop();},500);};
$("tick").oninput=()=>{stop();render();}; $("grow").onclick=load; load();
