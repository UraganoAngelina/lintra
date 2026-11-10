# Report per scambio e lettura in spreadsheet

Contratto D08 ratificato il 10 novembre 2026. JSONL, CSV e XLSX derivano dagli
stessi diagnostici Rust; l'esportazione non corregge i dati ricevuti.

| Formato | Uso | Trattamento dei valori |
|---|---|---|
| JSONL | Elaborazione e audit canonico | UTF-8, una diagnostica per riga, ordine canonico; summary separato |
| CSV | Scambio dei valori originali | RFC4180, CRLF, quoting di virgole/virgolette/newline; nessun prefisso aggiunto o valore normalizzato |
| XLSX | Consultazione in Excel/LibreOffice | Celle diagnostiche scritte esplicitamente come testo, anche zeri iniziali e valori che sembrano formule |

Per leggere il report in uno spreadsheet scegliere **XLSX**. Un CSV conserva
anche i prefissi `=`, `+`, `-`, `@`: il quoting CSV delimita la cella, ma non ne
stabilisce il tipo per il programma che lo apre. Se è necessario importare un
CSV in uno spreadsheet, usare l'importazione guidata e impostare le colonne
come testo prima di caricare i valori. L'apertura con doppio clic non fa parte
del contratto di preservazione dei tipi. Non viene creata una variante CSV
sanificata, né aggiunto un apostrofo ai dati canonici.

La GUI Slint corrente esporta tramite i writer Rust. La CLI corrente offre
JSONL/XLSX; questo documento non introduce un'opzione CSV che non esiste.
L'export è integrale, non una riscrittura dei soli risultati filtrati. La
receipt distingue i singoli output riusciti e gli errori: report e summary
non costituiscono una sola transazione di filesystem.

Prove automatizzate sintetiche: tests/csv_sink.rs verifica byte RFC4180, UTF-8,
zeri, valori formula e fallimenti tecnici; tests/xlsx.rs rilegge con un reader
indipendente i testi e l'assenza di formule; tests/xlsx_writer.rs ispeziona
anche XML e celle stringa. Non sono prove manuali di una versione specifica
di Excel/LibreOffice, che non viene dichiarata collaudata qui.
