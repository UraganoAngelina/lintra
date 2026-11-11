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
