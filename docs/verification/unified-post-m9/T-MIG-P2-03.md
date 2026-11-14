# T-MIG-P2-03 — Run identity, ownership e macchina stati

14 novembre2026. **DONE per il gate di design**, G-MIG-2 NOT_VERIFIED.
Prerequisito P2-02 ratificato e verificato; dettagli ordinari autorizzati.

[Contratto completo](../../migration/ipc-v1/LIFECYCLE.md) e reference eseguibile
adiacente: RunRef versionato, epoch backend e ID u64 testuale, binding piano e
owner nativo; una sola validazione attiva, ack terminale prima di sostituzione,
no replay/start implicito o stato vecchio presentato come esito corrente.
Full ha un solo run per tutto il bundle. Nessun endpoint producer/PERSIST/
approve esposto, nessuna nuova API di dominio o bridge implementato.

Transizioni e matrice race collegate a CancellationToken, ValidationService
LocalOnly/Full, ProgressEvent, writer e worker reali. RunCompleted precede
pubblicazione/join: non è da solo un successo UI. Errori I/O/schema/panic/
disconnessione restano tecnici; Partial/Unknown non diventano RunStatus fittizi.

53casi deterministici PASS/exit0: cancellazione prima/dopo ingresso/confine,
start concorrente, ack, risultato tardivo, terminale doppio, cross-owner,
restart/overflow, esiti e finalizzazione fallita, vecchio report e chiusura.
[Evidenze](T-MIG-P2-03-checks.json) con hash e fonti. Sono verifiche di una
reference di design, non qualificazione di un Job Manager Rust/Tauri reale.
Nessuna suite workspace, CI o VM rieseguita. G-MIG-2 e test nativiP4/P7 restano
obbligatori. Prossima READY **T-MIG-P2-04**.39/128 DONE (30,5%).
