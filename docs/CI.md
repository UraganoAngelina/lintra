# Verifiche Linux e Windows

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
