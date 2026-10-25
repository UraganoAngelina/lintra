# T-P14-04 — report delle differenze tra versioni

2026-10-25. Stato: sviluppo/verifiche in corso; nessuna CI ancora attestata.
Prerequisito14-03 verificato da80163/CI39574640180 Linux/MSVC; preflight e mandato
autonomo rinnovato in AGENTS. Importer14-01 fermo sul workbook regionale assente:
14-04 richiede solo14-03 e può procedere secondo Piano§14.2. Nessuna fonte
ARCH/DET/Piano o baseline modificata; nessuna dipendenza aggiunta.

## Contratto pubblico additivo

`diff_schemas(&RawFlowSchema, &RawFlowSchema) -> Result<SchemaDiff, SchemaDiffError>`
è tooling offline senza I/O, compiler, evaluator o storage. Entrambi i raw restano
immutati. Nessun collegamento dal validation loop, CLI o UI aggiunto. Il report
è descrittivo: non promette compatibilità normativa o migrabilità e non promuove
uno schema a compilato/verificato.

SchemaDiff contiene revisioni before/after e lista ordinata changes.
SchemaDiffRevision conserva flow_id/schema_version/regulatory_version e
Sha256Digest reale da schema_fingerprint già ratificato14-03. Non è l'hash dei
byte file originali: whitespace/ordine object e binding encoding runtime non
appartengono al confronto canonico. I nuovi DTO sono Serialize/Eq/Clone/Debug;
non sono eseguibili né alterano i tipi compiled non forgiabili.

SchemaChange contiene trace_id/field_id opzionali, due puntatori JSON separati
prima/dopo e SchemaChangeKind. Added conserva il valore after e solo after_path;
Removed conserva before e solo before_path; Modified conserva entrambi i valori
esatti e i puntatori nelle rispettive versioni. OrderChanged conserva l'ordine
completo degli ID nelle due liste, mentre i puntatori identificano le liste
stesse. Nessuna stringa di puntatore viene interpretata come ID di campo.

SchemaChangeKind è tagged per serializzazione machine-readable: added, removed,
modified, order_changed. Non contiene codici di ValidationError; non mescola
revisione schema con violazioni normative sui record.

SchemaDiffError distingue DifferentFlows, DuplicateTraceId e DuplicateFieldId,
con side Before/After e identificativi in payload. Gli errori Canonical e
Serialization conservano cause effettive con Error::source; Display è stabile.
Nessun errore fittizio o codice normativo inventato.

## Algoritmo e condizioni di correttezza

check_ids verifica prima entrambi i modelli, tracce globalmente e campi entro
ciascun trace. BTreeSet::insert=false causa l'errore tipizzato: nessun primo
candidato scelto e nessuna deduplicazione silenziosa. Campi con lo stesso ID in
tracciati differenti sono leciti. Altri problemi di lint non bloccano il diff:
una bozza con lunghezza zero o regola non supportata resta revisionabile.
Un flow_id diverso viene rifiutato prima del pairing: la funzione confronta
revisioni dello stesso flusso, non cerca corrispondenze fra flussi diversi.

revision copia gli identificativi dichiarati e richiama il fingerprint approvato.
La serializzazione to_value usa esattamente il modello raw e il suo contratto
sui default e i campi omessi, non il JSON sorgente testuale. Nessun trim, casefold,
normalizzazione Unicode o conversione floating point viene aggiunto.

child costruisce un puntatore aggiungendo un segmento: prima ~→~0, poi /→~1.
L'ordine evita di ri-escapare le tilde appena introdotte. Funziona anche con nomi
metadata contenenti slash/tilde e con stringhe ID non numeriche. Il puntatore
radice iniziale è vuoto; gli indici sono quelli delle rispettive liste originali.

Node/Context sono privati e distinguono il livello strutturale Flow/Trace/Field
rispetto a Other e gli ID del contesto. Soltanto traces del Flow e fields del
Trace sono liste identificate; un oggetto metadata chiamato fields o traces
resta JSON ordinario. Context mantiene la provenienza anche su proprietà annidate,
vincoli del field e metadata; cambiamenti di livello flow hanno ID assenti.

by_id riceve solo array prodotti dai DTO typed RawFlowSchema.traces e
RawRecordSchema.fields; ciascun elemento ha necessariamente id stringa. Le
expect/unwrap interne verificano questa invariante di serializzazione, non sono
un parser permissivo. Duplicati già rifiutati da check_ids. Nessun Value arbitrario
pubblico può entrare come nuova radice tipizzata del pairing.

identified costruisce BTreeMap ID→(indice,valore) per ciascun lato. Considera gli
ID condivisi nell'ordine dei due array e segnala OrderChanged soltanto quando
cambia il loro ordine relativo. Semplice aggiunta/rimozione non produce un falso
riordino; se riordino e aggiunta coesistono, il report conserva entrambi. Non
riordina gli schemi originali. Poi visita l'unione ID in ordine lessicografico,
creando i puntatori con gli indici veri. Rinomina di ID significa remove/add:
nessuna euristica di matching per nome, lunghezza o posizione. Il costo del
pairing è O(n log n), senza scansione quadratica dei candidati.

compare distingue quattro situazioni:
- solo before: emette una rimozione del sottoalbero intero;
- solo after: emette un'aggiunta del sottoalbero intero;
- entrambi diversi: object visita unione chiavi lessicografica; le due collezioni
  identificate delegano a identified, tutte le altre proprietà ricorrono senza
  significati normativi aggiuntivi. Array ordinari confrontano indici0..maxlen,
  conservando l'ordine di regole, join, vincoli, valori e operandi. Scalare diverso
  o cambio tipo emette Modified con i due valori esatti;
- valori uguali/nessuno: non emette cambiamenti, incluse collezioni vuote.

emit costruisce un DTO owned con ID/puntatori e valori clonati necessari al report.
La funzione pubblica controlla flow/ID, serializza i raw, calcola i fingerprint e
chiama compare dalla radice. Gli errori restituiscono Result senza report parziale.
L'ordine finale è deterministico: chiavi object lessicografiche; prima evento
ordine di ciascuna collezione, poi ID lessicografici; altre liste per indice.
L'ID "10" precede "2" come testo: non si assume una grammatica regionale.

Il diff opera sull'intero schema in memoria perché è tooling, non su record
sanitari. Nessuna promessa di memoria costante su schemi arbitrariamente grandi;
nessuna copia o lettura aggiuntiva nel validation loop. Non modifica la semantica
globale di compiler/engine, non introduce migrazioni o nuove regole regionali.

## Test e oracoli

Nove integration test iniziali in tests/schema_diff.rs:
- nessun cambiamento nel modello normativo con solo binding runtime diverso;
- field aggiunto/rimosso/modificato, niente falso riordino per indici traslati;
- trace e field riordinati, field modificato rimane abbinato al giusto trace/ID;
- trace rinominato=remove/add, ID field riutilizzati fra trace distinti;
- metadati, note originali, versione normativa e layout, numeri grandi e testo
  con zeri/spazi/Unicode preservati; puntatori escaped, inversione before/after;
- array ordinari confrontati per posizione, aggiunte finali distinte;
- errori tipizzati su entrambi i lati per ID duplicati, flow diversi; una bozza
  con geometria invalida è revisionabile e trace vuoti sono gestiti;
- ordine chiavi JSON irrilevante, eventi in ordine lessicografico riproducibile;
- confronto reale fra SIOC structural-minimal.1/.2: le quattro REL-* della
  ratifica sono aggiunte, policy revoking_act modificata, nessun field alterato.

assert_pointers verifica ogni valore dell'evento recuperandolo tramite pointer
nella vera serializzazione del lato corrispondente. Verifica gli OrderChanged
sugli ID effettivi dei due array. Gli expected non sono copiati dal diff runtime:
sono derivati dalle mutazioni dichiarate e dalla ratifica SIOC. I vecchi golden
restano invariati e saranno eseguiti dal gate globale.

Primo tentativo di compilazione test: accesso alla physical come struct anziché
enum e prestito concorrente nella costruzione della fixture duplicata. Corrette
solo le fixture usando serializzazione raw dichiarata e clone prima del push;
nessuna firma produttiva modificata per accomodare i test.

## Gate e handoff

Da registrare dopo esecuzione effettiva: fmt/check/clippy/test/release/GNU e CI
Linux/MSVC. Non dichiarare DONE, M7 o importer disponibile prima delle rispettive
evidenze. File: schema/diff.rs, mod.rs, lib.rs, tests/schema_diff.rs,
preflight/report/stato. Nuova API additiva; dipendenze e Cargo.lock invariati.

### Test globali eseguiti

2026-10-25: cargo test --offline --locked concluso positivamente,620 passati
in73 suite,0 fail,1 ignored (probe preesistente; memoria13-08 attiva).
Nove test mirati e clippy --all-targets -D warnings già positivi, fmt/check
positivi. Release/GNU ancora in corso al momento di questa registrazione.
33 hash degli artefatti13-08 e90 file protetti ricontrollati senza differenze.

### Gate locale completo e passaggio CI

fmt --all -- --check, check --all-targets, clippy --all-targets -- -D warnings,
test620/73 suite, build --release, check --all-targets --target Windows GNU
e git diff --check tutti passati con --offline --locked e CARGO_HOME isolata.
Nessuna nuova dipendenza. Native Linux/MSVC ancora da verificare sul commit
pubblicato. Lo stato resta IN_PROGRESS finché entrambi i job non sono positivi.

Durante il gate l'autore ha fornito schemas/xlsx/SIOC_tracciato_regionale_20260420-
definitivo_v05.xlsx, autorizzando la consultazione per14-01. Gli altri flussi
restano fuori scope. I nuovi source non vengono inclusi nel commit14-04;
il blocco precedente non è più assenza del workbook, resta da esaminare il mapping.
