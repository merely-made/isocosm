// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

import { drawSection, sectionColony } from './soil-view.js'
import { PlanetView } from './globe.js'
import { FieldMap, kindOf, sizeOf, measure } from './field-map.js'

const $ = (id) => document.getElementById(id)
const node = (tag, text, cls) => {
  const el = document.createElement(tag)
  if (text != null) el.textContent = text
  if (cls) el.className = cls
  return el
}
async function request(url, body) {
  const res = await fetch(
    url,
    body
      ? {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify(body),
        }
      : undefined,
  )
  if (!res.ok) throw new Error('Could not read the field (' + res.status + ').')
  return res.json()
}
function specimen(canvas, shape, sharedSpan, color) {
  const w = (canvas.width = 104),
    h = (canvas.height = 60)
  canvas.style.width = '52px'
  canvas.style.height = '30px'
  const c = canvas.getContext('2d')
  if (!shape.length) {
    c.fillStyle = '#536959'
    c.font = '22px ui-monospace, monospace'
    c.textAlign = 'center'
    c.fillText('—', w / 2, h * 0.65)
    return
  }
  const rects = shape.map(([at, b]) => [
    at[0] - b[0],
    at[2] - b[2],
    2 * b[0],
    2 * b[2],
    at[1] + b[1],
  ])
  const x = (Math.min(...rects.map((r) => r[0])) + Math.max(...rects.map((r) => r[0] + r[2]))) / 2
  const z = (Math.min(...rects.map((r) => r[1])) + Math.max(...rects.map((r) => r[1] + r[3]))) / 2
  const k = (h - 5) / sharedSpan
  for (const [l, t, width, height] of rects.sort((a, b) => a[4] - b[4])) {
    c.fillStyle = color
    c.fillRect(w / 2 + (l - x) * k, h / 2 + (t - z) * k, width * k, height * k)
    c.strokeStyle = '#405142'
    c.lineWidth = 0.75
    c.strokeRect(w / 2 + (l - x) * k, h / 2 + (t - z) * k, width * k, height * k)
  }
}
function account(key) {
  return key.startsWith('matter:') ? 'tissue' : key.split(':').at(-1).replaceAll('-', ' ')
}
function atSite(f, s) {
  return f.poses.filter((p) => p.site === s)
}
function findResident(rows, id) {
  return rows.find((r) => id >= r.first && id - r.first < r.count)
}
async function open() {
  const query = new URLSearchParams(location.search)
  const [terrain, latest] = await Promise.all([request('field/terrain'), request('field/frame')])
  const canvas = $('field'),
    map = new FieldMap(canvas, terrain)
  let globe,
    orbital = false
  const cache = new Map()
  let f = latest.frame,
    head = latest.head,
    selected = null,
    site,
    autocenter = true,
    playing = false,
    busy = false,
    revision = 0,
    raf = null,
    next = null,
    portion = 0
  const remember = (packet) => {
    head = Math.max(head, packet.head)
    cache.set(packet.frame.tick, packet.frame)
    if (cache.size > 60) cache.delete(cache.keys().next().value)
    return packet.frame
  }
  remember(latest)
  const shown = async (tick) =>
    cache.get(tick) || remember(await request('field/frame?tick=' + tick))
  const initialTick = Number(query.get('tick'))
  if (
    query.has('tick') &&
    Number.isSafeInteger(initialTick) &&
    initialTick >= 0 &&
    initialTick <= head
  )
    f = await shown(initialTick)
  const initialSite = Number(query.get('site'))
  site =
    query.has('site') && Number.isSafeInteger(initialSite) && terrain.sites[initialSite]
      ? initialSite
      : f.fieldSite
  map.soilLayer = !!f.sites[site].soil && query.get('layer') !== 'surface'
  const name = (id) => {
    const s = terrain.sites[id],
      d = s.neighbors.map(([i]) => terrain.sites[i].elevation - s.elevation)
    return (Math.max(Math.min(d[0], d[2]), Math.min(d[1], d[3])) > 0 ? 'Valley ' : 'Site ') + id
  }
  const count = (s) => s.residents.reduce((sum, r) => sum + r.count, 0)
  const draw = () => {
    if (orbital) {
      globe.size()
      globe.backdrop()
      const occupied = f.sites.flatMap((s, id) => (s.residents.length ? [id] : []))
      globe.orbit(globe.vector(site), name(site), occupied, [])
      $('extent').textContent = 'Dots mark inhabited sites'
    } else {
      map.draw(f, site, selected, next, portion)
      if (map.soilLayer && f.sites[site].soil)
        drawSection($('profile'), terrain, site, f, map, selected)
      $('extent').textContent = measure(map.span, terrain.unitMicrometres) + ' across'
    }
  }
  function mode(on) {
    orbital = on
    if (on) {
      globe ??= new PlanetView(canvas, terrain)
      globe.center(globe.vector(site))
    }
    $('orbit').textContent = on ? 'Return to ground' : 'Globe'
    $('lens').textContent = on ? 'Orbital view' : 'Ground survey'
    $('ground-tools').hidden = on
    canvas.setAttribute(
      'aria-label',
      on
        ? 'Globe. Drag to rotate or choose a site to visit.'
        : 'Overhead habitat. Drag to pan or select a body.',
    )
    draw()
  }
  const focus = () => {
    const poses = atSite(f, site)
    const bed = f.sites[site].soil
    if (map.soilLayer && bed) {
      const colony = poses.find((p) => p.soil)
      if (colony) {
        const [x, , z] = colony.pose.at
        const lx = (bed.litter[0][0] + bed.litter[1][0]) / 2
        const lz = (bed.litter[0][2] + bed.litter[1][2]) / 2
        map.center((x + lx) / 2, (z + lz) / 2)
        map.span = Math.max(76, (Math.abs(lz - z) + 22) * canvas.clientWidth / canvas.clientHeight)
        autocenter = false
        return
      }
    }
    map.focus(poses, f.sites[site].flows)
    autocenter = !poses.length
  }
  function selectSite(id) {
    site = id
    selected = null
    focus()
    if (orbital) mode(false)
    refresh()
  }
  function clock() {
    $('step').disabled = busy
    $('play').disabled = false
    $('play').textContent = playing ? 'Pause' : 'Play'
    $('tick').textContent = 'Tick ' + f.tick
    $('scrub').max = head
    $('scrub').value = f.tick
    $('latest').disabled = f.tick === head
  }
  function refresh() {
    if (autocenter && atSite(f, site).length) focus()
    const bed = f.sites[site].soil,
      soil = !!bed && map.soilLayer && !orbital
    $('layer-tools').hidden = !bed || orbital
    $('soil-section').hidden = !soil
    $('soil-layer').setAttribute('aria-pressed', String(soil))
    $('surface-layer').setAttribute('aria-pressed', String(!soil))
    $('lens').textContent = orbital
      ? 'Orbital view'
      : soil
        ? 'Shallow soil · plan'
        : 'Ground survey'
    if (soil) {
      const colony = sectionColony(f, site, selected, map.z)
      const at = bed.colonies.find((c) => c.id === colony?.id)
      const plant =
        bed.roots.find((r) => r.id === selected) || bed.roots.find((r) => r.id === at?.host)
      $('soil-reading').textContent =
        (at
          ? 'Fungus #' +
            at.id +
            ' · ' +
            at.parts +
            (at.parts === 1 ? ' part · ' : ' parts · ') +
            (at.contact ? 'touching litter' : 'away from litter') +
            ' · ' +
            at.p +
            ' available P. '
          : '') +
        (plant
          ? 'Plant #' +
            plant.id +
            ' · ' +
            plant.p +
            ' available P · ' +
            plant.used +
            ' incorporated P. '
          : at ? 'No living root in contact. ' : '') +
        'Litter holds ' +
        bed.remaining +
        ' P.'
    }
    const local = f.sites[site],
      rows = local.residents,
      poses = atSite(f, site)
    if (selected != null && !findResident(rows, selected)) selected = null
    $('planet').textContent =
      'Planet ' + terrain.seed + ' · ' + measure(terrain.side, terrain.unitMicrometres) + ' site' +
      (bed ? (f.branch.includes('-near-') ? ' · Near litter' : ' · Offset litter') : '')
    $('place').textContent = name(site)
    $('sites').value = String(site)
    for (const option of $('sites').options) {
      const id = Number(option.value)
      option.textContent = name(id) + ' · ' + count(f.sites[id])
    }
    const byKind = new Map()
    for (const r of rows) byKind.set(r.kingdom, (byKind.get(r.kingdom) || 0) + r.count)
    const plurals = { Fungus: 'fungi', Plant: 'plants', Grazer: 'grazers', Microbe: 'microbes' }
    $('census').textContent = byKind.size
      ? [...byKind]
          .map(
            ([key, n]) =>
              n + ' ' + (n === 1 ? kindOf(key)[0].toLowerCase() : plurals[kindOf(key)[0]]),
          )
          .join(' · ')
      : 'No residents at this tick.'
    const sharedSpan = Math.max(1, ...f.bodies.flatMap((shape) => sizeOf(shape).slice(0, 2)))
    const roster = $('roster')
    roster.replaceChildren()
    for (const row of rows)
      for (let id = row.first; id < row.first + row.count; id++) {
        const pose = poses.find((p) => p.id === id),
          [kind, color] = kindOf(row.kingdom),
          shape = f.bodies[row.body]
        const btn = node('button'),
          thumb = node('canvas'),
          label = node('span', kind + ' #' + id)
        btn.setAttribute('aria-pressed', String(selected === id))
        thumb.setAttribute('aria-hidden', 'true')
        specimen(thumb, shape, sharedSpan, color)
        const dims = sizeOf(shape)
        label.append(
          node(
            'small',
            (dims.length
              ? measure(Math.max(...dims), terrain.unitMicrometres) + ' span'
              : 'Size unresolved') +
              ' · ' +
              (pose?.soil ? 'In soil' : pose ? 'On ground' : 'Site only'),
          ),
        )
        btn.append(thumb, label)
        btn.onclick = () => {
          selected = selected === id ? null : id
          if (selected != null && pose) {
            if (orbital) mode(false)
            if (pose.soil) map.soilLayer = true
            else if (pose.kingdom !== 'kingdom:flora') map.soilLayer = false
            map.center(pose.pose.at[0], pose.pose.at[2])
            autocenter = false
          }
          refresh()
        }
        roster.append(btn)
      }
    const unplaced = count(local) - poses.length
    $('resolution').textContent = unplaced
      ? poses.length +
        ' positioned · ' +
        unplaced +
        ' at site resolution.' +
        (f.fieldSite === site
          ? ''
          : f.tick === head
            ? ' Advance a tick to examine this ground.'
            : ' This tick examined ' + name(f.fieldSite) + '. Choose Latest to examine new ground.')
      : ''
    const events = $('flows')
    events.replaceChildren()
    const organism = (id) => {
      const row = findResident(rows, id)
      return row ? kindOf(row.kingdom)[0] + ' #' + id : 'Organism #' + id
    }
    const flows = local.flows.filter(
      (x) => selected == null || x.from === selected || x.to === selected,
    )
    const priority = (x) =>
      x.process.startsWith('ecology:feed') ? 0 : x.from != null || x.to != null ? 1 : 2
    for (const x of flows
      .slice()
      .sort((a, b) => priority(a) - priority(b) || b.amount - a.amount)) {
      const src = x.from == null ? 'Site' : organism(x.from),
        dst = x.to == null ? 'Site' : organism(x.to)
      const event = node('p', src + ' → ' + dst, 'event')
      const process = ({
        'ecology:root-exchange': 'Root exchange',
        'ecology:recover-phosphate': 'Litter uptake',
        'ecology:root-production': 'Plant production',
      })[x.process] || x.process.slice(8).replace(/-\d+$/, '')
      event.append(
        node(
          'small',
          process +
            ' · ' +
            x.amount +
            ' ' +
            (x.process === 'ecology:root-exchange' && x.debit.startsWith('matter:') ? 'plant carbon' : account(x.debit)) +
            (account(x.credit) === account(x.debit) ? '' : ' → ' + account(x.credit)),
        ),
      )
      events.append(event)
    }
    if (!events.children.length)
      events.append(
        node(
          'p',
          f.tick === 0
            ? 'Advance time to see material move.'
            : 'No external transfers recorded ' +
                (selected == null ? 'here' : 'for #' + selected) +
                ' this tick.',
          'quiet',
        ),
      )
    const stores = $('stores')
    stores.replaceChildren()
    for (const [key, n] of Object.entries(local.accounts)) {
      const el = node('p', null, 'balance')
      el.append(node('span', account(key)), node('strong', n.toLocaleString()))
      stores.append(el)
    }
    $('neighbors').replaceChildren()
    terrain.sites[site].neighbors.forEach(([id], i) => {
      const b = node('button', ['↑ N', '→ E', '↓ S', '← W'][i] + ' · ' + id)
      b.title = 'Visit ' + name(id)
      b.onclick = () => selectSite(id)
      $('neighbors').append(b)
    })
    clock()
    draw()
  }
  function stop() {
    playing = false
    revision++
    cancelAnimationFrame(raf)
    raf = null
    next = null
    portion = 0
    clock()
    draw()
  }
  async function advance(animate) {
    if (busy) return
    const rev = revision,
      source = f.tick
    busy = true
    clock()
    try {
      let observed
      if (source < head) observed = await shown(source + 1)
      else {
        $('status').textContent = 'Resolving the next tick in ' + name(site) + '…'
        observed = remember(await request('field/advance', { tick: source, site }))
      }
      $('status').textContent = ''
      if (rev !== revision) return
      if (animate && playing) {
        next = observed
        const start = performance.now()
        const paint = (now) => {
          if (rev !== revision) return
          portion = Math.min(1, (now - start) / 1250)
          if (portion < 1) {
            if (next.walks?.some((w) => w.site === site)) draw()
            raf = requestAnimationFrame(paint)
            return
          }
          f = observed
          next = null
          portion = 0
          refresh()
          if (playing) advance(true)
        }
        raf = requestAnimationFrame(paint)
      } else {
        f = observed
        refresh()
      }
    } catch (err) {
      stop()
      $('status').textContent = err.message
    } finally {
      busy = false
      clock()
    }
  }
  async function seek(tick) {
    stop()
    const rev = revision
    try {
      const dest = await shown(tick)
      if (rev === revision) {
        f = dest
        refresh()
      }
    } catch (err) {
      $('status').textContent = err.message
    }
  }
  for (let i = 0; i < terrain.sites.length; i++) $('sites').add(new Option(name(i), i))
  $('sites').onchange = (e) => selectSite(Number(e.target.value))
  $('play').onclick = () => {
    if (playing) stop()
    else if (!busy) {
      playing = true
      clock()
      advance(true)
    }
  }
  $('step').onclick = () => {
    stop()
    advance(false)
  }
  $('scrub').oninput = (e) => seek(Number(e.target.value))
  $('latest').onclick = () => seek(head)
  $('soil-layer').onclick = () => {
    map.soilLayer = true
    focus()
    refresh()
  }
  $('surface-layer').onclick = () => {
    map.soilLayer = false
    focus()
    refresh()
  }
  $('orbit').onclick = () => {
    mode(!orbital)
    refresh()
  }
  $('more').onclick = () => {
    map.zoom(0.65)
    draw()
  }
  $('less').onclick = () => {
    map.zoom(1 / 0.65)
    draw()
  }
  $('fit').onclick = () => {
    map.fit()
    autocenter = false
    draw()
  }
  canvas.onwheel = (e) => {
    e.preventDefault()
    if (orbital) return
    map.zoom(e.deltaY > 0 ? 1.15 : 1 / 1.15)
    draw()
  }
  let pointer = null
  canvas.onpointerdown = (e) => {
    if (!e.isPrimary) return
    pointer = {
      id: e.pointerId,
      x: e.clientX,
      y: e.clientY,
      drag: false,
      ox: map.x,
      oz: map.z,
      yaw: globe?.yaw,
      pitch: globe?.pitch,
    }
    canvas.setPointerCapture(e.pointerId)
  }
  canvas.onpointermove = (e) => {
    if (!pointer || pointer.id !== e.pointerId) return
    const dx = e.clientX - pointer.x,
      dy = e.clientY - pointer.y
    if (Math.hypot(dx, dy) > 3) pointer.drag = true
    if (pointer.drag && orbital) {
      globe.dragging = true
      globe.yaw = pointer.yaw
      globe.pitch = pointer.pitch
      globe.rotate(dx * 0.007, dy * 0.007)
      draw()
      return
    }
    if (pointer.drag) {
      autocenter = false
      const k = map.span / canvas.clientWidth
      map.center(pointer.ox - dx * k, pointer.oz - dy * k)
      draw()
    }
  }
  canvas.onpointerup = (e) => {
    if (!pointer || e.pointerId !== pointer.id) return
    if (!pointer.drag) {
      const r = canvas.getBoundingClientRect(),
        x = e.clientX - r.left,
        y = e.clientY - r.top
      if (orbital) {
        const id = globe.siteAt(x, y)
        if (id != null) selectSite(id)
      } else {
        selected = map.pick(x, y) ?? null
        refresh()
      }
    }
    if (orbital) {
      globe.dragging = false
      draw()
    }
    pointer = null
  }
  canvas.onpointercancel = () => {
    pointer = null
    if (orbital) {
      globe.dragging = false
      draw()
    }
  }
  canvas.onkeydown = (e) => {
    const keys = { ArrowLeft: [-1, 0], ArrowRight: [1, 0], ArrowUp: [0, -1], ArrowDown: [0, 1] },
      d = keys[e.key]
    if (d && orbital) {
      e.preventDefault()
      globe.rotate(d[0] * 0.15, d[1] * 0.15)
      draw()
      return
    }
    if (d) {
      e.preventDefault()
      autocenter = false
      map.center(map.x + (d[0] * map.span) / 8, map.z + (d[1] * map.span) / 8)
      draw()
    }
  }
  new ResizeObserver(draw).observe(canvas)
  refresh()
  focus()
  draw()
}
open().catch((err) => {
  $('status').textContent = err.message
  $('place').textContent = 'Field unavailable'
})
