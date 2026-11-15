# T-MIG-P4-01 — Catalogo e selezione nel backend

15 novembre 2026. **DONE nel perimetro API catalogo/selezione**; prerequisito
P3-06/G-MIG-3 verificato. [Receipt](T-MIG-P4-01-checks.json).

`AnalysisBundle` è spostato da `ui/src/analysis.rs` nella library, in
`src/bundle_acquisition.rs`. Il percorso pubblico Slint lo riesporta: stessi
costruttori, risoluzione nativa, precedenza dei bundle, verifiche manifest,
ordine, errori e audit. Repository/reference sono condivisi tramite Arc;
non si importa Slint nel nuovo adapter e non si duplica l'acquisizione.
Otto test esistenti di configurazione Slint qualificano i chiamanti reali.

Il dispatcher espone `catalog_page`, `configuration_select` e
`configuration_page`, con DTO v1 esatti. Catalogo e descrittori hanno pagine
fisse di massimo 64 elementi, offset u64 testuali canonici e cap intero 1 MiB.
Nessun caricamento da path fornito in JS, raw schema o dataset trasferito.
La provenienza deriva dal BundleAudit reale; i metadati non dichiarano uno
schema certificato o un esito di validazione.

La selezione richiede schema_ref ed encoding utf8 espliciti. Il riferimento
appartiene al catalogo di questa istanza nativa, con nonce OS 128 bit; nessuna
selezione automatica dell'unica/ultima versione. Una singola acquisizione raw
alimenta snapshot in memoria, compilazione, hash e metadati. La configurazione
catturata non rilegge il file schema quando si consultano i descrittori.
Nuova selezione invalida i riferimenti vecchi; overflow, errori o risposta fuori
cap conservano la selezione precedente. Stato e raw snapshot restano Rust.

Il client TS verifica versioni anche annidate, shape senza extra, provenienza,
hash, riferimenti, conteggi u64 e paging. Non compila o interpreta le regole.
I DTO vengono decodificati prima dell'acquisizione; owner/origine, chiusura,
permission e concorrenza restano controlli Rust. Unica finestra main creata
dall'app, nessuna API per crearne altre; chiusura irreversibile del contesto.

Livello A/B locale: 16 test Rust nativi Linux, 20 Node, lint/typecheck/build
frontend e 8 test Slint `analysis_configuration` PASS, exit 0. Coperti paging
64/70, offset invalidi, riferimenti estranei/stale, snapshot e hash, overflow,
path/owner forgiati e JSON arbitrario. Le versioni registry dell'adapter sono
un sottoinsieme dei due lock già qualificati; encoding_rs resta 0.8.35.
Nessuna CI workspace/VM per questa micro-task: Windows e smoke del nuovo
adapter al checkpoint del blocco coerente, G-MIG-4 ancora aperto.
Segue P4-02, poi preparazione immutabile P4-03. Nessun worker avviato qui.
