# DEC-17 — ratifica conclusiva T-P17-05, 5 novembre 2026

**RATIFICATA — esito D: prototipo non adottato.** Fonte: decisione vincolante
integrale dell'autore del 5 novembre 2026, successiva alla ratifica A estesa.
Questo registro ne conserva il contenuto operativo; la precedente autorizzazione
sperimentale rimane storica e non autorizza ulteriori implementazioni.

La valutazione è completata; i contratti non sono interamente dimostrati, con
beneficio Full insufficiente o negativo nei workload misurati. Il ValidationEngine
di produzione rimane sequenziale e conserva l'architettura a due passate,
ContextProjectionPlan, ProjectedRecord e ContextStore come confine fra fasi.
Nessun cherry-pick del prototipo, worker applicativo, attivazione LocalOnly/Full,
parallelizzazione della seconda passata, euristica, opzione CLI/UI o scheduler.
`cfg(test)` non è un'attivazione implicita nel prodotto. Nessun cambio a norme,
classificazione delle regole, dipendenze, contratti storici o persistenza.

## Evidenze e limiti

A1 dimostra accelerazione di validazioni locali indipendenti: nel controllo heap
off, 500.000 record bilanciati 0,909→0,346 s (2,63×); molti errori locali
6,837→2,857 s (2,39×). Non è prova di beneficio Full o Historical.
A3 comprende 96 campioni Full con equivalenza JSONL e campi deterministici dei
summary; le proiezioni alla barriera sono verificate separatamente nei test.
Full bilanciato con pochi errori: 10.000 record 101,4→100,2 ms; 100.000
1.149,5→1.114,1 ms; 500.000 6,241→6,558 s. Con molti errori:
100.000 2,507→3,827 s; 500.000 12,986→19,750 s (sequenziale→4 worker).
Non attribuire una causalità esclusiva senza ulteriori misure di profiling.

La cancellazione dinamica parallela può restituire errore tecnico senza summary
anziché Cancelled/Partial: requisito di adozione NON superato, conservato come
esito negativo. Nessuna riscrittura della semantica o dei test per renderlo conforme.
Fallback sequenziali per Historical, max_errors, dipendenze reference e cancellazione
pregressa non dimostrano funzionalità parallelizzate. Nessun beneficio dimostrato
sull'intero workflow CAD con persistenza storica SQLite.
Code bounded, emissione ordinata/backpressure, aggregazione e ricostruzione degli
indici hanno costo; assenza di accumulo globale degli errori/spool non lo annulla.
RSS campionato e riuso dell'allocatore non equivalgono a memoria viva del contesto;
le misure Linux sintetiche non qualificano prestazioni operative Windows.

500.000 record e oltre sono stress test, non carichi operativi attualmente previsti.
Non è formalizzato un massimo rappresentativo per flusso. Nessuna distribuzione,
dimensione media o soglia inventata; nessuna estrapolazione a 100/500/1.000/5.000
record non misurati. Circa 1,2 ms a 10.000 record non giustificano ora complessità
concorrente. La qualità si misura sui benefici effettivamente utili agli utilizzatori.

## Strategia futura, senza nuove implementazioni autorizzate

Questa decisione riguarda il candidato attuale, non esclude per sempre il
parallelismo. Prima completare CAD/SIOC e profilare il percorso sequenziale reale,
su workload rappresentativi, quando il workflow sarà sufficientemente verificabile.
Non avviare automaticamente una nuova campagna o progetto prestazionale.
Misurare separatamente: risoluzione/compilazione schema, preflight, lettura/hash,
parsing, campi, regex/reference lookup, locale, proiezione, contesto/indici, join,
cross-record, temporali, storico, JSONL, finalizzazione report e preview storica.
Ottimizzare solo costi osservati: allocazioni/copie/normalizzazioni/lookup ripetuti,
indicizzazione, diagnostica e serializzazione sono ipotesi, non lavori approvati.
Cache richiedono valutazione di correttezza, memoria e versionamento.

100–500, 1.000, 2.500, 5.000, 10.000 e, se plausibile, 20.000 record sono ipotesi
di studio da rivedere sui volumi attendibili, non benchmark obbligatori immediati.
Usare schemi CAD/SIOC realmente compilati, fixture sintetiche senza dati sanitari
reali, distribuzioni/regole/reference/errori realistici, controlli cross-record e
storici quando verificati. Misurare il workflow dell'operatore, incluso Historical
se pertinente; build release equivalenti, ripetizioni sufficienti, Windows 11 e
gli eventuali studi Linux. Non ottimizzare per il solo benchmark sintetico.

Un nuovo studio concorrente richiede un collo di bottiglia migliorabile dimostrato
dal profiling e beneficio end-to-end relativo e assoluto utile, senza soglie di
speedup inventate. Dimostrare: correttezza normativa, determinismo, max_errors,
cancellazione dinamica equivalente, propagazione guasti, sink sicuri, ContextStore
equivalente, nessun contesto incompleto pubblicato come completo, memoria/I/O
controllati e complessità giustificata. Eventuali partizioni riutilizzabili,
riduzione della comunicazione per errore, merge senza indici ridondanti,
aggregazione deterministica bounded, checkpoint e backpressure sono solo ipotesi.

GUI reattiva, progress e cancellazione rimangono requisiti distinti dal parallelismo
normativo: una validazione sequenziale fuori dal thread UI preserva il confine
UI/core senza richiedere worker concorrenti nei controlli.

## Chiusura e conservazione

Gate finale [42038174791](https://github.com/UraganoAngelina/lintra/actions/runs/42038174791)
verificato completed/success sul commit sperimentale
`8a02f55fba06be0e9784465cb8aa6ee347cd583b`, branch
`spike/dec17-selective-pass1`: sette job richiesti success, packaging UI opzionale
skipped come previsto. Non sostituito dal run intermedio 42036498302.
16 artefatti associati allo stesso SHA, scaricati e conservati con inventario hash;
log CI, run, job e inventario server in `T-P17-05/native-final/`.
Gate locali: suite completa su 41defb9, unica variazione successiva full.rs nella
fixture; hash del refinement confermato, fmt/Clippy/5 test mirati positivi e CI
finale sul nuovo SHA. 365 hash della campagna A3 confermati. Ledger originali e
log locali conservati; nessuna limitazione convertita in PASS.

**T-P17-05 DONE: valutazione sperimentale completata, adozione del parallelismo
non autorizzata.** Audit, A1, Full, codice, fixture e riproduzione restano sul branch
sperimentale al commit sopra; risultati A1/A3, summary, ledger e log sono recuperabili
anche dai documenti di progetto. Il commit conclusivo documentale segue quel codice.
Il runtime di produzione non incorpora gli helper sperimentali, neanche cfg(test).

Conflitto sull'ordine: [CNF-DIAGNOSTIC-ORDER-01](CNF-DIAGNOSTIC-ORDER-01.md), aperto
separatamente, nessuna modifica runtime o ratifica automatica dell'ordine.
Riprendere T-CAD-02 dopo prerequisiti verificati, poi task READY autorizzate;
nessun subagente, Superpowers o sviluppo concorrente. Commit/push/CI autorizzati,
Quality Gate obbligatori, nessun PASS senza evidenza o modifica tacita ai contratti.
