# T-P16-05 — tabella Risultati da JSONL

## Preflight — 2026-10-28

Task READY indipendente da PERSIST04. T-P16-04 code311dcc6/357400e e
CI40589848698 qualificati; T-P5-04 JSONL DONE, golden canonico invariato.
Fonti: ARCH39/39D.6 criterio2/39D.7, DET99 con CNF11, piano4/14 e prompt05.
Il prototipo è soltanto autorità visiva e prova Slint: i suoi Vec/demo rows non
sono una sorgente dati di produzione. Il riferimento approvato resta intatto.

Scope previsto: `ui/src/results*`, wiring worker/controller/main, componenti
Slint e test UI. File JSONL atomico del worker come sorgente; solo offset delle
righe in memoria, decodifica lazy e cache limitata. Indicizzazione/filtri su
worker dedicato con cancellazione e proprietà esclusiva del risultato; nessuna
scansione di centinaia di migliaia di JSON sul thread UI. Ordine del report
preservato, filtri esatti e combinabili per codice/campo/ID record/CF/riga.

Selezione indipendente dai delegate visibili: il dettaglio resta leggibile
quando la riga selezionata esce dalla viewport. Contesto e metadati vengono dal
JSONL, nessun ID di flusso/campo hardcoded e nessuna norma duplicata nel client.
Errori di lettura/JSON sono tecnici, mai tabella vuota apparentemente valida.
Validità e completezza del riepilogo rimangono quelle del run, non del filtro.

Impeccable Operate/craft-floor, no-ai-slop, ui-ux-pro-max applicate entro il
mockup/font/palette esistenti, Hallmark adattato al desktop single-agent.
Il generatore design-system suggerisce un sistema web generico: esclusi font,
palette, claim e layout landing estranei. Context Impeccable non riconosce Slint
né docs/ui/DESIGN.md automaticamente; autorità recuperata direttamente da quel
documento, screenshot approvato e sorgenti. Nessun redesign o baseline riscritta.

Verifiche previste: JSONL reale del servizio e golden sintetici, filtri/malformed/
I/O/cache, selezione e dettaglio fuori viewport, rendering nei due temi/DPI e
minimo640×480, scorrimento300k/1M con offset reali. Gate workspace/core e CI
Linux/MSVC effettivi prima di DONE/M8. Export resta T-P16-06.

Stato: preflight; nessuna implementazione o gate di questa task dichiarato PASS.

## Implementazione e correttezza

`results/diagnostic.rs` descrive il wire JSONL canonico con DTO privati e
`deny_unknown_fields`. La conversione ricostruisce ValidationError e chiama
la validazione strutturale del dominio: nessuna seconda implementazione di
vincoli normativi. JSON non valido o diagnostica strutturalmente incoerente
sono errori tecnici redatti, non righe ignorate.

`ResultsFilter` applica cinque confronti esatti combinati con AND. Stringa vuota
rimuove il singolo vincolo; nessun trim/case folding degli identificativi.
`parse_line` ammette soltanto cifre ASCII e intero positivo rappresentabile:
zero, segni, spazi, decimali e overflow sono errori di input con istruzione utile.
Il filtro non modifica conteggi o completezza del riepilogo originale.

`ReportIndex::scan` usa BufReader e riusa un singolo buffer per riga. Conserva
solo Vec<u64> degli offset che soddisfano i filtri; l'ordine è quello del file.
Verifica LF finale, assenza CR fisici, diagnostica valida, flow/versione/scope,
conteggio totale e istogramma codici del summary, anche quando un filtro non
seleziona alcuna riga. Fallire una verifica scarta l'intero indice candidato.
Cancellazione osservata prima di ogni lettura; overflow e limite i32 della
UI sono espliciti. `offset_bytes` misura solo payload degli offset, non RSS.

`JsonlModel` implementa Model<ErrorRow>. `row_count` non legge JSON; `read_row`
fa seek all'offset e decodifica la sola riga richiesta. Cache FIFO con massimo
128 proiezioni: quando piena espelle la più vecchia prima dell'inserimento.
`position_of` cerca un offset per selezione con binary_search su indice ordinato.
`row_data` registra gli errori tecnici di lettura; il controller li segnala con
banner e non li spaccia per una diagnostica normativa. Proiezione per la UI
preserva posizioni one-based, field name, actual/raw_hex, expected e contesto
trace.field; conserva identità/episodio/technical IDs realmente presenti.
Nessuna inferenza dal nome del flusso o da ID campo.

`IndexJob` esegue scansione e filtri su un thread owned, con cancellazione e
canale terminale da un elemento. Non passa errori live o collezioni globali.
Un panic diventa WorkerUnavailable. Drop cancella e joina, nessun thread
sganciato. `ResultsController` vive soltanto sul thread Slint: Timer50ms verifica
il singolo risultato; nessuna scansione JSONL dentro i callback della finestra.
La chiusura ferma il timer e joina il job, poi rilascia i modelli e il report.
Il Timer cattura Weak, evitando un ciclo di ownership che trattenga il controller.

L'ultimo report completo del worker è condiviso con Arc<AnalysisReport>.
Indice, controller e modello ne mantengono l'ownership finché servono i file.
I reader sono rilasciati prima del TempDir anche su Windows. Nessuna nuova
persistenza delle righe o riapertura di un path utente nella tabella lazy.
L'Arc non cambia protocollo di validazione/pubblicazione: Cancelled/truncated
continuano a non pubblicare un report completo; start/errore/cancel non espongono
un vecchio report come risultato dell'ultimo tentativo.

`open` recupera summary/audit dell'analisi completata, non selezioni attualmente
modificate nel form. Riaprire lo stesso Arc conserva filtri/selezione. Un nuovo
report azzera il modello e avvia il nuovo indice. `apply` valida la riga e
pubblica soltanto il nuovo indice concluso; caricamento disabilita filtri e
selezione, il riepilogo resta quello del run. Un errore nella scansione cancella
le vecchie righe: non appaiono conformi ai nuovi criteri. `select` copia il
dettaglio e conserva l'offset indipendentemente dai delegate visibili; dopo un
filtro ritrova la selezione se presente, altrimenti la cancella esplicitamente.

`ResultsView` usa ListView virtualizzato, righe ad altezza costante e token
palette/font esistenti. Filtri accessibili riusano FieldEdit; Invio/Applica
avviano la ricerca. Puntatore e frecce/Home/End/PageUp/PageDown selezionano e
portano la riga in vista; scorrere senza cambiare selezione conserva il dettaglio.
A640px i pannelli si impilano nella page ScrollView; dettaglio scorrevole per
messaggi lunghi. Hover e selezione sono distinguibili nei due temi, focus
visibile. Stati loading, vuoto, filtro senza corrispondenze e I/O non sono fusi.
Export rimane alla task06, riferimenti approvati immutati.

## Prime verifiche e correzioni effettive

Cargo check positivo dopo fix ordinario di ownership del codice istogramma.
Sei integrazioni reader positive: golden/contesto/raw, cinque filtri, input riga,
malformed/BOM/UTF8/CRLF/troncamento, mismatch scope/count/istogramma e cancel;
cache e offset reali a300k/1M. Test Slint reale iniziale positivo: analisi1000
record, filtri/selezione e scrolling su indici disco300k/1M.

Prima capture ispezionata: tabella centrata implicitamente e sovrapposta al
pannello dettaglio. Corretto x/y esplicito e distribuzione verticale del
dettaglio, senza cambiare palette/font. I dati reali hanno codici E_* lunghi,
quindi la colonna codice180px sostituisce i90px dimostrativi del prototipo.
I tempi debug (~330–460ms a ridisegno completo) NON qualificano fluidità release.
La verifica ottimizzata usa il medesimo modello e i file300k/1M su entrambi gli
OS tramite workflow esplicito ui-results.yml, separato dalla CI ordinaria.
Nessuna soglia prodotto/DEC14 o nuovo riferimento approvato sono inventati.

Residui: gate dopo correzioni visive/tastiera, capture finali, regressioni,
misure release locali/native, commit/push e CI. Task IN_PROGRESS, M8 aperta.

Review ownership: anche il candidato ReportIndex trattiene Arc<AnalysisReport>
prima di essere inviato dal job, trasferendolo poi al modello. Se la finestra
chiude mentre un indice completo è ancora nel canale, reader/file vengono
rilasciati prima dell'ultimo TempDir owner: non si dipende dalla chiamata a
shutdown del consumer per l'ordine di cleanup Windows. Un test worker dedicato
verifica il modello lazy dopo drop del CompletedAnalysis e la rimozione finale.
Durante la costruzione di un nuovo indice, un errore cached del vecchio modello
non anticipa la fine del caricamento del candidato.

Primo gate release reale PASS:300k/1M con rotella30frame e salti fino alle righe
299990/999990; offset2,4/8MB, cache128,743decodifiche massime cumulative sulle
due scale. P95 locale rotella17,1–23,1ms1× e28,4–32,7ms2×, sotto altri carichi
di compilazione. Nessuna soglia prodotto inventata e nessuna garanzia desktop
universale dedotta. Capture chiare/scure ispezionate dopo fixx0: pannelli
non sovrapposti, dettaglio leggibile e palette/font forniti preservati.

Gli output release usano `ui-results-release-visual`, quelli debug
`ui-results-visual`: due qualificazioni sullo stesso workspace non possono
mescolare tempi/capture. Ripetere gate release sull'ultima revisione ownership
con output separati prima di archiviare evidenze finali.

## Gate locali finali — 2026-10-28

758/758test workspace default/all-features,95suite ciascuna,0failed/2ignored.
Fmt, Clippy workspace nei due assetti, grafo core senza GUI,21test Python e
build negativa reale del font tutti positivi. Clippy finale all-target UI dopo
l'ultimo assert del test positivo. Quattro RGB del form identici alla UI04
qualificataCI40589848698; confronto versionato nel dossierbootstrap-regression.

Release: gate `release-final-source`, exit0, sorgenti identici prima/dopo il
comando e hash registrati. Include assert sul modo locale esplicito nel
riepilogo, interazione puntatore/tastiera, selezione fuori viewport, filtri,
cleanup, rotella e salti su file300k/1M a1×/2×. Dati/tempi in local-scrolling.json;
offset_bytes è payload degli offset, non capacità allocata o RSS misurata.

Riepilogo/pannelli riposizionati all'altezza del mockup: intestazione56px,
azioni/riepilogo da72px e pannelli da124px; x24/840 e larghezze800/416 a1280px.
Il modo locale resta leggibile nel riepilogo; provenienza/ambito del run sono
nell'intestazione accessibile e in Informazioni. Codici simbolici lunghi,
field name, filtro applicabile e dettaglio con valori su righe separate sono
adattamenti funzionali dichiarati; il pannello dettaglio scorre, non tronca
contesto. Le nuove capture sono diagnostiche, non baseline autoapprovate.

File nuovi: results/{diagnostic,mod,controller}.rs, results.slint, due test
integration e workflow ui-results.yml; wiring in main/lib/worker/controller,
token/controlli, manifest/lock solo serde runtime già qualificata, documentazione.
Nessuna fonte ARCH/DET/normativa/golden o file utente delle skill modificato.

Stato locale completo, task IN_PROGRESS/WAITING_GATE fino a CI ordinaria e
scrolling release nativi Linux/MSVC. M8 e distribuzione UI rimangono aperte.
Next UI06 soltanto dopo gate05; il preflight backend05 ha ADR separata da
approvare, non una dipendenza nuova della UI.
