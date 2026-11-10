# Verifiche Linux e Windows

## Assetto corrente — 10 novembre 2026

Il workflow `.github/workflows/ci.yml` verifica nativamente Linux GNU e Windows
MSVC, con budget45min per runner: fmt, Clippy, test workspace default e
allfeatures, core senza bundle, boundary, tooling, smoke benchmark, release
riproducibile, artefatto e comandi README nelle shell native. Carica evidenze
sintetiche; la pubblicazione di release è nel workflow separato.
Tre job confrontano JSONL e pixel, due riaprono il database prodotto dall'altro OS.
I sette job richiesti devono riuscire sullo stesso SHA. Packaging UI è opzionale
e il suo skip esplicito non sostituisce alcun job richiesto.

Gate CAD12: [run42641969728](https://github.com/UraganoAngelina/lintra/actions/runs/42641969728)
su d45078e, sette success. [Dossier](verification/T-CAD-12-runtime-native-ci.json)
con log verificati e import SQLite reali. Il precedente timeout35min è
conservato come run non qualificata, senza ridurre controlli o cambiare runtime.
Commit/push/CI sul branch concordato sono autorizzati da AGENTS.md; autenticazione
e stato delle nuove run si verificano realmente, senza dedurli da gate passati.

## Cronologia di bootstrap — T-P1-03

Le sezioni seguenti descrivono il workflow iniziale e le sue prime evidenze.
I limiti allora riportati non rappresentano le capacità correnti descritte sopra.

T-P1-03 definisce `.github/workflows/ci.yml`: pull request, push su `main`
e avvio manuale. Il workflow ha permesso di sola lettura dei contenuti,
cancella le esecuzioni superate sulla stessa branch/PR e non pubblica artefatti.

La matrice compila nativamente su `ubuntu-24.04`
(`x86_64-unknown-linux-gnu`) e `windows-2025`
(`x86_64-pc-windows-msvc`). La versione Rust è letta da
`rust-toolchain.toml`, senza una seconda copia del pin nel workflow.
Le opzioni CRT statica sono limitate al target Windows MSVC;
la configurazione MinGW/Wine locale riguarda solo il target GNU.

Ogni runner esegue formattazione, clippy con warning negati, test e build
release con `--locked`. Il test `tests/platform_paths.rs` verifica soltanto
I/O sintetico su directory e nomi file con spazi, inclusi preservazione di
LF/CRLF/BOM/spazi finali ed errore `NotFound` per file assente.
Non implementa ancora lettura normativa, resolver del bundle o validazione.

## Attivazione e prova del gate

Per verificare QG-2 occorre caricare il repository sorgente su GitHub,
abilitare Actions ed eseguire questo workflow. I risultati di entrambi
i job devono essere verdi e registrati con URL/log in
`IMPLEMENTATION_STATUS.md`. Un controllo locale o GNU da Linux non
sostituisce il runner Windows reale.

L'autore ha creato il repository `lintra` e configurato
`origin` su `https://github.com/UraganoAngelina/lintra.git`.
L'accesso autenticato all'account è un prerequisito esterno: username o
password scritti in chat non configurano un'integrazione GitHub.
Non salvare password/token nei file o negli URL remote.

Il test/compilazione di sviluppo non è una release distribuibile.
Packaging, verifica delle dipendenze dell'eseguibile, baseline Linux e
pubblicazione sono nelle task P15.

## Fonti esterne verificate per il workflow

- [Sintassi GitHub Actions](https://docs.github.com/en/actions/reference/workflows-and-actions/workflow-syntax).
- [Runner GitHub ospitati](https://docs.github.com/en/actions/reference/runners/github-hosted-runners).
- [Immagine Windows 2025](https://github.com/actions/runner-images/blob/main/images/windows/Windows2025-Readme.md).
- [Release checkout v7.0.1](https://github.com/actions/checkout/releases/tag/v7.0.1),
  fissata al commit completo verificato nel workflow.
- [Configurazione Cargo per target](https://doc.rust-lang.org/cargo/reference/config.html#target).

Lo stato dell'esecuzione reale, compresa l'eventuale assenza di autenticazione,
è nel consuntivo. Il Git eseguito dall'agente non dispone ancora di credenziali
utilizzabili; i commit/tracking remoti rilevati nell'app non attestano una run
CI o la condivisione dell'autenticazione con il processo Git dell'agente.
Nessun esito CI viene dedotto dalla sola configurazione.

## Evidenza aggiornata — 2026-10-19

[Run 39232752518](https://github.com/UraganoAngelina/lintra/actions/runs/39232752518) sul commit
34e67da, avviata manualmente sul branch work/p1-foundations: Linux GNU e
Windows MSVC entrambi completed/success. gh usa il keyring dell’account;
l’accesso funziona fuori dal sandbox. Le precedenti indicazioni di mancata
autenticazione descrivono la sessione storica. Nuove modifiche Rust richiedono
una nuova run: questo esito non viene esteso a commit futuri.
