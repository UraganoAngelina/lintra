# U-GOV-02 — Riconciliazione delle fotografie storiche

Data: 2026-11-10, Europe/Rome. Stato: DONE. Prerequisito U-GOV-01:
commit `8e6bc55`, baseline e CI verificati nel relativo dossier.
G-U0 aggregato ancora NOT_VERIFIED, nessun gate runtime nuovo.

## Matrice di riconciliazione

| Fonte / affermazione 30 ottobre | Riscontro corrente | Esito operativo |
|---|---|---|
| HEAD fe5ada8 e WIP PERSIST08 | HEAD iniziale 7e2ff155, clean; af3bccc incluso | Obsoleta come baseline; usare U-GOV-01 |
| 117/140, M9 non raggiunta | IMPLEMENTATION_STATUS frontiera: 140/140 DONE, QG-8/M9 | Storico preservato, non correggere retroattivamente il preflight |
| PERSIST08 non qualificata | T-PERSIST-08 e PERSIST09–12 conclusi nel piano storico | Nessun isolamento WIP ancora necessario; mantenere le API qualificate |
| CAD non implementato | src/application/snapshot_validation.rs::validate_snapshot, dossier CAD12 e CI42641969728 | CAD83/C1/L1 library qualificati; nessuna UI CAD implicita |
| SQLite approvato, DELETE/EXTRA, CAS | src/persistence/sqlite/mod.rs, PRAGMA e repository attuali | Contratto valido, aggiungere regressioni af3bccc e future prove D15 |
| Rischi current_generation e lookup | af3bccc, audit_generation_read_is_coherent_across_concurrent_publication / audit_identity_lookup_rejects_dangling_latest | Due difetti corretti; conservare regressioni senza refactoring |
| UI Slint un file LocalOnly | ui/src/analysis.rs::prepare usa Trace/LocalOnly | Ancora attuale; Full, storico e approval non sono schermate già disponibili |
| Core separato da UI, clap nel package | Cargo.toml workspace root+ui, dipendenza clap nel package lintra | Chiarire package rispetto ai moduli; nessun refactor del core per slogan |
| Nessuno scaffold React/Tauri prodotto | Cargo.toml membri root+ui; apps/lintra-desktop assente | Ancora attuale; nessuna fattibilità Tauri già provata |
| Risultati bounded, indice e filtri Rust | ui/src/results; contratto MIG preservato | Riutilizzare logica applicabile, non esportare tutto JSONL o indice in JS |
| N-24 Slint; WebView non qualificato | ARCH §39D, DET Addendum B, piano N-24 | Nuovo stack autorizzato; formalizzazione U-GOV-03 e MIG-P0, prove MIG-P1/7/8 ancora necessarie |
| Parità visiva RGB Slint | Test legacy e CI42789338814 positivi | Vale per Slint; React ha parità comportamentale e prove WebView per OS |
| DEC-SIOC-HIST-01 audit-only | ARCH §45, DET §170, MASTER_AGENT_PROMPT; D11 successiva | Preservare vecchi profili; supersessione prospettica in U-GOV-03/U-NORM-05 |
| Sourcing di dataset / SIOC regionale completo | D12: dataset attuali definitivi, deferred trasparenti | Non aprire sourcing o inventare cataloghi; sviluppo normativo nuovo solo D11 o difetti provati |
| Cifratura/retention/policy ancora da scegliere | D03/D05/D06/D07 ratificate il 10 novembre | File per account, ACL, chiave comune, nessuna nuova crypto/retention; Backup-B verificato |
| Export shared o scope operativo indecisi | D02/D04/D14 ratificate | Uso interno effettivo; locale libero, shared con avviso e consenso; nessun gate enterprise |
| Power-loss solo limite non verificato | D15 requisito ingegneristico approvato | U-REL-03, kill/fault/reopen; hardware non provato resta NOT_TESTED_HARDWARE |
| Full MIG obbligatorio, CAD fuori MIG | Master unificato §6–9 | 89 task conservate; 9 U-CAD aggiungono storico dopo U-UI-02; G-REL richiede entrambi |
| Preview tecnica ≠ distribuzione | D14, G-REL | Installer Windows11 offline realmente qualificato, due account, update/uninstall conservativi |

Il preflight originale e i report di audit restano immutati. Le decisioni di
prodotto del 30 ottobre sono confermate dal mandato del 10 novembre; i gate non
sono dedotti dalle decisioni. Le norme e i profili legacy non vengono riscritti.

## Verifica / delta

Letture effettive: kit master/zero/audit/gap/roadmap/preflight/migration plan e
decision register, governance repository, frontiera, Cargo.toml, simboli citati.
`git diff af3bccc HEAD` sui percorsi di prodotto vuoto prima di questa task.
`git diff --check` pertinente alla sola documentazione; nessuna nuova CI runtime
necessaria o attribuita. File: questo dossier e stato unificato.
SHA task: cronologia del dossier; nessun hash autoreferenziale.
Nessun blocker. Next READY U-GOV-03, AUTO_CONTINUE.
