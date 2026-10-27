# T-P16-04 — worker di validazione e ponte Slint

## Prerequisiti e scope

UI03 DONE/0a2121b/CI40577920052: Linux/MSVC, JSONL e4RGB identici, anche alle
capture locali. T7-02/bacedba/CI39569282614 qualificata, contratti token/progress
preservati e ripetuti nelle regressioni native correnti. Fonti DET98–99/B.2,
ARCH39D.7 eCNF-11: canali live per contatori, dettaglio errori dal JSONL.

Il lavoro chiama l'API pubblica ValidationService sul thread dedicato; nessuna
regola normativa o interpretazione di campi/codici CAD/SIOC nella UI. Ambito
Trace/LocalOnly già dichiarato da03. Risultati paginati e export rimangono05/06.
No nuova modalità storica, database, schema modificato o capacità simulata.

## Worker e streaming

ui/src/worker.rs contiene:

- WorkerError con cause typed Spawn/TemporaryOutput/Validation/Report/Cleanup,
  Panicked/Disconnected. Display statico e redatto, errori tecnici separati da
  ValidationError. Cleanup conserva l'errore originario e quello della pulizia.
- WorkerProgress conserva fase, trace e conteggio cumulativo del trace corrente,
  più contatore cross. Non somma tracciati diversi, stima un totale o inventa una
  percentuale dalla dimensione del file/campione preliminare.
- ChannelObserver aggiorna un solo snapshot di metadati e invia in sync_channel(1).
  I contatori sono throttled50ms; i boundary possono tentare subito. try_send
  pieno non blocca il servizio e non accumula backlog; la summary terminale
  ristabilisce i conteggi esatti. Ogni slot inviato pubblica un solo wakeup Slint.
- Un secondo sync_channel(1) è dedicato all'unico risultato finale, mai scartato
  per progresso in coda. Nessun ValidationError viaggia nei canali e nessun Vec
  globale di errori viene creato. Notifiche massime pendenti limitate dai due slot.
- execute usa un nuovo TempDir, nomi statici errors.jsonl/.part e AtomicReportFile
  del core. JsonlSink<BufWriter> riceve gli errori direttamente da validate.
  validate efinish devono riuscire; su errore abort è esplicito e preserva cause
  di cleanup. complete riceve status/truncated veri: Cancelled e truncated non
  pubblicano uno spool completo. Nessun export XLSX/CSV dichiarato prematuramente.
- AnalysisReport mantiene l'ownership del TempDir per il consumer; il JSONL è
  sempre interno anche quando il checkbox export JSONL è spento, perché serve
  alla tabella futura. BundleAudit resta quello selezionato, summary/outcome
  sono quelli effettivi del servizio. Non si riscrivono input o output precedenti.
- WorkerHandle possiede thread, token e receiver. start usa Builder con nome,
  contiene gli unwind come errore tecnico senza successo fittizio. try_result
  consegna una volta sola; wait è esplicitamente bloccante per test/shutdown.
  Drop cancella ejoin; nessun thread staccato o daemon. Il main fa shutdown DOPO
  window.run, così non aspetta la fine della validazione nel loop grafico attivo.

Tempfile3.27.0 default-features=false passa da dev a runtime solo nell'adapter UI,
registrato prima della modifica manifest. Stesso crate/versione/checksum già
qualificato; nessuna nuova libreria di sistema o dipendenza GUI nel core.

## Ponte UI e correttezza degli stati

worker_controller.rs è Rc/RefCell confinato al thread Slint. start clona l'intento
preparato (solo metadata/path/service Arc, non righe), disabilita la configurazione,
avvia il worker e registra una Weak<MainWindow>. La callback Send/Sync si limita
a upgrade_in_event_loop e invoke_worker_poll: nessun accesso Rc/widget dal worker.
La capacità bounded dei canali limita anche i wakeup pendenti.

poll legge un progresso e l'eventuale terminale, non drena una coda illimitata.
RunCompleted dell'observer non termina la UI prima della pubblicazione atomica:
solo WorkerResult lo fa. Cancel imposta il token, mostra «richiesto» e attende
l'outcome; una cancellazione tardiva non riscrive un Completed già fissato dal core.

Normative invalidity ha il proprio flag/testo, Err tecnico un altro, Cancelled è
controllato e completezza parziale deriva da summary.completeness. Non si interpreta
Unknown come validità conclusiva. last_outcome è assente su Err; last_error conserva
la causa typed. last_completed nomina esplicitamente l'ultimo report completo,
conservato durante tentativi falliti/cancellati, senza attribuirgli l'esito corrente.
La preview viene identificata come iniziale e, su errore, si richiede un nuovo
controllo del percorso: un vecchio «nessun BOM» non descrive un input cambiato.

Slint riusa posizione/layout/token/font dei pannelli. Avvia diventa Annulla con
i token rossi, focus/tastiera del controllo esistente. Form bloccato, tema eInfo
ancora raggiungibili. Niente barra/percentuali prive di un totale vero. Gli stati
parziali hanno token warn e testo, errori/violazioni token error e testi distinti.

## Skills e collaudo

no-ai-slop per copy aderente agli esiti; Impeccable Operate/craft-floor per
responsive/stati/focus; ui-ux-pro-max --design-system e ricercaUX progress/error
recovery eseguite. Palette/fonts/layout forniti prevalgono sui suggerimenti
landing/nuovi font. Hallmark adattato al renderer nativo, nessun falso auditHTML.

worker.rs integration: servizio reale con input valido/invalidità normativa,
JSONL streaming eTempDir owned cleanup, pre-cancel reale senza report completo,
file rimosso/BOM cambiato DOPO configurazione => Err tecnico. Clock iniettato
registra l'ID del vero thread per provare che non è il caller. Nessun consumer
progresso attivo, quindi completamento dimostra l'assenza di deadlock da backlog.
Unit100kupdate dimostra single slot/throttling con Instant controllato, zero errori
accumulati. Quattro integration+unit passati prima della qualifica finale.

worker_window.rs usa la piattaforma Slint/software e il vero upgrade_in_event_loop
tramite proxy headless. Callback vengono eseguite sul thread UI. Una barriera nel
proxy di test sospende il worker al RunStarted (nessun hook nel codice prodotto):
UI continua a renderizzare/cambiare tema/cancellare, quindi il rilascio produce
Cancelled reale. Testa validità, violazioni distinte da Err/BOM, report precedente
conservato, cancellazione richiesta distinta dall'esito e cleanup a shutdown.
Screenshot1280×720 di running chiaro/scuro, errore e cancellazione sono render
reali diagnostici, non una nuova baseline approvata né percentuali dimostrative.
Il test usa un input relativo sintetico e ripristina la cwd del solo processo.

Gate completi locali in esecuzione. Non DONE04 prima della nuovaCI Linux/MSVC;
PERSIST03/P18-01 attendono indipendentementeCI40581933176/5bc8402.
AUTO_CONTINUE; next16-05 solo dopo gate04, nessuna delega/backgrounddaemon.

## Gate locali finali

741test/92suite sia workspace default cheall-features,0fail/2ignored; fmt/Clippy
workspace default/all-features positivi. Guard graph efont-negative actualbuild
pass. Dopo rifinitura report-intent:16test UI eClippy/fmt finali pass; ulteriore
revisione solo copy con6test mirati pass e nuove capture reali. Actionlint1.7.12
workflow positivo, upload screenshot worker aggiunto all’evidenza CI.

Formati richiesti conservati in CompletedAnalysis.requested_reports, senza
promettere export: caption visibile «Export non disponibile». Il JSONL interno
resta la fonte della tabella05; writer/export06 non sono anticipati. Messaggi
utente senza thread/worker jargon; cause typed restano nelle API tecniche.
Cinque capture diagnostiche in docs/ui/worker, hash ecomandi nel JSON gate.
Bootstrap4RGB continua a testare la finestra/configurazione isolata; le nuove
capture worker e testheadless provano il ponte collegato. Non si dichiara identità
crossOS di screenshot con tempi reali; i4RGB statici hanno il loro gate separato.

PERSIST03 e18-01 sono oraDONE/CI40581933176 su5bc8402 tutti4job, inclPS5.1reale.
Residuo04: commit/push/CI nuova, nonDONE/M8. Nessun conflitto contrattuale.
Next05 dopo gate04; PERSIST04 pronta indipendente perpreflight. AUTO_CONTINUE.

### Formattazione dopo la revisione copy

L'ultimo fmt --check aveva mostrato due wrapping non conformi, ma la sequenza
shell non fail-fast aveva proseguito con i6test (positivi). Corretto con cargo fmt
prima di push/CI, quindi fmt --check ripetuto in set-e e passato. Sole interruzioni
di riga Rust, nessun cambiamento semantico o test allargato; hash sorgenti aggiornati.
Il gate non è stato aggirato e la prima CI riceverà il codice già formattato.

## Chiusura nativa

DONE:311dcc6+fmt357400e, CI40589848698 su357400e completed/success,
Linux/MSVC/compareJSONL/compareRGB tutti pass. Artifact scaricati e verificati:
4RGBstatici identici fraOS eallecapturelocali,5PNGworker realmente presenti in
entrambi gli artifact di capture; i tempi dinamici non sono confrontati comegolden.
Nessun residuo04, nessun M8anticipato. JSONnative-ci/native-artifacts conservano
SHA/source, job/step ehash. LetturaAPI riuscita dopo unico retry autorizzato
per timeoutdel revisore (non una decisione di azione insicura). Nessun gate aggirato.
NextUI05 ora pronta; PERSIST04 indipendente corrente in verifica locale.
