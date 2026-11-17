# Blocco P4-04/05/06 — worker LocalOnly e lifecycle

17 novembre 2026. Mandato autore: ripresa del workflow monoagente, un solo
blocco funzionale implementato prima della verifica e dell'attivazione CI.
Baseline locale `d78e8d0`, primo commit `f6318d6`. Pubblicazione isolata sul
branch `work/p4-native-lifecycle-publish`, parent remoto `fa57d35`: esclusi
i tre commit locali precedenti di governance/configurazione. Sorgenti runtime,
test, fixture, manifest e workflow byte-identici alla verifica locale; incluse
le due piccole correzioni native già presenti in d78e8d0. Nessuna modifica a
hook, configurazione aiproto, card o istruzioni degli agenti.
Prerequisiti: P4-01/02/03 presenti nella baseline, G-MIG-3 qualificato.
ID, dipendenze e acceptance del Piano Unificato restano invariati.

## Worker ed esito (P4-04)

Il worker parziale della baseline viene completato mantenendo un solo `Job`
e un `JoinHandle` posseduto. `Configuration::job` accetta esclusivamente un
piano preparato LocalOnly dello stesso mode: piano assente/stale e Full
sono rifiutati prima dello spawn. `PreparedIntent` conserva la richiesta,
i parametri tipizzati, lo schema raw acquisito e l'audit del bundle. Cambiare
selezione nella form non modifica il run già catturato.

`execute` costruisce repository/registry dal raw congelato, risolve l'hash,
chiama `ValidationService::validate_with_typed_parameters` e usa gli stessi
`JsonlSink`, `AtomicReportFile` e `PartialOutputPolicy::Delete` della library.
Nessuna regola normativa nuova. La summary, la completezza e il RunStatus
provengono dal servizio reale. Prima di pubblicare si terminano il writer e
il controllo dell'identità schema. Cancelled/truncated non pubblicano report;
Partial senza truncation mantiene la propria semantica.

`Job::poll` esegue join solo dopo `is_finished`: nessuna attesa di I/O del
worker nell'event loop durante il polling. Il terminale è il risultato
ritornato dal thread, mai il callback `RunCompleted`. Panic, errore servizio,
flush, pubblicazione o schema sono errori tecnici; non diventano invalidità
normative. Il terminale resta leggibile fino all'ack, che è idempotente e
non autorizza persistenza o approvazione clinica. Start occupato è Busy;
start prima dell'ack o con l'ultimo request_id riutilizzato è InvalidState.
Run ID incrementato senza wrap; overflow non sostituisce lo stato.

## Progresso e cancel (P4-05)

`Progress` mantiene un solo campione latest. `update` consuma gli eventi reali,
controlla fase e contatori monotoni, identità trace, overflow della sequence
e sigillo dopo RunCompleted. Nessuna serializzazione per record, coda,
percentuale o tempo stimato. Le letture copiano il campione sotto un lock breve
e serializzano fuori dal lock; i trace ID sono risolti soltanto in Rust verso
riferimenti `trace_<indice>` dello scope immutabile del run.

`validation_progress` restituisce i sette campi ratificati: versione, RunRef,
sequence, phase, trace_ref, trace_records, cross_records. Sequence e conteggi
u64 restano testo canonico. Un evento impossibile sigilla il solo progresso:
la lettura del terminale e l'ack rimangono indipendenti, anche con mutex del
progresso avvelenato. Il client TS controlla shape/versioni/identità/limiti e
`acceptRunProgress` scarta campioni vecchi, ripetuti o successivi al terminale.

`validation_cancel` serializza la richiesta con il confine del servizio:
requested/already_requested prima del sigillo; too_late dopo RunCompleted;
already_terminal dopo join. Il token non fabbrica un esito Cancelled. Se
l'ingresso worker trova già il token, ritorna not_started; se il servizio è
entrato, si attende il suo risultato reale. Cancel/ack/progress richiedono
il RunRef corrente; un riferimento precedente o di un'altra istanza è stale.

Il dispatcher mantiene il controllo di origine/finestra della baseline e il
singolo ingresso senza code. Non sono ancora presenti query lente o bulk;
la qualifica di saturazione del trasporto Tauri rimane al gate P7 previsto.

## Report e chiusura (P4-06)

`Report` possiede il TempDir e un Arc del PreparedIntent: schema, input,
parametri e provenienza restano del run che ha prodotto il JSONL. `last_report`
conserva la coppia RunRef/report; un successivo errore o cancel non attribuisce
il vecchio report al nuovo esito. Mantiene al massimo il report precedente e
quello corrente, senza registro crescente. Gli Arc ne delimitano il lifetime;
Drop rimuove i soli temporanei. Nessun path, record, summary completa o JSONL
è aggiunto ai DTO. Paging e dettagli appartengono a P4-07/08/09.

Close richiede cancel. `AppState::shutdown`, chiamato da `RunEvent::Exit`,
acquisisce e rimuove la configurazione anche se il mutex è avvelenato,
rilascia il gate e cancella/attende il worker tramite Drop prima di liberare
i report. Nessun thread staccato; nessuna promessa di join istantaneo o
recovery dopo kill. Il temporaneo resta nella root OS già usata: il requisito
per-account U-OPS-03 rimane WAITING_INTEGRATION, senza DONE anticipato.

## Verifica e limiti

Test con dati sintetici: servizio reale LocalOnly valido/invalid, confronto
JSONL con chiamata diretta (solo started_at/elapsed runtime esclusi), golden
esistenti byte per byte, cancel/truncation, ack/Busy/stale/overflow, thread
reale con finalizzazione trattenuta per le race, consumer fermo su 10.000
aggiornamenti, lock avvelenato, join durante Drop e rimozione del TempDir.
I test negativi TS rifiutano path/bulk, versioni, run estranei e conteggi
non canonici. I log e la ricevuta riportano gli esiti effettivi.

Review Ponytail monoagente sul flusso reale: corretto il terminale nascosto
da un lock progresso avvelenato e aggiunta la regressione. Primo Clippy FAIL
per if annidato, corretto senza disabilitare warning; log conservato.

Il flag app_info.validation_available resta false perché la shell P3 non ha
ancora i controlli P5. Nessuna nuova UI o persistenza esposta. Full rimane
rifiutato fino a P4-11; G-MIG-4 e G-REL restano aperti. Nessuna nuova suite
workspace, VM, installer, dipendenza o modifica di CI/golden/core/CLI/Slint.
