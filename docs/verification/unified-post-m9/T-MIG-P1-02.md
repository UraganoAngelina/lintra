# T-MIG-P1-02 — Spike Windows / WebView2

Aggiornato il 13 novembre 2026. **DONE per lo spike Windows; build CI PASS e prove native PASS**.

La successiva autorizzazione amministrativa dell’autore supera il blocco
di disponibilità riportato nelle sezioni storiche seguenti. Installazione
offline nei due casi runtime presente/assente e due desktop standard PASS;
avvio dopo provisioning, disinstallazione conservativa e ripristino finale PASS.
Evidenze e digest: [manifest](T-MIG-P1-02-win11-20261113/manifest.json).
G-MIG-1 aggregato NOT_VERIFIED; prossima task READY T-MIG-P1-04.

## Evidenze storiche dell’11 novembre 2026
Prerequisito P0-08 DONE; P1-01 Linux DONE su b1f14ce.

Preflight storico dell'11 novembre: nessun virsh, QEMU, VBoxManage, vmrun, PowerShell, Wine,
NSIS locale, immagine VM Windows o /dev/kvm disponibile. La ricerca delle
immagini è stata limitata al workspace Lintra; nessuna scansione di dati
personali o modifica di checkout adiacenti. L'autore ha risposto alla domanda
mirata di disponibilità: **«Nessun ambiente Windows 11 al momento»**.
La CI Windows Server 2025 non soddisfa il requisito Win11 clean-room.

Preparati overlay Windows dello spike, ICO dal marchio vettoriale esistente e
workflow nativo MSVC/Clippy/NSIS. Installer configurato perMachine con
offlineInstaller WebView2, ID del probe separato dal prodotto. Nessun package
applicativo nuovo o modifica normativa, core, CLI, Slint, HMAC o SQLite.
Setup-node con SHA ufficiale immutabile serve a fissare Node nella build;
gli altri Actions pin riutilizzano quelli qualificati nel repository.

[Procedura e necessità documentate](../../../spikes/tauri-runtime/WINDOWS_QUALIFICATION.md).
Il workflow produrrà artefatto sintetico e manifest SHA/run/OS senza tag o
release. Il download degli strumenti durante la build non è traffico del
prodotto. Non attribuire a tale build installazione offline, due account,
versioni runtime, DPI o zero-network mai provati.

Esiti locali: JSON overlay sintatticamente valido e diff check PASS; review
manuale del workflow. Parser YAML/PowerShell non installati localmente:
nessun package aggiunto solo per il lint; parsing Tauri/Actions effettivo in CI.
Inventario e digest in T-MIG-P1-02-preflight.json;
CI43014153420 conclusa success su bdca1cf00121717b3c58504127ae4ce45f806e6b: frontend strict, fmt/Clippy MSVC release, build e NSIS offline configurato, upload artefatto sintetico. Metadati e log completi archiviati in native-43014153420.json/log.gz. Non sostituisce la prova Win11. Windows11 install/run NOT_RUN.
G-MIG-1 NOT_VERIFIED. La prova non è sostituita da simulazione o da un flag.

P1-03 è conclusa indipendentemente su Linux; next READY **U-OPS-04**. Nessun ingresso P2/P3 prima del vero gate G-MIG-1. AUTO_CONTINUE.

Artefatto sintetico scaricato nel target locale ignorato da Git e verificato
contro il manifest CI: exe8.722.432byte SHA7927f57a…2b1b0d;
setup219.878.356byte SHAf31b7064…c0897. Digest completi e percorsi nel
T-MIG-P1-02-artifact-receipt.json. Installazione non eseguita: nessuna
consegna del prodotto o PASS Win11 derivato dal solo file disponibile.

## Prima ripresa del 13 novembre 2026 — prima dell’autorizzazione amministrativa

L'autore rende disponibile la VM `Windows11-Lintra-Test` (Windows 11 x86_64,
QEMU/KVM su CachyOS), alias `lintra-win`, shell PowerShell, account standard
`lintra-agent`. Direttiva permanente recepita in `AGENTS.md`: nessun nuovo gate,
avvio o test ricorrente. Questo accesso è motivato esclusivamente dal requisito
Windows nativo già esplicito di P1-02; nessun avvio della VM o sync sorgenti.

`ssh -o BatchMode=yes -o ConnectTimeout=10 lintra-win "whoami"`: exit 0,
`desktop-rllad2m\lintra-agent`. La connessione è stata eseguita con escalation
approvata dell'ambiente Codex: nel sandbox il client rilevava configurazione
SSH di sistema con permessi non accettati e il socket era vietato. Nessuna
modifica di configurazione, chiave, host key, SSH, firewall o rete.

La query WMI `Get-CimInstance Win32_OperatingSystem` termina con exit 1,
`0x80041003 Accesso negato`; non è un PASS. Il successivo preflight in sola
lettura del registro/processi termina con exit 0 e riporta:

- registro OS: `ProductName=Windows 10 Enterprise Evaluation`,
  `DisplayVersion=26H2`, `CurrentBuild=26300`, `UBR=9457`; valori grezzi,
  senza attribuire alla sola stringa ProductName una qualifica dell'OS;
- architettura `AMD64`, PowerShell `5.1.26100.9444`;
- account `DESKTOP-RLLAD2M\lintra-agent`, token non elevato;
- processo SSH in sessione 0; Explorer presente in sessione 3: non è una prova
  di render né un'autorizzazione a interagire con la sessione di altro account;
- WebView2 machine-wide presente, directory versione `157.0.4341.46`;
- directory `C:\Program Files\Lintra Runtime Spike` assente; questo controllo
  limitato non certifica da solo l'assenza di ogni installazione del probe.

Ricalcolati localmente SHA256 e dimensioni dei due artefatti nel receipt
esistente: entrambi corrispondono alla CI43014153420/sorgente bdca1cf. Nessuna
ricompilazione o nuova CI, nessun trasferimento né esecuzione dei binari.

Blocco circoscritto: il semplice accesso SSH standard non abilita l'installazione
perMachine elevata, né dimostra la disponibilità di uno snapshot clean-room
senza runtime o la prova da due account standard. Chiesto all'autore se dispone
di snapshot ripristinabile e amministratore per quei passaggi; nessuna elevazione,
creazione account o rimozione del runtime tentata. La disponibilità generale
della VM non è più il prerequisito mancante. Install/run/offline restano NOT_RUN;
G-MIG-1 NOT_VERIFIED, conteggio DONE invariato. Nessuna task indipendente READY
alla frontiera registrata; riprendere la prova esistente quando predisposta.


## Autonomia amministrativa e prove native — successiva, 13 novembre 2026

L’autore autorizza l’amministrazione autonoma della sola VM sacrificabile,
account/snapshot/installazione/disinstallazione e automazione desktop; nessun
nuovo gate. Token `Lintra_tester` amministrativo effettivo. Alias host
`lintra-win-admin` aggiunto con stesso endpoint/chiave, lasciando invariato
`lintra-win`/`lintra-agent`; backup privato della configurazione SSH. Credenziali
non copiate nel repository o nei log. Mandato recepito in `AGENTS.md`.

Identità libvirt controllata prima dello snapshot/revert:
`Windows11-Lintra-Test`, UUID `8e50e338-7593-4e13-8556-bdf22097bcc6`, disco iniziale
`/var/lib/libvirt/images/Windows11-Lintra-Test.1794496233`, backing qcow2 dedicato.
Snapshot precedente `WIN11-CLEAN-BASELINE` conservato, non dichiarato verificato.
Creato `LINTRA-P1-02-PRETEST-20261113` dopo shutdown ordinato (Windows ha prima
completato gli aggiornamenti già pendenti), disco `sda` esterno, CD escluso.
Revert reale PASS: marker precedente conservato, marker successivo scomparso,
boot Windows/UEFI/TPM e SSH amministrativo riusciti; BitLocker disattivato.
È uno snapshot disco a guest spento, non un backup completo dei segreti TPM o
di ogni variabile firmware. Il firmware/TPM non è stato riconfigurato.

L’OS rilevato con token amministrativo è `Microsoft Windows 11 Enterprise
Evaluation`, build `26300`, x86_64. La vecchia stringa registro `Windows 10`
non è stata usata come autorità per l’identificazione del sistema.

### Identità dell’artefatto realmente installato

Installer CI43014153420, sorgenti `bdca1cf00121717b3c58504127ae4ce45f806e6b`:
SHA256 `f31b70649313652c6ebd2af373b41283e93e7a837072ede1147c30175f3c0897`.
Sorgenti/configurazioni/lockfile/icone/workflow dello spike identici alla CI;
nuovi soltanto gli script di qualificazione e i documenti. Nessuna build nuova
necessaria per provare questo artefatto immutato.

L’eseguibile installato ha SHA256
`886b399223028c97f99803b5e7056ea60c0be4faa60083a16cde3c1a20780015`, diverso dal
file standalone nel receipt. Confronto byte-per-byte: stessa dimensione
8.722.432 byte, **solo tre byte diversi**, marcatore Tauri
`__TAURI_BUNDLE_TYPE_VAR_UNK` → `__TAURI_BUNDLE_TYPE_VAR_NSS`; ogni altro byte è
identico, incluso tutto `.text`. Registrare entrambe le identità, non un falso
hash uguale. Nessuna modifica applicativa per ottenere il PASS.

### Prove effettive, separazione degli account e limiti

- Firewall del solo guest: regola outbound `Block`, destinazione `Internet`,
  gestione SSH locale conservata. Connessioni TCP verso `1.1.1.1:443` e
  `8.8.8.8:443` falliscono prima delle installazioni. Nessun controllo host
  disabilitato. Questo prova l’assenza di accesso Internet, non zero tentativi
  di rete del software Microsoft o l’intero gate CSP di P1-04.
- `setup.exe /S` con token amministrativo, runtime già presente: exit 0,
  installazione sotto `C:\Program Files\Lintra Runtime Spike`, metadati nella
  chiave HKLM Uninstall, nessun servizio Lintra. Configurazione `perMachine`
  confermata dal bundle e dagli effetti reali, non solo dal file JSON.
- `lintra-agent` resta standard; creato `lintra-test2`, anch’esso standard.
  Entrambi avviano l’app installata in un proprio vero desktop console,
  token limitato, sessione non zero. Le task hanno principal `Interactive`,
  `Limited`. ACK dopo render React/font/round-trip Rust, finestra visibile,
  capture ispezionata, chiusura ordinaria exit 0 e ulteriore `--smoke-test`
  con ACK/exit 0 per ciascuno. Le sessioni numericamente uguali dopo reboot
  non sono una sessione condivisa: identità e profili sono distinti.
- `%LOCALAPPDATA%\it.lintra.runtime-spike` distinto per account. Manifest dei
  file installati invariato dopo uso ordinario. Entrambi ricevono accesso
  negato sulla scrittura di un file di prova in Program Files e sulla lettura
  di SQLite/config/profilo WebView dell’altro utente; amministratore ha prima
  verificato l’esistenza reale di entrambi i file, quindi Missing non è PASS.
- SQLite/config di conservazione sono **sentinelle sintetiche**, create con
  il modulo SQLite della versione Node già installata; contengono soltanto
  il nome dell’account di test. Hash diversi e righe corrette per i due utenti.
  Il probe non usa il DB del prodotto: non attribuire a questi risultati la
  futura integrazione persistente React/Tauri, backup o aggiornamento.
- `uninstall.exe /S`: launcher exit 0; attesa esplicita della rimozione di
  eseguibile e registrazione HKLM, directory installata eliminata. Tutte le
  quattro sentinelle SQLite/config rimangono byte-identiche, profili WebView
  conservati. Nessuna cancellazione dei dati per-account.

### Runtime assente: preparazione reale e provisioning offline distinto

L’uninstaller WebView2 restituisce **93** e mantiene file/registrazione: FAIL
come tentativo di rimozione, non prova runtime assente. Nella sola VM già
ripristinabile: logout dei desktop sintetici, stop dei processi runtime e
servizi/task Edge Update pertinenti; file runtime spostati fuori da tutti i
percorsi di discovery, quattro registrazioni della sola app WebView2 esportate
nella VM e rimosse. Nessuna modifica al browser Edge o ai controlli dell’host.
Assenza verificata in percorsi macchina e dei tre account e in Clients HKLM.
È una rimozione controllata del runtime, **non una nuova installazione OS
pristina**; la copia interna non viene indicata come un runtime utilizzabile.

Con la stessa regola Internet attiva, lo stesso installer NSIS parte da tale
assenza e termina **exit 0**: nuova registrazione WebView2 `157.0.4341.46`,
file runtime al percorso normale, probe e HKLM ripristinati. Identificato il
redistributable estratto da NSIS (214.990.032 byte, firma Microsoft `Valid`,
SHA256 `1be9d2926954d99dd62653a22efe813160fe0c40f1e422b44e6aaf5473f68eec`)
e la catena processi con `/installsource offline` e `/offlinedir`. Nessun
PASS dedotto dal solo runtime preinstallato. Avvio desktop successivo PASS: nuovo risultato, nuova capture ispezionata,
ACK reale, chiusura normale e smoke exit 0. Sentinella SQLite del secondo
utente ancora byte-identica dopo reinstallazione.

### Correzioni dell’automazione, senza cambiamenti di prodotto

La revisione automatica ha rifiutato l’autologon con password plaintext in
Winlogon prima dell’esecuzione. Alternativa approvata: strumento Microsoft
Sysinternals Autologon ufficiale, firma valida, segreto LSA cifrato, password
casuali dedicate conservate nella sola VM in blob DPAPI LocalMachine protetti
da ACL Administrators/SYSTEM. Nessuna password plaintext Winlogon presente,
nessuna password nei log; login singoli e successivo ripristino previsto.
DPAPI CurrentUser non era disponibile nel token SSH: errore conservato e
risolto con scope macchina e ACL, senza esportare le credenziali.

Primo harness desktop FAIL prima del lancio Tauri: `process.argv[1]` apriva il
file JS come DB; corretto in `[2]`. Processi brevi in PowerShell 5 potevano
lasciare ExitCode null: helper SQLite con Process .NET posseduto prima di
Start, pipe lette in parallelo e timeout; handle del probe conservato.
Il retry nativo passa; primo FAIL conservato. Regola firewall: errore socket
atteso gestito come blocco, senza mascherare exit code. Policy script soltanto
per il processo di test nel guest; policy persistenti e host invariate.

Script riproducibili e spiegazioni in
`spikes/tauri-runtime/qualification/`. Nessuna dipendenza di prodotto,
validazione clinica, core, CLI, Slint o protocollo crittografico modificati.
G-MIG-1 aggregato resta NOT_VERIFIED fino alle altre prove P1 previste.


### Chiusura effettiva e handoff

Seconda disinstallazione dopo il provisioning del runtime: launcher exit 0,
file e HKLM rimossi, tutte e quattro le sentinelle byte-identiche, profili
WebView conservati. Ripristino finale dello stesso snapshot dopo shutdown
pulito e verifica UUID/disco: boot e SSH riusciti, runtime originale presente,
account temporaneo, directory delle credenziali, script/task, regola firewall
e probe assenti. `lintra-agent` ancora standard; entrambi gli alias SSH
funzionanti. L’alias amministrativo host richiesto rimane disponibile.

[Manifest e log nativi compressi](T-MIG-P1-02-win11-20261113/manifest.json),
con tre capture desktop originali ispezionate, comandi/exit code e FAIL
intermedi mantenuti distinti dai retry PASS. Una sola redazione esplicita del
payload identificativo del ping Microsoft nel log processi; catena offline
inalterata. Nessuna password, chiave, dato clinico o binario distribuito nel
repository. Nessuna nuova CI necessaria per il runtime immutato; la CI
ufficiale resta il run 43014153420 e non è sostituita dalla VM.

**T-MIG-P1-02 DONE**, criteri dello spike soddisfatti. Nessuna qualificazione
implicita di persistenza del prodotto o release. **32/128 DONE (25,0%)**.
T-MIG-P1-04 e P1-05 diventano READY; riprendere **P1-04**, poi le successive
secondo dipendenze/gate, senza nuovo checkpoint umano. G-MIG-1 aggregato resta
NOT_VERIFIED fino a P1-04/05/07 e alla decisione P1-06. AUTO_CONTINUE.
