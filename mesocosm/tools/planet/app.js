// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

import { PlanetView, locationText } from './globe.js'

const $ = (id) => document.getElementById(id)
const quantity = (n, singular, plural = singular + 's') => n + ' ' + (n === 1 ? singular : plural)
async function read(url) {
  const response = await fetch(url)
  if (!response.ok) throw new Error('The world could not be read (' + response.status + ').')
  return response.json()
}

async function openPlanet() {
  const [terrain, frames] = await Promise.all([read('terrain.json'), read('frames.json')])
  const ground = frames[0].branch ? await import('./ground.js') : null
  let fraction = 0,
    animation = 0,
    lastPaint = 0
  document.title = 'Planet ' + terrain.seed + ' · Mesocosm'
  document.querySelector('a[download]').download = 'planet-' + terrain.seed + '-founding.json'
  const canvas = $('scene'),
    view = new PlanetView(canvas, terrain)
  let level = 'orbit',
    tick = 0,
    watched = null,
    site = 0,
    playing = false,
    busy = false,
    timer = null,
    pending = false
  const valleyDepth = terrain.sites.map(({ elevation, neighbors }) => {
    const [n, e, s, w] = neighbors.map(([id]) => terrain.sites[id].elevation - elevation)
    return Math.max(Math.min(n, s), Math.min(e, w))
  })
  const name = (id) => (valleyDepth[id] > 0 ? 'Valley ' : 'Site ') + id
  const critter = (id) => 'Grazer #' + id
  const frame = () => frames[tick]
  const find = (f, id) => f.animals.find((a) => a.id === id)
  const anchored = (a) => a?.site ?? a?.journey?.from
  const residents = (id) => frame().animals.filter((a) => a.site === id)
  function valleys() {
    return terrain.sites
      .map((s, i) => i)
      .filter((i) => valleyDepth[i] > 0 && frame().counts[i][1] > 0)
      .sort((a, b) => {
        const rank = (i) =>
          valleyDepth[i] +
          (terrain.sites[i].elevation >= terrain.water ? 90 : 0) +
          (frame().counts[i][0] > 0 ? 35 : 0)
        return rank(b) - rank(a) || a - b
      })
      .slice(0, 4)
  }
  site = valleys()[0] ?? frame().animals.find((a) => a.site != null)?.site ?? 0
  if (ground) {
    watched = frames[0].observer
    site = anchored(find(frames[0], watched)) ?? site
    const params = new URLSearchParams(location.search)
    const requested = Number(params.get('tick'))
    if (Number.isInteger(requested) && requested >= 0 && requested < frames.length) tick = requested
    level = params.get('view') === 'globe' ? 'orbit' : 'animal'
    document.querySelector('.series').textContent = 'Mesocosm · Spatial branch'
  }
  const currentTrack = () =>
    ground && watched !== null ? ground.readTrack(frames, tick, watched, fraction, true) : null
  view.center(view.vector(site))
  view.rotate(0.35, 0.16)
  if (ground && level === 'orbit') view.center(marker())

  function watchState() {
    if (watched === null) return {}
    const a = find(frame(), watched)
    if (a) return { a, last: a, lastTick: tick }
    for (let i = tick - 1; i >= 0; i--) {
      const last = find(frames[i], watched)
      if (last) return { last, lastTick: i }
    }
    return {}
  }
  function markerFor(a, atTick) {
    if (!a?.journey) return view.vector(a?.site ?? site)
    const j = a.journey,
      mix = Math.max(0, Math.min(1, (atTick - j.departed) / Math.max(1, j.arrives - j.departed)))
    const from = view.vector(j.from),
      to = view.vector(j.to)
    const mid = from.map((x, i) => x * (1 - mix) + to[i] * mix)
    return mid.map((x) => x / Math.hypot(...mid))
  }
  function marker() {
    const r = currentTrack()?.resident
    if (r) return view.vector(r.site, r.pose.at[0] / terrain.side, r.pose.at[2] / terrain.side)
    const { last, lastTick } = watchState()
    if (ground && last?.journey?.departed === tick) {
      const exit = frame().walks?.find((w) => w.id === watched && w.exit === last.journey.to)
      if (exit) {
        const [x, , z] = exit.path.at(-1).at
        return view.vector(exit.site, x / terrain.side, z / terrain.side)
      }
    }
    return last ? markerFor(last, lastTick) : view.vector(site)
  }
  function paint() {
    pending = false
    const { a, last } = watchState()
    const groundTrack = currentTrack()
    const label =
      watched === null
        ? name(site)
        : groundTrack && !groundTrack.resident && a?.journey
          ? name(a.journey.from) + ' → ' + name(a.journey.to)
          : (a ? '' : 'Last seen · ') + critter(watched)
    const trail = []
    if (!ground && level === 'orbit' && watched !== null) {
      for (let i = Math.max(0, tick - 24); i <= tick; i++) {
        const at = anchored(find(frames[i], watched))
        if (at != null && at !== trail.at(-1)) trail.push(at)
      }
    }
    const crossing = level === 'animal' && groundTrack && !groundTrack.resident && a?.journey
    if (crossing) view.center(marker())
    if (level === 'animal' && groundTrack?.resident) {
      view.size()
      view.backdrop()
      ground.drawGround(view, frame(), groundTrack, a)
      const pos = $('ground-position')
      if (pos)
        pos.textContent =
          'Ground position · ' +
          groundTrack.resident.pose.at.map((n) => Number(n.toFixed(1))).join(', ')
      if (fraction > 0 && groundTrack.exit != null)
        $('gesture').textContent = 'Walking toward ' + name(groundTrack.exit)
      else if (fraction > 0 && groundTrack.target != null)
        $('gesture').textContent = 'Walking toward #' + groundTrack.target
    } else
      view.draw({
        level: crossing ? 'orbit' : level,
        site,
        label,
        trail,
        groundTrack,
        animal: a,
        marker: !last && watched !== null ? null : marker(),
        sites: level === 'orbit' ? valleys() : [],
        body: a ? frame().bodies[a.body] : null,
      })
    if ((level === 'orbit' || crossing) && groundTrack) ground.trackInset(view, groundTrack)
    $('coords').textContent = locationText(level === 'orbit' ? view.basis()[2] : marker())
  }
  function requestPaint() {
    if (!pending) {
      pending = true
      requestAnimationFrame(paint)
    }
  }

  function choose(text, detail, onSelect, selected = false) {
    const button = document.createElement('button')
    button.className = 'choice' + (selected ? ' selected' : '')
    for (const value of [text, detail]) {
      const span = document.createElement('span')
      span.textContent = value
      button.append(span)
    }
    button.onclick = onSelect
    $('list').append(button)
  }
  function note(text, detail) {
    const p = document.createElement('p')
    p.className = detail ? 'detail' : ''
    p.textContent = text
    $('events').append(p)
    return p
  }
  function history() {
    const graph = $('history'),
      h = 36,
      w = graph.clientWidth
    graph.width = w * 2
    graph.height = h * 2
    const c = graph.getContext('2d')
    c.scale(2, 2)
    const start = Math.max(0, tick - 23),
      series = frames.slice(start, tick + 1).map((f) => find(f, watched))
    const high = Math.max(1, ...series.map((a) => a?.tissue ?? 0))
    const dx = w / 24
    series.forEach((a, i) => {
      if (!a) return
      const height = (a.tissue / high) * (h - 6)
      c.fillStyle = a.fed ? '#aa7229' : '#739279'
      c.fillRect(i * dx, h - height, Math.max(1, dx - 1), height)
    })
    graph.setAttribute(
      'aria-label',
      'Tissue stores from tick ' + start + ' to ' + tick + '. Gold marks food intake.',
    )
    graph.title = graph.getAttribute('aria-label')
  }
  function observations(a, lastTick) {
    history()
    $('events').replaceChildren()
    if (!a) {
      note(
        lastTick === undefined
          ? 'This individual has no observation by this tick.'
          : 'Last observed alive at tick ' + lastTick + '.',
      )
      return
    }
    note(
      a.journey
        ? 'Travelling from ' + name(a.journey.from) + ' to ' + name(a.journey.to) + '.'
        : 'Resident in ' + name(a.site) + '.',
    )
    note('Tissue held · ' + a.tissue + ' units', true)
    let shown = 0
    for (let i = tick; i > 0 && shown < 3; i--) {
      const current = find(frames[i], watched)
      if (!current) continue
      const prev = find(frames[i - 1], watched)
      const events = []
      if (current.fed) {
        const kinds = {
          'kingdom:flora': 'plant',
          'kingdom:fauna': 'animal',
          'kingdom:myco': 'fungus',
          'kingdom:micro': 'microbe',
        }
        const sources = (current.sources || []).map(
          ([kind, id]) => (kinds[kind] || 'organism') + ' #' + id,
        )
        events.push(
          'took in ' +
            quantity(current.fed, 'unit') +
            ' of food' +
            (sources.length ? ' from ' + sources.join(', ') : '') +
            (ground && !frames[i].poses.some((p) => p.id === watched) ? ' (site observation)' : ''),
        )
      }
      if (current.journey && current.journey.departed === i)
        events.push('set out for ' + name(current.journey.to))
      if (current.site != null && prev && current.site !== prev.site)
        events.push('arrived at ' + name(current.site))
      if (!prev && current.born === i) events.push('born')
      if (events.length) {
        note('Tick ' + i + ' · ' + events.join(' · '))
        shown++
      }
    }
    if (ground) {
      const r = frame().poses.find((r) => r.id === watched)
      for (const w of frame().walks || []) {
        if (w.id !== watched) continue
        const goal =
          w.exit != null
            ? ' toward ' + name(w.exit)
            : w.target == null
              ? ''
              : ' toward #' + w.target
        const contact = w.contacts?.length
          ? ' · reached ' + w.contacts.map((id) => '#' + id).join(', ')
          : ''
        note('Walked ' + quantity(w.path.length - 1, 'ground step') + goal + contact + '.')
      }
      if (r) note('Ground position · ' + r.pose.at.join(', '), true).id = 'ground-position'
      else
        note(
          a.journey
            ? 'The sim records this crossing at site resolution.'
            : 'No ground pose at this tick.',
          true,
        )
    }
    if (!shown) note(a.born === 0 ? 'Present at founding.' : 'Born at tick ' + a.born + '.', true)
  }

  function refresh() {
    const { a, last, lastTick } = watchState()
    if (last) site = anchored(last) ?? site
    $('field-link').href = 'field?site=' + site
    $('seed').textContent = terrain.seed
    $('tick').textContent = 'Tick ' + tick
    $('scrub').value = tick
    $('scrub').max = frames.length - 1
    $('orbit').setAttribute('aria-current', level === 'orbit')
    $('valley').setAttribute('aria-current', level === 'valley')
    $('animal').setAttribute('aria-current', level === 'animal')
    $('valley').disabled = false
    $('animal').disabled = watched === null && !residents(site).length
    $('out').disabled = level === 'orbit'
    $('in').disabled = level === 'animal' || (level === 'valley' && $('animal').disabled)
    $('step').disabled = busy
    $('play').textContent = playing ? 'Pause' : 'Play'
    const onGround = Boolean(ground && frame().poses.some((r) => r.id === watched))
    $('mode').textContent = {
      orbit: 'Orbital view',
      valley: 'Terrain survey',
      animal: onGround
        ? 'Ground follow'
        : ground && a?.journey
          ? 'Border crossing'
          : 'Individual view',
    }[level]
    $('gesture').textContent = {
      orbit: 'Drag to spin · select a site to descend',
      valley: 'Gold outline · selected site and its neighbors',
      animal: onGround
        ? 'Ground follow · walking slowed for observation'
        : ground && a?.journey
          ? 'Site-scale crossing · ground follow resumes at the next recorded pose'
          : 'Body enlarged · position resolved to site or route',
    }[level]
    canvas.setAttribute('aria-label', $('mode').textContent + '. ' + $('gesture').textContent)
    $('list').replaceChildren()
    $('choices').hidden = level === 'animal'
    $('observe').hidden = level !== 'animal'
    if (level === 'orbit') {
      $('context').textContent = 'World atlas'
      $('title').textContent = 'The whole planet'
      const total = frame()
        .counts.flat()
        .reduce((sum, n) => sum + n, 0)
      $('description').textContent =
        terrain.sites.length + ' connected sites. ' + total + ' residents across land and water.'
      $('choicesTitle').textContent = 'Inhabited valleys'
      for (const i of valleys())
        choose(name(i), quantity(frame().counts[i][1], 'grazer'), () => openSite(i), site === i)
      if (!$('list').children.length)
        $('list').textContent = 'No valley has a resident grazer at this tick.'
    } else if (level === 'valley') {
      $('context').textContent = 'Planet ' + terrain.seed + ' / Local terrain'
      $('title').textContent = name(site)
      const [flora, fauna, myco, micro] = frame().counts[site]
      $('description').textContent = [
        quantity(flora, 'plant'),
        quantity(fauna, 'grazer'),
        quantity(myco, 'fungus', 'fungi'),
        quantity(micro, 'microbe'),
      ].join(' · ')
      $('choicesTitle').textContent = 'Resident grazers'
      for (const r of residents(site))
        choose(critter(r.id), 'Follow →', () => watch(r.id), watched === r.id)
      if (!$('list').children.length)
        $('list').textContent =
          'No resident grazers at this tick. Advance time or pick another site.'
    } else {
      $('context').textContent = 'Planet ' + terrain.seed + ' / ' + name(site)
      $('title').textContent = critter(watched)
      $('description').textContent = a
        ? 'Lineage ' +
          a.lineage.split(':').pop() +
          ' · ' +
          frame().bodies[a.body].length +
          ' living body parts'
        : 'Individual record'
      observations(a, lastTick)
    }
    $('position').textContent =
      watched === null
        ? 'Selected · ' + name(site)
        : a?.journey
          ? critter(watched) + ' · ' + name(a.journey.from) + ' → ' + name(a.journey.to)
          : a
            ? critter(watched) + ' · ' + name(a.site)
            : last
              ? 'Last seen · ' + name(site) + ' · tick ' + lastTick
              : 'Awaiting first observation'
    requestPaint()
  }

  function setLevel(to) {
    const from = level
    const location = marker()
    level = to
    const snapshot =
      from !== to && !matchMedia('(prefers-reduced-motion: reduce)').matches
        ? document.createElement('canvas')
        : null
    if (snapshot) {
      canvas.parentNode.querySelector('.scale-transition')?.remove()
      snapshot.className = 'scale-transition'
      snapshot.setAttribute('aria-hidden', 'true')
      snapshot.width = canvas.width
      snapshot.height = canvas.height
      snapshot.getContext('2d').drawImage(canvas, 0, 0)
      if (from === 'orbit') {
        const [x, y] = view.screenPoint(location)
        snapshot.style.transformOrigin = x + 'px ' + y + 'px'
      }
      canvas.after(snapshot)
    }
    if (to === 'orbit') view.center(location)
    refresh()
    if (snapshot) {
      const inward = from === 'orbit' || to === 'animal'
      const motion = snapshot.animate(
        [
          { opacity: 1, transform: 'scale(1)' },
          { opacity: 0, transform: inward ? 'scale(2.6)' : 'scale(.36)' },
        ],
        { duration: 350, easing: 'ease-out', fill: 'forwards' },
      )
      motion.onfinish = () => snapshot.remove()
    }
  }
  function openSite(id) {
    if (id !== site) watched = null
    site = id
    setLevel('valley')
  }
  function watch(id) {
    const resume = !ground || id !== watched
    watched = id
    setLevel('animal')
    if (!playing && resume) play()
  }
  function closer() {
    if (level === 'orbit') {
      setLevel('valley')
      return
    }
    if (level === 'valley') {
      const id = watched ?? residents(site)[0]?.id
      if (id != null) watch(id)
    }
  }
  function farther() {
    if (level === 'animal') setLevel('valley')
    else if (level === 'valley') setLevel('orbit')
  }
  function stop() {
    playing = false
    clearTimeout(timer)
    cancelAnimationFrame(animation)
    $('play').textContent = 'Play'
  }
  let revision = 0
  async function advance() {
    if (busy) return
    const requested = tick + 1,
      version = revision
    busy = true
    $('step').disabled = true
    try {
      if (requested === frames.length) {
        $('status').textContent = 'Advancing the live world…'
        frames.push(await read('frame?tick=' + requested))
        $('status').textContent = ''
      }
      if (version === revision) {
        tick = requested
        fraction = 0
      }
    } catch (err) {
      stop()
      $('status').textContent = err.message
    } finally {
      busy = false
      refresh()
    }
  }
  function schedule() {
    clearTimeout(timer)
    if (ground) {
      const currentRevision = revision
      const animate = async (now) => {
        if (!playing || currentRevision !== revision) return
        if (tick + 1 === frames.length) {
          if (busy) {
            animation = requestAnimationFrame(animate)
            return
          }
          busy = true
          try {
            $('status').textContent = 'Advancing the live world…'
            frames.push(await read('frame?tick=' + (tick + 1)))
            $('status').textContent = ''
          } catch (e) {
            stop()
            $('status').textContent = e.message
            return
          } finally {
            busy = false
          }
          lastPaint = performance.now()
          if (!playing || currentRevision !== revision) return
        }
        fraction += Math.min(125, Math.max(0, now - lastPaint)) / 1300
        lastPaint = now
        if (fraction >= 1) {
          tick++
          fraction = 0
          refresh()
        } else if (
          (level === 'animal' || level === 'orbit') &&
          frames[tick + 1]?.walks?.some((walk) => walk.id === watched)
        )
          requestPaint()
        animation = requestAnimationFrame(animate)
      }
      lastPaint = performance.now()
      cancelAnimationFrame(animation)
      animation = requestAnimationFrame(animate)
      return
    }
    timer = setTimeout(async () => {
      if (!playing) return
      await advance()
      if (playing) schedule()
    }, 1100)
  }
  function play() {
    playing = true
    $('play').textContent = 'Pause'
    schedule()
  }

  $('play').onclick = () => (playing ? stop() : play())
  $('step').onclick = () => {
    stop()
    advance()
  }
  $('scrub').oninput = (e) => {
    stop()
    revision++
    fraction = 0
    tick = Number(e.target.value)
    refresh()
  }
  $('orbit').onclick = () => setLevel('orbit')
  $('valley').onclick = () => setLevel('valley')
  $('animal').onclick = () => {
    const id = watched ?? residents(site)[0]?.id
    if (id != null) watch(id)
  }
  $('in').onclick = closer
  $('out').onclick = farther
  let pointer = null,
    moved = false,
    lastZoom = 0
  canvas.onpointerdown = (e) => {
    if (!e.isPrimary || level !== 'orbit') return
    pointer = { x: e.clientX, y: e.clientY, startX: e.clientX, startY: e.clientY, id: e.pointerId }
    moved = false
    canvas.setPointerCapture(e.pointerId)
  }
  canvas.onpointermove = (e) => {
    if (!pointer || e.pointerId !== pointer.id) return
    if (Math.hypot(e.clientX - pointer.startX, e.clientY - pointer.startY) > 4) moved = true
    if (moved) {
      view.dragging = true
      view.rotate((e.clientX - pointer.x) / view.radius, (e.clientY - pointer.y) / view.radius)
      requestPaint()
    }
    pointer.x = e.clientX
    pointer.y = e.clientY
  }
  canvas.onpointerup = (e) => {
    if (!pointer || e.pointerId !== pointer.id) return
    pointer = null
    view.dragging = false
    requestPaint()
    if (!moved && level === 'orbit') {
      const box = canvas.getBoundingClientRect()
      const id = view.siteAt(e.clientX - box.left, e.clientY - box.top)
      if (id != null) openSite(id)
    }
  }
  canvas.onpointercancel = () => {
    view.dragging = false
    requestPaint()
    pointer = null
  }
  canvas.addEventListener(
    'wheel',
    (e) => {
      if (Math.abs(e.deltaY) < 3) return
      e.preventDefault()
      const now = performance.now(),
        elapsed = now - lastZoom
      lastZoom = now
      if (elapsed <= 220) return
      if (e.deltaY < 0 && level === 'orbit') {
        const box = canvas.getBoundingClientRect(),
          at = view.siteAt(e.clientX - box.left, e.clientY - box.top)
        if (at != null) openSite(at)
      } else e.deltaY < 0 ? closer() : farther()
    },
    { passive: false },
  )
  canvas.onkeydown = (e) => {
    if (e.key === 'Escape' || e.key === '-') farther()
    else if (e.key === 'Enter' || e.key === '+') closer()
    else if (level === 'orbit' && e.key.startsWith('Arrow')) {
      const delta = {
        ArrowLeft: [-0.2, 0],
        ArrowRight: [0.2, 0],
        ArrowUp: [0, 0.2],
        ArrowDown: [0, -0.2],
      }[e.key]
      if (delta) view.rotate(...delta)
      requestPaint()
    } else return
    e.preventDefault()
  }
  new ResizeObserver(requestPaint).observe(canvas)
  refresh()
}
openPlanet().catch((err) => {
  $('title').textContent = 'Planet unavailable'
  $('status').textContent = err.message
})
