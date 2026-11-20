# G-MIG-4 — PASS per il backend nativo del blocco P4

Data 20 novembre 2026. **PASS** del checkpoint CI nativo del blocco P4
(`T-MIG-P4-01..12`, tutte DONE) su **`1ab34d868a654fcc84d05ba9617c157af1501ace`**,
stesso SHA di `integration` e `work/p1-foundations` (push autorizzato
dall'autore il 20/11). Il gate qualifica il backend Rust/IPC della GUI; non è
qualifica di release, installer o Windows 11 (G-MIG-8, G-REL).

Riferimento del gate: `FONTI/MIGRATION_PLAN_V1_1_2026-10-30.md` §10 —
QG-1/QG-3/QG-4/QG-6; worker/cancel/results/domain/export **LocalOnly+Full**
backend, DTO negative tests, zero bulk report IPC.

## Run remote sullo SHA

| Run | Workflow / ref | Esito |
| --- | --- | --- |
| 44706977518 | CI, `workflow_dispatch` su `integration` | success, 7 job (1 skipped: `Qualify UI archives`, solo con input `ui_packaging`) |
| 44706949307 | Desktop adapter, `push` su `work/p1-foundations` | success, 2 job |

La run precedente 44588006731 su `1ea8220` era fallita solo in Clippy
`x86_64-pc-windows-msvc` (`dead_code` su `source_for`, helper di test usato da
regressioni Unix); corretta in `1ab34d8` (`#[cfg(unix)]`, solo test).

## Condizioni e evidenze

| Condizione | Evidenza |
| --- | --- |
| QG-1 fmt/Clippy/test workspace e frontend | Verify MSVC: 39 step success/2 skipped OS-specifici, 3.287 test pass; Verify Linux: 37 success/4 skipped, 3.323 test pass; fmt, Clippy default e all-features `-D warnings`, test default/all-features/no-default-features su entrambi; Desktop: npm lint/test (41 pass, 0 fail) e build su entrambi |
| QG-2 nativo stesso SHA (supporto) | Linux e Windows MSVC nativi sullo stesso SHA; non è qualifica Windows 11 (G-MIG-8) |
| QG-3 vincoli di layer | `Verify workspace and core dependency boundary` success su entrambi |
| QG-4 goldens byte-identici | `Compare Linux and Windows JSONL bytes` success (`packaging cross-OS probe: PASS`); `Reopen counterpart snapshot` success in entrambe le direzioni |
| QG-6 memoria streaming | `Qualify synthetic five-trace memory and SQLite profiles` e `Smoke synthetic benchmark` success su entrambi |
| Adapter nativo LocalOnly+Full | src-tauri test release: 61 pass MSVC, 62 pass/1 ignored Linux; Clippy release all-targets `-D warnings`; build release; smoke X11 `NATIVE_DESKTOP_SMOKE_PASS` |
| Worker/cancel/results/domain/export, DTO negativi, zero bulk IPC | Acceptance per task nei dossier `T-MIG-P4-01..12` (review Codex APPROVED o deroga autore P4-11), ora eseguite nativamente nelle suite sopra |

## Verifiche locali (CachyOS, stesso SHA)

PASS: fmt, Clippy workspace default/all-features, test workspace
default/all-features/no-default-features, `check_workspace.py`, xtask
fmt/Clippy, desktop npm lint/test/build, src-tauri fmt/test/Clippy/build
release. Non probanti: `npm ci` FAIL ambientale (engine pin node 26.13.0 vs
26.13.1 locale); smoke Xvfb NOT_RUN (`xvfb-run` assente); Clippy Windows
locale BLOCKED (manca `x86_64-w64-mingw32-gcc`), sostituito dalla CI MSVC.

## Limiti

Non coperti da questo gate: Windows 11 reale, installer NSIS, CachyOS personale
(G-MIG-8); UI/redesign (G-MIG-5..7); release (G-REL). U-NORM-04 resta
WAITING_INTEGRATION e può riprendere ora che G-MIG-4 è chiuso, su mandato.
L'uscita permette **T-MIG-P5-01**.

Log integrali: `G-MIG-4-official-ci-44706977518.log.gz`
(SHA-256 1161c578…6ace) e `G-MIG-4-desktop-ci-44706949307.log.gz`
(SHA-256 ecd9e0ce…1bd0).
