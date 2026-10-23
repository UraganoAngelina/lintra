# Ripresa GitHub — 2026-10-23

Resolver locale 127.0.0.53: curl api.github.com, timeout di risoluzione dopo
10 secondi; gh API fallisce per connessione. Primo push interrotto dopo stallo;
secondo push con timeout 45s exit 124. Nessun rifiuto di approvazione automatica.
Non sono risultati negativi di CI: nessun dispatch era ancora stato eseguito.

Diagnosi indipendente: DNS-over-HTTPS pubblico con TLS e indirizzo esplicito
restituisce risposte valide per github.com e api.github.com; rete HTTPS funziona.
Git supporta http.curloptResolve, verificato con `git help --config`.
Push ripreso con risoluzione esplicita limitata al comando: 6e4b411..66c3d61
pubblicati con successo. Nessun file hosts/resolv.conf o servizio di sistema
modificato, nessuna disabilitazione TLS o configurazione Git persistente.

Chiamate API autorizzate tramite curl --resolve, URL/Host/SNI GitHub originali e
TLS verificato. Credenziale locale ottenuta da gh solo in memoria, passata alla
configurazione curl su stdin; non stampata, scritta su disco o inserita negli
argomenti visibili del processo. Helper temporaneo in /tmp, senza token salvato,
nessun processo in background. Restano massimo tre chiamate per gate e ≥45s
fra le chiamate Actions/check-runs. Verifica check-runs del commit permette
leggere insieme i due job nativi senza ulteriori richieste per ogni job.

Commit 373c44c (11-04) e discendente 66c3d61 (12-01) mantengono invariato il
codice/test reference di 11-04; una CI cumulativa su 66c3d61 può verificarli
entrambi. Gli esiti reali vengono registrati nei dossier, senza attribuire una
run separata inesistente a 373c44c.

## Aggiornamento dopo il primo gate

Il record DNS di api.github.com può cambiare fra le chiamate: un successivo
POST su un indirizzo precedente è terminato per connection timeout. Ora la
risposta DoH viene aggiornata prima di ogni richiesta; nessun retry POST nascosto.
Ogni tentativo Actions, anche fallito, viene contato nel massimo di tre.
Per 11-05: primo dispatch fallito, secondo riuscito, una lettura ha mostrato
Linux success e Windows in_progress. Nessuna quarta lettura della stessa run.
Per la qualifica XLSX: dipendenze recuperate nella sola cache Cargo isolata,
checksum verificati; nessuna modifica alla risoluzione DNS o ai source Cargo
registrati nel repository. La CI successiva usa registro/host standard.
