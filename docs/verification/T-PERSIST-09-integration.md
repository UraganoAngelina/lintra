# T-PERSIST-09 — integrazione Full e archivio audit-only

31 ottobre 2026. Prerequisiti ed esclusioni nel [preflight](T-PERSIST-09-preflight.md).
Il codice riprende il lavoro parziale presente sulla base `fafac99`; non modifica
schemi approvati, UI, CLI, formati del database, dipendenze o file esterni in
`docs/migration`. I gate effettivi sono registrati nel [ledger locale](T-PERSIST-09-local-gates.json);
questa relazione non sostituisce la CI Linux/MSVC.

## Comportamento consegnato

Il chiamante costruisce `HistoricalValidationService` con un `ValidationService`,
un repository persistente e un protector espliciti. `HistoricalValidationRequest`
contiene la richiesta Full/Flow, parametri tipizzati e un identificativo di invio
non sensibile. Nessun percorso, organizzazione, periodo o chiave viene scoperto
implicitamente. Le API ordinarie conservano il compilatore e il wiring precedenti.

`validate` esegue la validazione reale, prepara un candidato protetto, confronta
lo storico ed emette diff uno alla volta. Restituisce `ValidationOnly` per run
invalidi, cancellati durante il motore o inconclusivi: nessuna capability di
approvazione accompagna questi esiti. Restituisce `Preview` solo dopo confronto
e sink riusciti. Un errore tecnico resta un `HistoricalValidationError` con causa
tipizzata; nessun fallimento di storage viene convertito in violazione normativa.

La preview non scrive il database. `approve_commit` richiede una seconda azione
esplicita e una callback che pubblichi tutti i report richiesti. Un errore della
callback precede l'apertura della transazione. CAS, rollback, immutabilità,
idempotenza e riconciliazione di commit ambiguo sono quelli già qualificati
del coordinatore PERSIST07, non un protocollo transazionale parallelo.

## Compiler e registry: una sola sorgente

`schema/historical.rs` introduce due prodotti di sola lettura:

- `CompiledHistoricalFlowSchema` tiene insieme schema eseguibile, profilo di
  persistenza e definizioni delle proiezioni; il chiamante non può costruirlo
  deserializzando JSON o sostituendone una parte.
- `RecordByteProjectionDefinition` espone trace, descrittore e SHA-256 canonico.
  Sono metadati dello schema, senza record o chiavi crittografiche.

`compile_historical_audit` compila prima le policy e accetta solo audit dichiarato,
senza snapshot, identity invariants o regole temporali non implementate.
`compile_scoped` è un ingresso interno: abilita la sola capability audit e
conserva la policy nel prodotto compilato. Non cancella dichiarazioni dal raw.
Il compilatore pubblico ordinario passa `None` e continua a rifiutare queste
capability. Le altre condizioni di rifiuto restano attive anche nel percorso nuovo.

`resolve_historical_audit` carica esattamente la chiave selezionata, ne calcola
l'hash prima del binding encoding e usa quella stessa istanza raw per tutti i
prodotti. Risolve eventuali membri di un gruppo solo per versioni dichiarate;
il compiler ne qualifica i fingerprint. Verifica poi chiave/hash dei prodotti
prima di restituire `ResolvedHistoricalSchema`. Nessuna seconda lettura della
source corrente può sostituirla dopo la validazione.

`canonical_value_bytes` riusa l'ordinamento ricorsivo già usato dal fingerprint
schema. Il codec canonico normativo non cambia. Le definizioni includono versione
del descrittore e del formato, encoding effettivo, trace, lunghezza record e
layout/missing dei campi; i descrittori di campo sono ordinati per ID e le
definizioni per trace. Un encoding/layout differente cambia il contesto del
fingerprint invece di simulare equivalenza fra proiezioni.
Il contratto corrente del compilatore eseguibile accetta solo UTF-8: altri
binding encoding continuano a produrre `UnsupportedEncoding`. Il descrittore
conserva comunque il binding effettivamente qualificato, senza estendere qui
il supporto alle codifiche.

## Motore: evidenza borrowing durante la passata reale

`FlowRecordObserver` è un hook interno neutro. `ProjectionCollector` gli passa
record e `JoinKey` appena estratta per il controllo di unicità, prima che il
buffer di input venga riusato. Il motore non importa storage, protector o SQL.
La normale `FlowValidator::validate` delega con observer assente; l'ordinamento
degli errori e il controllo `E_DUPLICATE_RECORD_KEY` rimangono gli stessi.

`ProtectedCollector` nell'application conserva solo `StoredIdentity`. Per ogni
trace audit il suo lookup contiene scope e hash di definizione compilati.
L'observer applica HMAC alla PK logical canonica e al record fisico borrowing,
senza terminatore di riga. Non salva una copia della riga. Eventuali token
raccolti da record successivamente invalidi rimangono privati e vengono eliminati
quando l'esito non permette una preview.

La proiezione di confronto è **byte-esatta**, non un'equivalenza clinica:
modifiche di padding o byte non appartenenti alla PK possono produrre `Updated`.
Si conserva soltanto il token HMAC, non i byte. La definizione versionata rende
esplicita questa scelta e il confronto non genera nuovi errori normativi.

`protect_record_bytes` in `persistence/protection.rs` produce gli stessi frame
della proiezione JoinKey a componente singola: lunghezza esterna, lunghezza
interna, byte. Controlla conversione a u64 e somma, poi condivide `finish_record`
con la primitiva precedente. Scheme v1, binding della PK/definizione e attestazione
privata sono preservati; un test confronta i due percorsi byte per byte.

## Preflight, provenienza e callback terminali

`load_protection_audit` è una porta SQL-free additiva con default fail-closed.
SQLite restituisce l'audit già presente nella metadata e verifica il protector
dell'adapter; il fake richiede inizializzazione esplicita con audit qualificato.
Il nuovo servizio confronta sempre questo audit con il proprio protector prima
di aprire input, anche in un namespace vuoto. Una chiave diversa non crea
silenziosamente un nuovo insieme di identità.

`bind_audit_namespace` riusa il binding tipizzato di PERSIST07: parametri ignoti,
tipi errati, valori fuori dominio o namespace mancanti falliscono prima dei file.
Il servizio converte poi i valori già verificati nel formato del motore Full.
L'esito contiene gli stessi valori canonici nel summary, verificati di nuovo
da `ApprovalDraft::validated` insieme a hash input, schema, reference, clock,
conteggi e stato. I dati della source derivano dal run effettivo, mai da input
riaperti o da un summary sintetico fornito dall'adapter.

`source_time` usa aritmetica controllata e `DateTime::from_timestamp`: conserva
le frazioni prima dell'epoch e rifiuta clock fuori dal calendario rappresentabile,
senza panic di conversione. `draft_fingerprint` produce un seed HMAC distinto,
legato allo schema; `protect_submission` lo sigilla con tutta la provenienza e
lo stato reale prima che l'adapter veda il candidato.

`DeferredSink` inoltra subito gli errori ma differisce il summary; l'observer di
progresso differisce solo `RunCompleted`. Un audit fallito non lascia un summary
di successo o evento terminale di successo. Gli output possono essere parziali
su `Err`: la pubblicazione atomica e il cleanup restano responsabilità del
chiamante. Il servizio verifica generazione e cancellazione prima del termine.

## Archivio audit senza trimestre

`ApprovalBinding` distingue binding trimestrale preesistente e `AuditBinding`.
L'archivio audit lega flow, namespace e submission ID; usa il prefisso separato
`lintra-audit-session-v1:` nella metadata opaca già disponibile. Non attribuisce
anno/trimestre SIOC e non cambia un byte del codec trimestrale preesistente.
Il decode richiede roundtrip canonico e corrispondenza con la source.

Un file assente non richiede il consenso trimestrale e non sostituisce la sua
baseline presente. Il consenso snapshot passato a un archivio audit è rifiutato,
non ignorato. Un file presente con zero record resta invece presente e può
produrre scomparse nell'audit. `ApprovalPreview` conserva repository e candidato
privati; una nuova preview richiede che la generazione di approvazione coincida
con quella del confronto. Replay già approvato resta idempotente. Le vecchie
codifiche, i test di sessione e la gestione del commit ambiguo sono conservati.

## Accettazione sintetica e limiti

`tests/sioc_persistence.rs` deriva in memoria una fixture dichiaratamente di test
dal componente referenziale già tracciato, aggiungendo le policy audit della
source operativa senza modificare i file originali. Verifica le cinque tuple
PK contro la source e l'elenco ADR001: `(1.1,1.17,1.18)`, `(2.1,2.49,2.50)`,
`(3.1,3.2,3.35,3.36)`, `(4.1,4.7,4.8,4.19.1)`, `(4.20,4.22)`.
Le righe sintetiche sono sette: una per 1/2/3 e due per 4.1/4.2.

Copertura: duplicati in ciascuna trace, seconda trasmissione identica senza
errore cross-run, namespace separati, modifica non-PK, token protetti/roundtrip
SQLite e assenza di righe/chiave nel DB, caricamento source una sola volta,
report fallito, cancellazione reale, input mancante, chiave errata prima dei file,
CAS, replay esatto e invio approvato modificato, file opzionale assente contro
presente vuoto, descrittore deterministico e rifiuto degli encoding non supportati. I test precedenti
PERSIST04/07/08 e quelli dell'adapter coprono anche attestazione, guasti di
storage e riconciliazione dopo commit ambiguo.

Il profilo operativo SIOC completo mantiene i propri controlli/capability
non ancora supportati e continua a essere rifiutato quando non eseguibile:
nessuna certificazione normativa completa è dedotta da questa accettazione.
DEC-SIOC-HIST-01 resta aperta; le PK ripetute fra invii sono soltanto audit.
Storage format e migrazioni non cambiano; i nuovi dati usano il codec qualificato.
La memoria dei token è proporzionale al candidato, in accordo con il port owned
esistente: non si rivendica uno storage cursor o una soglia prestazionale nuova.
Provider di chiavi reali, retention e backup rimangono nei rispettivi perimetri.

## Gate locali effettivi, 31 ottobre 2026

Tutti i 9 comandi finali hanno exit 0; 257 hash sorgenti uguali prima/dopo. Il ledger
conserva anche i tentativi precedenti: due aspettative errate delle fixture
negative (encoding e proprietà JSON richiesta) e l'interruzione controllata
per rendere il test clock rappresentabile su Windows e ridurre la contesa.
Il campione finale usa 700 ns, multiplo del reticolo 100 ns di Windows verificato
nel sorgente std della toolchain 1.99.1. Nessun controllo produttivo è attenuato.

| Controllo | Esito effettivo |
|---|---|
| Quattro suite mirate |49 pass, 0 fail: 11 SIOC + 14 audit + 15 session + 9 protezione |
| Clock pre-epoch/fuori calendario |1 pass, 0 fail |
| fmt |exit 0 |
| Clippy workspace default/all-features |entrambi exit 0, warnings negati |
| Workspace default |846 pass, 0 fail, 4 ignored, 109 riepiloghi |
| Workspace all-features |846 pass, 0 fail, 4 ignored, 109 riepiloghi |
| Core senza bundle |803 pass, 0 fail, 4 ignored, 92 riepiloghi |
| Confini workspace/core |exit 0 |

I conteggi includono helper di processo e doctest effettivamente riportati dai
runner. La CI Linux/MSVC non è ancora attribuita a questo codice; pubblicazione
e gate nativo sono i passi successivi obbligatori prima di DONE.
