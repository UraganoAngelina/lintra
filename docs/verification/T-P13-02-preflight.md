# T-P13-02 — Preflight documentale della projection

2026-10-23. Nessun codice della task iniziato: prerequisito T-P13-01 locale
verificato (504 test), gate nativo sospeso prima del push. Workflow autonomo
autorizza la ripresa soltanto dopo evidenze Linux/Windows disponibili.

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
