# Paging, offset e filtri risultati

T-MIG-P2-05,14 novembre2026. Design sul [wire ratificato](CONTRACT.md),
[run/lifetime](LIFECYCLE.md) e controllo/progresso separati. Reference privata
su JSONL in [paging_reference.py](paging_reference.py); nessun bridge esposto.

## Limiti e selezione dei dati

Pagina default64righe, massimo128; il massimo riusa CACHE_ROWS128 del
JsonlModel Slint esistente. Cache client e registro token backend al massimo
due pagine, quindi256righe proiettate, più la singola selezione eventualmente
pinnata daP2-06. Non è un limite alle diagnostiche prodotte, indicizzate in Rust
o esportate su disco. Nessun endpoint “all”, dataset/JSONL/indice completo o
record sanitario raw attraversa IPC. Non sono consentiti loop automatici di
prefetch che ricostruiscano un trasferimento bulk nascosto nella UI.

Tetto messaggio1MiB in entrambe le direzioni, envelope compreso. Il producer
valida la dimensione reale dopo encoding/escaping prima di pubblicare pagina,
handle o generazione: se eccede, errore esplicito dell’intera risposta. Non
elimina righe o produce JSON tagliato per far passare il controllo.

## Query immutabile e indici privati

Il broker consegna al provider Rust un report completato posseduto e la sua
summary catturata, con schema/versione, scope e fingerprint. Non viene aperto
un path del renderer. ReportIndex::scan esistente costruisce un indice di byte
offset sul file già aperto, controlla ogni riga contro lo scope e confronta
conteggi totali/per codice con la summary. L’ordine è quello del JSONL canonico,
non un sort JS per riga, codice o testo. Malformed/truncated/scope/count mismatch
è errore tecnico, mai una pagina vuota o una diagnostica normativa inventata.

Una nuova query costruisce il candidato su worker Rust, fuori dall’event loop,
poi pubblica atomicamente l’handle se l’intenzione è ancora corrente. Errore,
cancel o candidato tardivo non fanno apparire applicato un nuovo filtro. La
vecchia query può restare consultabile solo con il proprio identificatore e
stato esplicito, non sotto i filtri appena falliti. La reference esegue il
flusso in modo sincrono per testare il contratto, senza qualificare i thread.

Il `query_ref` è un handle opaco, nativo e legato a owner, report/RunRef,
filtri, ordine e snapshot immutabile dell’indice. Non è un offset fisico o
un’autorizzazione ottenuta decodificando un numero. I nonce OS di query e righe
non sono chiavi HMAC o token C1; nessuna nuova crittografia applicativa.
Un handle estraneo/stale non consente di leggere un altro report.

## Filtri e ordinali

Filter DTO schemaVersion1: code, field, record, fiscal_code, line e trace_ref
opzionali. Vuoto nei filtri testuali esistenti significa nessuna restrizione;
altrimenti AND esatto, case-sensitive, senza trim, casefold o coercizione di
identificatori sanitari. field confronta FieldId, record sioc_record_id,
fiscal_code il dato catturato da Rust; niente dati di filtro nei log.
Line riusa la grammatica di ResultsFilter::parse_line: testo ASCII numerico
positivo u64, vuoto/None senza filtro, zeri iniziali accettati e risolti come
numero di riga. Nel DTO di risposta i contatori sono decimali canonici.
trace_ref è un riferimento al trace catturato nello scope, non un path.

`offset` è l’ordinale logico0-based della query filtrata, u64 testuale canonico;
non è il byte offset JSONL. Il backend conserva il vettore fisico e legge solo
le righe richieste. Filtri e scope non mutano mentre si percorrono le pagine.
Offset uguale a total_matches restituisce pagina vuota/has_more false; oltre
il totale, zero/>128/bool/string per limit e contatori malformati sono errori
espliciti. Nessun clamp che faccia sembrare completo un risultato diverso.

## Shape della pagina e selezione

Page DTO schemaVersion1: query_ref, RunRef versionato, page_generation u64
canonico, offset, rows, next_offset, has_more, total_matches e total_diagnostics.
Total_matches riguarda il filtro; total_diagnostics l’intero report verificato.
Una pagina vuota non dice CompletedValid. Il prossimo offset avanza del numero
reale di righe restituite, con ordine stabile e senza duplicati tra pagine.

Ogni Row DTO schemaVersion1 contiene diagnostic_ref opaco, ordinal della query,
line, trace_ref e preview di code/field-name. Niente actual, raw_hex, context,
record_identity o record grezzo nelle righe. I dettagli richiesti separatamente
sonoP2-06; il file canonico/export non viene sanificato o riscritto.

Le preview di metadati sono DTO schemaVersion1/text/complete, massimo256byte
UTF-8 di testo su un confine valido di carattere. Complete false è esplicito:
un prefisso non si spaccia per il codice/label completo né viene riutilizzato
come filtro esatto. I valori completi e le fonti restano Rust e possono essere
richiesti nel dettaglio previsto; niente HTML/URL attivi da label ostili.

Registro diagnostic_ref limitato alle due pagine trattenute. Cambio query o
espulsione di pagina rende i suoi token stale; una selezione già autorizzata
può trattenere il solo snapshot privato previstoP2-06. Una nuova richiesta con
un token vecchio non riapre il file arbitrariamente. Le risposte contengono
query_ref/page_generation: il client scarta risposte di vecchia query o pagina.
Riavvio/owner diverso non eredita l’autorità del token; non è un capability
basato sulla sola prevedibilità del nome. Errori di mint/collisioni falliscono
prima di pubblicare uno stato parziale.

## Integrazione e verifiche

Il provider riusa Diagnostic::into_error/ValidationError::validate e i servizi
Rust reali; la reference Python non riproduce la semantica normativa. Scansiona
il formato sintetico canonico per verificare paging, LF, scope e conteggi.
I limiti Slint i32 non diventano automaticamente limiti di validazione del
nuovo adapter; overflow/memoria/indice sono errori tecnici, senza troncare dati.

70casi PASS/exit0 su file privati sintetici: confini righe/offset, default/tail,
filtri esatti/AND e padding riga compatibile, token/owner, due pagine, sostituzione
query, LF/scope/count mismatch anche in coda, report vuoto, nonce/collisioni e
metadati UTF-8/control lunghi con messaggio completo entro1MiB. La query non
modifica i byte JSONL. Nessuna UI, dataset clinico, benchmark o CI nativa usata.
Le race reali, pin e intake del bridge restano ai task/gate applicabili.

## Emendamento 24 novembre 2026 — T-MIG-P6-01 (decisione dell’autore)

Su richiesta esplicita dell’autore, per la tabella dei risultati con la
motivazione di ogni diagnostica, il Row DTO aggiunge tre chiavi obbligatorie:
`message` (preview), `expected` e `actual` (preview o null). `actual` è il
solo valore visualizzabile (`display`) del dato trovato; restano esclusi
`raw_hex`, `context`, `record_identity` e il record grezzo. Le tre preview
hanno un limite di **128 byte** UTF-8 (`complete` false se tagliate), più
stretto dei 256 dei metadati: una pagina di 128 righe con ogni testo al
massimo dell’escaping JSON resta entro 1 MiB invece di fallire per intero.
Il valore trovato può essere un dato personale del file analizzato: compare
solo nelle righe disegnate (griglia virtualizzata), mai in log, console o
storage, e non cambia la regola di cancellazione delle pagine (due al massimo).

Nuova operazione `results_overview` {schemaVersion, run}, legata al run
pubblicato (altrimenti StaleContext/InvalidState come `results_query`):
`records_processed`, `valid_records`, `invalid_records`, `total_errors` dal
summary verificato, `by_code` [{code preview, count}] nell’ordine del summary
e `by_trace` [{trace_ref, trace preview, count}] per ogni tracciato dello
scope, zero inclusi, nell’ordine dello scope. I conteggi per tracciato
vengono dalla stessa scansione verificata dell’indice di pubblicazione;
la loro somma e quella per codice coincidono con `total_errors`, altrimenti
errore tecnico. Il renderer non ricava conteggi dalle righe né decodifica
`trace_ref`: il nome del tracciato arriva da questa risposta.

Reference e verifica aggiornate (`paging_reference.py`, `verify_paging.py`):
72 casi PASS, di cui 2 nuovi sulla motivazione (assenza di raw/context/
identità, preview esplicite entro 128 byte nel caso peggiore).
