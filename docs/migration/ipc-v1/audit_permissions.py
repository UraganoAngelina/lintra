"""Audit actual generated spike ACL and reuse native evidence only for identical inputs."""
import gzip
import hashlib
import json
from pathlib import Path
import subprocess


def main():
    root = Path(__file__).resolve().parents[3]
    spike = root / 'spikes/tauri-runtime/src-tauri'
    manifests = json.loads((spike / 'gen/schemas/acl-manifests.json').read_text())
    generated = json.loads((spike / 'gen/schemas/capabilities.json').read_text())
    declared = json.loads((spike / 'capabilities/probe.json').read_text())
    config = json.loads((spike / 'tauri.conf.json').read_text())
    checks = []
    def check(name, value):
        assert value, name
        checks.append(name)
    check('generated capability equals configured local probe capability', generated == {'probe': declared})
    check('one local named window only, no remote capability', declared['local'] is True and
          declared['windows'] == ['probe'] and 'remote' not in declared)
    check('only explicit custom permissions, no core/plugin default sets', declared['permissions'] == ['allow-runtime-probe', 'allow-render-ack'])
    app = manifests['__app-acl__']
    check('custom commands opt in without a default permission', app['default_permission'] is None and not app['permission_sets'])
    commands = []
    for permission in declared['permissions']:
        entry = app['permissions'][permission]
        check('permission has exact allow and no implicit denies ' + permission, len(entry['commands']['allow']) == 1 and entry['commands']['deny'] == [])
        commands += entry['commands']['allow']
    check('resolved command surface excludes fs shell SQL approval and events', commands == ['runtime_probe', 'render_ack'])
    check('configuration selects only audited capability', config['app']['security']['capabilities'] == ['probe'])
    check('release window has no devtools', all(window.get('devtools') is False for window in config['app']['windows']))
    csp = config['app']['security']['csp']
    directives = dict((parts[0], parts[1:]) for raw in csp.split(';') if (parts := raw.strip().split()))
    check('CSP is deny-by-default without inline eval wildcard or external origin', directives['default-src'] == ["'none'"] and
          not any(word in csp for word in ['unsafe-inline', 'unsafe-eval', '*', 'https:']))
    check('CSP connect origins are only pinned internal IPC endpoints', directives['connect-src'] == ['ipc:', 'http://ipc.localhost'])
    qualified = '608ccdc1d79c8495085ade5888e8df99f97c958f'
    source_paths = ['spikes/tauri-runtime/src-tauri/' + suffix for suffix in [
        'src/main.rs', 'src/windows_request_guard.rs', 'build.rs', 'tauri.conf.json',
        'capabilities/probe.json', 'Cargo.toml', 'Cargo.lock']]
    for path in source_paths:
        original = subprocess.check_output(['git', 'show', qualified + ':' + path], cwd=root)
        check('qualified native source unchanged ' + path, (root / path).read_bytes() == original)
    base = root / 'docs/verification/unified-post-m9/T-MIG-P1-04-native/2026-11-14'
    manifest = json.loads((base / 'manifest.json').read_text())
    checksums = {entry['path']: entry['sha256'] for entry in manifest['files']}
    for name in ['native-security.json.gz', 'linux-document-recovery.json.gz']:
        raw = (base / name).read_bytes()
        check('archived native receipt intact ' + name, hashlib.sha256(raw).hexdigest() == checksums[name])
        native = json.loads(gzip.decompress(raw))
        check('native negative invokes and CSP proved on unchanged spike ' + name,
              native['passed'] and native['exit_code'] == 0 and native['report']['forbidden_invokes'] and
              native['report']['invalid_reports'] and native['report']['text_only'] and
              native['collector_connections'] == native['collector_requests'] == 0 and native['report']['csp_events'] >= 12)
    print(json.dumps({'passed': True, 'checks': checks, 'resolved_commands': commands,
                      'qualified_runtime_source': qualified, 'new_native_run': False,
                      'generated_sha256': {str(path.relative_to(root)): hashlib.sha256(path.read_bytes()).hexdigest()
                          for path in [spike / 'gen/schemas/acl-manifests.json', spike / 'gen/schemas/capabilities.json']},
                      'scope': 'Actual spike ACL/CSP audit plus unchanged-source reuse; no product bridge permissions qualified'}, indent=2))


if __name__ == '__main__':
    main()
