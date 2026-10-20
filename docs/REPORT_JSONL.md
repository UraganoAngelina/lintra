# JSONL delle diagnostiche

Contratto implementato in T-P5-04, da ARCH §25/§28/§39C.3 e DET §52–55.
Un oggetto per violazione, UTF-8 senza BOM, singolo LF finale, nessun header
o summary. Ordine di emissione del chiamante preservato. Oggetti top-level
serializzati nell’ordine seguente, senza spazi di formattazione:

1. code: simbolo E_* della tabella centrale.
2. flow, schema_version, trace: ID esatti, nessuna normalizzazione.
3. line: numero intero one-based.
4. record_identity: sioc_record_id, fiscal_code, episode_id (string/null),
   technical_ids (oggetto ordinato lessicograficamente per chiave).
5. field: null per diagnostica record-level, altrimenti id/name/start/end;
   start e end sono one-based inclusivi, conversione già nel dominio.
6. actual: null oppure oggetto display/raw_hex (string, string/null).
7. expected: string/null.
8. context: array in ordine dichiarato di oggetti trace/field_id/value,
   dove value ha la stessa forma display/raw_hex.
9. message: testo umano, mai da parsare per recuperare metadata.

Alias esterni flow/trace/line e field.start/end seguono l’esempio ARCH §25;
DTO esplicite separano la forma su file dai nomi dei campi Rust. Il valore
semplice e il contesto abbreviato dell’esempio concettuale sono rappresentati
con i payload completi DET §34/§52–53: actual resta strutturato per non perdere
raw_hex, context è una sequenza con trace esplicito per non collidere su field
ID uguali di trace differenti. Nessun dato del modello viene perso o dedotto.
Non si inventa record_key_digest: l’identità contiene solo i target attualmente
definiti, un futuro digest richiede il relativo contratto di identità.

Null, stringa vuota e spazi restano distinti; le sequenze di controllo nel
JSON sono escaped dal serializer. raw_hex è diagnostico e resta invariato.
Mappe hash mai usate come ordine di output. Nessun path assoluto o dato
ambientale aggiunto automaticamente. Nessun timestamp nella singola violazione.

JsonlSink<W: Write> scrive durante on_error senza trattenere errori o creare
un Vec globale. Context e mappe vengono serializzati in prestito. on_summary
fa solo flush: il documento summary separato compete a T-P5-05. finish
consuma il sink, verifica flush e restituisce il writer solo in caso di successo.
Guasti JSON/Write/flush o metadati malformati rendono il sink terminale.
Un prefisso parzialmente scritto non è un report pubblicabile; .part e
pubblicazione atomica competono a T-P5-06, non a questo writer generico.

Fixture sintetica di riferimento: tests/fixtures/diagnostics.errors.jsonl.
Il confronto è byte per byte su Linux e Windows, senza normalizzazioni.
