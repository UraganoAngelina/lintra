# Fondamenta Lintra — piano esecutivo

> **For agentic workers:** usare Superpowers subagent-driven-development e
> dispatching-parallel-agents. L'autore autorizza dal 2026-10-16 esecuzione
> continua e parallelismo, rispettando le dipendenze funzionali.

**Goal:** completare la frontiera eseguibile dopo T-P1-01 e rendere espliciti
i gate che richiedono risorse o decisioni esterne.

**Architecture:** package Rust singolo esistente, core privato e CLI separata;
nessuna logica normativa introdotta dalle task di infrastruttura. L'inventario
P0 è indipendente dalla configurazione P1 e può procedere in parallelo.

**Tech Stack:** Rust/rustup, Git, GitHub Actions; nessuna dipendenza Rust nuova.

**Spec:** `docs/MASTER_IMPLEMENTATION_PLAN.md` §4, §10.2–10.4, §14;
prompt originali nella directory adiacente dello starter kit.

## Global Constraints

- Nessun dato sanitario reale, identificativo o credenziale nei file/test.
- Non modificare ARCH, DET, Master Plan o baseline normative.
- `--locked` nei controlli; Windows GNU solo sviluppo, MSVC sul runner reale.
- CI/Linux+Windows richiesta da T-P1-03 non è sostituibile da test locali.
- T-P2-01 dipende da T-P1-03 verificata; A-02 deroga G0, non la CI.
- L'autorizzazione corrente sostituisce lo stop ordinario dopo ciascuna task;
  decisioni normative e gate umani restano vincolanti.
- Ogni implementer modifica solo i propri file; coordinatore solo stato,
  ledger e integrazione. Nessun worker avvia altri agenti o altera Git.

## Review Focus

- Attributi Git: fixture/golden con LF/CRLF preservati anche su Windows.
- Rustup: versione fissata realmente disponibile e componenti/target installabili.
- CI: target nativo MSVC senza linker mingw/runner Wine per errore.
- Test I/O: directory con spazi, byte invariati e cleanup anche in caso di errore.
- Inventario: distinguere fonte ufficiale ricevuta, schema derivato e dato mancante;
  nessun gate normativo dedotto da documenti di audit o sample reali.

### Task 1: T-P1-02 — Toolchain e ciclo di sviluppo

**Prerequisito:** T-P1-01 verificata localmente; commit lockfile richiede identità Git.
**Files:** `rust-toolchain.toml`, `.gitattributes`, `.cargo/config.toml`,
`docs/DEVELOPMENT.md`; Cargo.lock esistente, da registrare nel commit.
**Interfaces:** produce toolchain fissata per CI; nessuna modifica a src o API.

- [x] Verificare disponibilità della versione 1.99.1 scelta dall'autore e fonti ufficiali;
  installazione toolchain/target solo tramite i permessi dell'ambiente.
- [x] Profilo minimal, rustfmt/clippy, target x86_64-pc-windows-gnu.
- [x] Attributi `* text=auto`, fixture e golden `-text`; verificare con
  `git check-attr` e roundtrip sintetico LF/CRLF in repository temporaneo.
- [x] Configurare linker mingw/runner wine esclusivamente per target GNU;
  documentare sviluppo Arch e separazione dalla CI MSVC.
- [x] Eseguire fmt/check/clippy/test e cross-check disponibile, registrare
  output reale e limitazioni; commit coeso se identità disponibile; review.

### Task 2: T-P0-01 — Inventario input normativi (indipendente)

**Prerequisito:** nessuna task; verifica disponibilità fonti, senza leggere pazienti.
**Files:** `docs/normative/INPUT_INVENTORY.md`.
**Interfaces:** classificazione ricevuto/mancante/non applicabile per DET §154.

- [x] Leggere DET §106 e §154, inventario preesistente e manifest del kit.
- [x] Verificare file realmente presenti e provenienza; separare PDF/XLSX
  normativi dagli schemi derivati, dataset ufficiali e fixture mancanti.
- [x] Coprire ogni categoria di DET §154 con stato, evidenza e lacune.
- [x] Verificare completezza/tracciabilità; non inventare test di runtime;
  consegnare elenco mancanti e review; nessuna chiusura G0.

### Task 3: T-P1-03 — CI Linux/Windows

**Prerequisito:** T-P1-02 completata; risorsa esterna: repository GitHub.
**Files:** `.github/workflows/ci.yml`, `tests/platform_paths.rs`.
**Interfaces:** consuma toolchain fissata; nessuna API di dominio.

- [ ] Dopo Task 1, implementare matrice Ubuntu/Windows con fmt, clippy,
  test, build release `--locked`, target nativi espliciti e permessi minimi.
- [ ] Test di I/O su percorso con spazi: roundtrip bytes sintetici e
  errore NotFound su file mancante; cleanup senza dipendenze nuove.
- [ ] Verificare YAML e test locali; review, poi eseguire CI reale nel
  repository indicato dall'autore, senza pubblicazioni implicite.
- [ ] Senza run reale annotare DONE_UNVERIFIED/BLOCKED per il gate;
  non avviare T-P2-01. Con run verdi proseguire secondo la mappa task.

### Task 4: T-P0-06 — Onboarding reference dataset (dopo Task 2)

**Prerequisito:** T-P0-01 verificata; indipendente da CI e toolchain.
**Files:** `docs/normative/reference-datasets.md`.
**Interfaces:** schede documentali DET §108–109, nessuna API/runtime o import dati.

- [x] Leggere prompt T-P0-06, inventario verificato, specifica reference SIOC,
  manifest SIOC/CAD, tre JSON CAD e relative decisioni approvate.
- [x] Scheda per ciascuna delle dodici dipendenze SIOC e dei tre dataset CAD:
  id/nome, versione/validità, fonte, formato, chiave/lunghezza, encoding,
  normalizzazione e validità temporale; dato assente = lacuna esplicita.
- [x] Distinguere catalogo completo non fornito, estrazione regionale fornita,
  sentinel dello schema e dataset posizionale; non inventare normalizzazioni
  o validità temporali né ridurre attributi a semplice insieme di codici.
- [x] Verificare schede rispetto agli input e registrare lacune/review;
  nessun test runtime artificiale e nessuna chiusura G0.

### Task 5: T-P0-02 — Tabelle campi per revisione (dopo Task 2)

**Prerequisito:** T-P0-01 verificata; nessuna dipendenza da T-P0-06 o CI.
**Files:** `docs/normative/sioc/<trace>-fields.json`,
`docs/normative/cad/<trace>-fields.json`, relativa nota di provenienza/revisione.
**Interfaces:** tabelle documentali dei campi, nessun compiler o modifica schemi.

- [x] Leggere prompt T-P0-02 e DET §107, inventario e baseline ricevute.
- [x] Trascrivere meccanicamente tutti i campi (ID, descrizione, tipo,
  posizione, lunghezza, obbligatorietà e note originali) dai JSON ricevuti,
  dichiarando sempre che per SIOC la fonte ricevuta è FROZEN derivata,
  mentre XLSX primario non è disponibile; CAD è RELEASE_CANDIDATE.
- [x] Per CAD confrontare disposizione fisica e note con PDF ufficiale
  disponibile, segnalando ogni limite/discrepanza senza correggere la fonte.
- [x] Verificare conteggi, fedeltà ai JSON e continuità/gap/overlap;
  alias mantenuti senza inventare normalizzazioni. Nessun dato sanitario.
- [x] Consegna per revisione umana a campione richiesta dalla task;
  senza approvazione non marcare DONE né avanzare a T-P0-03/04.

### Task 6: T-P0-03 — Matrici presenza/assenza/padding (dopo Task 5 approvata)

**Prerequisito:** T-P0-02, campione approvato dall'autore il 2026-10-16.
**Files:** `docs/normative/sioc/presence-matrix.md`,
`docs/normative/cad/presence-matrix.md`.
**Interfaces:** documentazione per campo; nessuna nuova policy runtime.

- [ ] Leggere prompt originale e DET §14–15/§106–107, schemi ricevuti e
  tabelle approvate; mantenere separate obbligatorietà sorgente e schema.
- [ ] Per tutti i 218 campi trascrivere presenza, condizioni dichiarate,
  missing-value policy e alignment/padding ricevuti; zero semantico mai
  automaticamente vuoto. Mancanza = boundary motivata, nessuna norma inventata.
- [ ] Documentare alias OBB/FAC/NBB e CAD SI/NO senza appiattire colonne,
  tipi sorgente o condizioni già configurate; citare eccezioni/decisioni.
- [ ] Verificare copertura e fedeltà a tutti i campi/AST/policy ricevuti;
  review indipendente e stato. Nessun compiler o test runtime artificiale.

### Task 7: T-P0-04 — Classificazione note (parallela a Task 6)

**Prerequisito:** T-P0-02 approvata; indipendente da Task 6.
**Files:** `docs/normative/sioc/notes-matrix.md`,
`docs/normative/cad/notes-matrix.md`.
**Interfaces:** matrice documentale note/regole già ricevute/ambiguità.

- [ ] Leggere prompt originale e ARCH §5.7/§36, DET §107/§144, audit e
  decisioni ricevute; nessuna traduzione automatica di note in runtime.
- [ ] Classificare ogni nota come metadata, regola eseguibile candidata o
  ambigua, motivando la classe con vincoli/AST esistenti o decisioni ricevute.
- [ ] Preservare note e riferimenti; registrare ambiguità puntuali DEC-05
  senza correggere o approvare norme, distinguendo metadati storici superati.
- [ ] Verificare copertura 218/218 e tracciabilità ai testi/tabelle/fonti;
  review indipendente. Nessuna modifica di schema o nuova regola eseguibile.

### Task 8: T-P0-05 — Regole, join e identità (dopo Task 7)

**Prerequisito:** T-P0-04 verificata. Indipendente dal completamento di Task 6.
**Files:** `docs/normative/sioc/rules-and-joins.md`,
`docs/normative/cad/rules-and-joins.md`.
**Interfaces:** catalogo documentale di regole/chiavi/policy ricevute.

- [ ] Leggere prompt originale, ARCH §21–22 e DET §66–72/§91, matrice note,
  JSON ricevuti e contratti persistenti approvati.
- [ ] Elencare regole cross-field/cross-trace, identità, PK composte e join
  con riferimenti esatti, cardinalità e policy mancanti senza inventare default.
- [ ] Ogni join ha cardinalità/policy ricevuta oppure DEC-* esplicita;
  distinguere requisiti di estensione e non-enforcement deliberato.
- [ ] Verificare completezza contro tutte le regole/join/chiavi sorgenti,
  rispettare audit-only SIOC inter-run e progressivi CAD/namespace; review.

## Handoff e ripresa

`IMPLEMENTATION_STATUS.md` conserva consuntivo, gate, decisioni, verifiche e
autorizzazione corrente. Ledger Superpowers dedicato a questo piano;
nessuna task successiva viene marcata eseguibile senza evidenze dei prerequisiti.
