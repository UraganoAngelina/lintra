# T-CAD-12 — preflight, 8 novembre 2026

Prerequisiti04..11/PERSIST12 verificati nei dossier CI:04 b59a5b9/42368062138;
05 84444c2/42394000741;06 3719d14/42372080602;07 6477bb2/42390593313;
08 3388527/42476999364;09 1740a6d/42231182241;10 46d574f/42004833488;
11 6af5f85/42203885693;PERSIST12 638feec/41857167059. Tutti gate documentati,
08 ricontrollata via API/step/log/import reali nella sessione corrente.
G0 approvato con rinvii al407f89e, non approvazione automatica di un freeze.

Fonti: prompt12, MASTER/protocollo14/20, ARCH45, DET166–174, ADR001,
CAD_HISTORICAL_PROFILE, candidate83campi/note/fixtureplan/PDF e componenti02..11.
I componenti qualificati sono parziali; non basta concatenarli o rimuovere
snapshot per produrre una certificazione clinica completa. Occorrono mapping
esplicito degli83campi, ledger di ogni regola/non-eseguibile e prove end-to-end.

Il compilatore pubblico ordinario e storico-audit rifiuta ancora combinazioni
snapshot/invarianti/temporali; il produttore integrato va collegato ai piani
compilati e allo stesso input, senza outcome inventati o riletture nascoste.
Il coordinatore già distingue Complete/Partial dall'esito normativo, verifica
copertura delle trace presenti e richiede PROCEED esplicito per le assenti.
Non estendere il contratto audit-only tacitamente per introdurre snapshot.

Scope iniziale: composizione verificata e prove private del vero motore a due
passate→proiezioni indivisibili→coordinatoreSQLite suT1/T2/T3, assente controvuoto,
PKcomplete e D→C senza progressivi, privacy e determinismo. Nessuna nuova API
produttiva abilitata finché l'intero ingresso/produttore non è qualificato.

Conflitto reale già OPEN: CNF-DIAGNOSTIC-ORDER-01, DET79 vs ordine per passata
ratificato come comportamento produttivo. Preparata proposta concreta A che
preserva il prodotto; la scelta richiede ratifica prima di chiudere il CNF/M9.
Le attività indipendenti CAD12 proseguono senza modificare l'ordine corrente.
Single-agent, no background/delega; docs/migration preesistente sempre esclusa.
