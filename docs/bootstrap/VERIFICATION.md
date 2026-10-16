# T-P1-01 — Evidenze di verifica

Data sessione: 2026-10-16; ora locale: 16:30:13 CEST.

Ambiente: Linux locale; nessuna CI o esecuzione Windows.
Output catturato dai comandi effettivamente eseguiti.

## `rustc --version`

Exit code: 0

```text
rustc 1.99.0 (b940084d7 2026-09-28)
```

## `cargo --version`

Exit code: 0

```text
cargo 1.99.0 (5f94df478 2026-08-27)
```

## `rustup target list --installed`

Exit code: 0

```text
x86_64-unknown-linux-gnu
```

## `cargo fmt --all -- --check`

Exit code: 0

```text
(nessun output)
```

## `cargo build --locked`

Exit code: 0

```text
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.04s
```

## `cargo check --all-targets --locked`

Exit code: 0

```text
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.00s
```

## `cargo clippy --all-targets --locked -- -D warnings`

Exit code: 0

```text
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.05s
```

## `cargo test --locked`

Exit code: 0

```text

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


running 1 test
test src/lib.rs - (line 5) ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


running 2 tests
test src/lib.rs - (line 11) - compile fail ... ok
test src/lib.rs - (line 17) - compile fail ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s

all doctests ran in 0.25s; merged doctests compilation took 0.22s
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.05s
     Running unittests src/lib.rs (target/debug/deps/lintra-a2dc8b9ddc4c5e15)
     Running unittests src/main.rs (target/debug/deps/lintra-b5634ac2ccd3f5be)
   Doc-tests lintra
```

## `cargo run --locked`

Exit code: 0

```text
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.02s
     Running `target/debug/lintra`
```

## `cargo tree --locked`

Exit code: 0

```text
lintra v0.1.0 (/home/alberto/Desktop/LINTRA/lintra)
```

## `git diff --check`

Exit code: 0

```text
(nessun output)
```

## Integrità starter kit

`python3 VERIFY_KIT.py`, cwd: directory originale del kit. Exit code: 1.

```text
Kit: LINTRA_STARTER_KIT_FINAL_20261016
File verificati (escluso manifest): 196
Prompt atomici: 140
Esito: FAIL
 - EXTRA: EXPORT_CAD/A.txt
 - EXTRA: EXPORT_CAD/B1.txt
 - EXTRA: EXPORT_CAD/C.txt
 - EXTRA: EXPORT_CAD/D.txt
```

Controllo indipendente Python hashlib/Path: **196/196** file censiti
con dimensioni e SHA-256 corretti, **140** prompt. Il FAIL complessivo
rimane dovuto ai quattro extra; il loro contenuto non è stato letto.

## Copie documentali

Confronto `read_bytes()` e SHA-256 delle copie rispetto alle fonti:

| Copia | SHA-256 (uguale alla fonte) |
|---|---|
| `docs/ARCHITECTURE.md` | `37c4b0bc43b65a90a9ccd3bb6a6ae0f5b17ca8424c923207b47c6a91c18d1093` |
| `docs/DETAILED_DESIGN.md` | `f626c3b0f3f321cca70476c41eefeb7c37b49db0bbfc8215d6ff7c81394987dc` |
| `docs/MASTER_IMPLEMENTATION_PLAN.md` | `6a2161709e929a8de66984eba1c12540d998df042d499376cd546f2bd9e88b40` |
| `docs/PERSISTENCE_CORE_SPEC.md` | `e3ce097710304e92bc6420ebe4844a08e68816a8220afc334f1cd9aeb1b6b945` |
| `docs/CONFLICT_RESOLUTION_MATRIX.md` | `c907503b525cee9b4080cfba901e0e408bfc22c3e1d8ac2d4e71f0c000fab124` |
| `docs/bootstrap/FIRST_TASK_T-P1-01_PROMPT.md` | `6ab9937bf99206798d0acf393c6dd5d477139b7336af6c3b7c5f2d449f5ffe30` |
| `docs/bootstrap/T-P1-01.md` | `c1054097d53d89db4d9e830368096789fd03909d24fd55d110c93180cf8f229d` |

## Protezioni Git

Per ogni percorso virtuale: `git check-ignore --no-index -q -- <path>`.
Exit 0 atteso per esclusi; exit 1 atteso per ammessi. Nessun file sanitario
creato per questa verifica. Tutti i **43** confronti hanno esito atteso.

| Percorso | Esito |
|---|---|
| `target/debug/lintra` | Ignorato (0) |
| `EXPORT_CAD/A.txt` | Ignorato (0) |
| `nested/EXPORT_CAD/D.txt` | Ignorato (0) |
| `A.txt` | Ignorato (0) |
| `B1.txt` | Ignorato (0) |
| `B2.txt` | Ignorato (0) |
| `C.txt` | Ignorato (0) |
| `D.txt` | Ignorato (0) |
| `tests/fixtures/A.txt` | Ignorato (0) |
| `tests/fixtures/nested/D.txt` | Ignorato (0) |
| `input/source.txt` | Ignorato (0) |
| `inputs/renamed.json` | Ignorato (0) |
| `patient-data/renamed.json` | Ignorato (0) |
| `private/data.json` | Ignorato (0) |
| `storage/snapshot.json` | Ignorato (0) |
| `state/metadata.json` | Ignorato (0) |
| `snapshots/snapshot.json` | Ignorato (0) |
| `history/session.json` | Ignorato (0) |
| `reports/report.csv` | Ignorato (0) |
| `output/errors.xlsx` | Ignorato (0) |
| `errors.jsonl` | Ignorato (0) |
| `trace.log` | Ignorato (0) |
| `.env` | Ignorato (0) |
| `.env.local` | Ignorato (0) |
| `keys/identity.key` | Ignorato (0) |
| `keys/private.pem` | Ignorato (0) |
| `secrets/config.json` | Ignorato (0) |
| `credentials.json` | Ignorato (0) |
| `local.sqlite3-wal` | Ignorato (0) |
| `LINTRA_STARTER_KIT_FINAL_20261016/docs/readme.md` | Ignorato (0) |
| `Cargo.toml` | Ammesso (1) |
| `Cargo.lock` | Ammesso (1) |
| `README.md` | Ammesso (1) |
| `IMPLEMENTATION_STATUS.md` | Ammesso (1) |
| `src/input/mod.rs` | Ammesso (1) |
| `src/cli/mod.rs` | Ammesso (1) |
| `docs/ARCHITECTURE.md` | Ammesso (1) |
| `tests/fixtures/synthetic.txt` | Ammesso (1) |
| `tests/fixtures/synthetic/record.txt` | Ammesso (1) |
| `tests/fixtures/synthetic/input.jsonl` | Ammesso (1) |
| `tests/golden/synthetic/expected.errors.jsonl` | Ammesso (1) |
| `reference-data/codes.csv` | Ammesso (1) |
| `schemas/example/trace.json` | Ammesso (1) |

## Completezza e scope

- `cargo metadata --no-deps --locked --format-version 1`: exit 0;
  un package `lintra`, due target lib/bin, zero dipendenze.
- Ispezione Python: undici mod.rs senza codice (solo newline rustfmt).
- Nessun file .txt o directory EXPORT_CAD nel repository.
- Smoke `cargo run --locked`: exit 0, stdout vuoto; stderr contiene solo Cargo.
- Test: un doctest positivo e due compile-fail superati; zero unit test.
- `git diff --check` non valuta i nuovi file untracked: verifica limitata
  allo stato Git corrente, senza pretendere una revisione di diff inesistenti.
- Target Windows assente: cross-check non eseguito; nessuna installazione.
- CI multipiattaforma, gate P1/M1, release e test normativi non eseguiti.

## Git status al termine

```text
## No commits yet on main
?? .gitignore
?? Cargo.lock
?? Cargo.toml
?? IMPLEMENTATION_STATUS.md
?? README.md
?? docs/ARCHITECTURE.md
?? docs/CONFLICT_RESOLUTION_MATRIX.md
?? docs/DETAILED_DESIGN.md
?? docs/MASTER_IMPLEMENTATION_PLAN.md
?? docs/PERSISTENCE_CORE_SPEC.md
?? docs/bootstrap/FIRST_TASK_T-P1-01_PROMPT.md
?? docs/bootstrap/T-P1-01.md
?? docs/bootstrap/T-P1-01_PLAN.md
?? docs/bootstrap/VERIFICATION.md
?? reference-data/.gitkeep
?? schemas/.gitkeep
?? src/application/mod.rs
?? src/cli/mod.rs
?? src/context/mod.rs
?? src/domain/mod.rs
?? src/engine/mod.rs
?? src/input/mod.rs
?? src/lib.rs
?? src/main.rs
?? src/persistence/mod.rs
?? src/reference_data/mod.rs
?? src/report/mod.rs
?? src/schema/mod.rs
?? src/validators/mod.rs
?? tests/fixtures/.gitkeep
?? tests/golden/.gitkeep
?? tests/integration/.gitkeep
?? tests/support/.gitkeep
```

Branch `main`, nessun commit, file nuovi untracked, nessun remote configurato.


## Riesame finale DET §163 / ARCH §37

Ricerca `rg -n` in `src/` per `SIOC`, `CAD`, `clap`, `slint`,
`unwrap(`, `trim(`, `pub mod`, `pub use`, `unsafe`: nessuna occorrenza.
Riesame diretto di manifest, lib/main e gitignore: nessuna API pubblica,
CLI separata dalla library, nessuna logica normativa o backend.
Piano locale marcato completato; T-P1-02 non iniziata.
