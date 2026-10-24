# T-P13-05 — Preflight integrazione cross-record

2026-10-24. Nessun codice ancora avviato: prerequisite 13-04 commit 05ac144
pubblicato, dispatch CI accettato alle 10:23 UTC, esito in attesa. Test locali
560/66 suite; gates completi passati. Fonti prompt 13-05, DET §74/§90/§91,
§18–20/§39/§149, authoring join/scalari/insiemi/consensus/policy, Piano §4/§14,
CNF-COMPILER-01 e dossier 13-01..04. Baseline/grammar/fonti non modificabili.

## Stato effettivo e consegna necessaria

Esistono join metadata, proiezioni complete per le dipendenze finora compilabili,
store reale e scalar resolver. Il compiler continua a rifiutare gli external
nelle regole e RecordExists/SetExists; il nuovo helper standalone non apre il
percorso Full. 13-05 deve consegnare piano cross realmente lowered ed executor
senza rilettura file, aggiornando la closure di projection per tutti i campi
source/target/predicati/assertioni ora compilabili. Non basta un test su AST
assemblato senza integrazione compiler. LocalOnly non deve eseguire una
regola cross come se fosse locale né ignorare primitive non supportate.

## Minima evoluzione prevista

Riutilizzare AST/validator/template locali, distinguendo binding context per
ref locali ed esterne. Classificare regole in local/cross sulla dipendenza
reale, inclusi when e assertion; conservare ordine relativo dichiarato. Le
formule non consegnate restano UnsupportedConstruct, nessun catch-all che
scarti una regola. I piani cross devono conservare join/trace/field risolti;
la projection deve includere target e campi coinvolti nelle diagnostiche.

Servono esiti cross tipizzati distinti Satisfied/Violated/Unknown/NotApplicable,
con JoinId/motivo/diagnostic quando applicabile. Non alterare inutilmente il
vecchio RuleValidationResult locale; non rappresentare non applicabilità come
Satisfied o saltare la registrazione. L’executor rimane engine-owned e passa
eventuali risultati al consumer futuro con una porta realistica, senza una
dipendenza da application. Orchestrazione input/pass1/service Full resta 13-06/07.

## Applicabilità e logica

Policy missing si applica alla dipendenza realmente responsabile dell’esito,
non a qualunque join dichiarata nel flow. In particolare False AND Unknown
resta False e True OR Unknown resta True anche quando l’indeterminatezza nasce
da una relazione assente: nessun errore di parent da un ramo non determinante.
Una visita cieca del primo ref Unknown è scorretta per gruppi annidati:
Or(And(False, missingA), missingB) deve conservare la causa di B, non A.

Prima di modificare il comune evaluator definire la minima metadata di causa
associata al risultato del nodo, preservando signature pubblica e tavole
TruthValue. Le riduzioni AND/OR/NOT devono conservare la causa dell’Unknown
che determina realmente il risultato. Missing policy Violation non deve
trasformare input/valore ambiguo in assenza accertata; applicare separatamente
on_missing_record e on_missing_trace e conservare la policy scelta.
NotApplicable è regola fuori ambito secondo l’authoring, non un bool aggiuntivo
arbitrario dentro la DSL. Nessuna falsa convalida tramite when o assertion.

## Insiemi/consensus e riferimenti

RecordExists/SetExists devono usare tutti i match e rispettare cardinalità e
Unknown; nessun first() su un insieme. include_if è un predicato target e ha
le stesse verità a tre valori; Unknown non diventa esclusione False silenziosa.
Consensus confronta tuple coerenti complete, non campi presi da righe diverse;
una sola tupla distinta consente scalar, conflitto -> Unknown con diagnostica,
nessuna tupla -> policy esplicita. Distinguere missing/empty e raw/logical;
nessuna conversione numerica o normalizzazione per dedurre uguaglianza.
Riferimenti scalari fuori dalla tupla ridotta non possono essere ricavati
arbitrariamente da uno dei record. Esecuzione reference metadata/tempo e
primitive non supportate rimangono rifiutate, non coperte da Unknown generico.

## Verifiche e limiti

Compiler reale + projector + store + executor: regole cross positive/negative,
Violation/NotApplicable/Unknown senza confusione, join assente vs trace non
fornito, zero/uno/molti e self-join, condizioni annidate con cause diverse,
projection completa e byte ownership, contesto diagnostico con trace/riga/campi
corretti, ordine deterministico e sink failure tecnica. Parametri namespace
espliciti/controllati, nessun input relazionale ricostruito da file o schema
per nome. Full non eseguibile dal service finché 13-06/07 non integrano gate,
completezza/audit/contatori e cancellazione. Tutti i gate locali e CI Linux/MSVC.
