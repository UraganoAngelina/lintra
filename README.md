# Lintra

Motore generico e versionato per validare flussi sanitari fixed-width.
**Il motore conosce come validare; gli schemi conoscono cosa è valido.**

La libreria espone `ValidationService` e una factory filesystem LocalOnly,
con compiler incrementale, validazione strutturale e controlli locali supportati
(presenza base, Numeric/AN, date DDMMYYYY, AllowedValues e range interi inclusivi),
report JSONL/summary, audit degli input, progress e cancellazione. Full,
reference data e persistenza completa rimangono consegne future esplicite.
Il consuntivo verificato è in [IMPLEMENTATION_STATUS.md](IMPLEMENTATION_STATUS.md).

La CLI esegue `validate` nel perimetro Trace/LocalOnly: produce errori JSONL,
summary JSON e riepilogo testuale, mantenendo distinti gli errori tecnici dalle
violazioni locali. `--version` mostra versione e commit di build. Full resta futuro.
`schema lint` verifica gli schemi con il compiler corrente. Ctrl+C richiede
la cancellazione cooperativa e restituisce 130, conservando i report definitivi
precedenti e rimuovendo le parti create dal run interrotto.

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

## Contratto CLI — T-P8-03

Exit code: **0** analisi locale valida; **1** errore tecnico o di sintassi;
**2** analisi completata con violazioni; **130** cancellazione controllata.
`--help` e `--version` terminano con 0. Il risultato valido riguarda soltanto
i controlli LocalOnly supportati, senza dichiarare conformità globale.

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

Esempio con bundle esterno e percorsi espliciti:

```sh
lintra validate --flow SYNTHETIC --schema-version v1 --trace T1 \
  --input "input data.txt" --encoding utf8 \
  --schema-dir ./bundle/schemas --reference-data-dir ./bundle/reference-data \
  --output ./errors.jsonl --summary ./summary.json
```

Lo schema selezionato si trova in `schemas/<flow>/<version>/flow.json`.
Preferire `--output` (alias `--errors`) per il JSONL: il reindirizzamento `>`
di PowerShell 5.1 può produrre UTF-16. JSONL e summary sono UTF-8 senza BOM;
il riepilogo testuale va su stdout, gli eventi applicativi strutturati su stderr.
I report passano da file `.part` esclusivi e sono pubblicati solo dopo un run
completo: gli esiti invalidi sono completi e vengono pubblicati; guasti tecnici
prima della pubblicazione eliminano i file provvisori creati dalla CLI.
File `.part` preesistenti restano intatti e causano errore.
L'atomicità è per singolo file: un errore di rename del secondo report può
lasciare il primo già pubblicato; l'exit code resta 1. I percorsi di output
non possono coincidere fra loro, con input, bundle o file provvisori del run.

Il commit è acquisito al build tramite Git, senza dipendenza Git a runtime;
`LINTRA_BUILD_COMMIT` può fornire un hash completo per build da archivio.
Senza provenienza disponibile viene dichiarato `commit unavailable`.
Le build locali restano strumenti di sviluppo, non artefatti di release.

## Verifica degli schemi

```sh
lintra schema lint --schema-dir ./bundle/schemas \
  --flow SYNTHETIC --schema-version v1 --encoding utf8
```

La selezione è esplicita e usa lo stesso repository/registry/compiler della
validazione. Il comando non richiede input o directory reference data e non
attiva provider mancanti. L'esito valido riguarda il sottoinsieme supportato dal
compiler attuale; non certifica completezza normativa. Gli exit code sono 0
schema compilabile, 2 issue del compiler, 1 errore tecnico (inclusi caricamento,
JSON malformato/duplicato, identità diversa o sintassi CLI).
Le diagnostiche schema sono oggetti JSON su stdout, una per riga nell'ordine
del compiler, seguite dall'esito; gli errori tecnici vanno su stderr.

Ctrl+C viene intercettato soltanto durante `validate`, prima di elaborare input
e report. L'handler imposta il token: il servizio si ferma ai propri checkpoint
e il riepilogo stdout indica CANCELLED e incompletezza. Il JSONL/summary del run
annullato non sostituisce i definitivi. Un guasto tecnico durante la finalizzazione
resta exit 1. Dopo il checkpoint RunCompleted un segnale non riscrive l'esito già
fissato dal servizio. L'handler Ctrl+C non interrompe `schema lint`.
