# Spike Windows 11: artefatto e collaudo

T-MIG-P1-02: **DONE per lo spike Windows**, 13 novembre 2026.
La VM dedicata `Windows11-Lintra-Test` è disponibile via `ssh lintra-win`
(account standard) e `ssh lintra-win-admin` (`Lintra_tester`, amministrativo).
Installazione perMachine offline nei casi runtime presente/assente, due desktop
standard reali, isolamento ACL, disinstallazione conservativa e ripristino
provato: PASS sull’artefatto CI43014153420. Il caso assente è stato preparato
rimuovendo file/registrazioni di discovery, dopo rifiuto93 dell’uninstaller
WebView2; non equivale a un OS appena installato. La VM è tornata allo snapshot
pre-test; credenziali/task/firewall temporanei e secondo account rimossi.

[Dossier e limiti di scope](../../docs/verification/unified-post-m9/T-MIG-P1-02.md),
[manifest/log/capture](../../docs/verification/unified-post-m9/T-MIG-P1-02-win11-20261113/manifest.json)
e [script desktop/ACL spiegati](qualification/README.md).
Il probe non apre database del prodotto: le sentinelle SQLite/config sono solo
prove sintetiche di isolamento e conservazione. G-MIG-1 aggregato resta aperto;
prossima task P1-04, non una release. Applicare l’uso selettivo in
[AGENTS.md](../../AGENTS.md), senza nuovi controlli ricorrenti.

La build separata `.github/workflows/tauri-runtime-spike.yml` usa il runner
Windows Server 2025 esclusivamente per compilazione MSVC, Clippy, NSIS e
produzione dell'artefatto sintetico. Non può certificare una clean-room Win11.
L'overlay `src-tauri/tauri.windows.conf.json` configura NSIS perMachine e
WebView2 offlineInstaller secondo le
[istruzioni Tauri](https://v2.tauri.app/distribute/windows-installer/).
È previsto accesso alla rete durante la build per acquisire gli strumenti e
il redistributable; il prodotto non riceve un updater o un server.

## Necessità delle dipendenze

Nessun nuovo package applicativo rispetto allo spike P1-01. NSIS e WebView2
sono componenti già previsti da D14. Setup-node è un'azione di build necessaria
per riprodurre la versione Node già fissata: commit immutabile della fonte
ufficiale, senza dipendenze frontend aggiuntive. Checkout/upload-artifact
riutilizzano i digest già adottati nel repository. L'ICO deriva dallo stesso
SVG Lintra tramite il CLI Tauri fissato; i file mobile generati dal CLI non
vengono aggiunti al repository e non abilitano altri target.

Il workflow si avvia sui soli delta dello spike in work/p1-foundations o con
dispatch quando disponibile. Registra SHA di sorgente, binario e installer,
configurazione e OS effettivo, marcando installazione, offline e due account
NOT_RUN. Pubblica soltanto l'artefatto di test in Actions, non tag o release.
Non distribuire questo probe ai colleghi come applicativo Lintra.

## Procedura di riferimento per la prova mirata

1. Preparare una VM/macchina Windows 11 x86_64 autorizzata, snapshot iniziale
   recuperabile, un amministratore per installazione e due account standard
   sintetici. Registrare edizione/build OS, stato WebView2 iniziale e scaling.
   Non rimuovere il runtime condiviso da un PC di lavoro: la prova di runtime
   assente richiede un'immagine di test ripristinabile adeguata.
2. Acquisire l'artefatto del run esatto e verificare i digest di
   `windows-spike-build.json`. Conservare anche log e run ID. Disconnettere
   la rete della sola clean-room prima dell'installazione e documentarlo.
3. Eseguire il setup con elevazione, verificare installazione perMachine e
   installazione/riconoscimento WebView2 senza download. Registrare esito,
   versione WebView2 effettiva e percorsi del solo probe, non di Lintra reale.
4. Da ciascuno dei due account standard, senza elevazione, aprire il probe.
   Verificare la scritta di collegamento Rust, UI leggibile, font locali e
   chiusura. `--smoke-test` deve completare il render e uscire con codice zero;
   acquisire stdout esplicitamente se si usa il binario GUI senza console.
   Non interpretare il solo exit code senza acknowledgment come prova di render.
5. Conservare soltanto capture sintetiche della finestra del probe e metadati
   tecnici. Il probe non valida tracciati o apre DB: non usarlo per dichiarare
   già provata la conservazione dei DB/report del prodotto.
6. T-MIG-P1-04/05 devono qualificare CSP/capability/rete e profilo WebView/DPI
   su entrambe le piattaforme. P1-07 completa installer, uninstall e relative
   prove dopo i propri prerequisiti. Il gate G-MIG-1 richiede le prove reali,
   mentre la consegna finale dipende anche da tutti i gate G-REL.

Finché manca l'ambiente, mantenere la task aperta e continuare le attività
indipendenti READY. Nessuna seconda ratifica dello stack o dell'OS è richiesta.
