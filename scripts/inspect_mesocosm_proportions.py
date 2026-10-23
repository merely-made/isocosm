#!/usr/bin/env python3
# Copyright 2026 Mark Alan Boykin
# SPDX-License-Identifier: MPL-2.0
"""Compare recorded GPU viewports and the native recipes that supplied them."""
import argparse
import hashlib
import json
from pathlib import Path

from PIL import Image, ImageDraw


def inspect(prefix):
    saved = json.loads(prefix.with_suffix('.trace.json').read_text())
    genesis = saved['native']['genesis']
    frames_dir = prefix.parent / (prefix.name + '-motion')
    frames = json.loads((frames_dir / 'frames.json').read_text())
    frame = next(frame for frame in frames if frame['roots'])
    source = frames_dir / frame['image']
    lineages = {}
    for key, lineage in genesis['lineages'].items():
        development = lineage.get('development')
        if not development:
            continue
        recipe = development['recipe']
        lineages[key] = {'kingdom': lineage['kingdom'], 'traits': lineage['traits'],
                         'recipe': recipe}
    sizes = []
    cell_mm = genesis['world']['base_unit_micrometres'] / 1000
    for body in frame['roots']:
        bounds = body.get('bounds') or []
        if bounds:
            size = [(max(hi[k] for _, hi in bounds) - min(lo[k] for lo, _ in bounds)) * cell_mm
                    for k in range(3)]
            sizes.append({'id': body['id'], 'extent_mm': size, 'yaw': body['yaw']})
    record = {'seed': genesis['seed'], 'source': str(source),
              'source_sha256': hashlib.sha256(source.read_bytes()).hexdigest(),
              'lineages': lineages, 'physical_bodies': sizes,
              'base_unit_micrometres': genesis['world']['base_unit_micrometres'],
              'window': frame['window'], 'native_hash': f"{saved['native']['state_hash']:016x}"}
    return Image.open(source).convert('RGB'), record


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('prefixes', nargs='+', type=Path)
    parser.add_argument('--sheet', required=True, type=Path)
    parser.add_argument('--summary', required=True, type=Path)
    args = parser.parse_args()
    for output in [args.sheet, args.summary]:
        if output.exists():
            parser.error(f'preserving existing receipt: {output}')
    views = [inspect(prefix) for prefix in args.prefixes]
    width = max(image.width for image, _ in views)
    height = max(image.height for image, _ in views) + 58
    sheet = Image.new('RGB', (width * 2, height * ((len(views) + 1) // 2)), '#18231f')
    draw = ImageDraw.Draw(sheet)
    for index, (image, record) in enumerate(views):
        x, y = (index % 2) * width, (index // 2) * height
        draw.text((x + 12, y + 9), f"Seed {record['seed']}", fill='white')
        dims = ', '.join(f"{b['id']}: {' x '.join(f'{v:g}' for v in b['extent_mm'])} mm"
                         for b in record['physical_bodies'])
        draw.text((x + 12, y + 29), dims, fill='#c4d4c8')
        sheet.paste(image, (x, y + 58))
    sheet.save(args.sheet)
    args.summary.write_text(json.dumps({'method': 'unchanged GPU pixels, captioned; native boxes in millimetres',
        'runs': [record for _, record in views]}, indent=2) + '\n')
    print(json.dumps([{'seed': r['seed'], 'bodies': r['physical_bodies']} for _, r in views], indent=2))


if __name__ == '__main__':
    main()
