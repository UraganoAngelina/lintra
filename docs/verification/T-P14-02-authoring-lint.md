# T-P14-02 — normalizzazione esplicita e lint avanzato

2026-10-25. Checkpoint dell'autore: completare questa task e fermarsi con la
percentuale verificata. Prerequisiti14-01/3-04 verificati. CI14-01 40238421197 su
7b8bd8b Linux121304841503 e MSVC121304841698 completed/success; la stessa evidenza
qualifica codice/test invariati14-04.87/140 DONE prima di14-02. Nessun altra task
avviata, nessuna modifica a baseline/documenti sorgente/grammatica o dipendenze.

## Contratto e normalizzazione

ARCH§8/14, DET§142 e SCHEMA_AUTHORING§Import/mapping già approvati. L'esempio
architetturale A→Alphabetic non è un mapping globale: l'authoring ratificato
specifica A SIOC→alphanumeric. Le nuove API sono generiche e ricevono alias
espliciti dal chiamante. Il nuovo JSON dati SIOC riporta soltanto AN/A/N/D/Data
nei significati già approvati. Nessun T CAD o altra capacità di flusso aggiunta.

schema/authoring.rs è tooling puro separato dall'I/O importer; dipende da raw e
profilo14-01, non da engine/application/CLI. Linter usa i suoi DTO, non esegue
import o letture filesystem. Gli export sono API additive, compiled non forgiabili
invariati. Nuovi tipi Serialize/Deserialize deny_unknown_fields ove applicabile,
nessuna modifica della grammatica authoring1/2 o dei source normativi.

ImportedPhysicalType contiene solo i4 tipi già supportati, con tag canonical
Numeric/Alphanumeric/Date/Decimal. tag e canonical sono match esaustivi su tali
nomi, non su note/field_id/flow_id. TypeAlias collega un alias testuale esplicito
a un tipo; i dati SIOC sono fuori dal codice. Nessuna deduzione per nome Importo
o Compartecipazione e nessuna conversione di un blueprint esistente.

normalize_import_profile valida una tabella di alias con BTreeMap: rifiuta nome
vuoto, duplicato o ridefinizione di un tag canonical verso un tipo diverso.
I nomi canonical sono identità; gli altri token richiedono un mapping esatto,
senza trim, casefold o normalizzazione Unicode. Unknown genera UnmappedAlias
con trace/field/token; non restituisce una trasformazione parziale.

Clona il profilo e visita trace/field in ordine dichiarato; sostituisce soltanto
reviewed_physical_type, raccogliendo TypeNormalization con identificativi e
before/after. Source_type originale, header, ID, coordinate, note e blueprint
restano intatti. NormalizedImportProfile contiene profilo e lista cambiamenti;
il costruttore14-01 continua a verificare il tipo contro quello del blueprint.
Un alias Decimal non riclassifica automaticamente un campo Numeric o AN: il
blueprint deve già dichiarare Decimal nel formato supportato dal compiler.

SchemaAuthoringError separa InvalidAlias, UnmappedAlias e InvalidNoteReview con
payload/cause di authoring, senza ValidationError o codici normativi. È puro:
nessun errore I/O nascosto e nessuna nuova dipendenza.

## Note e revisione esplicita

ImportedNoteDisposition è Metadata con reason non vuota oppure Mapped con una
lista non vuota di rule_id già esistenti/univoci nello stesso trace. Il tipo
registra la classificazione esplicita del chiamante, non la determina dal testo.
Non prova automaticamente l'equivalenza fra prosa e DSL; questa resta revisione
umana. Regole e vincoli non vengono creati, eliminati, disabilitati o approvati
per effetto di questa funzione. Note non ancora classificabili possono rimanere
in warning; nessuna affermazione di piena copertura normativa viene aggiunta.

ImportedNoteReview conserva SHA-256 dei byte UTF-8 esatti e disposizione. note_hash
usa sha2/Sha256Digest già approvati: CRLF, zeri, spazi e Unicode sono significativi.
valid_disposition distingue reason.trim non vuota (senza modificare il valore) e
riferimenti unici che devono identificare esattamente una regola. Non seleziona
arbitrariamente il primo duplicato. Non interpreta condizioni o note naturali.

review_imported_note verifica unicità trace/field, presenza della nota nel
namespace owned xlsx_source, corrispondenza con source_note visualizzato e
validità della classificazione. Solo dopo inserisce xlsx_note_review. Le find
interne sono precedute dal conteggio esattamente1: non mascherano ambiguità.
Gli errori non mutano il modello. I DTO contengono soltanto stringhe/liste, quindi
la loro serializzazione non introduce un ramo fallibile con dati numerici speciali.
Aggiornare la revisione non modifica testo originale, regole o source import status.

## Lint e severity

lint_raw_schema conserva i controlli strutturali e aggiunge lint_authoring dopo
la struttura di ogni trace, in ordine field. SchemaIssue rimane il contratto
tipizzato condiviso col compiler; tre codici nuovi: UnreviewedImportedNote,
InvalidImportedNoteReview e ContradictoryConstraints. Nessun cambio RunStatus,
ValidationError, total_errors o skipped_rules dei flussi sanitari.

Per note non vuote del solo namespace importato xlsx_source:
- source_note divergente dal testo originale: InvalidImportedNoteReview Error;
- review assente o legata a un hash diverso: UnreviewedImportedNote Warning;
- review malformata o riferita a regole mancanti/ambigue: Error;
- review valida e testo esatto: nessuna issue di review.

Regulatory_metadata arbitrari/legacy non vengono reinterpretati. Il source_note
originale è conservato nella issue (le presentazioni JSON esistenti lo escapano),
non convertito in regole. Un warning resta un avviso di authoring, non una
violazione normativa né una dimostrazione di codice normativo non implementato.

Per ogni field i domini AllowedValues sono congiuntivi. lint_authoring calcola
l'intersezione BTreeSet sullo stesso target raw oppure logical, valori testuali
esatti e distinti, senza normalizzare zeri/spazi. Non confronta target differenti.
Se almeno2 domini non hanno valore presente comune: Error solo quando base è
Required senza alcun forbidden_if; Warning per optional o applicabilità potenzialmente
forbidden. Questo evita di dichiarare impossibile l'intera validazione quando
un campo può essere assente. Il test delle condizioni non viene eseguito o
semplificato dal lint. Non è un solver completo di vincoli; range/regex/tipi e
altri controlli del compiler restano inalterati. Nessuna contraddizione di questo
tipo rilevata nei dati SIOC.2 effettivamente presenti, verificato separatamente.

L'ordine è deterministico: struttura trace, field dichiarati, issue note, poi
domini raw/logical. Nessuna iterazione HashMap guida il nuovo output. Ogni causa
viene emessa una volta per field/target, non per record di input.

Il compiler esistente blocca soltanto SchemaIssueSeverity::Error, già conforme
DET§142: i test verificano che warning soli compilino e che Error impediscano
la compilazione. Nessuna logica normativa globale dell'engine è stata cambiata.
Le API di lint rendono disponibili warning tipizzati. Il comando schema lint
attuale espone le issue del compiler quando questo fallisce; l'esposizione dei
warning di una compilazione riuscita resta wiring per14-05, non anticipata qui.
Il criterio14-02 è l'API authoring/lint, non l'implementazione della futura CLI.

## Test e verifiche

10 integration in schema_authoring.rs:134 alias originali del profilo SIOC
normalizzati nel profilo canonical identico, tipo vuoto4.5 rimane esplicito;
errori unknown/duplicate/canonical-rebinding e transazione senza mutazione;
warning note non revisionate senza generare regole; hash esatto/stale review;
classificazione metadata/mapped, riferimenti non inventati e regole cancellate;
review malformate/reason vuota/entità assenti; note visualizzate modificate non
possono conservare review vecchie; intersezioni disgiunte Error/Warning secondo
presenza; raw/logical distinti e valori esatti, ordine ripetibile; metadata legacy
non reinterpretati. Tre codici nuovi esercitati, compiler Error vs Warning verificato.

Un primo test optional perdeva encoding.resolved nel round-trip serde (escluso
per contratto dal JSON): corretto il binding della fixture, non cambiato il
compiler. Clippy iniziale ha richiesto let-chain e rimozione di una unit expression,
corrette senza allow o soppressione di controlli.10 test mirati poi positivi.

Gate globale/locali e CI da registrare dopo esecuzione. Test/read source della
precedente task e report normativi restano intatti. Al termine14-02 STOP e
resoconto percentuale secondo mandato, non avviare14-05/P15 o altri flussi.

Helper dei test: raw carica lo schema sintetico e applica il binding UTF-8 esplicito;
noted aggiunge solo il namespace note importate con testo significativo; reason
fornisce una classificazione manuale di prova; domains costruisce due insiemi
letterali e sceglie presence/target, riapplicando il binding runtime dopo serde.
Gli expected sono valori/codici dichiarati, non snapshot copiati dall'output.
Display degli errori authoring usa descrizioni stabili senza riportare note o
nomi alias non escapati; i payload tipizzati restano ispezionabili.

Prima regressione globale: la vecchia fixture first_semantic_failure di
field_pipeline.rs aveva due domini raw required disgiunti{0012}/{0014}. Il nuovo
lint ne rifiuta correttamente la compilazione prima della prova di short-circuit.
Il secondo dominio ora contiene0012/0014:0013 continua a fallire entrambi e deve
emettere un solo errore. Aggiunta verifica che0012 sia davvero ammesso da entrambi.
Nessuna aspettativa runtime indebolita, nessun golden o schema normativo alterato;
la fixture ora ha un modello soddisfacibile, come richiesto dal nuovo lint.

### Gate globale dopo adeguamento della fixture

642 test passati/75 suite,0 fallimenti/2ignored. Fmt/check/clippy positivi;
release/GNU e qualificazione SIOC post-lint ancora in completamento.
Log target/verification-p14-02/T-P14-02-test.log, target isolato nel workspace
con CARGO_PROFILE_DEV_DEBUG=0/CARGO_PROFILE_TEST_DEBUG=0/CARGO_INCREMENTAL=0,
CARGO_HOME isolata/--offline --locked. Nessuna disattivazione di test.
90 fonti protette e33 hash13-08 invariati, nessun cambio alle dipendenze14-02.

### Gate locale definitivo —2026-10-26

Comando completo exit0: fmt/check-all-targets/clippy-all-targets -D warnings/
cargo test/build-release/check Windows GNU/diff.642/75 suite,0 fail/2ignored.
Qualificazione manuale sul workbook originale di nuovo positiva dopo il lint:
135field/130formule/8note, PERSIST completo ancora correttamente rifiutato.
Generator135binding in check-only passato, fonti/schema/lock invariati.
Gate nativo sul commit pubblicato ancora da verificare:14-02 IN_PROGRESS.
