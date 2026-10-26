# PROMPT ATOMICO — T-CAD-02: Sessioni trimestrali, ULSS e chiavi CAD

Sei coding agent Lintra. Implementa **soltanto T-CAD-02**. Il contratto storico generico è **ADR-PERSIST-001 / `PERSISTENCE_CORE_SPEC.md`**, non un plugin storage CAD-only. Il profilo CAD dichiara la normativa e riutilizza i servizi core.

## Prerequisiti
T-CAD-01, T-PERSIST-02; P0 CAD quando la task tocca note normative; verifica prerequisiti sullo stato reale e segnala `BLOCKED` se mancanti.

## Obiettivo e comportamento richiesto
Definisci il profilo CAD: namespace ULSS continua tra anni, trimestre, A PK(1.1,1.23), B1 PK(2.1,2.36), B2 PK(3.1,3.5), C PK(4.1,4.2,4.4,4.5,4.11), D PK(5.1,5.2,5.3,5.4,5.7). Test isolamento aziende/eventi e composizione PK.

## Fonti obbligatorie
- `SOURCE_DOCUMENTS/ARCHITECTURE.md` §45; `SOURCE_DOCUMENTS/DETAILED_DESIGN.md` §166–174; `PERSISTENCE_CORE_SPEC.md`.
- `CAD_REVISED_CANDIDATE/`, `CAD_NOTES_AUDIT.md`, `CAD_FIXTURE_PLAN.md`, PDF regionale in `SCHEMA_INPUTS/CAD_2024-06_SOURCE`.
- `SOURCE_DOCUMENTS/MASTER_IMPLEMENTATION_PLAN.md` §14, §20; `IMPLEMENTATION_STATUS.md`.

## Gate ed evidenze
Test sintetici positivi/negativi e edge case, output deterministico, policy dichiarative in schema/version, no hardcoding CAD/SIOC nel core, nessuna PHI. Esegui realmente i comandi Cargo disponibili; documenta test Windows non eseguiti e blocchi. Aggiorna stato e handoff, quindi prosegui automaticamente con la prossima task READY secondo AGENTS.md.
