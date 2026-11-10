# U-NORM-02 — Dataset consegnati definitivi

Data: 2026-11-10, Europe/Rome. Prerequisito U-NORM-01 `a5689ae`.
Stato: DONE per freeze dell'inventario D12. G-SIOC0/1 NOT_VERIFIED.

U-NORM-02-dataset-freeze.json conserva percorso, byte, SHA256, identità,
versione/stato e dimensione del contenuto di **otto fonti consegnate**:

- Tre estrazioni CAD ricevute: prestazioni pagina5, bisogni70posizioni pagina6,
  sintomi ICD9 pagina7; SOURCE_EXTRACTED_REVIEWED, non cataloghi universali.
- Membership runtime CAD: cad-icd9-cm-clinical-signs /2024-06.pdf7.1,63chiavi;
  cad-prestation-types /2024-06.pdf5.1,33chiavi. Hash dei byte attuali registrati.
- Dataset embedded di esempio, esplicitamente SYNTHETIC/demo e non copertura
  regionale ufficiale.
- Workbook SIOC ricevuto e PDF CAD ricevuto: fonti normative, non membership
  dataset. La data del filename PDF non sostituisce la versione CAD2024-06.

I dodici dataset SIOC richiesti nel FROZEN sono assenti dal set consegnato:
foreign-country-codes, health-authority-codes, hsp11-structure-codes,
icd9-cm-diagnosis-codes, icd9-cm-procedure-codes, icpc-codes,
istat-municipality-codes, italian-region-codes,
regional-entity-manager-codes-dgr-2961-2012, regional-udo-codes,
ria11-structure-codes, sts11-structure-codes. PENDING_DATASET è il loro stato
storico ricevuto; nel piano corrente D12 significa **assenza dichiarata e
deferred nel perimetro definitivo**, non una campagna di acquisizione pendente.

Le membership CAD limitate non colmano automaticamente le assenze SIOC;
ICPC/ISTAT CAD non vengono inventati. reference-data nella root non contiene
file: non è prova di dataset vuoti. Le fonti embedded restano demo. Nessuna
nuova norma, catalogue completeness o conformità universale è dedotta dai nomi.
I controlli progettati/qualificati e i rinvii di U-NORM-01 restano invariati.
Nuovo normativo solo D11 o bug reali dimostrati, come ratificato.

Verifiche effettuate: enumerazione mirata di sole sorgenti normative/dizionari
già consegnati, parsing intestazioni/versioni/conteggi, SHA256 e presenza dei
file. Nessun input paziente, archivio operativo o chiave letti. Nessuna rete,
sourcing, modifica del kit o dei dataset; checkout adiacenti non modificati.
Questa task non rilegge o ricertifica ogni voce normativa: preserva l'inventario
e le provenienze già qualificate. Nessun test/CI runtime nuovo richiesto per
questa documentazione; gate di enforcement ancora da eseguire.
File: freeze JSON, dossier e stato. SHA nella cronologia del dossier.
Next READY U-NORM-05, poi U-NORM-03; AUTO_CONTINUE.
