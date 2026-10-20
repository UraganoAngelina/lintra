# Audit input e summary — T-P5-05 / CNF-AUDIT-01 approvata

L’audit è tecnico, generico e separato da schema_hash e identità di record.
Non contiene record sanitari, identificativi paziente o percorsi assoluti.

## Flusso e fonte autorevole

1. Creare InputAuditSession e ValidationSummaryBuilder all’avvio del run.
2. session.open(trace_id, &Path) apre il file e legge la dimensione dal
   medesimo handle. Il basename deve essere UTF-8. Per una sorgente Read
   già aperta, wrap riceve esplicitamente nome e dimensione iniziale.
3. Comporre source -> AuditedReader -> preflight_bom -> BufReader ->
   RecordReader. Hash e contatore vedono i byte originali prima di BOM,
   newline, slicing o decoding, senza doppio conteggio del prefisso replay.
4. Una sola sessione mantiene gli stati di tutti i reader in ordine di
   registrazione. snapshot fornisce copie diagnostiche di valore, senza accesso in scrittura
   allo stato autorevole: non sono una seconda fonte runtime mutabile e non possono essere iniettate
   in RunMetadata prima di finish. Nessun audit è creato per trace assenti.
5. Su stop normale o controllato, session.finish(metadata) consuma la
   sessione e congela tutti i reader senza drain o letture supplementari.
   Richiede metadata.inputs vuoto, poi lo popola dallo stato autorevole.
   I reader ancora posseduti dal chiamante non possono più acquisire byte.
6. Conservare il Copy metadata.started_at per il writer; il builder consuma
   RunMetadata e muove inputs in ValidationSummary senza ricostruire una
   lista indipendente. Nessuno stato runtime mutabile rimane nei metadata.

Il mutex protegge solo lo stato in memoria e comprende il read in corso:
finish non può congelare metadati a metà acquisizione. Non è un lock del
file e non viene creato uno snapshot della sorgente. Non sono esposti Seek,
get_mut o into_inner per aggirare l’audit del reader.

## EOF, errori e cancellazione

bytes_read conta soltanto byte restituiti con successo dal Read sottostante,
compreso read-ahead. Non conta i byte validati, i tentativi Interrupted
o contenuti eventualmente scritti nel buffer da una chiamata fallita.
Ogni incremento è checked. Un read con buffer vuoto non osserva EOF.

Complete viene prodotto soltanto dopo un read non vuoto che restituisce
zero, con bytes_read == file_size_bytes. Raggiungere la dimensione senza
osservare EOF lascia Incomplete. A EOF un disaccordo di dimensione è un
errore tecnico e impedisce finish dei metadata. I guasti I/O conservano
la causa originale e il kind attraverso InputError: nessun summary definitivo.
Anche ignorando il primo errore, la sessione non permette la finalizzazione.

La cancellazione è orchestrata dal futuro service: smettere di chiamare
read, congelare la sessione e marcare il builder Cancelled. Non vi sono
letture in Drop o finish per completare il digest. Cancelled/truncated
possono avere audit completo o incompleto secondo l’effettivo EOF.
Non si espone il digest del prefisso. Un run normale rifiuta audit incompleti;
record elaborati senza alcun audit sono un errore di metadata.

Dimensione e hash descrivono i byte osservati. Non garantiscono l’assenza
di modifiche concorrenti che preservino la dimensione; nessun meccanismo
aggiuntivo di locking/snapshot sorgenti è introdotto. EOF congela l’impronta
osservata, non certifica l’immutabilità futura del file.

## Builder e confini

record_processed/record_valid/record_invalid/error_emitted/rule_skipped
sono checked e restituiscono Result; un errore rende il builder terminale.
finish(metadata) restituisce Result<ValidationSummary, SummaryBuildError>.
Questa precisazione idiomatica al DET §58 rende gli overflow/metadati
inconsistenti guasti espliciti anziché panic, saturazione o summary falsi.
Ownership e responsabilità originali preservate (DET §147).

Cancelled ha precedenza; altrimenti errori o record invalidi producono
CompletedInvalid, assenza di entrambi CompletedValid. truncate conserva
il flag indipendente, non simula una cancellazione e non attiva limiti
operativi dell’engine ancora da implementare. Classificazioni e contatori
sono verificati al finish; elapsed usa il tempo monotono di vita del builder.
Schema/version/reference versions sono metadata espliciti del chiamante:
il calcolo del canonical schema_hash resta nella task dedicata, non è
sostituito dal digest di un input.

## Documento errors.summary.json

write_summary(writer, &summary, started_at) scrive un JSON UTF-8 compatto
con LF finale e flush, separato da errors.jsonl. Non legge il clock/locale
dell’host. Prima del primo byte verifica summary e rappresentabilità del
timestamp. Il writer resta del chiamante: pubblicazione atomica .part e
rollback sono T-P5-06, non implementati qui. Un Err non è un report completo.

Ordine dei campi: result (VALID/INVALID/CANCELLED), flow, schema_version,
scope, records_processed, valid_records, invalid_records, total_errors,
errors_by_code, skipped_rules, truncated, validator_version, schema_hash,
reference_data_versions, started_at, elapsed, inputs.

Mappe per simbolo/dataset ordinate lessicograficamente; scope e inputs
preservano l’ordine selezionato. started_at è UTC RFC3339 con Z, elapsed
ha seconds/nanoseconds per preservare la precisione senza float.
Ogni input usa il payload approvato: trace_id/file_name/file_size_bytes/
bytes_read/completion. completion è {kind:complete,sha256:<64 hex minuscoli>}
oppure {kind:incomplete}, senza alcun digest nel secondo caso.

I payload pubblici di dominio restano valori: validate rileva incoerenze
strutturali, ma la prova di EOF/hash deriva dal reader autorevole, non
dalla costruzione manuale di un InputAuditCompletion::Complete.
I nomi devono essere basename portabili: niente separatori, drive, NUL o
componenti . / ..; nessuna conversione to_string_lossy.

Fixture approvate docs/proposals/input-audit e regressioni summary.complete/
summary.cancelled sono sintetiche. La fixture del writer fissa solo elapsed
per il confronto byte: gli audit vengono sempre ottenuti da letture reali.
