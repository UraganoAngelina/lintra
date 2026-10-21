# Lintra

Motore generico e versionato per validare flussi sanitari fixed-width.
**Il motore conosce come validare; gli schemi conoscono cosa è valido.**

La libreria espone `ValidationService` e una factory filesystem LocalOnly,
con compiler incrementale, validazione strutturale e controlli locali supportati,
report JSONL/summary, audit degli input, progress e cancellazione. Full,
reference data e persistenza completa rimangono consegne future esplicite.
Il consuntivo verificato è in [IMPLEMENTATION_STATUS.md](IMPLEMENTATION_STATUS.md).

La CLI è attualmente uno scheletro: `--help` e il parsing di `validate` sono
disponibili; l’esecuzione del comando verrà collegata nella task T-P8-03.
Una richiesta preparata non esegue validazioni e termina con errore tecnico
esplicito. Non vengono creati report o dichiarati risultati conformi.

## Sviluppo locale

Eseguire dalla directory di questo README, con Rust/Cargo e i componenti
rustfmt e clippy disponibili:

```sh
cargo build --locked
cargo fmt --all -- --check
cargo check --all-targets --locked
cargo clippy --all-targets --locked -- -D warnings
cargo test --locked
cargo run --locked -- --help
```

Le dipendenze sono registrate in Cargo.toml/Cargo.lock e nel registro
[DEC-15](docs/DEPENDENCIES.md). Rust è fissato alla versione **1.99.1**;
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
- `tests/{integration,golden,fixtures,support}/`: test sintetici e harness
  golden. I doctest verificano il confine pubblico della libreria;
  i test di piattaforma verificano I/O sintetico su percorsi con spazi.
- `docs/`: copie inalterate delle specifiche approvate.
- [IMPLEMENTATION_STATUS.md](IMPLEMENTATION_STATUS.md): consuntivo,
  verifiche reali, decisioni e handoff.

`context` ospiterà il contesto effimero della validazione a due passate;
`persistence` è il confine distinto della persistenza core generica.
La persistenza fisica resta da implementare nelle task PERSIST.

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

## Contratto CLI iniziale — T-P8-01

Exit code: **0** analisi locale valida; **1** errore tecnico o di sintassi;
**2** analisi completata con violazioni; **130** cancellazione controllata.
`--help` termina con 0; lo scheletro non produce ancora esiti di validazione.

Il parser richiede flow, schema-version, trace, input, errors (alias output),
summary ed encoding esplicito (`utf8`, unico profilo attualmente supportato).
Nessuna selezione implicita dell’ultima versione o di Full.

Risoluzione di bundle esterno per coppie complete, in ordine: opzioni
`--schema-dir`/`--reference-data-dir`; variabili `LINTRA_SCHEMA_DIR`/
`LINTRA_REFERENCE_DATA_DIR`; directory `schemas/`/`reference-data/` accanto
all’eseguibile. Una coppia incompleta o una directory scelta non valida produce
errore, senza ripiego su livelli inferiori. Nessuna ricerca in cwd/home.
Lo snapshot incorporato sarà introdotto in P15: senza bundle esterno il build
corrente fallisce esplicitamente. La risoluzione dei path non fornisce ancora
un provider reference data; i relativi controlli restano P11.

DEC-12 approvata: righe vuote conservate come record di zero byte, nessun
record extra dopo l’ultimo terminatore, BOM rifiutato. `--bom strip` non è
un’opzione della prima CLI. Sono conservati gli spazi del record.
