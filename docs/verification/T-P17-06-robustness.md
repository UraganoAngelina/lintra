# T-P17-06 — Robustezza e proprietà generative

## Preflight

Prerequisito17-01 DONE, CI40476599897 su485c98f Linux/MSVC e benchmark smoke
positivi; due run10k/100k/1M già archiviate. DET112/117/118 e piano4/14/P17
consultati. Task indipendente dal gate UI16-00 in attesa; sviluppo single-agent
sequenziale. Nessuna nuova dipendenza, policy normativa, API o crate di fuzzing.

## Test e ragioni di correttezza

`tests/robustness.rs` usa un generatore64bit con stato iniziale fissato e bit
mescolati: stessi casi riproducibili sui runner x64, senza dipendere dall’OS.
Non è generazione crittografica né prova esaustiva di ogni input possibile.

- ByteRange:20.000 coppie fra limiti piccoli e word64complete più estremi;
  oracolo u128 indipendente dall’addizione checked del codice verificato.
  Verifica overflow/lunghezza0, bounds, contenuto e puntatore della slice,
  così una copia o una posizione fuori dal record non passano come estrazione.
- Newline:512sequenze generate,8payload ciascuna con byte arbitrari esclusi
  solo CR/LF, righe interne anche vuote, terminatori LF/CRLF mescolati, ultimo
  terminatore presente/assente e capacità BufReader1..7. I payload attesi sono
  costruiti prima del framing: non si copia l’algoritmo di parsing come oracolo.
- Read fault: errore reale di Read a ogni offset di input multi-record,
  capacità1/2/7 e quattro ErrorKind. Confronta righe complete, RecordRead e
  causa originaria; vieta falsoEOF, riga parziale pubblicata e ripresa dopo
  PreviousFailure. Il conteggio atteso deriva dai terminatori già trasferiti.
- JoinKey:1.280tuple, stessi byte concatenati divisi in5modi e tutti256valori
  del primo byte, inclusiNUL/255/separatori/spazi. Il set deve avere1.280elementi,
  quindi una concatenazione senza confini non può passare. Verifica anche record
  corti e indice usize::MAX restituiti come errori. È proprietà dell’encoding
  delle chiavi, non certificazione lessicale/normativa dei payload binari.
- Canonical:512roundtrip su quattro profili sintetici con metadatiUnicode,
  controlli/escape/zeri iniziali e numero decimale oltre precisione floating point.
  Confronta modello, byte canonici idempotenti e fingerprint, senza float.
- JSON esterno malformato: ogni prefisso troncato di uno schema, sostituzioni
  NUL/UTF-8 invalido ogni7byte e nesting512: deve restituire Err senza panic
  o modello parziale. Nessun catch_unwind che nasconde crash.
- File grande:250.000record realmente scritti su filesystem, letti in streaming
  con buffer64byte; verifica contenuto, line numbers, puntatore/capacità invariati,
  senza Vec globale dei record. Il test non pretende di misurare RSS.
- Sink: guasto BrokenPipe a ognuno dei64callback on_error nel servizio vero;
  nessun callback successivo, summary o RunCompleted. Include controllo positivo,
  guasto on_summary e file input realmente mancante classificato Input, non
  CompletedInvalid. max_errors restaNone e DEC-16 non viene anticipata.

L’API applicativa ammette solo InputSource::Path: il guasto di lettura a metà
stream è iniettato al confine RecordReader/Read, senza inventare un input reader
applicativo simulato o modificare la request. Il servizio viene verificato con
I/O preflight reale mancante e fault sink; non si dichiara un’iniezione midstream
applicativa non eseguita. Test precedenti sulle cause di errore restano attivi.

## Gate e stato

Prime7prove mirate positive. Aggiunta l’ottava prova JSON e rafforzato il
mixing del generatore; corretti seed esadecimale/range usize segnalati daClippy.
Fmt/Clippy default/all-features positivi dopo correzione, regressione completa
in esecuzione. Nessun codice produttivo modificato perché non è emerso un bug
nel corpus verificato. Nessun DONE prima della CI nativa Linux/MSVC.

## Gate locali completati

Fmt e Clippy --all-targets default/all-features --offline --locked -D warnings
positivi. cargo test --offline --locked689pass e --all-features697pass,80suite,
0fail/2ignored. Otto nuovi test tutti positivi, inclusi schemi malformati.
Evidenza T-P17-06-local-gates.json. Nessuna modifica produttiva o nuova
dipendenza. CI nativa ancora da avviare; stato IN_PROGRESS, nessun DONE anticipato.

## Chiusura nativa

CI40486090209 su a332ca5466c3e62ba2ed483f1154d7d33eaa30d1: Linux/MSVC e confronto JSONL completed/success. Evidenza native-ci archiviata. T-P17-06 DONE; nessun residuo o nuova semantica.
