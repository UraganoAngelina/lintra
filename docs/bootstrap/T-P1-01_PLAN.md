# T-P1-01 — Piano operativo di bootstrap

Data: 2026-10-16. Ambito autorizzato: esclusivamente T-P1-01.

Fonti: ARCH §3, §31, §39A–D, §45; DET §4–5, §94, §145,
§166–174; Master Plan v1.1 §4, §9/P1, §10.3, §14–15;
`FIRST_TASK_T-P1-01_PROMPT.md` e `TASK_PROMPTS/T-P1-01.md` dello starter kit.
Non modifica il Master Implementation Plan.

## Prerequisiti verificati

- A-02/DEC-01: avvio P1 prima di G0 esplicitamente autorizzato;
  DEC-03: nome Cargo `lintra` confermato.
- Rust e Cargo 1.99.0 disponibili, host `x86_64-unknown-linux-gnu`.
- Nessun repository Git inizializzato nel workspace. La directory tecnica
  `.git` del workspace è vuota e in sola lettura: nuovo repository locale
  nella sottocartella `lintra/`, conforme al layout DET §4, separato dal kit.
- `VERIFY_KIT.py`: FAIL per quattro extra `EXPORT_CAD/{A,B1,C,D}.txt`;
  nessun hash errato, file censito mancante o dimensione errata segnalato
  sui 196 file del manifest; 140 prompt presenti. Non leggere, copiare,
  modificare o versionare gli export reali. Nessun blocco allo scaffold.

## Intervento

- [x] Copiare senza modifiche ARCH, DET, PLAN, ADR-PERSIST-001 e matrice
  di riconciliazione in `docs/`; conservare la task e il primo prompt in
  `docs/bootstrap/`. Verificare le copie byte per byte.
- [x] Creare `Cargo.toml` (package singolo, lib e bin, zero dipendenze),
  `src/lib.rs` con moduli privati e `src/main.rs` con `mod cli` e main vuoto.
  Creare solo `mod.rs` vuoti per gli undici moduli DET §4, inclusa persistence.
  Nessun nuovo contratto pubblico. Riservare le directory dati e test con
  `.gitkeep`, senza importare schemi o dataset normativi.
- [x] Documentare il confine della library con un doctest positivo di
  importazione e doctest negativi per gli interni engine/persistence.
  Non introdurre test di dominio né comandi CLI prima di P8.
- [x] Aggiungere README, `.gitignore` difensivo e stato derivato dal seed.
  Verificare che export, stato runtime, report e segreti siano ignorati,
  mentre sorgenti, documenti, lockfile e fixture sintetiche restino ammessi.
- [x] Eseguire fmt, build/check con lockfile, clippy con warning negati,
  test e smoke dell'entrypoint; registrare output e Git status nello stato.
  `Cargo.lock` è generato da Cargo per consentire i controlli `--locked`:
  pin toolchain, attributi Git e configurazione Windows restano T-P1-02.
- [x] Riesaminare lo scope, aggiornare handoff e fermarsi. Prossima task:
  T-P1-02, solo dopo autorizzazione. CI Linux/Windows rimane T-P1-03.

## Riesame finale

Controllare: nessuna esposizione di moduli interni; `cli` solo nel binario;
modulo persistence vuoto; nessuna dipendenza; nessun file sanitario copiato;
documenti sorgente invariati; nessun claim CI/Windows; nessun avanzamento
delle task successive. Le evidenze consuntive sono in `IMPLEMENTATION_STATUS.md`.
