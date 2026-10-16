# Avvio del primo agente Lintra — T-P1-01

Prima di qualunque modifica leggi `MASTER_AGENT_PROMPT.md` e `SOURCE_DOCUMENTS/{ARCHITECTURE,DETAILED_DESIGN,MASTER_IMPLEMENTATION_PLAN}.md`, consultando le sezioni necessarie, più `PERSISTENCE_CORE_SPEC.md`, `CONFLICT_RESOLUTION_MATRIX.md` e `IMPLEMENTATION_STATUS_SEED.md`.

**I conflitti documentali precedentemente segnalati sono RISOLTI** e l'autore ha autorizzato la revisione del piano. **La persistenza è una capacità del CORE** (non soltanto CAD), ma in questo task ne predisponi solo il `src/persistence/mod.rs` vuoto senza scegliere database o implementare API. Nome prodotto/Cargo `lintra`.

**Esegui ESCLUSIVAMENTE `TASK_PROMPTS/T-P1-01.md`.** Il repository non è ancora inizializzato. Autorizzata eccezione A-02 (avvio P1 prima della chiusura di G0). Crea scaffold `Cargo.toml`, `src/lib.rs`, `src/main.rs`, moduli vuoti, `docs/` con architettura, detailed design, master plan e ADR, `.gitignore` difensivo e `IMPLEMENTATION_STATUS.md` dal seed.

**Non implementare** schemi compilati, runtime normativo SIOC/CAD, persistenza backend, UI Slint, report XLSX. Non importare A/B1/C/D reali.

Esegui `cargo build` e gli altri controlli disponibili, registra output reale, status, HANDOFF e git status. Fermati al termine della task e attendi indicazioni.
