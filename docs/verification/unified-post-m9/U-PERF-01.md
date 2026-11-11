# U-PERF-01 — Volumi da fonte aggregata

11 novembre 2026. Prerequisito U-GOV-02 DONE; D13 e produzione sequenziale
vigenti. Preflight34f5b71. U-OPS-04 resta sospesa WAITING_CI43050943529;
U-REL-03 è stata soltanto esaminata in lettura, nessuna implementazione avviata.

La fonte anonima necessaria è già nel kit: CAD_SAMPLE_ANALYSIS.md del16ottobre,
sole statistiche aggregate. Nessun nuovo sourcing, lettura/copia dei file
personali originari, benchmark su dati reali o modifica delle fonti dell'autore.
Receipt con SHA, provenienza e verifiche in U-PERF-01-source-receipt.json.

REPRESENTATIVE_WORKLOADS.md/json definiscono un carico CAD sintetico derivabile
dalla forma osservata: A2748,B12825,C45104,D137249;187926record,8540805byte senza
terminatori. B2 è non fornito, non zero: mantenere warning/consenso e completezza
reale. Larghezze verificate sul profilo operativo CAD83 corrente. Separati piccoli
oracle, ipotesi di densità diagnostica e stress tabella300k/1M. Nessun default500k.

Questo è l'output di analisi/oracle richiesto dalla task, non un generatore o una
misura già eseguiti. Il documento dichiara i limiti: un solo invio osservato,
SIOC non calibrato, storico/B2/tasso diagnostico/hardware non determinati;
nessuna media/massimo/SLA o approvazione di ottimizzazioni. I valori mancanti
non bloccano lo sviluppo né diventano quantità inventate. U-PERF-02 userà il
bridge reale dopo MIG-P7-08, con benchmark interamente sintetici.

Acceptance: conteggi/aritmetica e larghezze confrontati con fonti indipendenti,
manifest JSON leggibile e disclosure dei parametri; check piano128nodi e
diff-check. Nessun sorgente runtime, golden, norma, dipendenza o140/140 modificato.
CI non richiesta per questo delta documentale; CI43050943529 qualifica soltanto
il codice export34f5b71, non viene riattribuita a questa analisi. G-PERF aggregato
resta NOT_VERIFIED. **DONE per l'analisi e l'oracle dei volumi**, dopo controllo
effettivo di tutti i conteggi, null, aritmetica LF/CRLF e cinque larghezze dello
schema: receipt presente, piano128nodi aciclico e diff-check PASS. Nessuna misura
prestazionale attribuita. SHA nella cronologia di questo dossier.
Conteggio alla chiusura:29/128DONE,99rimanenti (20U e79MIG);140M9 separatamente
concluse. Next: chiudere U-OPS-04 solo se CI43050943529 positiva; indipendente
READY U-REL-03, prerequisiti U-OPS-01/U-FIX-01 già verificati.
