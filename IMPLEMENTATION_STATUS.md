# IMPLEMENTATION_STATUS.md — Lintra

Consuntivo del 2026-10-16, derivato da `IMPLEMENTATION_STATUS_SEED.md`.
Piano v1.1; architettura e detailed design revisione 2026-10-16.

## Posizione corrente e prerequisiti

- P1 / **T-P1-01 — DONE**: scaffold compilabile del package `lintra`.
- Prossima task **T-P1-02 — NOT_STARTED**, solo dopo nuova autorizzazione.
- A-02/DEC-01: avvio P1 prima di G0 autorizzato nel kit e dalla richiesta
  dell'autore; DEC-03: nome `lintra` confermato.
- Repository locale `lintra/`, accanto allo starter kit; nessun remote.
  Lo starter kit non è parte del repository né una dipendenza di runtime.
- Rust/Cargo 1.99.0, host `x86_64-unknown-linux-gnu`; nessun pin toolchain ora.
- UI: prototipo Slint dichiarato funzionante dall'autore, non verificato,
  non presente in questo scaffold e non considerato MVP.
- `CNF-01..11`, `CNF-LINTRA-SOURCE-01`, `CNF-PERSIST-01` risolti
  documentalmente; ADR-PERSIST-001 approvata; `PCR-CAD-001` superata.

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
| G1 — repository CI | IN_PROGRESS | Repository locale presente; T-P1-02/03 ancora da eseguire |
| QG-1 — controlli locali | PASS per lo scaffold Linux | fmt, build, check, clippy, test |
| QG-2 — CI Linux/Windows | NOT_STARTED | T-P1-03; nessun merge gate dichiarato superato |
| QG-3 — architettura | PASS per lo scaffold | Moduli privati, CLI solo nel binario, nessun codice normativo o dipendenza |
| QG-8 — documenti | PASS per lo scaffold | Copie identiche, nessun contratto modificato |
| G-PERSIST | NOT_STARTED | ADR approvata; solo modulo vuoto |
| G-SIOC-HIST | OPEN_POLICY | DEC-SIOC-HIST-01, non blocca scaffold |
| G-CAD-RUNTIME | NOT_STARTED | Richiede compiler, cross-record e persistenza |

| Milestone | Stato | Evidenza |
|---|---|---|
| M0 | NOT_REACHED | G0 non chiuso |
| M1 | IN_PROGRESS | Solo T-P1-01; P1/P2 e CI da completare |
| M2–M9 | NOT_REACHED | Nessuna implementazione anticipata |

`DONE` riguarda il criterio della task (scaffold compilabile, nessun codice
di dominio). Non dichiara completati P1, M1 o il gate multipiattaforma.

## Task

| Task | Stato | Ultima sessione | Evidenza |
|---|---|---|---|
| T-P1-01 | DONE | 2026-10-16 | Build e controlli Linux; 3 doctest; copie e gitignore verificati |
| T-P1-02 | NOT_STARTED | — | Pin toolchain, attributi Git e configurazione Windows rinviati |
| T-P1-03 | NOT_STARTED | — | Nessun workflow CI o repository remoto creato |
| T-P2-01..03 | NOT_STARTED | — | Nessun tipo dominio |
| T-PERSIST-01..12 | NOT_STARTED | — | Nessun contratto o backend |
| T-CAD-01..12 | NOT_STARTED | — | Nessun runtime CAD |

## Registro verifiche — 2026-10-16, Linux locale

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

- Controllo completo kit FAIL per gli export extra, nessun impatto bloccante
  su T-P1-01: documenti censiti integri e dati esclusi dal nuovo repository.
- Remote GitHub assente; CI da predisporre/eseguire in T-P1-03.
- `DEC-SIOC-HIST-01`: fino ad approvazione solo audit cross-run;
  non dedurre nuovi errori normativi dalla ripetizione di una PK.
- Backend storage, gestione chiavi/retention/protezione prima dell'uso reale:
  decisioni future, nessun database scelto per default.
- G0 CAD e versione runtime della DSL ancora da verificare.

## Decisioni prese in implementazione

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

Nessuno per T-P1-01. ARCH, DET, PLAN e ADR copiati senza modifiche.
Nessun nuovo conflitto o cambiamento di contratto pubblico.

## Plan change log

- **2026-10-16, autore (dal seed):** autorizzato adeguamento ARCH/DET al piano,
  nome Lintra confermato, persistenza promossa a core; ADR-PERSIST-001 approvata.
- **Questa sessione:** nessuna modifica al Master Implementation Plan;
  il piano operativo locale descrive solo l'esecuzione T-P1-01.

## Handoff

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
