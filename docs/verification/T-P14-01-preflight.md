# T-P14-01 — formato regionale XLSX non disponibile

2026-10-25. Prerequisiti T-P3-01/G0 soddisfatti; nuova prosecuzione autorizzata.
ARCH§13 e DET§143 richiedono tooling XLSX→RawFlowSchema con note originali,
seguito da lint/revisione. Piano§9/P14 indica esplicitamente come blocco il
«formato XLSX regionale reale (esterno)».

La ricerca rg --files --hidden --no-ignore nell'intero workspace, estensioni
.xls/.xlsx/.ods, non restituisce workbook. Nessun contenuto sanitario reale è
stato letto. Sono presenti JSON normativi, PDF e specifiche, che non definiscono
il formato tabellare da importare. Nessuna assunzione su header, fogli, celle
unite, righe o mapping OBB/FAC/NBB complesso viene introdotta.

BLOCCO: T-P14-01, formato sorgente esterno mancante. Serve il percorso del workbook
regionale/schema tabellare esplicito per costruire una fixture sintetica fedele.
Non è un conflitto architetturale e non richiede di riaprire decisioni ratificate.
Nessun importer finto o parser JSON dentro un XLSX è implementato. T-P14-02 resta
dipendente dall'importer; T-P14-05 dipende anche da DEC-18. Non dichiarare M7.

Richiesto all'autore il percorso tramite domanda asincrona. Prosegue T-P14-04,
indipendente: richiede solo T-P14-03 già verificata (da80163, CI39574640180 Linux/
MSVC success). Non modifica ordine/dipendenze del piano: esegue una task pronta
mentre l'attività interessata è ferma, secondo protocollo§14.2.

## Aggiornamento successivo dell'autore — 2026-10-25

Fornito il percorso schemas/xlsx/SIOC_tracciato_regionale_20260420-definitivo_v05.xlsx.
Consultazione esplicitamente autorizzata; gli altri documenti della cartella
riguardano flussi futuri e non sono scope di14-01. Il precedente inventario
era corretto al momento della ricerca; ora la sorgente esiste. Non sono stati
sostituiti gli schemi approvati. Il blocco di disponibilità è superato; esame
formato/mapping dopo il gate locale14-04, senza anticipare implementazione.

## Esame effettivo del workbook SIOC ricevuto

File912412 byte, SHA-256 esatto
6be8ae60a2c236e5a25b731365b42f8e7bca849066d68747abfc78b92ce273e3.
Lettura locale ZIP/XML stdlib, sola consultazione normativa; gli altri file
forniti dall'autore non sono stati analizzati o inclusi nei commit.

Fogli: Intestazione_SIOC, Tracciato_SIOC definitivo, Indicazioni operative.
Il foglio tracciato ha header riga5: A=N, B=Chiave, C=Descrizione Campo,
D=LUNGH., E=POS. INIZ., F=TIPO, G=OBB, H=Istruzioni per la codifica.
Sezioni1/2/3/4.1/4.2 nelle righe7/28/81/119/142. La sezione corrente è
necessaria: i field4.x appartengono a4.1 oppure4.2 secondo intestazione, non
si può dedurre il trace dal solo prefisso del field. L'ID4.19.1 è una stringa
valida, non un numero decimale o una coppia di soli interi.

135 campi unici:18/50/36/20/11 nei cinque trace. Confrontati con
structural-minimal.1: tutti gli ID presenti, nessun campo mancante/extra, tutte
le posizioni e lunghezze identiche. Le note di8 campi(1.11,1.15,2.11,2.30,3.29,
3.30,3.32,3.34) differiscono solo per whitespace: verificato con confronto di
testo normalizzato esclusivamente per classificare il delta, non per modificare
il contenuto importato. L'importer dovrà preservare la stringa originale, incluse
newline e spazi, distinguendola dalla precedente trascrizione approvata. Nessuna
nota è stata tradotta automaticamente in DSL o resa nuova regola eseguibile.

F contiene AN/N/Data/D/A e un vuoto in4.5. Sono già documentati nelle fonti
ratificate: docs/normative/sioc/presence-matrix.md e4.1-fields.json indicano
4.5 source_type=null→numeric;4.6/4.10 sono alphanumeric nel profilo approvato.
Non introdurre Alphabetic o nuove interpretazioni da A; eventuali mapping del
profilo importer devono essere espliciti e mantenere il token originale per audit.
Le formule Excel sono presenti in POS. INIZ.; una lettura del valore cached non
equivale a ricalcolo verificato. Specificare e testare la gestione di formule e
valori assenti/incoerenti prima dell'implementazione; niente esecuzione di macro.

Il foglio Indicazioni operative conferma allineamento numerico/alfanumerico e
la regola monetaria Decimal già ratificata. Non riclassificare codici di
compartecipazione o altri campi per nome. Encoding/PK/join/presenza condizionale
restano contratti approvati: non possono essere inferiti dalle sole colonne.

Disponibilità XLSX risolta; nessun nuovo conflitto normativo sostanziale rilevato
su layout o note. Implementazione14-01 non avviata mentre14-04 attende il gate.
Sorgente utente preservata non staged. Alla ripresa chiudere14-04, poi definire
profilo/mapping esplicito e fixture sintetica fedele a questo layout entro14-01;
non implementare capacità di flussi futuri o modificare documenti protetti.

## Preflight implementativo — mapping esplicito, ripresa2026-10-25

L'importer deve completare RawFlowSchema, ma il workbook non specifica da solo
encoding, missing per eccezioni, DSL condizionale, PK/identity/join, persistenza
e capability. Queste decisioni restano nel blueprint dichiarativo revisionato
fornito al costruttore; non vengono dedotte dal testo H. Il workbook deve
fornire realmente ID/nomi/coordinate/note, con mapping di celle esplicito e
verificato; non si restituisce il blueprint senza leggere il file. Nessun flow
o field ID hardcoded nell'implementazione generica. I profili sono dati.

Lettura di un unico snapshot byte del file, hash SHA-256 degli stessi byte usati
per ZIP/XML e provenienza in source. Le righe/formule e i token originali devono
restare tracciabili. Nessuna macro o formula Excel eseguita: policy tecnica
esplicita di lettura del valore numerico cached per le coordinate, distinta da
verifica del ricalcolo; rifiutare cache mancante, non numerica o non intera.
Valori stringa/codici/note non convertiti da float, nessun trim delle note.

Ogni binding collega una cella/row a un field del blueprint nello stesso trace;
richiedere copertura completa e univoca, rifiutare ID inattesi/duplicati e
header/sezioni non corrispondenti. Le decisioni semantiche del blueprint devono
essere esplicite: non sostituire NBB con optional o A con Alphabetic. Eventuali
nuovi campi/tipi richiedono il corrispondente mapping revisionato, non default
normativi inventati. Il risultato raw deve attraversare lint/revisione/compiler
come ogni schema importato, senza promessa di validità automatica.

DEC-15 candidato prima dell'eventuale modifica Cargo: calamine=0.36.1 già
qualificata come dev-only in12-05, promozione allo scope tooling produttivo
schema/importer richiesta da14-01 e coerente con ARCH§39C.1. MIT/pure-Rust,
MSRV1.88 già verificati; nessuna nuova versione, default-features=false, nessun
uso di calamine nel validation loop. Verificare lock/features effettive dopo
promozione; nessuna modifica alle dipendenze di scrittura XLSX.

Solo preflight durante l'attesa del gate14-04: non ancora implementazione14-01.

Verifica aggiuntiva sul file originale:130 celle formula, tutte numeriche con
cache presente; nessuna vbaProject.bin. La qualificazione runtime deve comunque
rifiutare cache mancanti/non intere e formule in celle testuali. L'assenza di
macro in questo file non autorizza esecuzione di macro in altri workbook.
Il subtree calamine0.36.1 letto dalla versione installata resta pure-Rust,
comprende quick-xml0.41.0 e zip8.6.0 con deflate/zlib-rs; nessun backend di
sistema aggiunto. calamine è ancora dev-only finché14-01 non viene avviata.
