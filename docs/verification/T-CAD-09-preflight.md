# T-CAD-09 — derivato età cross-trace, 6 novembre 2026

READY → PREFLIGHT → IN_PROGRESS, base d81b520. CAD11 locale qualificata/pubblicata
attende dispatch dopoCI03; task09 indipendente da03/11. P9/P10/G0 e authoring
T-P0-07 qualificati, ref/consensus/namespace già implementati nelle task precedenti.
Fonti: prompt09, governance/piano4/14/20, ARCH45, DET166–174, specifica persistente,
CAD_NOTES_AUDIT/CAD_FIXTURE_PLAN06, candidatoA/B1, decisioniCAD05/07 econtratto
SCHEMA_AUTHORING derivati/Ref esterne/consensus già ratificato72f9f5e.

Note originali confermano1.24 richiesto con delta anni<=17,2.17 da3,2.18 da6.
Data nascitaA1.11, inizioB1.2.4, DDMMYYYY valido. Nessuna età compiuta, host date,
coercizione, first() suB1 o nuova soglia normativa. PiùB1 dell'evento richiedono
consensus su data inizio; conflitti restanoUnknown/diagnostica dichiarata.

Gap implementativo reale: derived.rs usa solo resolve_local_ref; calendar_year_delta
non può compilare la data sull'altro trace. ProjectedEvaluationContext sa già
risolvere piani conRef esterne eRuleDependencies sa già proiettarne i campi.
RecordValidator rifiuta però qualunque derivato nonlocale anche se usato solo
in regolecross. ExpressionEvaluator non conserva la joincausale quandoUnknown
proviene da un derivato, necessario per applicare missing policy senza inferenze.

Scope autorizzato: completare il solo calendar_year_delta con riferimenti date
esterni tipizzati, mantenendo substring/age_in_completed_years locali. Nessuna
nuova grammar/capability/operazioneDSL/API pubblica o mutazione input. Risoluzione
tramiteReferenceBindings già esistente, controllo tipoDate/consensus e proiezioni
realmente richieste. Guardia locale continua a rifiutare obblighi nonlocali in
presence/predicati di set, mentre i derivatiusati in regolecross sono metadati
validi per la passata1. Regole di richiesta presenzaFieldPresent in passata2,
come consentito dall'authoring. LocalOnly conta i controlli crossskipped/Partial.

PerUnknown causale: helperprivato evaluator con i piani derivati compilati;
when/ConditionTrue cross usano quel contesto, senza cambiare FieldValueResolver,
EvaluationContext o DerivedValue pubblici. Propagare soltanto la sorgente decisiva
nel medesimo ordine di valutazione, preservando boolean dominance e missing policy.

Fileprevisti: compiler/derived/rules, metadata validation/evaluator/flowvalidator,
componenteage versionato con fonti/hash, test compilatore e serviziFull reali,
report/ledger/status. Test soglie2/3/5/6/17/18 senza compleanno, date invalide/
mancanti/parent assente/vuoto/ambiguo, consensoB1, namespace, proiezioni edeterminismo.
Gate completi locali/nativi dopo il delta core, nessunDONE senza gate.
