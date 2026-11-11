# T-MIG-P0-05 — OS minimi e distribuzione offline

Data11 novembre2026. Stato: **DONE per recepimento della decisione OS**.
Prerequisito P0-03 DONE92d6d3d. Ingressoeaf989b clean; runtimecb70bee con
CI42990775714 qualificata. Nessun installer o GUI creato in questa task.

ADR-MIG-002 recepisce autore/fonte/data D14/DEC-MIG-02: Windows11 x86_64,
MSVC/NSIS perMachine offline, WebView2 offline completo, install elevata e run
ordinaria non-admin, due account, aggiornamento manuale, dati/chiave conservati
all'uninstall. Default per-account, backup verificato e nessuna migrazione
silenziosa. Linux solo personale CachyOS/Hyprland/Wayland; niente release GUI
Linux ufficiale. Nessuna scelta di prodotto nuovamente richiesta.

ARCH§39A, DET Addendum A e nota nel master delimitano le eccezioni GUI-only.
N-19/N-20/N-23 restano identiche per core/CLI. Non si deduce supporto Win10/
Server GUI né si promette portabilità AppImage/deb. Non si inventano versioni
minime WebView o config Tauri già verificata: i dettagli/payload reali sono P1/P8.

Preflight host effettivo: /etc/os-release identifica CachyOS rolling;
pkg-config non trova webkit2gtk-4.1. È un prerequisito futuro dello spike
Linux P1-01, non una qualifica Tauri né un blocco alla ratifica OS. Il comando
aggregato non prova la versione GTK e non viene presentato come tale.
Win11 clean-room/non-admin/due account/WebView2/NSIS: NOT_RUN. La CIWindows2025
cb70bee non sostituisce tali prove, né il job UI archive opzionale skipped.

Verifica: fonte/master/prompts e ARCH/DET letti; link locali, confronto righe
N-19/20/23, check_unified_plan.py e diff check. Soli documenti: nessun nuovo
test runtime/CI richiesto/attribuito. Nessun artefatto pubblicato o tag creato.
G-MIG-0 attende06/07/08. Next READY P0-06, AUTO_CONTINUE single-agent.
