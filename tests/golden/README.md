# Golden Lintra — T-P5-07

Questa suite contiene solo dati sintetici. Non rappresenta l'accettazione
normativa SIOC/CAD né un servizio di validazione completo.

Esecuzione normale, senza modificare gli attesi:

```sh
cargo test --locked --test golden
```

Aggiornamento deliberato (Bash):

```sh
LINTRA_UPDATE_GOLDEN=1 cargo test --locked --test golden committed_scenarios_match_byte_for_byte
```

PowerShell, rimuovendo il flag anche se il comando fallisce:

```powershell
$env:LINTRA_UPDATE_GOLDEN = '1'
try { cargo test --locked --test golden committed_scenarios_match_byte_for_byte }
finally { Remove-Item Env:LINTRA_UPDATE_GOLDEN }
```

Solo `1` autorizza aggiornamenti; variabile assente significa confronto.
Altri valori sono errori di configurazione. Revisionare sempre il diff:
l'aggiornamento registra i byte prodotti, non approva nuove regole normative.
Non impostare la variabile nella CI. Gli aggiornamenti sono per singolo
documento, senza transazione fra tutti gli attesi: un errore richiede revisione
delle modifiche eventualmente già avvenute. `input.txt` e schema non cambiano.

Ogni directory scenario contiene i tre file DET §111: `input.txt`,
`expected.errors.jsonl`, `expected.summary.json`. Il runner visita ricorsivamente
la cartella e confronta entrambi i report byte per byte, senza parsing o
normalizzazione. File attesi mancanti, suite vuota, attesi senza input e link
simbolici sono errori. L'ordine delle directory è deterministico per i nomi
ASCII qui adottati. I path degli scenari sono metadati di test.

## Contratto del profilo sintetico v1

`synthetic/v1/schema.json` definisce un unico trace T1, un campo di sei byte,
UTF-8 fornito esplicitamente dall'adattatore e nessun vincolo semantico.
Da T-P9-06 l’adattatore esegue il RecordValidator LocalOnly reale; usa reader, audit,
ErrorFactory, JsonlSink, summary builder e serializer reali. Un nuovo profilo
richiede un produttore registrato: non eredita automaticamente questo controllo.

| Scenario | Byte originali | Processati / validi / invalidi | Diagnostica attesa |
|---|---|---|---|
| valid | `ABC   \nABCDEF\n` | 2 / 2 / 0 | Nessuna; i tre spazi sono byte significativi |
| invalid-length | `ABC\nABCDEFG\nABCDEF\n` | 3 / 1 / 2 | E_RECORD_LENGTH, riga 1: 3 byte; riga 2: 7 byte |
| crlf-utf8 | `ABCDEF\r\néABCD\r\n` in UTF-8 | 2 / 2 / 0 | Nessuna; é occupa due byte, entrambi i record ne hanno sei |
| no-final-newline | `ABCDEF` senza newline | 1 / 1 / 0 | Nessuna |
| empty-file | Zero byte | 0 / 0 / 0 | Nessuna; hash completo del file vuoto dopo EOF |
| empty-record | `\n` | 1 / 0 / 1 | E_RECORD_LENGTH, riga 1: 0 byte |

Gli attesi iniziali sono stati costruiti dal contratto sopra, con conteggi
espliciti e SHA-256 indipendenti tramite Python hashlib; non copiando output
Rust né invocando la modalità di aggiornamento per stabilire l'oracolo.
Messaggi e struttura JSON derivano dai contratti già verificati T-P5-02/04/05.

Input e schema sono sottoposti a hash sui byte originali. Il summary usa
`validator_version = golden-test`, `started_at = UNIX_EPOCH` ed `elapsed = 0`.
Quest'ultimo valore viene fissato nel dominio del solo adattatore di test prima
della serializzazione, perché il builder misura altrimenti il tempo reale.
Non si mascherano o modificano byte dei report dopo la produzione.

`.gitattributes` applica `-text` a tutto `tests/golden/**`: Git deve conservare
CRLF, LF, spazi e assenza del terminatore. La CI esegue la stessa suite su Linux
e Windows MSVC. Per spiegazioni di strutture, funzioni e condizioni vedere
[il report T-P5-07](../../docs/verification/T-P5-07.md).


## Profilo semantico T-P9-06

`semantic/v1/schema.json` è un nuovo schema sintetico separato, con tre campi:
Numeric width 3/range inclusivo 1..4, Date DDMMYYYY width 8, AN width 3 con
dominio logico A e stripping degli spazi finali. Nessun campo reale o migrazione.
`valid` contiene 2 record validi ai limiti del range, bisestile e anno 0001.
`invalid` contiene 5 record, 1 valido e 4 invalidi, 7 errori: 2 OutOfRange,
2 InvalidDate, 2 InvalidDomain, 1 InvalidType. L'ultimo record dimostra lo stop
primario del tipo prima del range; errori di campi indipendenti restano presenti.
Actual del dominio mantiene gli spazi fisici; l'ordine è riga poi campo.

Il produttore comune ora chiama `RecordValidator::validate_local` e non simula
più la validazione della sola lunghezza. I sei scenari precedenti mantengono
identici tutti i byte attesi. Il runner registra esplicitamente i due profili e
rifiuta profili sconosciuti, senza inferire schema dal contenuto dei record.
`generate_semantic_goldens.py` in docs/verification costruisce soltanto gli
attesi nuovi da una tabella di casi esplicita e dal contratto JSON; usa hashlib
indipendente e non legge output Rust. Nessuna modalità update usata per definire
l'oracolo. Il test di ripetizione confronta due esecuzioni indipendenti complete.

## Profilo DSL T-P10-06

`dsl/v1/schema.json` è un nuovo profilo sintetico di cinque byte: DETAIL AN
width 3 con stripping dichiarato, ORIGIN Numeric Exact width 2. Presence base
optional; regola REQUIRED-DETAIL con ORIGIN not_in 01/02 e FieldPresent DETAIL,
regola FORBID-X con guard esplicito IsPresent DETAIL AND Eq X e assertion
ConditionTrue ORIGIN=01. Il guard esplicito conserva la semantica missing→Unknown
senza trasformare automaticamente un confronto su dato mancante in false.

valid: 3 record validi. invalid: 4 record, 2 validi/2 invalidi, un
E_CONDITIONAL_REQUIRED e un E_CROSS_FIELD nell'ordine dichiarato. Il primo ha
target/actual fisico e context del controller; il secondo è una rule generica
senza target univoco, con entrambi i valori nel context. Summary e audit EOF
sono interamente confrontati; skipped_rules zero perché tutti i guard sono
decidibili in questi scenari. Unknown/abort sono verificati nei test di processo.

L'oracolo separato `docs/verification/generate_dsl_goldens.py` contiene gli esiti
espliciti, usa hashlib sugli originali e non legge output Rust. Nessun vecchio
atteso è riscritto. Il runner registra il terzo profilo, ora 10 scenari, e una
seconda esecuzione indipendente del caso invalid deve riprodurre ogni byte.
Solo `tests/golden/dsl/**/input.txt` è aggiunto alle eccezioni per testo sintetico.

## Profilo derivati T-P10-06 estesa

`derived/v1/schema.json` è separato dai profili precedenti: DETAIL AN 3 byte,
BIRTH/REF Date DDMMYYYY 8 byte, AGE Numeric Exact 3 byte. Definisce i tre piani
ratificati: età compiuta con policy mar01, sottostringa raw dell'anno REF e
delta degli anni. Le regole richiedono DETAIL per adulti con riferimento 2021,
confrontano AGE presente con l'età vera e limitano il delta anni a 25 (soglia
soltanto sintetica del profilo, nessuna regola di flusso reale).

valid: cinque record, inclusi 28/02 con nascita 29/02 e policy mar01, anno 0001,
zero e riferimento precedente alla nascita con età/delta firmati. AGE mancante
è esclusa da un guard IsPresent esplicito. invalid: quattro record, uno valido
e tre invalidi, nell'ordine E_CONDITIONAL_REQUIRED, E_CROSS_FIELD per età
non coincidente, E_CROSS_FIELD per delta oltre soglia. Context contiene i byte
raw delle vere fonti; il target esiste solo per FieldPresent DETAIL.

`generate_derived_goldens.py` costruisce questi tre errori da casi espliciti e
gli hash da hashlib, senza leggere output Rust o aggiornare vecchi attesi.
Il produttore resta quello reale comune, con clock/elapsed fissati solo nel
dominio di test prima della serializzazione; audit originali completi dopo EOF.
La suite ora registra quattro profili e dodici scenari; due esecuzioni nuove
del profilo invalid devono produrre ogni byte identico. L'eccezione input.txt
aggiunta è limitata a `tests/golden/derived/**/input.txt`.
