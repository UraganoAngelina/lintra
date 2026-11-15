# T-MIG-P3-01 — Scaffold desktop affiancato

15 novembre2026. **IN_PROGRESS: Linux PASS, Windows CI pendente**.
Ingresso: G-MIG-2 approvato dall’autore, P2-10 DONE. Nuovo eseguibile in
apps/lintra-desktop, workspace/lock Rust e frontend propri. React/TS strict,
asset locali, capability vuote, finestra creata con guard prima del documento.
Guard WebView2 P1-04 riusato byte-identico; nessuna validazione ancora collegata.

Build frontend e install offline senza lifecycle PASS; build release Linux
PASS (508s prima compilazione di quel profilo). Due test Rust sugli origin e
header PASS, usando cache dev esistente (3,33s). Lock Cargo/npm senza nuove
versioni registry rispetto allo spike. [Receipt con SHA](T-MIG-P3-01-checks.json).
CLI/Slint/core/test/manifest e lock principali invariati rispetto a45fce38.

Workflow Windows condizionato ai path dell’adapter, fmt/test/clippy/build
release e artefatto. Cache ufficiale GitHub pinnata e separata per OS/toolchain/
lock; nessun controllo omesso sui cache hit. Manifest/istruzioni ufficiali
cache verificati sul commitcaa296126883cff596d87d8935842f9db880ef25.
Windows Server CI non è Windows11/installer o qualifica del prodotto.
Nessun DONE finché la build Windows non supera i controlli; G-MIG-3 aperto.
