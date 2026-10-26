# PROMPT ATOMICO — T-CAD-12: Golden CAD multi-trimestre e freeze runtime

Sei coding agent Lintra. Implementa **soltanto T-CAD-12**. Il contratto storico generico è **ADR-PERSIST-001 / `PERSISTENCE_CORE_SPEC.md`**, non un plugin storage CAD-only. Il profilo CAD dichiara la normativa e riutilizza i servizi core.

## Prerequisiti
T-CAD-04..11, T-PERSIST-12; P0 CAD quando la task tocca note normative; verifica prerequisiti sullo stato reale e segnala `BLOCKED` se mancanti.

## Obiettivo e comportamento richiesto
Golden end-to-end con fixture sintetiche 5 file e mancanti B2, correlazione D→C su 4 campi senza progressivo, PK complete con progressivi, sessioni T1/T2/T3, snapshot riavviati, privacy, performance, G0/G-runtime report; non dichiarare FROZEN senza gate.

## Fonti obbligatorie
- `SOURCE_DOCUMENTS/ARCHITECTURE.md` §45; `SOURCE_DOCUMENTS/DETAILED_DESIGN.md` §166–174; `PERSISTENCE_CORE_SPEC.md`.
- `CAD_REVISED_CANDIDATE/`, `CAD_NOTES_AUDIT.md`, `CAD_FIXTURE_PLAN.md`, PDF regionale in `SCHEMA_INPUTS/CAD_2024-06_SOURCE`.
- `SOURCE_DOCUMENTS/MASTER_IMPLEMENTATION_PLAN.md` §14, §20; `IMPLEMENTATION_STATUS.md`.

## Gate ed evidenze
Test sintetici positivi/negativi e edge case, output deterministico, policy dichiarative in schema/version, no hardcoding CAD/SIOC nel core, nessuna PHI. Esegui realmente i comandi Cargo disponibili; documenta test Windows non eseguiti e blocchi. Aggiorna stato e handoff, quindi prosegui automaticamente con la prossima task READY secondo AGENTS.md.
