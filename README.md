# Lintra

Motore generico e versionato per validare flussi sanitari fixed-width.
**Il motore conosce come validare; gli schemi conoscono cosa è valido.**

Questo repository contiene lo scaffold della sola **T-P1-01**: una libreria,
un entrypoint CLI vuoto e i confini dei moduli. Il binario termina senza
elaborare input o produrre output. La validazione sarà implementata nelle
task successive del piano.

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

Il package non ha dipendenze esterne. `Cargo.lock` è incluso per eseguire
i controlli con `--locked`. Il pin della toolchain e la configurazione
di sviluppo Windows spettano a T-P1-02; la CI Linux/Windows a T-P1-03.
Le verifiche locali non attestano il superamento di questi gate.

## Struttura

- `src/lib.rs`: moduli privati `application`, `domain`, `schema`, `input`,
  `engine`, `validators`, `context`, `persistence`, `reference_data`, `report`.
- `src/main.rs` e `src/cli/`: adapter CLI separato dalla libreria.
- `schemas/`, `reference-data/`: directory riservate ai bundle versionati;
  nessuno schema o dataset importato nello scaffold.
- `tests/{integration,golden,fixtures,support}/`: directory riservate ai test
  futuri. I test attuali sono doctest sul confine pubblico della libreria.
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
La prossima task prevista è **T-P1-02**, da avviare soltanto dopo revisione
e autorizzazione dell'autore.

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
