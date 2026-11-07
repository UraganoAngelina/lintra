# CAD — componente sospensioni 2024-06.suspensions.1

Componente parziale A/B2/C per T-CAD-06: non è un profilo CAD clinico completo,
uno snapshot operativo o uno schema FROZEN. Dati di test esclusivamente sintetici.

Source_fields conserva13campi originali; source conserva digest PDF e tracciati
e decisioni CAD-14/G0. A:1.1/1.23; B2:tutti5campi; C:4.1/4.2/4.4/4.5/4.7/4.11.
PK complete originali, incluso progressivo C. Record B2 lungo41, C45, A218byte;
gli intervalli non si basano sui progressivi. Filler dei test esplicito sui
campi non selezionati, senza attribuirgli verifica clinica.

Il namespace sending_ulss viene verificato sui campi A tramite il binding
formato3 già qualificato e circoscrive il join C→B2. Ogni B2 per lo stesso evento
contribuisce per tutti i sei motivi validi 1/2/3/4/7/9. Sede1 è vietata soltanto
nei giorni strettamente interni; end Missing apre l'intervallo. Sede6 da sola
non implica una sospensione. FileB2 assente dà Unknown; presente vuoto è vuoto
accertato. La seconda passata corrente non apre né modifica lo storico.

I campi fisici DDMMYYYY riusano il parser calendario; numerici byte-esatti e
regex raw mantengono zeri e coordinate originali. Il proiettore conserva solo
le dipendenze compilate. Nessun nuovo linguaggio, driver o formato storage.
Dettagli e risultati effettivi: docs/verification/T-CAD-06-suspensions.md.
