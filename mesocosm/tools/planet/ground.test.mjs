// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

import { strict as assert } from 'node:assert'
import { test } from 'node:test'
import { readTrack } from './ground.js'

const at = (x) => [x, 7, 5]
const pose = (x) => ({ at: at(x), feet: {} })
const boxes = (x) => [[1, at(x), [x + 1, 8, 6], 'mouth']]
const resident = (site, x) => ({ id: 2, site, pose: pose(x), boxes: boxes(x) })
const walk = (site, xs) => ({ id: 2, target: 3, site, path: xs.map(pose), boxes: xs.map(boxes) })

test('the ground pose, articulated body and track use the same native substep', () => {
  const w = {
    ...walk(9, [3, 4, 5]),
    steps: [
      [0, 0],
      [0, 500],
      [2, 0],
    ],
  }
  const frames = [{ poses: [resident(9, 3)] }, { poses: [resident(9, 5)], walks: [w] }]
  const { resident: r, target, tracks } = readTrack(frames, 0, 2, 0.5)
  assert.deepEqual(r.pose.at, at(3.5))
  assert.deepEqual(r.boxes, boxes(3.5))
  assert.deepEqual(tracks, [{ site: 9, points: [at(3), at(3.5)] }])
  assert.equal(target, 3)
  assert.equal(readTrack(frames, 0, 2).tracks.length, 0)
})

test('keeps observed walks across a crossing without drawing an invented connecting path', () => {
  const empty = { poses: [], walks: [] }
  const frames = [
    { poses: [resident(9, 3)] },
    { poses: [resident(9, 5)], walks: [walk(9, [3, 4]), walk(9, [4, 5])] },
    ...Array(30).fill(empty),
    { poses: [resident(14, 8)], walks: [walk(14, [7, 8])] },
  ]
  assert.equal(readTrack(frames, 12, 2).resident, undefined)
  assert.deepEqual(readTrack(frames, frames.length - 1, 2).tracks, [
    { site: 9, points: [at(3), at(4), at(5)] },
    { site: 14, points: [at(7), at(8)] },
  ])
  assert.deepEqual(readTrack(frames, 1, 8).tracks, [])
})

test('slow playback exposes each step of a short approach and preserves the endpoint', () => {
  const w = {
    ...walk(9, [3, 4, 5]),
    steps: [
      [0, 0],
      [2, 0],
      [2, 0],
      [2, 0],
    ],
  }
  const frames = [{ poses: [resident(9, 3)] }, { poses: [resident(9, 5)], walks: [w] }]
  assert.deepEqual(readTrack(frames, 0, 2, 0.5, true).resident.pose.at, at(4))
  assert.deepEqual(
    readTrack(frames, 0, 2, 1, true).resident.pose.at,
    readTrack(frames, 1, 2).resident.pose.at,
  )
})

test('follows departure stances until the native journey takes the resident off the ground', () => {
  const w = { ...walk(98, [7, 8, 9]), target: undefined, exit: 83 }
  const frames = [{ poses: [resident(98, 7)] }, { poses: [], walks: [w] }]
  const middle = readTrack(frames, 0, 2, 0.5)
  assert.equal(middle.exit, 83)
  assert.deepEqual(middle.resident.pose.at, at(8))
  const crossing = readTrack(frames, 1, 2)
  assert.equal(crossing.resident, undefined)
  assert.deepEqual(crossing.tracks, [{ site: 98, points: [at(7), at(8), at(9)] }])
})
