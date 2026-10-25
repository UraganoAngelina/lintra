# Matrice di riconciliazione dei documenti Lintra

**Data approvazione:** 2026-10-16 · **Ambito:** precedenti conflitti tra piano, ARCH, DET e persistency CAD. La risoluzione è **documentale**: compilazione e test Rust rimangono per le task successive.

| ID | Incongruenza | Risoluzione nel pacchetto | Stato |
|---|---|---|---|
| CNF-01 | MVP vs gate intermedio | DET §151 = M3, ARCH §43 = MVP M5 | DOCUMENTED_RESOLVED |
| CNF-02 | DSL vs reference | P10 e P11 indipendenti, ordine dai prerequisiti | DOCUMENTED_RESOLVED |
| CNF-03 | GUI | Slint, egui solo dopo spike; prototipo non MVP | DOCUMENTED_RESOLVED |
| CNF-04 | Package / workspace / servizi | Single package P1, workspace P16; dependency injection e bundle | DOCUMENTED_RESOLVED |
| CNF-05 | Codici errori | Solo E_* simbolici stabili; legacy numerici non pubblici | DOCUMENTED_RESOLVED |
| CNF-06 | Naming | Lintra / lintra / lintra-* | DOCUMENTED_RESOLVED |
| CNF-07 | ValidationRequest | DET §25 normativo; esempio portabilità illustrativo | DOCUMENTED_RESOLVED |
| CNF-08 | Report XLSX | XlsxReportWriter da JSONL; non sink nel loop | DOCUMENTED_RESOLVED |
| CNF-09 | CI Windows Linux | --locked e runner Linux baseline CI | DOCUMENTED_RESOLVED |
| CNF-10 | RequiredUnless | Compile in RequiredIf(Not(E)) | DOCUMENTED_RESOLVED |
| CNF-11 | ChannelSink vs JSONL UI | JSONL tabella; channel soltanto progresso/contatori | DOCUMENTED_RESOLVED |
| CNF-LINTRA-SOURCE-01 | Missing §39A-D Addendum A/B | Adottate versioni del 2026-10-09 e integrate | DOCUMENTED_RESOLVED |
| CNF-PERSIST-01 | CAD-only vs core persistence | Core generico + policy SIOC/CAD diverse; ADR-PERSIST-001 | DOCUMENTED_RESOLVED |

## Fonti di base effettive

- ARCH rev. 2026-10-09, prima disponibile come `regulatory_fixed_width_validator_ARCHITECTURE(1).md`, da cui è tratta la parte §39A–D; revisionata ulteriormente con §45.
- DET rev. 2026-10-09, prima disponibile come `regulatory_fixed_width_validator_DETAILED_DESIGN(1).md`, con Addendum A/B, revisionata con §166–174.
- MASTER_IMPLEMENTATION_PLAN v1.0, da cui deriva v1.1 per esplicita autorizzazione dell'autore; gli ID delle 116 task esistenti sono invariati.
- `PERSISTENCE_CORE_SPEC.md` come ADR-PERSIST-001, decisione nuova approvata esplicitamente.

## Fuori da questo gate documentale

Le task di implementazione e le decisioni tecniche su backend storage/gestione chiavi, le norme SIOC cross-run (`DEC-SIOC-HIST-01`) e i restanti gate CAD-runtime restano da eseguire. Sono task e decisioni aperte, **non conflitti tra fonti**.

## Ratifiche successive — appendice cronologica

| ID | Conflitto | Decisione successiva | Stato |
|---|---|---|---|
| CNF-SIOC-REFERENTIAL-G0-01 | Obblighi parent G0 senza regole esplicite; atto revocante negativo N/A contro obbligo E012 | Nuova revisione operativa derivata con quattro REL-* e policy revoking_act corretta; no controlli impliciti; .1/FROZEN invariati; sottoinsieme golden tracciato autorizzato | APPROVED 2026-10-25; [ratifica](verification/CNF-SIOC-REFERENTIAL-G0-01-ratification.md); implementazione/gate da verificare |
