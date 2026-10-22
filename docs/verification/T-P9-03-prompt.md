# T-P9-03 — override ratificato del prompt Decimal

2026-10-22, autore: «approvo questa estensione, procedi pure» in risposta alla
proposta docs/proposals/decimal-amount-contract.md. Il prompt originario nello
starter kit resta disponibile; questo documento ne precisa lo scope autorizzato.
Non si modifica una baseline normativa o si presume supporto da una sola struct.

Implementare DecimalSpec/validator e l'intera via eseguibile raw → lint/compiler
→ compiled → datatype/pipeline/record → servizio/CLI. Il formato 2 aggiunge
il nodo Decimal esplicito; formato 1 resta invariato e lo rifiuta. Campi codice,
domini e range ricevuti non si convertono; niente classificazione per nome/flow.
Profilo importi: exact, scala due, punto al terzultimo byte, unsigned,
non negativi con zero ammesso, nessun separatore delle migliaia/float.
Presenza/missing restano esplicite e precedono datatype, una causa primaria.
Range Numeric non si converte implicitamente a centesimi/parte intera di Decimal.

Autorizzati meta-schema v2 separato, authoring/lowering, generazione DTO superset,
DET §137, mappa compatibilità e test/docs/handoff pertinenti. Baseline v1, normative
e golden preesistenti protetti; nessuna migrazione, nuova dipendenza o format
ulteriore al 2. API compiled di sola lettura e non-forgiabilità preservate.

Eseguire fmt/check/Clippy/test/release, riproducibilità raw, check authoring v1/v2,
check Windows GNU e CI nativa Linux/Windows. Report dettagliato di strutture,
funzioni, condizioni e ragioni di correttezza; stato e handoff con evidenze reali.
Dopo gate positivo proseguire secondo il mandato autonomo; stop solo su blocchi
contrattuali reali. Niente Superpowers/subagenti/processi agente in background.
