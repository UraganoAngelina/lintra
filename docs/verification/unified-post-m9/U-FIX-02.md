# U-FIX-02 — Contratto CSV/XLSX

Data: 2026-11-10, Europe/Rome. Prerequisito U-GOV-03 `2cd4bdf`.
Stato: DONE, acceptance documentale e regressioni locali verificate.

D08 recepita in docs/REPORT_FORMATS.md, collegato dal README: CSV canonico
byte-fedele, XLSX per lettura spreadsheet con celle testo; nessun escaping
silenzioso, prefisso apostrofo o variante safe inventata. Prefissi =/+/-/@
preservati dal contratto; un consumatore CSV deve imporre il tipo testo.
Non viene dichiarato collaudo manuale di Excel/LibreOffice non eseguito.

Sorgenti/formati/test e golden invariati. CsvSink quota delimitatori e newline,
non modifica contenuti. XlsxReportWriter usa write_string; reader indipendente
e ispezione XML nei test dimostrano stringhe e assenza di formule.

Comando su **52ec340** prima del delta documentale:
`cargo test -p lintra --locked --offline --test csv_sink --test xlsx --test xlsx_writer`
exit0: 8+5+5 = **18 pass**, zero fail, una probe memoria ignored.
[Risultato e digest](U-FIX-02-formats-linux.json), log .log.gz. I/O failure,
malformed/mixed JSONL, rollover, summary Partial e cancellazione coperti.
Il messaggio StorageFull nel log è la fault injection attesa, non un test fallito.

File della task: REPORT_FORMATS, README, dossier/JSON/log e stato unificato.
CI nuova non richiesta per il solo delta documentale; nessuna CI precedente
attribuita al nuovo documento. G-HARD aggregato resta NOT_VERIFIED.
SHA task nella cronologia del dossier. Next READY U-FIX-03, AUTO_CONTINUE.
