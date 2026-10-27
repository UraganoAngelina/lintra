# T-PERSIST-01 — profili dichiarativi verificati

## Perimetro e prerequisiti

ADR-PERSIST-001, ARCH45, DET166–174 e authoring della persistenza.
P3/P10 verificati: T-P3-05 CI39264362959/b658e05 e T-P10-06 CI39920597392/c2e0d1d.
I modelli raw/compiled delle policy erano già presenti: riutilizzati senza
riscrivere JSON FROZEN o inventare backend e scelte di protezione dei dati.

## API e ragioni di correttezza

`SchemaCompiler::compile_persistence_profile(raw, other_versions)` restituisce
`CompiledPersistenceProfile`, separato da CompiledFlowSchema. Il normale compile
continua a rifiutare persistenza runtime non implementata. La nuova API permette
la verifica dichiarativa senza simulare audit storici. Campi privati e getter
immutabili; nessuna deserializzazione del proof (doctest compile_fail).

Il linter verifica formato e struttura; il compiler dei parametri verifica i tipi.
Namespace obbligatorio, non vuoto e senza duplicati: ogni nome deve risolversi in
un parametro dichiarato, nessun valore implicito. `keys` risolve trace e PK reali,
conserva ordine, rifiuta riferimenti mancanti, liste/field duplicati e codici errati.
Audit esplicito richiede persistence.audit.v1; non abilita capacità di esecuzione.

Per declared_group, versioni distinte dello stesso flow devono essere fornite
realmente. Sono conservati fingerprint dei sorgenti, namespace identico e tipi
compatibili, PK selezionate con stessi ID/ordine e semantiche fisiche/logical/missing.
Solo l’offset può cambiare: ciascun sorgente estrae la stessa tupla nella propria
posizione. Nessuna conversione o alias implicito. Una vecchia versione può ancora
dichiarare same_schema_version: la nuova dichiarazione di gruppo richiede comunque
il suo sorgente concreto e namespace esplicito. Vecchi profili disabilitati o con
namespace diverso sono respinti. SchemaVersion senza Ord rimane invariato; elenco
hash in ordine corrente poi versioni dichiarate. HashSet usato solo per membership,
mai per l’ordine del risultato. BTreeMap ordina i binding per indice trace.

Policy disabilitata produce None e non cancella le PK locali; snapshot, vincoli
identità storica e regole temporali sono rifiutati come unsupported, non scartati.
I successivi task introducono repository/transazioni/runtime; questo artifact non
può essere passato a ValidationService come schema eseguibile.

## File

- src/schema/persistence_profile.rs: proof, risoluzione namespace/PK, gruppi e API.
- src/schema/compiler.rs e src/lib.rs: modulo/export pubblici additivi.
- tests/persistence_profile.rs: otto casi positivi/negativi, fixture sintetiche.
- Dossier/gate/status/indice: evidenze e continuità della task.

## Verifiche e limiti

Fmt --all --check e Clippy --all-targets --locked -D warnings, default/all-features,
positivi (cache offline). cargo test --offline --locked:698pass/81suite;
all-features706pass/81suite, entrambi0fail/2ignored, inclusi10doctest.
Otto nuovi test verificano namespace, formato sconosciuto, riferimenti, disabilitata,
compatibilità reale/negativa, movimento PK, rigetto feature future e non-eseguibilità.
Nessuna nuova dipendenza; nessun dato paziente o scelta crittografica. Runtime non
attivato, DEC-SIOC-HIST-01 rimane aperta. CI nativa da completare prima di DONE.

## Handoff

Task IN_PROGRESS/WAITING_GATE dopo pubblicazione. Prossima indipendente pronta:
T-P16-01 (spike16-00 approvato e CI positiva). Non avanzare PERSIST-02 con questo
gate pendente. Nessun consenso ordinario richiesto, AUTO_CONTINUE.

## Chiusura nativa

CI40510757138 su ceb90b7: Linux/MSVC/compare JSONL completed/success. Tutti i
gate reali soddisfatti: T-PERSIST-01 DONE. Il proof resta solo dichiarativo,
nessun backend o audit eseguibile. Nessun rinvio nascosto. Next UI16-01 già
locale verificata; PERSIST-02 richiede anche P2 completa, il cui residuo2-03
non viene qui chiuso implicitamente.
