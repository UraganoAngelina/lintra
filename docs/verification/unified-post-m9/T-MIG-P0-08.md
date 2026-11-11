# T-MIG-P0-08 — Verifica governance e avvio autorizzato

Data11 novembre2026. Stato: **DONE**, G-MIG-0 **PASS/APPROVED nel proprio scope**.
Preflight4e0233b; P0-01..07 tutti DONE con dossier e requisiti soddisfatti.
Runtime di ingressocb70bee; nessuna implementazione Tauri avviata prima del gate.

Verificati ADR001..004 con owner/fonte/data e ratifica già ricevuta, aggiornamenti
N-24/ARCH/DET, scope LocalOnly+Full, OS, threat model, brief UX e misure senza
soglie inventate. Registro corrente MIGRATION_DECISIONS_CURRENT collega i9DEC
e i6CNF: nessun blocco della fase0, senza riscrivere la fonte30 ottobre o
trasformare i rischi futuri in successi. Clap resta package-vs-module;
build.rs Git è provenance di build, non requisito di esecuzione.

Verifica tecnica: baseline/audit antenati, CI42990775714 success7 job, log e
hash archiviati;21 oracle SIOC3 volte/OS e ACL Windows3 volte. Locale1096pass/
7ignored,fmt/Clippy/24Python qualificati. Tutti520 input uguali al commit di
codice qualificato; original140/140 e15 fonti/dataset congelati preservati.
Il checker conferma128nodi aciclici/fonti/prerequisiti/dossier, non inventa prove
runtime. Nessun confronto con una CI antenata di codice diverso.

G-MIG-0.md riporta PASS solo per governance/ingresso agli spike. La ratifica
non rende PASS i gate successivi. Win11 clean-room/WebView2/NSIS e non-admin/
due account sono NOT_RUN; WebKitGTK4.1 manca sul PC CachyOS e va soddisfatto
inP1. I residui normativi/applicativi SIOC e dati per-account non sono aggirati.
Nessun gate enterprise o altra ratifica globale richiesti per costruire la UI.

Delta solo documentale: gate, registro corrente, checks, status e dossier.
Controlli macchina salvati nel JSON, link/diff/check_unified_plan. Nessun nuovo
runtime/test/CI necessario per questo delta; le nuove modifiche P1 avranno
le proprie prove e limiti. Nessun tag/release o decommission Slint.
Next READY T-MIG-P1-01; AUTO_CONTINUE single-agent.
