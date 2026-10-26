# Lintra — Percorso d'implementazione con persistenza core

**Task atomiche e sequenziali con un solo agente principale. Dopo verifica dei gate, avviare automaticamente la prossima READY secondo AGENTS.md; nessuna delega.** Gli identificativi P0–P18 precedenti sono invariati.

```mermaid
flowchart TD
  P1[P1 repository e CI] --> P2[P2 dominio]
  P2 --> P3[P3 schema raw/compiled]
  P3 --> P6[P4-P6 input/report/validation]
  P6 --> P7[P7 ApplicationService]
  P7 --> P10[P9-P11 DSL e lookup]
  P10 --> P13[P13 cross-record e ContextStore effimero]
  P13 --> PER[PERSIST-01..12 storage generico]
  PER --> SIOC[SIOC persistent-key audit]
  PER --> CAD[CAD-01..12 historical/temporal profile]
  CAD --> G[Golden Rust e gate CAD runtime]
  P7 --> P8[P8 CLI]
  P8 --> P12[P12 XLSX/CSV]
  P12 --> P16[P16 Slint UI]
```

Le sequenze non impediscono di preparare i **tipi/trait** PERSIST nelle task P2/P3/P7 quando è nello scope; non significa anticipare backend o plugin. SIOC già include deduplicazione intra-trace in JSON FROZEN; tale controllo si può esercitare non appena sono pronti il compiler e il flow validator.

## Gate separati
- **Gate di documentazione:** questo kit contiene ARCH/DET/PLAN già riconciliati, verificabile offline.
- **Gate di implementazione storage:** T-PERSIST-01..12, test crash/restart/atomic commit/privacy.
- **Gate normativa SIOC:** PK intra-run già FROZEN; DEC-SIOC-HIST-01 open solo per enforcement cross-run.
- **Gate normativa CAD:** DRAFT/RELEASE_CANDIDATE finché non si compila e valida il profilo e non si chiude il gate umano.
