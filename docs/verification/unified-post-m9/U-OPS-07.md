# U-OPS-07 — Gestione manuale senza retention/purge automatici

Data:11 novembre2026. Stato: **DONE per acceptance documentale**.
Prerequisiti: U-OPS-01 `c42d44e`, U-OPS-02 `10dc66b`, entrambi DONE.
Preflight: HEAD `8941bbe`, branch work/p1-foundations, working tree pulita.
U-NORM-03 sospesa sul solo gate remoto; nessuna task locale concorrente.

`docs/MANUAL_DATA_MANAGEMENT.md` attua D05: niente TTL o durate inventate,
nessuna cancellazione per età, gestione delle sole copie autorizzate,
immutabilità delle sessioni, continuità SIOC/CAD, chiave originale, archivi
chiusi e restore isolato prima di rimuovere un originale necessario. Distingue
pulizia dello spool posseduto, `.part` e journal transazionale da purge di
archivi/report. Non propone erase fisico garantito, cancellazioni SQL o un
nuovo DB per rendere artificialmente nuove le PK già trasmesse.

PERSISTENCE_OPERATIONS e LOCAL_DATA_INVENTORY collegano la procedura.
Il documento distingue percorsi attuali e root Tauri futura, BACKUP-B e
uninstaller ancora da qualificare. Nessun gate enterprise o ulteriore ratifica
D05 introdotti. Export shared rimane consentito con avviso/consenso.

## Riscontri e verifica proporzionata

Lettura del codice sucb70bee (runtime invariato da8941bbe):
`src/report/atomic.rs` possiede solo il `.part` creato con create_new; abort/Drop
non rimuovono una part preesistente. `src/cli/error_spool.rs` chiude il proprio
NamedTempFile. `ui/src/worker.rs` mantiene TempDir nel risultato; i dati non
sono cancellati per scadenza. Nessuna API di purge/TTL nell'adapter SQLite.
Le occorrenze di remove_file/remove_dir_all nei test di filesystem/XLSX sono
fixture di test, non una procedura di cancellazione dei dati dell'operatore.
Nessun archivio, report o file dell'autore aperto o rimosso.

Delta soltanto documentale e reversibile; nessun test runtime nuovo necessario.
Controlli effettivi: dipendenze/fonti/dossier con check_unified_plan.py,
collegamenti locali e git diff --check. La CI42990775714 continua a qualificare
il codice precedente cb70bee; non è attribuita a questo delta documentale,
né usata come prova Windows11/installer o due account. Le fonti originali
140/140 restano invariate. SHA della task nella cronologia di questo dossier.

G-OPS2 aggregato **NOT_VERIFIED** fino a backup/restore e prove previste.
Next READY indipendente U-OPS-03 per backend accessi/continuità; MIG resta
in attesa della baseline nativa. AUTO_CONTINUE single-agent.
