# T-P13-02 — Preflight documentale della projection

2026-10-23. Nessun codice della task iniziato. Prerequisito T-P13-01 verificato
nativamente nella CI 39959838976 su 8fc31f5 (Linux/MSVC success).
T-P12-05 pubblicata 8ce60e6, dispatch accettato alle 15:15 UTC; gate in attesa.
Avvio implementativo 13-02 solo dopo il gate corrente, come dichiarato.

## Contratti recuperati

Prompt atomico dello starter kit: schema/projection.rs e context/projected_record.rs;
DET §§68–70/89/164; QG-3. ContextProjectionPlan esistente contiene field_indexes
e join_indexes, ma il compiler conserva attualmente solo i campi record_key.
CompiledJoinSpec contiene i metadati reali consegnati in 13-01; nessun relativo
runtime viene simulato. CNF-RECORD-01 consente piani non consumati in LocalOnly.

## Piano circoscritto dopo il gate

Derivare nel compiler, per ogni trace, l'unione deterministica dei campi di key,
identity, join source/target e consensus realmente dichiarati. Ordine fisico e
deduplicazione una volta sola. I campi cross-record devono essere aggiunti dal
visitatore del piano supportato, mai dedotti dal nome o da regole normative.
AST cross-record non supportati devono continuare a essere rifiutati: nessuna
asserzione di completezza per forme non compilabili.

L'API DET project_record riceve solo CompiledRecordSchema: il progetto della
join deve quindi contenere anche i binding di campi necessari, non soltanto
indici globali che il projector non può risolvere senza il flow. Verificare
un'estensione tipizzata del piano nel compiler con JoinId e campi risolti,
conservando i vecchi lettori dove possibile. La firma non deve pretendere
accesso globale nascosto o ricostruire metadati da file nel validation loop.

ProjectedRecord owned contiene trace, line_number, identity passata e solo
valori selezionati più join keys; nessun buffer della riga completa. Usare
JoinKey length-prefix di 13-01; assenza/dato incompleto restano esiti tipizzati,
non una chiave vuota inventata. Distinguere byte di campo conservati ed
estrazione logical dichiarata della chiave; nessuna normalizzazione implicita.
Nessun ContextStore, cross evaluator, Full service o persistenza anticipati.

## Verifiche pianificate

Piano completo su entrambi i lati di join, coppie in ordine, identity/consensus,
deduplicazione, campi non richiesti esclusi; projection sopravvive al rilascio
della riga, nessuna copia integrale per comodità; byte/NUL/zeri preservati;
record corto/indice invalido/missing tipizzati. Regressioni LocalOnly e piani
immutabili, gate locali e CI Linux/Windows. Nessun test della task eseguito
o dichiarato prima dell'implementazione. Fonti/grammar/baseline invariate.

## Precisazioni sulle API e sui casi realmente supportati

RecordView reale contiene solo byte, non la riga: project_record deve ricevere
line_number esplicito (>0), oltre a view/schema/identity, anziché inventare zero.
Errore di proiezione posseduto dal layer context, senza import application;
nessuna costruzione di ValidationRunError o esecuzione Full in questa task.

La projection deve contenere binding locali risolti con JoinId, indice globale,
lato source/target e campi ordinati. Una self-join può avere campi source/target
differenti nello stesso trace: conservarli distinti evita sovrascritture su
una sola chiave JoinId. La rappresentazione owned dovrà preservare il lato.
Le chiavi mancanti secondo MissingValuePolicy devono restare uno stato tipizzato,
non chiavi vuote sintetiche né violazioni normative emesse dal projector.
Le failure tecniche di schema/range/logical/allocation restano Result separati.

Le attuali regole cross con RelatedRef/RecordExists/SetExists non raggiungono il
compiled flow: il compiler le rifiuta esplicitamente. Non anticipare il loro
lowering/evaluator per rendere nominalmente completo il piano. L’invariante
si applica a tutte le dipendenze effettivamente compilabili: PK/identity,
entrambe le join key e tuple/predicati consensus; nessuna copia dei campi locali
irrilevanti. I predicati consensus richiedono visita esaustiva CompiledExpr,
comprese dipendenze field/derived (e relativi input). Una nuova variante senza
visita deve essere errore di compilazione Rust, non cadere in un catch-all.
Il riuso dei byte raw nei values e logical esplicito nelle key deve restare
separato; test con fill/NUL/zeri e rilascio del buffer originario.
