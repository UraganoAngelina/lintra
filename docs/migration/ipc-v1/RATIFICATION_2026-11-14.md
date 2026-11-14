# Ratifica T-MIG-P2-02 — 14 novembre 2026

Autorità: risposta esplicita dell’autore alla richiesta sulla proposta
`MIG-IPC-WIRE-V1-2026-11-14`, ricevuta in questa sessione. Nessun ID di chat
o firma ulteriore inventato. Testo della decisione conservato integralmente:

APPROVO la proposta IPC v1 per T-MIG-P2-02, con le seguenti precisazioni vincolanti:
1. Protocollo: JSON UTF-8 trasportato mediante raw bytes Tauri, con envelope e DTO esplicitamente versionati.
2. Dimensione: massimo 1 MiB (1.048.576 byte) per singolo messaggio, applicato sia alle richieste sia alle risposte, includendo l'intero envelope. Rifiuto esplicito dei payload eccedenti il limite, senza elaborazioni parziali.
3. Profondità: massimo 32 livelli JSON, con validazione effettiva prima dell'elaborazione applicativa.
4. Compatibilità: schemaVersion = 1 come contratto iniziale. Nessuna compatibilità implicita con versioni sconosciute; modifiche additive compatibili solo se esplicitamente documentate e validate. Modifiche incompatibili richiedono una nuova versione.
5. Error taxonomy: separazione rigorosa fra errori tecnici, errori di protocollo e risultati normativi della validazione. Un errore tecnico non deve mai essere rappresentato come CompletedInvalid o CompletedValid.
6. Trasferimento dati: nessun dataset completo, record sanitario raw o trasferimento massivo di diagnostici attraverso IPC. Utilizzare paging, handle opachi e query eseguite in Rust secondo i contratti previsti.
7. Verifiche: includere test mirati per payload validi, UTF-8/JSON malformati, dimensioni e profondità ai confini consentiti, versioni incompatibili e corretta classificazione degli errori.
Considera questi punti parte della ratifica di T-MIG-P2-02, senza aprire ulteriori richieste di approvazione per i dettagli implementativi ordinari.
Recepisci la decisione nei contratti pertinenti, completa le evidenze richieste da P2-02 e prosegui autonomamente con i task successivi secondo le dipendenze del Piano Unificato.
G-MIG-2 rimane aperto fino al completamento dei suoi requisiti complessivi.
Applica il workflow di sviluppo già approvato: verifiche mirate e proporzionate, senza rieseguire inutilmente l'intera suite o la CI dopo questa singola ratifica.

## Recepimento tecnico

Il primo wire ratificato usa `schemaVersion`, sia nell’envelope sia nel DTO
`payload` o `error`. Il nome `wire_version` apparteneva alla proposta non
ratificata su0f65539 e viene rifiutato senza alias impliciti. Errori DTO con
`category` verificata contro il codice: `protocol` o `technical`; gli esiti
normativi restano risultati. La classificazione non è scelta dal renderer.
I dettagli ordinari delle task successive sono autorizzati senza nuove
richieste; G-MIG-2 mantiene tutti i requisiti di fase e il checkpointP2-10.
