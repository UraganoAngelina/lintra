# Budget di risorse soft (D09): estensione additiva IPC v1

T-MIG-P5-05, 23 novembre 2026. Estensione additiva del [wire ratificato](CONTRACT.md)
e del [lifecycle](LIFECYCLE.md), autorizzata dall'autore per completare D09
nella GUI (Master Plan §4, [RESOURCE_BUDGETS](../../RESOURCE_BUDGETS.md),
U-FIX-03). Nessun campo o valore esistente cambia: `schemaVersion` resta 1,
le operazioni precedenti hanno payload e risposte identici, gli stati dello
snapshot restano quelli del lifecycle.

## Configurazione

Il file è il JSON versionato della CLI (`--resource-budget`), stesso parser
stretto (`ResourceBudget::parse` nel core): `version` 1, `measurement_source`
non vuota, soglie intere positive o `null`, nessun campo sconosciuto. Si
chiama `resource-budget.json` e sta nella directory di configurazione
per-utente che Tauri risolve per l'app (`app_config_dir`, identifier
`it.lintra.desktop`). Rust legge il file a ogni avvio di un run, entro 64 KiB
(limite di lettura, non budget).

| File | Stato | Effetto |
| --- | --- | --- |
| assente (o directory non risolvibile) | `unconfigured` | nessun controllo soft, come la CLI senza file; lo stato è mostrato |
| valido | `configured` | preflight Rust prima della validazione |
| presente ma non valido, troppo grande, non regolare o illeggibile | `invalid` | l'avvio è rifiutato (`IPC_INVALID_STATE`); mai saltato in silenzio |

Nessuna soglia di prodotto viene inventata: i valori arrivano solo dal file.

## Operazioni

| Operazione | Payload | Risposta |
| --- | --- | --- |
| `resource_budget_state` | `{schemaVersion}` | `{schemaVersion, state: unconfigured\|configured\|invalid, input_bytes, reference_bytes}`; soglie u64 positive in testo o `null`, `null` se non `configured` |
| `validation_budget` | `{schemaVersion, run}` | `{schemaVersion, run, state, audit, confirmation_ref}` |
| `validation_budget_decide` | `{schemaVersion, run, confirmation_ref, decision: proceed\|cancel}` | `{schemaVersion, run, disposition: proceeding\|cancelled}` |

Stati di `validation_budget` (gate del run, catturato all'avvio):

| state | audit | confirmation_ref | Significato |
| --- | --- | --- | --- |
| `unconfigured` | null | null | run avviato senza controllo soft |
| `checking` | null | null | il preflight legge input e reference selezionati |
| `within_budget` | presente, nessuna soglia superata | null | il run procede |
| `confirmation_required` | presente, almeno una soglia superata | handle | il worker attende Annulla / Procedi comunque |
| `proceeded` | presente, soglia superata | null | Procedi comunque per questo preflight |
| `declined` | presente | null | Annulla o cancel mentre l'avviso era aperto |
| `not_completed` | null | null | preflight interrotto da cancel o errore hard: vale l'outcome del run |

`audit` ha esattamente `schemaVersion`, `input_bytes`, `reference_bytes`,
`missing_reference_files`, `input_budget`, `reference_budget`,
`input_budget_exceeded`, `reference_budget_exceeded`. I contatori sono quelli
del preflight Rust; i flag devono coincidere con `byte > soglia` (il client
rifiuta un DTO incoerente). Nessun path, digest, testo di `measurement_source`
o dato di record attraversa l'IPC.

## Regole

- Il preflight gira sul thread del worker, prima che venga aperto qualsiasi
  output; lo snapshot resta `preparing`. Il `PreparedResourcePreflight` del
  core resta in Rust: il renderer non può falsificare totali o accettazione.
- `confirmation_ref` è un nonce di 128 bit generato per run, pubblicato solo
  in `confirmation_required` e consumato dalla prima decisione: una seconda
  decisione, un handle diverso, un run diverso o una decisione dopo cancel
  danno `IPC_STALE_CONTEXT`; una decisione senza avviso pendente su un run
  ancora aperto dà `IPC_INVALID_STATE`.
- Annulla equivale a una richiesta di cancel di quel run: terminale
  `not_started` con `cancel_requested=true`, nessun report.
- Procedi comunque esegue il run completo: nessun troncamento, EOF, SHA,
  audit, `max_errors` e cancel invariati. Subito prima della validazione
  `verify_unchanged` ricontrolla richiesta, schema e file: un file sostituito
  o modificato invalida la conferma (errore tecnico, nuovo avvio richiesto).
- I limiti hard (I/O, file non regolare, overflow, confine reference) sono
  errori tecnici: nessuna decisione li aggira.
- Cancel e chiusura dell'app svegliano il worker in attesa (token controllato
  ogni 50 ms); un secondo start mentre l'avviso è aperto è `IPC_BUSY`.
- La scelta è tracciata: il gate del run la espone fino al nuovo run, e il
  processo scrive su stderr la stessa riga aggregata della CLI
  (`ResourceBudgetAudit`, nessun path o identificativo).

Test: `src-tauri/src/budget.rs`, `src-tauri/src/jobs/budget_lifecycle.rs`,
`configuration.rs::budget_file_is_read_at_each_start_and_invalid_never_skips_the_gate`,
`tests/run.test.mjs`, `tests/resource_budget.rs` (parser).
