# T-P16-01 — workspace core/CLI e desktop

## Prerequisiti e scope

T-P16-00 DONE: CI40485215039/bdef3c3, Linux/MSVC/pixel identici, riferimento
chiaro approvato esplicitamente dall’autore il2026-10-27. Font e differenze note
accettati. ARCH31/39D.2, DET Addendum B.1, piano P16 e QG-2/QG-5 applicati.

## Modifiche e ragioni di correttezza

Cargo.toml conserva il package lintra in root e aggiunge workspace members
`.` e `ui`, resolver3 coerente con edition2024. default-members root conserva
cargo build/run/xtask release per la CLI. CI usa esplicitamente --workspace per
Clippy e test, default e tutte le feature. Cargo.lock aggiunge solo lintra-ui,
nessun download/dipendenza di prodotto nuovo. xtask e spike esclusi: rimangono
strumenti isolati con i rispettivi lock, nessun trasferimento automatico di Slint.

ui/Cargo.toml dichiara un eseguibile separato lintra-ui, dipendente da lintra
tramite API pubblica e default-features=false. ui/src/main.rs restituisce codice2
con messaggio esplicito: scaffold privo di finestra fino a16-02, nessuna falsa
capacità. README documenta comandi/confini. Nessuna modifica a validazione,
schema, input, report o regole normative. Il package storico core/CLI non si
sposta e le API pubbliche restano compatibili.

scripts/check_workspace.py consulta cargo metadata --locked --all-features
sul target nativo di rustc, verifica esattamente i due membri e il collegamento
UI→core. Attraversa tutte le dipendenze transitive del core, inclusi build/dev
nel grafo risolto, e rifiuta Slint/i-slint/slint-* oppure UI. Il filtro target
qualifica il grafo effettivo per ciascun runner; evita di scaricare package Apple
non necessari sul runner Linux. Metadati mancanti o Cargo fallito producono errore,
mai successo per assenza non verificata. Il controllo è eseguito da entrambi
Linux/MSVC in CI. Tre test Python coprono GUI solo nell’adapter (positivo),
GUI transitiva nel core (negativo), grafo irrisolto o collegamento assente.

## Gate

19test Python pass; cargo check --workspace offline pass; grafo effettivo pass.
Fmt e Clippy --workspace --all-targets default/all-features positivi.
Regressioni Rust workspace698/706pass,82suite,0fail/2ignored e actionlint1.7.12
positivi. Binario scaffold eseguito: exit2, stdout vuoto, messaggio esplicito.
CI nativa da avviare dopo conclusione
della CI PERSIST-01, evitando la cancellazione del gate precedente.
QG-5 è il probe di release/verify-artifact della CI: non è ancora passato su
questa revisione. Nessuna allowlist grafica implicita; UI packaging è16-08.

## Handoff

IN_PROGRESS, nessun DONE anticipato. File: manifest/lock, ui scaffold,
guard/test Python, workflow, dossier/status/indice. Next16-02 solo con CI16-01
positiva: font originali, Slint pinned/software, palette chiara identica,
selettore e scuro nero/rosso/arancione richiesti. Nessun PHI, nuova scelta di
sicurezza o modifica FROZEN. AUTO_CONTINUE.
