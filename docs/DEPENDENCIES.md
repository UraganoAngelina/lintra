# DEC-15 — Registro dipendenze aggiuntive

T-P5-05, 2026-10-20; CNF-AUDIT-01 approvata con precisazioni dall’autore.
Registrazione prima dell’aggiunta a Cargo.toml; verifica concreta del lock
e dei target Linux GNU / Windows MSVC nelle evidenze della task.

| Crate | Versione fissata | Scopo | Licenza | Pure Rust / configurazione |
|---|---|---|---|---|
| sha2 (default-features=false) | 0.11.1 | SHA-256 incrementale dei byte originali dell’input | MIT OR Apache-2.0 | RustCrypto, nessuna libreria crypto dell’host; backend software disponibile su entrambi i target |
| chrono | 0.4.45 | Solo formattazione timestamp UTC RFC3339 | MIT OR Apache-2.0 | default-features=false, std; niente clock locale, timezone database, libc o dipendenze Windows abilitate per il clock |

Fonti primarie consultate: [sha2 0.11.1](https://docs.rs/crate/sha2/0.11.1),
[API incrementale](https://docs.rs/sha2/0.11.1/sha2/),
[chrono 0.4.45](https://docs.rs/crate/chrono/0.4.45); per chrono verificati
anche Cargo.toml e licenza della copia locale del crate.
Nessun algoritmo crypto o calendario riscritto nel core.

Cargo.toml sha2 locale: MSRV 1.88, compatibile con Rust 1.99.1,
nessun build script. Default oid/alloc disabilitati perché non necessari
a SHA-256 incrementale. cpufeatures è Rust; la presenza di libc nel lock
non implica una dipendenza attiva sui target x86_64: verificare con cargo tree.

Verifica locale effettiva: cargo tree --locked su x86_64-unknown-linux-gnu
e x86_64-pc-windows-msvc non abilita libc; chrono ha solo std/alloc.
Controllati anche MIT OR Apache-2.0 e MSRV di cpufeatures, digest,
block-buffer, crypto-common, hybrid-array, typenum, num-traits, autocfg, cfg-if.
Tutti compatibili con la toolchain 1.99.1; esito native CI nel report T-P5-05.

Compatibilità nativa effettivamente verificata dalla [CI 39399359949](https://github.com/UraganoAngelina/lintra/actions/runs/39399359949)
su 85f1ac7: Linux x86_64 GNU e Windows x86_64 MSVC entrambi success,
con test/build release; oltre al check locale Windows GNU all-targets.

## T-P8-01 — clap, dipendenza nominata dal piano

clap =4.6.7, MIT OR Apache-2.0, MSRV 1.85: compatibile con pin 1.99.1.
Fonti primarie: https://docs.rs/clap/4.6.7/clap/ e Cargo.toml locale della
versione scaricata con cargo info clap@4.6.7 (licenza/MSRV/feature verificati).
Solo std/help/usage/error-context, default disabilitati: niente derive, color,
env o suggestions. EnvProvider rimane una porta esplicita testabile.
Nessun import clap nel core: è usato esclusivamente dal target binario CLI.
Lock e dipendenze transitive/target verificati nei gate T-P8-01. Nessun nuovo
crate di test. clap è già nominato da ARCH §4/Piano T-P8-01, nessuna nuova
scelta di framework o deroga DEC-15 richiesta.

Lock T-P8-01: clap_builder 4.6.7 (MIT OR Apache-2.0, MSRV 1.85),
clap_lex 1.1.1 (MIT OR Apache-2.0, MSRV 1.85), anstyle 1.0.14
(MIT OR Apache-2.0, MSRV 1.66). Metadati verificati dai Cargo.toml scaricati.
Cargo tree --locked su Linux GNU e Windows MSVC: stesso ramo clap →
clap_builder → anstyle/clap_lex, nessuna nuova dipendenza nativa o build script.
