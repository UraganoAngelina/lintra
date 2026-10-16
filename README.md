# Lintra

Motore generico e versionato per validare flussi sanitari fixed-width.
**Il motore conosce come validare; gli schemi conoscono cosa è valido.**

Il repository contiene le fondamenta Rust: una libreria, un entrypoint CLI
vuoto, i confini dei moduli e la configurazione di sviluppo. Il binario
termina senza elaborare input o produrre output. Il consuntivo delle task
e dei gate realmente verificati è in
[IMPLEMENTATION_STATUS.md](IMPLEMENTATION_STATUS.md).

## Sviluppo locale

Eseguire dalla directory di questo README, con Rust/Cargo e i componenti
rustfmt e clippy disponibili:

```sh
cargo build --locked
cargo fmt --all -- --check
cargo check --all-targets --locked
cargo clippy --all-targets --locked -- -D warnings
cargo test --locked
cargo run --locked
```

Il package non ha dipendenze esterne. Rust è fissato alla versione **1.99.1**;
rustup installa/seleziona il pin dichiarato e `Cargo.lock` è versionato.
La [guida di sviluppo](docs/DEVELOPMENT.md) descrive preparazione su Arch,
cross-check Windows GNU e conservazione dei byte delle fixture.
La CI T-P1-03 usa runner Linux e Windows MSVC reali; le verifiche locali
non attestano il superamento del gate multipiattaforma.

## Struttura

- `src/lib.rs`: moduli privati `application`, `domain`, `schema`, `input`,
  `engine`, `validators`, `context`, `persistence`, `reference_data`, `report`.
- `src/main.rs` e `src/cli/`: adapter CLI separato dalla libreria.
- `schemas/`, `reference-data/`: directory riservate ai bundle versionati;
  nessuno schema o dataset importato nello scaffold.
- `tests/{integration,golden,fixtures,support}/`: directory riservate ai test
  futuri. I doctest verificano il confine pubblico della libreria;
  i test di piattaforma verificano I/O sintetico su percorsi con spazi.
- `docs/`: copie inalterate delle specifiche approvate.
- [IMPLEMENTATION_STATUS.md](IMPLEMENTATION_STATUS.md): consuntivo,
  verifiche reali, decisioni e handoff.

`context` ospiterà il contesto effimero della validazione a due passate;
`persistence` è il confine distinto della persistenza core generica.
Tutti i `mod.rs` sono vuoti; backend e contratti non sono ancora implementati.

## Documentazione e ripresa del lavoro

Leggere in ordine [ARCHITECTURE](docs/ARCHITECTURE.md),
[DETAILED_DESIGN](docs/DETAILED_DESIGN.md),
[MASTER_IMPLEMENTATION_PLAN](docs/MASTER_IMPLEMENTATION_PLAN.md) (§4, §14),
[ADR-PERSIST-001](docs/PERSISTENCE_CORE_SPEC.md),
[matrice dei conflitti risolti](docs/CONFLICT_RESOLUTION_MATRIX.md) e lo stato.
Il [piano locale T-P1-01](docs/bootstrap/T-P1-01_PLAN.md) registra il perimetro;
il [prompt atomico](docs/bootstrap/T-P1-01.md) conserva i criteri di accettazione.

Origine delle copie: directory adiacente
`../LINTRA_STARTER_KIT_FINAL_20261016/`. Il kit originale contiene anche
`MASTER_AGENT_PROMPT.md`, `TASK_DEPENDENCY_MAP.md`, `ATOMIC_TASKS_INDEX.md`
e i prompt delle task successive. Non è una dipendenza di build/runtime.
L'autore ha autorizzato l'esecuzione continua e parallela dal 2026-10-16.
Le task seguono le dipendenze effettive: nessun avanzamento supera un gate
CI o normativo ancora aperto. Consultare lo stato per la frontiera corrente.

L'[inventario degli input normativi](docs/normative/INPUT_INVENTORY.md)
distingue fonti ricevute, configurazioni derivate e materiali mancanti;
la ricezione di uno schema FROZEN non equivale a validazione Rust o chiusura G0.

## Dati locali

Non copiare o versionare dati sanitari reali, identificativi o credenziali.
Il kit locale contiene quattro export aggiuntivi in `EXPORT_CAD/`, esclusi
da questo repository; il controllo di integrità completo del kit li segnala
come extra. I test devono usare esclusivamente dati sintetici.

`.gitignore` esclude export, file `.txt`, stato runtime, report e segreti.
I `.txt` sintetici in `tests/fixtures/` e i JSONL attesi sotto test sono
ammessi; i nomi degli export `A.txt`, `B1.txt`, `B2.txt`, `C.txt`, `D.txt`
restano esclusi ovunque. Questa protezione non sostituisce la revisione
dei file prima di aggiungerli a Git.
