# CNF-DIAGNOSTIC-ORDER-01 — ordine delle diagnostiche

Stato: OPEN, registrato il 5 novembre 2026 durante T-P17-05; risoluzione separata.
Fonti: DET §79 e §115, contratto a due passate, audit T-P17-05 e ratifica conclusiva DEC-17.

Il runtime sequenziale emette per passata: prima locali/PK di tutti i tracciati,
poi cross-record/cross-trace. La formulazione trace → line → field → stage → rule,
se interpretata globalmente, descrive un ordine differente: nella fixture dello
spike le locali di T1 precedono le cross di S0. Il confronto byte-per-byte positivo
prova equivalenza al sequenziale osservato, non conformità a entrambe le letture.

Nessuna nuova regola approvata; nessun cambio prodotto, golden o test nella
chiusura T-P17-05. Prima di futura adozione concorrente dipendente dall'ordine,
riconciliare esplicitamente le fonti e preservare tracciabilità/determinismo/golden.
Qualunque modifica del contratto pubblico o dell'output osservabile richiede il
normale processo di approvazione architetturale. Il conflitto non blocca CAD-02
che deve preservare il comportamento corrente senza risolverlo implicitamente.
