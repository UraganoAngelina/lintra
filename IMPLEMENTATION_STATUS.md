# IMPLEMENTATION_STATUS.md — Lintra

Consuntivo aggiornato al **2026-10-19**, derivato da `IMPLEMENTATION_STATUS_SEED.md`.
Piano v1.1; architettura e detailed design revisione 2026-10-16.

## Posizione corrente e prerequisiti

- P1 / **T-P1-01 — DONE**: scaffold compilabile del package `lintra`.
- Sessione corrente: **T-P0-07 DONE**, contratto del commit 72f9f5e
  approvato dall’autore con «Approvo il contratto T-P0-07». Verifiche
  strutturali e limiti restano registrati nel report.
  **T-P0-08 DONE nello scope minimale autorizzato**: profilo versionato con
  flow e cinque trace, 135 campi, cinque PK, sei join; ledger dei rinvii e
  piano fixture verificati. DEC-SIOC-MINIMAL-01: nessun dataset, semantica
  minima con 050 nell’ambito confermato; encoding runtime esplicito, mar01
  per il futuro controllo età. Lint Rust obbligatorio appena T-P3-04 esiste.
  Nessun compiler o validatore Rust normativo implementato.
  **T-P0-09 DONE / G0 APPROVED / M0 REACHED**: l’autore ha approvato
  la checklist del commit 407f89e con i rinvii indicati.
  **T-P1-03 DONE / QG-2 PASS**: [CI Linux/Windows verde](https://github.com/UraganoAngelina/lintra/actions/runs/39232752518)
  sul commit 34e67da; autenticazione GitHub funzionante fuori sandbox.
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
| G0 — normativa | APPROVED_WITH_DEFERRALS | Autore: «Approvo G0 con i rinvii indicati», checklist commit 407f89e, 2026-10-19 |
| G1 — repository CI | PASS | [Run 39232752518](https://github.com/UraganoAngelina/lintra/actions/runs/39232752518), Linux GNU e Windows MSVC success, commit 34e67da |
| QG-1 — controlli locali | PASS per la frontiera implementata | fmt, build release, check, clippy, 2 integration test + 3 doctest, cross-check GNU |
| QG-2 — CI Linux/Windows | PASS per 6ac0fb8 | [Run reale](https://github.com/UraganoAngelina/lintra/actions/runs/39233324913), entrambi i runner verdi; nuove modifiche Rust richiedono nuova CI |
| QG-3 — architettura | PASS per lo scaffold | Moduli privati, CLI solo nel binario, nessun codice normativo o dipendenza |
| QG-8 — documenti | PASS per lo scaffold | Copie identiche, nessun contratto modificato |
| G-PERSIST | NOT_STARTED | ADR approvata; solo modulo vuoto |
| G-SIOC-HIST | OPEN_POLICY | DEC-SIOC-HIST-01, non blocca scaffold |
| G-CAD-RUNTIME | NOT_STARTED | Richiede compiler, cross-record e persistenza |

## G0 — checklist T-P0-09 approvata

**Approvazione ricevuta il 2026-10-19:** «Approvo G0 con i rinvii indicati».
La risposta si riferisce alla checklist concreta del commit `407f89e`.
T-P0-01..08 verificati su artefatti effettivi: [report T-P0-09](docs/verification/T-P0-09.md).
Contratto formato 1 approvato in T-P0-07; profilo iniziale SIOC minimale
approvato in DEC-SIOC-MINIMAL-01. L’approvazione chiude G0 per la formalizzazione corrente con i rinvii
espliciti sotto; non dichiara implementate le capability o ricevuti i dataset.

| Voce DET §106 | Esito proposto, evidenza e rinvio | Task/accettazioni bloccate dal rinvio |
|---|---|---|
| Forma JSON canonica | CHIUSA per formato 1: SCHEMA_AUTHORING e meta-schema approvati; flow SIOC minimale e cinque trace validi. CAD canonico operativo rinviato alle task CAD: candidato ricevuto non promosso | Esecuzione/certificazione profilo CAD completo, non tipi raw generici |
| FieldType | CHIUSO sottoinsieme numeric/alphanumeric/date con lessico esplicito. Alphabetic/Decimal RINVIATI: assenza di repertorio e specifica separatore/segno/scala | T-P6-04 ramo Alphabetic e T-P9-03 Decimal non accettabili come supportati senza ampliamento approvato; nessun no-op |
| AlignmentRule | CHIUSA: exact/right_pad/left_pad, fill_byte e content_policy per campo; preserve_width per codici significativi. Nessun trim globale | Nessun blocco normativo aggiuntivo per il sottoinsieme corrente |
| Sintassi DSL | CHIUSA: AST tipizzato, operatori/capability chiusi, lowering esaustivo. Deroga limitata substring/età compiuta/delta anni già approvata; implementazione futura | Capability non implementate bloccano esecuzione profili che le richiedono; non la definizione dei contratti |
| Mapping OBB/FAC/NBB | CHIUSO: basi/condizioni approvate e fonte conservata per 218 campi, check_presence positivo; NBB non diventa optional incondizionato | Nessun blocco aggiuntivo; nuove ambiguità normative richiedono decisione |
| Regole cross-trace | CHIUSE strutturali SIOC: parent, PK e revoca con policy esplicite. RINVIATE congruenze cliniche/anno/età/temporali in minimal. CAD storico formalizzato nel contratto, runtime successivo | Golden e accettazione SIOC semantico completo; task runtime CAD e relativo gate finché compiler/persistenza e prove non esistono |
| Identità e join | CHIUSE: cinque PK e sei join canonici SIOC, inventario CAD e scope storico documentati; NotApplicable esplicito. Enforcement cross-run SIOC resta fuori dall’audit | Enforcement storico SIOC oltre audit senza DEC-SIOC-HIST-01 risolta; runtime CAD prima delle task persistenti |
| Formato dataset | CHIUSO contratto id/version/hash/path/normalizzazione e metadata; 15 schede verificate. RINVIATA fornitura dei dodici dataset ufficiali SIOC, esclusi dal minimale | Accettazione T-P11-05/06 su dati reali SIOC e golden di lookup completo; provider generici verificabili con dati sintetici dichiarati |
| Versioning reale tabelle | CHIUSO formato revisioni/hash, no latest; RINVIATE release/efficacia/provenienza reali delle tabelle SIOC mancanti e lacune temporali CAD documentate | Preflight e audit di tali tabelle in T-P11-03/06 e accettazioni normative complete; nessuna versione inventata |

Ulteriori limiti trasversali: encoding obbligatorio runtime, controllo
minuscole rinviato; layout export reali da chiarire prima del loro uso;
lint Rust sugli schemi obbligatorio in T-P3-04; fixture/golden sintetiche
pianificate ma non ancora prodotte o approvate. Nessun blocco viene nascosto
come validazione superata. Tutti i dettagli sono nel report T-P0-08.

G0 è chiuso con i rinvii indicati e M0 REACHED. Il solo prerequisito G0
è soddisfatto; restano applicabili tutte le altre dipendenze e i blocchi
puntuali della tabella. La CI reale del commit 34e67da ha ora soddisfatto il prerequisito di T-P2-01.
ARCH/DET/Piano sorgenti non sono modificati; questa checklist registra
contratti approvati e rinvii secondo la procedura di T-P0-09.

## Milestone

| Milestone | Stato | Evidenza |
|---|---|---|
| M0 | REACHED | G0 approvato dall’autore con rinvii espliciti, 2026-10-19 |
| M1 | IN_PROGRESS | T-P1-01/02 completate; T-P1-03 implementata, CI reale e P2 mancanti |
| M2–M9 | NOT_REACHED | Nessuna implementazione anticipata |

`DONE` riguarda i criteri della singola task. Non dichiara completati P1,
M1 o il gate multipiattaforma; G0 è approvato con i rinvii sopra.

## Task

| Task | Stato | Ultima sessione | Evidenza |
|---|---|---|---|
| T-P0-01 | DONE | 2026-10-16 | Commit 0b13581; inventario 11 categorie/27 righe; integrità e completezza verificate, review approvata |
| T-P0-02 | DONE | 2026-10-16 | Commit 419600d/6a4e9ac; 218 campi/0 gap, review indipendente e campione approvato dall'autore |
| T-P0-03 | DONE | 2026-10-19 | 218/218 campi e policy/AST verificati; fonti integre; conflitto CAD 2.12 esplicitato, nessuna correzione normativa applicata |
| T-P0-04 | DONE | 2026-10-19 | 218 note classificate, hash/riferimenti/DEC-05 verificati; 000, ID legacy SIOC e ambito iniziale CAD risolti |
| T-P0-05 | DONE | 2026-10-19 | Dossier SIOC/CAD: 74 regole, 10 join + ricerca esistenza; PK, identity e policy/DEC verificate; parent opzionale SIOC risolto dall’autore |
| T-P0-06 | DONE | 2026-10-16 | Commit 69dc995/a55a634; 15 schede/150 metadati, gap espliciti, review approvata |
| T-P0-07 | DONE | 2026-10-19 | Commit 72f9f5e: contratto completo/artefatti approvati dall’autore; 33 casi negativi e checker passati, limiti espliciti |
| T-P0-08 | DONE | 2026-10-19 | Profilo structural-minimal.1 autorizzato; flow+5 trace, 135 campi/5 PK/6 join, ledger, lint authoring e piano verificati; ricontrollo T-P3-04 obbligatorio |
| T-P0-09 | DONE | 2026-10-19 | Checklist commit 407f89e approvata dall’autore: «Approvo G0 con i rinvii indicati»; G0 chiuso, M0 raggiunta |
| T-P1-01 | DONE | 2026-10-16 | Build e controlli Linux; 3 doctest; copie e gitignore verificati |
| T-P1-02 | DONE | 2026-10-16 | Commit 0186c0b; pin Rust 1.99.1, attributi/config; STD-VERIFY e cross-check GNU superati; review approvata |
| T-P1-03 | DONE | 2026-10-19 | CI su commit 34e67da: Linux GNU e Windows MSVC verdi, [run 39232752518](https://github.com/UraganoAngelina/lintra/actions/runs/39232752518) |
| T-P2-01 | DONE | 2026-10-19 | 7 newtype + ByteRange; 5 unit/2 integration/5 doctest; CI Linux/Windows verde su 6ac0fb8, run 39233324913 |
| T-P2-02 | DONE_UNVERIFIED | 2026-10-19 | Mapping unico 15 codici core + RegulatoryRule, registro e 4 nuovi test; fmt/check/clippy/test/release verdi, nuova CI richiesta |
| T-P2-03 | NOT_STARTED | — | Prerequisito T-P2-01 soddisfatto |
| T-PERSIST-01..12 | NOT_STARTED | — | Nessun contratto o backend |
| T-CAD-01..12 | NOT_STARTED | — | Nessun runtime CAD |

## Registro verifiche corrente — 2026-10-19, Linux locale

Evidenze e proposta: [T-P0-03](docs/verification/T-P0-03.md).

| Comando | Esito |
|---|---|
| `python3 docs/verification/check_presence.py` | PASS, exit 0: 135 SIOC + 83 CAD; dieci trace, policy/AST fedeli, quattordici fonti integre |
| `python3 docs/verification/check_notes.py` | PASS, exit 0: 218 note, indici/hash/pagine, constraint e ID regola, conteggi classi e citazioni DEC-05 |
| `python3 docs/verification/check_rules_and_joins.py` | PASS, exit 0: dieci trace, 74 regole, dieci join con cardinalità/policy o DEC, trenta presence, tre gruppi, profilo CAD completo e riferimenti; policy SIOC NotApplicable e spiegazione verificate |
| `python3 docs/verification/check_fixtures_plan.py` | PASS, exit 0: 135 campi/72 regole/376 specifiche di caso, indici e hash delle fonti, template/casi condizionali/constraint, obbligo T-P3-04; nessuna fixture eseguita |
| `uv run … --with jsonschema==4.26.0 python docs/verification/check_authoring.py` | PASS: 2 meta-schemi, 3 flow e manifest; 4 hash, copertura 19 constraint/14 assertion legacy e 13 nodi Expr; 33 casi negativi rifiutati; nessuna compilazione DSL Rust |
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

- **SIOC completo rinviato:** dodici dataset con versioni/hash/metadata ICD9
  assenti; controlli semantici esclusi dal profilo minimale per decisione
  dell’autore. mar01 approvato, encoding richiesto al chiamante senza dedurlo
  dai campioni. Nessuna domanda precedente resta pendente su questi punti.
- **Export SIOC:** larghezze aggregate diverse dai layout normativi; chiarire
  il formato prima dell’uso come input del futuro validatore. Nessun adattatore
  o stripping dedotto. [Evidenze](docs/verification/T-P0-08.md).
- **Contratto T-P0-07 approvato:** DEC-AUTHORING-FORMAT-01 e forme dei
  piani insiemi/consenso ratificate; restano da implementare con i gate.
- Dossier ed evidenze: [T-P0-05](docs/verification/T-P0-05.md).
- Dettagli e citazioni in [T-P0-04](docs/verification/T-P0-04.md). Le clausole
  SIOC 1.6/4.28 mantengono il non-enforcement v1 già ricevuto; G03, formato
  UVMD ricevuto e sentinel Barthel 99 non sono nuove decisioni aperte.
- Controllo completo kit FAIL per gli export extra, nessun impatto bloccante
  su T-P1-01: documenti censiti integri e dati esclusi dal nuovo repository.
- Accesso GitHub ora verificato tramite gh/keyring fuori sandbox. Push
  non forzato 69dc995→34e67da e CI manuale sul branch preesistente;
  precedenti limiti di autenticazione sono storici, non più bloccanti.
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

- **T-P0-07 approvata dall’autore il 2026-10-19:** «Approvo il contratto
  T-P0-07», revisione completa del commit 72f9f5e. Formato 1, policy
  persistenti, insiemi/consenso e piani compiled ratificati; non approva
  automaticamente dati mancanti, G0 o runtime.

- **CNF-LINTRA-DSL-01 risolta dall’autore il 2026-10-19:** «Approvo la
  deroga circoscritta (consigliata)». Contratti dei soli substring, età
  compiuta e differenza anni necessari ai profili; implementazione futura
  nelle task pertinenti con gate/test. [Decisione](docs/normative/DECISIONS.md).
  Non approva automaticamente il contratto completo di authoring.
- **Tool documentale:** jsonschema 4.26.0 in ambiente/cache temporanei /tmp
  via uv, download autorizzato tramite escalation; nessuna dipendenza Rust
  o pacchetto aggiunto al progetto. Fonte primaria [EXT] nel README artefatti.

- **DEC-05-SIOC-OPTIONAL-PARENT risolta dall’autore il 2026-10-19:**
  `NotApplicable` per target assente, con evidenza esplicita di anagrafica
  presente e dati del ricovero correlato assenti; nessun errore parent.
  Superata la proposta Unknown per questo caso, senza estendere la policy
  a target ambiguo o invalido. [Decisione registrata](docs/normative/DECISIONS.md).

- **DEC-05-CAD-VALUTAZIONE-INIZIALE risolta dall’autore il 2026-10-19:**
  «Nello storico dell’evento», incluso invio corrente come nella domanda.
  Non richiede un’iniziale nuova in ciascun invio né esattamente una.
  Namespace ULSS conservato, contesto indisponibile distinto da violazione.
  [Decisione registrata](docs/normative/DECISIONS.md).

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

ARCH, DET, PLAN e ADR sorgenti rimangono inalterati. CNF-LINTRA-DSL-01
risolta dalla deroga circoscritta approvata dall’autore; l’allineamento
contrattuale delle task in SCHEMA_AUTHORING.md/lowering.json è stato ratificato
con l’approvazione completa; implementazione solo dopo prerequisiti e gate.
Il contratto completo T-P0-07 è approvato; nessun sorgente architetturale
originale viene riscritto per registrare l’accettazione.
Nessun contratto pubblico Rust modificato o implementazione anticipata.

## Plan change log

- **2026-10-19, autore:** approvato il contratto completo T-P0-07
  (commit 72f9f5e), inclusi formato e allineamenti compiled. Nessuna
  modifica autonoma ai documenti sorgente, nessun avanzamento dei gate runtime.

- **2026-10-19, autore:** approvata la proposta A, deroga circoscritta per
  substring, età compiuta e differenza anni. Implementazione futura con
  prerequisiti/gate/test; nessuna modifica automatica a Piano o DET.
  Allineamento delle task proposto nel contratto T-P0-07 da revisionare.

- **2026-10-16, autore (dal seed):** autorizzato adeguamento ARCH/DET al piano,
  nome Lintra confermato, persistenza promossa a core; ADR-PERSIST-001 approvata.
- **Sessione successiva:** esecuzione continua e parallela autorizzata
  dall'autore; nessuna modifica al Master Implementation Plan. Il piano
  operativo locale organizza la frontiera realmente eseguibile.

## Handoff

HANDOFF — 2026-10-19 — T-P2-02 / mapping unico locale verificato

- **Stato:** DONE_UNVERIFIED; CI della modifica da completare.
- **Prerequisito:** T-P2-01 DONE, CI 39233324913 su 6ac0fb8 entrambi success.
- **File:** domain/errors.rs, export lib/domain, docs/VALIDATION_CODES,
  report T-P2-01/02 e questo stato.
- **Evidenze:** fmt/check/clippy/test/release exit 0; 9 unit, 2 integration,
  5 doctest. Quattro nuovi unit per round-trip, simboli attesi, unicità,
  ordine stabile, RegulatoryRule e negativi/alias/legacy.
- **Decisioni:** DEC-02 applicata senza cambiare ARCH/DET; nessun serializer
  JSON anticipato, mapping unico riutilizzabile dai futuri sink.
- **Next:** CI Linux/Windows, poi preflight T-P2-03.

HANDOFF — 2026-10-19 — T-P2-01 / implementazione locale

- **Stato:** DONE_UNVERIFIED, CI della modifica da eseguire.
- **File:** domain/{ids,byte_range,mod}.rs, lib.rs, report T-P2-01 e stato.
- **Evidenze:** fmt/check/clippy/test/build release exit 0; 5 unit + 2
  integration + 5 doctest, inclusi overflow, zero, slice byte senza copie,
  griglia limite 576 combinazioni e impossibilità di scambiare FlowId/TraceId.
- **Contratti:** sette newtype DET §6 e ByteRange a costruzione verificata;
  errori di costruzione tipizzati, nessuna conversione one-based nel dominio.
- **Limiti:** nessun compiler/persistenza/validator normativo; zero dipendenze.
- **Next:** CI Linux/Windows sul nuovo commit prima di T-P2-02.

HANDOFF — 2026-10-19 — T-P1-03 / CI reale completata

- **Stato:** DONE; G1/QG-2 PASS sul commit 34e67da.
- **Evidenza:** [run 39232752518](https://github.com/UraganoAngelina/lintra/actions/runs/39232752518), workflow_dispatch sul branch
  work/p1-foundations. Linux job 117994423636 e Windows job 117994423276:
  entrambi completed/success; fmt, clippy, test e build release eseguiti.
- **Operazioni:** gh auth status fuori sandbox positivo; repository privata,
  push non forzato dei soli commit, nessuna modifica remota estranea sovrascritta.
- **File:** stato, docs/CI.md e report T-P1-03 aggiornati con evidenza corrente.
- **Next:** T-P2-01 autorizzata e prerequisito verificato. Nessuna deroga a CI;
  le modifiche di dominio richiederanno una nuova run su entrambi i runner.

HANDOFF — 2026-10-19 — T-P0-09 / approvazione ricevuta

- **Stato:** DONE; G0 APPROVED_WITH_DEFERRALS, M0 REACHED.
- **Evidenza umana:** «Approvo G0 con i rinvii indicati», riferita al commit
  407f89e. Conservati tutti i nove esiti e i rinvii puntuali approvati.
- **File:** IMPLEMENTATION_STATUS, report T-P0-09, registro DECISIONS.
- **Verifica:** coerenza stato/checklist/report e diff-check; nessun codice
  Rust modificato, test precedenti non dichiarati come nuove esecuzioni.
- **Next:** completare T-P1-03 con CI Linux/Windows reale prima di T-P2-01.
  gh è ora installato; accesso autenticato verificato fuori sandbox.

HANDOFF — 2026-10-19 — T-P0-09 / checklist pronta

- **Stato:** DONE_UNVERIFIED; G0 AWAITING_AUTHOR_APPROVAL, M0 NOT_REACHED.
- **File:** sezione G0 di IMPLEMENTATION_STATUS e docs/verification/T-P0-09.md.
- **Evidenze:** nove voci DET §106 coperte, ciascuna chiusa per contratto
  corrente o rinviata con ragione e accettazioni/task interessate; riscontro
  di 218 proiezioni campo e riesecuzione controllo T-P0-06 (15/150).
  Controlli T-P0-03..08 e Cargo già positivi nella stessa sessione;
  link e diff-check finali. Nessun test/CI normativo dichiarato.
- **Decisione richiesta:** approvazione esplicita della checklist G0 con
  scope minimale e rinvii. Obbligo dal prompt T-P0-09/Piano §9, non da skill.
- **Limiti:** nessun avanzamento delle task dipendenti fino all’approvazione;
  CI Linux/Windows reale resta prerequisito separato per T-P2-01.
- **Git:** nessun push; modifica storica estranea lasciata fuori dal commit.
- **Next:** registrare risposta autore per chiudere G0; poi risolvere QG-2
  prima dello sviluppo dei tipi T-P2-01. Nessuna task successiva iniziata.

HANDOFF — 2026-10-19 — T-P0-08 / profilo minimale autorizzato

- **Stato:** DONE per DEC-SIOC-MINIMAL-01; baseline FROZEN non modificata.
- **File:** schemas/sioc/2026-04-20.structural-minimal.1 (flow, cinque trace,
  ledger, README); fixtures-plan/catalogo, decisioni, report e verificatori.
- **Evidenze effettive, exit 0:** sioc_minimal_profile.py (JSON Schema,
  riproducibilità, 135 layout, riferimenti, 5 PK/6 join, due negativi);
  check_authoring.py (33 negativi), check_fixtures_plan.py (376 casi pianificati),
  check_rules_and_joins.py, check_notes.py, check_presence.py (218 campi).
  cargo fmt --all -- --check; cargo check --locked;
  cargo clippy --all-targets --locked -- -D warnings; cargo test --locked
  (2 integration test e 3 doctest). Nessun test normativo runtime.
- **Verifiche JSON:** uv run --cache-dir /tmp/lintra-authoring-uv --offline
  --no-project --with jsonschema==4.26.0 python docs/verification/<script>.py.
- **Decisioni:** 050 solo tre prefissi inviante e 4.7; niente lookup; mar01
  futuro; regole omesse esplicite. Persistenza PK/audit resta contratto.
- **Limiti:** lint Rust da rieseguire in T-P3-04; niente compiler, golden,
  validazione pazienti o CI reale. Export letti solo per aggregati tecnici;
  larghezze anomale documentate, nessun dato sanitario copiato nel repo.
- **Git:** file storico Superpowers già modificato lasciato intatto e fuori
  dal commit; nessun push. Risultati più recenti prevalgono sui handoff storici.
- **Next:** T-P0-09, checklist concreta e approvazione finale obbligatoria.
  P2 resta separatamente bloccata sulla CI reale.

HANDOFF — 2026-10-19 — T-P0-08 / preflight e fixture-plan

- **Stato:** BLOCKED sulle parti dipendenti da input normativi; prerequisito
  T-P0-07 DONE con accettazione umana completa, commit 2ba2e95.
- **File:** fixtures-plan.md, sioc/fixtures-catalog.json, checker/report
  T-P0-08 e questo stato. Nessun file schema incompleto presentato come valido.
- **Evidenze:** piano copre 135 campi, 72 regole, 376 specifiche di caso
  campo; hash/indici/template/dipendenze tracciati. Checker authoring e
  matrici passati, 218 campi/policy ricevute fedeli.
- **Blocchi:** dataset/versioni e metadata ufficiali, encoding/caratteri
  per no_lowercase, convenzione compleanno 29 febbraio. Domande inviate,
  nessuna decisione inferita. File reali paziente non consultati.
- **Test:** solo controlli documentali/JSON Schema; niente golden o
  record normativi eseguiti. Cargo non rieseguito, nessun Rust cambiato.
- **Ricontrollo obbligatorio:** linter automatico appena T-P3-04 esiste, sui
  canonical schema da produrre; il presente checker non sostituisce il linter.
- **Git:** work/p1-foundations; piano storico preesistente preservato;
  nessun push o nuova CI.
- **Next:** risolvere gli input e riprendere la stessa T-P0-08; T-P0-09
  non avviata, G0 aperto, T-P2-01 continua ad attendere CI reale.

HANDOFF — 2026-10-19 — T-P0-07 / accettazione umana ricevuta

- **Stato:** DONE; autore: «Approvo il contratto T-P0-07» sul commit 72f9f5e.
- **File:** stato/decisioni, intestazioni di authoring/artefatti e report
  aggiornati; nessun cambiamento delle regole o degli esempi approvati.
- **Verifiche:** check_authoring.py rieseguito, 33 casi negativi superati;
  git diff --check superato. Nessun Rust modificato o Cargo rieseguito.
- **Next:** T-P0-08, previo riscontro delle boundary normative; G0 resta aperto.

HANDOFF — 2026-10-19 — T-P0-07 / contratto pronto per revisione

- **Stato:** DONE_UNVERIFIED, attende revisione umana finale del contratto.
- **Decisione:** deroga circoscritta ai tre derivati approvata dall’autore;
  CNF-LINTRA-DSL-01 risolta, registrata senza cambiare i documenti sorgente.
- **File:** SCHEMA_AUTHORING.md, docs/schema-authoring/* (2 meta-schemi,
  lowering, indice, 3 flow, dataset/manifest sintetici), check_authoring.py,
  report, DECISIONS.md, attributi Git limitati agli esempi e questo stato.
- **Evidenze:** JSON Schema reale jsonschema 4.26.0 temporaneo: 3 flow e
  manifest validi, 4 hash verificati, 33 casi negativi rifiutati, 15 confronti byte con filtri Git autocrlf;
  famiglie 19 constraint/14 assertion legacy/13 Expr coperte; checker P0 precedenti
  passati, 218 campi/fonti integri. Nessun Rust modificato o Cargo rieseguito.
- **Limiti:** prove strutturali, non compilazione DSL Rust né test normativi;
  boundary lessicali/dataset e policy 29 febbraio del profilo esplicite.
  Formato/payload compiled/persistenza richiedono la revisione completa.
- **Git:** work/p1-foundations; piano storico preesistente preservato;
  nessun push o nuova CI. Nessuna dipendenza Cargo aggiunta.
- **Next:** ricevere revisione T-P0-07 e solo poi esaminare T-P0-08; G0
  rimane aperto, P2 continua ad attendere la CI reale Linux/Windows.

HANDOFF — 2026-10-19 — T-P0-07 / riconciliazione bloccata

- **Stato:** BLOCKED; T-P0-03/04/05 verificati, nessun problema di accesso
  ai documenti. Decisione SIOC parent opzionale applicata nei documenti
  e nel checker; T-P0-05 resta DONE (commit 20ab9de).
- **File:** docs/SCHEMA_AUTHORING.md (bozza), report T-P0-07 e stato.
- **Evidenze:** tre checker documentali superati; due JSON sintetici della
  bozza parsabili, digest e link verificati; fonti integre. Cargo non
  rieseguito per soli documenti, nessuna validazione JSON Schema dichiarata.
- **Conflitto:** CNF-LINTRA-DSL-01, valori derivati necessari ai profili ma
  rinviati dal Piano §13.5/DET §135; scelta autore sulla proposta A/B.
- **Preparato:** proposta di formato numerico distinta dalla DSL legacy,
  manifest di bundle, vincoli e checklist authoring ancora da completare.
- **Git:** branch work/p1-foundations; piano storico preesistente inalterato.
  Nessun nuovo push, CI o dipendenza. ARCH/DET/PLAN non modificati.
- **Next:** riprendere T-P0-07 dopo decisione di scope; completare contratto
  ed esempi e ottenere revisione umana. T-P0-08 non avviata; P2 attende CI.

HANDOFF — 2026-10-19 — esecuzione diretta / T-P0-05

- **Stato:** DONE, criteri documentali verificati; G0 ancora aperto.
- **File:** rules-and-joins.md SIOC/CAD, checker e report T-P0-05;
  DECISIONS.md, matrice note CAD, report T-P0-04 e stato aggiornati.
- **Evidenze:** checker regole/join, note e presenza superati; 74 regole,
  dieci join, trenta presence condizionali; fonti ricevute integre.
- **Decisione:** iniziale CAD nello storico; ricerca atto SIOC conserva
  match sull’ID ricevuto (esistenza), senza introdurre filtro UDO.
- **Limiti:** policy parent opzionale SIOC da chiarire; piani CAD da
  formalizzare in T-P0-07. Nessun Rust, runtime normativo o schema mutato.
- **Verifiche:** git diff --check superato; Cargo non rieseguito per soli
  documenti. Nessuna nuova CI/Linux-Windows o push.
- **Git:** work/p1-foundations; modifiche preesistenti al piano storico
  lasciate inalterate.
- **Next:** chiarimento SIOC e quindi T-P0-07; P2 continua ad attendere CI.

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
