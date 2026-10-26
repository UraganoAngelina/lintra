# PROMPT ATOMICO — T-CAD-07: Regole temporali B1 di conclusione e decesso

Sei coding agent Lintra. Implementa **soltanto T-CAD-07**. Il contratto storico generico è **ADR-PERSIST-001 / `PERSISTENCE_CORE_SPEC.md`**, non un plugin storage CAD-only. Il profilo CAD dichiara la normativa e riutilizza i servizi core.

## Prerequisiti
T-CAD-02 e P13/P10; P0 CAD quando la task tocca note normative; verifica prerequisiti sullo stato reale e segnala `BLOCKED` se mancanti.

## Obiettivo e comportamento richiesto
C.4.7=1 dopo data conclusione B1.2.6 → errore per qualunque motivo, non lo stesso giorno; dopo decesso (2.7=3/4/7/82) vietate sedi 1/2/4/5/6, ammessa 3 visita cordoglio. Test accessi prima/same-day/dopo, conclusione mancante e più B1 stesso evento.

## Fonti obbligatorie
- `SOURCE_DOCUMENTS/ARCHITECTURE.md` §45; `SOURCE_DOCUMENTS/DETAILED_DESIGN.md` §166–174; `PERSISTENCE_CORE_SPEC.md`.
- `CAD_REVISED_CANDIDATE/`, `CAD_NOTES_AUDIT.md`, `CAD_FIXTURE_PLAN.md`, PDF regionale in `SCHEMA_INPUTS/CAD_2024-06_SOURCE`.
- `SOURCE_DOCUMENTS/MASTER_IMPLEMENTATION_PLAN.md` §14, §20; `IMPLEMENTATION_STATUS.md`.

## Gate ed evidenze
Test sintetici positivi/negativi e edge case, output deterministico, policy dichiarative in schema/version, no hardcoding CAD/SIOC nel core, nessuna PHI. Esegui realmente i comandi Cargo disponibili; documenta test Windows non eseguiti e blocchi. Aggiorna stato e handoff, quindi prosegui automaticamente con la prossima task READY secondo AGENTS.md.
