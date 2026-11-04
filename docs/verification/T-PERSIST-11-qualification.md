# T-PERSIST-11 — visibilità, recovery e portabilità SQLite

**Stato corrente: DONE su633f61f, CI41805357515; evidenze native in fondo.**

4 novembre 2026. Ripresa dal working tree corrente, senza ricostruire sessioni
dalla cronologia Git. Scope: test sintetici, hook già limitati a `cfg(test)` e CI.
Nessuna API pubblica, query produttiva, schema normativo, dipendenza o UI cambia.
Preflight: [prerequisiti e coverage esistente](T-PERSIST-11-preflight.md).

## Perché queste prove

ARCH45, DET166–174, ADR001 e la ratifica ADR002 richiedono snapshot completi,
rollback reale, CAS e distinzione fra fallimento tecnico ed esito normativo.
Le prove precedenti di crash nel commit hook, commit riuscito senza risposta,
contesa fra processi, indici e corruzione restano nella suite: non sono sostituite
dal semplice roundtrip di una serializzazione Rust.

Il modulo `src/persistence/sqlite/tests/qualification.rs` aggiunge letture durante
una scrittura interrotta e scambio del **file SQLite chiuso** fra runner nativi.
Il trasferimento usa esclusivamente dati e chiave pubblici sintetici. Non è
un'importazione/restore disponibile nel prodotto né una politica di backup.

## Hook e processi

`TestFault::PauseBeforeCommit` sospende il writer dopo gli INSERT reali e prima
di COMMIT. `before_commit` forza `cache_flush`: le pagine sporche raggiungono
il file sotto rollback journal. Uccidere il processo richiede quindi recovery
del disco, oltre alla perdita del suo staging in memoria. La variante già
esistente di uscita nel commit hook conserva il proprio comportamento.

`wait_at_barrier` scrive un marker dopo il punto controllato, poi attende `go`
con deadline di20secondi. `process_worker` usa la barriera dopo staging oppure
nel punto SQL; i marker distinti impediscono di confondere le due fasi.
Sono helper privati del test executable: non vengono compilati nel prodotto.

`OwnedChild` possiede il processo figlio. In `Drop` verifica se è già terminato;
altrimenti lo uccide e lo attende. Così un'asserzione fallita nel parent non
lascia un writer orfano. `wait_for_marker` controlla anche l'uscita precoce del
figlio e ha una deadline di25secondi: l'assenza di sincronizzazione fallisce.
I figli sono necessari alla simulazione esplicita di crash/concorrenza; non
sono daemon né processi shell di lavoro lasciati in background.

`committed_or_busy` confronta l'intero risultato con la baseline. Solo nella
fase SQL ammette `StorageBusy`, errore tecnico esplicito: non converte lock o
errore in snapshot vuoto. La fase di solo staging deve rimanere leggibile.

Il test di interruzione usa una connessione reader indipendente e verifica
generazione1, snapshot corrente completo, assenza della revisione2, assenza
della submission candidata e assenza della nuova identità. Dopo kill/wait,
chiude il reader, riapre e ripete i controlli sullo stato recuperato. Infine
pubblica lo stesso candidato e ne verifica snapshot/receipt: un archivio
leggibile ma non più scrivibile non supera il test.

## Fixture nativa e validazione del trasferimento

`expected_snapshots` costruisce tre snapshot: revisioni1/2 del namespaceA e
revisione1 del namespaceB. Le identità omesse dalla revisione2 devono restare
ricercabili con provenienza della revisione1. I risultati attesi derivano dai
costruttori sintetici esistenti, non dal contenuto riaperto.

`TransferManifest` contiene kind versionato, OS produttore e SHA256 del DB;
`deny_unknown_fields` rifiuta estensioni tacite. `digest` calcola l'hash di
trasporto; non sostituisce la verifica HMAC dello storage. `storage_io` traduce
errori del test transfer in errore tecnico redatto. `bounded_file` legge al
massimo limite+1byte, rifiutando payload oltre4MiB e manifest oltre16KiB.
Questi limiti riguardano la piccola fixture, non i carichi del prodotto.

`export_fixture` crea l'archivio attraverso l'adapter reale, pubblica le tre
revisioni e chiude tutte le connessioni prima di leggere `lintra.db`. Cerca
esplicitamente la chiave pubblica ripetuta e alcuni marker raw sintetici nel
file: è un controllo mirato, non una prova generale di anonimizzazione.
Accetta solo una directory di output nuova e crea il DB con `create_new`;
un'esportazione precedente non viene sostituita silenziosamente.

`import_fixture` verifica JSON, kind, OS supportato e hash **prima** di creare
un archivio. Crea poi un contenitore privato attraverso l'adapter, lo chiude,
copia il piccolo DB sintetico nel file privato già creato, sincronizza e riapre
con chiave esplicita. I permessi degli artifact scaricati non diventano ACL
accettate dal backend. Nessuna funzione di restore è esportata fuori dai test.

`verify_fixture` controlla formato storage2, DELETE/EXTRA effettivi, audit di
protezione, ogni snapshot e ogni identità autenticati, receipt risolvibili,
generazione corrente per entrambi i namespace, provenienza della chiave omessa
e paginazione delle submission. Non confronta l'uguaglianza binaria di due DB
creati separatamente: verifica lo stesso DB trasportato e i suoi dati attesi.

I test locali verificano roundtrip, rifiuto chiave errata con file immutato,
hash corrotto senza creazione, SQLite malformato anche con hash coerente senza
reinizializzazione, JSON malformato/campi extra/manifest sovradimensionato senza
creazione. Gli `unwrap` appartengono esclusivamente agli oracle sintetici:
un errore fa fallire il test, non introduce fallback runtime.

## CI e limiti delle evidenze

Ogni job `verify` esporta la fixture dopo la smoke SQLite e carica artifact
distinti per target. `cross-os-snapshots` dipende da `verify`, scarica l'artifact
del target opposto e invoca l'import test esplicitamente con `--ignored --exact`.
L'OS dichiarato deve essere quello atteso e diverso dal consumer; i risultati
riportano hash originale, entrambi gli OS e controlli effettuati. I test helper
sono ignorati nella suite ordinaria perché richiedono percorsi espliciti della
CI: non costituiscono test passati finché il job nativo non li esegue.

Il job ha timeout15minuti, matrice Linux/MSVC e Rust/azioni già fissati dal
progetto. La nuova pipeline mantiene tutti i gate precedenti. La lettura di
T-PERSIST-09 ha confermato la conclusione della pipeline precedente, quindi
il nuovo dispatch non ne cancella il lavoro in corso.

Esiti locali e nativi saranno registrati dopo l'esecuzione effettiva. Nessuna
CI Windows o portabilità bidirezionale viene dedotta dal roundtrip Linux.

Verifiche iniziali concluse: `cargo test -p lintra --lib --locked --offline
persistence::sqlite::tests -- --nocapture` ha prodotto23pass/4ignored;
`actionlint -shellcheck= .github/workflows/ci.yml` è terminato con exit0.
Actionlint1.7.7 è stato estratto in `/tmp` dopo confronto SHA256 dell'archivio
con il checksum della release (`023070a287cd8cccd71515fedc843f1985bf96c436b7effaecce67290e7e0757`).
Non è una dipendenza aggiunta al prodotto; ShellCheck non è stato eseguito.


## Gate locali finali

Tutti gli8comandi del [ledger](T-PERSIST-11-local-gates.json) hanno exit0;
370impronte sorgenti prima/dopo identiche, `CARGO_BUILD_JOBS=2`, modalità offline.

| Controllo | Esito effettivo |
|---|---|
| Formattazione | PASS |
| Clippy workspace/all-targets, default e all-features | PASS entrambi |
| Test workspace default |850pass,0fail,6ignored;109riepiloghi |
| Test workspace all-features |850pass,0fail,6ignored;109riepiloghi |
| Core senza bundle |807pass,0fail,6ignored;92riepiloghi |
| Grafo workspace/core | PASS |
| Tooling Python |23test pass |
| Workflow actionlint | PASS, ShellCheck non eseguito |

I conteggi aggregano anche i riepiloghi degli helper subprocessi, come nei
ledger precedenti; non sono conteggi di soli test top-level. I6ignored dei run
completi comprendono helper, probe e i2trasferimenti nativi espliciti. Non sono
successi impliciti. Gate remoto e file realmente scambiati restano da verificare;
al termine dei soli gate locali la task era IN_PROGRESS, non DONE.


## Gate nativo concluso — DONE

Codice633f61f, [CI41805357515](https://github.com/UraganoAngelina/lintra/actions/runs/41805357515)
completed/success: Linux/MSVC, tre confronti JSONL/pixel e due import incrociati
sono tutti success. Packaging UI opzionale skipped. [Evidenza](T-PERSIST-11-native-ci.json).

Scaricati4artifact snapshot: hash ricalcolato su entrambi i DB esportati, uguale
al manifest del rispettivo produttore e al source hash dell'import sul runner
opposto. Ogni consumer ha verificato3snapshot,2namespace, protezione e provenienza
conservata. Non è un roundtrip Linux spacciato per Windows. Nessun residuo11.
La successiva task10 cambia provider/CLI ma non è qualificata da questa CI11.
