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
L'adattatore di test controlla **solo la lunghezza**; usa reader, audit,
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
