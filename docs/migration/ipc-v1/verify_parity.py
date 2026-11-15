"""Verify frozen synthetic oracle inputs; compare private fixed-clock native artifacts when supplied."""
import hashlib
import json
from pathlib import Path
import sys
import tempfile


def verify(manifest, root, artifacts=None):
    assert manifest['schemaVersion'] == 1 and manifest['synthetic_only'] is True
    assert len(manifest['cases']) == 14
    for path, expected in manifest['expected_provenance'].items():
        assert hashlib.sha256((root / path).read_bytes()).hexdigest() == expected, 'Oracle provenance changed'
    count = 0
    for case in manifest['cases']:
        for path, expected in case['sha256'].items():
            file = root / path
            assert file.is_file() and not file.is_symlink(), 'Fixture is not a regular file'
            assert hashlib.sha256(file.read_bytes()).hexdigest() == expected, 'Frozen fixture changed'
            count += 1
        if artifacts is not None:
            for key, name in [('errors', 'errors.jsonl'), ('summary', 'summary.json')]:
                actual = artifacts / case['id'] / name
                assert actual.is_file() and not actual.is_symlink(), 'Missing native output'
                assert actual.read_bytes() == (root / case[key]).read_bytes(), 'Native byte parity failed'
    return count


def main():
    root = Path(__file__).resolve().parents[3]
    manifest = json.loads((Path(__file__).parent / 'parity-fixtures.json').read_text())
    assert len(sys.argv) <= 2, 'Supply only the private native artifact directory'
    artifacts = Path(sys.argv[1]) if len(sys.argv) == 2 else None
    count = verify(manifest, root, artifacts)
    # Negative checks exercise the oracle boundary without modifying any fixture.
    changed = json.loads(json.dumps(manifest))
    path = next(iter(changed['cases'][0]['sha256']))
    changed['cases'][0]['sha256'][path] = '0' * 64
    try:
        verify(changed, root)
        raise RuntimeError('Changed oracle accepted')
    except AssertionError:
        pass
    with tempfile.TemporaryDirectory(prefix='lintra-parity-comparator-') as temporary:
        folder = Path(temporary)
        for case in manifest['cases']:
            target = folder / case['id']
            target.mkdir(parents=True)
            for key, name in [('errors', 'errors.jsonl'), ('summary', 'summary.json')]:
                (target / name).write_bytes((root / case[key]).read_bytes())
        verify(manifest, root, folder)
        first = folder / manifest['cases'][0]['id'] / 'errors.jsonl'
        first.write_bytes(first.read_bytes() + b'\n')
        try:
            verify(manifest, root, folder)
            raise RuntimeError('Extra output byte accepted')
        except AssertionError:
            pass
    print(json.dumps({'passed': True, 'cases': 14, 'hashed_fixture_files': count,
                      'changed_oracle_rejected': True,
                      'extra_output_byte_rejected': True,
                      'artifact_comparison': 'PASS' if artifacts is not None else 'NOT_RUN',
                      'native_parity': 'NOT_ATTESTED; execution provenance requires a native task receipt',
                      'scope': 'Frozen independent synthetic golden contract; no expected regeneration or engine execution'}, indent=2))


if __name__ == '__main__':
    main()
