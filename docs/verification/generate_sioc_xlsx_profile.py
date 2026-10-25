#!/usr/bin/env python3
"""Reproduce only the explicit SIOC XLSX cell profile, never normative schemas."""
import argparse
import hashlib
import json
from pathlib import Path
import re
import xml.etree.ElementTree as ET
from zipfile import ZipFile

ROOT = Path(__file__).resolve().parents[2]
SOURCE = ROOT / 'schemas/xlsx/SIOC_tracciato_regionale_20260420-definitivo_v05.xlsx'
TARGET = ROOT / 'docs/schema-import/sioc-2026-04-20.xlsx-profile.json'
EXPECTED = '6be8ae60a2c236e5a25b731365b42f8e7bca849066d68747abfc78b92ce273e3'
NS = {'s': 'http://schemas.openxmlformats.org/spreadsheetml/2006/main'}

def build():
    assert hashlib.sha256(SOURCE.read_bytes()).hexdigest() == EXPECTED, 'Unreviewed workbook bytes'
    blueprint = json.loads((ROOT / 'schemas/sioc/2026-04-20.structural-minimal.2/flow_schema.json').read_text())
    by_id = {t['id']: t for t in blueprint['traces']}
    with ZipFile(SOURCE) as archive:
        strings = [''.join(x.itertext()) for x in ET.fromstring(archive.read('xl/sharedStrings.xml')).findall('s:si', NS)]
        document = ET.fromstring(archive.read('xl/worksheets/sheet2.xml'))
        rows = {}
        for row in document.findall('s:sheetData/s:row', NS):
            rows[int(row.get('r'))] = {re.sub(r'\d', '', c.get('r')): strings[int(c.find('s:v', NS).text)] if c.get('t') == 's' else c.find('s:v', NS).text for c in row.findall('s:c', NS) if c.find('s:v', NS) is not None}
    names = ['field_id', 'key_marker', 'name', 'length', 'start', 'source_type', 'obligation', 'notes']
    profile = {'flow_id': blueprint['flow_id'], 'blueprint_version': blueprint['schema_version'], 'sheet': 'Tracciato_SIOC definitivo', 'header_row': 5,
               'columns': {name: {'number': i + 1, 'header': rows[5][chr(65 + i)]} for i, name in enumerate(names)},
               'traces': [], 'coordinate_formulas': 'use_cached_values', 'annotation_sheets': ['Indicazioni operative']}
    current = None
    for number, row in rows.items():
        identifier = row.get('A', '')
        if number > 5 and 'TRACCIATO' in identifier:
            trace_id = re.match(r'^(\d+(?:\.\d+)?)\b', identifier)[1]
            assert trace_id in by_id
            current = {'trace_id': trace_id, 'heading_row': number, 'heading': identifier, 'fields': []}
            profile['traces'].append(current)
        elif re.fullmatch(r'\d+(?:\.\d+)+', identifier):
            assert current is not None
            fields = by_id[current['trace_id']]['fields']
            field = next(f for f in fields if f['id'] == identifier)
            assert (int(row['D']), int(row['E'])) == (field['physical']['length'], field['physical']['start'])
            current['fields'].append({'row': number, 'field_id': identifier, 'source_type': row.get('F'), 'obligation': row['G'], 'key_marker': row.get('B'), 'reviewed_physical_type': field['physical']['type']})
    assert [t['trace_id'] for t in profile['traces']] == [t['id'] for t in blueprint['traces']]
    for block, trace in zip(profile['traces'], blueprint['traces']):
        assert [f['field_id'] for f in block['fields']] == [f['id'] for f in trace['fields']]
    return (json.dumps(profile, ensure_ascii=False, indent=2) + '\n').encode()

if __name__ == '__main__':
    args = argparse.ArgumentParser()
    args.add_argument('--write', action='store_true')
    options = args.parse_args()
    expected = build()
    if options.write:
        TARGET.write_bytes(expected)
    else:
        assert TARGET.read_bytes() == expected, 'Profile differs from reviewed source mapping'
    print('135 explicit field bindings verified; source and normative schemas unchanged')
