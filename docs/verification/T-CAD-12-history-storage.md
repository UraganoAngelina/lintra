# T-CAD-12 — persistenza dei fatti storici, 9 novembre 2026

Stato: implementazione di componente, non completamento CAD12. La ratifica
sostitutiva autorizza questo lavoro; C1 e L1 sono già decise. Nessuna nuova
approvazione ordinaria richiesta. Single-agent; docs/migration esclusa.

## Modello e condizioni

`persistence/history.rs` separa payload deserializzabile e `HistoryFactBatch`
non costruibile dal chiamante. `HistoryAssessment` distingue positivo,
non-testimone e indeterminato con causa. Sono persistiti solo fingerprint
protetti, indice del token nello snapshot e classificazione minima: nessuna riga,
CF, valore del predicato o chiave di evento grezza. L'ordinal non è una linea
clinica; la linea reale per i diagnostici rimarrà nel produttore PASS1.

`validate` controlla formato, trimestre, revision/base consecutivi, fonte
ApprovedImmutable/Full/CompletedValid, namespace/schema/hash/fingerprint/periodo,
inventario Present contro Absent, numero di fatti, unicità e key ID. Le origini
C1 richiedono periodo valido non futuro, fonte/hash/responsabile/data e unicità.
La verifica strutturale non certifica la verità della fonte esterna: il produttore
applicativo dovrà acquisire l'attestazione qualificata e l'approvazione esplicita.

`create` protegge la proiezione canonica dell'intero payload con
IdentityProtector::protect_projection esistente. `verify` verifica le prove
private, ricostruisce scope/chiave/proiezione e confronta identità e fingerprint.
Un token valido trapiantato su un payload diverso non basta. Nessun nuovo
MacPurpose, algoritmo, chiave, derivazione o modifica ai byte HMAC1 precedenti.

`with_history_facts` lega i batch alla revisione e fonte dello snapshot, ordina
per digest del piano, rifiuta definizioni duplicate. `verify_records` verifica
numero dei record fisici target, ordinal, trace/key_kind, chiave e fingerprint
esatti; i token secondari non possono sostituire il record fisico.

## Inventario autenticato e query senza record grezzi

Un sigillo per batch non rileverebbe DELETE dell'intero batch. `SnapshotManifest`
è obbligatorio per ogni snapshot nuovo di formato3, anche senza fatti. Impegna
fonte completa, revisione, formato, root dei token protetti ordinati e root dei
batch canonici ordinati per definizione. Root SHA256 incrementali con framing
lunghezza/elemento; nessuna duplicazione in memoria dell'intero inventario dei
token per calcolare la root. Il manifest stesso è una proiezione tecnica protetta
con la stessa primitiva qualificata. Il numero SQL non è un commitment.

SQLite3 aggiunge `history_batches` e `snapshot_manifests`, FK namespace/revisione,
PK indicizzate e prove private separate. Snapshot, indici, batch e manifest entrano
nella stessa transazione IMMEDIATE/CAS. Rollback, receipt e risoluzione del COMMIT
ambiguo comprendono quindi l'intera unità. Nessuna scrittura separata in preview.
`read_snapshot` verifica anche gli ordinal e la root dell'inventario dei token.

`load_history_facts` restituisce `HistoricalFactSnapshot`, capability di lettura
non deserializzabile: fonte/revisione, batch verificati e supporto inventario
esplicito. `read_fact_inventory` verifica sigillo del manifest e root dei batch,
senza caricare la tabella records. Tre seek sulle PK, nessuna scansione globale
delle identità. È lettura di evidenza, non un verdetto di conformità o copertura.
Il chiamante deve ancora fissare/verificare la generazione fra più letture.
Il default della port restituisce StorageUnavailable; nessuna assenza inventata.

## Compatibilità e qualifica

Reader/header accettano esattamente1/2/3. Su1/2 i percorsi precedenti restano
leggibili; la nuova port segnala inventory_supported=false. Un writer con nuovi
fatti rifiuta archivi1/2 prima della mutazione. L'upgrade esplicito1→2 conserva il
significato originario: consenso omissioni richiede2, non3, e nessuna tabella di
fatti è creata tacitamente. Nessuna migrazione o rigenerazione di archivi approvati.

Test SQLite: reopen/receipt, payload/periodo/origini/versione alterati, eliminazione
batch/manifest/token con conteggio SQL riscritto, record estraneo/digest/ordinal,
CAS/rollback/COMMIT ambiguo, replay fra revisioni, inventario vuoto/legacy/assente,
piani EXPLAIN indicizzati e ordine dei batch indipendente dall'inserimento.
La fixture nativa scambiata dalla CI include ora4snapshot,2namespace e un batch
storico realmente protetto; verifica manifest e nuova port anche sul consumer.
Il roundtrip locale non è una prova Windows: occorre la CI propria.

Risultati finali e impronte saranno registrati nel ledger omonimo. Le prime
regressioni emerse (header formato2, requisito upgrade consenso e aspettativa
fixture corrente) sono corrette mantenendo immutate le fixture legacy congelate.

## Lavoro ancora necessario dentro CAD12

Compiler del sottoinsieme history_including_current/set_exists/Eq e identità
semantica; produttore PASS1 con validità degli operandi; query su vista lifecycle
L1 (mai OR dei positivi di ogni vecchio snapshot); copertura C1/False/Unknown;
contesto neutro PASS2, gate Partial/ValidationOnly e approval atomica dei fatti.
Composizione eseguibile CAD83, golden applicativi T1/T2/T3, privacy/performance e
Quality Gate completo sul codice finale. Il compilatore pubblico storico resta
fail-closed finché questi collegamenti non sono realmente qualificati.
CAD12 non DONE, P18-04 non READY, nessun FROZEN/M9/release/tag.


## Handoff tecnico per il collegamento runtime

L'observer attuale `FlowRecordObserver::on_record` riceve schema/riga/PK ma non
la validità dei singoli operandi. `ProjectionCollector` ammette intenzionalmente
righe fisicamente valide con errori locali indipendenti: il solo `local.valid`
non è una prova corretta per il predicato storico. Il prossimo delta deve
trasferire l'evidenza locale pertinente alle dipendenze del piano nello stesso
PASS1, preservando diagnostici/budget/cancellazione. Non presumere che il callback
attuale significhi riga interamente valida e non ricavare un positivo da un
operando non qualificato solo perché la PK è leggibile.

`joins::compile_joins` e il lowering capability rifiutano ancora il nuovo scope;
non è sufficiente sbloccare quei due controlli. Prima servono piano semantico e
producer/context eseguibili. `HistoricalAuditService` usa la più recente fonte
Present per trace, distingue Absent e mantiene le generazioni; la query L1 deve
riusare tale semantica di lifecycle, non l'indice latest delle occorrenze che
per contratto conserva anche identità omesse. I batch di questo delta non sono
ancora una query clinica abilitata. I casi C1/L1 non richiedono di ripresentare
le decisioni già ricevute all'autore.


## Gate locali effettivi — sequenza finale

Sette comandi exit0: fmt, Clippy workspace/alltargets/allfeatures, workspace
intero default e allfeatures, core lib senza default features, release e boundary.
283impronte della selezione sorgenti/UI/bundle/tooling ferme, indice e HEAD
invariati. Altri193file tests/ sono uguali al commitbase e invariati fra controllo
supplementare ed esito finale. Log e SHA256 verificati nel ledger. Le due suite
workspace riportano ciascuna1028pass/0fail/6ignored (conteggio degli esiti delle
suite, inclusi subprocess e doctest); corelib141pass/0fail/4ignored.35testSQLite
positivi,10nuovi. Nessun test ignorato è contato come eseguito.

La sequenza preliminare si era fermata a Clippy per un if annidato; correzione
idiomatica applicata, log/ledger preliminari preservati separatamente. La sequenza
finale non eredita successi parziali. CI nuova del componente ancora necessaria;
nessuna qualifica Windows o nuova esistenza storica runtime dedotta dai locali.
