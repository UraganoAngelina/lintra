# T-P15-06 — release dati indipendente

Preflight: T-P15-04 DONE su f7426a5ef737fb392e79eb3a80fda8ebf53a7b11,
CI40343261368 Linux/MSVC positiva. DEC-08 ratificata da
DEC-RELEASE-SETUP-01 A, destinazione privata lintra-data già creata e immutabilità
nativa abilitata. Non dipende dalla CI pendente15-05; sviluppo sequenziale durante
la sua attesa, senza delega o background locale.

Implementazione in repository dedicato UraganoAngelina/lintra-data: workflow
.github/workflows/release.yml, scripts/release_data.py, sei test stdlib, bundle
sintetico, README e RELEASE_ENGINEERING.md con spiegazione dettagliata delle
funzioni/condizioni e dei limiti. Il workflow esegue il bundler Rust reale fissato
al commit qualificato15-04, non ricompila/ripubblica l’eseguibile. Il publish job
ha Contents write solo sul proprio repository; nessun nuovo PAT/cross-repo write.

Versione bundle esplicita synthetic-data-release-v1, separata da quella binario.
Manifest e payload sottoposti a verifica hash/inventario/ZIP prima degli upload.
Prenotazione tag atomica, draft, verifica digest/size/state server e inventario,
poi pubblicazione e controllo immutable=true. Errori non cancellano/riprendono
versioni già riservate; serve una nuova versione. Nessuna certificazione normativa
implicita, dataset o tracciato regionale ridistribuito.

Evidenze locali iniziali: sei test positivi, actionlint1.7.12 positivo (senza
plugin esterni shellcheck/pyflakes non disponibili), diff check positivo.
Rust fmt/clippy/test non applicabili al repo Python/YAML/dati; bundler già coperto
nei gate15-04, prova locale del bundler e release reale ancora da registrare.
Stato IN_PROGRESS: nessun DONE o M5 fino a pubblicazione reale verificata.

## Gate avviato

Commit dati3a6cf48727c88307c7033eb4ff47d99f0ebaaa18 pubblicato su main.
Release dispatch40350440129, lettura15:34:45UTC queued. Nessun esito presunto.
Prova locale bundler reale: ZIP4074byte, SHA256
a8ed7da00d6de35ba430e2dae8f6ab85a73a9a42229d186a58c59dd82e4a799f;
inspect verifica inventario/manifest/checksum.

## Chiusura verificata — recovery 2026-10-27

Stato **DONE**. Evidenza strutturata: [T-P15-06-published-release.json](T-P15-06-published-release.json).
Pubblicazione CI completata con successo, release privata non draft e immutable=true.
Scaricati tutti gli asset pubblicati; inventario esatto, digest SHA256, dimensioni
e stato uploaded confrontati con API GitHub. SHA256SUMS esterno verificato;
ZIP verificato con inspect contro manifest committato, inventario, checksum interni e hash di tutti i payload. Digest ZIP coincidente anche con la prova locale già registrata.
Test protocollo ripetuti: 6/6 positivi. Nessun codice modificato;
Rust non rieseguito, gate già disponibili sugli SHA implementativi invariati.
Nessun gate pendente della task. M5 resta aperta fino a T-P15-07.
Handoff completo in IMPLEMENTATION_STATUS.md; candidata15-07 non avviata,
secondo arresto alla conclusione delle attività aperte richiesto nella recovery.
