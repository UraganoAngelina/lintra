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

- [ ] Verificare disponibilità della versione 1.99.1 scelta dall'autore e fonti ufficiali;
  installazione toolchain/target solo tramite i permessi dell'ambiente.
- [ ] Profilo minimal, rustfmt/clippy, target x86_64-pc-windows-gnu.
- [ ] Attributi `* text=auto`, fixture e golden `-text`; verificare con
  `git check-attr` e roundtrip sintetico LF/CRLF in repository temporaneo.
- [ ] Configurare linker mingw/runner wine esclusivamente per target GNU;
  documentare sviluppo Arch e separazione dalla CI MSVC.
- [ ] Eseguire fmt/check/clippy/test e cross-check disponibile, registrare
  output reale e limitazioni; commit coeso se identità disponibile; review.

### Task 2: T-P0-01 — Inventario input normativi (indipendente)

**Prerequisito:** nessuna task; verifica disponibilità fonti, senza leggere pazienti.
**Files:** `docs/normative/INPUT_INVENTORY.md`.
**Interfaces:** classificazione ricevuto/mancante/non applicabile per DET §154.

- [ ] Leggere DET §106 e §154, inventario preesistente e manifest del kit.
- [ ] Verificare file realmente presenti e provenienza; separare PDF/XLSX
  normativi dagli schemi derivati, dataset ufficiali e fixture mancanti.
- [ ] Coprire ogni categoria di DET §154 con stato, evidenza e lacune.
- [ ] Verificare completezza/tracciabilità; non inventare test di runtime;
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

- [ ] Leggere prompt T-P0-06, inventario verificato, specifica reference SIOC,
  manifest SIOC/CAD, tre JSON CAD e relative decisioni approvate.
- [ ] Scheda per ciascuna delle dodici dipendenze SIOC e dei tre dataset CAD:
  id/nome, versione/validità, fonte, formato, chiave/lunghezza, encoding,
  normalizzazione e validità temporale; dato assente = lacuna esplicita.
- [ ] Distinguere catalogo completo non fornito, estrazione regionale fornita,
  sentinel dello schema e dataset posizionale; non inventare normalizzazioni
  o validità temporali né ridurre attributi a semplice insieme di codici.
- [ ] Verificare schede rispetto agli input e registrare lacune/review;
  nessun test runtime artificiale e nessuna chiusura G0.

### Task 5: T-P0-02 — Tabelle campi per revisione (dopo Task 2)

**Prerequisito:** T-P0-01 verificata; nessuna dipendenza da T-P0-06 o CI.
**Files:** `docs/normative/sioc/<trace>-fields.json`,
`docs/normative/cad/<trace>-fields.json`, relativa nota di provenienza/revisione.
**Interfaces:** tabelle documentali dei campi, nessun compiler o modifica schemi.

- [ ] Leggere prompt T-P0-02 e DET §107, inventario e baseline ricevute.
- [ ] Trascrivere meccanicamente tutti i campi (ID, descrizione, tipo,
  posizione, lunghezza, obbligatorietà e note originali) dai JSON ricevuti,
  dichiarando sempre che per SIOC la fonte ricevuta è FROZEN derivata,
  mentre XLSX primario non è disponibile; CAD è RELEASE_CANDIDATE.
- [ ] Per CAD confrontare disposizione fisica e note con PDF ufficiale
  disponibile, segnalando ogni limite/discrepanza senza correggere la fonte.
- [ ] Verificare conteggi, fedeltà ai JSON e continuità/gap/overlap;
  alias mantenuti senza inventare normalizzazioni. Nessun dato sanitario.
- [ ] Consegna per revisione umana a campione richiesta dalla task;
  senza approvazione non marcare DONE né avanzare a T-P0-03/04.

## Handoff e ripresa

`IMPLEMENTATION_STATUS.md` conserva consuntivo, gate, decisioni, verifiche e
autorizzazione corrente. Ledger Superpowers dedicato a questo piano;
nessuna task successiva viene marcata eseguibile senza evidenze dei prerequisiti.
