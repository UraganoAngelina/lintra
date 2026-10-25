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
