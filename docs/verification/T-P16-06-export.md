# T-P16-06 — export e Informazioni

Stato: implementazione e test mirati positivi; regressioni complete/CI pendenti.
Prerequisiti16-05 e12-04 DONE, PERSIST07 chiusa prima di avviare questa task.
Ordine autore:06→07→08UI. Nessuna nuova dipendenza, migrazione, norma CAD/SIOC,
FROZEN, palette o font modificati. Diagnostici dettagliati concordati rimangono
aperti nel perimetro successivo prima della chiusura UI: non sono dichiarati fatti.

## Writer e replay core

`replay_canonical_report` legge un solo record JSONL alla volta. Il decoder è
quello già qualificato di XlsxReportWriter, estratto in una funzione crate-private
senza cambiare wire o conversione. CSV/JSONL passano rispettivamente per CsvSink e
JsonlSink: nessuna quotatura, colonna o norma reimplementata nella UI. Il replay
controlla summary, stato nonCancelled/nontroncato, LF, dominio dei diagnostici,
flow/versione/scope, conteggio e istogramma. Un errore interrompe il replay prima
della pubblicazione; non si saltano righe malformate. La destinazione rimane .part
fino al successo. Memoria O(massima riga + codici dello schema), non O(tutti errori).

L'XLSX usa esattamente XlsxReportWriter::write, come la CLI12-04. Il test usa lo
stesso spool e summary reali e verifica **byte identici**, anche senza errori.
JSONL torna byte-identico allo spool canonico; CSV coincide con il sink core.
Ogni formato riceve anche `<file>.summary.json` attraverso il writer comune con
BundleAudit del run. L'XLSX mantiene il proprio foglio Riepilogo come nella CLI.

## Ownership e input

AnalysisReport conserva il path canonico del file validato: modificarlo nel form
successivamente non cambia la protezione del vecchio run. Non viene persistito
nel database o nei log. Il worker resta Trace/LocalOnly come prima; un altro scope
non viene trasformato implicitamente in un file a caso.

ResultsController conserva report, summary e preferenze insieme quando apre il
risultato. `displayed_analysis` restituisce una copia dei soli metadati e un Arc
al report effettivamente visualizzato. Export e Informazioni usano questo stesso
snapshot, non l'ultimo worker o il form corrente. Un test esegue davvero un run
successivo valido e verifica che l'export della vista precedente contenga ancora
la sua diagnostica. I filtri non riducono gli errori esportati.

ExportFormat seleziona solo il writer; le preferenze del run determinano il
formato iniziale, modificabile esplicitamente prima di salvare. Le label indicano
scope, modalità effettiva e verifiche parziali quando pertinenti. Nessun export
trasforma CompletedInvalid in validità o LocalOnly in conformità completa.

## Destinazione e pubblicazione

`preflight` richiede un path assoluto con estensione coerente e parent esistente,
rifiuta directory/symlink come file, protegge input e spool privato, considera
anche la destinazione del riepilogo. Se un file esiste, nessuna sovrascrittura
senza conferma visibile; cambiare percorso/formato revoca quella conferma.

`export_report` crea due AtomicReportFile usando create_new per i rispettivi
.part. Scrive tutto privatamente, verifica completamento/cancellazione e ripete
il preflight prima di pubblicare. Un errore abortisce esplicitamente le guardie
create e conserva eventuali errori di cleanup; non elimina .part preesistenti di
altri scrittori. Un test crea realmente quel conflitto sul secondo output.

La pubblicazione è atomica **per file**, non fra i due rename. ExportFailure
conserva causa, file già pubblicati e cleanup falliti: se il secondo rename fallisce
il messaggio non sostiene che nessun file esista o che l'export sia completo.
Non si aggiunge una garanzia contro mutazioni ostili concorrenti dei path: resta
il contratto di AtomicReportFile (directory controllata dal chiamante).

CancelReader/CancelWriter verificano il token durante lettura e scrittura,
anche per l'XLSX. Restituiscono ConnectionAborted: usare Interrupted farebbe
riprovare indefinitamente read_until/write_all. Il test verifica l'errore terminale
e il ritorno delle due API. Prima dei rename l'annullamento abortisce gli output;
dopo l'ultimo checkpoint si completa la pubblicazione, senza promettere undo.
La causa tecnica rimane disponibile anche se è arrivato un annullamento.

## Thread e interfaccia

ExportJob possiede un thread joinable, token e canale terminale a una posizione;
l'Arc mantiene vivo lo spool mentre si salva. Errori/spawn/panic/disconnessione
non sono un risultato riuscito. Drop/shutdown cancellano e attendono il thread.
Nessun demone, backlog di errori o formattazione del workbook nel thread UI.

ExportController tocca componenti/Rc solo nel thread Slint. Un Timer40ms è attivo
soltanto durante un salvataggio; il canale non blocca. Gli handler catturano debolmente
il controller. Aprire il pannello congela il run; salvataggio, conferma e annullamento
non cambiano la validazione. Esito e file salvati sono mostrati dopo la receipt reale.
La terminazione dell'event loop chiude prima export, poi reader risultati e worker.

Il pannello usa token/font/controlli esistenti, tre scelte XLSX/JSONL/CSV,
destinazione esplicita, riepilogo indicato, conferma di sostituzione quando necessaria.
Niente librerie native, portali o helper esterni. Tema chiaro approvato e nero/
arancione invariati. Informazioni usa la callback nativa immediata, versioni schema/
bundle della vista, o selezione corrente nel form; AboutSlint è il widget reale
SDK1.18.1 con attribuzione/versione, non un logo o testo sostitutivo.

## Test e limiti della verifica

Quattro test export con vero ValidationService/worker: formati e byte XLSX/core,
provenienza bundle, run validi/invalidi, input protetto, overwrite, path errati,
cancel, spool danneggiato, conteggi/scope/framing, .part estraneo conservato.
Un test unit dimostra che gli adapter cancellati non fanno retry infiniti.
Un test Slint esercita callback e timer reali, filtri, overwrite, Info/versioni,
run visualizzato distinto dal successivo, rendering1×/2× entrambi temi e640×480.
PNG diagnostiche in docs/ui/exports, **non** nuovi riferimenti autoapprovati.

Viste realmente le capture export chiaro1×/scuro2× e Informazioni chiaro1×;
restante verifica con quelle degli altri assetti e gate nativi ancora da chiudere.
Il primo test Info leggeva prima del callback deferred: la UI usa ora un'azione
esplicita toggle-information che aggiorna subito i metadati prima del rendering.
Una fixture successiva002 era fuori dominio (lo schema sintetico ammette solo001):
corretta la fixture alla fonte, senza alterare il dominio per far passare il test.
Una compilazione del test terminata143 senza diagnostico Rust è stata ripetuta;
la successiva exit0 è verificata. Nessun pass o causa OOM inventati.

Impeccable context/operate/craft-floor applicati, detector[] sulle estensioni Slint
non costituisce prova di rendering web o nativo. ui-ux-pro-max interrogato senza
sostituire il mondo già ratificato con il suo suggerimento web; no-ai-slop applicato
ai testi e verificato con eval. Single-agent, nessun Superpowers o subagente.

## Correzione della regressione input

La regressione completa ha rilevato che la prima canonicalizzazione anticipava
l'errore tecnico del servizio quando l'input era rimosso dopo il preflight.
Il path di protezione usa ora canonicalizzazione oppure path assoluto lessicale,
senza sostituire la successiva lettura/diagnostica autorevole di ValidationService.
Il test worker precedente resta invariato; non è stato cambiato per accettare una
classificazione sbagliata. CI40895114689 su ff600d3 intermedia, richiesta cancellazione
per qualificare la correzione. Nessun DONE06 su un gate pendente o fallito.

Correzione verificata: worker5test, export4, finestra nativa1 eunit2 positivi,
Clippy workspace default/all-features pass. Viste anche Informazioni scuro2× ed
export chiaro2×; layout/colori/font preservati, nessun riferimento autoaggiornato.
Gate completi corretti ancora in esecuzione, CI finale da avviare sul fix.

### Correzione nativa Windows — gate40897488097

Il job123530502473 ha individuato un difetto reale nel confronto dei percorsi
protetti: canonicalize della destinazione Windows introduce il prefisso extended
length, mentre il percorso TempDir dello spool non lo possedeva. Il test dello
spool (exports.rs) ha correttamente rifiutato il falso successo dell'esportazione.
La protezione dell'input era già canonica; la root privata va canonicalizzata
anch'essa prima del confronto. Nessuna modifica al test di accettazione per
mascherare il difetto. Gate06 resta aperto fino alla correzione e nuova CI.

### Seconda diagnosi nativa —40900252019

La protezione dello spool passa Windows. Il nuovo gate ha poi trovato un limite
del vecchio harness dei processi SQLite: assumeva che ogni contesa producesse
solo StorageBusy/CAS prima di COMMIT. Con un lettore concorrente il COMMIT può
invece fallire mentre la transazione resta attiva; il prodotto conserva già
correttamente CommitOutcomeUnknown. Fonte primaria: SQLite Transaction §2.3,
https://www.sqlite.org/lang_transaction.html.

La correzione è nel solo test: chiudere il tentativo e risolvere il candidato
immutabile da una nuova connessione. Exit0 richiede Published con revisione
esatta, exit3 richiede NotPublished o errore precommit già definito; ogni
conflitto o errore diverso fallisce. Ripetizioni limitate alle letture occupate,
nessun retry di staging/COMMIT. L'assertion finale resta un vincitore, un perdente
e una sola generazione pubblicata. Nuovo caso deterministico con SHARED reader
riproduce il COMMIT incerto e verifica entrambi gli esiti della risoluzione.
Nessuna modifica dell'adapter, della durabilità, di CAS o del contratto ambiguo.
