# ADR-HISTORY-EXISTENCE-001 — proposta, 8 novembre 2026

Stato: PROPOSTA, non ratificata. Nessuna modifica produttiva/storage/crypto.
Ambito: chiudere il requisito storico reale di T-CAD-12, non ridefinire CAD.

## Evidenza che rende necessaria la scelta

DEC-05-CAD-VALUTAZIONE-INIZIALE, approvata il19ottobre, richiede almeno una
B1.2.8=1 nell'intero storico dell'evento incluso il corrente, non in ogni invio.
SCHEMA_AUTHORING §Join e §Temporal formalizza history_including_current,
set_exists e history.existence.v1. Il compiler corrente rifiuta tale scope in
src/schema/joins.rs con «join history requires the persistent execution contract».
La nuova prova cad12_history_obligation_is_rejected_instead_of_replaced_with_current_input
controlla precisamente quel rifiuto, non un errore generico di fixture.

I contratti qualificati03/04/05 conservano record_key, fingerprint di riga,
progressivo ed evento come tokenHMAC; non conservano campi B1.2.8 consultabili
né una prova indicizzata della verità del predicato. Un fingerprint dell'intera
riga non permette di ricavare 2.8. Nel caso T1con iniziale, T2con sole rivalutazioni,
non si può sostituire lo storico con i soli recordT2 né dichiarare violazione
per un indice che non è mai stato costruito. Non si può neppure persistere
arbitrariamente una riga clinica per rendere valutabile la query: DET172 e
ADR001§5 impongono minimizzazione e protezione. Il motore temporaneo a due
passate resta distinto dal repository persistente (ARCH45/DET166).

Questo è un contratto architetturale persistente mancante, non una nuova scelta
normativa «storico oppure trimestre»: quella decisione è già approvata.
Il compiler fail-closed è corretto e rimane attivo finché il nuovo produttore
non avrà un contratto completo e propri gate. Le prove03..11 non coprono tale
esecuzione e non la rendono implicitamente autorizzata o implementata.

## A — fatti d'esistenza protetti e minimali (proposta consigliata)

1. Piano compilato generico derivato solo da join history_including_current e
   assertion set_exists con predicato locale chiuso. Nessun ID CAD nel core.
   Rifiutare predicati fuori dal sottoinsieme implementato prima dell'I/O.
2. Il produttore applicativo calcola fatti del predicato sulle stesse righe della
   passata1 realmente validate, senza rilettura, righe grezze persistite o valuta
   clinica in chiaro. Chiavi di correlazione protette nel namespace/flusso/trace
   esplicito; definizione legata a schema e predicato, non al solo nome regola.
3. I fatti e la relativa copertura sono autenticati e pubblicati soltanto con la
   medesima preview/approvazione/commit atomico della sessione. Rollback, errore
   report, Cancelled e CAS fallito non lasciano nuovi fatti consultabili.
4. La query combina invio corrente e storia approvata verificata per evento e
   ULSS. True richiede una prova; False richiede un universo/coprimento verificato;
   archivio/definizione/indice non qualificati restituiscono Unknown o errore
   tecnico secondo la causa. L'assenza di un indice non prova assenza di iniziali.
5. Il contratto dovrà esplicitare codec/versione, copertura autenticata e indice
   prima del codice storage. Se occorre un'estensione storage, introdurre versione
   esplicita: nessuna migrazione automatica, nessuna reinterpretazione o riscrittura
   di archivi precedenti. HMACv1, gestionechiavi e protezioni filesystem invariati.
   Un archivio privo delle prove richieste non ottiene falsa Complete per default.
6. Prove: T1iniziale/T2sole rivalutazioni; nessuna iniziale su universo completo;
   contesto assente; più iniziali; namespace distinti; riaperturaSQLite; rollback,
   consenso/omissione, CAS, privacy e comparazione nativa. Conservare la decisione
   normativa esistente e distinguere contesto storico dal contesto temporaneo.

La ratifica A autorizza il solo contratto fact-only e il completamento della
specifica attuativa verificabile prima della modifica del codec. Non approva
ancora esecuzioni reali, cifratura, retention, migrazione, release o FROZEN.
Non si aggiungono cryptoalgoritmi né repository CAD-specific.

## B — rinviare history.existence mantenendo il rifiuto

Continuare soltanto qualifiche di componenti, senza attivare un profilo CAD
completo o dichiarare DONE12/M9. Gli83campi, i join correnti e il tempo non
sostituiscono DEC-05. I gate reali delle task precedenti restano validi.

## Stato e attività concluse senza la decisione

CAD08DONE con CI42476999364 e settejob richiesti reali. CAD12: tre nuove prove
private positive collegano file reali sintetici→Full→proiezioni→approvazione/
SQLiteT1/T2/T3, omissioneB2conPROCEED/Partial, privacy e relazione D→C/JSONL.
Il loro schema è dichiaratamente di componente, con filler, non CAD83campi.
La composizione83campi resta bozza non eseguibile; nessun requisito è convertito
in deferred/not_enforced per farla compilare. Il gate fail-closed resta reale.
Il conflitto d'ordine CNF-DIAGNOSTIC-ORDER-01 è separato e non risolto da A.
