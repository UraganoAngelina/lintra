# T-P17-01 — Harness di benchmark sintetico

## Preflight

Prerequisito T-P13-08 DONE su52762c8525b878a72c7a59f9589c207a17e1efc6,
CI40192491629 Linux/MSVC positiva, golden reale/memoria attiva registrati.
DET§114 e piano§4/14/P17 consultati: misure10k/100k/1M, valid/error-heavy,
projection; nessuna soglia arbitraria. Indipendente dal gate15-07 in corso.
Nessun crate aggiunto, contratto runtime o baseline modificati.

## Implementazione e correttezza

Cargo dichiara `[[bench]] validation`, harness=false. `benches/validation.rs`
è un eseguibile di misura opt-in: la normale suite non avvia automaticamente
milioni di record; --output PATH obbligatorio per misurare, --smoke limita a100.
OpenOptions/create_new riserva un output nuovo prima di lavoro costoso, senza
sovrascrivere evidenze precedenti. Un errore lascia un output incompleto e un
exit nonzero: non costituisce una misura valida.

Schema riusa la regola cross-trace sintetica qualificata nei test Full; allarga
PK/KEY a8byte, DATA resta2byte, lunghezza18. `records` scrive in streaming chiavi
uniche riproducibili; i conteggi sono il TOTALE di due trace, ciascuna conN/2righe.
In valid-heavy entrambe hanno DATA=OK; in error-heavy solo S ha NO, producendo
esattamenteN/2 violazioni cross-trace. Il benchmark controlla records_processed,
numero errori sia nel sink sia nel summary, stato finale e Complete: misure di
un run sbagliato o parziale falliscono invece di diventare throughput.

`CountSink` conta senza conservare errori: misura validazione, non JSONL/XLSX/IO
report. Il percorso misurato Full include service/preflight/compilazione e due
passate; generazione fixture e costruzione della factory sono fuori dal timer.
`projection` compila lo stesso schema fuori dal timer, chiama davvero
project_record per entrambe le trace, verifica i tre campi selezionati e consuma
il risultato con black_box prima di rilasciarlo. Non simula un ContextStore e
non confonde il costo isolato della projection con il costo totale della run.

`Meter` inoltra alloc/alloc_zeroed/dealloc/realloc a System preservando contratti
pointer/layout; aggiorna solo contatori atomici, senza allocare ricorsivamente.
Solo allocazioni riuscite modificano il contatore; realloc mantiene il vecchio
conteggio quando fallisce. `measure` registra baseline prima della singola
misura sincrona e il massimo dei byte heap richiesti oltre quella baseline.
Questo è **heap Rust richiesto, non RSS**, non include overhead dell’allocatore,
stack, mapping o page cache. Gli atomici influenzano i tempi: confrontare solo
misure ottenute con lo stesso harness/toolchain. Nessun dato inventato per RSS.

Output JSON include schema formato benchmark, flag synthetic, versione/commit
compiler, hash SHA256 del sorgente harness e helper schema, toolchain OS/arch,
flag optimized e metrica memoria esplicita. I tempi variano fra esecuzioni;
fixture/scenari/metriche e condizioni sono riproducibili, non i tempi identici.
Non stabilisce soglie prestazionali o idoneità a carichi regionali reali.

## Riproduzione

Dalla root, con output nuovo ogni volta:

```sh
cargo bench --bench validation --locked -- --output target/benchmark-run1.json
cargo bench --bench validation --locked -- --output target/benchmark-run2.json
cargo test --bench validation --locked -- --output target/benchmark-smoke.json --smoke
```

La CI esegue soltanto lo smoke su entrambi gli OS, verificando i conteggi reali;
la regressione completa e il gate15-07 restano attivi. Le misure estese locali
ottimizzate vengono archiviate separatamente con il presente dossier. Il comando
--smoke usa100record e NON soddisfa da solo l’accettazione10k/100k/1M.

## Stato e gate

IN_PROGRESS: Clippy iniziale default positivo, build ottimizzata/smoke in corso.
Misure complete, ripetizione, no-clobber negativo e gate finali ancora da eseguire.
Nessun DONE anticipato. File Cargo.toml, benches/validation.rs, smoke CI,
indice/status e dossier/evidenze. Nessun backend o runtime nuovo.

## Misure ottimizzate locali

Due esecuzioni complete9scenari ciascuna archiviate in
[T-P17-01-benchmark-run1.json](T-P17-01-benchmark-run1.json) e
[T-P17-01-benchmark-run2.json](T-P17-01-benchmark-run2.json).
Entrambe optimized=true, hash harness corrispondente al sorgente e inventario
completo10k/100k/1M × valid-heavy/error-heavy/projection verificati.
Il compiler_commit0b152aa identifica il prodotto al momento della build; la
revisione del benchmark è identificata separatamente dall’hash harness
(perché ancora in sviluppo nella stessa working tree). Nessun dato regionale.

Smoke ottimizzato100record positivo su tutti3scenari; tentativo negativo con
output esistente rifiutato, digest del file preservato. Prima build fallita
per formato LowerHex non implementato dal digest sha2 corrente; conversione
esadecimale byte per byte corretta e build/smoke/runs ripetuti con successo.
Nessuna misura del tentativo fallito è usata.

## Gate locali conclusi

Fmt, Clippy --all-targets default/all-features --offline --locked -D warnings
positivi; cargo test --offline --locked681pass/79suite/0fail/2ignored.
Smoke cargo test --bench validation (profilo test) positivo: stesso comando della
CI. Actionlint1.7.12 e diff check positivi. Gate strutturati in
[T-P17-01-local-gates.json](T-P17-01-local-gates.json).
Stato resta IN_PROGRESS fino alla CI nativa. Non viene avviata una seconda
pipeline sullo stesso branch mentre il gate corretto15-07 è attivo, perché la
concurrency del workflow cancellerebbe la precedente. Pubblicazione autorizzata,
dispatch successivo appena concluso15-07. Nessun blocco umano nuovo17-01.
