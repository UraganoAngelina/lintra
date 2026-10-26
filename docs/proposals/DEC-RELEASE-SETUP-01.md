# DEC-RELEASE-SETUP-01 — Repository, credenziale e baseline release

Nota successiva: **A ratificata** dall’autore; vedere [ratifica](../verification/DEC-RELEASE-SETUP-01-ratification.md).
Testo storico della proposta conservato.

2026-10-26. **PROPOSTA, NON RATIFICATA al momento della redazione.**
Nota successiva15:02:28 UTC: T-P15-04 DONE, CI40343261368 Linux/MSVC success.
Il gate di questa proposta resta aperto; i fatti seguenti descrivono il preflight. Preflight dei prerequisiti umani
DEC-08/09/11 di T-P15-05; non è richiesta di permesso per normali commit/push/CI.
T-P15-04 resta in verifica indipendente; nessun workflow release implementato.

## Evidenze e decisioni già applicabili

- T-P15-02 DONE, CI40329791116 Linux/MSVC success; T-P15-03 DONE,
  CI40339655671 success. Packaging15-04 locale positivo, gate40343261368 pendente.
- Master Plan§11 dichiara DEC-08/09/11 prerequisiti espliciti15-05; ARCH§39C.5
  separa sorgente, binari e dati. Non esistono workflow release nel repository.
- API GitHub in sola lettura2026-10-26 14:51:38 UTC: sorgente
  UraganoAngelina/lintra pubblico; nomi variabili e secret a livello repository
  entrambi vuoti. Nessun valore segreto letto/stampato/modificato. Questo non
  è una verifica dei secret di eventuali environment futuri.
- Rust1.99.1 e baseline toolchain Windows10+/Server2016+ x86_64 sono già
  documentati in T-P1-02/DEVELOPMENT e non vengono rimessi in decisione.
- CNF-ARTIFACT-LINUX-01 A permette quattro SONAME GNU espliciti e conferma
  Linux interno/personale, distribuzione ad altri via Windows. La ratifica
  lascia espressamente aperta la scelta della baseline Linux di release.

## Proposta A — raccomandata

1. **DEC-08:** mantenere il sorgente attuale senza modificarne la visibilità;
   destinazioni `UraganoAngelina/lintra-binaries` e
   `UraganoAngelina/lintra-data`, entrambe **private**. Autorizzare creazione
   se assenti e bootstrap minimo README, poi solo dati/metadata/Release propri
   del rispettivo ciclo. Se esistono già, verificarne identità e visibilità
   prima dell’uso, senza modifiche distruttive. Nessuna autorizzazione implicita
   a rendere pubblici tracciati regionali, invitare collaboratori o cambiare
   repository omonimi incompatibili. Prime prove con soli dati sintetici già
   dichiarati: nessuna attestazione di ridistribuibilità CAD/SIOC.
2. **DEC-09:** per pubblicare dal sorgente nel repo binari utilizzare un PAT
   fine-grained dedicato, limitato a `lintra-binaries`, Contents:write e metadata
   read, scadenza90giorni. Secret del sorgente `LINTRA_RELEASE_TOKEN`; il token
   va configurato direttamente in GitHub, senza inviarne il valore in chat.
   Non copiare la credenziale personale locale dell’agente nei workflow.
   La futura CI del repository dati può usare il proprio GITHUB_TOKEN con
   contents:write per pubblicare nel medesimo repository, senza duplicare PAT.
   Approvare il contratto non rende il secret magicamente disponibile: finché
   non è configurato, niente release di prova superata o DONE15-05.
3. **DEC-11, solo residuo Linux:** build GNU `x86_64-unknown-linux-gnu` sul
   runner nativo `ubuntu-24.04` già usato dalla CI, con glibc fornita da tale
   baseline e registrazione della versione effettiva nell’audit release.
   Uso interno/personale, nessuna nuova promessa di portabilità verso distro
   più vecchie; nessun musl introdotto. Windows resta MSVC nativo/CRT statica,
   baseline documentata invariata. Allowlist, LinuxCI, riproducibilità a due
   build pulite, verifier/negative probe, lock e Quality Gate restano obbligatori.
4. Autorizzare la successiva release di prova **solo dalla CI**, versione di
   prerelease esplicita nuova, destinazioni private approvate, nessuna firma
   Authenticode come già deciso. Rifiutare versione/tag/release già presenti,
   senza cancellarli o sovrascriverli. Mai pubblicare come successo una release
   incompleta. La verifica effettiva, non la sola sintassi YAML, chiude15-05.

## Alternativa B — distribuzione binaria pubblica

Stesso contratto A, ma `lintra-binaries` pubblico per download senza accesso
al repository privato; `lintra-data` resta privato. Richiede approvazione
esplicita della visibilità pubblica e non autorizza dati regionali nello snapshot.
Nomi o meccanismo di autenticazione diversi possono essere indicati dall’autore.

## Fonti primarie consultate

- [Windows MSVC, rustc book](https://doc.rust-lang.org/stable/rustc/platform-support/windows-msvc.html):
  baseline di piattaforma Windows10+/Server2016+; distinta dai test realmente
  eseguiti sul runner Windows2025. La pagina specifica1.99.1 non era accessibile
  nel tool web: non viene dichiarata verificata; pin e fonte storica restano T-P1-02.
- [GitHub, token fine-grained](https://docs.github.com/en/authentication/keeping-your-account-and-data-secure/managing-your-personal-access-tokens):
  restrizioni per repository e permessi; preferibili ai PAT classic quando applicabili.
- [GitHub, creazione Release](https://docs.github.com/en/rest/releases/releases#create-a-release):
  permessi Contents write; ulteriori permessi workflow possono essere necessari
  se il commit bersaglio modifica workflow. Il repo binari proposto non contiene
  workflow: nessuna estensione dei permessi concessa implicitamente.

Decisione necessaria prima di sviluppare il workflow15-05. Non modifica ARCH/DET,
baseline normative, semantiche runtime o gate. Gli artefatti già conclusi non
vengono riaperti;14-05/M7 e15-01..03 conservano i propri esiti verificati.
