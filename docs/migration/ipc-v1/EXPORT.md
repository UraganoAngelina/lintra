# Export IPC v1 — P2-07

Design14 novembre2026; schemaVersion1 su envelope e DTO, cap1MiB/depth32.
Riusa D04/ADR-MIG-003 e [policy vigente](../../EXPORT_DESTINATIONS.md).
Non modifica CLI, writer canonici, report normativi o consenso storico C1.

## Percorso e autorità

prepare accetta esclusivamente schemaVersion, report_ref opaco e format
jsonl/csv/xlsx. Il broker Rust risolve il report autorizzato, cattura Arc e
summary del medesimo RunRef e avvia la selezione nativa. Path, nomi file,
directory e autorizzazioni non transitano nel renderer. Il dialogo nativo
mostra percorso e l’intero insieme report+summary scelto dall’operatore.
Non esiste un comando write_file, né path/overwrite/shared_confirmed nel DTO.

Rust usa preflight e ReportDestinationAssessment::inspect su entrambi i file:
destinazione assoluta, estensione/formato coerenti, parent accessibile, file
regolari, nessun symlink/alias protetto, nessun input o report privato
sovrascritto. Per Full devono essere protetti tutti gli input catturati:
l’attuale preflight Slint protegge input_path singolo; l’adapter Full dovrà
integrare il bundle senza esportare i path. Questo requisito non è già
implementato o qualificato dalla reference P2.

Local accessibile non richiede conferma privacy aggiuntiva. Remote o
Undetermined richiedono warning e consenso esplicito per questa selezione,
poi sono consentiti. Overwrite richiede una scelta distinta anche se esiste
solo il sidecar summary. Un consenso non sostituisce l’altro. Le conferme
sono raccolte dal broker/dialogo nativo, non da un booleano IPC o dall’ID
della richiesta. Cancel/decline non creano staging o autorizzazioni.

Il plan_ref export_ref opaco è legato a owner nativo, report/RunRef immutabile,
formato, insieme ordinato dei path e classificazioni. Rust conserva
AuthorizedReportDestinations; il renderer riceve soltanto export_ref,
report_ref, RunRef, format, state e destination receipt di conteggi testuali.
Nessun token C1/chiave HMAC, bypass globale o permesso PERSIST/approve.
Cambio selezione/formato, chiusura o owner diverso invalida il piano.
Conferme native tardive non possono resuscitare una selezione chiusa.

## Esecuzione e risultato

execute accetta solo schemaVersion/export_ref. Una job export posseduta per
istanza; piano consumato prima dell’esecuzione, senza replay automatico.
Rust ripete preflight e verify_unchanged prima dello staging e prima delle
pubblicazioni, usando lo stesso report catturato anche se cambia il form/run
visualizzato. Il callback writer della reference rappresenta questo servizio
nativo completo; non implementa la seconda verifica o i filesystem in Python.
La cache del piano non costituisce una garanzia contro modifiche ostili fra
syscall; restano i limiti TOCTOU espliciti del servizio esistente.

Report cancellati o truncated non sono esportabili. Partial normativo è
distinto da truncated: non viene rifiutato solo perché completeness è Partial.
L’export comprende il report intero catturato, indipendentemente dai filtri
visivi. Replay JSONL/CSV e XlsxReportWriter restano Rust, senza download blob
massivi nel browser. AtomicReportFile pubblica ciascun file separatamente.
Non esiste una transazione multi-file o report+SQLite e non si promette rollback
di file già pubblicati.

ExportResult DTO versionato: export_ref/report_ref/RunRef/format, state
succeeded/failed/cancelled, published_roles (report/summary), cleanup_failed,
durability_confirmed, destination con conteggi e warning_acknowledged, error
tecnico versionato quando applicabile. Sono metadati della job export, non
CompletedValid/CompletedInvalid della validazione. Successo richiede entrambi
i file pubblicati e nessun errore/cancel/cleanup/durabilità pendente.

ExportFailure::published e ReportError::output_was_published conservano anche
rename riuscito seguito da sync fallito. Pubblicazione parziale, cancellazione
e cleanup fallito restano espliciti; non si reinterpretano come zero file o
successo. In un guasto inatteso dell’adapter o perdita di risposta IPC,
l’errore tecnico non attesta assenza di pubblicazione: nessun retry cieco.
Il broker mantiene la job/receipt nativa per riconciliare l’esito e riaprire
una selezione nativa se serve. Dettagli I/O contenenti path restano nel
contesto nativo controllato, senza raw Error::to_string in IPC/console.

## Verifica attribuibile

La reference export_reference.py è collegata al codec/RunRef comuni;
verify_export.py usa solo callback nativi e percorsi sintetici non scritti.
60casi PASS: direct invoke forgiati, owner/versione, share/overwrite indipendenti,
path/classificazione/esistenza cambiati, stale/replay/chiusura tardiva, report
catturato, tre formati e receipt parziali/fallite/cancellate. Corretto un catch
che riclassificava stale come errore generico; il run fallito è conservato
nella receipt. Nessuna prova di dialogo reale, writer byte-parity, rename o
durabilità nativa aggiunta: sono requisiti dell’integrazione export/P7/P8,
distinti dalle qualifiche core già esistenti. G-MIG-2 rimane aperto.
