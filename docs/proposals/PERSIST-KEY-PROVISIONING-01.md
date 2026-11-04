# PERSIST-KEY-PROVISIONING-01 — provider della chiave per la CLI

31 ottobre 2026. **Proposta storica; alternativa A ratificata con modifiche il
4 novembre 2026.** Fa fede la [ratifica aggiornata](../verification/PERSIST-KEY-PROVISIONING-01-ratification.md),
che autorizza chiave comune nel bundle privato portabile Slint e risoluzione
deterministica relativa all'eseguibile. Riguarda T-PERSIST-10;
non cambia HMAC, formati persistenti, regole normative o autorizzazioni sui dati.

## Decisione effettivamente mancante

La [ratifica ADR-PERSIST-002](../verification/ADR-PERSIST-002-ratification.md)
prescrive il provider esterno e lascia esplicitamente aperto il provisioning:
«Cifratura a riposo/retention/provisioning/backup sicuro restano decisioni da
risolvere prima dei dati reali».

Il codice offre `PersistenceKeyProvider`, ma non un provider configurabile
dall'eseguibile. I provider esistenti nei test usano soltanto materiale sintetico.
La CLI non può inventare una chiave, incorporare quella dei test, ignorare il
parametro o fingere che la configurazione sia completa. Occorre scegliere
l'adapter concreto prima di aggiungere i relativi argomenti operativi.

## A — file locale protetto, fornito esplicitamente dall'operatore

Proposta minima per completare il wiring senza nuova dipendenza:

- L'operatore fornisce una chiave binaria casuale di 32 byte, un ID non segreto e
  un percorso esplicito. Il programma non genera o sostituisce chiavi.
- Gli argomenti contengono ID e percorso, mai il materiale segreto. Nessuna
  variabile d'ambiente con la chiave, scoperta automatica o fallback.
- Il lettore accetta solo un file regolare, con lettura limitata a 33 byte per
  rifiutare sia lunghezza errata sia dati aggiunti. Trasferisce il materiale
  direttamente a memoria azzerabile e chiude il file prima di aprire gli input.
- Permessi/ownership Linux e ACL Windows devono impedire la lettura ad altri
  utenti secondo il medesimo confine già imposto all'archivio. Symlink, reparse
  point, hard link e sostituzioni della destinazione richiedono controlli sul
  file realmente aperto; nessuna attenuazione dei controlli del backend.
- Chiave, archivio e report hanno percorsi distinti; nessun output o file `.part`
  può sovrascrivere la chiave. Errori e Debug sono redatti.
- L'archivio verifica sempre la key-confirmation già memorizzata, anche per
  namespace vuoto. Un ID/materiale diverso non inizializza una storia nuova.
- Creare un archivio resta un'azione esplicita, distinta dall'apertura di quello
  esistente. La modalità preview non committa e l'approvazione è separata.

Il file non cifra la chiave: la sua protezione dipende dal sistema e dalle misure
operative scelte dall'autore. Approvare A autorizza questo adapter; **non chiude**
cifratura a riposo, retention, backup o prontezza per dati reali. Nessun dato reale
verrebbe usato per implementazione o collaudo.

File previsti: adapter dedicato del provider, CLI args/wiring, contratti
applicativi e test sintetici su entrambi gli OS. Nessuna chiave reale versionata,
nessuna migrazione/rotazione o modifica dello stile Slint.

## B — provider del sistema operativo prima del wiring CLI

Progettare prima l'integrazione con il deposito credenziali dei sistemi
supportati, qualificando API, disponibilità, dipendenze e comportamento quando
il deposito manca o è bloccato. Nessun backend OS o formato di provisioning è
già approvato da questa proposta. Il wiring dipendente attende quella decisione;
API generiche e task indipendenti possono proseguire.

## Verifiche richieste dopo la scelta

Chiave assente/corta/lunga/illeggibile, ACL o permessi non sicuri, link e
sostituzione, collisioni con input/archivio/report/part, stessa label con materiale
diverso, provider non disponibile e zero segreti negli output. Preview/abort,
approvazione esplicita, report fallito prima del commit, CAS, cancellazione ed
esito ambiguo rimangono verifiche necessarie del percorso CLI storico.

Decisione richiesta: approvare A con i limiti sopra, oppure scegliere B ed
esaminare il relativo contratto prima dell'implementazione dipendente.
