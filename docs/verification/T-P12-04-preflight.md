# T-P12-04 — Preflight dell'integrazione CLI XLSX

2026-10-23. Nessun codice della task iniziato; gate 12-03 ancora da chiudere.
Prerequisiti: 12-03 e 8-03, verificare evidenze native prima della prosecuzione.
Prompt: opzione XLSX post-analisi da JSONL, JSONL temporaneo se non richiesto,
atomico; nessun workbook nuovo su Err o Cancelled. Parziale Completed è lecito
purché il Riepilogo conservi completezza e provenienza ratificate.

CLI attuale richiede --errors e --summary. Estensione prevista: --xlsx opzionale,
--errors opzionale quando è richiesto XLSX, almeno una destinazione errori/XLSX;
summary resta esplicito. Vecchi argomenti mantengono il comportamento precedente.
Controllare collisioni fra tutti i final/.part, input e bundle prima dei guard.

Il backend richiede Write+Send, AtomicReportFile::writer restituisce dyn Write:
serve un accesso writer additivo Send sul File già owned, preservando writer esistente,
senza unsafe, wrapper Send fittizio o thread. Per il JSONL .part servirà rilettura
del guard dopo flush, senza pubblicarlo prima del successo dell'export. Valutare
un metodo reader additivo che apra il part owned, senza rendere mutabile il path;
restrizioni di directory controllata/TOCTOU restano quelle documentate.

Per JSONL temporaneo usare RAII con creazione esclusiva nella directory temporanea
di sistema, permessi sicuri e cleanup anche su errore/cancellazione. tempfile è
stato qualificato nel subtree del backend; un eventuale uso diretto richiede
pin della versione già presente, verifica licenza/MSRV e registrazione DEC-06/15,
non un filesystem buffer fittizio o un file persistente lasciato silenziosamente.

Generare XLSX soltanto dopo validate + finish JSONL + summary effettivo. I guard
restano non pubblicati fino al completamento dei produttori; ogni file conserva
la pubblicazione atomica propria, senza promettere una transazione multi-file.
Cancelled può mantenere la politica precedente per JSONL/summary, ma non pubblica
un nuovo XLSX e conserva l'eventuale workbook già esistente. Nessun writer nel loop.

Test futuri: processo vero con XLSX solo/insieme a JSONL, fallback temporaneo e
cleanup; output in errore, collisioni final/part e input/bundle, report precedenti
preservati, Cancelled senza workbook nuovo, Partial dichiarato nel Riepilogo;
regressioni CLI ed evidenza nativa Linux/Windows. Nessun esito già dichiarato.
