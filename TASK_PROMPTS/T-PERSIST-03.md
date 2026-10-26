# PROMPT ATOMICO — T-PERSIST-03: Trait del repository persistente e adapter in memoria

Sei un coding agent del progetto **Lintra**. Esegui ESCLUSIVAMENTE **T-PERSIST-03** sotto le regole del `MASTER_AGENT_PROMPT.md`, del piano §14 e dell'ADR `PERSISTENCE_CORE_SPEC.md`. Dopo gate e handoff prosegui automaticamente con la prossima task READY, senza delega, secondo AGENTS.md.

## Prerequisiti vincolanti
T-PERSIST-02 e P13. Verifica che i prerequisiti siano segnati DONE **con evidenze effettive**, non soltanto con una casella spuntata. Se non soddisfatti, segnala `BLOCKED`.

## Obiettivo atomico
Implementa PersistentContextRepository e PersistenceTransaction con fake in-memory che simuli restart, commit, abort e compare-and-swap; estrai interfaccia distinta dal ContextStore a due passate. Test di stage/commit/rollback, stato non modificato prima di commit, generazione errata.

## Contratti da rispettare
`SOURCE_DOCUMENTS/ARCHITECTURE.md` §45, `SOURCE_DOCUMENTS/DETAILED_DESIGN.md` §166–174, `SOURCE_DOCUMENTS/MASTER_IMPLEMENTATION_PLAN.md` §20 e le fonti relative al modulo interessato. Persistenza generica, policy normative esterne, `ContextStore` intra-run separato da `PersistentContextRepository`, errori normativi distinti da errori I/O, determinismo, no PHI nei test.

## Test, accettazione e handoff
1. Dichiara file e API da toccare prima di implementare; scrivi unit/integration test con casi positivi/negativi ed edge case indicati sopra.
2. Esegui `cargo fmt --all -- --check`, `cargo clippy --all-targets --locked -- -D warnings`, `cargo test --locked` dove disponibili; registra output reale. Non dichiarare CI Windows verde senza runner.
3. Aggiorna `IMPLEMENTATION_STATUS.md` con commit/git status se disponibili, rischio privacy, conferme DEC/ADR e identificativo della prossima task; registra la prosecuzione automatica verso la prossima task READY secondo AGENTS.md.

## Esclusioni
Nessuna implementazione hardcoded SIOC/CAD nel core; non scegliere database per comodità; nessun `unwrap` su dati esterni; non leggere o committare i file reali dei pazienti; non cambiare schema SIOC FROZEN né promuovere CAD a FROZEN.
