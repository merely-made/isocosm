// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0
import {matchesRule, ruleForPart, svg} from "./drawing.js";

// Oblique anatomical coordinates. The main stem runs across the page, height
// runs up it and the sideways component separates left and right branches.
function project([x,y,z]) { return [(z+0.36*x)*16,(-y+0.88*x)*11]; }

function outline(part) {
    const corners=[];
    for (const x of [-1,1]) for (const y of [-1,1]) for (const z of [-1,1]) {
        const signs=[x,y,z];
        corners.push(project(part.center.map((p,i)=>p+signs[i]*part.half[i])));
    }
    corners.sort((a,b)=>a[0]-b[0] || a[1]-b[1]);
    function boundary(points) {
        const edge=[];
        for (const p of points) {
            while (edge.length>=2) {
                const [a,b]=edge.slice(-2);
                if ((b[0]-a[0])*(p[1]-a[1])-(b[1]-a[1])*(p[0]-a[0])>0) break;
                edge.pop();
            }
            edge.push(p);
        }
        return edge.slice(0,-1);
    }
    return [...boundary(corners),...boundary([...corners].reverse())];
}

export function partName(part) {
    const [run,segment,slot]=part.situs;
    if (!run) return slot ? 'Frond '+(segment+1) : 'Stem '+(segment+1);
    return slot ? 'Defence' : 'Side stem';
}

function sockets(part,founder,map) {
    const [run,,slot]=part.situs;
    const stems=founder.parts.filter(p=>!p.situs[0] && !p.situs[2]).sort((a,b)=>a.situs[1]-b.situs[1]);
    if (run===0 && slot>0) {
        const parent=founder.parts.find(p=>p.id===part.parent);
        return ["Above","Below"].map(frond_socket=>{
            const [x,y,z]=parent.center, sign=frond_socket==="Above"?1:-1;
            return {at:project([x,y+sign*(parent.half[1]+part.half[1]),z]),edit:{frond_socket}};
        });
    }
    if (run!==1 || slot!==0) return [];
    const choices=[];
    // Short stems can express the same position with multiple anchors. Retain
    // the authored anchor where possible so a drop in place round-trips it.
    const anchors={Base:0,Middle:Math.floor(stems.length/2),Tip:stems.length-1};
    for (const anchor of new Set([map.grammar.seed[1].anchor,...Object.keys(anchors)])) {
        const parent=stems[anchors[anchor]];
        for (const side of ["Left","Right"]) {
            const [x,y,z]=parent.center, sign=side==="Right"?1:-1;
            const at=project([x+sign*(parent.half[0]+part.half[0]),y,z]);
            if (!choices.some(c=>c.at[0]===at[0] && c.at[1]===at[1])) choices.push({at,edit:{side,anchor}});
        }
    }
    return choices;
}

export function drawAttachments(root,founder,map,rule,partId,choose,commit) {
    const tissue=svg("g",{"aria-hidden":true,class:"tissue-outlines"});
    const links=svg("g",{"aria-hidden":true,class:"attachments"});
    const nodes=svg("g"), guides=svg("g",{class:"socket-guides","aria-hidden":true});
    root.replaceChildren(tissue,links,nodes,guides);
    const byId=new Map(founder.parts.map(p=>[p.id,p]));
    const bounds=[];
    for (const part of founder.parts) {
        const [x,y]=project(part.center), name=partName(part), source=ruleForPart(part,map);
        const selected=partId===null ? matchesRule(part,map,rule) : part.id===partId;
        const category=source==="repeat" ? "stem" : source;
        const points=outline(part);
        tissue.append(svg("polygon",{points:points.join(" "),class:category+(selected?" selected":"")}));
        bounds.push(...points);
        if (part.parent!==null) {
            const [x1,y1]=project(byId.get(part.parent).center);
            links.append(svg("line",{x1,y1,x2:x,y2:y,"data-parent":part.parent,"data-child":part.id}));
        }
        const choices=sockets(part,founder,map);
        bounds.push(...choices.map(c=>c.at));
        const node=svg("g",{class:"part-node "+category+(selected?" selected":"")+(choices.length?" movable":""),
            "data-part":part.id,transform:`translate(${x} ${y})`,role:"button",tabindex:0,
            "aria-label":name+" · part "+part.id,"aria-pressed":selected});
        node.append(svg("title",{},name+" · part "+part.id),
            svg("circle",{r:14,class:"hit"}),svg("circle",{r:9}),svg("text",{y:3.5},String(part.id)));
        nodes.append(node);
        let start=null, dragged=false, target=null;
        const point=e=>new DOMPoint(e.clientX,e.clientY).matrixTransform(root.getScreenCTM().inverse());
        function cancel() { start=null; target=null; guides.replaceChildren(); }
        node.onkeydown=e=>{
            if (e.key==="Enter"||e.key===" ") { e.preventDefault(); choose(part); }
            if (e.key==="Escape") cancel();
        };
        node.onclick=()=>{ if (!dragged) choose(part); };
        if (!choices.length) continue;
        node.onpointerdown=e=>{
            if (e.button!==0) return;
            start=point(e); dragged=false; node.setPointerCapture(e.pointerId);
        };
        function updateTarget(e) {
            const p=point(e);
            if (!dragged && Math.hypot(p.x-start.x,p.y-start.y)<4) return;
            if (!dragged) {
                for (const {at} of choices) guides.append(svg("circle",{cx:at[0],cy:at[1],r:13}));
                dragged=true;
            }
            const distance=c=>Math.hypot(c.at[0]-p.x,c.at[1]-p.y);
            const closest=choices.reduce((a,b)=>distance(a)<=distance(b)?a:b);
            target=distance(closest)<=20 ? closest : null;
            [...guides.children].forEach((dot,i)=>dot.classList.toggle("target",choices[i]===target));
        }
        node.onpointermove=e=>{ if (start) updateTarget(e); };
        node.onpointerup=e=>{
            if (!start) return;
            updateTarget(e);
            const picked=target;
            cancel();
            if (picked) commit(part,picked.edit);
        };
        node.onpointercancel=cancel;
    }
    const xs=bounds.map(p=>p[0]), ys=bounds.map(p=>p[1]);
    const minX=Math.min(...xs)-22, minY=Math.min(...ys)-20;
    root.setAttribute("viewBox",[minX,minY,Math.max(...xs)+22-minX,Math.max(...ys)+20-minY].join(" "));
}
