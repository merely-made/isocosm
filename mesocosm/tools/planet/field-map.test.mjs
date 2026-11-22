// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0
import assert from 'node:assert/strict'
import { test } from 'node:test'
import { FieldMap, measure, sizeOf } from './field-map.js'

test('separated parts and asymmetric half-extents give one physical width, depth and height', () => {
  const body = [
    [
      [-2, 3, 5],
      [2, 1, 3],
    ],
    [
      [5, -1, 2],
      [3, 2, 1],
    ],
  ]
  assert.deepEqual(sizeOf(body), [12, 7, 7])
  assert.deepEqual(sizeOf([]), [])
  assert.equal(measure(12, 1000), '12 mm')
  assert.equal(measure(12, 25), '300 µm')
  assert.equal(measure(512, 1000), '51.2 cm')
})

test('entering a feeding patch centers the two participants at the current magnification', () => {
  const map = new FieldMap({ clientWidth: 800, clientHeight: 400 }, { side: 512 })
  const pose = (id, x, z) => ({ id, pose: { at: [x, 100, z] } })
  const residents = [pose(1, 400, 400), pose(2, 42, 60), pose(3, 46, 64)]
  const span = map.span
  map.focus(residents, [{ process: 'ecology:feed-1', from: 2, to: 3 }])
  assert.deepEqual([map.x, map.z, map.span], [44, 62, span])
  map.focus([])
  assert.deepEqual([map.x, map.z, map.span], [256, 256, span])
  map.fit()
  assert.ok(map.span / 2 >= 512, 'whole site fits both axes')
})
