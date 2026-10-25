# T-P14-01 — importer XLSX reale con mapping revisionato

2026-10-25. Sviluppo autorizzato indipendentemente dalla lettura MSVC14-04 ancora
pendente: il prerequisito atomico è3-01/G0, già soddisfatto, non14-04. Nessuna
quarta chiamata al vecchio gate. Il nuovo gate sarà di una consegna reale, con
regressioni14-04 incluse, non un dispatch per aggirare il limite precedente.
Nessun Superpowers, subagente o processo locale in background.

## Fonti e perimetro

ARCH§13, DET§143, Piano§4/9P14/14 e prompt14-01 letti. Workbook SIOC fornito
dall'autore e già confrontato nel preflight:135 campi, coordinate coincidenti con
il profilo ratificato,8 differenze di solo whitespace nelle note,130 formule con
cache numerica. Altri flussi forniti dall'autore non consultati/implementati.

Sorgente locale912412 byte, SHA-256
6be8ae60a2c236e5a25b731365b42f8e7bca849066d68747abfc78b92ce273e3.
Non è fixture CI né contenuto di questo commit; nessun documento originale o
baseline normativa modificato. Nuovo profilo dati in docs/schema-import copre
le celle del workbook e riferisce structural-minimal.2, conservata invariata.

L'XLSX non descrive da solo DSL, eccezioni di missing/alignment, PK/join/identity,
capability, reference/persistenza e encoding. L'importer richiede quindi un
blueprint RawFlowSchema revisionato, oltre al mapping delle celle. Non inventa
questi contratti, non traduce il testo delle note in regole e non restituisce
il blueprint senza leggere la fonte: ID/token sono verificati, nomi/coordinate/
note sono letti realmente, annotazioni e hash derivano dai byte del workbook.

## Contratto pubblico e strutture

SchemaImporter::import(&Path)->Result<RawFlowSchema,ImportError> segue DET§143.
XlsxSchemaImporter::new(blueprint,profile) restituisce un errore se i mapping sono
ambigui/incompleti. Blueprint/profile sono posseduti e privati: dopo costruzione
non possono essere alterati da un altro chiamante. Ogni import produce un nuovo
raw senza mutare il blueprint, pronto per lint/revisione/compiler. Nessun tipo
compiled viene costruito o aggirato. Nessun uso nel validation loop, engine o
application, nessuna CLI anticipata.

XlsxImportProfile è dati Serialize/Deserialize con deny_unknown_fields: flow_id,
versione blueprint, foglio/header, colonne, sezioni/binding, policy formule e fogli
annotazione espliciti. Non deduce versioni dal nome del file o latest.
XlsxColumn contiene numero1-based e header esatto; XlsxColumns nomina8 colonne
ID/chiave/descrizione/lunghezza/inizio/tipo/obbligatorietà/note. all restituisce
riferimenti alle8 colonne per controlli comuni, senza mappa duplicata implicita.

XlsxTraceBlock identifica trace, cella intestazione e field row ordinati.
XlsxFieldRow contiene riga, ID, token tipo/obbligatorietà/chiave attesi e tipo
canonical del blueprint. I token sono binding revisionati per field, non alias
indovinati. Questo consente D/Data/A o tipo vuoto4.5 senza introdurre Alphabetic,
riclassificare campi per nome, modificare presence NBB o generare PK dal marcatore
X. I marcatori X sono provenienza; le PK normative restano quelle ratificate.

CoordinateFormulaPolicy è un'unica scelta tipizzata: Reject oppure UseCachedValues.
La seconda legge il valore cached solo delle coordinate e conserva il testo
formula, senza promettere ricalcolo o cache fresca. Nessuna macro eseguita.

ImportError separa Io, Workbook, Canonical con cause effettive Error::source da
InvalidProfile, Cell con foglio/riga/colonna/motivo, UnexpectedRow e MacroWorkbook.
Non usa ValidationError o codici di errore normativi. Display non stampa valori
originali o path dell'host; il payload tecnico rimane ispezionabile dal chiamante.

## Funzioni, condizioni e correttezza

physical_type esegue match esaustivo sui4 tipi raw: produce il tag canonical da
confrontare col binding, non interpreta A/D/testo naturale. valid_row limita
coordinate profilo alle righe Excel1..1048576. invalid centralizza solo la
costruzione degli errori del profilo, senza nascondere cause filesystem/parser.

new controlla flow/versione, foglio/header e sezioni non vuoti, collisioni con
namespace di provenienza xlsx_import e copertura trace completa. Le colonne
richiedono header esatto,1..16384 e unicità. I blocchi seguono ordine blueprint e
worksheet; trace/field ID sono esatti e unici nel proprio scope. Campi con lo
stesso ID in trace diversi sono ammessi. Non si scinde un ID sul punto né lo si
converte in numero:4.19.1 rimane integro. Le righe dei binding sono tutte distinte,
positive e strettamente ordinate tra le sezioni. La cardinalità dei binding deve
coprire ogni field: zip non può troncare silenziosamente uno dei lati. Il tipo
physical del blueprint deve coincidere con quello revisionato nel profilo;
i token opzionali Some("") sono rifiutati per evitare doppie rappresentazioni di
assenza. Le annotazioni hanno fogli espliciti e unici. Collisioni field metadata
xlsx_source sono rifiutate: non si sovrascrive una provenienza preesistente.
Il blueprint nuovo può mantenere il binding runtime esplicito del chiamante;
esso resta escluso dal fingerprint come nel contratto canonical14-03.

Sheet raggruppa Range di valori/formule dello stesso foglio. error aggiunge
contesto strutturale. formula restituisce soltanto espressioni non vuote.
text rifiuta formule e dati non String: ID/testo/zeri iniziali non vengono
ricostruiti da float o coercizioni. Empty/stringa vuota sono assenza; nessun trim
su stringhe non vuote. required_text rifiuta l'assenza, expect verifica il token
opzionale esatto. Note vuote rimangono stringa vuota nel risultato.

coordinate applica prima la policy formule; accetta Int convertibile in u32 o
Float finito, integralmente rappresentabile e nel rangeu32. Questi interi sono
esattamente rappresentabili inf64; nessun importo sanitario passa per questo
ramo. Rifiuta testo numerico, frazioni, negativi, overflow, errori/assenze o cache
non numeriche. Lo zero può essere rappresentato nel raw: non viene dichiarato
valido, sarà il linter a rifiutare start/length zero. Non esegue formule.

absolute converte indici relativi dei Range in coordinate assolute1-based con
addizioni/conversioni checked e limiti Excel: le celle iniziali vuote non
traslano la provenienza e coordinate invalide non provocano overflow.

import legge fs::read una sola volta, costruisce Xlsx sopra Cursor degli stessi
byte e rifiuta progetti VBA tramite il lettore. Il digest attesta i byte letti
anche se il file viene modificato dopo: non si promette un blocco atomico del
filesystem. Header e sezioni devono coincidere esattamente, inclusi gli spazi.
Gli insiemi id_rows/geometry_rows coprono header, heading e field applicabili.
check_row visita valori e formule: un ID o una coordinata fuori mapping dopo
l'header causa UnexpectedRow, anche se è un field aggiunto in fondo. Stringhe
vuote non creano falsi field. Le formule senza cache non possono sparire dal
controllo delle righe, perché è visitato anche il Range delle espressioni.

Dopo le verifiche, il blueprint viene clonato. Per ogni field si verificano ID e
token, si leggono nome/start/length/note, si aggiornano le coordinate con match
esaustivo sui tipi raw. Alignment/lexical/date format/Decimal policy, record_length
dichiarata, presence/condizioni, vincoli, regole/PK/join/identity e persist policy
non cambiano. Una geometria incoerente rimane una bozza per lint: nessuna pretesa
che l'import garantisca validità o corregga automaticamente la lunghezza totale.

source_note e xlsx_source.notes_original contengono le note effettive; row/sheet,
token originali e formule sono conservati. La fonte precedente resta nel
blueprint originale, immutato e identificato dal suo fingerprint. I fogli
annotazione selezionati conservano stringhe con coordinate e spazi originali;
le relative formule sono rifiutate, non eseguite o interpretate come note.

source.xlsx_import conserva hash/dimensione byte, hash canonical del blueprint,
profilo completo, annotazioni, status requires_review e formula_cache_verified=false.
Non registra un hash atteso come se fosse il digest letto, non include path
assoluti, clock o segnali di approvazione normativa. La policy non trasforma raw
in trusted compiled e non cambia RunStatus/completeness della validazione.

## Profilo SIOC e riproduzione

Il profilo copre18/50/36/20/11 field nei5 trace, header riga5 e8 colonne reali.
Il generator stdlib Python verifica l'hash dell'originale, legge ZIP/XML, ricava
solo i binding e verifica ID/ordine/coordinate contro.2. Default confronta il
profilo; --write scrive soltanto il nuovo JSON, mai workbook, blueprint o baseline.
Lettura di shared strings conserva testo; l'ID con più punti è stringa. Le
sezioni danno il trace esplicito: il field4.30 non viene interpretato come trace4.
I nomi/tipo/obbligatorietà sorgente restano nella provenienza; niente nuove regole.
README del profilo spiega uso API e limiti. Nessun parser PDF o altri flussi.

## Dipendenze

calamine=0.36.1/default-features=false promossa da dev-only a tooling normale,
registrata in DEC-15 prima di Cargo. MIT/pure-Rust/MSRV1.88 già qualificata12-05.
Cargo.lock è byte-identico: nessun package/versione/checksum nuovo. I subtree
zip8.6/quick-xml0.41/encoding_rs e deflate zlib-rs restano pure-Rust. Writer XLSX
zip9/rust_xlsxwriter invariati; controllare i due target nei gate. Il lettore
normativo è offline tooling e non aggiunge letture XLSX nel validation loop.

## Test e qualificazione del file originale

12 integration sintetici: fixture XLSX vera con header/sezioni fedeli, ID con
zeri e riutilizzati tra trace, CRLF/Unicode/spazi delle note, dati distinti dal
blueprint, prove di geometria raw invalida demandata al compiler, source hash,
provenienza blueprint/formule, annotazioni, errori I/O/parser con causa, drift
header/ID/tipo/obbligatorietà/key, ID numerici non convertiti, field/coordinate
extra fuori mapping, profili incompleti/duplicati/out-of-range, policy Reject,
cache frazionaria/negativa/overflow/assente e formule in celle testuali.
Il test profilo completo verifica copertura135/costruttore anche in CI, senza
richiedere il workbook esterno. Gli expected derivano da valori espliciti delle
fixture e dall'hash originale già calcolato indipendentemente nel preflight.

Test manuale separato, esplicitamente ignored nella suite ordinaria perché la
fonte normativa non è fixture distribuita: eseguito positivamente con
LINTRA_SIOC_XLSX=/percorso/esplicito cargo test --test schema_importer
original_sioc_workbook -- --ignored --nocapture. Usa lo stesso adapter produttivo,
verifica135 field,130 formule,8 note originali aggiornate, hash912412 byte e
contratti immutati, annotazioni sugli importi conservate. Il compiler continua
a rifiutare il completo per PersistenceAuditV1: nessun requisito rimosso.

Due errori iniziali di test riguardavano API zip9.name fallibile e formatter
sha2array: adeguati alle versioni effettive. Due fixture malformate non erano state
prodotte dal writer come previsto: write_string("") non cancella una cella e
set_result("") scrive il default numerico. Corrette tramite mutazione ZIP/XML
controllata per vera cella/cache assente; nessuna modifica al parser per accettare
una cache mancante. Tutti12 test e qualificazione originale poi positivi.

## Gate, stato e handoff

Da registrare dopo effettiva conclusione. fmt/check/clippy/test/release/GNU e CI
Linux/MSVC richiesti. Baseline90 e33 hash13-08 ricontrollati invariati; workbook
utente e proposta reference newline preservati/non staged. Non dichiarare
14-01 o14-04 DONE prima delle evidenze native; avanzare comunque con attività
successive indipendenti se non emergono criticità, secondo mandato aggiornato.

### Prima esecuzione del gate globale

fmt/check/clippy passati, unit/integration test tutti positivi:620 passati in72
suite e2 ignored (probe preesistente e qualificazione workbook esterno, questa
ultima eseguita separatamente con successo). Il comando cargo test globale si
è fermato ai doctest con E0463: rustdoc non risolve serde/calamine negli artefatti
locali. Non attestare PASS di cargo test su questo output. Avviata verifica
doctest con CARGO_TARGET_DIR=/tmp/lintra-doc-p14-01 isolata, senza cancellare
artefatti esistenti o disabilitare doctest. Esito ancora da registrare.

### Isolamento doctest verificato

CARGO_TARGET_DIR=/tmp/lintra-doc-p14-01 cargo test --doc --offline --locked
concluso exit0:12 doctest passati. Le sorgenti non sono state modificate per
risolvere E0463. Per attestare anche un comando cargo test completo positivo,
avviato nuovo gate in /tmp/lintra-verify-p14-01: CARGO_INCREMENTAL=0 e
CARGO_PROFILE_DEV_DEBUG=0/CARGO_PROFILE_TEST_DEBUG=0 riducono i soli simboli e
lo spazio locale, senza disabilitare test/assertion. fmt/check/clippy/test/release/
GNU saranno registrati dopo esito effettivo; nessuna cancellazione del target
preesistente e nessun controllo ignorato.

### Gate locale completo verificato

Il gate in target isolato /tmp/lintra-verify-p14-01 è terminato exit0:
fmt/check-all-targets/clippy-all-targets -D warnings/cargo test/build-release/
check-all-targets Windows GNU/diff passati con --offline --locked.
632 test in74 suite,0 fallimenti,2 ignored (probe preesistente e workbook
esterno; quest'ultimo eseguito separatamente con successo). Il primo E0463
resta registrato come tale; il gate completo positivo usa artefatti nuovi,
CARGO_PROFILE_DEV_DEBUG=0/CARGO_PROFILE_TEST_DEBUG=0/CARGO_INCREMENTAL=0.
Nessun doctest disabilitato o sorgente modificata per aggirare l'errore locale.

La CI nativa sarà eseguita sul commit pubblicato con impostazioni standard
del workflow, includendo regressioni14-04. Native ancora non attestata.

Checkpoint aggiornato: l'autore chiede di completare la prossima task e fermarsi
con la percentuale. Identificato esplicitamente14-02 come checkpoint successivo,
previa verifica14-01, senza ulteriori task. Dopo interruzione /tmp è stato
ricreato: log/target temporanei precedenti non più disponibili, esiti storici
restano verificati dagli output effettivi e dal dossier, non inventati.

### Chiusura verificata — CI discendente40238421197

2026-10-25: sul commit7b8bd8bad5d67a2f538909b94f0561356336f091 entrambi i
job Linux121304841503 e MSVC121304841698 completed/success, letti direttamente.
14-01 e14-04 DONE: il codice/test14-04 sono invariati e inclusi nel gate della
nuova consegna reale14-01. La precedente CI40211195991 rimane storicamente
con MSVC non verificata; non è stata fatta una quarta chiamata a quel gate.
Nuovo gate: dispatch+GET, due API effettive≥60s; un GET prima respinto prima
del processo per review timeout, retry consentito poi riuscito.
632 test/74 suite locali e qualificazione workbook originale positivi.
Avanzamento87/14062,1%. Prosegue14-02, poi stop richiesto dall'autore.
