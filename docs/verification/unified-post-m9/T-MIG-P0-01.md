# T-MIG-P0-01 — Baseline tecnica post-audit

Data: 2026-11-10, Europe/Rome. Prerequisito integrativo U-UI-01 `4f758d3`.
Stato corrente11 novembre: **WAITING_GATE**. Le due CI reference sono
fallite su Windows; anche CI42989468054 è FAIL; nuova CI42990775714 sucb70bee in corso. G-MIG-0
NOT_VERIFIED. La fotografia del10 novembre riportata sotto resta storica.

Baseline esecutiva scelta: **dfc36044e0315f3ab060f98594192d3d1dfcad95**,
branch work/p1-foundations, working tree pulita alla fotografia. af3bccc è
antenato confermato; tutte le140task originali restanoDONE. Nessun WIP08 da
isolare: l'ultimo delta runtime è76fa08a, con API/codice committati.
Differenza76fa08a→dfc3604 vuota su src/ui/tests/manifest/lock/build/scripts/
schemas/bundles/CI. Comandi/output/hash in T-MIG-P0-01-baseline.json.

| Prova | SHA / stato reale |
|---|---|
| Audit SQLite baseline | af3bccc, CI42789338814 success |
| Ripetizione nativa regressioni | 4c169cd, CI42823787199 success, log3pass per caso/OS |
| Budget completo locale | 4864c34,1073pass/7ignored,125riepiloghi,fmt/Clippy |
| Budget nativo | 539b9bc,CI42829144043 success, nuovi test3volte/OS |
| Reference nuovo locale | 76fa08a,89pass/zero fail,fmt/Clippy,447input invariati |
| Reference nuovo nativo | **dfc3604,CI42834962873 avviata; PENDING**, non ereditata dalla CI budget |
| Windows type-check isolato ACL | PASS, solo metadata del modulo/test; prodotto GNU bloccato da MinGW mancante |
| Tauri/WebView/installer/Windows11 clean-room | NOT_RUN; nessuna qualifica dedotta da M9 o Windows2025 CI |

La diff da539b9bc include sei file source/test del controllo reference: non è
sola documentazione e richiede la propria CI. La baseline non è promossa a
qualificata prima dell'esito nativo; P0-02 dipendente non viene avviata con un
DONE fittizio. Nessuna implementazione Tauri prima di G-MIG-0.

Nessun sorgente, norma, golden o milestone originale modificato dalla task:
solo baseline JSON/dossier/stato. Aggiornare gate quando la CI nativa reale
è completata e verificata sui log. U-NORM-05 ha una domanda mirata distinta,
non blocca questa fotografia tecnica né il percorso UI sintetico indipendente.
Next READY indipendente: U-OPS-05 (chiave/recovery), poi U-OPS-07 applicabile.

## Frontiera corrente11 novembre

CI42834962873 e42840799653: Windows FAIL, Linux PASS. La fixture CLI e la
preparazione dei permessi del runner sono corrette senza bypass runtime.
Candidate corrente `97f76883a339f2b5401e1b68809f5c2cf9f6bfd1`, con il nuovo
produttore SIOC normativo: nessuna equivalenza implicita con dfc3604.
21 test mirati e fmt/Clippy PASS; regressione completa in corso.
CI42989468054 sullo stesso SHA in corso. La promozione richiede esito e log
pertinenti; P0-02 resta non avviata. Né Windows2025 né CachyOS personale
sostituiscono Windows11 clean-room/installer. U-NORM-05 è ora DONE con
addendum-A01, senza più decisioni ordinarie pendenti.

Frontiera successiva:97f7688 non qualificato (CI42989468054 FAIL su entrambi
gli OS). Candidate `cb70bee19f2d470c3ac2e98828343b32e23b3f0a`,
CI42990775714 in corso; preparazione/lettura file Windows e test Python
superati, niente PASS nativo dedotto finché il workflow non termina.
