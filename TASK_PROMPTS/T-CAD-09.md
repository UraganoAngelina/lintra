# PROMPT ATOMICO — T-CAD-09: Età e presence condizionale CAD

Sei coding agent Lintra. Implementa **soltanto T-CAD-09**. Il contratto storico generico è **ADR-PERSIST-001 / `PERSISTENCE_CORE_SPEC.md`**, non un plugin storage CAD-only. Il profilo CAD dichiara la normativa e riutilizza i servizi core.

## Prerequisiti
P9, P10; P0 CAD quando la task tocca note normative; verifica prerequisiti sullo stato reale e segnala `BLOCKED` se mancanti.

## Obiettivo e comportamento richiesto
Age policy = year(data inizio presa in carico) - year(nascita); B1.2.17 required if age>=3; B1.2.18 required if age>=6; A.1.24 policy età <=17 quando confermata da schema; evitare confronti su date non parseable. Test soglie 2/3 e 5/6, Unknown.

## Fonti obbligatorie
- `SOURCE_DOCUMENTS/ARCHITECTURE.md` §45; `SOURCE_DOCUMENTS/DETAILED_DESIGN.md` §166–174; `PERSISTENCE_CORE_SPEC.md`.
- `CAD_REVISED_CANDIDATE/`, `CAD_NOTES_AUDIT.md`, `CAD_FIXTURE_PLAN.md`, PDF regionale in `SCHEMA_INPUTS/CAD_2024-06_SOURCE`.
- `SOURCE_DOCUMENTS/MASTER_IMPLEMENTATION_PLAN.md` §14, §20; `IMPLEMENTATION_STATUS.md`.

## Gate ed evidenze
Test sintetici positivi/negativi e edge case, output deterministico, policy dichiarative in schema/version, no hardcoding CAD/SIOC nel core, nessuna PHI. Esegui realmente i comandi Cargo disponibili; documenta test Windows non eseguiti e blocchi. Aggiorna stato e handoff, quindi prosegui automaticamente con la prossima task READY secondo AGENTS.md.
