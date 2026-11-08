# ADR-HISTORY-EXISTENCE-001 — ratifica sostitutiva implementativa, 8 novembre 2026

Fonte: messaggio dell'autore «Ratifica sostitutiva — Implementazione completa del
requisito storico di esistenza», successivo alla prima ratifica della giornata.
Il messaggio integrale prevale su questa registrazione operativa.

Revocato il limite alla sola specifica. Autorizzati progettazione, implementazione,
integrazione e qualifica della capability generica history.existence.v1, dello scope
history_including_current e set_exists, dentro T-CAD-12 e prima di P18-04/M9.
La specifica tecnica di fatti/versioni/indici/provenienza/copertura/compatibilità/
errori/atomicità/recovery precede immediatamente il codice: nessuna seconda
approvazione ordinaria e nessun rinvio postM9 o fase nuova.

Confermati: DEC-05 iniziale nello storico approvato più corrente; niente iniziale
per trimestre, current-only/deferred o regole eliminate. Compiler tipizzato solo
per costrutti implementati e dichiarati, genericità senza nomiCAD nel core,
dipendenze/predicato/chiavi/namespace/identitàsemantica verificate. PASS1 reale
streaming produce evidenze minime, niente rilettura o righe persistite. Campi
necessari invalidi non provanoTrue/False; errori indipendenti non distruggono
automaticamente evidenze valide. ContextStore resta effimero, produzione sequenziale,
nessuna terza passata normativa non necessaria e DEC-17 invariata.

True con prova positiva autonoma ancora valida, anche con altri gap; False solo
senza prove e con universo pertinente completamente qualificato; altrimentiUnknown.
Non dedurre inizio storia da primo trimestreDB o indice vuoto. Se i contratti non
consentono di provare copertura completa, conservarne esplicitamente l'indeterminatezza.
Indici/token non sostituiscono fatto/copertura/provenienza. ErroriI/O/corruzione/
integrità/chiave distinti daUnknown eFalse, nessuna falsa Complete o no-op.

Autorizzate estensioni necessarie del repository generico e codec con versioning
esplicito se incompatibili, inventario reale versioni, nessuna reinterpretazione,
migrazione automatica, ricostruzione da fingerprint o falsacopertura legacy.
Archivi già approvati invariati. HMACv1/keyprovider/ACL/filesystem/distribuzione
segreti invariati; nessun dato sanitario superfluo o nuova chiave/cryptoalgoritmo.
Se l'autenticità non è ottenibile con contratti crypto esistenti: sospendere solo
la parte incompatibile, descrivere esattamente la dipendenza e chiedere scelta
mirata; non usarla come rinvio generico del lavoro. Nessun nuovo protocollo inventato.

Fatti pubblicati con la stessa approval e transazione/CAS dello snapshot. Nessuna
visibilità da preview o dopo rollback/report/cancel/CASfallito/crash non recuperato;
recovery eCOMMITambiguo come già ratificati. Sessioniimmutabili invariate.
Correzioni/annullamenti/riattivazioni secondo contratto vigente: non basta essere
stato vero in una qualunque versione, né si elimina un fatto ancora valido perché
il nuovo invio ha rivalutazioni. Casi normativi realmente non coperti richiedono
solo una decisione circoscritta, non una nuova ratifica ordinaria di programmazione.

Accettazione reale applicativa: input→parser→locale→proiezioni/fatti→preview→T1commit→
reopenSQLite→T2query/cross/diagnostica/approval. Venti scenari richiesti: positivi
storici/correnti, assenza completa/incompleta, legacy, namespace, più iniziali,
annullamenti/correzioni, anni, missingconsent, CAS/rollback/cancel/report, reopen,
versioni/integrità/privacy/JSONL. Profilo eseguibile completo83campi e tutte le
regole ratificate, non filler o disabilitazioni; ulteriori gap implementativi
si risolvono primaDONE12. Gate locali/golden/transazioni/recovery/compatibilità/
privacy/nativeLinux/MSVC/JSONL/persistenza sullo SHA effettivo, senza PASS presunti.
Poi12DONE→P18-04→M9 solo con condizioni reali, non per conteggio task.

Single-agent/no delega/sessioni autonome/parallelismo; commit/push/CI/correzioni e
prosecuzioneREADY autorizzati. Stop solo per contraddizioni normative materiali,
nuove sceltecrypto/sicurezza, incompatibilità persistenti non risolvibili con i
contratti o cambiamenti semantici che richiedano ratifica. Questa ratifica supera
soltanto i limiti operativi incompatibili della prima, preservandone i vincoli.
