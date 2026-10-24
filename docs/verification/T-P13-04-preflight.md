# T-P13-04 — Preflight riferimenti esterni e Unknown

2026-10-24, nessun codice della task iniziato. Prerequisito 13-03 commit 61ee382
pubblicato, dispatch accettato alle 10:06 UTC, esito nativo in attesa. Prereq
10-03 già verificato in CI 39794989247 e regressione cumulativa corrente.
Fonti lette: prompt, DET §18–20/§39/§71–74/§149, authoring join scalari/insiemi/
policy/consensus, dossier 13-01/02/03 e CNF-COMPILER-01.

## Contratti attuali verificati

ResolvedFieldRef::External esiste già con indici field/trace/join e ID. Il
lowering pubblico del compiler rifiuta gli external, RecordExists/SetExists;
LocalEvaluationContext li rifiuta con UnsupportedPlan. EvaluationContext
usa la porta FieldValueResolver, introdotta in P10 per evitare modifiche
incompatibili a tutti i consumer. UnknownReason ha già RelatedRecordMissing,
AmbiguousJoin, TraceInputUnavailable. Non duplicare enum/esecutori locali.
ContextStore restituisce tutti i target; source key e raw values posseduti
sono nel ProjectedRecord. Il lookup set vuoto non garantisce input disponibile.

## Porzione autorizzata e API minima

Aggiungere un resolver reale sui projected/related, consumabile dalla stessa
EvaluationContext senza cambiare la firma dell’evaluator. Validare indici/ID
contro il compiled flow e join source/target, mantenendo schema non forgiabile.
Accesso locale projected usa soltanto i campi selezionati e le stesse policy
missing/raw/logical. Accesso esterno usa il binding Source del record per
interrogare il trace Target e la join corretta. Nessuna rilettura input.

Disponibilità input deve essere esplicita: assente -> TraceInputUnavailable;
disponibile senza riscontri -> RelatedRecordMissing; più candidati quando si
richiede uno scalare -> AmbiguousJoin, mai selezione arbitraria. Lookup tecnici
falliti non possono diventare assenza normativa: errore di costruzione/preflight
o stato tecnico conservato, nessun unwrap o False fittizio.

Namespace parameters richiesti da join vanno forniti e verificati prima di
qualunque lookup; non prendere allowed_values/min/max come default runtime.
Il context rimane di un singolo run/schema/namespace, non uno store storico.
Per forme non consegnate (es. reduction/esecuzione set, se non nello scope
effettivo di questo passo), rifiuto esplicito: non mascherarle con Unknown
RelatedRecordMissing o chiamarle supportate soltanto perché i metadati esistono.

NotApplicable è un esito di regola distinto: authoring lo vieta come sinonimo
di Satisfied o Unknown. La porta scalare deve conservare causa/JoinId/policy
per il futuro evaluator 13-05, senza emettere una conclusione normativa in
questa task. RuleValidationResult locale non va alterato inutilmente; un
esito cross dedicato può conservare la non applicabilità quando integrato.

Coerentemente con P10-03 e CNF-COMPILER-01, non anticipare l’intero compiler/
executor cross di 13-05 per aprire artificialmente Full. Il resolver e lowering
esterno devono essere realmente testabili, mentre i gate del percorso pubblico
non ancora integrato restano esplicitamente chiusi. L’integrazione compiler
richiederà aggiornamento esaustivo della projection per tutte le ref nuove.

## Verifiche previste

Schema/join/proiezioni/store reali; local projected raw/logical/missing;
external scalare presente/assente/trace indisponibile/ambiguo; nessun falso
IsPresent=False per relazione indisponibile; tavole AND/OR/NOT sui motivi
Unknown; indici/ID incoerenti, self-join e chiave composta, byte esatti;
namespace non configurato rifiutato prima del lookup. Contesto e errori senza
application/CLI, nessuna IO/normalizzazione/hardcoding. Gate completi e CI.
