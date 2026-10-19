# IMPLEMENTATION_STATUS.md — Lintra

Consuntivo aggiornato al **2026-10-19**, derivato da `IMPLEMENTATION_STATUS_SEED.md`.
Piano v1.1; architettura e detailed design revisione 2026-10-16.

## Posizione corrente e prerequisiti

- P1 / **T-P1-01 — DONE**: scaffold compilabile del package `lintra`.
- Sessione corrente: **T-P0-04 DONE** per la classificazione documentale:
  218 note verificate. CAD B1 `2.12` risolta dall'autore: `000` valido,
  spazi mancanti, obbligo conservato. **T-P0-05 non avviata**: resta
  il chiarimento sulla valutazione iniziale CAD. Deroga ID SIOC per ricoveri
  iniziati nel 2025 e terminati nel 2026 risolta dall’autore (vedi decisioni).
  **T-P1-03 DONE_UNVERIFIED**: nessuna nuova evidenza CI in questa sessione.
- A-02/DEC-01: avvio P1 prima di G0 autorizzato nel kit e dalla richiesta
  dell'autore; DEC-03: nome `lintra` confermato.
- Repository locale `lintra/`, accanto allo starter kit; `origin` configurato
  dall'autore su `https://github.com/UraganoAngelina/lintra.git`.
  Lo starter kit non è parte del repository né una dipendenza di runtime.
- Toolchain corrente: **Rust 1.99.1**, scelta esplicitamente dall'autore,
  installata e fissata; target `x86_64-pc-windows-gnu` disponibile.
- UI: prototipo Slint dichiarato funzionante dall'autore, non verificato,
  non presente in questo scaffold e non considerato MVP.
- `CNF-01..11`, `CNF-LINTRA-SOURCE-01`, `CNF-PERSIST-01` risolti
  documentalmente; ADR-PERSIST-001 approvata; `PCR-CAD-001` superata.

## Autorizzazione corrente — 2026-10-19

L'autore ha revocato l'uso delle skill Superpowers. Esecuzione diretta,
una task alla volta, senza subagenti in questa sessione. Dopo la verifica
si procede senza conferme ordinarie solo se prerequisiti e risultati sono
chiari; le ambiguità richiedono chiarimento prima della task successiva.
Queste istruzioni sono conservate in `AGENTS.md`. Nessuna skill Superpowers,
orchestrazione o nuovo piano del relativo framework utilizzati il 2026-10-19.

## Autorizzazione storica — 2026-10-16

L'autore ha autorizzato esecuzione continua e parallelizzazione delle task,
rispettando le dipendenze funzionali della mappa e dei prompt atomici.
Questo mandato sostituisce lo stop ordinario dopo ogni task, senza derogare
ai gate normativi, alla CI reale o alle decisioni umane ancora aperte.
Ha inoltre autorizzato creazione autonoma del repository GitHub e fornito
identità per i commit locali; nessuna password viene salvata nei file.
DEC-11: scelta Rust 1.99.1 ricevuta. Baseline Linux di release ancora aperta.

Piano storico: [fondamenta](docs/superpowers/plans/2026-10-16-foundations.md).
Skill utilizzate nella sessione precedente: Superpowers pianificazione, esecuzione tramite agenti,
parallelismo e review/verifica; integrazione GitHub suggerita perché assente.

## Integrità e baseline consultata

`VERIFY_KIT.py`: **FAIL, exit 1**, per quattro file extra:
`EXPORT_CAD/A.txt`, `B1.txt`, `C.txt`, `D.txt`. Nessun hash/dimensione errato
o file censito mancante segnalato. Verifica indipendente: tutti i 196 file
del manifest conformi a dimensioni/SHA-256, 140 prompt presenti. Non dichiarare
PASS per il kit completo. Contenuto degli export non letto, copiato o versionato.

Consultati ARCH §3, §31, §39A–D, §45; DET §4–5, §94, §145, §166–174;
Piano §4, §9/P1, §10.3, §14–15; ADR, mappa dipendenze, indice e prompt T-P1-01.
Le sette copie documentali nel repository coincidono byte per byte con le fonti.

SIOC FROZEN: cinque PK nei JSON, `uniqueness_scope: trace`,
`E_DUPLICATE_RECORD_KEY`; dipendenze reference esterne dichiarate, non tutte
fornite come dataset. CAD RELEASE_CANDIDATE: cinque trace, 83 campi, profilo
storico e tre dataset consultati. Nessuna modifica/importazione degli schemi.
Le analisi aggregate preesistenti non sono test Rust o golden approvati.

## Gate e milestone

| Gate | Stato | Evidenza / limite |
|---|---|---|
| G0 — normativa | IN_PROGRESS | SIOC FROZEN documentale, CAD release candidate; nessuna nuova chiusura |
| G1 — repository CI | IN_PROGRESS | Toolchain pronta e workflow revisionato; autenticazione e run reali mancanti |
| QG-1 — controlli locali | PASS per la frontiera implementata | fmt, build release, check, clippy, 2 integration test + 3 doctest, cross-check GNU |
| QG-2 — CI Linux/Windows | BLOCKED | Accesso Git remoto non autenticato; nessuna run reale disponibile |
| QG-3 — architettura | PASS per lo scaffold | Moduli privati, CLI solo nel binario, nessun codice normativo o dipendenza |
| QG-8 — documenti | PASS per lo scaffold | Copie identiche, nessun contratto modificato |
| G-PERSIST | NOT_STARTED | ADR approvata; solo modulo vuoto |
| G-SIOC-HIST | OPEN_POLICY | DEC-SIOC-HIST-01, non blocca scaffold |
| G-CAD-RUNTIME | NOT_STARTED | Richiede compiler, cross-record e persistenza |

| Milestone | Stato | Evidenza |
|---|---|---|
| M0 | NOT_REACHED | G0 non chiuso |
| M1 | IN_PROGRESS | T-P1-01/02 completate; T-P1-03 implementata, CI reale e P2 mancanti |
| M2–M9 | NOT_REACHED | Nessuna implementazione anticipata |

`DONE` riguarda i criteri della singola task. Non dichiara completati P1,
M1, G0 o il gate multipiattaforma; i limiti sono annotati separatamente.

## Task

| Task | Stato | Ultima sessione | Evidenza |
|---|---|---|---|
| T-P0-01 | DONE | 2026-10-16 | Commit 0b13581; inventario 11 categorie/27 righe; integrità e completezza verificate, review approvata |
| T-P0-02 | DONE | 2026-10-16 | Commit 419600d/6a4e9ac; 218 campi/0 gap, review indipendente e campione approvato dall'autore |
| T-P0-03 | DONE | 2026-10-19 | 218/218 campi e policy/AST verificati; fonti integre; conflitto CAD 2.12 esplicitato, nessuna correzione normativa applicata |
| T-P0-04 | DONE | 2026-10-19 | 218 note classificate, hash/riferimenti/DEC-05 verificati; 000 e ID legacy SIOC risolti, una decisione CAD aperta |
| T-P0-05 | BLOCKED | 2026-10-19 | Non avviata: attende chiarimento CAD-VALUTAZIONE-INIZIALE secondo il mandato sequenziale |
| T-P0-06 | DONE | 2026-10-16 | Commit 69dc995/a55a634; 15 schede/150 metadati, gap espliciti, review approvata |
| T-P1-01 | DONE | 2026-10-16 | Build e controlli Linux; 3 doctest; copie e gitignore verificati |
| T-P1-02 | DONE | 2026-10-16 | Commit 0186c0b; pin Rust 1.99.1, attributi/config; STD-VERIFY e cross-check GNU superati; review approvata |
| T-P1-03 | DONE_UNVERIFIED | 2026-10-16 | Commit 4f71663; workflow e 2 test, controlli locali/review approvati; CI reale non eseguita |
| T-P2-01 | BLOCKED | 2026-10-16 | Prerequisito T-P1-03/QG-2 reale non soddisfatto; nessun codice dominio scritto |
| T-P2-02..03 | NOT_STARTED | — | Attendono T-P2-01 |
| T-PERSIST-01..12 | NOT_STARTED | — | Nessun contratto o backend |
| T-CAD-01..12 | NOT_STARTED | — | Nessun runtime CAD |

## Registro verifiche corrente — 2026-10-19, Linux locale

Evidenze e proposta: [T-P0-03](docs/verification/T-P0-03.md).

| Comando | Esito |
|---|---|
| `python3 docs/verification/check_presence.py` | PASS, exit 0: 135 SIOC + 83 CAD; dieci trace, policy/AST fedeli, quattordici fonti integre |
| `python3 docs/verification/check_notes.py` | PASS, exit 0: 218 note, indici/hash/pagine, constraint e ID regola, conteggi classi e citazioni DEC-05 |
| `cargo fmt --all -- --check` | PASS |
| `cargo check --all-targets --locked` | PASS |
| `cargo clippy --all-targets --locked -- -D warnings` | PASS |
| `cargo test --locked` | PASS: due integration test e tre doctest |

Nessun Rust modificato e nessun test normativo inventato. Nessuna nuova
run Windows/GitHub eseguita o presunta. Controlli documentali diretti;
nessun nuovo agente o review tramite framework.

## Registro storico — 2026-10-16, Linux locale

[Trascrizione finale](docs/verification/FOUNDATIONS.md),
[T-P1-02](docs/verification/T-P1-02.md),
[T-P0-01](docs/verification/T-P0-01.md),
[T-P0-02](docs/verification/T-P0-02.md),
[T-P0-06](docs/verification/T-P0-06.md).

| Comando / controllo | Esito reale |
|---|---|
| `rustc --version` | 1.99.1 (58cf8f9c3 2026-10-13), pin attivo |
| `cargo fmt --all -- --check` | PASS, exit 0 |
| `cargo check --all-targets --locked` | PASS, exit 0 |
| `cargo clippy --all-targets --target x86_64-unknown-linux-gnu --locked -- -D warnings` | PASS, exit 0 |
| `cargo test --target x86_64-unknown-linux-gnu --locked` | PASS, exit 0: 2 integration test + 3 doctest |
| `cargo build --release --target x86_64-unknown-linux-gnu --locked` | PASS, exit 0; sola verifica locale |
| `cargo check --all-targets --target x86_64-pc-windows-gnu --locked` | PASS, exit 0; nessun linking/esecuzione Windows |
| `cargo run --locked` | PASS, exit 0, nessun output applicativo |
| Manifest Cargo e moduli | Nessuna dipendenza; undici mod.rs senza codice |
| Copie ARCH/DET/PLAN/ADR/matrice | PASS, ancora identiche byte per byte alle fonti |
| Roundtrip Git attributi | PASS, 66 confronti byte in tre modalità autocrlf |
| Review indipendenti T-P1-02/T-P0-01/T-P1-03 | Approvate, nessun finding implementativo |
| T-P0-02 | 218/218 proiezioni esatte, 0 gap/overlap; CAD 83/83 righe e note riscontrate nel PDF; 16 fonti conformi agli hash |
| Accettazione umana T-P0-02 | Ricevuta: «Campione verificato: approvo»; nessuna approvazione runtime/G0 dedotta |
| T-P0-06 | 15 schede/150 metadati, 12 dataset SIOC mancanti e 3 CAD forniti riconciliati; review approvata |
| `git ls-remote --heads origin` | Prima fallisce per DNS sandbox; rieseguito con escalation, exit 128 `unable to get password from user`: autenticazione non disponibile |
| Secondo `git ls-remote --heads origin` fuori sandbox | Exit 128: `could not read Username for 'https://github.com': No such device or address` |
| GitHub Actions Linux/Windows | Non eseguite dall'agente; nessun push dell'agente effettuato |

## Registro storico bootstrap T-P1-01 — 2026-10-16

Trascrizione finale: [VERIFICATION.md](docs/bootstrap/VERIFICATION.md).

| Comando / controllo | Esito reale |
|---|---|
| `python3 VERIFY_KIT.py` nel kit | FAIL, exit 1: solo i quattro extra elencati |
| Python: SHA-256/dimensioni manifest; confronto copie | PASS: 196 file censiti; 7 copie identiche |
| `cargo fmt --all` | PASS, exit 0; mod.rs contengono solo newline emesso da rustfmt |
| `cargo fmt --all -- --check` | PASS, exit 0 |
| `cargo build` | PASS, exit 0; genera lockfile iniziale |
| `cargo build --locked` | PASS, exit 0 |
| `cargo check --all-targets --locked` | PASS, exit 0 |
| `cargo clippy --all-targets --locked -- -D warnings` | PASS, exit 0 |
| `cargo test --locked` | PASS: 3 doctest (1 positivo, 2 compile-fail), 0 unit test |
| `cargo run --locked` | PASS, exit 0; entrypoint senza output applicativo |
| `cargo metadata --no-deps --locked --format-version 1` | PASS: singolo package, lib e bin, zero dipendenze |
| `git check-ignore --no-index -q -- <path>` su 43 percorsi | PASS: 30 sensibili/artefatti esclusi, 13 sorgenti/fixture sintetiche ammessi |
| Ispezione file | PASS: 11 mod.rs senza codice; nessun .txt/export nel repository |
| `git diff --check`; `git status` | Eseguiti; nuovi file non committati, nessun preesistente modificato |
| Cross-check `x86_64-pc-windows-gnu` | Non eseguito: target non installato; installazione/config in T-P1-02 |
| CI Linux/Windows, release, test normativi | Non eseguiti: task successive, fuori scope |

I doctest verificano importabilità e divieto di importare `engine` e
`persistence` dall'esterno (DET §94). Nessun test di validazione fittizio.

## Problemi noti e decisioni aperte

- **DEC-05-CAD-VALUTAZIONE-INIZIALE:** precisare se l'almeno una iniziale
  riguarda lo storico dell'evento o il solo invio corrente.
- Dettagli e citazioni in [T-P0-04](docs/verification/T-P0-04.md). Le clausole
  SIOC 1.6/4.28 mantengono il non-enforcement v1 già ricevuto; G03, formato
  UVMD ricevuto e sentinel Barthel 99 non sono nuove decisioni aperte.
- Controllo completo kit FAIL per gli export extra, nessun impatto bloccante
  su T-P1-01: documenti censiti integri e dati esclusi dal nuovo repository.
- Remote GitHub configurato dall'autore, accesso autenticato non disponibile;
  integrazione GitHub risulta non installata. Workflow pronto, nessun push/run
  dell'agente. Rilevati commit intermedi esterni al flusso agenti 1c1e94f e
  69dc995, conservati; branch ora traccia origin/work/p1-foundations. Questo
  stato locale non attesta accesso autenticato o esecuzione CI del processo Git.
- `DEC-SIOC-HIST-01`: fino ad approvazione solo audit cross-run;
  non dedurre nuovi errori normativi dalla ripetizione di una PK.
- Backend storage, gestione chiavi/retention/protezione prima dell'uso reale:
  decisioni future, nessun database scelto per default.
- G0 CAD e versione runtime della DSL ancora da verificare.
- Metadati SIOC ricevuti: manifest/flow dichiarano 2026-04-20, trace split e
  annidati 2025-12-16; FREEZE_REPORT distingue baseline 2025 e profilo 2026.
  Entrambi conservati e documentati nelle tabelle; nessuna ratifica della
  versione runtime dedotta dalla sola approvazione della trascrizione.
- CAD posizione 60: G03 già risolta (blank fuori UCPDOM); i pending/undecided
  ancora in alcuni artefatti sono metadati storici superati, non nuova norma aperta.

## Decisioni della sessione corrente

- **DEC-05-SIOC-ID-2026 risolta dall’autore il 2026-10-19:** per ricoveri
  iniziati nel 2025 e terminati nel 2026, ID alfanumerico valorizzato,
  diverso da tutti zeri, larghezza fisica 20 byte e padding ricevuto.
  Nessun controllo di lunghezza logica otto, sole cifre, prefisso anno o
  progressivo in questo ramo. [Decisione registrata](docs/normative/DECISIONS.md).
  Fonti FROZEN inalterate; applicazione nella task di schema pertinente.

- **DEC-05-CAD-ICPC-000 risolta dall'autore il 2026-10-19:** «Confermo: 000
  valido, spazi mancanti». Campo obbligatorio, nessun lookup ICPC aggiunto.
  [Decisione registrata](docs/normative/DECISIONS.md); correzione della
  configurazione nella task pertinente, senza alterare il kit originale.

## Decisioni della sessione precedente — 2026-10-16

- Esecuzione continua/parallelismo autorizzati: implementer su file disgiunti,
  coordinatore unico per stato e Git; review indipendente per task e finale.
- Branch di lavoro `work/p1-foundations`; identità Git fornita dall'autore;
  nessuna password/token in file, comandi o URL remote.
- Rust 1.99.1 scelta dall'autore, installazione completata; nessuna nuova
  dipendenza Rust o mutazione delle fonti architetturali/normative.
- CI su Ubuntu 24.04/Windows 2025, target nativi espliciti, CRT statica MSVC,
  permessi di sola lettura, checkout fissato a SHA verificato. Nessuna release
  o scelta della baseline Linux distribuibile anticipata.
- T-P2-01 resta bloccata dalla CI reale. P0 documentale procede dove gli
  input consentono inventario/trascrizione, senza approvare norme o G0.

## Decisioni storiche bootstrap T-P1-01

| Decisione | Motivazione e fonte |
|---|---|
| Repository in `lintra/`, branch `main` | `.git` tecnica del workspace vuota e in sola lettura; DET §4 raccomanda `lintra/`; kit/dati separati senza alterare permessi |
| Package 0.1.0, edition 2024, `publish = false` | Metadati iniziali con toolchain disponibile; nessuna scelta MSRV/licenza/pubblicazione, pin in T-P1-02 |
| Moduli library privati, `cli` solo in main | DET §5/§94; nessuna API prematura |
| Undici mod.rs senza codice | T-P1-01, ADR-PERSIST-001: nessun tipo/driver/database anticipato |
| Lockfile iniziale incluso | Generato dalla build necessaria per i controlli `--locked`; non completa T-P1-02 |
| Doctest sul confine pubblico | Casi positivo/negativi pertinenti a DET §94, senza dominio |
| Nessun commit/staging/push | Nuovi file disponibili per revisione; nessuna identità Git inventata |
| Nessuna nuova dipendenza | N-26/DEC-15: lockfile contiene solo `lintra` |

## Documenti da aggiornare

Nessuno per le task implementate. ARCH, DET, PLAN e ADR copiati senza modifiche.
Nessun nuovo conflitto o cambiamento di contratto pubblico.

## Plan change log

- **2026-10-16, autore (dal seed):** autorizzato adeguamento ARCH/DET al piano,
  nome Lintra confermato, persistenza promossa a core; ADR-PERSIST-001 approvata.
- **Sessione successiva:** esecuzione continua e parallela autorizzata
  dall'autore; nessuna modifica al Master Implementation Plan. Il piano
  operativo locale organizza la frontiera realmente eseguibile.

## Handoff

HANDOFF — 2026-10-19 — chiarimento SIOC-ID-2026

- **Decisione:** registrata la deroga autorizzata per ingressi 2025 conclusi
  nel 2026; aggiornati registro decisioni, matrice note e report T-P0-04.
- **Verifica:** check_notes.py, check_presence.py e git diff --check superati;
  nessun Rust o schema ricevuto modificato, nessuna nuova esecuzione Cargo.
- **Next:** T-P0-05 resta non avviata, in attesa della sola decisione CAD
  sull’ambito della valutazione iniziale (storico evento o singolo invio).

HANDOFF — 2026-10-19 — esecuzione diretta / T-P0-04

- **Stato:** DONE per classificazione e registro delle ambiguità; avanzamento
  a T-P0-05 sospeso per due decisioni normative ancora aperte.
- **File:** matrici note SIOC/CAD, DECISIONS.md, report e checker T-P0-04;
  aggiornamento del report T-P0-03 e di questo stato.
- **Evidenze:** 218 note con hash/indici/riferimenti e citazioni verificate;
  26 metadata, 186 candidate, sei note ambigue (incluse clausole differite v1).
- **Decisioni:** ricevuta e registrata la risposta CAD 2.12; nessuna modifica
  delle fonti. Nessuna nuova esecuzione Cargo rivendicata per soli documenti.
- **Limiti:** definire vincoli ID SIOC 2025 e ambito iniziale CAD. Nessun
  codice Rust, compiler, schema normativo o dipendenza modificato.
- **Git:** branch work/p1-foundations; vecchio piano preesistente lasciato
  invariato; nessun push o nuova evidenza CI in questa sessione.
- **Next:** chiarire i due punti e poi procedere a T-P0-05, senza Superpowers.

HANDOFF — 2026-10-19 — esecuzione diretta / T-P0-03

- **Stato:** DONE per la consegna documentale; avanzamento sospeso per ambiguità.
- **File:** matrici presenza SIOC/CAD, report T-P0-03, checker documentale,
  AGENTS.md e questo stato. Le bozze note e il vecchio piano sono conservati.
- **Evidenze:** 218/218 campi, contesti e fonti verificati; fmt/check/clippy
  e test Rust superati, due integration test e tre doctest.
- **Decisioni:** niente Superpowers o nuovi subagenti; nessuna modifica
  a policy, contratti pubblici, dipendenze o documenti architetturali.
- **Problema:** DEC-05-CAD-ICPC-000, proposta e domanda inviate all'autore.
- **Git:** branch work/p1-foundations; nessun push in questa sessione;
  bozze T-P0-04 e piano precedente preesistenti non alterati da questa task.
- **Next:** registrare il chiarimento, poi verificare T-P0-04; nessuna
  task successiva avviata autonomamente in presenza dell'ambiguità.

HANDOFF — 2026-10-16 — Superpowers / T-P0-02 e T-P0-06

- **Stato:** entrambe DONE. T-P0-02: review indipendente approvata e
  approvazione umana testuale «Campione verificato: approvo» ricevuta.
- **File:** dieci tabelle JSON e README SIOC/CAD; reference-datasets.md;
  report riproducibili versionati in docs/verification/.
- **Evidenze:** 218 campi identici ai JSON, zero gap/overlap; CAD 83 righe e
  83 note corroborate dal PDF; 16 fonti conformi agli hash. Dataset: 15 schede,
  150 metadati con lacune, controlli e review superati.
- **Limiti:** fonte primaria XLSX SIOC assente e metadati ereditati dichiarati;
  nessuna norma cambiata. I JSON `document_status: DONE_UNVERIFIED` e i report
  iniziali sono snapshot antecedenti all'approvazione; lo stato corrente è
  registrato qui e nei README. G0 e gate runtime rimangono aperti.
- **Next:** T-P0-03 e T-P0-04 avviate in parallelo su file disgiunti.

HANDOFF — 2026-10-16 — Superpowers / T-P1-03

- **Stato:** DONE_UNVERIFIED, commit 4f71663; review indipendente approvata.
- **File:** .github/workflows/ci.yml e tests/platform_paths.rs; guida in docs/CI.md.
- **Evidenze:** controlli locali e GNU verdi, 2 test di I/O su percorsi con
  spazi e 3 doctest superati; nessuna dipendenza nuova o codice di dominio.
- **Limite bloccante:** GitHub non autenticato, nessun push o run reale.
  QG-2 resta aperto; T-P2-01 non avviata. Repo creato/configurato dall'autore,
  accesso tramite integrazione o credenziali Git del computer ancora necessario.
- **Next:** autenticare accesso, caricare il codice revisionato e verificare
  entrambi i job; solo poi avviare T-P2-01 e le task dipendenti.

HANDOFF — 2026-10-16 — Superpowers / T-P0-01

- **Stato:** DONE; commit 0b13581, review indipendente approvata.
- **File:** [INPUT_INVENTORY.md](docs/normative/INPUT_INVENTORY.md), matrice
  completa DET §154: undici categorie, 27 sottovoci con stato e motivo.
- **Evidenze:** controllo di completezza/tracciabilità superato; 196/196 file
  censiti integri; kit completo FAIL solo per quattro extra non letti.
- **Mancanti consegnati:** sorgente XLSX SIOC, dodici reference dataset SIOC,
  fixture/golden sintetiche approvate e dettagli di efficacia/versioning
  non desumibili dai materiali ricevuti. Elenco puntuale nell'inventario.
- **Limiti:** G0 aperto; nessuna norma, source document, API o Rust modificati.
- **Next:** T-P0-06 (schede con lacune ammesse) indipendente da CI;
  T-P0-02 richiede fonti e revisione umana a campione, non presunte.

HANDOFF — 2026-10-16 — Superpowers / T-P1-02

- **Stato:** DONE. File: rust-toolchain.toml, .gitattributes, .cargo/config.toml,
  docs/DEVELOPMENT.md. Lockfile già committato in c7ee1cd, invariato.
- **Evidenze:** Rust 1.99.1 installata con componenti e target GNU; fmt,
  check, clippy con warning negati, test (3 doctest), build e cross-check GNU
  tutti exit 0; 66 confronti byte Git su fixture/golden sintetici superati.
- **Review indipendente:** approvata, nessun finding implementativo.
- **Decisioni:** Rust 1.99.1 scelta umana; baseline Linux di release aperta;
  Windows 10+/Server 2016+ secondo architettura e documentazione toolchain.
- **Limiti:** MinGW/Wine non installati, nessuna build collegata/test Wine;
  nessuna CI reale dichiarata. Nessun contratto pubblico/dependency aggiunto.
- **Git:** branch work/p1-foundations, commit 0186c0b; documenti di sessione
  ancora in aggiornamento, nessun remote autenticato.
- **Next:** T-P1-03, prerequisito locale verificato; nessuna deroga a QG-2.

HANDOFF — 2026-10-16 16:30 CEST — Codex / bootstrap Lintra

- **Fase/Task attiva:** P1 / T-P1-01; **stato: DONE**.
- **Fatto:** Cargo.toml/lock, lib/main, undici mod.rs vuoti, directory dati/test
  riservate, README, gitignore, sette documenti approvati, piano, evidenze e stato.
- **Evidenze:** registro sopra e trascrizione collegata; controlli Linux
  effettivamente eseguiti; nessuna CI dichiarata.
- **Test aggiunti/eseguiti:** tre doctest tutti superati; nessun tipo/regola
  normativa ancora implementato; test Windows non eseguiti.
- **Contratti pubblici aggiunti/modificati:** nessuno.
- **Decisioni:** tabella sopra; nessun database o policy normativa scelti.
- **Documenti da aggiornare:** nessuno per lo scope implementato.
- **Problemi/blocchi:** nessun blocco T-P1-01; kit con quattro extra esclusi;
  CI e gate successivi aperti.
- **Elementi rinviati:** toolchain/config Windows/attributi in T-P1-02;
  CI in T-P1-03; dominio, compiler, persistenza, validatori, report e Slint
  nelle rispettive task.
- **Stato Git:** branch `main`; nessun commit; working tree non pulito
  (nuovi file untracked); target ignorato; nessun remote.
- **Prossima task:** T-P1-02, prerequisito scaffold verificato;
  attende autorizzazione dell'autore. Non avviata.
