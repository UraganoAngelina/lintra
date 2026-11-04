# T-PERSIST-12 — preflight

4 novembre 2026. Prerequisiti09/10/11 realmente qualificati nei rispettivi
ledger nativi, tutti completed/success.17-02 attende CI41846160280 ed è indipendente.
Fonti: ARCH45, DET166–174, ADR-PERSIST-001, piano4/14/20, prompt12 e ratifica
PERSIST-KEY-PROVISIONING-01 aggiornata. Nessun gate aggirato o nuova decisione.

File previsti: helper sintetico condiviso in tests/support, benchmark dedicato
memory/SQLite (Cargo target senza nuovi crate), test storage/audit/privacy,
documento operativo retention/ripristino e report di qualifica. Nuova CI solo
quando17-02 terminata per non cancellarne il gate. Smoke su entrambi gli OS.

Carichi cinque trace con arità PK [3,3,4,4,2] e[2,2,2,5,5] derivate dalle forme
SIOC/CAD in ADR-PERSIST-001, con larghezze/dati arbitrari sintetici dichiarati.
Misurano storage/protezione/transazioni, non conformità normativa o motore CAD.
Volumi fino a centinaia di migliaia, misure temporali/RSS/disco senza soglie.

La review sicurezza distingue digest HMAC da cifratura, namespace/metadati in
chiaro e campi protetti. Sentinelle sintetiche vietate nei file storage e audit;
nessun valore operativo generato. Retention senza purge implicito, ripristino
manuale offline qualificato sui test esistenti; politiche per dati reali aperte.
Negativi: chiave errata, file corrotto/permessi non sicuri, nessun fallback/storico
alternativo o sostituzione dell'archivio attivo. Handoff CAD senza norme inventate.
