# T-P16-03 — configurazione Nuova analisi

## Perimetro e prerequisiti

Bootstrap16-02: code9832d85/fix216e662, CI40522626495, quattro job success,
font reali e RGB1×/2× identici Linux/MSVC. Catalogo14-05: code8a9ffd8,
CI40298536590, DEC-18 ratificata. Nessuna autorizzazione normativa nuova.
Il riferimento chiaro Risultati già approvato non viene sostituito.
Questa task prepara un intento LocalOnly su un singolo tracciato; worker,
risultati ed export appartengono rispettivamente a16-04 e task successive.

## API e ragioni di correttezza

- src/bundle_resolution.rs estrae la precedente implementazione CLI senza
  cambiarne priorità, errori o coppia di directory. CLI paths.rs resta uno shim;
  tutti i test CLI esistenti continuano a chiamare le stesse funzioni. Il modulo
  adapter-support è solo std: il core non importa Slint o application/UI.
  BundleDirectories conserva le directory esplicite, BundleEnvironment consente
  test senza mutare l'ambiente, BundleSelection espone radici o origine embedded.
  La coppia incompleta è sempre errore, mai fallback o scoperta da cwd/home.
- external_bundle_manifest_path è il precedente corpo CLI, condiviso con UI:
  discovery implicita solo per schemas/reference-data sorelle, MANIFEST assente
  opzionale; presente/esplicito errato o symlink è errore. Getter BundleAudit
  espongono origine/versione/stato reali senza rendere mutabili gli audit.
- AnalysisBundle acquisisce il catalogo reale da repository filesystem o bundle
  incorporato, conserva repository/preparer/audit associati. Un manifest presente
  viene rispettato anche durante l'acquisizione degli schemi. load_raw per ogni
  entry impedisce che metadata corrotti vengano mostrati come dati disponibili;
  acquisizione non equivale a certificazione o eseguibilità. Nessun filtro per
  nome SIOC/CAD o rimozione di capability per abilitare artificialmente uno schema.
  Flussi e versioni usano ordine catalogo/UTF8, tracciati ordine dichiarato.
- AnalysisSelection non imposta default di flusso/versione/tracciato/encoding.
  Cambiare antenato azzera i discendenti; prepare verifica appartenenza al
  catalogo e richiede al vero SchemaRegistry/SchemaCompiler schema e hash.
  Costruisce ValidationRequest Trace/LocalOnly con StrictSkipRecord,
  AutoLfOrCrlf, max_errors None, e ValidationService/preparer reali con policy
  ContinueWithAvailableEvidence. Non invoca validate e non produce report.
  ReportSelection conserva soltanto i tre formati scelti, anche tutti falsi.
- InputPreview controlla file regolare prima e dopo open, applica il preflight
  BOM Reject del core, legge al massimo65536byte con buffer4096. Conta LF,
  CRLF e CR isolati anche fra chunk; CR finale conta solo se EOF osservato.
  Un campione terminato sul limite non attesta EOF né il totale dei record.
  File vuoto è lecito; directory/BOM/errori IO restano errori tecnici. Non decodifica,
  normalizza, riscrive o conserva righe e non controlla valori clinici/regionali.
- ConfigurationController collega i callback con Weak<MainWindow>, evitando
  cicli fra finestra e stato. refresh azzera sempre l'intento precedente, poi
  pubblica soltanto una configurazione realmente preparata. Digitare un nuovo
  percorso invalida immediatamente anche la descrizione del campione precedente;
  Invio o selezione browser confermano il percorso. Errore bundle è visibile,
  senza provider fittizi. Nessun thread di validazione è avviato in16-03.
- browse_directory elenca solo metadata, directory prima dei file e ordine nomi
  deterministico. Percorso esplicito richiesto, nomi nonUTF8 o oggetti non regolari
  visibili ma non selezionabili. Il browser si apre solo per azione operatore;
  nessuna lettura delle cartelle personali nei test. Enter/apri, parent e cancel
  hanno callback reali; file aperto dal probe solo dopo selezione esplicita.
- main risolve la stessa sorgente della CLI relativamente all'eseguibile/ambiente,
  poi installa lo stato della finestra. Default embedded del membro UI non cambia
  il default del package CLI. CI aggiunge un test core senza default features:
  l'unificazione delle feature workspace non deve nascondere regressioni CLI.

## Controlli e riferimento visivo

controls.slint raccoglie button/radio, input, combobox e checkbox. Gli stessi
callback servono puntatore, tastiera e azione accessibile; gli stati disabilitati
impediscono modifiche. Focus visibile e palette derivati dai token esistenti.
analysis.slint riproduce geometria Nuova analisi del PNG fornito1280×720:
config(24,80,604,420), exec(652,80,604,516), padding/gap24. Sotto1100px le card
si impilano in un viewport scorrevole. Origin completa nel corpo, header con
elisione e nome accessibile. File/encoding/versione sono espliciti, UTF8 è l'unica
capacità encoding esposta dal motore corrente. Il PNG illustrava Windows1252 e
una run62%: non si copiano capacità, conteggi o progressi fittizi. Header usa il
token del prototipo Slint#16232E, come bootstrap approvato, anziché#14202B del PNG.

Skills no-ai-slop/Impeccable/Hallmark applicate nel mondo visivo fornito e in-thread,
nessuna delega. Capture native, non browser/sloplint HTML, verificano Slint.
Nuove capture sono diagnostiche; nessuna baseline approvata viene riscritta.

## Test e gate

analysis_configuration.rs: otto test con filesystem temporaneo, due versioni
reali SYNTHETIC, request e hash compilati, schema mancante/selection invalida,
BOM UTF8/16/32, newline misti/CR/empty/directory, campione70KB limitato64KiB,
corruzione/incompletezza bundle senza fallback, browser e nomi nonUTF8 Unix.
bootstrap.rs: piattaforma Slint/software reale, puntatore/Space/Tab/resize,
AboutSlint, quattro capture temi/DPI e contrasti; aggiunge callback controller
con file sintetico CRLF, azzeramento versione/tracciato, invalidazione preview,
formati e selezione browser. Nessun worker o output report. Capture di confronto
1280×720 con ancore geometriche delle card in entrambi i temi.

Gate locali/CI da completare e registrare nelle evidenze dedicate prima di DONE.
Nessun nuovo crate runtime, dato paziente, backend storico o norma introdotto.
Next16-04 dopo gate03; PERSIST-03 indipendente già pronta. AUTO_CONTINUE.
