// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

import { soilBackdrop } from './soil-view.js'
import { readTrack } from './ground.js'

const faunaAccents = { mouth: '#b87444', sense: '#ece3af', limb: '#bfa267' }
const kinds = {
  'kingdom:flora': ['Plant', '#528347'],
  'kingdom:fauna': ['Grazer', '#d4aa5f'],
  'kingdom:myco': ['Fungus', '#a86d9b'],
  'kingdom:micro': ['Microbe', '#49afa7'],
}
export const kindOf = (key) => kinds[key] || ['Organism', '#b6a67d']
export function sizeOf(shape) {
  if (!shape.length) return []
  return [0, 2, 1].map(
    (axis) =>
      Math.max(...shape.map(([p, b]) => p[axis] + b[axis])) -
      Math.min(...shape.map(([p, b]) => p[axis] - b[axis])),
  )
}
export function measure(cells, unit) {
  const mm = (cells * unit) / 1000
  if (mm >= 1000) return Number((mm / 1000).toPrecision(3)) + ' m'
  if (mm >= 100) return Number((mm / 10).toPrecision(3)) + ' cm'
  return mm >= 1 ? Number(mm.toPrecision(3)) + ' mm' : Number((mm * 1000).toPrecision(3)) + ' µm'
}

// A single orthographic lens for the terrain and every part, including small organisms.
export class FieldMap {
  constructor(canvas, terrain) {
    this.canvas = canvas
    this.t = terrain
    this.x = this.z = terrain.side / 2
    this.span = terrain.side / 4
    this.hit = []
    this.textures = new Map()
  }
  center(x, z) {
    const clamp = (v) => Math.max(0, Math.min(this.t.side, v))
    this.x = clamp(x)
    this.z = clamp(z)
  }
  zoom(factor) {
    this.span = Math.max(16, Math.min(this.t.side * 1.45, this.span * factor))
  }
  fit() {
    this.span = this.t.side * Math.max(1, this.canvas.clientWidth / this.canvas.clientHeight) * 1.03
    this.center(this.t.side / 2, this.t.side / 2)
  }
  focus(poses, flows = []) {
    if (!poses.length) return this.center(this.t.side / 2, this.t.side / 2)
    const meal = flows.find(
      (flow) => flow.process.startsWith('ecology:feed') && poses.some((p) => p.id === flow.to),
    )
    if (meal) {
      const partners = poses.filter((p) => p.id === meal.to || p.id === meal.from)
      const mid = (axis) => partners.reduce((sum, p) => sum + p.pose.at[axis], 0) / partners.length
      return this.center(mid(0), mid(2))
    }
    const count = (p) =>
      poses.filter(
        (q) => Math.hypot(q.pose.at[0] - p.pose.at[0], q.pose.at[2] - p.pose.at[2]) < this.span / 3,
      ).length
    const anchor = poses.reduce((a, b) => (count(b) > count(a) ? b : a))
    this.center(anchor.pose.at[0], anchor.pose.at[2])
  }
  pick(x, z) {
    return this.hit.findLast((b) => x >= b.x && x <= b.x + b.w && z >= b.y && z <= b.y + b.h)?.id
  }
  texture(site) {
    if (this.textures.has(site)) return this.textures.get(site)
    const t = this.t,
      data = t.sites[site].ground || t.sites[site],
      n = data.samples || t.samples
    const img = document.createElement('canvas')
    img.width = img.height = 384
    const ctx = img.getContext('2d'),
      rgba = ctx.createImageData(img.width, img.height)
    const get = (i, j) => data.heights[j * (n + 1) + i] / 16
    for (let y = 0; y < img.height; y++)
      for (let x = 0; x < img.width; x++) {
        const u = ((x + 0.5) * n) / img.width,
          v = ((y + 0.5) * n) / img.height,
          i = Math.floor(u),
          j = Math.floor(v)
        const a = get(i, j),
          b = get(i + 1, j),
          c = get(i, j + 1),
          d = get(i + 1, j + 1)
        const h =
          (a * (1 - u + i) + b * (u - i)) * (1 - v + j) + (c * (1 - u + i) + d * (u - i)) * (v - j)
        const depth = h - t.water,
          land = depth >= 0
        const base = land ? [178, 183, 132] : [108, 157, 158],
          end = land ? [127, 151, 106] : [41, 88, 107]
        const q = Math.min(1, Math.abs(depth) / (land ? 55 : 42))
        const contour = Math.abs(h / 5 - Math.round(h / 5)) < 0.038 ? 0.92 : 1
        const light = contour * Math.max(0.76, Math.min(1.15, 1 + (((a - d) * n) / t.side) * 0.19))
        const k = (y * img.width + x) * 4
        for (let c = 0; c < 3; c++) rgba.data[k + c] = (base[c] + (end[c] - base[c]) * q) * light
        rgba.data[k + 3] = 255
      }
    ctx.putImageData(rgba, 0, 0)
    if (this.textures.size >= 8) this.textures.delete(this.textures.keys().next().value)
    this.textures.set(site, img)
    return img
  }
  draw(frame, site, selected, next, portion = 0) {
    const canvas = this.canvas,
      ctx = canvas.getContext('2d'),
      w = canvas.clientWidth,
      h = canvas.clientHeight
    const dpr = window.devicePixelRatio || 1
    if (canvas.width !== Math.round(w * dpr) || canvas.height !== Math.round(h * dpr)) {
      canvas.width = Math.round(w * dpr)
      canvas.height = Math.round(h * dpr)
    }
    ctx.setTransform(dpr, 0, 0, dpr, 0, 0)
    const scale = w / this.span,
      X = (v) => w / 2 + (v - this.x) * scale,
      Z = (v) => h / 2 + (v - this.z) * scale
    ctx.fillStyle = '#284139'
    ctx.fillRect(0, 0, w, h)
    ctx.drawImage(this.texture(site), X(0), Z(0), this.t.side * scale, this.t.side * scale)
    const soil = this.soilLayer && frame.sites[site].soil
    if (soil) soilBackdrop(ctx, soil, X, Z, scale, this.t.side)
    // Each square spans one native length unit; only draw the lattice when it resolves.
    if (scale >= 4) {
      ctx.save()
      ctx.beginPath()
      ctx.rect(X(0), Z(0), this.t.side * scale, this.t.side * scale)
      ctx.clip()
      ctx.strokeStyle = 'rgba(246,243,209,.17)'
      ctx.lineWidth = 0.65
      ctx.beginPath()
      for (let x = X(0) % scale; x < w; x += scale) {
        ctx.moveTo(x, 0)
        ctx.lineTo(x, h)
      }
      for (let z = Z(0) % scale; z < h; z += scale) {
        ctx.moveTo(0, z)
        ctx.lineTo(w, z)
      }
      ctx.stroke()
      ctx.restore()
    }
    const poses = frame.poses.filter(
      (p) => p.site === site && (soil ? p.soil || p.kingdom === 'kingdom:flora' : !p.soil),
    )
    const parts = [],
      labels = []
    const changes = new Map()
    for (const flow of frame.sites[site].flows) {
      for (const [id, amount] of [
        [flow.from, -flow.amount],
        [flow.to, flow.amount],
      ]) {
        if (id == null) continue
        if (!changes.has(id)) changes.set(id, [])
        const material = (amount > 0 ? flow.credit : flow.debit).split(':')
        const label = material[0] === 'matter' ? 'tissue' : material.at(-1)
        changes.get(id).push((amount > 0 ? '+' : '−') + Math.abs(amount) + ' ' + label)
      }
    }
    for (let p of poses) {
      if (next && portion > 0) p = readTrack([frame, next], 0, p.id, portion, true).resident || p
      const visible = soil && !p.soil ? p.boxes.filter(([id]) => id === 0) : p.boxes
      for (const [, lo, hi, role] of visible) parts.push({ lo, hi, role, p })
      if (!visible.length) continue
      const top = Math.min(...visible.map((b) => b[1][2]))
      labels.push({ x: X(p.pose.at[0]), y: Z(top) - 7, p })
    }
    ctx.lineWidth = 1.5
    ctx.strokeStyle = '#ffdfa1'
    for (const walk of (portion > 0 ? next?.walks : frame.walks) || []) {
      if (soil || walk.site !== site) continue
      const pts =
        portion > 0
          ? readTrack([frame, next], 0, walk.id, portion, true).tracks.flatMap((s) => s.points)
          : walk.path.map((p) => p.at)
      ctx.beginPath()
      pts.forEach(([x, , z], i) => (i ? ctx.lineTo(X(x), Z(z)) : ctx.moveTo(X(x), Z(z))))
      ctx.stroke()
    }
    this.hit = []
    for (const { p, lo, hi, role } of parts.sort(
      (a, b) => a.hi[1] - b.hi[1] || a.lo[1] - b.lo[1],
    )) {
      const x = X(lo[0]),
        y = Z(lo[2]),
        bw = (hi[0] - lo[0]) * scale,
        bh = (hi[2] - lo[2]) * scale
      if (x > w || y > h || x + bw < 0 || y + bh < 0) continue
      ctx.fillStyle =
        (p.soil && '#dcaecd') ||
        (p.kingdom === 'kingdom:fauna' && faunaAccents[role]) ||
        kindOf(p.kingdom)[1]
      ctx.fillRect(x, y, bw, bh)
      ctx.strokeStyle = p.id === selected ? '#fff0bd' : '#334a37'
      ctx.lineWidth = p.id === selected ? 1.4 : 0.7
      ctx.strokeRect(x, y, bw, bh)
      ctx.fillStyle = 'rgba(255,255,225,.22)'
      ctx.fillRect(x, y, bw, Math.min(bh, 1.5))
      this.hit.push({ x, y, w: bw, h: bh, id: p.id })
    }
    const used = []
    ctx.font = '11px ui-monospace, monospace'
    for (const { x, y, p } of labels.sort(
      (a, b) => Number(b.p.id === selected) - Number(a.p.id === selected),
    )) {
      const change = changes.get(p.id)?.[0]
      const name = kindOf(p.kingdom)[0] + ' #' + p.id + (change ? ' · ' + change : ''),
        width = ctx.measureText(name).width + 10
      if (x < 4 || x > w - 4 || y < 38 || y > h - 35) continue
      let labelY = y
      while (
        used.some((a) => Math.abs(labelY - a.y) < 17 && Math.abs(x - a.x) < (a.width + width) / 2)
      )
        labelY -= 17
      if (labelY < 35) continue
      used.push({ x, y: labelY, width })
      ctx.fillStyle = p.id === selected || changes.has(p.id) ? '#5c4829' : '#223c30df'
      ctx.fillRect(x - width / 2, labelY - 12, width, 16)
      ctx.fillStyle = '#fff6d7'
      ctx.fillText(name, x - width / 2 + 5, labelY)
    }
    const maxCells = Math.min(100, w * 0.22) / scale,
      maxMetres = (maxCells * this.t.unitMicrometres) / 1e6
    const pow = 10 ** Math.floor(Math.log10(maxMetres)),
      metres = [5, 2, 1].find((x) => x * pow <= maxMetres) * pow
    const cells = (metres * 1e6) / this.t.unitMicrometres,
      len = cells * scale
    ctx.fillStyle = '#17372de6'
    const left = w - len - 28,
      top = 46
    ctx.fillRect(left - 7, top, len + 14, 26)
    ctx.strokeStyle = '#f6edcd'
    ctx.lineWidth = 1.5
    ctx.beginPath()
    ctx.moveTo(left, top + 16)
    ctx.lineTo(left, top + 20)
    ctx.lineTo(left + len, top + 20)
    ctx.lineTo(left + len, top + 16)
    ctx.stroke()
    ctx.fillStyle = '#fff3cc'
    ctx.fillText(measure(cells, this.t.unitMicrometres), left, top + 12)
  }
}
