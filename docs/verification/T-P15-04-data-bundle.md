# T-P15-04 — Bundle dei dati

## Preflight e piano —2026-10-26

Prerequisito unico15-03 verificato: commitc538493, CI40339655671 Linux/MSVC
completed/success letti14:38:47 UTC. Prompt atomico, ARCH§39C.2, manifest
authoring e ratifica CNF-BUNDLE-AUDIT-01 A consultati. Nessuna pubblicazione
richiesta da questa task; DEC-08/09/11 dei workflow release restano aperte.

Perimetro: comando xtask sequenziale che confeziona un runtime bundle esplicito
con manifest, payload dichiarati verificati e SHA256SUMS; archivio deterministico
schemas-<bundle_version>.zip. Nessuna conversione flow_schema.json, scansione
arbitraria o dataset ufficiale inventato. Fixture positiva: snapshot sintetico
esistente; test negativi integrità/path/symlink/output e prova del bundle
estratto con eseguibile già compilato. Formattazione, lint/test prodotto e
tooling, CI Linux/MSVC secondo protocollo, senza dichiarare gate anticipati.

Stato IN_PROGRESS, sviluppo single-agent autorizzato;92 DONE, M5 aperta.

## Implementazione e ragioni di correttezza

- `Options` del solo xtask richiede input e output-dir espliciti e univoci;
  nessuna ricerca o fallback. L’opzione è separata dal prodotto CLI.
- `directory` rifiuta root vuote/inesistenti/non directory/symlink e restituisce
  la root canonica. Entrambi i livelli runtime schemas/reference-data devono
  esistere. Output fuori dalla sorgente protegge gli input e non li modifica.
- `file` percorre solo i componenti già validati dal parser condiviso, richiede
  directory intermedie/file finale regolari e rifiuta symlink. Nessuna scansione
  arbitraria, deduzione di identità o conversione authoring. Non promette
  protezione da sostituzioni ostili concorrenti.
- `build` acquisisce MANIFEST.json con rifiuto dei duplicati/percorsi pericolosi
  del contratto esistente. Mantiene byte e versione originali, usa il nome
  schemas-<version>.zip solo se sicuro come filename. Controlla ogni payload
  con BundleManifest::verify_bytes prima di scriverlo: un solo file in memoria
  per volta, nessuna lettura di directory/dataset non dichiarati.
- ZIP Stored senza codec esterni, directory esplicite, ordine stabile BTreeMap,
  data1980-01-01 e permessi fissi eliminano mtime/umask/clock delle sorgenti.
  SHA256SUMS UTF-8/LF contiene tutti i payload e i byte originali del manifest,
  in ordine per path. Non include sé stesso, evitando un’autoreferenza impossibile.
- NamedTempFile nel filesystem di destinazione conserva staging RAII. Si
  completa lo ZIP, si esegue sync_all, poi persist_noclobber rifiuta sempre un
  target esistente. Il file definitivo contiene uno ZIP completato; non si
  promettono atomicità universale della rimozione del nome temporaneo o
  resistenza ai crash su ogni filesystem (limiti documentati da tempfile).
  I normali errori di input eliminano lo staging senza pubblicare l’archivio.
- CLI scrive il risultato solo dopo la pubblicazione e propaga gli errori stdout.
  Nessun cambiamento di engine, grammar, service o policy Strict/Continue.
- Il test di accettazione usa una versione bundle diversa dallo snapshot,
  estrae in una cartella con spazi e invoca il binario esistente da altra cwd
  con root esplicite. Verifica audit external/versione nuova/present, JSONL
  vuoto ed hash del binario invariato: aggiornamento dati senza ricompilazione.
  Ignorato esplicitamente nel run tooling ordinario se non richiesto; CI usa
  --include-ignored e LINTRA_TEST_BIN esplicito, quindi lo esegue davvero.

Le dipendenze sono versioni già qualificate, riusate nel lock separato xtask;
root Cargo.lock e contratti runtime invariati. Non è una release pubblicata.

## Evidenze locali e handoff al gate nativo

Tutti i comandi richiesti exit0: fmt prodotto/tooling; Clippy all-targets prodotto
(default e all-features) e tooling -D warnings; test prodotto default681 e
all-features689,79suite,0fail,2ignored già previsti; tooling15pass/0fail/0ignored
con --include-ignored e LINTRA_TEST_BIN assoluto. Log in target/verification-p15-04,
cache di compilazione riusata da verification-p15-03. Il primo avvio del test
manuale con path relativo al repository non trovava il binario dalla cwd xtask:
corretto l’ambiente di esecuzione con path assoluto; CI lo imposta esplicitamente.

Controllo lock: iniziale risoluzione offline aveva selezionato encoding_rs0.8.42
e relative nuove dipendenze; allineata con cargo update --precise0.8.35 alla
versione prodotto già qualificata, eliminando tutte le aggiunte involontarie.
Verificato che ogni coppia crate/versione del lock finale appartenga al lock
prodotto o al precedente lock xtask. Nessuna nuova versione non qualificata.

Comando reale cargo xtask bundle-data positivo sullo snapshot sintetico. Verifica
indipendente Python zipfile/hashlib: SHA256SUMS corretto per ogni file, archivio
SHA256773026a67f84cc8e4c6499794eac4241caf90ae77f84ef5b5e2603a5032baa3b.
Seconda invocazione reale stesso output: exit1, messaggio already exists,
archivio byte-identico e nessuno staging residuo. Test zip byte-identico anche
fra due destinazioni differenti. Nessun artefatto locale pubblicato come release.

Stato DONE_UNVERIFIED, CI Linux/MSVC ancora da avviare sulla consegna completa.
T-P15-05 resta soggetta al proprio preflight DEC-08/09/11; nessun prerequisito o
milestone M5 dichiarato soddisfatto.92/140 task DONE,15-04 ancora aperta.
