# Lintra

Motore generico e versionato per validare flussi sanitari fixed-width.
**Il motore conosce come validare; gli schemi conoscono cosa è valido.**

La libreria espone `ValidationService` per Trace/LocalOnly e Flow/Full, con
compiler incrementale, controlli locali, membership reference e validazione
cross-record a due passate. I record sono byte-oriented; Decimal nel formato 2
riguarda soltanto importi dichiarati esplicitamente. Report JSONL/XLSX/summary,
audit degli input, progress e cancellazione mantengono distinti errori normativi,
warning e completezza. Persistenza storica, lookup metadata/temporali e capacità
normative non ancora compilabili restano consegne future esplicite.
Il consuntivo verificato è in [IMPLEMENTATION_STATUS.md](IMPLEMENTATION_STATUS.md).

La CLI esegue `validate` per Trace/LocalOnly e `validate-flow` per Flow/Full;
produce report e un riepilogo testuale. `--version` mostra versione e commit.
`schema lint` verifica gli schemi con il compiler corrente. Ctrl+C richiede
la cancellazione cooperativa e restituisce 130, conservando i report definitivi
precedenti e rimuovendo le parti create dal run interrotto.

## Avvio con il binario distribuito

Scaricare ed estrarre l’archivio nativo dalla release privata autorizzata.
Non serve Rust sulla macchina che esegue il binario. `SYNTHETIC/v1` è un
esempio incorporato di collaudo: non è uno schema sanitario certificato.
Eseguire l’esempio in una cartella nuova contenente il solo eseguibile estratto,
senza directory dati esterne né variabili `LINTRA_SCHEMA_DIR` e
`LINTRA_REFERENCE_DATA_DIR`. Le due spaziature dopo `A` fanno parte del record.

Linux, dalla cartella dell’eseguibile:

<!-- lintra-quickstart:bash -->
```sh
./lintra --version
./lintra --help
printf '001A  \r\n' > input.txt
./lintra validate --flow SYNTHETIC --schema-version v1 --trace T1 --encoding utf8 --input input.txt --errors errors.jsonl --summary summary.json --xlsx report.xlsx
```
<!-- /lintra-quickstart -->

Windows PowerShell5.1 o successivo, dalla cartella di `lintra.exe`:

<!-- lintra-quickstart:powershell -->
```powershell
.\lintra.exe --version
.\lintra.exe --help
[System.IO.File]::WriteAllBytes((Join-Path $PWD 'input.txt'), [byte[]](48,48,49,65,32,32,13,10))
.\lintra.exe validate --flow SYNTHETIC --schema-version v1 --trace T1 --encoding utf8 --input input.txt --errors errors.jsonl --summary summary.json --xlsx report.xlsx
if ($LASTEXITCODE -ne 0) { throw 'Controllare stderr e summary prima di proseguire' }
```
<!-- /lintra-quickstart -->

Attesi: un record elaborato, zero errori, `errors.jsonl` vuoto e summary con
`validation_mode: local_only`, `result: VALID`, origine `embedded`.
`report.xlsx` contiene Riepilogo ed Errori e si apre direttamente in Excel.
L’esempio verifica solo i controlli locali del tracciato: leggere sempre scope,
completezza, warning e stati reference nel riepilogo. Non attesta conformità
normativa completa SIOC/CAD. Per più tracciati usare `validate-flow --help`
e una versione esplicita che dichiari le capacità realmente implementate.

### Codici di uscita

| Codice | Significato |
|---|---|
|0|Analisi completata senza violazioni nell’ambito richiesto; help/version sono anch’essi0.|
|1|Errore tecnico, argomenti errati o esecuzione non completata.|
|2|Analisi completata con violazioni normative.|
|130|Cancellazione cooperativa tramite Ctrl+C.|

Su Linux leggere `$?` subito dopo il comando; in PowerShell usare `$LASTEXITCODE`.
Un errore di sintassi non usa il codice2 di default di clap: Lintra restituisce1.
L’annullamento non equivale a report completo. I file definitivi precedenti sono
preservati durante l’interruzione; la pubblicazione dei report è atomica per
singolo file, non una transazione fra tutti i file. Un errore tecnico richiede
la consultazione di stderr e degli output effettivamente disponibili.

### Dati, encoding e report

La risoluzione seleziona una coppia completa di directory, in quest’ordine:

1. `--schema-dir PATH --reference-data-dir PATH`.
2. `LINTRA_SCHEMA_DIR` e `LINTRA_REFERENCE_DATA_DIR`.
3. `schemas/` e `reference-data/` accanto all’eseguibile, indipendentemente dalla directory di lavoro.
4. Snapshot incorporato, se compilato nel binario (incluso nelle release).

Una coppia incompleta o un bundle selezionato errato ferma il run: nessun mix
fra livelli né ripiego per singolo file. Flow, tracciato, versione ed encoding
sono espliciti; il catalogo non certifica gli schemi e non sceglie una versione
«più recente». Versione, hash e provenienza effettivi si leggono nel summary.
Per aggiornare i soli dati seguire il [runbook](docs/RUNBOOK_SCHEMA_UPDATE.md).

La CLI corrente accetta `--encoding utf8`; non converte automaticamente altri
encoding. I record sono posizionali in byte. LF e CRLF sono riconosciuti;
una riga vuota è un record di zero byte. Il terminatore finale non inventa un
record extra. Un BOM UTF8/UTF16/UTF32 causa errore tecnico: non viene rimosso
implicitamente. Conservare una copia dell’originale prima di modificarne i byte.

JSONL e summary usano UTF8 senza BOM; JSONL termina le righe con LF anche su
Windows. Preferire `--errors PATH` (alias `--output`) e `--summary PATH`:
**non usare `>` per ottenere i report**. In PowerShell5.1 il reindirizzamento
può produrre UTF16, e stdout della CLI è comunque un riepilogo testuale.
L’XLSX mantiene i campi testuali come testo e le intestazioni; usarlo per la
lettura in Excel. La CLI attuale espone JSONL/XLSX, senza un’opzione CSV fittizia.
I percorsi degli output devono essere distinti da input e bundle dati.

### Primo avvio Windows

Le release private correnti non hanno firma Authenticode, come deciso per l’uso
locale dell’autore e dei colleghi tecnici. Windows può mostrare «app non
riconosciuta»: per un archivio della provenienza prevista, scegliere «Ulteriori
informazioni → Esegui comunque», oppure usare Sblocca nelle Proprietà dello zip.
`SHA256SUMS` è disponibile nella release per confrontare i digest; la firma
andrà rivalutata se cambia il perimetro di distribuzione. Versione e commit
sono sempre consultabili con `--version`.

## Sviluppo locale

Eseguire dalla directory di questo README, con Rust/Cargo e i componenti
rustfmt e clippy disponibili:

```sh
cargo build --locked
cargo fmt --all -- --check
cargo check --workspace --all-targets --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
python3 scripts/check_workspace.py
cargo run --locked -- --help
```

Le dipendenze sono registrate in Cargo.toml/Cargo.lock e nel registro
[DEC-15](docs/DEPENDENCIES.md). Rust è fissato alla versione **1.99.1**;
rustup installa/seleziona il pin dichiarato e `Cargo.lock` è versionato.
La [guida di sviluppo](docs/DEVELOPMENT.md) descrive preparazione su Arch,
cross-check Windows GNU e conservazione dei byte delle fixture.
La CI T-P1-03 usa runner Linux e Windows MSVC reali; le verifiche locali
non attestano il superamento del gate multipiattaforma.

La ricetta release della CI applica remapping dei path, CRT statico e linker
deterministico su MSVC, poi confronta due compilazioni pulite:

```sh
cargo xtask build-release --target x86_64-unknown-linux-gnu --verify-reproducible --verify-artifact
# Sul runner Windows nativo:
cargo xtask build-release --target x86_64-pc-windows-msvc --verify-reproducible --verify-artifact
```

Il tooling è Rust, con parser ELF/PE Goblin e manifest/lock autonomi in xtask/. L'opzione
--offline usa esclusivamente dipendenze già acquisite. Le directory di verifica
sono nuove sotto target/release-profile e vengono conservate per diagnosi.
Il gate riguarda lo stesso checkout/toolchain/runner; non promette identità
tra OS diversi. Le release distribuibili restano prodotte esclusivamente in CI;
packaging, autosufficienza e pubblicazione richiedono i rispettivi gate P15.

Per verificare un artefatto esistente:

```sh
cargo xtask verify-artifact --target x86_64-unknown-linux-gnu --artifact PATH
```

Il verifier controlla import diretti, path host (UTF-8/UTF-16) e dependency tree
locked senza openssl-sys. GNU ammette soltanto i quattro SONAME ratificati in
[CNF-ARTIFACT-LINUX-01](docs/verification/CNF-ARTIFACT-LINUX-01-ratification.md).
Windows applica DLL di sistema/API set e rifiuta runtime CRT/GCC dinamici;
immagini con delay imports non ispezionabili vengono rifiutate esplicitamente.
L'opzione build --verify-artifact verifica anche il fallimento del comando reale
su una copia alterata con path host, senza modificare l'originale. Non certifica
la risoluzione delle librerie su ogni macchina destinataria o caricamenti via plugin.

Lo snapshot incorporato si abilita con `--features embedded-bundle`; la ricetta
`cargo xtask build-release` lo include sempre. Gli asset SYNTHETIC/v1 sono esempi
sintetici dichiarati, senza dataset ufficiali o certificazione normativa.
La CLI seleziona una coppia completa di root: opzioni esplicite, variabili
LINTRA_SCHEMA_DIR/LINTRA_REFERENCE_DATA_DIR, directory accanto all’eseguibile,
infine snapshot (solo se compilato). Errori o file mancanti nella sorgente scelta
non attivano fallback allo snapshot.

Per la validazione esterna `--bundle-manifest PATH` è opzionale. In sua assenza
si considera soltanto MANIFEST.json del parent comune delle root standard
schemas/ e reference-data/. Un manifest valido verifica lo schema selezionato e
la coerenza delle dichiarazioni reference; i byte dei dataset seguono ancora
Strict/Continue. Il summary JSON riporta bundle_source, bundle_version (null se
manifest esterno assente) e bundle_manifest_status. Present non significa che i
dataset siano Verified. Tooling DEC-18 e API generiche conservano i propri contratti.
Stato e gate nel [dossier T-P15-03](docs/verification/T-P15-03-embedded-bundle.md).

Il packager dati usa un manifest runtime esplicito e non sovrascrive gli archivi:

```sh
mkdir -p target/data-bundle
cargo xtask bundle-data --input bundles/embedded --output-dir target/data-bundle
```

Produce `schemas-<bundle_version>.zip` con `schemas/`, `reference-data/`, il
MANIFEST.json originale e SHA256SUMS (payload e manifest, escluso sé stesso).
Include soltanto i file dichiarati e verificati; rifiuta file mancanti/alterati,
symlink e output dentro la sorgente. ZIP Stored, timestamp e permessi fissi,
ordine stabile. L’esempio confeziona dati sintetici; non compila né certifica
normativamente gli schemi. Pubblicazione delle release nei workflow successivi.
Il test di accettazione del tooling richiede un binario già costruito:
`LINTRA_TEST_BIN=/percorso/lintra cargo test --manifest-path xtask/Cargo.toml --locked -- --include-ignored`.

Il workflow `Release executable` si avvia manualmente con versione identica a
Cargo.toml, su uno SHA che abbia superato la CI nativa. Destinazione approvata:
`UraganoAngelina/lintra-binaries` privata, con immutabilità GitHub abilitata.
Richiede nel sorgente il secret `LINTRA_RELEASE_TOKEN` dedicato a quella sola
destinazione (Contents write, Metadata read, scadenza90giorni); non riusa token
locali e non li passa alle build. Tag/release/draft esistenti fanno fallire il run.
Gli upload sono controllati in draft prima della pubblicazione immutabile;
errori lasciano la versione riservata, da diagnosticare senza sovrascriverla.
La prerelease privata0.1.1-rc.1 e i relativi asset sono stati verificati:
[stato T-P15-05](docs/verification/T-P15-05-executable-release.md).

## Struttura

- `src/lib.rs`: moduli privati `application`, `domain`, `schema`, `input`,
  `engine`, `context`, `persistence`, `reference_data`, `report`, `bundle` e `bundle_resolution`.
- `src/main.rs` e `src/cli/`: adapter CLI separato dalla libreria.
- `ui/`: membro workspace `lintra-ui`, adapter desktop distinto; bootstrap
  grafico verificato; Nuova analisi T-P16-03 in chiusura del gate nativo. [Stato e comandi](ui/README.md).
- `schemas/`, `reference-data/`: directory riservate ai bundle versionati;
  nessuno schema o dataset importato nello scaffold.
- `tests/{integration,golden,fixtures,support}/`: test sintetici e harness
  golden. I doctest verificano il confine pubblico della libreria;
  i test di piattaforma verificano I/O sintetico su percorsi con spazi.
- `docs/`: copie inalterate delle specifiche approvate.
- [IMPLEMENTATION_STATUS.md](IMPLEMENTATION_STATUS.md): consuntivo,
  verifiche reali, decisioni e handoff.

`context` ospita il contesto effimero della validazione a due passate;
`persistence` è il confine distinto della persistenza core generica.
La persistenza fisica resta da implementare nelle task PERSIST.

Il package `lintra` resta il membro predefinito: `cargo build` conserva la build
CLI, `--workspace` controlla anche l’adapter UI. `xtask` e il prototipo in
`spikes/ui-prototype` rimangono workspace indipendenti. Il guard delle dipendenze
verifica che Slint non sia raggiungibile dal core/CLI su ciascun runner nativo.

## Documentazione e ripresa del lavoro

Per la ripresa leggere [AGENTS.md](AGENTS.md),
[MASTER_AGENT_PROMPT](MASTER_AGENT_PROMPT.md), la frontiera di
[IMPLEMENTATION_STATUS](IMPLEMENTATION_STATUS.md), ultimo handoff e decisioni
pertinenti. Usare [indice operativo](ATOMIC_TASKS_INDEX.md),
[mappa dipendenze](TASK_DEPENDENCY_MAP.md) e prompt in TASK_PROMPTS/.
Consultare poi soltanto le sezioni pertinenti di ARCH/DET/ADR e del piano.

La [governance GOV-2026-10-26-01](docs/governance/README.md) rende permanenti
esecuzione con un solo agente, divieto di Superpowers/subagenti/delega,
autorizzazione a commit/push/CI e avanzamento automatico dopo i gate.
Errori ordinari si correggono autonomamente; decisioni sostanziali non ratificate
restano gate umani. Una CI pendente non soddisfa una dipendenza.

Il kit adiacente ../LINTRA_STARTER_KIT_FINAL_20261016/ resta intatto come fonte
storica/normativa, non è una dipendenza di build/runtime. docs/bootstrap,
vecchi piani e handoff conservano le istruzioni del tempo senza imporre stop
correnti. Le versioni operative dei master e dei prompt futuri/parziali sono
ora versionate nel repository. Nel piano è cambiato soltanto il protocollo §14;
ARCH/DET, norme, dipendenze e criteri di accettazione restano invariati.
Template di task/handoff/recovery in docs/governance/templates/.

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

Exit code: **0** analisi completata senza violazioni normative rilevate; **1** errore tecnico o di sintassi;
**2** analisi completata con violazioni; **130** cancellazione controllata.
`--help` e `--version` terminano con 0. Il risultato riguarda lo scope selezionato:
con completezza `partial`, assenza di violazioni non significa piena conformità.

Il parser richiede flow, schema-version, trace, input, errors (alias output),
summary ed encoding esplicito (`utf8`, unico profilo attualmente supportato).
Nessuna selezione implicita dell’ultima versione o di Full.

Risoluzione di bundle esterno per coppie complete, in ordine: opzioni
`--schema-dir`/`--reference-data-dir`; variabili `LINTRA_SCHEMA_DIR`/
`LINTRA_REFERENCE_DATA_DIR`; directory `schemas/`/`reference-data/` accanto
all’eseguibile. Una coppia incompleta o una directory scelta non valida produce
errore, senza ripiego su livelli inferiori. Nessuna ricerca in cwd/home.
Lo snapshot incorporato sarà introdotto in P15: senza bundle esterno il build
corrente fallisce esplicitamente. Le reference membership vengono preparate dal
provider filesystem: `--reference-policy continue` usa le evidenze disponibili,
con audit distinto per dataset verificati/degradati/indisponibili; `strict`
richiede preparazione verificata. Nessun dataset normativo viene inventato.

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
schema compilabile (anche con warning), 1 errore tecnico o di authoring (incluse
issue del compiler, JSON malformato/duplicato, identità diversa o sintassi CLI).
Il codice 2 resta riservato alle violazioni normative dei record.
Le diagnostiche schema sono oggetti JSON su stdout, una per riga nell'ordine
del compiler, seguite dall'esito; gli errori tecnici vanno su stderr.
I warning sono conservati anche se la compilazione riesce, senza duplicazioni.

Gli altri strumenti di authoring richiedono sorgenti esplicite:

```sh
lintra schema list --schema-dir ./bundle/schemas
lintra reference-data list --schema-dir ./bundle/schemas \
  --reference-data-dir ./bundle/reference-data --flow SYNTHETIC --schema-version v1
lintra schema import --input ./source.xlsx --blueprint ./blueprint.json \
  --profile ./profile.json --output ./draft.json
```

`schema list` enumera soltanto `root/flow/version/flow.json`, ordinando flow/version
per byte UTF-8 e conservando l'ordine dichiarato dei trace. Non compila gli schemi,
non seleziona latest e non installa cartelle di authoring `flow_schema.json`.
Errori sui candidati, identità incoerenti e symlink enumerati fanno fallire l'elenco.

`reference-data list` ispeziona soltanto le dipendenze dello schema selezionato,
con `continue_with_available_evidence`. Il JSON distingue `verified`, `degraded`
e `unavailable`, digest atteso/effettivo e cause diagnostiche; non espone le chiavi.
Exit0 significa inventario costruito, anche con dataset mancanti o degradati:
non garantisce che una validazione Strict o Full sia eseguibile. Root invalide,
dichiarazioni invalide e violazioni di boundary rimangono errori fatali.

`schema import` usa blueprint e profilo revisionati obbligatori; gli alias
opzionali sono un array `TypeAlias` JSON passato con `--type-aliases PATH`.
Le note non diventano regole. Errori di lint impediscono l'output; warning JSON
completi vanno su stderr. Il draft canonical viene pubblicato atomicamente in un
file nuovo, senza sovrascritture, e non è dichiarato compilato o installato.
Tutti i comandi tooling usano exit0 successo/exit1 errore tecnico o di authoring.

Ctrl+C viene intercettato durante `validate` e `validate-flow`, prima di elaborare input
e report. L'handler imposta il token: il servizio si ferma ai propri checkpoint
e il riepilogo stdout indica CANCELLED e incompletezza. Il JSONL/summary del run
annullato non sostituisce i definitivi. Un guasto tecnico durante la finalizzazione
resta exit 1. Dopo il checkpoint RunCompleted un segnale non riscrive l'esito già
fissato dal servizio. L'handler Ctrl+C non interrompe `schema lint`.

## Importi Decimal — formato 2

Il formato 1 resta invariato. Il formato 2 aggiunge campi dichiarati
esplicitamente `decimal`: punto al terzultimo byte, due cifre decimali,
nessun segno o separatore delle migliaia, zero ammesso. Il tipo conserva tutta
la larghezza fisica e non converte i dati con floating point.
Codici/range di compartecipazione restano Numeric/AN con gli stessi controlli:
nessuna classificazione per nome né migrazione degli schemi esistenti.
Esempio sintetico e contratto in [authoring](docs/SCHEMA_AUTHORING.md).


## Validazione Full — T-P13-07

```sh
lintra validate-flow --flow SYNTHETIC --schema-version v1 --encoding utf8 \
  --trace-input "S=./sorgente.txt" --trace-input "T=./target.txt" \
  --schema-dir ./bundle/schemas --reference-data-dir ./bundle/reference-data \
  --errors ./errors.jsonl --summary ./summary.json --xlsx ./errors.xlsx
```

L'ordine degli input a riga di comando è libero; elaborazione e report seguono
l'ordine dello schema. Trace indispensabili mancanti, duplicati o ignoti sono
errori di preflight. Tutti gli input sono protetti da collisioni con gli output,
compresi i nomi `.part`. I path possono contenere spazi e `=` dopo il separatore.

Gli eventuali namespace richiedono binding espliciti, per esempio
`--parameter org=001 --parameter year=2025`, soltanto se dichiarati nello schema.
Il tipo deriva dalla dichiarazione: `001` rimane una stringa per un parametro
stringa; gli interi sono esatti e vengono verificati contro il range dichiarato.
Parametri duplicati, ignoti, mancanti o fuori dominio vengono rifiutati.
La libreria offre `validate_with_parameters`; `validate` equivale a binding vuoti.
Full richiede scope Flow. Le factory storiche con nome `local` restano API
compatibili e possono comporre anche richieste Flow/Full.

Il summary Full conserva `full_validation`, `completeness`, copertura per trace,
verifiche PK e conteggi per regola: satisfied, violations, not_applicable, unknown
e skipped. Le cause inconcludenti sono aggregate con JoinId e motivazione;
non diventano errori JSONL né incrementano total_errors. I contatori misurano
valutazioni record/regola; cross_rules_declared conta invece definizioni distinte.
`NotApplicable` conserva il suo significato di scope e non equivale a Unknown.

SHA completo prova lettura fino a EOF; un indice con righe non proiettabili
resta parziale. Una ricerca negativa in tale indice non prova un parent assente.
Il riepilogo testuale e XLSX espongono la completezza; un run parziale concluso
regolarmente può pubblicare report definitivi dichiaratamente parziali.
Cancelled o errore tecnico mantengono le regole di rollback dei report.
Nessuno snapshot storico viene aggiornato durante queste validazioni.


Il [profilo SIOC referenziale derivato .2](schemas/sioc/2026-04-20.structural-minimal.2/README.md)
registra gli obblighi strutturali espliciti autorizzati e conserva le capability
ancora mancanti. Il [golden referenziale](tests/golden/cross-trace/sioc-referential/v1/README.md)
esercita un componente operativo distinto con dati sintetici e ledger delle
esclusioni: non certifica il SIOC completo né sostituisce il profilo .1/FROZEN.
