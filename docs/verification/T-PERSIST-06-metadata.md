# T-PERSIST-06 — metadati, consenso dichiarato e compatibilità

## Prerequisiti e perimetro

PERSIST05 DONE sul codice3712ed6: CI40844540287/release40844543949/probe40844049209
success.06 implementa metadati durevoli, non approvazione, servizio storico, UI,
backup completo o regole CAD/SIOC. Nessuna nuova dipendenza. Tutti i dati test
sono SYNTHETIC; chiavi fixture pubbliche, mai materiale di produzione.

## Tipi e condizioni

`MissingTraceProceedConsent` rappresenta una dichiarazione PROCEED per un insieme
non vuoto di ID trace e un istante UTC. Il costruttore controlla label, ordina
l'insieme e rifiuta duplicati. `validate_inventory` richiede che ciascun trace
sia davvero Absent nell'invio; Present(0) non è assente. Non decide quali omissioni
richiedano consenso: la policy applicativa futura verificherà tale copertura.
Il wire è tagged `decision: proceed`, senza varianti/default ABORT: decisioni
sconosciute, campi extra o timestamp invalidi vengono rifiutati. L'offset valido
viene normalizzato in UTC, come già avveniva per created_at; non è stata inventata
una restrizione nuova sui timestamp. Debug non stampa ID o valori.

`SourceSubmission::new_with_missing_trace_consent` costruisce dati prima della
protezione. Non muta un oggetto storico, non cambia lo stato dichiarato e non
produce un token di approvazione. Il costruttore precedente e tutti i campi di
`SourceSubmissionParts` rimangono compatibili. `missing_trace_consent()` restituisce
None come evidenza non registrata: ApprovedImmutable non implica PROCEED.

`SourceSubmissionWire` privato valida sia il vecchio payload sia il campo nuovo,
riutilizzando l'ordinamento e i controlli preesistenti dei Parts. Il campo opzionale
è omesso se None: vecchi byte canonici e MAC restano identici. Se Some, la
serializzazione completa già utilizzata da protect_submission lo lega al namespace,
invio e agli altri metadati; nessun nuovo algoritmo crittografico. Il test che
altera il solo timestamp della ricevuta, mantenendo il MAC, deve fallire.

La presenza/non presenza e l'hash input restano coerenti: Present(0)/Present(n)
richiedono hash, Absent non lo ha. Schema hash, reference/versioni, UTC, periodo,
namespace, fingerprint e stato dichiarato sopravvivono al restart. Nessun tipo
interpreta quarter, flow o ID campo normativi. Nessun consenso implicito e
nessuna scrittura di un ABORT: il relativo lifecycle resta applicativo.

## Formati e upgrade

Formato fisico2 dichiara la capacità di memorizzare il consenso. Nuovi archivi
nascono2; il reader06 accetta1/2. Reader05 rifiuta2 come UnsupportedStorageVersion,
invece di trattare campi futuri come archivio vuoto/corrotto. La versione normativa
rimane indipendente. Settings restituisce la versione effettiva, non una costante.

Open non migra. Dati legacy1 restano leggibili/verificabili; pubblicare il nuovo
consenso su1 dà StorageUpgradeRequired prima della scrittura. Il controllo viene
ripetuto sotto transazione al commit; lock verifica anche versioni eventualmente
cambiate, oltre alla guard non-autocommit per rollback fallito.

`upgrade_storage_v1_to_v2` è esplicito, idempotente e non distruttivo. Richiede
istanze delle versioni precedenti chiuse: non è un protocollo di coesistenza fra
versioni o postazioni. Acquisisce SQLite EXCLUSIVE, ricontrolla la versione e
modifica soltanto user_version. Snapshot/source/proof/audit/indici/generazioni
non vengono riscritti, rifirmati o caricati tutti in RAM. Conservati DELETE/EXTRA,
ACL e preflight della chiave. Contesa prima della transazione produce Busy senza
modifiche. Dopo COMMIT un errore è StorageUpgradeOutcomeUnknown: riaprire e
verificare la versione, senza dichiarare rollback per presunzione.

I failpoint sono solo cfg(test): errore prima di COMMIT, hook che rifiuta il
commit, risposta perduta, uscita di processo dentro COMMIT dopo cache_flush e
uscita dopo pubblicazione. Nei crash il reader1/2 può completare recovery e
restituisce formato/dati precedenti oppure quello committato. Non è un test di
power-loss fisico. Migrazioni ulteriori restano da definire nelle task competenti.

## Oracle legacy e test

Prima di modificare il modello è stato eseguito realmente l'adapter05, con
source-submission.json e chiave pubblica0xaa×64/keyID legacy-fixture-key. Congelati
source UTF8, firma e audit in legacy-storage-v1.json; codice helper in
legacy-storage-v1-generator.rs.txt. Il generatore rifiuta storage diverso da1:
usarlo sul checkout prodotto3712ed6. Il suo progetto isolato dipende da lintra,
rusqlite0.41.0 bundled/cache/hooks, serde_json1.0.151 e tempfile3.27.0; usare il
Cargo.lock del checkout come seed, `cargo metadata --offline` e poi
`cargo run --locked --offline -- /percorso/checkout`. Verificati pin/checksum
dell'helper contro il prodotto prima dell'esecuzione. Nessun .db è versionato.

Quell'oracle impedisce un falso test legacy costruito dal nuovo serializer: i
test inseriscono le BLOB originarie e richiedono autenticazione e byte identici,
prima/dopo upgrade e dopo restart di processo. Altra prova preserva payload,
prove e lookup delle identità e la generazione corrente. Vecchie revisioni
rimangono immutabili dopo l'aggiunta del nuovo invio con consenso.

Test mirati:5integration metadata,7protezione e8tipi positivi;16unit SQLite e
2helper ignorati/invocati dai parent positivi. Handoff attuale: gate completi
ancora da completare, nessun DONE06 prima di CI Linux/MSVC. Vedere i log target/
verification-persist-06-* e il preflight. Hash/consenso non sono anonimizzazione;
cifratura/retention/provisioning chiavi/backup prima dei dati reali restano aperti.

## Chiusura verificata — 2026-10-29

Code3e8fae4: CI40853726275 tutti4job completed/success, Linux eWindows/MSVC,
JSONL e pixel.779test principali/96suite più3helper per assetto locale,0fail/
4ignored; Clippy workspace default/all-features positivo. Evidenze native-ci.json
e local-gates.json con hash dei log.06 DONE; prossima07 macchina a stati,
AUTO_CONTINUE single-agent. La sezione precedente descrive il checkpoint iniziale.
