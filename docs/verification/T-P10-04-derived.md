# T-P10-04 — Presenza condizionale con derivati ratificati

2026-10-23. Prerequisiti: T-P10-03 estesa 878b033 con CI 39794989247,
workflow e job Linux/Windows completed/success, verificati alla ripresa;
T-P6-06 già consegnata. Prompt atomico, Piano §4/§14, DET §21–22/39–42,
ARCH §9/CNF-10, authoring ratificato T-P0-07 e rettifica P10 applicati.
Nessuna decisione normativa nuova o modifica alle fonti.

## Perimetro e codice

La normalizzazione di policy, resolve_presence e l'ordine degli stage sono già
corretti e restano riusati. Non si duplica il calcolo delle condizioni nel
validator di presenza né si trasferisce EvaluationContext nel livello schema.
Questa consegna integra i veri piani dell'executor nella frontiera del record.

engine/derived_evaluator::local_derived_type esamina soltanto metadata e
restituisce Some(Bytes/Integer) oppure None. La closure field ammette solo
ResolvedFieldRef::Local con indice esistente e ID corrispondente, evitando
che un indice numericamente valido designi un altro campo. Substring controlla
che l'estremo esclusivo del ByteRange già checked non superi la larghezza della
fonte: la vista logical può essere più corta, ma quello è Unknown runtime,
non un range compilato fuori dal campo. I due piani calendario richiedono
entrambe le fonti Date con lo stesso formato esplicito del piano. Nessun dato
di record viene letto da questo controllo; nessuna data mancante è inventata.

expression_evaluator::expression_supported raccoglie l'iterazione AST già
esistente, con contesto schema opzionale. local_expression_supported la invoca
senza schema e conserva il gate precedente delle regole: la loro diagnostica
derivata non è ancora consegnata. local_presence_expression_supported passa
lo schema reale, perché la presenza dispone del contesto e non espande template
di regola. Entrambe percorrono tutti i rami, anche se una costante basterebbe
a decidere la verità: un obbligo ineseguibile non è nascosto dal short-circuit.

Per DerivedEq vengono controllati schema disponibile, ref locale, indice/ID
del campo e tipo del piano risolto dall'indice. Bytes confronta il logical
senza conversione; Integer è ammesso solo con campo Numeric, mai con Decimal
o AN convertito per deduzione. DerivedCompare richiede coppia Bytes/Bytes o
Integer/Integer tra risultato del piano e literal compilato. Indice derivato
assente, piano invalido e coppia incompatibile rifiutano il piano. Gli altri
nodi conservano i controlli precedenti su ref locali e liste non vuote.

record_validator::check_local_plan sostituisce il divieto indiscriminato della
raccolta derived_values con la verifica di ogni piano tramite local_derived_type.
Anche un piano inutilizzato ma malformato viene rifiutato prima di contare
record o emettere errori. Per le policy richiama il nuovo controllo con schema.
Regole non integrate, repeated_groups e constraint future restano esplicitamente
rifiutate; ProjectionPlan non consumato in LocalOnly resta lecito (CNF-RECORD-01).

L'esecuzione resta quella reale di LocalEvaluationContext: legge lo stesso
RecordView/schema della pipeline, risolve il piano memorizzato ed esegue
DerivedValueEvaluator. resolve_presence preserva gli obblighi base, il conflitto
required+forbidden e i motivi Unknown. La pipeline riusa la decisione una sola
volta. Unknown incrementa skipped_rules una volta e ferma gli stage successivi;
il record termina con UndeterminedPresence tipizzato, senza classificare un
record indeterminato come valido o pubblicare una violazione normativa inventata.
Il validator rimane terminale dopo quell'errore tecnico.

## Confine incrementale

Il compiler pubblico conserva il gate derived_values/capability fino a T-P10-05,
che deve consegnare assertion e diagnostica complete. Non dichiariamo ancora
eseguibili dalla CLI schemi derivati. Le fixture private inseriscono piani veri
nel solo cfg(test), preservando la non-forgiabilità del modello pubblico.
Nessun provider fittizio o costruttore pubblico aggiunto, nessun ramo futuro
ignorato. P10 resta IN_PROGRESS; 10-05/06 sono residui espliciti.

## Test e correttezza

Sei test nel nuovo compiled/derived_presence_tests, con record sintetico:
DETAIL precede le date sorgenti e AGE Numeric, per verificare letture di campi
successivi senza mutare flag o dati. La fixture compila il vero schema scalar
e aggiunge privatamente i tre piani, senza evaluator di test preconfezionati.

- Tutti gli stati Required/Optional/Forbidden/Conflict/Unknown da vere date;
  base Required non eliminata, missing e data invalida mantengono motivi distinti.
- RecordValidator accetta i piani eseguibili, segnala required mancante come
  Invalid e lascia optional mancante valido; byte originali identici dopo il run.
- Data invalida produce UndeterminedPresence per DETAIL, zero errori normativi;
  tentativo successivo sullo stesso validator dà PreviousFailure.
- FieldPipeline restituisce E_CONDITIONAL_REQUIRED/E_FORBIDDEN/E_RULE_CONFLICT
  come solo errore primario nei casi pertinenti; sorgente invalida produce
  SkippedDueToUnknownPresence e un solo skipped_rules, anche con due obblighi.
- DerivedEq intero e DerivedCompare substring/year delta composti con And,
  Not per RequiredUnless e Or dominante rispettano la logica a tre valori.
- Indice derivato assente, literal incompatibile, campo AN confrontato a età,
  ref indice/ID incoerente, fonte calendario AN e substring fuori campo sono
  rifiutati anche sotto Or(Const(true), ...), prima di leggere record.

La vecchia fixture negativa DerivedValues ora usa un range oltre la larghezza:
un piano valido ha il suo executor e non deve più figurare tra funzionalità
assenti. Il test conserva il vero rifiuto di piani non eseguibili/malformati.

## Evidenze e handoff

438 test/53 suite, zero falliti; sei nuovi test mirati passati. Log completo
/tmp/lintra-t-p10-04-derived-test.log. fmt/check all-targets/Clippy all-targets
-D warnings/test/release/Windows GNU check all-targets/diff --check exit 0,
Cargo --locked. 64 file protetti SHA-256 identici a HEAD; QG-3: engine senza
application/adapter o norme di flow, input checked e nessun unwrap produttivo.
CI nativa Linux/Windows MSVC pendente; il controllo GNU locale non la sostituisce.

File modificati:
engine/derived_evaluator/expression_evaluator/record_validator, wiring/test
compiled/derived_presence_tests e negativi precedenti, dossier/stato.
Nessun Cargo/lock, baseline normativa, golden o formato raw modificato.
Next T-P10-05 residuo dopo gate; poi 10-06 e ripresa 11-02 autorizzata.

CI [39917228132](https://github.com/UraganoAngelina/lintra/actions/runs/39917228132)
su cb0a0445e1d13af550a0957794a6795855c7a72f: workflow e job Linux GNU/Windows
MSVC tutti completed/success, verificati con tre chiamate (dispatch e due letture)
a intervalli >45s. Il commit comprende 48acdf5 (T-P10-04); engine e test presenza
non modificati fra i due commit. Gate cumulativo effettivo, non un risultato
retroattivo inventato per la run precedente ancora pendente all'ultima lettura.

T-P10-04 estesa DONE; next 10-05 dopo gate, come autorizzato.
