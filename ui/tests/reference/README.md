# Riferimenti visivi

I PNG autorevoli restano `docs/ui/review/results-{1x,2x}.png`, ratificati
il 27/10/2026: nessuna copia rigenerata implicitamente dai test.

La UI operativa aggiunge controlli, virtualizzazione e dettagli consultabili.
Il confronto conserva le zone geometriche comuni del riferimento ratificato:
header, sfondo, bordi superiori e laterali dei due pannelli. Le altezze correnti
sono verificate separatamente (508px, T-P16-05), senza confrontarle con il pannello
illustrativo corto del prototipo. Non sono immagini dello stesso contenuto.

Le zone testo del marchio hanno tolleranza per canale dichiarata nel test; i
bordi e le superfici hanno tolleranza zero. I nuovi contenuti e controlli vengono
renderizzati nei due temi e scale e confrontati tra Linux e Windows senza
alcuna tolleranza. Questo non equivale ad approvare un nuovo PNG di riferimento.
