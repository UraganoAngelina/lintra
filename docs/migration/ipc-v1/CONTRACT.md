# Proposta envelope IPC v1

T-MIG-P2-02,14 novembre2026. **PROPOSED_NOT_RATIFIED**:
`MIG-IPC-WIRE-V1-2026-11-14`. [Parametri macchina](proposal.json),
[codec di riferimento](reference_codec.py), [vettori sintetici](verify_contract.py).
La proposta non cambia i due comandi dello spike e non implementa il bridge.
Le firme restano non definitive fino aG-MIG-2; P3 non viene anticipata.

## Scelta proposta

JSON UTF-8 strict, trasportato come byte raw nelle due direzioni; versione1
esplicita in ogni messaggio. Massimo **1.048.576byte (1MiB)** inclusivo per
richiesta o risposta intera, contenitori annidati al massimo32livelli, root
inclusa. Sono limiti tecnici di trasporto proposti, non budget di validazione,
soglie DEC-14 o limiti ai record, all’audit o ai report su disco.

Un solo tetto simmetrico mantiene semplice l’envelope; le pagine DTO avranno
anche i propri limiti di righe/campi inP2-05/06.1MiB è una scelta conservativa
di protocollo da approvare e collaudare, non un numero ricavato da benchmark
né una garanzia di throughput.32livelli permettono oggetti DTO strutturati
ma rifiutano input profondi prima del parser completo. Nessun troncamento o
clamp silenzioso: messaggio fuori limite rifiutato, dataset Rust invariato.

## Forma e correlazione

Richiesta, con esattamente questi campi:

```json
{"wire_version":1,"request_id":"fixture_1","kind":"request","operation":"synthetic_probe","payload":{}}
```

Risultato di comando:

```json
{"wire_version":1,"request_id":"fixture_1","kind":"result","payload":{}}
```

Errore tecnico/protocollo:

```json
{"wire_version":1,"request_id":"fixture_1","kind":"error","error":{"code":"IPC_INVALID_MESSAGE"}}
```

`synthetic_probe` è un vettore di prova, non un endpoint applicativo esistente
o autorizzato. `payload` è un oggetto DTO: l’envelope ne controlla grammatica,
byte, profondità e scalari, **non** ne concede l’uso. Il dispatcher dovrà
controllare registry dell’operazione, DTO specifico esatto, origine nativa,
contesto e stato prima dell’I/O. Niente handler generico shell/fs/SQL/approval.
Le shape dei payload dei job, pagine, dettaglio, export e Full restano nelle
task P2-03..11; nessun `serde_json::Value` libero diventa autorità applicativa.

`request_id` contiene1..64caratteri ASCII fra lettere, cifre, `_` e `-`.
Identifica la singola richiesta nel suo contesto finestra/istanza, non è un
run ID, un token di autorizzazione o una garanzia di idempotenza. Duplicati
in-flight, identità job/run e messaggi tardivi si specificano inP2-03.
`operation` contiene1..64caratteri ASCII, inizia con lettera minuscola e
prosegue con lettere minuscole, cifre o `_`; essere sintatticamente valido
non significa essere registrato. `plugin:shell|execute` non è un nome valido.

Il risultato richiede correlazione. Nell’errore `request_id` può essere null
quando il confine ha rifiutato il messaggio prima di poter verificare l’ID;
non si riflettono stringhe arbitrarie del chiamante. Root ed errore rifiutano
campi extra. I payload specifici dovranno fare altrettanto, inclusi gli oggetti
annidati. Eventi/progresso non sono risposte a comando: il loro contratto èP2-04,
con decoder e identità propri; non vengono accettati dal codec request/response.

## Parsing, numeri e compatibilità

1. Verifica nativa di finestra/origine/permission e stato di ingresso applicabile.
2. Richiedere il body raw; verificarne la lunghezza prima del decode completo.
3. UTF-8 strict, niente BOM, profondità massima prima del parser JSON completo.
4. JSON completo unico, chiavi duplicate rifiutate a ogni livello, niente NaN/
   Infinity, surrogate isolate o numeri float nel wire.
5. `wire_version` è un intero1..65535, mai boolean/stringa/float. Una versione
   ben formata diversa da1 produce IPC_UNSUPPORTED_VERSION; valori malformati
   o fuori intervallo producono IPC_INVALID_MESSAGE.
6. Verificare forma, correlazione, scalari e poi DTO/registry/stato specifici.

I numeri JSON ammessi sono interi entro ±9.007.199.254.740.991. Contatori u64,
identificatori numerici ampi e decimali esatti si trasportano come testo
canonico nei DTO specifici, senza passare da Number JS. Il codec preserva la
stringa `18446744073709551615`, ma la sua validità come campo u64 è una verifica
del DTO dedicato: una stringa arbitraria non acquista tipo dal contenuto.
Questo non cambia i numeri/formati dei report canonici esistenti.

Nessun fallback silenzioso alla shape vecchia. Una modifica incompatibile a
campi, significati o codici richiede nuova versione e relativi test. Frontend
e backend sono nel medesimo artefatto offline; un mismatch è un errore chiaro,
non un motivo per saltare validazioni. Il protocollo1 sintetico dello spike,
formati report e formato storico/HMAC rimangono contratti distinti.

Il producer deve serializzare una risposta attraverso un writer con limite,
prima di inviarla, senza pubblicare JSON parziale. Oversize restituisce l’errore
bounded e non trasforma il job in fallito/cancellato: stato e risultato restano
Rust. Terminali minimi e paging devono renderli consultabili secondoP2-03..06.
Nessun retry automatico di operazioni con effetti; replay/idempotenza non sono
inferiti dal semplice riuso di request_id.

## Error taxonomy

| Codice | Significato |
| --- | --- |
| IPC_UNSUPPORTED_VERSION | Versione ben formata non supportata |
| IPC_INVALID_MESSAGE | Encoding, grammatica, profondità, duplicati, tipi o envelope illegali |
| IPC_PAYLOAD_TOO_LARGE | Byte della richiesta/risposta oltre il tetto; nessuna risposta tagliata |
| IPC_UNKNOWN_OPERATION | Operazione non registrata, senza esecuzione |
| IPC_UNAUTHORIZED | Origine/permission/authority non ammesse |
| IPC_STALE_CONTEXT | Contesto o handle non più corrispondente al proprietario valido |
| IPC_BUSY | Limite di concorrenza applicabile; non accodare input senza limite |
| IPC_INVALID_STATE | Intenzione non ammessa nello stato autorevole |
| IPC_TECHNICAL_FAILURE | Fallimento tecnico del servizio/I/O, senza esito normativo inventato |

Nessun campo libero message/detail/path/record/SQL/stacktrace: la UI traduce il
codice con testi statici. Cause tecniche typed restano Rust e seguono la policy
log minimizzata. CompletedInvalid è un risultato di validazione, Cancelled
un esito controllato, Partial una dimensione di completezza e Unknown una
valutazione non definitiva: non sono errori del trasporto né sinonimi di PASS.
`RunStatus` della library non viene riscritto per questa taxonomy.

## API realmente disponibili e limiti della prova

Verificati nei sorgenti locali pinnati Tauri2.12.5: `ipc::Request::body()`
restituisce `InvokeBody::Json` o `Raw(Vec<u8>)`; `Response::new(Vec<u8>)` diventa
`InvokeResponseBody::Raw`. La TS API installata dichiara invoke con Uint8Array/
ArrayBuffer e risposta raw ArrayBuffer. Nessuna API interna instabile `Invoke`
è necessaria. Manifest/lock e digest del sorgente consultato sono nel receipt.
Le pagine docs.rs della versione non erano accessibili: verifica contro il
sorgente effettivamente compilato, senza dedurre firme da un’altra release.

Questa è una proposta di integrazione: raw soltanto per poter applicare il
limite applicativo prima del parser JSON. Tauri/WebView possono avere già
allocato il buffer di trasporto; non si promette un tetto globale di memoria
contro flood o un limite prima di qualunque allocazione framework. Concorrenza,
permission e contenimento effettivi restanoP2-08/P3/P7. La route Json non può
sostituire il confine raw dichiarando falsamente una misura prima del parsing.

Il codec Python è una reference eseguibile di design senza dipendenze extra,
non un handler, un dispatcher o una certificazione Rust↔TS.71vettori PASS:
limite±1byte anche in risposta, UTF-8 multibyte, profondità32/33, duplicati,
versioni/tipi/extra, precisione interi, Unicode, errori e correlazione. Le prove
negative di byte/UTF-8/profondità impediscono esplicitamente l’ingresso al
parser completo. Nessun file, rete, chiave o API del prodotto viene chiamato.

Comando mirato:

```sh
python docs/migration/ipc-v1/verify_contract.py
```

Per l’adozione servono ratifica del contratto, DTO delle task successive e
G-MIG-2; i test Rust/TS e gli invarianti del bridge restano ai checkpoint
previsti. Nessuna CI workspace o prova Windows ripetuta per il solo documento.
