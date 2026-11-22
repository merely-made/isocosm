// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

const $=id=>document.getElementById(id);
const plural=(n, noun)=>n+" "+noun+(n===1?"":"s");

export function showSurvival(data, founder) {
    const last=data.frames.at(-1), index=Number(founder.gland), t=last.totals;
    const fixing=founder.parts.filter(p=>p.cells["function:fix"]>0);
    const exposed=fixing.filter(p=>p.canopy).length;
    $("check-range").textContent="Founding → tick "+last.tick;
    const role=founder.trophic?.toLowerCase();
    const needsCanopy=data.inputs.needs_fixing_canopy;
    $("feeding").classList.toggle("mismatch",needsCanopy && !exposed);
    const tissue=plural(exposed,"fixing frond")+" in the canopy.";
    if (needsCanopy) {
        $("feeding").textContent=tissue+" "+(exposed
            ? "This founder can use light to turn soil into body matter."
            : "Growth is blocked. This founder has no exposed fixing tissue.");
    } else {
        $("feeding").textContent=tissue+" This founding allows producer growth at any frond position.";
    }
    if (role) $("feeding").textContent+=" Its anatomy reads as a "+role+".";
    const s=founder.site;
    $("site-reading").textContent=`Each growth act needs ${data.inputs.light} light and ${data.inputs.soil} soil. `+
        (s ? `Site ${s.id} starts with ${s.light.toLocaleString()} stored light and ${s.soil} soil. ${s.habitable?"The site is habitable.":"The site is uninhabitable."}` : "This founder has no settled site.");
    $("inspect-frond").disabled=!fixing.length;
    const start=data.frames[0].living[index], end=last.living[index], births=t.births[index];
    $("form-trial").textContent=(index?"Defended":"Fronds-only")+" plants in this world";
    $("survival-result").textContent=`${start} founded · ${end} alive at tick ${last.tick} · ${births} born.`;
    $("survival-budget").textContent=`${t.soil_fixed[index]} soil converted to body matter. ${t.upkeep[index]} matter paid for basic upkeep, ${t.defence_upkeep[index]} for defence and ${t.grazed[index]} lost to grazers.`;
    const present=data.reference.filter(r=>r.initial[index]>0);
    const persisted=present.filter(r=>r.living[index]>0).length;
    const reproduced=present.filter(r=>r.births[index]>0).length;
    $("reference-result").textContent=present.length
        ? `This form persisted to tick ${last.tick} in ${persisted} of ${present.length} worlds where it was founded. It reproduced in ${reproduced}. Each seed redraws the founders and their habitat.`
        : "None of the three reference seeds founded this form.";
    $("reference-runs").replaceChildren(...data.reference.map(r=>{
        const row=document.createElement("tr");
        for (const value of [r.seed,r.initial[index],r.living[index],r.births[index]]) {
            const cell=document.createElement("td"); cell.textContent=value; row.append(cell);
        }
        return row;
    }));
}
