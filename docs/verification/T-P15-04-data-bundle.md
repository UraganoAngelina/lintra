# T-P15-04 — Bundle dei dati

## Preflight e piano —2026-10-26

Prerequisito unico15-03 verificato: commitc538493, CI40339655671 Linux/MSVC
completed/success letti14:38:47 UTC. Prompt atomico, ARCH§39C.2, manifest
authoring e ratifica CNF-BUNDLE-AUDIT-01 A consultati. Nessuna pubblicazione
richiesta da questa task; DEC-08/09/11 dei workflow release restano aperte.

Perimetro: comando xtask sequenziale che confeziona un runtime bundle esplicito
con manifest, payload dichiarati verificati e SHA256SUMS; archivio deterministico
schemas-<bundle_version>.zip. Nessuna conversione flow_schema.json, scansione
arbitraria o dataset ufficiale inventato. Fixture positiva: snapshot sintetico
esistente; test negativi integrità/path/symlink/output e prova del bundle
estratto con eseguibile già compilato. Formattazione, lint/test prodotto e
tooling, CI Linux/MSVC secondo protocollo, senza dichiarare gate anticipati.

Stato IN_PROGRESS, sviluppo single-agent autorizzato;92 DONE, M5 aperta.
