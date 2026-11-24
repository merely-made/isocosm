// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

// A vertical slice through the selected collar. Both axes use the field's
// physical scale. Only body boxes crossing the slice appear in section.
export function sectionColony(frame, site, selected, z) {
  const soil = frame.sites[site].soil
  const local = frame.poses.filter((p) => p.soil && p.site === site)
  const root = soil.roots.find((r) => r.id === selected)
  if (root) {
    return local.find((colony) => colony.boxes.some(([, lo, hi]) => {
      const overlap = lo.map((v, i) => Math.min(hi[i], root.bounds[1][i]) - Math.max(v, root.bounds[0][i]))
      return overlap.every((v) => v >= 0) && overlap.filter((v) => v > 0).length >= 2
    }))
  }
  return local.find((p) => p.id === selected) ||
    local.sort((a, b) => Math.abs(a.pose.at[2] - z) - Math.abs(b.pose.at[2] - z))[0]
}
export function drawSection(canvas, terrain, site, frame, map, selected) {
  const soil = frame.sites[site].soil
  const focus = frame.poses.find((p) => p.site === site && p.id === selected && p.kingdom === 'kingdom:flora')
    || sectionColony(frame, site, selected, map.z)
  if (!focus) {
    canvas.getContext('2d').clearRect(0, 0, canvas.width, canvas.height)
    return
  }
  const [cx, cy, z] = focus.pose.at,
    w = canvas.clientWidth,
    h = canvas.clientHeight,
    dpr = window.devicePixelRatio || 1
  canvas.width = Math.round(w * dpr)
  canvas.height = Math.round(h * dpr)
  const ctx = canvas.getContext('2d')
  ctx.setTransform(dpr, 0, 0, dpr, 0, 0)
  const k = w / map.span,
    X = (x) => w * 0.48 + (x - cx) * k,
    Y = (y) => h * 0.58 - (y - cy) * k
  ctx.fillStyle = '#d7dfc6'
  ctx.fillRect(0, 0, w, h)
  const data = terrain.sites[site].ground || terrain.sites[site],
    n = data.samples || terrain.samples
  const height = (x) => {
    const u = Math.max(0, Math.min(n - 1e-6, (x * n) / terrain.side)),
      v = Math.max(0, Math.min(n - 1e-6, (z * n) / terrain.side))
    const i = Math.floor(u),
      j = Math.floor(v),
      a = u - i,
      b = v - j
    const at = (i, j) => data.heights[j * (n + 1) + i] / 16
    return (
      (1 - b) * ((1 - a) * at(i, j) + a * at(i + 1, j)) +
      b * ((1 - a) * at(i, j + 1) + a * at(i + 1, j + 1))
    )
  }
  ctx.beginPath()
  ctx.moveTo(0, h)
  for (let u = 0; u <= w + 2; u += 2) {
    ctx.lineTo(u, Y(height(cx + (u - w * 0.48) / k)))
  }
  ctx.lineTo(w, h)
  ctx.closePath()
  ctx.fillStyle = '#614c3b'
  ctx.fill()
  const box = (bounds, color) => {
    const [a, b] = bounds
    if (a[2] > z || b[2] < z) return
    ctx.fillStyle = color
    ctx.fillRect(X(a[0]), Y(b[1]), (b[0] - a[0]) * k, (b[1] - a[1]) * k)
    ctx.strokeStyle = '#352d28'
    ctx.lineWidth = 0.8
    ctx.strokeRect(X(a[0]), Y(b[1]), (b[0] - a[0]) * k, (b[1] - a[1]) * k)
  }
  box(soil.litter, soil.remaining ? '#e3af68' : '#87735c')
  for (const p of frame.poses.filter(
    (p) => p.site === site && (p.soil || p.kingdom === 'kingdom:flora'),
  ))
    for (const [, a, b] of p.boxes) box([a, b], p.soil ? '#deb5d3' : '#759c60')
  ctx.fillStyle = '#f9f5e8'
  ctx.font = '12px ui-monospace, monospace'
  const caption = (focus.soil ? 'Fungus #' : 'Root #') + focus.id + ' · section at z ' + z
  ctx.fillStyle = '#453a32e8'
  ctx.fillRect(5, h - 26, ctx.measureText(caption).width + 12, 23)
  ctx.fillStyle = '#fff3de'
  ctx.fillText(caption, 11, h - 10)
}
export function soilBackdrop(ctx, soil, X, Z, k, side) {
  ctx.fillStyle = 'rgba(60,39,32,.74)'
  ctx.fillRect(X(0), Z(0), side * k, side * k)
  const [lo, hi] = soil.litter
  ctx.fillStyle = soil.remaining ? '#e7b76b' : '#86735e'
  ctx.fillRect(X(lo[0]), Z(lo[2]), (hi[0] - lo[0]) * k, (hi[2] - lo[2]) * k)
  ctx.lineWidth = 1
  ctx.strokeStyle = '#ffdc9d'
  ctx.strokeRect(X(lo[0]), Z(lo[2]), (hi[0] - lo[0]) * k, (hi[2] - lo[2]) * k)
  ctx.font = '12px ui-monospace,monospace'
  ctx.fillStyle = '#fff0c2'
  ctx.fillText('Litter · ' + soil.remaining + ' P', X(lo[0]), Z(hi[2]) + 17)
}
