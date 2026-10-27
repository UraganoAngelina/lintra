# T-P15-07 — Test cross-OS e bundle

## Preflight e perimetro

Prerequisiti verificati: T-P15-03 CI40339655671 su c538493; T-P15-05 CI40349711927
su bbfdb93243e8f316940b73d9aec8920a625ce7ee e release40352109303 con asset
pubblicati immutabili verificati nella recovery. Contratti ARCH§39C.3/43,
DET Addendum A.4, piano§4/14/P15 consultati. Nessuna nuova dipendenza o modifica
runtime/normativa. Mandato autonomo del27ottobre2026 registrato in AGENTS.md.

## Codice e ragioni di correttezza

`tests/packaging_probe.py` esegue veri processi CLI nativi, senza contesti CI
simulati o hook runtime. `artifact` legge l’unico artifact= dal log xtask e
richiede un file esistente: la ricetta produce alberi puliti con nomi unici,
quindi un path target convenzionale potrebbe indicare un binario vecchio.
Il workflow usa bash/pipefail/tee su entrambi i runner: un errore di build resta
un fallimento e il log conserva il path del binario realmente qualificato.

`produce` copia quel binario in una directory temporanea con spazi e usa una cwd
diversa; rimuove soltanto le due variabili di override del bundle dal processo
figlio. Prima verifica fallback embedded con nessuna directory esterna; poi
estrae accanto al binario il ZIP generato dal bundler Rust già qualificato.
Nessun --schema-dir/--reference-data-dir: la sorgente esterna può provenire solo
dalla risoluzione accanto all’eseguibile. Controlla audit external/manifest e
identità dei JSONL rispetto al fallback. Input sintetico include CRLF, identità
UTF-8 accentata, valore invalido e record corti/lunghi; i JSONL restano byte grezzi.

Aggiorna poi solo la copia temporanea di uno schema sintetico, relativo digest
manifest e versione del bundle: il valore2 prima invalido diventa valido con
zero errori. Questa differenza di comportamento dimostra la precedenza esterna,
oltre al controllo dell’audit. Corrompere lo schema esterno deve produrre exit1
senza report: vietato ripiegare sullo snapshot incorporato. L’hash del binario
prima/dopo assicura che aggiornare i dati non richieda ricompilazione.

`jsonl` verifica UTF-8 senza BOM, terminatori LF e oggetti JSON per riga.
`compare` richiede esattamente tre file su ciascun OS, confronta i byte senza
normalizzazioni e rifiuta scenari diagnostici vuoti. I due artifact native sono
caricati solo dopo gate/probe riusciti; il job dipendente compare-jsonl li scarica
separatamente e confronta direttamente i risultati dei runner. Nessuna equivalenza
presunta dalla sola uguaglianza con un golden. Actions riusate agli SHA già
qualificati15-05; timeout/retention e permessi read espliciti.

`scripts/tests/test_packaging_probe.py` verifica positivi e negativi: selezione
artifact assente/duplicata/file mancante, inventario incompleto/extra, differenza
byte, diagnostiche tutte vuote, BOM/CRLF/UTF-8 invalido/non oggetti JSON.
Non sono mock dell’engine: il percorso reale è verificato dal producer nativo.

## Gate locali e limiti

- Python16/16 positivi; actionlint1.7.12 positivo con shellcheck/pyflakes esterni
  non disponibili; git diff --check positivo.
- cargo fmt --all -- --check e Clippy default/all-features --all-targets
  --offline --locked -- -D warnings positivi.
- cargo test --offline --locked:681 pass,79suite,0fail,2ignored.
- cargo test --all-features --offline --locked:689 pass,79suite,0fail,2ignored.
- Probe reale Linux positivo sul binario scaricato della release0.1.1-rc.1 e
  ZIP dati pubblicato synthetic-data-release-v1, già verificati nella recovery;
  confronto locale degli stessi artifact positivo, NON evidenza cross-OS.
- Tentativo bundler locale via Cargo fallito per rete ristretta e goblin assente
  nella cache corrente; nessun fallimento considerato successo. La prova usa
  invece il ZIP già prodotto dal bundler Rust nella CI dati qualificata.
- CI nativa nuova richiesta per il producer Linux/MSVC e confronto effettivo.
  Nessun DONE o M5 prima dei gate reali. Rust runtime/lock/baseline invariati.

## Handoff

Stato IN_PROGRESS; gate locali completati, pubblicazione/CI da completare.
File: workflow CI, producer tests/packaging_probe.py, test comparator Python,
AGENTS mandato, indice/status e presente dossier. Prossima candidata P16-00,
che richiede prototipo e PNG1×/2× da verificare prima di ogni implementazione;
P17-01 indipendente ha prerequisito13-08 già verificato. Prosecuzione automatica
quando i prerequisiti reali sono soddisfatti, nessuna conferma ordinaria.
