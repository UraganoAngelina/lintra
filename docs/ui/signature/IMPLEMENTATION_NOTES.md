# Spiegazione tecnica del refactoring Signature

## Token e controlli

`Tok` resta l'unica global Slint di design. Ogni colore dipende da `dark-mode`;
nessun codice Rust deve reinterpretare colori o regole. I font già incorporati
non cambiano. `on-accent` separa il colore del testo del pulsante pieno dal
colore della superficie: bianco sul rame chiaro, grafite sul rame scuro.

`ActionButton` mantiene `activated`, `enabled`, focus e accessibilità. Aggiunge
solo `primary` e `navigation`: il primo seleziona una superficie piena, il secondo
il trattamento della barra di prodotto. L'ordine delle condizioni evita che
uno stato disabilitato sembri un'azione primaria disponibile. Il bordo2px indica
il focus senza cambiare le dimensioni del controllo; nessuna animazione sposta
il contenuto. `button-touch` sostituisce l'ID `pointer` per non oscurare l'enum
nativo `mouse-cursor: pointer`. Puntatore, Space/Return e azione accessibile
invocano la stessa callback solo se enabled. Il segno della checkbox usa lo
stesso `on-accent`, per mantenere contrasto nei due temi.

## Barra di prodotto e schermate

`MainWindow` conserva tutte le proprietà e callback pubbliche. Analisi e
Risultati sono destinazioni stabili: Risultati richiede `results-available`,
assenza di busy e assenza di export aperto. Analisi cambia soltanto la vista;
non azzera configurazione o report. Aprire Risultati disattiva Informazioni
prima della callback esistente. Nessun nuovo accesso al servizio.

Il marchio è un Path geometrico, senza font/icona di sistema. Sotto760px si
omette il nome, mantenendo le destinazioni e i controlli; sotto900px le etichette
di navigazione sono compatte. Il contesto esteso compare solo con spazio
sufficiente e conserva un'etichetta accessibile completa. Informazioni usa lo
stesso widget ufficiale AboutSlint, dimensionato e dentro un'area scorrevole.
Versione, schema e bundle continuano a essere aggiornati dal controller export.

## Configurazione ed esecuzione

`AnalysisForm` riceve `schema-label` già derivata dal controller; la mostra senza
inferire flusso o regole. La colonna di configurazione usa60% dello spazio
residuo; sotto il breakpoint1100px i pannelli si impilano nello stesso Flickable.
La sequenza dei selettori è ora flusso/versione/tracciato, coerente con gli
azzeramenti già presenti nel controller Rust. I binding e le condizioni enabled
rimangono quelli esistenti. Nessuna selezione predefinita nuova.

Il pannello di esecuzione scorre separatamente dall'azione ancorata in fondo:
un messaggio lungo non copre Avvia/Annulla. Colore e testo usano gli stati
esistenti configuration-error, validation-invalid ed execution-partial. Non si
aggiungono percentuali, soglie o interpretazioni di Unknown. La callback Avvia
è ammessa soltanto con ready e worker-available; durante busy invoca Annulla.
Le preferenze di formato restano intenzioni: il salvataggio richiede l'export
esplicito del run, con consenso overwrite invariato.

## Risultati

`workspace-height` è il massimo fra480px e lo spazio disponibile dopo titolo,
margini e note. In una finestra alta si vedono più righe, senza caricarle tutte.
La tabella mantiene ListView, binding di scroll e modello JSONL lazy; nessun Vec
di diagnostiche nuovo nell'applicazione. Il solo Vec del test è una fixture.
Il dettaglio scorre indipendentemente e resta legato a `selected` del controller.

Il ScrollView orizzontale della tabella conserva una larghezza minima704px:
sotto il breakpoint i pannelli sono impilati ma le colonne non diventano
illeggibili. Le etichette dei filtri persistono quando il campo è compilato.
Applica/Invio invocano la stessa ricerca esatta esistente. Il testo «righe
visibili» conta il modello filtrato, non pretende di essere il totale del run.

Il marcatore di selezione è fissato esplicitamente a x0; x/y espliciti per titolo
e pannelli evitano il centraggio implicito di Slint. Le frecce/Home/End/Page
mantengono move-selection e portano la riga nella viewport. La causa completa
precede i metadati nel dettaglio; actual/expected, posizione, dominio e contesto
restano dati del report/controller, senza riscrivere diagnostici o regole.

## Export e prove

`ExportPanel` limita la larghezza di lettura a920px. Pulsante pieno e selettori
usano gli stessi componenti, mentre save/close/choose-format, consenso e timer
rimangono invariati. Le prove reali verificano che i filtri non riducano il
report esportato e che un nuovo run non ritargetti quello visualizzato.

L'harness Signature renderizza gli stessi componenti di produzione prima/dopo
con clock fermo, otto viste, due temi e cinque combinazioni dimensione/DPI.
I dati sono esplicitamente di presentazione. Bootstrap e results_window
mantengono eventi reali puntatore/tastiera: sono aggiornate soltanto le
coordinate cambiate e il passo riga36px. Il test ruota dentro la lista, non
sopra i nuovi filtri. CI confronta i pixel nativi Linux/MSVC byte per byte;
i riferimenti storici non vengono sovrascritti.
