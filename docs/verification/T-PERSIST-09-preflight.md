# T-PERSIST-09 — preflight integrazione SIOC audit-only

Ripresa 31 ottobre2026: branch work/p1-foundations su fafac99, remoto tracciato
identico; lavoro09 parziale già presente conservato e completato. Nessun override
annidato trovato in src/tests/docs. docs/migration resta materiale esterno escluso.
Gate precedenti riletti dai dossier versionati, nessuna task DONE riaperta.
La relazione tecnica corrente è T-PERSIST-09-integration.md; gate in corso,
nessun DONE locale o nativo dedotto dal solo preflight.

30 ottobre2026, base fafac99 pubblicata. Prerequisiti:08 CI41069605562,
6-07 CI39439627879, P13 CI40192491629, tutti realmente positivi. UI9/9 e M8
qualificate; task17-03 documentale completata separatamente, soglie non definite.
119/140DONE=85%. Materiale esterno docs/migration preservato e non incluso.

Fonti: prompt09, ADR001, ARCH45, DET166–174, piano20; contratti04 protezione,
07 lifecycle,08 audit, compiler/registry/servizio Full e profili SIOC operativi.
DEC-SIOC-HIST-01 resta aperta: una PK ripetuta fra run non diventa una nuova
violazione. Solo intra-run E_DUPLICATE_RECORD_KEY già previsto nello schema.

## Stato dei prerequisiti tecnici

I profili structural-minimal.1/.2 dichiarano già cross_run_audit sulle cinque
trace e namespace organisation_namespace integer1..4294967295. Snapshot=None:
nessuna sessione trimestrale o annullamento CAD va introdotto per SIOC.
Il compiler ordinario rifiuta attualmente ogni persistence_policy/capability.
Il registry carica un'unica source per risoluzione e ne conserva l'hash prima
della binding encoding. Non si può rimuovere la policy dal raw e simulare
supporto storico: serve un percorso compilato/servizio esplicito.

Il protector04 autentica PK e una proiezione canonica con digest di definizione;
il descrittore deve provenire dal profilo compilato. Il record non va copiato
integralmente né archiviato grezzo. Il servizio08 confronta token/definizioni,
non decide quali campi clinici siano equivalenti. Prima di implementare fissare
nel codice e nei test il descrittore della proiezione usata per il fingerprint.

Il coordinatore07 accetta al momento soltanto snapshot con anno/trimestre.
Per audit-only è necessaria una preparazione/approvazione di archivio priva di
quella semantica, mantenendo preview, report prima del commit, CAS, immutabilità,
idempotenza ed esiti ambigui. Le codifiche delle sessioni trimestrali esistenti
non vanno riscritte o reinterpretate. Approvare un archivio non certifica norme.

## Perimetro previsto

Compiler/registry con percorso storico esplicito, ancora fail-closed per piani
storici/temporali non implementati; servizio applicativo con scope Full e
namespace esplicito, raccolta/protezione delle sole identità/fingerprint prima
dell'adapter, audit08 e lifecycle07 esteso in modo additivo per archivio audit.
Se necessario aggiungere un observer neutro dell'engine per ricevere record/key
borrowing durante la validazione reale; nessuna query SQL nel motore e nessuna
seconda scansione non verificata di file potenzialmente cambiati.

Source/outcome/reference/input audit devono provenire dalla stessa esecuzione;
nessun schema riaperto dopo il run, placeholder o validazione fittizia. Errore,
cancel o validazione non riuscita non pubblicano identità. UI e CLI ordinarie
conservano il proprio wiring; nessuna configurazione DB/chiave implicita.

Test sintetici con cinque trace SIOC e tuple PK lette dalle fonti: duplicato
intra-trace invalid, stessa PK fra run audit-only, fingerprint/namespace/provider,
preview e consenso espliciti, report fallito/no commit, CAS/restart/ambiguità,
nessun raw/CF/secret nel default codec. Normative FROZEN e profili esistenti
preservati. Fmt/Clippy/workspace due assetti/core-no-bundle/boundary e CI reali.
Nessun DONE prima della qualifica; single-agent, nessun subagente/background.

## Scelte implementative entro i contratti approvati

Percorso iniziale Full/Flow esplicito: compiler storico con proof non forgiabile,
registry da una singola source, nessuna rimozione della policy/capability dal raw.
Il percorso ordinario conserva il rifiuto delle capability storiche; snapshot,
identity invariants e temporal non implementati restano fail-closed.

Per il fingerprint audit-only si usa una proiezione esplicita dei byte fisici del
record, senza terminatore di riga. Il descrittore versionato include encoding e
layout dei campi della source compilata; il suo digest resta nello scheme04.
Cambiamenti di byte/padding possono produrre Updated, senza significato clinico
implicito. Il record viene dato all'HMAC a segmenti borrowing: frame equivalente
al JoinKey di una singola componente, senza copia completa e senza nuovo codec DB.
La PK usa sempre il JoinKey logical canonico esistente. Il descrittore è prodotto
dal compiler, non da un nome di flow o da una scelta implicita nell'adapter.

Observer neutro Full: riceve record e PK già estratta nel collector pass-one,
prima di consumare il buffer; il motore non conosce crypto/SQL. Il producer
applicativo conserva soltanto StoredIdentity protetti. Source/outcome e diff
rimangono separati; la finalizzazione viene differita fino all'esito storico.

Aggiungere al port la lettura dell'audit di protezione: SQLite legge la metadata
già esistente, il fake può essere inizializzato esplicitamente per i nuovi test.
Il protector del servizio deve verificarla anche con un namespace vuoto, prima
di leggere file: una chiave incompatibile non crea un universo di PK nuovo.

Riutilizzare il lifecycle07 estendendo il binding privato con una variante
archivio audit basata su namespace/flow/submission ID, priva di anno/trimestre.
Prefisso/versione distinto della metadata opaca; le codifiche trimestrali
esistenti restano byte-identiche. Snapshot policy rimane fonte della richiesta
PROCEED per file assenti; in audit-only l'assenza non annulla lo storico.
Immutabilità, report prima di commit, CAS e risoluzione esito ambiguo restano
quelli del coordinatore; nessuna approvazione clinica automatica.
