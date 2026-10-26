# PROMPT ATOMICO — T-CAD-08: CAD-17 tre mesi calendario sospensioni pause

Sei coding agent Lintra. Implementa **soltanto T-CAD-08**. Il contratto storico generico è **ADR-PERSIST-001 / `PERSISTENCE_CORE_SPEC.md`**, non un plugin storage CAD-only. Il profilo CAD dichiara la normativa e riutilizza i servizi core.

## Prerequisiti
T-CAD-05, T-CAD-06, T-CAD-07, P9; P0 CAD quando la task tocca note normative; verifica prerequisiti sullo stato reale e segnala `BLOCKED` se mancanti.

## Obiettivo e comportamento richiesto
Da ultimo accesso C con almeno una prestazione D correlata oppure inizio B1.2.4; 3 mesi di calendario (non 90gg), esclusi giorni di sospensione computabili, sovrapposizioni unificate, valutazione fine trimestre, D assente = NOT_EVALUABLE; B1.2.7=12 quando requisito raggiunto. Test febbraio/leap-year, più pause, nessuna prestazione, D missing.

## Fonti obbligatorie
- `SOURCE_DOCUMENTS/ARCHITECTURE.md` §45; `SOURCE_DOCUMENTS/DETAILED_DESIGN.md` §166–174; `PERSISTENCE_CORE_SPEC.md`.
- `CAD_REVISED_CANDIDATE/`, `CAD_NOTES_AUDIT.md`, `CAD_FIXTURE_PLAN.md`, PDF regionale in `SCHEMA_INPUTS/CAD_2024-06_SOURCE`.
- `SOURCE_DOCUMENTS/MASTER_IMPLEMENTATION_PLAN.md` §14, §20; `IMPLEMENTATION_STATUS.md`.

## Gate ed evidenze
Test sintetici positivi/negativi e edge case, output deterministico, policy dichiarative in schema/version, no hardcoding CAD/SIOC nel core, nessuna PHI. Esegui realmente i comandi Cargo disponibili; documenta test Windows non eseguiti e blocchi. Aggiorna stato e handoff, quindi prosegui automaticamente con la prossima task READY secondo AGENTS.md.
