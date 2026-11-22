// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

const TAU = 2 * Math.PI
const clamp = (v, lo, hi) => Math.max(lo, Math.min(hi, v))
const dot = (a, b) => a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
const unit = (v) => {
  const length = Math.hypot(...v)
  return v.map((x) => x / length)
}
const ink = '#d9e5c7'
const gold = '#f6cd79'

export function locationText(v) {
  const lat = (Math.asin(v[1]) * 180) / Math.PI
  const lon = (Math.atan2(v[0], v[2]) * 180) / Math.PI
  return (
    Math.abs(lat).toFixed(1) +
    '° ' +
    (lat < 0 ? 'S' : 'N') +
    '   ' +
    Math.abs(lon).toFixed(1) +
    '° ' +
    (lon < 0 ? 'W' : 'E')
  )
}

function polygon(ctx, points, fill, stroke = fill) {
  ctx.beginPath()
  points.forEach(([x, y], i) => (i ? ctx.lineTo(x, y) : ctx.moveTo(x, y)))
  ctx.closePath()
  ctx.fillStyle = fill
  ctx.fill()
  ctx.lineWidth = 0.65
  ctx.strokeStyle = stroke
  ctx.stroke()
}

const stops = [
  [-110, [15, 55, 80]],
  [-45, [25, 85, 112]],
  [-18, [45, 125, 144]],
  [-1, [109, 176, 165]],
  [0, [216, 204, 146]],
  [4, [173, 184, 125]],
  [18, [104, 149, 100]],
  [37, [129, 151, 91]],
  [57, [177, 163, 109]],
  [80, [199, 185, 151]],
  [109, [225, 224, 203]],
]
function terrainColor(height, water, slope) {
  const elevation = height - water
  let hi = 1
  while (hi < stops.length - 1 && elevation > stops[hi][0]) hi++
  const [loH, lo] = stops[hi - 1],
    [hiH, high] = stops[hi]
  const t = clamp((elevation - loH) / (hiH - loH), 0, 1)
  const contour = ((height % 12) + 12) % 12 < 1 ? 0.93 : 1
  const light = clamp(1 + slope * (elevation < 0 ? 0.019 : 0.035), 0.71, 1.23) * contour
  return lo.map((v, i) => clamp((v + (high[i] - v) * t) * light, 0, 255))
}
const css = (rgb) => 'rgb(' + rgb.map(Math.round).join(',') + ')'

export class PlanetView {
  constructor(canvas, terrain) {
    this.canvas = canvas
    this.ctx = canvas.getContext('2d')
    this.terrain = terrain
    this.yaw = 0.4
    this.pitch = 0.23
    this.raster = document.createElement('canvas')
    this.textureSize = terrain.n * terrain.samples
    this.textures = this.buildTextures()
  }

  siteIndex(face, x, y) {
    const n = this.terrain.n
    return (Math.floor(face / 3) * n + y) * 3 * n + (face % 3) * n + x
  }

  vector(site, fx = 0.5, fy = 0.5) {
    const { n, axes } = this.terrain
    const col = site % (3 * n),
      row = Math.floor(site / (3 * n))
    const face = Math.floor(row / n) * 3 + Math.floor(col / n)
    const u = (2 * ((col % n) + fx)) / n - 1,
      v = (2 * ((row % n) + fy)) / n - 1
    const [normal, right, down] = axes[face]
    return unit(normal.map((a, i) => a + u * right[i] + v * down[i]))
  }

  buildTextures() {
    const { samples: k, water, sites, n } = this.terrain,
      s = this.textureSize
    return this.terrain.axes.map((axes, face) => {
      const pixels = new Uint8ClampedArray(s * s * 3)
      for (let y = 0; y < s; y++)
        for (let x = 0; x < s; x++) {
          const site = this.siteIndex(face, Math.floor(x / k), Math.floor(y / k))
          const h = sites[site].heights
          const i = (y % k) * (k + 1) + (x % k)
          const dx = (h[i + 1] - h[i]) / 16,
            dy = (h[i + k + 1] - h[i]) / 16
          const slope = dx * (axes[1][0] - axes[1][1]) + dy * (axes[2][0] - axes[2][1])
          pixels.set(
            terrainColor((h[i] + h[i + 1] + h[i + k + 1] + h[i + k + 2]) / 64, water, slope),
            (y * s + x) * 3,
          )
        }
      return pixels
    })
  }

  center(v) {
    this.yaw = Math.atan2(v[0], v[2])
    this.pitch = Math.asin(v[1])
  }

  rotate(dx, dy) {
    this.yaw -= dx
    this.pitch = clamp(this.pitch + dy, -Math.PI / 2, Math.PI / 2)
  }

  basis() {
    const a = Math.sin(this.yaw),
      b = Math.cos(this.yaw)
    const c = Math.cos(this.pitch),
      d = Math.sin(this.pitch)
    return [
      [b, 0, -a],
      [-a * d, c, -b * d],
      [a * c, d, b * c],
    ]
  }

  screenPoint(dir) {
    const [right, up] = this.basis()
    return [this.cx + this.radius * dot(dir, right), this.cy - this.radius * dot(dir, up)]
  }

  siteAt(px, py) {
    const u = (px - this.cx) / this.radius,
      v = (this.cy - py) / this.radius,
      z2 = 1 - u * u - v * v
    if (z2 < 0) return null
    const [right, up, front] = this.basis()
    const dir = right.map((r, i) => u * r + v * up[i] + Math.sqrt(z2) * front[i])
    const { axes, n } = this.terrain
    const powers = axes.map(([normal]) => dot(dir, normal))
    const d = Math.max(...powers),
      f = powers.indexOf(d)
    const cell = (axis) => clamp(Math.floor(((dot(dir, axis) / d + 1) * n) / 2), 0, n - 1)
    return this.siteIndex(f, cell(axes[f][1]), cell(axes[f][2]))
  }

  size() {
    const w = Math.round(this.canvas.clientWidth),
      h = Math.round(this.canvas.clientHeight)
    const scale = Math.min(1.5, window.devicePixelRatio || 1)
    const width = Math.round(w * scale),
      height = Math.round(h * scale)
    if (width !== this.canvas.width || height !== this.canvas.height) {
      this.canvas.width = width
      this.canvas.height = height
    }
    this.ctx.setTransform(scale, 0, 0, scale, 0, 0)
    this.w = w
    this.h = h
    this.cx = w * 0.5
    this.cy = h * 0.5
    this.radius = Math.min(w * 0.435, h * 0.395)
  }

  backdrop() {
    const g = this.ctx.createRadialGradient(
      this.cx,
      this.cy,
      10,
      this.cx,
      this.cy,
      Math.max(this.w, this.h) * 0.65,
    )
    g.addColorStop(0, '#214348')
    g.addColorStop(0.67, '#102a32')
    g.addColorStop(1, '#081b24')
    this.ctx.fillStyle = g
    this.ctx.fillRect(0, 0, this.w, this.h)
  }

  orbit(marker, label, sites, trail, groundTrack) {
    const { ctx, cx, cy, radius: r } = this
    const [right, up, front] = this.basis()
    // Cache the terrain layer while the observer or the clock changes.
    const diameter = this.dragging
      ? Math.min(300, Math.ceil(2 * r))
      : Math.ceil(2 * r * Math.min(1.35, devicePixelRatio || 1))
    const key = [diameter, this.yaw, this.pitch].join('/')
    if (key !== this.rasterKey) {
      this.rasterKey = key
      this.raster.width = this.raster.height = diameter
      const surface = this.raster.getContext('2d'),
        pixels = surface.createImageData(diameter, diameter)
      const s = this.textureSize,
        axes = this.terrain.axes
      for (let j = 0; j < diameter; j++)
        for (let i = 0; i < diameter; i++) {
          const u = ((i + 0.5) * 2) / diameter - 1,
            v = 1 - ((j + 0.5) * 2) / diameter,
            q = 1 - u * u - v * v
          if (q <= 0) continue
          const z = Math.sqrt(q)
          const x = u * right[0] + v * up[0] + z * front[0],
            y = v * up[1] + z * front[1],
            zz = u * right[2] + v * up[2] + z * front[2]
          const ax = Math.abs(x),
            ay = Math.abs(y),
            az = Math.abs(zz),
            d = Math.max(ax, ay, az)
          const face = d === az ? (zz > 0 ? 0 : 2) : d === ax ? (x > 0 ? 1 : 3) : y > 0 ? 4 : 5
          const [, a, b] = axes[face]
          const tx = clamp(((1 + (a[0] * x + a[1] * y + a[2] * zz) / d) * s) / 2 - 0.5, 0, s - 1)
          const ty = clamp(((1 + (b[0] * x + b[1] * y + b[2] * zz) / d) * s) / 2 - 0.5, 0, s - 1)
          const x0 = Math.floor(tx),
            y0 = Math.floor(ty),
            fx = tx - x0,
            fy = ty - y0
          const offset = (y0 * s + x0) * 3,
            dx = x0 < s - 1 ? 3 : 0,
            dy = y0 < s - 1 ? s * 3 : 0
          const dest = (j * diameter + i) * 4,
            texture = this.textures[face]
          const light = 0.7 + 0.34 * Math.max(0, -0.38 * u + 0.46 * v + 0.8 * z)
          const air = (1 - z) ** 5 * 0.45
          for (let c = 0; c < 3; c++) {
            const at = offset + c,
              top = texture[at] * (1 - fx) + texture[at + dx] * fx
            const bottom = texture[at + dy] * (1 - fx) + texture[at + dx + dy] * fx
            pixels.data[dest + c] =
              (top * (1 - fy) + bottom * fy) * light * (1 - air) + [127, 199, 204][c] * air
          }
          pixels.data[dest + 3] = Math.min(255, z * diameter * 255)
        }
      surface.putImageData(pixels, 0, 0)
    }
    ctx.save()
    ctx.shadowColor = 'rgba(111,195,194,.33)'
    ctx.shadowBlur = 16
    ctx.drawImage(this.raster, cx - r, cy - r, r * 2, r * 2)
    ctx.restore()
    const project = (dir) => [cx + r * dot(dir, right), cy - r * dot(dir, up), dot(dir, front)]
    ctx.strokeStyle = 'rgba(235,245,217,.15)'
    ctx.lineWidth = 0.7
    const arc = (fn) => {
      ctx.beginPath()
      let visible = false
      for (let i = 0; i <= 120; i++) {
        const [x, y, z] = project(fn(i / 120))
        if (z > 0) visible ? ctx.lineTo(x, y) : ctx.moveTo(x, y)
        visible = z > 0
      }
      ctx.stroke()
    }
    const geographic = (lon, lat) => [
      Math.sin(lon) * Math.cos(lat),
      Math.sin(lat),
      Math.cos(lon) * Math.cos(lat),
    ]
    for (let lon = 0; lon < 360; lon += 30)
      arc((t) => geographic((lon * Math.PI) / 180, (t - 0.5) * Math.PI))
    for (let lat = -60; lat <= 60; lat += 30) arc((t) => geographic(t * TAU, (lat * Math.PI) / 180))
    ctx.strokeStyle = gold
    ctx.lineWidth = 1.9
    for (let i = 1; i < trail.length; i++) {
      const a = this.vector(trail[i - 1]),
        b = this.vector(trail[i])
      arc((t) => unit(a.map((v, j) => v + (b[j] - v) * t)))
    }
    if (groundTrack) {
      // Physical positions have a site and an x/z coordinate inside its oriented face.
      // A missing segment (including an unposed border journey) leaves a gap.
      for (const { site, points } of groundTrack.tracks) {
        ctx.beginPath()
        let visible = false
        for (const [x, , z] of points) {
          const [px, py, depth] = project(
            this.vector(site, x / this.terrain.side, z / this.terrain.side),
          )
          if (depth > 0) visible ? ctx.lineTo(px, py) : ctx.moveTo(px, py)
          visible = depth > 0
        }
        ctx.stroke()
      }
    }
    for (const site of sites) {
      const [x, y, z] = project(this.vector(site))
      if (z > 0.06) {
        ctx.beginPath()
        ctx.arc(x, y, 3, 0, TAU)
        ctx.fillStyle = ink
        ctx.fill()
        ctx.strokeStyle = '#374e43'
        ctx.stroke()
      }
    }
    if (marker) {
      const [x, y, z] = project(marker)
      if (z > 0.01) this.pin(x, y, label)
    }
  }

  pin(x, y, label) {
    const ctx = this.ctx
    ctx.strokeStyle = gold
    ctx.fillStyle = gold
    ctx.lineWidth = 1.5
    ctx.beginPath()
    ctx.arc(x, y, 7, 0, TAU)
    ctx.stroke()
    ctx.beginPath()
    ctx.arc(x, y, 2.7, 0, TAU)
    ctx.fill()
    ctx.font = '12px ui-monospace, monospace'
    const width = ctx.measureText(label).width + 15
    const left = clamp(x + 15, 7, this.w - width - 7),
      top = clamp(y - 29, 40, this.h - 60)
    ctx.fillStyle = 'rgba(11,34,36,.90)'
    ctx.fillRect(left, top, width, 22)
    ctx.fillStyle = '#f4dead'
    ctx.fillText(label, left + 7, top + 15)
  }

  region(site) {
    const center = { id: site, x: 0, y: 0, turn: 0 }
    const steps = [
      [0, -1],
      [1, 0],
      [0, 1],
      [-1, 0],
    ]
    const cross = (tile, d) => {
      const native = (d + 4 - tile.turn) % 4
      const [id, enters] = this.terrain.sites[tile.id].neighbors[native]
      return { id, turn: (d + 6 - enters) % 4, x: tile.x + steps[d][0], y: tile.y + steps[d][1] }
    }
    const sides = steps.map((_, d) => cross(center, d))
    const region = [center, ...sides]
    for (let d = 0; d < 4; d++) {
      const next = (d + 1) % 4
      const a = cross(sides[d], next),
        b = cross(sides[next], d)
      // Three faces meet at a cube vertex. Leave that corner of the chart open.
      if (a.id === b.id && a.turn === b.turn) region.push(a)
    }
    return region
  }

  relief(site, local = false, groundTrack = null) {
    const { samples: k, water, sites, side } = this.terrain
    const ctx = this.ctx,
      tiles = local ? [{ id: site, x: 0, y: 0, turn: 0 }] : this.region(site)
    const width = local ? 1 : 3
    const length = Math.min(this.w * 0.41, this.h * 0.63) / width
    const base = this.cy + (local ? this.h * 0.17 : this.h * 0.035)
    const datum = sites[site].elevation
    const reliefScale = length / Math.min(side, 140)
    const project = (x, y, h) => [
      this.cx + (x - y) * length,
      base + (x + y - 1) * length * 0.51 - (h - datum) * reliefScale,
    ]
    const flat = new Set(tiles.map((t) => t.x + ',' + t.y))
    const floor = Math.min(water, ...tiles.map((t) => sites[t.id].elevation)) - 16
    const stride = local ? 1 : 2
    let center, border
    for (const tile of tiles.sort((a, b) => a.x + a.y - b.x - b.y)) {
      const heights = sites[tile.id].heights
      const h = (x, y) => {
        let a = x,
          b = y
        if (tile.turn === 1) {
          a = y
          b = k - x
        }
        if (tile.turn === 2) {
          a = k - x
          b = k - y
        }
        if (tile.turn === 3) {
          a = k - y
          b = x
        }
        return heights[b * (k + 1) + a] / 16
      }
      const p = (x, y, height) => project(tile.x + x / k, tile.y + y / k, height)
      const pt = (x, y) => p(x, y, Math.max(water, h(x, y)))
      for (let y = 0; y < k; y += stride)
        for (let x = 0; x < k; x += stride) {
          const u = x + stride,
            v = y + stride
          const a = h(x, y),
            b = h(u, y),
            c = h(x, v),
            d = h(u, v)
          const color = css(terrainColor((a + b + c + d) / 4, water, (b - a) * 0.7 + (c - a) * 0.3))
          polygon(ctx, [pt(x, y), pt(u, y), pt(u, v), pt(x, v)], color)
        }
      for (let i = 0; i < k; i += stride) {
        if (!flat.has(tile.x + 1 + ',' + tile.y))
          polygon(
            ctx,
            [pt(k, i), pt(k, i + stride), p(k, i + stride, floor), p(k, i, floor)],
            '#514f43',
          )
        if (!flat.has(tile.x + ',' + (tile.y + 1)))
          polygon(
            ctx,
            [pt(i, k), pt(i + stride, k), p(i + stride, k, floor), p(i, k, floor)],
            '#6c6552',
          )
      }
      if (tile.id === site) {
        center = pt(k / 2, k / 2)
        const localPoint = ([x, , z]) => {
          const u = (x * k) / side,
            v = (z * k) / side
          const ix = clamp(Math.floor(u), 0, k - 1),
            iz = clamp(Math.floor(v), 0, k - 1)
          const low = h(ix, iz) * (1 - u + ix) + h(ix + 1, iz) * (u - ix)
          const high = h(ix, iz + 1) * (1 - u + ix) + h(ix + 1, iz + 1) * (u - ix)
          return p(u, v, Math.max(water, low * (1 - v + iz) + high * (v - iz)))
        }
        for (const segment of groundTrack?.tracks || []) {
          if (segment.site !== site) continue
          ctx.beginPath()
          segment.points.forEach((at, i) =>
            i ? ctx.lineTo(...localPoint(at)) : ctx.moveTo(...localPoint(at)),
          )
          ctx.strokeStyle = gold
          ctx.lineWidth = 2
          ctx.stroke()
        }
        if (groundTrack?.resident?.site === site) center = localPoint(groundTrack.resident.pose.at)
        border = []
        for (let i = 0; i < k; i += 2) border.push(pt(i, 0))
        for (let i = 0; i < k; i += 2) border.push(pt(k, i))
        for (let i = k; i > 0; i -= 2) border.push(pt(i, k))
        for (let i = k; i > 0; i -= 2) border.push(pt(0, i))
      }
    }
    if (!local) {
      ctx.beginPath()
      border.forEach(([x, y], i) => (i ? ctx.lineTo(x, y) : ctx.moveTo(x, y)))
      ctx.closePath()
      ctx.strokeStyle = 'rgba(255,219,128,.94)'
      ctx.lineWidth = 1.6
      ctx.stroke()
    }
    return center
  }

  body(blocks) {
    if (!blocks?.length) return
    // Project the actual attached parts. World Y is up in the body document.
    const project = ([x, y, z]) => [0.88 * x - 0.72 * z, 0.38 * x + 0.46 * z - y]
    const blocks3d = blocks
      .map(([center, half, role, held]) => {
        const pts = []
        for (const y of [-1, 1])
          for (const z of [-1, 1])
            for (const x of [-1, 1])
              pts.push(
                project([
                  center[0] + half[0] * x,
                  center[1] + half[1] * y,
                  center[2] + half[2] * z,
                ]),
              )
        return { pts, role, held, depth: center[0] * 0.72 + center[2] * 0.88 + center[1] * 0.64 }
      })
      .sort((a, b) => a.depth - b.depth)
    const pts = blocks3d.flatMap((b) => b.pts),
      xs = pts.map((p) => p[0]),
      ys = pts.map((p) => p[1])
    const xmin = Math.min(...xs),
      xmax = Math.max(...xs),
      ymin = Math.min(...ys),
      ymax = Math.max(...ys)
    const scale = Math.min((this.w * 0.6) / (xmax - xmin || 1), (this.h * 0.4) / (ymax - ymin || 1))
    const palette = {
      tissue: [156, 164, 94],
      limb: [130, 153, 102],
      sense: [196, 131, 82],
      mouth: [217, 158, 88],
    }
    const ctx = this.ctx
    for (const b of blocks3d) {
      const shape = b.pts.map(([x, y]) => [
        this.cx + (x - (xmin + xmax) / 2) * scale,
        this.cy - 5 + (y - (ymin + ymax) / 2) * scale,
      ])
      const base = palette[b.role]
      const tint = Math.min(0.36, (b.held || 0) * 0.05)
      const rgb = base.map((v, i) => v * (1 - tint) + [213, 198, 90][i] * tint)
      for (const [indices, light] of [
        [[1, 3, 7, 5], 0.71],
        [[2, 3, 7, 6], 0.86],
        [[4, 5, 7, 6], 1.12],
      ])
        polygon(
          ctx,
          indices.map((i) => shape[i]),
          css(rgb.map((c) => c * light)),
          'rgba(36,53,38,.65)',
        )
    }
  }

  draw({ level, site, marker, label, sites, body, animal, trail, groundTrack }) {
    this.size()
    this.backdrop()
    if (level === 'orbit') this.orbit(marker, label, sites, trail, groundTrack)
    if (level === 'valley') {
      const [x, y] = this.relief(site, false, groundTrack)
      this.pin(x, y, label)
    }
    if (level === 'animal') {
      if (body) {
        this.ctx.save()
        this.ctx.globalAlpha = 0.48
        this.relief(site, true)
        this.ctx.restore()
        this.body(body)
        const caption = animal.fed
          ? '+' + animal.fed + ' food'
          : animal.journey
            ? 'In transit'
            : 'Resident'
        this.ctx.textAlign = 'center'
        this.ctx.fillStyle = animal.fed ? gold : ink
        this.ctx.font = '15px ui-monospace, monospace'
        this.ctx.fillText(caption, this.cx, this.h * 0.79)
        this.ctx.textAlign = 'left'
      } else this.relief(site)
    }
  }
}
