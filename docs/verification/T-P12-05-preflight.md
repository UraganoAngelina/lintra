# T-P12-05 — Preflight della rilettura XLSX indipendente

2026-10-23. Prerequisito formale 12-03 verificato in CI 39967822377;
12-04 pubblicata dc4dd47, gate in attesa. Nessun codice 12-05 ancora avviato.
Prompt atomico richiede parser di lettura, celle String esatte 00123/05,
ordine uguale al JSONL, rollover parametrizzato e valori del Riepilogo.

## Candidato DEC-06/DEC-15 prima dell'aggiunta

calamine =0.36.1 solo dev-dependency, default-features=false. Non viene usato
per input sanitari, nel validation loop o nel writer di produzione. Manifest
ufficiale del tag: MIT, Rust 1.88, default features vuote; chrono/picture non
necessari. Fonte primaria letta:
https://raw.githubusercontent.com/tafia/calamine/v0.36.1/Cargo.toml
Changelog/release letti: https://github.com/tafia/calamine/releases
La release 0.36.1 corregge anche la risoluzione dei link Strict OOXML della
0.36.0; non scegliere una versione precedente per deduzione da esempi vecchi.

Richiede zip 8.6 senza default features con deflate e quick-xml 0.41 con
encoding. Il subtree e le licenze/MSRV effettivi devono essere verificati dopo
la risoluzione; non assumere backend pure-Rust dalla sola descrizione. Tenere
invariate tutte le versioni/checksum preesistenti, ammettendo nuovi pacchetti
soltanto per il lettore di test. Nessuna feature backend di sistema autorizzata.

## Verifiche pianificate

Generare veri JSONL/summary con schema sintetico e service reale, poi esportare
con XlsxReportWriter e leggere tramite calamine. Assert espliciti Data::String,
non confronti dopo conversione generica che nascondano celle Float/Date.
Header e proiezione attesa indipendenti da ERROR_COLUMNS::value, confrontate
con i campi del JSONL originale. Ordine completo attraversando Errori (2), ecc.
Zero errori/header unico, capacità esatta senza foglio vuoto extra, limite minimo.

Riepilogo confrontato con il summary canonico effettivo, compresa completezza
Partial/audit/dataset quando applicabili. Nessuna normalizzazione che mascheri
perdite di contenuto; gli audit runtime devono coincidere con il summary della
run, non con valori inventati uguali fra macchine. Fixture/header/celle normative
attese uguali sui due runner, senza pretendere identità dei byte ZIP.
Calamine può materializzare celle nei test piccoli; la misura low-memory del
writer di produzione rimane il probe distinto di 12-03.

Esito positivo nativo, insieme a 12-04/P9/P10/P11, consentirà valutare la chiusura
M4 nello scope locale ratificato. Non chiude automaticamente release M5, Full,
metadata/tempo, persistenza o conformità normativa completa SIOC/CAD.
