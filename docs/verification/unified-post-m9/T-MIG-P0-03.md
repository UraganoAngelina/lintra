# T-MIG-P0-03 — ADR di sostituzione mirata N-24

Data11 novembre2026. Stato: **DONE documentale/ratifica recepita**.
Prerequisito P0-02 DONE e verificata e41e434/56270b0. Preflight56270b0 clean,
code baselinecb70bee e CI42990775714 qualificati; nessun nuovo runtime.

`docs/adr/ADR-MIG-001.md` registra owner autore, decisione10 novembre, fonte
chat recepita in ADR-POST-M9-001/U-GOV-03 `2cd4bdf`, trascrizione agente11
novembre e limiti dei gate. L'autore ha già approvato lo stack e lo scope:
la condizione «ADR approvato dall'autore» si soddisfa tramite recepimento fedele,
non inventando una nuova firma né riproponendo il Decision Pack come pendente.

Il delta è mirato: nuovo adapter React/TS strict/Vite/Tailwind/shadcn/TanStack/
Tauri2 dopo G-MIG-0; Slint resta sino a parità/rollback. Client del vero servizio
Rust, IPC bounded, una validazione per istanza, Full obbligatorio e CAD/SIOC nei
propri gate. Nessun refactor core/CLI o package-clap richiesto. Nessun renderer
software/pixel equality cross-WebView impliciti. Il testo precedente N-24 è
conservato nell'ADR e nelle sezioni legacy ARCH/DET.

ARCH§39D e DET Addendum B espongono l'applicabilità corrente; DET§98 rinvia
all'ADR invece di imporre Slint al nuovo adapter. La voce N-24 del master
originale cambia soltanto per il nuovo perimetro ratificato. Nessuna delle140
task/stati originali viene riaperta o riscritta. N-19/N-20/N-23 invariati;
OS/security e firme IPC concrete sono nelle task successive, non già qualificate.

Verifiche: fonte ratifica/master e DEC-MIG approvate; confronto testuale legacy,
link locali, check_unified_plan.py e diff check. Nessun test/CI runtime nuovo
necessario per questo delta documentale. Nessun G-MIG-n PASS dalla sola firma.
G-MIG-0 attendeP0-04..08; next READY P0-04 (scope/capability), AUTO_CONTINUE.
