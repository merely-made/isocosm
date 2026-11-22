// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

const NS = "http://www.w3.org/2000/svg";
export function svg(tag, attrs = {}, content = "") {
    const el = document.createElementNS(NS, tag);
    for (const [key, value] of Object.entries(attrs)) el.setAttribute(key, value);
    if (content) el.textContent = content;
    return el;
}

export function ruleForPart(part, map) {
    const [run, segment, slot] = part.situs;
    if (map.grammar.rules.some(r => r.Partition?.tagma === run)) return "fronds";
    if (slot > 0) return map.grammar.seed[run].bears === "region:gland" ? "gland" : "fronds";
    if (run > 0) return "side";
    return repeated(segment,map) ? "repeat" : "stem";
}

function repeated(segment,map) {
    const base=map.grammar.seed[0].segments;
    return segment>=base && segment<base+(map.grammar.rules.find(r => r.Repeat?.tagma === 0)?.Repeat.by ?? 0);
}

export function matchesRule(part, map, rule) {
    const [run, segment, slot] = part.situs;
    return ruleForPart(part, map) === rule ||
        (rule === "stem" && run === 0 && slot === 0) ||
        (rule === "repeat" && run === 0 && repeated(segment,map));
}

// A body-space box for each native part, in the front, side and top dimetric faces.
// World Y is up. All founder frames in this growth map have zero yaw.
const palettes = {
    stem: ["#b3bb85", "#829265", "#566f50"],
    fronds: ["#9ec19f", "#6d9a78", "#4e795c"],
    gland: ["#f5c66e", "#d99840", "#ae722b"],
};
function project([x, y, z]) { return [x - z, (x + z) / 2 - y]; }

// Rotate in body space before projection. Bend each fixing leaf about the
// centre of its attached face so the point on the stem stays fixed.
function bodyPoint(part, vertex, turn, bend, socketSign) {
    let [x,y,z] = vertex;
    if (bend && part.cells["function:fix"]) {
        const [cx,cy] = part.center;
        const base = cy-socketSign*part.half[1], dx=x-cx, dy=y-base;
        const angle=bend*Math.PI/180;
        x=cx+dx*Math.cos(angle)-dy*Math.sin(angle);
        y=base+dx*Math.sin(angle)+dy*Math.cos(angle);
    }
    for (let i=0;i<turn;i++) [x,z]=[z,-x];
    return [x,y,z];
}

export function drawBody(root, founder, map, selected, onPart, turn=0, bend=0, partId=null) {
    root.replaceChildren();
    const layer = svg("g");
    root.append(layer);
    const bounds = [Infinity, Infinity, -Infinity, -Infinity];
    const flex = [];
    const socketSign=map.grammar.seed[0].socket==="Below" ? -1 : 1;
    const depth = p => bodyPoint(p,p.center,turn,bend,socketSign).reduce((a,b)=>a+b,0);
    const sorted = [...founder.parts].sort((a, b) => depth(a) - depth(b));
    for (const part of sorted) {
        const rule = ruleForPart(part, map);
        const highlighted = partId===null ? matchesRule(part,map,selected) : part.id===partId;
        const colors = palettes[rule] ?? palettes.stem;
        const group = svg("g", {"data-rule": rule, "data-part": part.id});
        const title = `Part ${part.id} · ${rule} · ${Object.keys(part.cells).map(k => k.replace("function:", "")).join(", ")}`;
        group.append(svg("title", {}, title));
        if (onPart) {
            group.setAttribute("role", "button");
            group.setAttribute("aria-pressed",highlighted);
            group.setAttribute("tabindex", "0");
            group.setAttribute("aria-label", title);
            group.onclick = () => onPart(rule, part);
            group.onkeydown = e => {
                if (["Enter", " "].includes(e.key)) { e.preventDefault(); onPart(rule, part); }
            };
        }
        const [x, y, z] = part.center;
        const [w, h, d] = part.half;
        const faces = [
            [[1,1,-1],[1,1,1],[-1,1,1],[-1,1,-1]],
            [[-1,-1,1],[1,-1,1],[1,1,1],[-1,1,1]],
            [[1,-1,-1],[1,1,-1],[1,1,1],[1,-1,1]],
        ];
        for (const [index, face] of faces.entries()) {
            const corners = face.map(([a,b,c]) => {
                // Rotate face normals back into body space to select the three
                // camera-facing surfaces at every quarter turn.
                for (let i=0;i<turn;i++) [a,c]=[-c,a];
                return [x+a*w,y+b*h,z+c*d];
            });
            const projected = angle => corners.map(c => project(bodyPoint(part,c,turn,angle,socketSign)));
            const points = projected(bend);
            for (const [u,v] of points) {
                bounds[0] = Math.min(bounds[0],u); bounds[1] = Math.min(bounds[1],v);
                bounds[2] = Math.max(bounds[2],u); bounds[3] = Math.max(bounds[3],v);
            }
            const polygon = svg("polygon", {points: points.map(p => p.join(",")).join(" "),
                fill:colors[index], stroke:highlighted ? "#573e17" : "#4b634e",
                "stroke-width":highlighted ? 1.9 : 0.65, "vector-effect":"non-scaling-stroke"});
            group.append(polygon);
            if (onPart && part.cells["function:fix"])
                flex.push(angle => polygon.setAttribute("points", projected(angle).join(" ")));
        }
        layer.append(group);
    }
    const [l,t,r,b] = bounds, pad = onPart ? 2.5 : 1;
    if (onPart) {
        // Name the functional tissue at its developed location, including when
        // a repeat changes the number and placement of the fronds.
        for (const [rule, direction, title] of [["fronds",-1,"Fronds"],["gland",1,"Defence"]]) {
            const parts = founder.parts.filter(p => ruleForPart(p,map) === rule);
            if (!parts.length) continue;
            const part = parts[0], [x,y] = project(bodyPoint(part,part.center,turn,bend,socketSign));
            const elbow = direction < 0 ? l-2 : r+2, end = elbow+direction*5;
            const label = svg("g", {class:"callout","aria-hidden":"true"});
            label.append(svg("path",{d:`M ${x} ${y} L ${elbow} ${y-1.5} H ${end}`,
                fill:"none",stroke:"#677667","stroke-width":0.8,"vector-effect":"non-scaling-stroke"}),
                svg("circle",{cx:x,cy:y,r:.23,fill:"#455643"}),
                svg("text",{x:end+.55*direction,y:y-1.05,"text-anchor":direction<0?"end":"start",
                    "font-size":1.75,fill:"#314a3a"},title));
            if (rule==="fronds") {
                const [line,dot,caption] = label.children;
                flex.push(angle => {
                    const [px,py] = project(bodyPoint(part,part.center,turn,angle,socketSign));
                    line.setAttribute("d", `M ${px} ${py} L ${elbow} ${py-1.5} H ${end}`);
                    dot.setAttribute("cx",px); dot.setAttribute("cy",py);
                    caption.setAttribute("y",py-1.05);
                });
            }
            label.onclick=() => onPart(rule,part);
            layer.append(label);
        }
    }
    const sidePad = onPart ? 18 : pad;
    root.setAttribute("viewBox", `${l-sidePad} ${t-pad} ${r-l+2*sidePad} ${b-t+2*pad}`);
    return angle => { for (const apply of flex) apply(angle); };
}

export const nodes = {
    repeat: {label:"Repeat", ref:"grammar.rules[0]", pos:[22,20]},
    stem: {label:"Main stem", ref:"seed[0] · segment", pos:[224,20]},
    fronds: {label:"Fronds", ref:"seed[0] · bears", pos:[224,110]},
    side: {label:"Side stem", ref:"seed[1] · segment", pos:[22,110]},
    gland: {label:"Defensive organ", ref:"region:gland", pos:[22,197]},
};
const links = [["repeat","stem"],["stem","fronds"],["stem","side"],["side","gland"]];
const layoutKey = "mesocosm.lineage-editor.optional-gland.layout.v1";
const clamp = (v, max) => Math.min(max, Math.max(0, v));

export function graphEditor(root, choose) {
    const positions = Object.fromEntries(Object.entries(nodes).map(([key,n]) => [key,[...n.pos]]));
    try {
        const stored = JSON.parse(localStorage.getItem(layoutKey));
        for (const k of Object.keys(nodes)) {
            const p = stored?.[k];
            if (Array.isArray(p) && p.length === 2 && p.every(Number.isFinite))
                positions[k] = [clamp(p[0],246),clamp(p[1],205)];
        }
    } catch { /* Layout storage is optional. */ }
    const edges = links.map(() => svg("path", {class:"edge"}));
    root.append(...edges);
    const elements = {};
    function place() {
        for (const [k,[x,y]] of Object.entries(positions))
            elements[k].setAttribute("transform", `translate(${x} ${y})`);
        for (const [i,[a,b]] of links.entries()) {
            const [x,y] = positions[a], [u,v] = positions[b];
            if (Math.abs(y-v) < 43) {
                const direction = Math.sign(u-x);
                edges[i].setAttribute("d", `M ${x+77+direction*77} ${y+22} L ${u+77-direction*77} ${v+22}`);
            } else {
                const direction = Math.sign(v-y);
                const start = y+22+direction*22, end = v+22-direction*22, mid = (start+end)/2;
                edges[i].setAttribute("d", `M ${x+77} ${start} C ${x+77} ${mid} ${u+77} ${mid} ${u+77} ${end}`);
            }
        }
    }
    function storeLayout() { try { localStorage.setItem(layoutKey, JSON.stringify(positions)); } catch {} }
    function at(event) {
        return new DOMPoint(event.clientX,event.clientY).matrixTransform(root.getScreenCTM().inverse());
    }
    for (const [key,n] of Object.entries(nodes)) {
        const g = svg("g",{class:"node",role:"button",tabindex:0,"data-rule":key,"aria-label":n.label});
        elements[key] = g;
        g.append(svg("rect",{width:154,height:44,rx:6}),
                 svg("text",{x:11,y:19},n.label),svg("text",{x:11,y:34,class:"ref"},n.ref));
        root.append(g);
        let drag = null;
        g.onkeydown = e => {
            if (["Enter"," "].includes(e.key)) { e.preventDefault(); choose(key); }
        };
        g.onpointerdown = e => {
            if (e.button !== 0) return;
            drag = {start:at(e), origin:[...positions[key]], moved:false};
            g.setPointerCapture(e.pointerId);
        };
        g.onpointermove = e => {
            if (!drag) return;
            const p = at(e), dx = p.x-drag.start.x, dy = p.y-drag.start.y;
            if (!drag.moved && Math.hypot(dx,dy)<3) return;
            drag.moved = true;
            positions[key] = [clamp(drag.origin[0]+dx,246),clamp(drag.origin[1]+dy,205)];
            place();
        };
        g.onpointerup = () => { if (drag?.moved) storeLayout(); drag = null; };
        g.onclick = () => choose(key);
        g.onpointercancel = () => { if (drag) positions[key]=drag.origin; drag=null; place(); };
    }
    place();
    return {
        select(key) { for (const [k,g] of Object.entries(elements)) {
            g.classList.toggle("active", k === key); g.setAttribute("aria-pressed", k === key);
        } },
        reset() { for (const [k,n] of Object.entries(nodes)) positions[k]=[...n.pos]; place(); storeLayout(); },
    };
}

export function drawChart(root, frames, selected) {
    const left=26, right=473, top=8, bottom=104;
    const peak=Math.max(1,...frames.flatMap(f => f.living));
    const x=t => left + (right-left)*t/(frames.length-1);
    const y=n => bottom - (bottom-top)*n/peak;
    root.replaceChildren();
    for (const value of [0,Math.round(peak/2),peak]) {
        root.append(svg("line",{x1:left,y1:y(value),x2:right,y2:y(value),stroke:"#d5dbcf"}),
            svg("text",{x:left-6,y:y(value)+3,"text-anchor":"end","font-size":10,fill:"#546356"},String(value)));
    }
    for (const [index,color] of ["#3b6855","#9c611e"].entries()) {
        // A count changes at a tick boundary. Interpolating would invent fractional plants.
        const points = frames.map((f,i) => i ? `H ${x(i)} V ${y(f.living[index])}` : `M ${x(0)} ${y(f.living[index])}`);
        root.append(svg("path",{d:points.join(" "),fill:"none",stroke:color,"stroke-width":2.6,
            "stroke-dasharray":index ? "6 3" : "none"}));
    }
    const tick=selected.tick;
    root.append(svg("line",{x1:x(tick),x2:x(tick),y1:top,y2:bottom,stroke:"#566058","stroke-dasharray":"2 2"}));
    selected.living.forEach((n,i) => root.append(svg("circle",{cx:x(tick),cy:y(n),r:3.6,fill:i ? "#9c611e" : "#3b6855",stroke:"#fff"})));
    root.setAttribute("aria-label", `Living plants at tick ${tick}: ${selected.living[0]} fronds only and ${selected.living[1]} with a defence.`);
}
