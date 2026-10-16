# Sviluppo locale

Riferimenti: [ARCH §39B](ARCHITECTURE.md#39b-ambiente-di-sviluppo-cachyos--arch-linux),
[DET Addendum A.4–A.5](DETAILED_DESIGN.md#a4-test-aggiuntivi),
[Piano T-P1-02](MASTER_IMPLEMENTATION_PLAN.md#103-fase-p1--fondamenta-repository-e-ci).

La toolchain è fissata a **Rust 1.99.1** in `rust-toolchain.toml`, con profilo
`minimal`, `rustfmt`, `clippy` e il target `x86_64-pc-windows-gnu`.
La versione è stata scelta esplicitamente dall'autore il 2026-10-16 (DEC-11).
Usare `rustup`, eseguire i comandi dalla root del repository e mantenere
`Cargo.lock` versionato. I comandi `--locked` impediscono aggiornamenti impliciti
del lockfile; la task non introduce dipendenze.

## Preparazione su CachyOS / Arch Linux

Installare gli strumenti di sviluppo con il gestore di pacchetti della macchina:

```sh
sudo pacman -S rustup mingw-w64-gcc wine
rustup toolchain install 1.99.1 --profile minimal --component rustfmt --component clippy --target x86_64-pc-windows-gnu
```

Il file di pin permette a rustup di selezionare la versione corretta senza
cambiare la toolchain predefinita globale. Su altri sistemi usare l'installazione
rustup appropriata al sistema operativo. MinGW e Wine servono allo sviluppo
Windows da Linux e non sono dipendenze runtime del prodotto.

## Verifiche prima dell'handoff

```sh
cargo fmt --all -- --check
cargo check --all-targets --locked
cargo clippy --all-targets --locked -- -D warnings
cargo test --locked
cargo check --all-targets --locked --target x86_64-pc-windows-gnu
```

Il cross-check intercetta errori di compilazione per Windows; non esegue
il linker né i test. Per verificare anche linking e comportamento sotto Wine:

```sh
cargo build --locked --target x86_64-pc-windows-gnu
cargo test --locked --target x86_64-pc-windows-gnu
```

`.cargo/config.toml` associa **solo** il target GNU al linker
`x86_64-w64-mingw32-gcc` e al runner `wine`. Non imposta un target globale:
i comandi senza `--target` compilano per l'host. La CI Windows usa il target
nativo `x86_64-pc-windows-msvc`, al quale queste opzioni GNU non si applicano.
La release Windows MSVC viene costruita sul runner Windows; la CRT statica
e i controlli di autosufficienza appartengono alle task di release.
Wine e il cross-check locale non sostituiscono la CI su Windows reale.

La baseline Windows della toolchain è **Windows 10 / Windows Server 2016
o successivi**, per x86_64 (supporto ufficiale
[GNU](https://doc.rust-lang.org/rustc/platform-support/windows-gnu.html) e
[MSVC](https://doc.rust-lang.org/rustc/platform-support/windows-msvc.html)).
DEC-11 resta aperta per la strategia di distribuzione
Linux: baseline glibc oppure musl da decidere nella relativa task di packaging;
la configurazione locale non seleziona una baseline Linux di release.

## Fixture, golden e terminatori di riga

`.gitattributes` applica `* text=auto` ai normali file di testo e `-text` a
`tests/fixtures/**` e `tests/golden/**`. Queste eccezioni conservano tutti
i byte, inclusi LF, CRLF e BOM, anche nei checkout Windows con
`core.autocrlf=true`. Non normalizzare manualmente fixture o golden:
record fixed-width e output attesi richiedono confronti byte per byte.

Controllo rapido degli attributi, anche per percorsi ancora inesistenti:

```sh
git check-attr text -- Cargo.toml tests/fixtures/synthetic-lf.txt tests/golden/synthetic-crlf.jsonl
```

Esito atteso: `auto` per `Cargo.toml`, `unset` per fixture e golden.
Le verifiche dei terminatori devono usare dati sintetici in un repository
temporaneo; non importare dati sanitari reali. Il normale testo può essere
convertito da Git secondo la configurazione locale, mentre i due alberi
di test devono restare identici byte per byte in indice e checkout.

Fonti ufficiali della versione:
[Rust 1.99.0](https://blog.rust-lang.org/2026/10/01/Rust-1.99.0/),
[Rust 1.99.1](https://blog.rust-lang.org/2026/10/15/Rust-1.99.1/).
