# CNF-DIAGNOSTIC-ORDER-01 — ordine delle diagnostiche

Stato corrente: A RATIFICATA, gate locali positivi, CI propria da qualificare.
Registrato il 5 novembre 2026 durante T-P17-05; storia originale preservata.
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


## Attuazione della ratifica A — 8 novembre 2026

Ratifica umana con condizioni in CNF-DIAGNOSTIC-ORDER-01-ratification.md.
Audit e quattro prove sintetiche nuove in dossier audit e tests/diagnostic_order.rs:
ordine interno effettivo documentato, schemaorder, eventi senza campo, JSONLbyte
identico, prefissi budget nelle due passate e cancellazione ai checkpoint reali.
Nessun runtime, sink, algoritmo, golden o codice diagnostico modificato.
DET79/115 e riferimenti necessari ARCH17/N12 allineati alla ratifica.
Cinque gate locali exit0,477impronteferme:957core/0fail/6ignored e47regressioni
allfeatures/0fail; fmt/Clippyworkspaceallfeatures/boundary/pixel positivi.
Ledger/inventario verificati. La chiusura tecnica RESOLVED attende i gate nativi
pertinenti sul proprioSHA; nessuna chiusuraP18-04/M9 o DONE12 implicita.
