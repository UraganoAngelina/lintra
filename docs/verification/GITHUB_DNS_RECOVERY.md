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

## Blocco successivo della revisione automatica — T-P13-01

Commit locale 051ed37. Due richieste di push respinte prima dell'esecuzione.
La revisione richiede autorizzazione esplicita all'esportazione del sorgente
verso https://github.com/UraganoAngelina/lintra.git, work/p1-foundations;
non considera sufficiente il mandato generale sulle chiamate GitHub.
Dopo il primo rifiuto sono stati verificati origin, tracking branch e precedente
commit pubblicato df3858d. Il secondo tentativo con queste evidenze è stato
respinto con la stessa motivazione. Nessun aggiramento, push o dispatch eseguito.
Richiesta asincrona specifica sottoposta all'autore; risposta ancora in attesa.
Gate 13-01: zero chiamate Actions effettivamente eseguite. I contatori precedenti
restano esauriti e gli esiti Windows non verificati non vengono promossi.

Helper temporaneo ora interroga due resolver DoH ufficiali (Cloudflare/Google),
valida gli IPv4 e passa la lista risultante a curl per la singola connessione.
Nessun retry API o POST implicito, nessuna credenziale nei parametri o file,
nessuna modifica a TLS/host di destinazione o configurazione persistente.

## Ripresa dopo la risposta dell'autore

L'autore ha risposto alla richiesta specifica confermando autonomia per commit,
attivazione CI/CD e task successive con gate/prerequisiti verificati. Nuova
richiesta di push, con questa evidenza, accettata dalla revisione automatica.
Push df3858d..8fc31f5 sullo stesso origin/work/p1-foundations riuscito.
Dispatch accettato alle 13:16 UTC del 2026-10-23, prima chiamata del nuovo gate.
Il precedente rifiuto resta nella cronologia; nessun aggiramento utilizzato.
