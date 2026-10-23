# T-P10-03 — Executor dei tre derivati ratificati

**Stato corrente 2026-10-23: DONE.** CI 39794989247, workflow e job
Linux/Windows MSVC completed/success (gate cumulativo per 10-04, spiegato sotto).
P10 estesa ora chiusa con 458 test e CI finale 39920597392. I gate temporaneamente
chiusi o pendenti nel registro seguente descrivono questa specifica consegna;
compiler/presenza/regole pubblici sono stati integrati nelle task successive.

## Registro della consegna e delle verifiche

2026-10-22, gate locali passati; CI nativa pendente.
Prerequisito modello esteso 24b6204/CI 39791606233 workflow completed/success;
dettaglio Windows non allineato in quella risposta conservato nel dossier 10-01,
non spacciato come singolo job completed verificato. Autorità e scope nella
[rettifica P10](P10-derived-recovery.md): authoring completo T-P0-07 approvato.
Nessuna nuova funzione arbitraria, modifica del formato o fonte normativa.

## Valore e contesto

DerivedValue espone Bytes borrow, IntegerLiteral esatto owned e Unknown(reason).
L'operazione può leggere il record o restituire un valore, mai mutare sorgenti o
emettere errori normativi. Non esistono stringhe di funzione eseguibili o default
empty/zero/data. DerivedValueEvaluator è stateless e usa soltanto i tre piani
chiusi già compilati, non legge l'orologio né versione/locale dell'host.

La lifetime del risultato Bytes segue il FieldValueResolver di EvaluationContext,
non il temporaneo wrapper del contesto o del piano. source_bytes prende raw/
logical dal target esplicito: Present→borrow, Missing→Unknown FieldUnavailable,
Unavailable→stesso motivo. Non prova un altro target né decodifica/normalizza.
Substring usa ByteRange::slice checked sulla vera vista scelta; una vista logical
più corta del range produce FieldUnavailable, mai padding o slicing fuori limite.
È zero-copy anche se include byte non Unicode; validità del campo è un controllo
separato della pipeline, non viene dichiarata automaticamente dal derivato.

dates risolve entrambe le fonti logical e usa DateLiteral::from_ddmmyyyy comune.
La rappresentazione e il calendario sono quelli già approvati: anno 0001..9999,
sole cifre ASCII, DDMMYYYY. Dato presente invalido è InvalidFieldValue, dato
mancante/record fuori misura è FieldUnavailable. Cause di join/trace indisponibili
fornite dalla porta vengono propagate, non convertite in false o dati fittizi.

CalendarYearDelta calcola year(reference)-year(birth), checked e firmato,
senza confrontare mese/compleanno. AgeInCompletedYears sottrae anche l'anno
incompleto rispetto all'anniversario esplicito. Entrambi conservano risultati
firmati quando reference precede birth: SCHEMA_AUTHORING, «Valori derivati
approvati e casi limite», lascia alle constraint del profilo il significato
dell'età negativa. Nessun wrap unsigned o errore normativo implicito. La review
ha rilevato e rimosso prima della consegna un controllo cronologico che
avrebbe introdotto una restrizione non autorizzata. Test aggiunti per -4/-5 e
per l'intervallo massimo invertito -9999 anni compiuti / -9998 delta anni.

Per Age l'anniversario è mese/giorno di nascita. Solo nascita 29/02 e anno di
riferimento non bisestile usano l'enum del piano Feb28 oppure Mar01. La verifica
di anno bisestile usa chrono::NaiveDate::from_ymd_opt, non una tabella normativa
hardcoded o una nuova funzione calendario. Se reference mese/giorno precede
l'anniversario si sottrae un anno dal delta. Tutte le sottrazioni checked;
i quattro digit degli input vincolano già i valori a i32 senza overflow.
Nessun default per 29/02 o interpretazione per nome del campo/flow.

integer converte il risultato i32 in IntegerLiteral tramite il parser esatto
esistente. Nessun floating point, cast monetario o arrotondamento. Un fallimento
interno della conversione (stringa generata da i32) è UnsupportedPlan, mai panic
con unwrap. Entrambi i risultati possono essere negativi: decide il profilo.

## ExpressionEvaluator e porta reale

FieldValueResolver aggiunge resolve_derived(index). Default esplicito
Unknown UnsupportedPlan per contesti che non offrono quell'operazione, non
un vettore vuoto promosso a supporto. LocalEvaluationContext implementa il metodo
realmente: get del piano in schema.derived_values e DerivedValueEvaluator sullo
stesso record/schema. Indice assente è FieldUnavailable. La grammatica dei tre
piani legge solo field e non altri derived; quindi nessuna ricorsione nascosta
fra piani, ciclo di calcolo o cache mutabile. I riferimenti restano read-only.

DerivedEq risolve field logical e valore derivato. Bytes confrontati esattamente,
Integer richiede cifre ASCII nonempty nel campo e IntegerLiteral esatto (zeri
significativi come valore intero, non trim). Segni +/− non diventano nuova sintassi
Numeric. Missing/unavailable e input tipizzato invalido mantengono Unknown.
Non si interpreta Decimal come intero: il compiler del modello già lo rifiuta.
DerivedCompare usa compare_derived con il literal del piano: Bytes/Bytes e
Integer/Integer, altri abbinamenti diretti sono UnsupportedPlan senza coercizione.
ordered_truth mappa i sei ComparisonOperator su Ordering, con esiti True/False;
Unknown conserva il motivo. Booleani e il loro ordine restano quelli già provati.

Root re-export rende nominabili evaluator/valore e metadata necessari (piani,
DateFormat/LeapDayAnniversary e IntegerLiteral/parser error). Non apre constructor
CompiledFlow/Record/Field e non espone un repository fittizio. Nessun import
application/CLI, dipendenza o payload futuro ignorato.

## Gate incrementale mantenuto

Questo passo consegna calcolo ed Expr pure. Il compiler continua a rifiutare
la raccolta derived_values e le capability relative fino alla consegna di
presence/rules/diagnostica estese; nessun compiled pubblico scarta i piani.
I gate del record per i piani ancora non integrati restano chiusi. P10 è ancora
IN_PROGRESS: un test positivo dell'executor non è esecuzione CLI promessa.
Non si sostituisce un servizio/provider con un mock per aprire quel percorso.

## Test

Sette integration test su record scalar con due Date, AGE Numeric Exact e CODE
AN LeftPad: età prima/durante compleanno e delta anni distinto; Feb28/Mar01 su
29/02 con anni normali, 1900 non bisestile, 2000 bisestile; estremi 0001/9999;
date invalide/mancanti/record corto e ordine inverso (entrambi firmati senza wrap); substring raw/logical con confronto pointer zero-copy e
range che non entra nel logical; DerivedEq per byte e intero con zeri, segni,
missing; sei operatori DerivedCompare, literal incompatibili e indice assente;
età zero e propagazione di tutti i motivi Unknown.

Il Context composto del test inoltra alle vere field lookup e al vero evaluator
sui piani, non restituisce età preconfezionate; è adattatore di test, non provider
di produzione o presunto schema normativo. Un ulteriore unit test dentro il
modulo compiled inserisce privatamente un vero piano nella sola fixture e prova
LocalEvaluationContext.resolve_derived su quel piano/record; non apre API di
costruzione e non aggira il gate del compiler in produzione. Gli stessi confini
privati sono già usati dai test del modello/record delle task precedenti.
Il negativo precedente DerivedEq su uno schema privo di piani ora esige
FieldUnavailable invece di "executor assente"; gli altri piani futuri restano
UnsupportedPlan. Il primo test ha rilevato che IntegerLiteral non era ancora
nominabile dal root: aggiunto il re-export effettivamente usato dal contratto.

## Evidenze e handoff

432 test/53 suite, zero falliti; log /tmp/lintra-t-p10-03-derived-test.log.
fmt --check/check all-targets/Clippy all-targets -D warnings/test/release/
Windows GNU check all-targets/diff --check exit 0, Cargo --locked. Ultima
esecuzione completa dopo correzione signed age, non prove precedenti riusate.
64 file protetti (schema/normative/golden/meta-schema v1) SHA-256 identici a HEAD.
QG-3: revisione sorgenti, assenza hardcoding normativo, engine indipendente
application, solo input checked; native Windows MSVC ancora da verificare.

File engine/derived_evaluator/expression_evaluator/mod, export,
test derived/expression e wiring unit compiled; dossier/stato. Nessuna baseline,
source o Cargo/lock modificati. Next residuo T-P10-04 dopo gate nativo, poi 10-05
per assertion/diagnostica e 10-06 golden estesi; P10 ancora aperta, 11-02 attende.

## Chiusura gate nativo — 2026-10-23

CI [39794989247](https://github.com/UraganoAngelina/lintra/actions/runs/39794989247)
su 878b033fa8e785f68c673da33cbf1838bf0e001b ora completed/success;
entrambi i job Linux GNU e Windows MSVC completed/success, verificati con gh.
Le tre letture/dispatch del 22 ottobre avevano esiti ancora pendenti; nessuna
chiusura allora presunta. La ripresa autorizzata del 23 ottobre recupera il
risultato definitivo. T-P10-03 estesa DONE; next residuo T-P10-04.
