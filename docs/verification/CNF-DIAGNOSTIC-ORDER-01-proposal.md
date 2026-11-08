# CNF-DIAGNOSTIC-ORDER-01 — proposta concreta, 8 novembre 2026

Stato: PROPOSTA, non ratifica. Nessun contratto o output modificato.

Fonti: DET §79 prescrive trace→line→field→stage→rule; DET §115 e la ratifica
DEC-17 mantengono invece il runtime sequenziale a due passate. Il conflitto
è già OPEN nel dossier omonimo dal 5 novembre. Il runtime emette le locali/PK
anche del secondo trace prima delle cross del primo trace. Stabilità e identità
cross-OS non dimostrano il rispetto dell'ordinamento globale descritto da §79.
Task P18-04 richiede espressamente nessun CNF aperto; non può chiuderlo d'ufficio.

## A — preservare il contratto osservabile corrente (proposta consigliata)

Testo proposto per DET §79, da applicare soltanto dopo ratifica:

> Per lo stesso input/schema e le stesse opzioni, il report mantiene ordine
> stabile. La chiave esterna è la passata: prima controlli locali e unicità delle
> chiavi di tutti i tracciati, poi controlli cross-record/cross-trace. Dentro
> ciascuna passata l'ordine è trace dichiarato nello schema → riga → campo in
> ordine fisico, dove applicabile → stage → dichiarazione della regola. Un
> controllo di evento senza campo locale mantiene il proprio ordine dichiarato;
> non riceve coordinate cliniche inventate. Nessun riordinamento globale tra
> passate, rilettura o buffer globale di errori. Budget e cancellazione preservano
> il prefisso effettivamente emesso e il riepilogo di interruzione.

Aggiornamento proposto per DET §115: sostituire il riferimento al conflitto OPEN
con il link alla ratifica A e mantenere DEC-17 D e il runtime sequenziale.
Allineare le formulazioni equivalenti in ARCH/MASTER soltanto nel perimetro
P18-04, con tracciabilità della ratifica. Nessuna modifica a norme CAD, storage,
HMAC, stage locali, sink, golden esistenti o comportamento del prodotto.
La prova dedicata deve mostrare locale del secondo trace prima della cross del
primo, due esecuzioni byte-identiche e il prefisso in caso di budget.

## B — mantenere l'ordine globale originario di §79

Confermare trace→line→field→stage→rule come ordine globale richiede un disegno
attuativo separato: riconciliare streaming, memoria, report/budget/cancellazione,
provenienza e due passate, poi cambiare prodotto e golden con nuovi gate.
Non è una correzione documentale né un riordino da applicare tacitamente.
Senza il relativo contratto approvato non si modifica il runtime.

La scelta riguarda un conflitto architetturale reale; non è un'approvazione
ordinaria di commit/CI. CAD08 rimane DONE. Proseguono le attività CAD12 che non
presuppongono una nuova ratifica dell'ordine; nessun freeze/M9 implicito.
