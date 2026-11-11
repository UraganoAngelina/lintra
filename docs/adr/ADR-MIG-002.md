# ADR-MIG-002 — Target GUI e distribuzione offline interna

Decision owner: autore Lintra. Fonte: D14/DEC-MIG-02 e mandato unificato
ricevuti il10 novembre2026, recepiti in ADR-POST-M9-001/U-GOV-03 `2cd4bdf`.
Trascrizione11 novembre2026, T-MIG-P0-05.
Stato: **USER_APPROVED / REPOSITORY_RATIFIED**, qualifica GUI **NOT_VERIFIED**.
Riferimenti: [ratifica](../unified-post-m9/RATIFICATION_2026-11-10.md),
[master](../unified-post-m9/MASTER_PLAN_UNIFICATO.md),
[ADR-MIG-001](ADR-MIG-001.md). Nessuna nuova firma o ratifica globale richiesta.

## Matrice di applicabilità

| Componente | Target e distribuzione | Prova ancora richiesta |
| --- | --- | --- |
| Core/library e CLI | Matrice Linux/Windows x86_64 e contratti N-19/N-20/N-23 originali | CI nativa, release/artifact e golden esistenti preservati |
| Nuova GUI per colleghi | **Windows11 x86_64**, MSVC nativo, NSIS.exe **perMachine**, offline | Spike Win11 clean-room, installer/run/update/uninstall e gate MIG |
| Nuova GUI Linux | Solo build/smoke personale **CachyOS/Hyprland/Wayland** | Ambiente/librerie reali e smoke sul PC personale |
| Vecchia GUI Slint | Resta fino a parità e rollback; propri contratti/gate | Nessun ritiro o decommission anticipato |

Non si estende il supporto GUI a Windows10, Server, altri OS o distribuzioni
Linux per analogia. Non si produce una release Linux GUI ufficiale né si
promette portabilità universale AppImage/deb. Il nuovo target GUI non restringe
retroattivamente la policy della CLI o i target della library.

## Windows: contratto di distribuzione

L'installer NSIS installa l'app per macchina in Program Files, con l'elevazione
necessaria all'installazione. L'esecuzione ordinaria è **non elevata**. Il
prodotto non richiede UAC per validare o scrivere dati dell'account autorizzato.
WebView2 è fornito tramite **offlineInstaller** completo: niente bootstrapper
che scarica dipendenze dalla rete. Payload, versione, provenienza, licenze e
hash effettivi devono essere verificati/pinnati nelle task tecniche; questo
ADR non inventa un minimo runtime o afferma che l'installer sia già disponibile.

La clean-room Win11 senza WebView2 deve installare e avviare offline; va provato
anche il percorso con runtime compatibile già presente e l'errore con prerequisito
incompatibile/mancante, senza riparazione online implicita. Il processo app e
le risorse HTML/CSS/font restano locali; niente CDN, telemetry o auto-updater.
Le destinazioni filesystem shared scelte dall'operatore seguono il consenso
D04: non autorizzano servizi HTTP o canali di aggiornamento applicativi.

Dati mutabili per account in **%LOCALAPPDATA%/Lintra**: archivi, report, config,
spool/cache di lavoro sotto controlli OS. Program Files contiene app/asset,
non DB o chiave operativa condivisa in chiaro accessibile a tutti gli utenti.
La chiave comune originale32B/Key ID viene provisionata privatamente, senza
nuove chiavi generate, password custom, cifratura o rotazione automatica.
Il profilo WebView/cache e i temporanei devono rispettare lo stesso isolamento;
il renderer non conserva PHI in web storage persistente.

Due account OS distinti devono poter eseguire l'app con dati propri senza
leggere/sovrascrivere quelli altrui. È un test reale da eseguire; due directory
di fixture o il runner Windows2025 amministratore non lo sostituiscono.
Non si abilita condivisione del database o sincronizzazione fra account.

Update **manuale e offline**: installer della nuova versione, continuità della
chiave e conservazione di DB/report. Nessuna migrazione silenziosa di path,
namespace, key material o schema SQLite. Prima di upgrade persistente
potenzialmente irreversibile: BACKUP-B verificato, altrimenti stop; restore
isolato e prove proprie U-OPS-06/08. Non è una promessa di wizard già disponibile.

Uninstall rimuove software/asset installati e conserva DB, report, configurazione
e materiale chiave dell'account. Nessun wipe o purge implicito; la gestione
manuale autorizzata è [separata](../MANUAL_DATA_MANAGEMENT.md). Le prove includono
sentinelle di dati e chiave dopo update, uninstall e reinstall, failure/spazio/
permessi e assenza di overwrite accidentale. Il rollback usa l'artefatto vecchio
qualificato e la compatibilità persistente provata, non una modifica retroattiva
alle sessioni approvate.

## Linux personale e confine N-20

La GUI può usare le librerie di sistema del runtime WebView/desktop, con
inventario effettivo e versione dell'ambiente personale. Non si applica per
analogia il requisito CLI di autosufficienza a una WebView GTK non qualificata.
Le eccezioni dinamiche sono **GUI-only**: non entrano nei dependency graph del
core/CLI, non inseriscono path compile-time dell'host nel prodotto e non
consentono comandi shell esterni richiesti a runtime. La build personale non
è un artefatto Linux GUI ufficiale di release.

I default dati personali sono XDG_DATA_HOME/Lintra, fallback
HOME/.local/share/Lintra, risolti nel backend con controlli di ownership/link;
nessun trattamento dell'assenza di un archivio come autorizzazione a perdere
storia. GTK/WebKit e runtime reali si verificano nello spike P1-01; una libreria
mancante è un prerequisito tecnico da soddisfare, non una modifica della policy.

## Gate e limiti

P0-05 chiude il recepimento OS, non installazione o runtime GUI. Servono P1,
P8/P9 e gli altri gate. CIWindows2025 prova core/CLI/adattatori lì eseguiti;
non equivale a Windows11 clean-room, WebView2 offline o non-admin/due account.
Cross-build/Wine non sono sostituti. La matrice di test registra OS/build,
runtime, file/artefatto SHA, rete disabilitata e risultati reali.

N-19/N-20/N-23 e QG core/CLI restano invariati, inclusi release da CI, immutabilità,
SHA256SUMS e policy di firma originale. Il cambiamento mirato GUI richiede
traccia esplicita delle sue dipendenze e prove. MM9/cutover non autorizza
distribuzione; G-REL richiede tutti i gate convergenti e il signoff previsto.
Nessun tag/release/pubblicazione è autorizzato da questo solo documento.
