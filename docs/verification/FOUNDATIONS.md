# Verifica finale delle fondamenta

Sessione: 2026-10-16, 17:06:21 CEST.

Ambiente Linux locale, toolchain fissata nel repository. Nessuna run GitHub o Windows reale.

## `rustc --version`

Exit code: 0

```text
rustc 1.99.1 (58cf8f9c3 2026-10-13)
```

## `cargo fmt --all -- --check`

Exit code: 0

```text
(nessun output)
```

## `cargo check --all-targets --locked`

Exit code: 0

```text
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.00s
```

## `cargo clippy --all-targets --target x86_64-unknown-linux-gnu --locked -- -D warnings`

Exit code: 0

```text
    Checking lintra v0.1.0 (/home/alberto/Desktop/LINTRA/lintra)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.07s
```

## `cargo test --target x86_64-unknown-linux-gnu --locked`

Exit code: 0

```text

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


running 2 tests
test missing_file_under_path_with_spaces_returns_not_found ... ok
test byte_roundtrip_preserves_data_under_paths_with_spaces ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


running 1 test
test src/lib.rs - (line 5) ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


running 2 tests
test src/lib.rs - (line 11) - compile fail ... ok
test src/lib.rs - (line 17) - compile fail ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s

all doctests ran in 0.18s; merged doctests compilation took 0.15s
   Compiling lintra v0.1.0 (/home/alberto/Desktop/LINTRA/lintra)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.20s
     Running unittests src/lib.rs (target/x86_64-unknown-linux-gnu/debug/deps/lintra-e90efc235e87bc5e)
     Running unittests src/main.rs (target/x86_64-unknown-linux-gnu/debug/deps/lintra-37f536405d774a9a)
     Running tests/platform_paths.rs (target/x86_64-unknown-linux-gnu/debug/deps/platform_paths-c0e1a9284bce4317)
   Doc-tests lintra
```

## `cargo build --release --target x86_64-unknown-linux-gnu --locked`

Exit code: 0

```text
   Compiling lintra v0.1.0 (/home/alberto/Desktop/LINTRA/lintra)
    Finished `release` profile [optimized] target(s) in 0.06s
```

## `cargo check --all-targets --target x86_64-pc-windows-gnu --locked`

Exit code: 0

```text
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.01s
```

## `cargo run --locked`

Exit code: 0

```text
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.00s
     Running `target/debug/lintra`
```

## `git diff --check`

Exit code: 0

```text
(nessun output)
```

## Controlli aggiuntivi

- cargo metadata --no-deps --locked --format-version 1: exit 0, package singolo senza dipendenze.
- Undici mod.rs senza codice; nessun file .txt nel repository.
- ARCH, DET, PLAN, ADR e matrice conflitti identici byte per byte agli originali.
- Test di piattaforma: 2; doctest: 3. Nessuna regola normativa implementata.
- Test Windows GNU eseguiti solo in compilazione; linking/Wine e runner MSVC non disponibili.
- YAML controllato manualmente dall’implementer/reviewer, nessun actionlint eseguito.
- QG-2 resta aperto finché entrambi i job reali non producono evidenza verde.
