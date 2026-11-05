# DEC-17 — perimetro della valutazione del parallelismo

**Ratifica successiva del5novembre2026:** [A estesa](../verification/DEC-17-ratification.md).
Il testo seguente conserva la proposta originaria e i suoi limiti storici;
per il mandato corrente vale la ratifica, senza riscrittura retroattiva.

5 novembre 2026. Proposta, non ratifica. Piano§13.1 richiede DEC-17 prima di
T-P17-05; DET§115 esclude parallelismo nel primo MVP, piano§7.8 consente
valutazione misurata e adozione solo con beneficio e determinismo provati.

T-P17-02 DONE, tre baseline/36scenari reali, nessuna ottimizzazione runtime
adottata. Queste misure Full non misurano lo speedup di tracce indipendenti.
Non possono da sole approvare un’implementazione parallela nel prodotto.

## A — solo spike misurato, prodotto sequenziale (proposta)

Autorizzare T-P17-05 in un branch/check-out dedicato, eseguita da un solo agente.
Confrontare validazioni LocalOnly indipendenti sequenziali e con pochi worker
Rust std, senza nuovi crate e senza cambiare API/CLI/UI del prodotto. Eseguire
foreground un unico benchmark posseduto; nessun daemon/background, subagente,
Full parallelo o accesso concorrente allo storico. I worker del solo spike sono
parte dell’esperimento runtime, non agenti/task parallele.

Dati sintetici e schema identici, più volumi e ripetizioni, stessa build release,
tempi e memoria misurati separati dalla compilazione. Output JSONL identico
byte-per-byte, ordine dei trace del chiamante, input/errori realmente osservati.
Provare limiti e guasti; usare spool limitati su disco se necessario per mantenere
streaming/ordine, senza Vec globale di errori. Non presentare memoria/spool come
gratis. Produrre report e decidere motivatamente se conviene.

La ratifica A autorizzerebbe soltanto la valutazione: l’adozione nel prodotto
resterebbe una decisione successiva basata sui risultati e sui contratti di
max_errors/cancel/sink. Nessuna soglia di speedup inventata come norma.

## B — mantenere l’esclusione MVP e rinviare lo spike

Confermare runtime sequenziale senza svolgere lo spike ora. T-P17-05 rimane
esplicitamente rinviata e non DONE; il piano/milestone conserva questo residuo.
La scelta non autorizza a sostituire misure effettive con una dichiarazione.

La decisione non blocca T-P18-03 o il percorso CAD con prerequisiti soddisfatti.
