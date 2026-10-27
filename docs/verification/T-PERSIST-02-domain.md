# T-PERSIST-02 — identità, invii e metadati storici

## Prerequisiti e scope

P2/M1 complete: residuo2-03 codeb415d73/CI40515191339 su3cd162e, tutti job
success. PERSIST-01/ceb90b7/CI40510757138 qualificata. Applicati ADR-PERSIST-001,
ARCH45, DET166–174. Nessuna nuova norma, regola storica SIOC o scelta di backend.
UI16-02 è sospesa sul gate corretto40522626495/216e662, dopo diagnosi UTF-8.

## Tipi e correttezza

src/domain/persistence.rs contiene solo dati, costruttori verificati e codec:

- PersistenceTypeError ha cause strutturali e messaggi statici: non include
  namespace, chiavi, fingerprint o payload del chiamante. label rifiuta valore
  vuoto/solo whitespace e controlli, preservando le altre stringhe esatte.
- NamespaceValue distingue String e Integer. PersistenceNamespace ordina i nomi
  dei parametri lessicograficamente per encoding stabile, non trasforma valori,
  padding, case o tipi. Il costruttore richiede almeno un parametro e respinge
  duplicati; il decoder richiama lo stesso costruttore. Debug mostra solo conteggio.
  L’ordine tecnico dei nomi è indipendente dall’ordine di valutazione schema/run,
  che resta invariato; nomi e valori mantengono la propria associazione.
- Fingerprint conserva protection_scheme/key_id e byte opachi espliciti, senza
  default. Hex wire canonico minuscolo; decoder accetta solo coppie ASCII hex,
  preservando tutti gli zeri. Debug non espone token/key ID. same_protection_context
  impedisce di confrontare token prodotti sotto chiavi/schemi diversi.
  Non calcola HMAC/cifratura né attesta autenticità/protezione: sono dati che un
  futuro producer/preflight sicuro deve qualificare prima dello storage reale.
- PersistentRecordIdentity contiene namespace, FlowId, versione o gruppo esplicito,
  TraceId, tipo chiave e Fingerprint. Eq/Hash includono tutti questi elementi;
  digest uguale non unisce tenant/versioni/chiavi diversi. Gruppo è una dichiarazione
  di metadati, non un proof di compatibilità: il compiler01/futuro preflight devono
  verificarlo. Constructor/decoder richiedono label valide; Debug redatto.
- SessionState serializza draft/validated/approved_immutable, senza default o
  transizioni/commit simulati. TracePresence distingue Absent e Present{0}.
  SnapshotRevision è u64 esplicito: next controlla overflow, assenza di base resta
  Option::None, non una generazione inventata.
- SourceSubmissionParts è il DTO esplicito del chiamante; SourceSubmission valida
  e conserva dati immutabili: namespace/ID/schema/hash, periodo opaco, istante UTC,
  versione validator/reference, inventory trace/hash input, fingerprint e stato.
  Dataset e trace sono ordinati per ID senza cambiare ordine engine/report.
  Duplicati sono errori; Present richiede hash input, Absent lo proibisce. Un file
  vuoto può essere Present{0} con SHA256 dell’input vuoto. Nessun filename o record
  grezzo è conservato. utc normalizza l’istante in RFC3339 UTC con9cifre frazionarie;
  il periodo non viene interpretato come trimestre o regola CAD. Getter di sola
  lettura, Debug con stato/conteggio. Il tipo non è un consenso/proof di commit:
  immutabilità storica e transazioni saranno applicate nelle task successive.
- HistoricalDiff è un singolo elemento streaming con identità/base revision,
  kind e fingerprint precedente/corrente. Nessun Vec globale di record/errori.
  Inserted/Removed/Updated/Unchanged/Reactivated richiedono combinazioni coerenti;
  transizioni con base mancante o token non confrontabili sono respinte. Removed
  è una dichiarazione esplicita del futuro executor, mai dedotto da TracePresence
  Absent. Non introduce un algoritmo di riconciliazione o annullamento.
  RecordFingerprint/HistoryDiff sono alias dei nomi nominali DET, stessa semantica.

Gli ID neutri FlowId/SchemaVersion/TraceId/DatasetId/Version e SchemaKey aggiungono
serde senza cambiare spelling/invarianti esistenti. Sha256Digest aggiunge decoder
64ASCIIhex verificato prima degli indici, nessun panic Unicode o default. I vecchi
serializer/golden rimangono invariati. Nessun compiled proof diventa deserializzabile.
Moduli/domain/lib esportano soltanto API additive; ContextStore/engine restano
separati, nessuna importazione inversa domain→engine/application.

## Test ed evidenze

tests/persistence_types.rs: otto test con metadata sintetici. Namespace canonico
ma value/type esatti, dupe/empty/control e redaction; token hex e contesto chiave;
identità separate per scope; presence/state/revision overflow; source roundtrip e
UTC equivalente con offset; inventory/hash incoerenti e duplicati respinti; diff
positive/negative; PK estratta dall’API esistente build_join_key col vero schema
compilato e ordine record_key, con padding/UTF-8/missing. Nessun hasher nuovo per
trasformare chiavi raw in identità persistibili: i token sono fixture opache.

Golden tests/fixtures/persistence/source-submission.json scritto separatamente
con Python da valori attesi espliciti, non dall’output Rust; confronto byte per
byte dopo canonical ordering. Non rappresenta un audit di una validazione reale.
Otto mirati passati. Check/fmt positivi; Clippy ha segnalato la forma modulo2,
corretta a is_multiple_of senza suppression. Gate completi ancora in esecuzione.

## Privacy, rinvii e handoff

Nessun record sanitario, CF, chiave segreta, filesystem backend o nuovo crate.
Fingerprint non è un certificato di anonimizzazione: nessuna esecuzione storica
è abilitata. Richiesti prima dell’uso reale protector/chiavi/ACL/retention e
preflight sicuro; nessuna scelta algoritmica effettuata in questa task.
I codec sono contratti dei nuovi dati; envelope/format storage definitivo restano
al backend. Manca ancora la CI del nuovo commit: IN_PROGRESS, non DONE.
Next PERSIST-03 solo dopo02 verificata; UI16-03 dopo gate16-02. AUTO_CONTINUE.

### Isolamento fixture tecniche

Il runner dei golden di flusso respinge correttamente profili senza producer.
La nuova fixture di serializzazione metadata è in tests/fixtures/persistence,
confrontata dal suo test dedicato: nessuna modifica al runner/golden normativi.
La regressione iniziale ha rilevato la collocazione errata, ora corretta.

### Audit e decoding strict

SourceSubmission conserva anche validation_mode e validation_status espliciti
dal chiamante: ADR §5 richiede il risultato della validazione e il suo scope
non va dedotto dal solo stato sessione. RunStatus aggiunge serde snake_case,
senza modificare i serializer manuali dei report esistenti. Nessuna approvazione
è attestata dal DTO: autorizzazione/mutazioni rimangono ai service transazionali.
La regressione absent+record_count ha rilevato il comportamento Serde dei tagged
unit variant: PresenceWire usa un empty struct variant per il decode strict,
mantenendo il wire output Absent invariato e respingendo dati contraddittori.

## Gate locali finali completati

Fmt/Clippy workspace all-targets default/all-features positivi, senzaallow.
711/719test,88suite,0fail/2ignored, inclusi tutti i golden precedenti e quello
metadata indipendente aggiornato con modo/esito reali. Test strictPresence
ripassato: absent con record_count è errore, output Absent invariato.
Nessun nuovo crate/feature Cargo o backend/crypto; interfacce engine invariate.
File e payload sintetici pronti alla pubblicazione; attende CI, non DONE.
