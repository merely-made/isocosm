// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

// A pose is a published observation. Only an accepted walking edge connects two poses.
const mix = (a, b, f) => a.map((v, i) => v + (b[i] - v) * f)
const same = (a, b) => a.every((v, i) => v === b[i])
const placed = (frame, id) => frame?.poses?.find((p) => p.id === id)
const actions = (frame, id) => frame?.walks?.filter((w) => w.id === id && w.path.length > 1) || []

function within(w, portion, slowWalking) {
  const end = w.path.length - 1
  if (!w.steps?.length) return Math.min(end, portion * end)
  // An entire approach can finish in the first native substep. On the viewing
  // clock, expand that moving part of the minute so its stances are visible.
  const arrived = slowWalking ? w.steps.findIndex(([edge]) => edge >= end) : -1
  const time = portion * (arrived > 0 ? arrived : w.steps.length - 1)
  const lo = Math.floor(time),
    hi = Math.min(w.steps.length - 1, lo + 1)
  const elapsed = (i) => Math.min(end, w.steps[i][0] + w.steps[i][1] / 1000)
  return elapsed(lo) + (elapsed(hi) - elapsed(lo)) * (time - lo)
}

function sampled(w, position) {
  const i = Math.min(w.path.length - 1, Math.floor(position)),
    j = Math.min(w.path.length - 1, i + 1)
  const f = position - i
  const next = new Map(w.boxes[j].map((box) => [box[0], box]))
  const boxes = w.boxes[i].map(([id, lo, hi, role]) => {
    const b = next.get(id)
    return b ? [id, mix(lo, b[1], f), mix(hi, b[2], f), role] : [id, lo, hi, role]
  })
  return { at: mix(w.path[i].at, w.path[j].at, f), boxes, index: i }
}

export function readTrack(frames, tick, id, fraction = 0, slowWalking = false) {
  const tracks = []
  // Keep gaps distinct even when two unrelated observations fall within the same site.
  const add = (site, points) => {
    if (!points.length) return
    const last = tracks.at(-1)
    if (last?.site === site && same(last.points.at(-1), points[0]))
      last.points.push(...points.slice(1))
    else tracks.push({ site, points: points.slice() })
  }
  for (let i = 1; i <= tick; i++) {
    for (const w of actions(frames[i], id))
      add(
        w.site,
        w.path.map((p) => p.at),
      )
  }
  let resident = placed(frames[tick], id),
    target = null,
    exit = null
  if (fraction > 0 && resident) {
    const walks = actions(frames[tick + 1], id)
    const time = fraction * walks.length
    for (let i = 0; i < walks.length && i <= time; i++) {
      const w = walks[i]
      if (w.site !== resident.site) break
      const held = sampled(w, within(w, Math.min(1, time - i), slowWalking))
      add(w.site, [...w.path.slice(0, held.index + 1).map((p) => p.at), held.at])
      resident = { ...resident, pose: { at: held.at }, boxes: held.boxes }
      target = w.target
      exit = w.exit
    }
  }
  return { resident, tracks, target, exit }
}

function surface(terrain, site, x, z) {
  const data = terrain.sites[site].ground || terrain.sites[site]
  const n = data.samples || terrain.samples
  const u = Math.min(n, Math.max(0, (x * n) / terrain.side)),
    v = Math.min(n, Math.max(0, (z * n) / terrain.side))
  const row = n + 1,
    i = Math.min(n - 1, Math.floor(u)),
    j = Math.min(n - 1, Math.floor(v))
  const h = (a, b) => data.heights[b * row + a] / 16
  return (
    (1 - v + j) * ((1 - u + i) * h(i, j) + (u - i) * h(i + 1, j)) +
    (v - j) * ((1 - u + i) * h(i, j + 1) + (u - i) * h(i + 1, j + 1))
  )
}

function facet(ctx, points, color, edge = color) {
  ctx.beginPath()
  points.forEach(([x, y], i) => (i ? ctx.lineTo(x, y) : ctx.moveTo(x, y)))
  ctx.closePath()
  ctx.fillStyle = color
  ctx.fill()
  ctx.strokeStyle = edge
  ctx.lineWidth = 0.7
  ctx.stroke()
}
const paint = (rgb, light) =>
  'rgb(' + rgb.map((v) => Math.min(255, Math.round(v * light))).join(',') + ')'

export function drawGround(view, frame, track, animal) {
  const { ctx, w, h, terrain } = view,
    { resident, tracks } = track
  if (!resident) return
  const { site, pose } = resident,
    [cx, cy, cz] = pose.at
  const scale = Math.min(w / 98, h / 70),
    floor = terrain.water
  const project = ([x, y, z]) => [
    w * 0.5 + ((x - cx) * 0.87 - (z - cz) * 0.68) * scale,
    h * 0.56 + ((x - cx) * 0.31 + (z - cz) * 0.41 - (y - cy) * 0.86) * scale,
  ]
  const radius = Math.ceil(((w + h) / scale) * 0.65),
    stride = 4,
    side = terrain.side
  const x0 = Math.max(0, Math.floor((cx - radius) / stride) * stride),
    x1 = Math.min(side, cx + radius)
  const z0 = Math.max(0, Math.floor((cz - radius) / stride) * stride),
    z1 = Math.min(side, cz + radius)
  for (let z = z0; z < z1; z += stride)
    for (let x = x0; x < x1; x += stride) {
      const x2 = Math.min(x + stride, side),
        z2 = Math.min(z + stride, side)
      const xy = [
          [x, z],
          [x2, z],
          [x2, z2],
          [x, z2],
        ],
        heights = xy.map(([a, b]) => surface(terrain, site, a, b))
      const corners = xy.map(([a, b], i) => project([a, Math.max(heights[i], floor), b]))
      if (
        corners.every(([u, v]) => u < 0) ||
        corners.every(([u]) => u > w) ||
        corners.every(([, v]) => v < 0) ||
        corners.every(([, v]) => v > h)
      )
        continue
      const mean = heights.reduce((a, b) => a + b) / 4
      const rgb =
        mean < floor ? [57, 119, 127] : mean < floor + 3 ? [173, 163, 109] : [124, 150, 91]
      const contour = Math.floor(mean / 4) % 2 ? 0.98 : 1.02
      const light = contour * Math.max(0.65, Math.min(1.27, 1 + (heights[0] - heights[2]) * 0.055))
      facet(ctx, corners, paint(rgb, light))
    }
  ctx.strokeStyle = '#ffe9a2'
  ctx.lineWidth = 2.5
  for (const segment of tracks.filter((s) => s.site === site)) {
    ctx.beginPath()
    segment.points.forEach(([x, y, z], i) => {
      const p = project([x, surface(terrain, site, x, z) + 0.6, z])
      if (i) ctx.lineTo(...p)
      else ctx.moveTo(...p)
    })
    ctx.stroke()
  }
  const boxes = [],
    labels = [],
    contacts = []
  const sourceIds = new Set((animal?.sources || []).map(([, id]) => id))
  for (const other of frame.poses.filter((o) => o.site === site)) {
    const r = other.id === resident.id ? resident : other
    if (Math.hypot(r.pose.at[0] - cx, r.pose.at[2] - cz) > radius) continue
    const plant = r.kingdom === 'kingdom:flora'
    let topY = Infinity
    for (const [, lo, hi, role] of r.boxes) {
      const points = []
      for (const y of [lo[1], hi[1]])
        for (const z of [lo[2], hi[2]])
          for (const x of [lo[0], hi[0]]) points.push(project([x, y, z]))
      topY = Math.min(topY, ...points.map((p) => p[1]))
      const rgb = plant
        ? [79, 135, 78]
        : role === 'mouth'
          ? [217, 151, 84]
          : role === 'sense'
            ? [125, 83, 51]
            : r.id === resident.id
              ? [192, 162, 92]
              : [150, 141, 105]
      boxes.push({
        points,
        rgb,
        depth: (lo[0] + hi[0]) * 0.68 + (lo[2] + hi[2]) * 0.87 + (lo[1] + hi[1]) * 0.5,
      })
    }
    if (sourceIds.has(r.id)) {
      // Mark a touching pair of occupied part bounds on a tick with an actual transfer.
      for (const [, lower, upper] of r.boxes) {
        const touching = resident.boxes.find(([, lo, hi]) =>
          lower.every((v, k) => v <= hi[k] && lo[k] <= upper[k]),
        )
        if (touching) {
          contacts.push(
            project(
              lower.map(
                (v, k) => (Math.max(v, touching[1][k]) + Math.min(upper[k], touching[2][k])) / 2,
              ),
            ),
          )
          break
        }
      }
    }
    if (r.id === resident.id || plant || r.id === track.target) {
      const [x] = project(r.pose.at)
      const feeding = r.id === resident.id && animal?.fed ? ' · +' + animal.fed + ' food' : ''
      const label = (plant ? 'Plant #' : r.id === resident.id ? 'Grazer #' : '#') + r.id + feeding
      labels.push({
        x,
        y: topY - 9,
        label,
        active: r.id === resident.id || sourceIds.has(r.id),
        id: r.id,
      })
    }
  }
  for (const { points, rgb } of boxes.sort((a, b) => a.depth - b.depth)) {
    for (const [indices, light] of [
      [[1, 3, 7, 5], 0.67],
      [[2, 3, 7, 6], 0.83],
      [[4, 5, 7, 6], 1.12],
    ])
      facet(
        ctx,
        indices.map((i) => points[i]),
        paint(rgb, light),
        'rgba(50,60,33,.44)',
      )
  }
  for (const [x, y] of contacts) {
    ctx.beginPath()
    ctx.arc(x, y, 5, 0, 2 * Math.PI)
    ctx.strokeStyle = '#ffe7a3'
    ctx.lineWidth = 2
    ctx.stroke()
  }
  ctx.font = '12px ui-monospace, monospace'
  const used = []
  for (const l of labels.sort((a, b) => Number(b.active) - Number(a.active) || a.id - b.id)) {
    const width = ctx.measureText(l.label).width + 12
    if (l.x < 0 || l.x > w || l.y < 40 || l.y > h - 40) continue
    let y = l.y
    while (used.some((p) => Math.abs(p.y - y) < 22 && Math.abs(p.x - l.x) < (p.width + width) / 2))
      y -= 22
    if (y < 35) continue
    used.push({ x: l.x, y, width })
    ctx.fillStyle = l.active ? '#493f28' : '#294537'
    ctx.fillRect(l.x - width / 2, y - 15, width, 19)
    ctx.fillStyle = '#fff4ce'
    ctx.fillText(l.label, l.x - width / 2 + 6, y - 1)
  }
}

export function trackInset(view, track) {
  const last = track.tracks.at(-1)
  if (!last) return
  const site = last.site
  const at = track.resident?.site === site ? track.resident.pose.at : last.points.at(-1)
  const { ctx, w, h } = view,
    width = Math.min(w * 0.28, 152),
    height = width * 0.62
  const left = 16,
    top = h - height - 58
  const segments = track.tracks.filter((s) => s.site === site)
  if (!segments.length) return
  const pts = segments.flatMap((s) => s.points).concat([at])
  const xs = pts.map((p) => p[0]),
    zs = pts.map((p) => p[2])
  const xmin = Math.min(...xs),
    xmax = Math.max(...xs),
    zmin = Math.min(...zs),
    zmax = Math.max(...zs)
  const k = Math.min(
    (width - 18) / Math.max(4, xmax - xmin),
    (height - 26) / Math.max(4, zmax - zmin),
  )
  const project = ([x, y, z]) => [
    left + width / 2 + (x - (xmin + xmax) / 2) * k,
    top + height * 0.6 + (z - (zmin + zmax) / 2) * k,
  ]
  ctx.fillStyle = '#172f33ef'
  ctx.fillRect(left, top, width, height)
  ctx.fillStyle = '#dee5ce'
  ctx.font = '11px ui-monospace, monospace'
  ctx.fillText('Ground track · ' + site, left + 7, top + 14)
  ctx.strokeStyle = '#f4cd78'
  ctx.lineWidth = 2
  for (const s of segments) {
    ctx.beginPath()
    s.points.forEach((p, i) => (i ? ctx.lineTo(...project(p)) : ctx.moveTo(...project(p))))
    ctx.stroke()
  }
  ctx.beginPath()
  ctx.arc(...project(at), 3, 0, Math.PI * 2)
  ctx.fillStyle = '#fff1c4'
  ctx.fill()
}
