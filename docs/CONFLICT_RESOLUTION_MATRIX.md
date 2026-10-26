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

Chiusura T-P13-08 verificata2026-10-25 alla ripresa: CNF-SIOC-REFERENTIAL-G0-01
implementata in structural-minimal.2/componente tracciato, gate locale611 test e
CI40192491629 Linux/MSVC success sul codice52762c8. Nessuna modifica alle fonti
protette; rinvii/capability del completo conservati. Nuovo mandato supera lo stop.

### DEC-18 — ratifica successiva,2026-10-26

Decisione del registro Master Plan risolta dall'autore senza riscrivere il piano:
[ratifica definitiva](verification/DEC-18-ratification.md). Autorizzata T-P14-05:
SchemaCatalog separato da load_raw, inventory reference schema-scoped con stati
Verified/Degraded/Unavailable reali, import draft canonical/lint e warning Registry.
Tooling exit0/1, mai2 per authoring. Listing/import non certificano conformità.
Nessun cambiamento a norme/capability/Strict; DONE/M7 solo dopo gate effettivi.
STOP al termine14-05, nuova autorizzazione necessaria per task successive.

Applicazione DEC-18 verificata2026-10-26: T-P14-05 DONE e M7 raggiunta nel
perimetro tooling;671 test/77 suite locali e CI40298536590 Linux/MSVC success
su8a9ffd8. Primo gate fallito per path non canonical nell'atteso del test Windows,
correzione e diagnosi registrate nel [report](verification/T-P14-05-schema-tooling.md).
Nessuna semantica normativa modificata; nessuna task successiva avviata.

### CNF-ARTIFACT-LINUX-01 — aperto,2026-10-26

Preflight T-P15-02 avviato automaticamente dopo T-P15-01 DONE/CI40307076976.
ARCH§39C.1 e DET A.3 limitano le dipendenze CLI Linux alla «libc di base» senza
allowlist esplicita; l'artefatto GNU locale importa anche libgcc_s.so.1 (runtime
GCC). Nessuna eccezione ratificata trovata; DEC-11 Linux resta aperta. Non si
estende il gate né si cambia linking/panic/target implicitamente.
[Proposta A circoscritta e alternativa](proposals/CNF-ARTIFACT-LINUX-01.md),
[evidenza e handoff](verification/T-P15-02-preflight.md). Stato PROPOSTO,
non approvato/non implementato;15-02 BLOCKED al preflight. Nessuna modifica
normativa o architetturale applicata. Governance permanente single-agent e Git/CI
autorizzati conservati: la pausa riguarda soltanto una decisione contrattuale reale.

### CNF-ARTIFACT-LINUX-01 — ratifica A successiva,2026-10-26

L'autore approva A: Linux previsto solo interno/personale, distribuzione ad altri
via Windows. [Ratifica](verification/CNF-ARTIFACT-LINUX-01-ratification.md).
Allowlist CLI GNU chiusa: libc.so.6, libm.so.6, ld-linux-x86-64.so.2,
libgcc_s.so.1; altro SONAME rifiutato. Windows invariato, nessuna scelta implicita
di baseline Linux o riduzione dei gate. Preflight15-02 sbloccato, sviluppo ripreso.

### CNF-BUNDLE-AUDIT-01 — aperto,2026-10-26

T-P15-03: ARCH§39C.2/DET A.1 richiedono bundle_source/bundle_version, mentre il
contratto CLI consegnato ammette root esterne senza manifest e anche con parent
distinti. Nessuna ratifica su versione assente o rottura di quella configurazione.
CNF-REFERENCE-FORMAT-01 mantiene Strict/Continue e non viene riaperta: il manifest
non può annullarne le evidenze con un abort globale indiscriminato.
[A compatibile con null esplicito/B manifest obbligatorio](proposals/CNF-BUNDLE-AUDIT-01.md).
Stato PROPOSTO, non ratificato; wiring/audit non implementati, backend preservati.

### CNF-BUNDLE-AUDIT-01 — risolto, ratifica A,2026-10-26

Approvazione esplicita dell’autore: [contratto ratificato](verification/CNF-BUNDLE-AUDIT-01-ratification.md).
Ripresa T-P15-03; audit additivo, manifest esterno opzionale, policy reference invariate.

### DEC-08/09/11 — preflight release15-05, ancora aperte

Non sono nuovi conflitti runtime: prerequisiti umani espliciti del piano.
Proposta concreta [DEC-RELEASE-SETUP-01](proposals/DEC-RELEASE-SETUP-01.md),
nessuna ratifica presunta.15-04 prosegue indipendentemente con il proprio gate.
