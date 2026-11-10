# Reference tree locale controllata

D10: schema e cataloghi devono essere forniti dall'operatore in una tree locale
controllata, aggiornata quando non sono in corso caricamenti. Il provider carica
e verifica i byte reali una volta; il run usa lo snapshot immutabile preparato.

Prima di leggere un dataset, Lintra controlla root, directory intermedie e file
usati. Il proprietario deve essere l'account corrente oppure un amministratore
dell'OS. Su Unix sono ammessi UID corrente/root e sono rifiutati bit di scrittura
gruppo/altri. Su Windows si controllano owner e DACL: account corrente, System
e Administrators possono modificare; una ACE che concede modifica ad altri
account/gruppi blocca il caricamento. ACL non interpretabili producono un errore
esplicito, senza assumere che siano sicure. Gruppi aziendali non documentati
non diventano automaticamente soggetti fidati.

La lettura condivisa dei cataloghi è ammessa: non sono richieste ACL private
da database per un elenco di codici. I controlli relativi a path traversal,
symlink nei percorsi relativi e confinement restano obbligatori. Il percorso
root esplicitamente scelto è risolto come nel provider precedente; il deployment
deve mantenerne stabile la selezione. Non si promette resistenza a sostituzioni
ostili concorrenti fra controlli per path e apertura.

Se Lintra segnala permessi o proprietario non attendibili, scegliere una copia
locale controllata oppure far correggere i permessi all'operatore responsabile.
L'applicazione non cambia automaticamente proprietà, chmod o ACL. Non vengono
modificati i file dell'autore per far superare il controllo.

Il rifiuto di una tree modificabile da altri è un errore tecnico di deployment
sia in Strict sia in ContinueWithAvailableEvidence. Il superamento di un budget
soft non può aggirarlo. Mancanza di un dataset, contenuto malformato, SHA diverso
o identità incoerente conservano i contratti Verified/Degraded/Unavailable e
Unknown già ratificati: l'assenza non diventa un insieme verificato vuoto.

Questi controlli di integrità non cifrano i dati e non sostituiscono protezioni
per-account di DB, spool, chiavi e report. I controlli delle librerie filesystem
non attestano una qualifica hardware o la sicurezza universale di ogni mount.
