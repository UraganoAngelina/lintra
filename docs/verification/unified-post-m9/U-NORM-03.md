# U-NORM-03 — Enforcement SIOC cross-run generico

Data: 11 novembre 2026, Europe/Rome. Stato: **IN_PROGRESS**.
Prerequisiti DONE: U-NORM-01 `a5689ae`, U-NORM-02 `672459e`, U-NORM-05
`d49386b6c8f5567e2131166f88704d6b494d2c97`.
Decisione: [DEC-SIOC-HIST-02-A01](../../normative/DEC-SIOC-HIST-02-A01.md).
G-SIOC1 **NOT_VERIFIED**: qualifica locale/nativa e bridge UI da completare.
Le 140 task M9 non cambiano.

## Perimetro e provenienza

Il commit di salvataggio `e357f1c` contiene l'avvio effettivo del runtime,
compiler, profilo e test: non chiude la task. Il delta successivo rafforza
contratti della summary, golden canonico, CLI/SQLite, bootstrap approvato e
cancellazione nella prima passata. Nessuna CI precedente qualifica questo
codice. La preparazione del runner Windows è separata in `ab93e71`; i due FAIL
precedenti restano tracciati nel dossier U-FIX-04.

Nuovo profilo: `schemas/sioc/2026-04-20.cross-run.1/flow_schema.json`, con
scope-ledger. Eredita il componente referenziale effettivamente qualificato
SHA256 `b7a5a8d91ffe6aecff39065ea4cfd634551b7cc83d4ceff219bdc6bca9b96ec7`:
26 campi fisici, sei controlli esistenti, cinque tracciati. Aggiunge solo D11;
non dichiara eseguibile/conforme l'intero workbook di135 campi e non rimuove i
deferred di structural-minimal.2. PK originali identiche, in particolare
4.1=(4.1,4.7,4.8,4.19.1). Nessuna modifica FROZEN, golden legacy o dataset.

Documentazione attuativa e limiti: [SIOC_CROSS_RUN](../../SIOC_CROSS_RUN.md).

## Strutture, funzioni e condizioni

- `schema/raw.rs`, `compiled/plans.rs`, `cross_run.rs`, `persistence_profile.rs`:
  nodo opzionale formato3, capability esplicita `persistence.cross_run.v1`,
  parametri anno/mese interi compilati e separati dal namespace; binding di
  tracciato/record_key e codici verificati prima di input/storage. Combinazioni
  non implementate con snapshot/temporal/invarianti sono rifiutate. Policy
  assente conserva serializzazione e comportamento precedenti. Meta-schema
  authoring formato3 aggiornato; formati1/2 restano invariati.
- `schema/historical.rs`, `registry.rs`, `compiler.rs`, `namespace_binding.rs`:
  il produttore reale è selezionato dalla policy compilata. I vecchi entrypoint
  ordinario/audit-only rifiutano il nuovo nodo; nessuna rimozione di capability
  per aggirare il compiler, nessun nome SIOC nel motore generico.
- `application/session.rs`: binding mensile canonico nel campo period esistente,
  con namespace, flow, nomi/valori parametri e submission ID. I binding CAD/audit
  precedenti restano byte-compatibili. Il costruttore pubblico di draft rifiuta
  la promozione di un profilo normativo; solo il servizio effettivo può produrre
  la validazione storica necessaria alla preview e all'approval CAS.
- `application/cross_run.rs`: `prepare` verifica pagine bounded128, continuità,
  generazione, HMAC, stato approvato, schema/hash e binding; seleziona una sola
  baseline PRESENT pertinente per tracciato/anno e verifica snapshot/conteggi.
  `observe` confronta PK protetta e fingerprint del buffer realmente validato,
  distingue carry/new/mismatch/riuso, ed esclude il record rifiutato dai join.
  Per assenza nella baseline interroga l'ultima occorrenza esatta anche negli
  anni precedenti: nessun reset annuale delle identità o somma indiscriminata
  delle ricoperture. `finish` verifica copertura e omissioni senza cancellare
  lo storico. Errori storage/protezione/CAS restano tecnici, carenza di prova
  resta Unknown. Il replay esatto usa la propria revisione immutabile.
- Un database inizializzato non prova automaticamente l'assenza normativa di
  PK. Prima di qualsiasi acquisizione, il servizio ammette un invio realmente
  vuoto di gennaio per preview/review/commit di una baseline iniziale approvata;
  `initial_baseline` lo espone nel report. Una riga non vuota prima di tale
  approvazione resta Unknown, nuovi=0. Nessun flag di bypass o scrittura diretta
  di witness. Ciò riguarda l'archivio locale selezionato, non attesta altri
  archivi aziendali; non migra né ripara una storia incompleta/legacy.
- `historical_validation.rs`: mantiene raccolta dai buffer reali, differisce
  solo le callback terminali, restituisce ValidationOnly per Invalid/Partial/
  Cancelled; usa il servizio ordinario di review/commit per i risultati completi.
  Una preview non approvata non modifica mai la baseline.
- `engine/flow_execution.rs`: hook neutrali di default preservano i producer
  legacy. Solo i tracciati dichiarati differiscono il duplicato intra-invio
  alla seconda passata (4.1 nel profilo SIOC). Errori storici e duplicati hanno
  ordine stabile per tracciato/riga, budget e cancellazione esistenti, conteggi
  invalidi una sola volta per record, hash/conteggi fisici immutati. Il record
  duplicato è escluso dalle proiezioni valide. L'uscita anticipata conserva i
  diagnostici rilevati ma non emessi, senza emetterli nella passata sbagliata.
- `domain/full_validation.rs`, `application/full.rs`, `lib.rs`: summary opzionale
  versionata con generazione/revisione, anno/mese, inizializzazione, classificazioni,
  omissioni, Unknown e errori non emessi; niente PK/righe cliniche nei report.
  Il contratto controlla overflow, conteggi, codici, unicità dei tracciati e
  coerenza con le evidenze reali. None non altera i vecchi report.

Il contenuto canonico dichiarato è l'intera riga fixed-width, inclusi padding
ed eventuali span opachi, esclusi soltanto LF/CRLF. Si riusa il fingerprint HMAC
esistente con definizione fisica/encoding qualificata, senza trim/equivalenze
inventate, chiavi nuove, HMAC1/codec nuovo o righe in chiaro persistite.

## Oracle sintetico e controlli

`tests/cross_run_validation.rs` usa il servizio applicativo reale, repository
in memoria e SQLite effettivo, CLI subprocess e writer JSONL esistente. Il golden
nuovo `tests/golden/sioc-cross-run/v1/expected.errors.jsonl` contiene due errori
attesi indipendenti, confrontati byte per byte su ogni OS della nuova CI.

| Acceptance ratificata | Prova implementata |
| --- | --- |
| 1/2/3/4/12: gennaio, febbraio, marzo e ricoperture | Approvazioni sequenziali reali; carry2/3, nuovi1, nessun evento logico moltiplicato |
| 5: stessa PK contenuto diverso | Rifiuto in seconda passata, errore canonico; caso anche su profilo SIOC con join e skipped fedeli |
| 6: duplicato intra-invio | 1–3 preservati; 4.1 emesso dopo CrossStarted e scartato dalle proiezioni; max_errors/cancel |
| 7: omissione obbligatoria | Conteggio mancante, Partial/ValidationOnly, generazione e snapshot immutati |
| 8: storia assente/incompleta | Primo febbraio Unknown; anche gennaio non vuoto senza approvazione iniziale Unknown; pagina/snapshot/lookup guasti tecnici |
| 9/10: namespace e tracciati | Stessa PK numerica in namespace approvato distinto e tracciato diverso rimane isolata |
| 11/14: reopen e replay | SQLite reale su filesystem locale, riapertura, replay della stessa sessione senza nuova revisione |
| 13: report e CAS falliti | Nessuna modifica della baseline perdente; commit concorrente vincente resta unica revisione nuova |
| 15: legacy e fingerprint | Snapshot vecchio invariato, nuovo controllo Unknown; LF/CRLF equivalenti solo per terminatore |
| 16: determinismo nativo | Golden byte-exact presente; esecuzione Linux locale, Windows/Linux CI ancora da acquisire |
| 4.1 quattro campi | Confronto PK con fonte .2; ciascun componente distingue; riuso identico/modificato cross-run rifiutato |
| Cambio d'anno | Copertura nuova annualità, identità pregresse non diventano nuove |
| Confini applicativi | Compiler fallisce chiuso su sei binding errati; public draft non promuove audit a norma; CLI usa lo stesso Rust |

Evidenza negativa preservata `U-NORM-03-cancel-negative`: la cancellazione dopo
4.1 in passata1 aveva prodotto `InconsistentFullAudit` perché i duplicati
correttamente differiti non erano ancora emessi. La correzione conserva il
conteggio `unreported_errors` e mantiene Cancelled con zero diagnostici emessi.
Non si alterano max_errors, cancellation, phase-major o golden per nascondere
il problema. I due tentativi del caso SIOC mismatch avevano una fixture errata:
il primo offset modificava la PK1.1 anziché uno span opaco. La prova finale
modifica il byte21, fuori dalla PK, e attende il codice dichiarato dal profilo
SIOC `E_HISTORICAL_KEY_CONTENT_MISMATCH`; nessuna modifica del runtime per
adattarlo a quelle aspettative errate. I risultati finali e le identità dei sorgenti sono acquisiti in
JSON/log.gz separati, senza sovrascrivere i tentativi.

## Limiti e handoff

La UI React/Tauri non è ancora realizzata: consumerà il servizio Rust e queste
summary; niente norma TypeScript. G-SIOC1 finale e G-REL non sono PASS.
I join referenziali restano current-run: l'assenza di un tracciato amministrativo
necessario può rendere Full Partial; non viene inventato un fallback storico,
né richiesta una ricopertura4.1 illecita per mascherarlo. Il bridge operativo
andrà qualificato nelle task UI previste dopo G-MIG-4.

Task attiva03;04 non avviata prima del suo prerequisito effettivo. In attesa
del gate remoto è READY indipendente U-OPS-07. T-MIG-P0-01 attende nuova CI
nativa pertinente; nessuno scaffold o P0-02 prima della baseline qualificata.
