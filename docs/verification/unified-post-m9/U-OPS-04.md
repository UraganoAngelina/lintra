# U-OPS-04 — Export locale/shared

11 novembre 2026. **IN_PROGRESS**, nessun nuovo gate PASS.
Prerequisito U-OPS-02 DONE. Preflight31e4ea4, ratifica D04 e master §2:
locale scrivibile senza ulteriore consenso; remoto/condiviso riconosciuto o
natura non determinabile richiedono avviso e scelta esplicita, poi salvataggio.

Codice effettivo esaminato: ui/src/export.rs/export_controller.rs,
src/cli/command.rs/historical.rs, report/atomic.rs e relativi test.
Overwrite, input/report protetti, cancellazione, receipt e failure parziali
sono già implementati; manca il consenso sulle destinazioni non locali.
Non viene riutilizzata la allowlist privata SQLite per negare export shared.
L'inventario U-OPS-02 distingue spool interni privati da `.part` che eredita
il confine della destinazione scelta: quel contratto resta invariato.

Implementato primo componente Rust generico `application/report_destination`:
ispezione readonly del solo insieme di output, Local/Remote/Undetermined,
autorizzazione legata ai path canonici e al tipo osservato, receipt senza path,
verifica prima dello staging e pubblicazione prevista negli adapter. Su Linux
statfs usa tipi UAPI; FUSE/overlay/altri backend non dimostrati restano
Undetermined e sono consentiti dopo avviso, non vietati. Windows usa UNC e
GetVolumePathName/GetDriveType, includendo drive mappati e mounted volume.
Nessuna promessa che un volume locale sia privato o privo di software OS di sync.

Sette unit test PASS sullo stato sorgente esatto archiviato in
U-OPS-04-policy-unit-v1.json/log.gz: locale reale senza conferma o creazione;
remoto/indeterminato non approvati e poi ammessi; binding file/formato/tipo;
più destinazioni e debug senza path; parent mancante tecnico; cambio symlink;
tipi filesystem. Nessun test è una prova di SMB/NFS montato realmente.

**Integrazione CLI/Slint ancora in corso**: nessun DONE per il solo helper.
Previsti flag CLI esplicito per il comando e checkbox legacy per l'export
selezionato, entrambi attraverso questo servizio Rust. Non è un flag di
approval storico e non modifica alcun gate normativo. Nuova GUI React dopo i
gate MIG richiesti, con stesso servizio; nessuna logica normativa/PK in JS.

Writer JSONL/CSV/XLSX, summary legacy, norme, Unknown/Partial/CAS, SQLite,
HMAC/key bundle e consuntivo 140/140 non vengono adattati per far passare test.
Nessun package nuovo. Dossier finale deve includere integrazione effettiva,
regressioni appropriate, CI Linux/Windows sul nuovo delta e limiti nativi.

## Integrazione implementata e stato del gate

CLI normale e storica, worker export e checkbox Slint ora usano la policy
comune. Contratto/funzioni/struct e limiti in docs/EXPORT_DESTINATIONS.md.
Locale libero; remoto/indeterminato con consenso per la selezione esatta;
CLI con --allow-shared-export, evento di soli conteggi. Conferma overwrite
separata e visibile anche quando si richiede il consenso destinazione.
Nessuna nuova dipendenza: fixture SMB solo sul runner Windows usa e getta.

Locale: 7 unit policy PASS; 7 test end-to-end PASS (4 writer/export,1controller
con vero filesystem opaco /proc,2CLI). I file Windows producono 0 test su
Linux per cfg, non sono considerati qualificati da quella esecuzione.
Clippy workspace/all-targets/all-features -D warnings PASS. Detector0 rilievi;
PNG chiaro del nuovo avviso osservato con vision, dati esclusivamente sintetici.

La precedente targeted-v2 applicava un filtro di nome anche ai test UI,
quindi ne eseguiva 0: quel log prova soltanto i7unit. local-e2e-v1 esegue
veramente i7test di integrazione senza tale filtro. Evidenze negative/non
eseguite conservate, nessuna reinterpretazione di0test come PASS funzionale.

**WAITING_CI**: due casi ignored Windows sono eseguiti esplicitamente in CI
su share SMB/drive mappato veri, includendo consenso,3formati byte-fedeli,
receipt, invalidità normativa preservata, cambio scope, overwrite e controller.
Mancano ancora CI del commit finale e suite locale completa sul suo codice.
Nessun DONE o G-OPS1 PASS finché quelle prove mancano. Windows11 e frontend
React restano gate separati; la ratifica brand ufficiale è documentale.

## Prima CI e correzione Windows

CI43043752821 su5fbe8cf: la fixture SMB/drive mappato è stata creata
correttamente; il job Windows è fallito in compilazione Clippy per import
delle costanti DRIVE_* dal namespace errato. Il log completo è conservato
in U-OPS-04-windows-43043752821-failed.log.gz. Corretto esclusivamente il
blocco cfg(windows): funzioni da Storage::FileSystem, costanti qualificate
wp::DRIVE_* da System::WindowsProgramming, feature già abilitata nel progetto.
Non aggiunte dipendenze, disattivati lint o cambiata la classificazione.

La prima suite locale completa è stata interrotta dall'agente durante il
linking, prima dei test, per eccessiva concorrenza di build/load60 e swap.
Evidenza workspace-v1 marcata exit130/NOT_A_TEST_PASS. Rilancio con -j2 sul
commit corretto; nessun limite del validatore o caso di test è stato cambiato.

## Qualifica locale sul commit corretto

Codice `2a5e317a007fd7e23ca247926676869215528472`, CI nativa corrente
`43049206479` ancora in esecuzione. Locale workspace-v2: **1105 pass,
0 fail,7 ignored**,571.021s; Clippy-v2 all-targets/all-features con
`-D warnings`: PASS; tooling-v1:24 test Python PASS. Tutti i receipt
conservano528 input identici, senza modifiche durante le verifiche.
Fmt e diff-check PASS. Otto fonti dataset congelate U-NORM-02 e hash del
consuntivo M9 invariati. I due test SMB Windows non sono qualificati dalla
suite Linux: serve ancora il risultato nativo del nuovo SHA.

**WAITING_CI**, nessun incremento delle task DONE. L'attesa remota consente
il preflight sequenziale indipendente U-REL-03, come previsto dalla governance;
nessuna delega o compilazione locale parallela a un'altra task.

CI43049206479 ha successivamente fallito Clippy per il solo import inutilizzato
`ComponentHandle` nel nuovo test Windows `shared_exports`. Rimosso l'import,
senza cambiare assertion o lint. Log completo conservato nel relativo
`windows-43049206479-failed.log.gz`. Le1105 prove locali restano evidenza del
codice2a5e317; per il nuovo SHA si verifica equivalenza dei sorgenti di prodotto
e si richiede comunque nuova CI Windows. Preflight U-REL-03 solo lettura,
nessuna implementazione o task DONE durante questa diagnosi.

CI corrente43050943529 sul34f5b71: il job Linux130630856915 ha fallito prima
della compilazione per DNS del mirror azure.archive.ubuntu.com durante apt,
exit100; log completo linux-43050943529-failed.log.gz conservato. Windows
130630856153 ancora in esecuzione all'ultima lettura. Non è una CI positiva
né una nuova regressione Linux dimostrata. U-PERF-01 è stata conclusa come
analisi indipendente documentale; ora U-REL-03 modifica la pubblicazione report
ed è qualificata separatamente. U-OPS-04 resta WAITING_CI finché la matrice
nativa applicabile non è positiva; niente riattribuzione di CI precedenti.

Aggiornamento finale CI43050943529: **job Windows SUCCESS**, incluse entrambe
le prove ignored effettivamente invocate su share e drive mappato. Nel log
Windows completo compaiono i due casi `... ok` alle16:09:15 e16:09:53UTC;
metadata e log gzip archiviati, SHA256 del log
57bc843775b8556515aee69219dc2559c6631cff4fdff4270f39bd6805a84683.
L'intera CI è FAILURE per apt/DNS Linux, e i confronti cross-OS sono skipped:
non vengono promossi a PASS. La prossima CI completa includerà anche il delta
U-REL-03; vecchio codice34f5b71 resta distinto da quel nuovo SHA.

Nuova matrice richiesta: CI43060623511 su7d4c02e, che include il delta
durabilità e conserva le due prove SMB/drive mappato. Task ancora WAITING_CI.
