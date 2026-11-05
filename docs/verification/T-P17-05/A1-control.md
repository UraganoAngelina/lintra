# A1 — controllo dell'overhead di misura

Controllo release sul medesimo eseguibile
`57a7b520bfb2a50b13c3b268e87b55f8885fed15f88b70a0b821cae2ea086ca1`,
sorgenti `83e54deceea70045e4b655138e71e72cc656b2a3`. Quattro processi
foreground sequenziali, ordine on/off/off/on, 24campioni ciascuno,96totali;
402,564s complessivi, nessuna build concorrente,345hash sorgenti invariati.
Tutti i confronti JSONL byte-per-byte e summary (sola differenza elapsed esplicita,
originali conservati) superati. Ledger e96campioni in `A1-linux-control/`.

500.000record sintetici, tracciati bilanciati,6ripetizioni per cella. Mediane:

| Errori | Heap counter | Sequenziale s | 1worker s | 2worker s | 4worker s | Speedup4 |
|---|---|---:|---:|---:|---:|---:|
| Pochi | off |0,9090|1,1999|0,6083|0,3460|2,63x|
| Pochi | on |0,9020|1,2059|0,5924|0,3169|2,85x|
| Molti | off |6,8369|7,0897|4,0329|2,8573|2,39x|
| Molti | on |7,1799|7,4765|5,1971|4,9168|1,46x|

Il contatore globale delle allocazioni aumenta sensibilmente la contesa nel caso
con molti errori e più worker. Le misure iniziali con heap tracking non bastano
per descrivere lo speedup non strumentato. Off elimina gli aggiornamenti atomici
condivisi, mantiene il controllo readonly della modalità e il campionatore RSS:
non è una build priva di ogni strumentazione. Heap off è null, non zero.
Le misure restano sintetiche, con cache scaldata, sul solo LocalOnly indipendente;
nessuna inferenza automatica su Full, produzione o soglie DEC-14.
