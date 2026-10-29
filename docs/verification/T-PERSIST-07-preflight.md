# T-PERSIST-07 — preflight della macchina a stati

Prerequisiti:06DONE,code3e8fae4,CI40853726275 tutti4job success; P7DONE nel
ledger. Fonti:ADR001§4/6,ARCH45,DET166–174,prompt07. Single-agent,nessuna norma
CAD/SIOC né contratto di protezione modificati. Dipendenze nuove non necessarie.

File/API previsti: compilazione dichiarativa SnapshotPlan in persistence_profile,
coordinatore applicativo dedicato sessioni con stati privati e snapshot candidato
immutabile; export library; query repository generica per una sessione approvata,
adapter SQLite/fake; test lifecycle e policy. Nessun SQL nell'engine/applicazione.

La preview07 riguarda il candidato e la generazione di base; il motore diff08,
integrazione09 e regole temporali CAD rimangono successivi. Il compilatore dei
profili può accettare il solo piano snapshot supportato, continuando a rifiutare
invarianti storiche/temporali e senza dichiarare il profilo eseguibile da Full.
Il lifecycle riceve un risultato di validazione e ne verifica binding/provenienza:
non trasforma metadati ApprovedImmutable in un certificato di validità clinica.

Da precisare prima del codice: binding della sessione ai parametri compilati,
recupero dello stato approvato dopo altre sessioni e restart, consenso per tutte
le omissioni, esito ambiguo e idempotenza senza eseguire nuovamente il commit.
L'API pubblica attuale ha soltanto submission_id e periodo opaco: non contiene
un'identità di sessione approvata né una query corrispondente. Evitare di usare
l'ultimo snapshot come prova che una sessione precedente sia modificabile.

Test previsti: transizioni valide/invalide; preview senza scritture; cancel,
report/IO fallito senza commit; errore E_APPROVED_SESSION_IMMUTABLE su modifica;
retry esatto/ack perduto; CAS concorrente e transazione doppia; scope e policy
cambiati; riapertura SQLite. Solo fixture sintetiche. Gate fmt/Clippy/test e
Linux/MSVC reali prima di DONE07.

## Scelte implementative precisate

Binding tecnico versionato nella stringa periodo opaca dei soli nuovi draft,
protetto insieme alla Source dal contrattoHMAC04/05; nessun nuovo campo wire o
schema SQL. Periodi legacy non vengono reinterpretati o migrati: un'approvazione
non associabile in modo verificabile ferma il nuovo lifecycle con errore esplicito.
Query repository additive: generazione e cursor128 per soli metadati Source,
con autenticazione e PK(ns,revision). Costo O(invii) per preview, non O(identità)
né per record; limite trasparente nel rapporto, nessun indice sessione implicito.
Tipo/DOMINI e consenso derivano dal profilo compilato, non da flow hardcoded.
Coordinatore standalone verificato con outcome sintetici di componente;
integrazione09 e diff08 restano separati. Il risultato pubblico non è un certificato
crittografico di Full. Dopo i gate07, priorità UI16-06→07→08 autorizzata29/10.
