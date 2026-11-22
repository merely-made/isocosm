// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0
import {drawBody, drawChart, graphEditor, matchesRule, nodes, ruleForPart, svg} from "./drawing.js";

import {drawAttachments, partName} from "./anatomy.js";
import {showSurvival} from "./survival.js";

const $ = id => document.getElementById(id);
let data, selectedId, selectedRule = "gland", selectedPart = null;
let revision = 0, ready = false, pendingSave = false;
let mapView="parts";
let edits = {mapped_sites: true, direct_grazers: true};
let quarterTurns = 0, pickingAnchor = false, setFlex, swayFrame;
const graph = graphEditor($("graph"), rule => selectRule(rule));
const founder = () => data.founders.find(f => f.id === selectedId);
const selectedFrame = () => data.frames[Number($("tick").value)];
const formName = f => f.gland ? "With defence" : "Fronds only";

function status(message, error=false) {
    $("status").textContent = message;
    $("status").classList.toggle("error", error);
}
function controls() {
    $("found").disabled = !ready || pendingSave;
    $("enter").hidden = !data?.launch_key;
    $("enter").disabled = !ready || pendingSave;
    $("template").disabled = !ready;
    $("pick-anchor").disabled = !ready;
    $("tick").disabled = !ready;
    for (const id of ["odds","repeat","rent","cohort","organ","anchor","side","variance","grain","bend","orbit","sites","sway","frond-socket"]) $(id).disabled = !data;
}
async function post(path, body) {
    const response = await fetch(path, {method:"POST",headers:{"Content-Type":"application/json"},body:JSON.stringify(body)});
    if (!response.ok) throw Error((await response.text()).trim() || response.statusText);
    return response;
}
function download(blob, name) {
    const link=document.createElement("a"), url=URL.createObjectURL(blob);
    link.href=url; link.download=name; document.body.append(link); link.click(); link.remove();
    setTimeout(() => URL.revokeObjectURL(url), 10000);
}
function selectRule(rule, part=null) {
    pickingAnchor=false;
    $("body").classList.remove("placing");
    selectedRule=rule;
    selectedPart=part?.id ?? null;
    graph.select(rule);
    if (data) drawSpecimen();
}
function setMapView(view) {
    mapView=view;
    $("parts-tab").setAttribute("aria-pressed",view==="parts");
    $("rules-tab").setAttribute("aria-pressed",view==="rules");
    $("attachment-view").hidden=view!=="parts";
    $("rule-view").hidden=view!=="rules";
    if (data) drawSpecimen();
}
function pickPart(part,from) {
    selectRule(ruleForPart(part,data.map),part);
    $(from).querySelector(`[data-part="${part.id}"]`)?.focus({preventScroll:true});
}
function drawPartKey(f) {
    const groups=[["stem","Stem",p=>p.situs[0]===0 && p.situs[2]===0],
        ["fronds","Fronds",p=>p.situs[0]===0 && p.situs[2]>0],
        ["side","Side stem",p=>p.situs[0]===1 && p.situs[2]===0],
        ["gland","Defence",p=>p.situs[0]===1 && p.situs[2]>0]];
    $("part-key").replaceChildren(...groups.map(([key,name,match])=>{
        const parts=f.parts.filter(match), b=document.createElement("button");
        b.textContent=name+" "+(parts.length?parts.map(p=>p.id).join(", "):"· absent");
        b.setAttribute("aria-pressed",selectedRule===key);
        b.onclick=()=>selectRule(key);
        return b;
    }));
}
function drawSpecimen() {
    const f=founder(), map=data.map, g=map.grammar;
    if (mapView==="parts") {
        drawAttachments($("attachment-map"),f,map,selectedRule,selectedPart,
            part=>pickPart(part,"attachment-map"),(part,change)=>{
                if (!ready || pendingSave) return;
                selectedRule=ruleForPart(part,map); selectedPart=part.id;
                edits={...edits,...change}; preview();
            });
        drawPartKey(f);
    }
    showSurvival(data,f);
    $("cohort").value=String(f.id);
    for (const b of $("gallery").children) b.setAttribute("aria-pressed",Number(b.dataset.id)===f.id);
    $("form").textContent=formName(f);
    $("rule-title").textContent=nodes[selectedRule].label;
    const [main,side]=g.seed;
    const descriptions={
        stem:`The seed starts with ${main.segments} main stem segment. Each segment bears a frond.`,
        repeat:`Repeat adds ${g.rules[0].Repeat.by} to the main stem. Each segment that develops carries its own frond.`,
        fronds:`Each main stem segment grows ${main.per_segment} frond at its ${main.socket.toLowerCase()} socket. Its fixing tissue keeps it present in both forms.`,
        side:`The side stem attaches at the ${side.anchor.toLowerCase()} of the main stem, facing ${side.facing.toLowerCase()}. It can carry a defence on its ${side.socket.toLowerCase()} socket.`,
        gland:`The side stem can grow ${map.regions["region:gland"].replace("part:","")}. ${data.cohort_size===1 ? "Each founder draws separately." : "Each founder cohort draws once."} Offspring inherit that form.`,
    };
    $("rule-desc").textContent=descriptions[selectedRule];
    $("placement").hidden=selectedRule!=="side";
    $("frond-row").hidden=selectedRule!=="fronds";
    $("frond-socket").value=main.socket;
    $("grain-row").hidden=$("variance-row").hidden=selectedRule!=="repeat" && selectedRule!=="stem";
    $("bank-row").hidden=$("trait-list").hidden=selectedRule!=="gland";
    const entry=data.bank.find(p => p.key===map.regions["region:gland"]);
    $("trait-list").textContent=entry ? entry.traits.map(t=>t.replace("trait:","").replaceAll("-"," ")).join(" · ") + ` · ${entry.shape.replace("part-shape:","")} · spine ${entry.spine}, toxin ${entry.toxin} · ${entry.cells} cells` : "";
    $("odds-row").hidden=selectedRule!=="gland";
    $("repeat-row").hidden=selectedRule!=="repeat";
    setFlex=drawBody($("body"), f, map, selectedRule, (rule,part) => {
        if (pickingAnchor) {
            const [run, segment, slot]=part.situs;
            if (run!==0 || slot!==0) { status("Pick a segment of the main stem."); return; }
            const length=f.parts.filter(p=>p.situs[0]===0 && p.situs[2]===0).length;
            const anchor=segment===0 ? "Base" : segment===length-1 ? "Tip" :
                segment===Math.floor(length/2) ? "Middle" : null;
            if (!anchor) { status("Choose the base, middle or tip segment."); return; }
            selectRule("side"); edit("anchor",anchor); return;
        }
        pickPart(part,"body");
    }, quarterTurns, Number($("bend").value), selectedPart);
    const matches=f.parts.filter(p => matchesRule(p,map,selectedRule));
    const part=matches.find(p => p.id===selectedPart) ?? matches[0];
    let detail="This cohort grew no part from the selected rule.";
    if (part) {
        const cells=Object.entries(part.cells).map(([key,n]) => `${n} ${key.replace("function:","")}`).join(" · ");
        const parent=f.parts.find(p=>p.id===part.parent);
        detail=`Part ${part.id} · ${partName(part)} · ${parent?"attached to "+partName(parent)+" (#"+parent.id+")":"root"} · ${cells} cells`;
        if (selectedPart===null && matches.length>1) detail+=` · ${matches.length} matching parts`;
    } else if (selectedRule==="gland") { detail="This cohort omitted its defensive organ. Its fronds still grow."; }
    $("anatomy").textContent=detail;
    if (selectedRule==="fronds" && part && matchesRule(part,map,"repeat")) {
        const source=document.createElement("button");
        source.className="source-rule"; source.textContent="From Repeat";
        source.onclick=() => selectRule("repeat"); $("anatomy").append(" ",source);
    }
}
function chooseCohort(id) {
    const situs=selectedSitus();
    selectedId=Number(id);
    restorePart(situs);
    drawSpecimen();
}

// IDs are local to a developed body. A change in length or organ presence can
// give the same ID to different tissue. Follow the growth address if it exists.
function selectedSitus() {
    return data && founder()?.parts.find(p=>p.id===selectedPart)?.situs;
}
function restorePart(situs) {
    const part=situs && founder().parts.find(p=>p.situs.every((v,i)=>v===situs[i]));
    selectedPart=part?.id ?? null;
    if (part) selectedRule=ruleForPart(part,data.map);
    graph.select(selectedRule);
}

function drawGallery() {
    $("cohort").replaceChildren(); $("gallery").replaceChildren();
    for (const f of data.founders) {
        const label=`#${f.id} · ${formName(f)} · ${f.count} ${f.count===1 ? "plant" : "plants"}`;
        $("cohort").add(new Option(label,f.id));
        const b=document.createElement("button"), mini=svg("svg",{"aria-hidden":"true"});
        const number=document.createElement("small"); number.textContent="#"+f.id;
        b.className="thumb"; b.dataset.id=f.id; b.title=label; b.setAttribute("aria-label",label);
        b.onclick=() => chooseCohort(f.id);
        drawBody(mini,f,data.map,null,null);
        b.append(mini,number); $("gallery").append(b);
    }
}
function drawHistory() {
    const frame=selectedFrame(), t=frame.totals;
    $("tick-label").value=frame.tick;
    $("found").textContent=`Save world at tick ${frame.tick}`;
    $("stats").replaceChildren();
    for (const [label,counts] of [["Alive",frame.living],["Born since founding",t.births],["Matter grazed",t.grazed]]) {
        const tr=document.createElement("tr"), th=document.createElement("th");
        th.scope="row"; th.textContent=label; tr.append(th);
        for (const n of counts) { const td=document.createElement("td"); td.textContent=n; tr.append(td); }
        $("stats").append(tr);
    }
    $("costs").textContent=`Defence upkeep paid: ${t.rent} matter. Grazer cells injured: ${t.wounds} by toxin, ${t.spine_wounds} by spines.`;
    drawChart($("chart"),data.frames,frame);
}
async function preview() {
    const version=++revision;
    ready=false; controls(); status("Growing founders and simulating 35 ticks…");
    try {
        const result=await (await post("/preview",{edits})).json();
        if (version!==revision) return;
        const situs=selectedSitus();
        data=result;
        if (!data.founders.some(f => f.id===selectedId))
            selectedId=(data.founders.find(f => f.gland) ?? data.founders[0]).id;
        const [n,d]=data.map.absence ?? [0,1];
        $("odds").value=`${n}/${d}`;
        $("repeat").value=data.map.grammar.rules[0].Repeat.by;
        $("rent").value=data.rent_cells;
        $("organ").replaceChildren(...data.bank.map(p=>new Option(p.key.replace("part:","").replaceAll("-"," "),p.key)));
        $("organ").value=data.map.regions["region:gland"];
        $("grain").value=data.cohort_size;
        $("sites").value=data.mapped_sites ? (data.direct_grazers ? "playable" : "mapped") : "single";
        $("variance").value=data.map.grammar.seed[0].variance ?? 0;
        $("side").value=data.map.grammar.seed[1].facing;
        $("anchor").value=data.map.grammar.seed[1].anchor;
        $("seed").textContent="Seed "+data.seed;
        $("tick").max=data.frames.length-1;
        $("end-tick").textContent=$("tick").max;
        restorePart(situs); drawGallery(); drawSpecimen(); drawHistory();
        const [plain,gland]=data.frames[0].living;
        $("subtitle").textContent=`One plant lineage · ${plain+gland} founders in ${data.founders.length} cohorts · ${plain} fronds only, ${gland} with a defence`;
        status(""); ready=true;
    } catch (e) {
        if (version===revision) status(`Preview unavailable. ${e.message}`,true);
    } finally { if (version===revision) controls(); }
}
function edit(key,value) { edits={...edits,[key]:value}; preview(); }

$("enter").onclick=async () => {
    if (!ready || pendingSave) return;
    const frame=selectedFrame();
    pendingSave=true; controls();
    try {
        const result=await (await post("/enter",{edits,tick:frame.tick,hash:frame.hash,launch_key:data.launch_key})).json();
        status(`Launched Mesocosm at tick ${frame.tick}. World files: ${result.folder}`);
    } catch (e) { status(`Cannot enter world. ${e.message}`,true); }
    finally { pendingSave=false; controls(); }
};
$("pick-anchor").onclick=() => {
    pickingAnchor=!pickingAnchor;
    $("body").classList.toggle("placing",pickingAnchor);
    status(pickingAnchor ? "Pick the base, middle or tip of the main stem on the specimen." : "");
};
$("orbit").onclick=() => { quarterTurns=(quarterTurns+1)%4; drawSpecimen(); };
function stopSway() {
    cancelAnimationFrame(swayFrame);
    swayFrame=undefined;
    $("sway").textContent="Preview sway";
    $("sway").setAttribute("aria-pressed","false");
    $("angle").value=$("bend").value+"°";
    setFlex?.(Number($("bend").value));
}
$("bend").oninput=stopSway;
$("sway").onclick=() => {
    if (swayFrame!==undefined) { stopSway(); return; }
    const start=performance.now(), amplitude=Number($("bend").value) || 18;
    function pose(now) {
        const angle=amplitude*Math.sin((now-start)*0.002);
        setFlex?.(angle);
        $("angle").value=Math.round(angle)+"°";
        swayFrame=requestAnimationFrame(pose);
    }
    $("sway").textContent="Stop sway";
    $("sway").setAttribute("aria-pressed","true");
    swayFrame=requestAnimationFrame(pose);
};
$("parts-tab").onclick=()=>setMapView("parts");
$("rules-tab").onclick=()=>setMapView("rules");
$("inspect-frond").onclick=()=>{
    setMapView("parts");
    const frond=founder().parts.find(p=>p.cells["function:fix"]>0);
    if (frond) {
        pickPart(frond,"attachment-map");
        $("attachment-map").scrollIntoView({block:"nearest"});
    }
};
$("frond-socket").onchange=e=>edit("frond_socket",e.target.value);
$("organ").onchange=e => edit("organ",e.target.value);
$("side").onchange=e => edit("side",e.target.value);
$("anchor").onchange=e => edit("anchor",e.target.value);
$("sites").onchange=e => {
    edits={...edits,mapped_sites:e.target.value!=="single",direct_grazers:e.target.value==="playable"};
    preview();
};
$("grain").onchange=e => edit("cohort_size",Number(e.target.value));
$("variance").onchange=e => edit("stem_variance",Number(e.target.value));
$("cohort").onchange=e => chooseCohort(e.target.value);
$("odds").onchange=e => edit("absence",e.target.value.split("/").map(Number));
$("repeat").onchange=e => edit("repeat",Number(e.target.value));
$("rent").onchange=e => edit("rent_cells",Number(e.target.value));
$("tick").oninput=() => { if (ready) { drawHistory(); status(""); } };
$("layout").onclick=() => graph.reset();
$("template").onclick=() => {
    download(new Blob([data.source],{type:"application/toml"}),`mesocosm-seed-${data.seed}.toml`);
    status("Exported the founding template.");
};
$("found").onclick=async () => {
    if (!ready || pendingSave) return;
    const version=revision, frame=selectedFrame(), seed=data.seed;
    pendingSave=true; controls(); status(`Checking the native world at tick ${frame.tick}…`);
    try {
        const save=await (await post("/save",{edits,tick:frame.tick,hash:frame.hash})).blob();
        if (version===revision) {
            download(save,`mesocosm-seed-${seed}-tick-${frame.tick}.json`);
            status(`Downloaded the Cosmograph world at tick ${frame.tick}.`);
        }
    } catch (e) { if (version===revision) status(`Save failed. ${e.message}`,true); }
    finally { pendingSave=false; controls(); }
};
graph.select(selectedRule);
preview();
