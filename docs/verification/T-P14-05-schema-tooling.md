# T-P14-05 — Cataloghi e CLI schema tooling

2026-10-26. Implementazione autorizzata dalla ratifica DEC-18. Stato iniziale
IN_PROGRESS: i risultati dei gate vengono aggiunti soltanto dopo esecuzione.
Nessun Superpowers, subagente o processo locale in background.

## Contratto e prerequisiti

T-P14-01 e T-P8-04 DONE; DEC-18 risolta con
[ratifica vincolante](DEC-18-ratification.md). ARCH§30/DET§10/63 e prompt14-05
consultati. Nessuna modifica a fonti architetturali/normative, piano o grammatica.
STOP dopo questa task e gate/handoff; nessuna task successiva autorizzata.

## Catalogo: tipi, condizioni e correttezza

SchemaCatalog è porta Send+Sync indipendente da load_raw. SchemaCatalogEntry
contiene chiave esplicita e trace dichiarati. SchemaCatalogError distingue
I/O con path/source, componente non portabile, symlink, errore di load e trace
non validi. Display descrive l'operazione e Error::source conserva la causa reale.
Conversione From incapsula SchemaLoadError senza trasformarlo in errore normativo.

Il modulo catalog è figlio privato di repository: può riusare root, mappa e
controlli portabili senza rendere mutabili i dati del repository. Le esportazioni
pubbliche sono limitate a trait/entry/error. Nessun compiler nel percorso listing.

entry controlla con HashSet che ogni trace ID sia non vuoto e unico, poi conserva
l'ordine dell'array originale. Non usa l'iterazione del set per produrre output.
In-memory ordina gli schemi prima di costruire entry: anche gli errori seguono
flow/version, senza ordine casuale HashMap. Le stringhe mantengono byte e case.
La costruzione delle entry prende RawFlowSchema in prestito e copia soltanto
chiave/ID trace: il catalogo in memoria non clona campi, regole o interi schemi.

Filesystem canonicalizza la root esplicita e children raccoglie/ordina i nomi
prima di ispezionarli. Nomi UTF-8 validi sono confrontati esattamente; i nomi
nativi invalidi sono ordinati per rappresentazione diagnostica con spareggio
nativo, poi rifiutati se directory. Nessuna conversione lossy per costruire chiavi.
metadata rifiuta symlink nei livelli enumerati; component richiede un nome UTF-8
non vuoto con le stesse restrizioni portabili di schema_path. I file estranei ai
due livelli sono ignorati; le directory sono visitate soltanto a profondità2.

Assenza del candidato flow.json non è schema installato: skip esplicito, non
conversione di flow_schema.json. Un flow.json presente deve essere file regolare,
mai directory/FIFO/dispositivo; quindi il catalogo non tenta letture bloccanti di
oggetti speciali. Symlink, errori diversi da NotFound e JSON invalido sono fatali.
load_raw verifica la chiave contro il percorso e il parser rifiuta proprietà
duplicate anche nei metadati. entries viene restituito solo a scansione conclusa.
Un errore successivo non produce un catalogo parziale di successo.

La root può risolvere attraverso un symlink scelto dal chiamante; i livelli
enumerati no. Controlli su albero stabile: nessuna garanzia handle-relative
contro mutazioni ostili concorrenti. Il layout authoring resta distinto dal runtime.

read_raw_schema_file è acquisizione di un file blueprint esplicito; usa lo stesso
parse_raw_schema estratto dal loader (UniqueJson più deserializzazione tipizzata).
Il load originale mantiene NotFound, chiave e percorso immutati. Il nuovo helper
non promette selezione, lint o compilabilità e non bypassa il compiler nei run.

## Reference dichiarate ed evidenze reali

compile_reference_dependencies è wrapper pubblico del lowering già usato dal
compiler. ReferenceDeclarationError conserva codice SchemaIssueCode e messaggio:
non inventa un diverso formato o errori normativi dei record. Sono verificati
ID/versioni/path/digest/duplicati e metadata unsupported come prima; nessun
CompiledFlowSchema può essere ottenuto tramite questa API.

ReferenceListArgs contiene entrambe le root e SchemaKey, tutte esplicite.
directory rifiuta path vuoti e apre read_dir per verificare accesso senza
inventariare dataset estranei. Anche con zero dipendenze root invalide falliscono.
Il raw è caricato per chiave; solo reference_data viene abbassato. L'intero raw
può richiedere capability future senza impedire questo inventario diagnostico.

Il provider reale prepara con ContinueWithAvailableEvidence. Per ogni binding
si associa la DatasetEvidence tramite la sua DatasetKey, poi si ordina la coppia
dichiarazione/evidenza per dataset ID/versione/declaration ID. Non si zip-ordina
separatamente hash e identità. L'expect esprime l'invariante del provider concreto
che ha appena preparato un'evidenza per ciascuna dipendenza validata.

diagnostic traduce esaustivamente DatasetAnomaly negli stessi diagnostici tipizzati
di audit del dominio: I/O con operation/kind, formato con issue, integrità e identità
distinte. Non stampa chiavi, contenuti file o messaggi arbitrari degli errori I/O.
Availability e digest sono letti dagli accessor dell'evidenza, mai ricalcolati o
promossi dalla CLI. L'identità effettivamente dichiarata è mantenuta se disponibile.

Provider/preparer/load e Strict restano invariati. Hash mismatch con formato e
identità affidabili resta Degraded; JSON non interpretabile resta Unavailable,
anche se i digest coincidono. Nessun recupero selettivo aggiunto. Boundary/path
invalidi rimangono fatali e precedono stdout. Dataset mancanti o malformati
classificabili non impediscono di conoscere lo stato delle altre dipendenze.

## Import e output

ImportArgs contiene XLSX, blueprint, profilo, output e alias opzionale. Il parser
clap richiede gli input; percorsi sempre PathBuf nativi. Non ci sono default di
flusso/versione/root o provider simulati. CliCommand distingue le operazioni e
run converte solo gli esiti di validate in codici RunStatus: tooling usa0/1.

schema_import rifiuta qualsiasi destinazione esistente con symlink_metadata:
include collisioni dirette, symlink anche dangling, hard link e directory. Nessun
input è aperto in scrittura. Blueprint viene acquisito con parser condiviso;
profilo/alias usano DTO reali; normalize_import_profile trasforma solo alias
espliciti. Senza alias il costruttore richiede già tipi canonical. XlsxSchemaImporter
verifica workbook/binding e conserva note/hash/formule secondo14-01/02.

lint_raw_schema eseguito sul raw importato emette tutti i diagnostici JSON su
stderr. Qualsiasi Error impedisce la pubblicazione; Warning da solo permette
il draft e resta visibile, senza trasformare note in regole. Non si compila il
draft: le capability future saranno ancora rifiutate dalla compilazione reale.

canonical_schema_bytes serializza prima dell'apertura del temporaneo. NamedTempFile
è creato nella directory di destinazione, quindi sullo stesso filesystem; bytes
completi vengono scritti, flush e sync_all verificati, poi persist_noclobber
pubblica senza sostituire una destinazione comparsa nel frattempo. Un file
definitivo non viene esposto con contenuto parziale. Errori mantengono la causa
tecnica e il temporaneo resta gestito da RAII; nessuna promessa di durabilità
directory/crash o protezione contro mutazioni ostili dell'albero.

emit costruisce prima l'intero JSON in memoria, aggiunge LF e soltanto dopo
scrive/flush su stdout. Errori di enumerazione non lasciano prefissi di catalogo.
Una pipe che si chiude durante write può ricevere un prefisso: l'errore è
propagato, non viene promessa atomicità stdout. Non esistono regole normative
nel nuovo adapter e il core non importa clap.

## Warning Registry e compatibilità

ResolvedSchema conserva una lista privata di warning della stessa snapshot raw
caricata e binding encoding scelto. warnings() è una vista read-only. Il lint
viene valutato in preflight prima del compiler, senza seconde letture file;
il compiler rimane autorità per l'eseguibilità e restituisce le sue issue su errore.
Non si cambia il payload compilato né la semantica dei run. Il costo aggiunto è
un passaggio di lint per risoluzione, mai per record o lookup.

schema_lint stampa i warning su successo una sola volta; su errore stampa soltanto
la sequenza issue del compiler, quindi non duplica la lista raccolta. issue_json
è condiviso con import: escaping JSON, severity/codice/flow/trace/field/message/
source_note conservati. INVALID compiler ora exit1 come ratificato da DEC-18,
prima exit2; README e aspettative CLI aggiornati intenzionalmente. Validate con
violazioni continua a usare2. Il flusso Registry/compiler resta reale.

## Test e risultati

Nuovi test catalogo e CLI usano fixture sintetiche filesystem/XLSX e il binario
reale. Helpers raw/install costruiscono identità esplicite; Fixture inventario
calcola digest con SHA-256 dei byte scritti, dataset contiene una chiave sentinella
per provare che non venga esposta. success/failure controllano codice e confini
stdout. Le prove Strict sul medesimo provider impediscono di confondere exit0
informativo con dataset utilizzabili in Strict.

Test import riusano workbook/blueprint/profilo già qualificati: confronto raw con
importer diretto, bytes canonical, note esatte, input identici e nessun temporaneo
residuo nei fallimenti ordinari. Alias non mappati falliscono; domini disgiunti
Required verificano il rifiuto lint; output già esistente e collisioni non mutano
gli input. Test stdout usa un writer BrokenPipe; un candidato invalido prova
che non venga scritto alcun JSON. Symlink e permessi verificati su Unix;
se un processo privilegiato ignora i permission bits, la limitazione è esplicita.

Prima compilazione test: digest sha2 0.11 non implementa LowerHex sull'array;
helper corretto usando due cifre hex per byte, nessun codice produzione cambiato.
Prima prova lint Error: fixture ereditava presence optional e produceva correttamente
Warning; resa Required senza forbidden_if, senza cambiare severity del linter.
Test mirati successivi positivi; gate globale/nativo da registrare qui sotto.

### Qualificazione manuale sul workbook regionale fornito

Percorso CLI reale con SIOC_tracciato_regionale_20260420-definitivo_v05.xlsx,
blueprint structural-minimal.2 e profilo revisionato: import exit0,135 campi,
130 formule cached e135 warning note da revisionare. Hash input invariato
6be8ae60a2c236e5a25b731365b42f8e7bca849066d68747abfc78b92ce273e3.
Physical/presence/constraints, regole, join e PK confrontati col blueprint e
identici. Installazione soltanto temporanea per prova: catalogo exit0; schema lint
del completo importato continua a restituire1/UnsupportedConstruct. Nessuna
promozione delle capability future. Input invariato e draft temporaneo rimosso.
Evidenze in target/verification-p14-05/T-P14-05-SIOC-qualification.json e
T-P14-05-SIOC-warnings.jsonl; nessun workbook nuovo reso prerequisito della CI.

### Gate locale definitivo

Eseguita sequenza con exit0 per ogni comando: cargo fmt --all -- --check;
cargo check --all-targets --offline --locked; cargo clippy --all-targets
--offline --locked -- -D warnings; cargo test --offline --locked; cargo build
--release --offline --locked; cargo check --target x86_64-pc-windows-gnu
--offline --locked.671 test/77 suite,0 fail/2ignored, inclusi9 doctest.
Log per comando in target/verification-p14-05/T-P14-05-{nome}.log.
CARGO_HOME=/tmp/lintra-cargo-home, CARGO_TARGET_DIR=target/verification-p14-05,
CARGO_PROFILE_DEV_DEBUG=0/CARGO_PROFILE_TEST_DEBUG=0/CARGO_INCREMENTAL=0.
Primo clippy richiedeva literal ottali per i permission bits dei test: corretti
in0o0, senza allow. Dopo revisione eliminato clone integrale dei raw dal catalogo
in-memory e ripetuta l'intera sequenza sul codice definitivo, tutti exit0.

Hash13-08 verificati:3 sorgenti/33 artefatti invariati. Git diff conferma engine,
application, provider, schemi, norme, ARCH/DET/piano e Cargo.toml/lock invariati.
Ricerca delle dipendenze conferma clap nel solo adapter CLI. Gate nativo ancora
da eseguire sul codice pubblicato: T-P14-05 IN_PROGRESS, M7 non dichiarata.

## Handoff prima della pubblicazione

- HEAD iniziale ad0d66c, branch work/p1-foundations; commit della consegna da
  registrare dopo creazione/pubblicazione autorizzata. Nessun branch nuovo.
- File: catalog/repository/lowering/export, Registry warnings, args/dispatch/
  tooling/lint CLI, test catalog/inventory/import/lint/parser, README, ratifica,
  proposta storica, matrice append-only, AGENTS/stato/preflight e questo report.
- Residuo14-05: soltanto pubblicazione/CI Linux/MSVC e handoff di chiusura,
  salvo eventuali regressioni native. Nessun prerequisito contrattuale pendente.
- Rinvii invariati: embedded/release/P15, UI, persistente storico, dataset
  ufficiali mancanti/metadata-tempo, Full SIOC non supportato e task fuori scope.
- Next: verificare nativi, poi STOP. Eventuale prossima task richiede nuova
  autorizzazione, anche se il piano ne indica già una candidata.

### Pubblicazione e avvio gate nativo

Commit174272c6db7bd5e4ac55afa4f6ea8bb8ef289cd1 pubblicato su
UraganoAngelina/lintra, branch work/p1-foundations. Dispatch ci.yml accettato
2026-10-26 circa09:00:32 UTC, prima chiamata Actions del gate. Accesso GitHub
verificato mediante autenticazione reale; nessun token registrato nei log.
Run/job/esiti ancora da leggere; nessun DONE/M7 sulla sola pubblicazione.

Seconda chiamata Actions alle09:02:39 UTC: run
[40296287288](https://github.com/UraganoAngelina/lintra/actions/runs/40296287288)
con head_sha esatto174272c, in_progress/conclusion null. Ultima chiamata
riservata alla lettura dei job dopo attesa; nessun risultato nativo presunto.

### Primo gate nativo fallito, non chiuso

Terza chiamata circa09:12:59 UTC: Linux121483043969 completed/success,
MSVC121483043646 completed/failure nello step Test; fmt e Clippy success,
release skipped. JSON completo job/step conservato in
target/verification-p14-05/T-P14-05-native-jobs.json. T-P14-05 IN_PROGRESS,
M7 non raggiunta. Esaurite le tre chiamate, chiesta eccezione all'autore per
recuperare il log diagnostico senza violare il limite registrato in AGENTS.

Riesame indipendente: il test di ordine del primo candidato confrontava il path
canonical del catalogo con quello originale tempfile. Su Windows canonicalize
usa sintassi extended-length, secondo la
[documentazione std 1.99.1](https://doc.rust-lang.org/std/fs/fn.canonicalize.html).
L'atteso usa ora fs::canonicalize del preciso candidato A: continua a provare
A prima di B e nessuna omissione, senza cambiare il catalogo o il suo contratto.
Il log resta necessario per confermare che questo sia il fallimento nativo
osservato e individuare eventuali ulteriori cause. Nessun esito MSVC presunto.

### Correzione portabilità della fixture

Modificato soltanto tests/schema_catalog.rs; nessuna variazione del codice
produttivo, delle fixture normative o dell'ordinamento atteso A/B. Prova mirata
11/11 pass, poi fmt/clippy-all-targets -D warnings/cargo test tutti exit0,
671 test/77 suite/0 fail/2ignored. Log T-P14-05-portability-{nome}.log.
Release/GNU già qualificati sullo stesso codice produttivo invariato.
Nuova consegna reale della correzione autorizzata nel workflow: pubblicare e
avviare un nuovo gate, con budget proprio di tre chiamate distanziate≥60s.
Nessuna quarta richiesta al vecchio gate; l'eccezione per il log resta pendente.
