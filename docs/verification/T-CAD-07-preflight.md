# T-CAD-07 — preflight, 7 novembre 2026

READY suCAD02/P10/P13/G0 qualificati; indipendente dal gate06 pendente.
06pubblicata3719d14, CI42372080602 prima lettura in_progress sullo SHA esatto.
Nessun dispatch duplicato. Single-agent, nessun background/subagente.

Fonti: prompt07, ARCH45/DET166–174/specpersistenza, master14/20, norme/
CAD_HISTORICAL_PROFILE5, CAD-14/T-01/T-03, note2.6/2.7/4.7, fixture17,
CAD_RUNTIME_PROFILE.temporal e piani raw/compiled AfterDateRestriction.

Data accesso strettamente successiva alla conclusione: sede1vietata per ogni
motivo; stesso giorno tollerato. Per decessi03/04/07/82, vietate inoltre
2/4/5/6, sede3sempre ammessa da queste regole. Si riusa il codice normativo
E_CAD_ACCESS_AFTER_CONCLUSION già formalizzato, senza inventare un codice
decesso. Decomposizione in due piani:1universale e2/4/5/6perdecesso, evitando
di duplicare lo stesso errore sulla sede1. L'unione coincide con la policyfonte.

Il compiler risolverà date/owner/join, when/target_when locali anche attraverso
derivati, forbidden_values e capabilitytemporal.after_date.v1. Join scalare o
insieme con consenso esplicito necessario; nessun first(). Il componente CAD
usa zero_or_many, consensus su2.6/2.7 e include_if data presente. Tupla
confliggente -> Unknown/AmbiguousJoin con diagnostico del consenso esistente.

Run solo su inputcorrenti, senza snapshot/storia cumulativa: vincoli persistenti
restano rifiutati. File B1assente Unknown; nessuna riga evento Unknown secondo
policyjoin; B1evento presente senza conclusioni qualificata come evento aperto
solo con coveragecompleta. Date invalide non diventano conclusioni mancanti.
Il target_when si applica alla tupla qualificata, non prima del consenso, per
non nascondere conclusioni confliggenti dietro un filtro sul motivo.

File: schema temporal/compiler/plans/dependencies; engine contesto target
privato/seconda passata/diagnostica; componente A/B1/C versionato, test reali
servizio e report. Nessuna APIapplicativa/CLI/UI, crate, storage o norma nuova.
Test: prima/same-day/dopo, tutti motivi/sedi/decessi, assenti/vuoti/aperti,
B1multipliuguali/confliggenti/filtrati, datiinvalidi/partial, determinismo,
namespace/rinomina, guardiecompiler e budget/cancellazione. Locali/CI propria
primaDONE07. Nessuna conformità CAD globale prima12.
