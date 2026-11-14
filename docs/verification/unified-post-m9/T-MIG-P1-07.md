# T-MIG-P1-07 — Installer NSIS offline perMachine

14 novembre2026. **DONE**, acceptance installer PASS; decisione G-MIG-1 inP1-06.
Prerequisiti P1-02 eP1-04 DONE. Fonte runtime608ccdc1d79c8495085ade5888e8df99f97c958f,
CI43595367523 SUCCESS; la CI Windows Server non è una prova clean-room Win11.

## Artefatto e scope della prova

Installer sintetico220060935byte, SHA256
`b2bc84627e1286927b24e2ed79a00c77246bef334af14084cecbfd54ba1eedc1`.
Standalone CI9062912byte, SHA256
`93736932328f4f433b12a64075f08a30db0aa540522e5becc8b6c6e3d3057917`.
Manifest CI e hash/dimensioni verificati sull’host e dopo il trasferimento.
Nessun binario, segreto o dato reale viene versionato nel repository.

Il runtime applicativo ha il delta P1-04; configurazione NSIS perMachine,
offlineInstaller, AppID, versione Tauri/bundler e lockfile restano quelli
qualificati. Non si ripetono suite Rust/core/golden o build equivalenti.
P1-02 conserva la propria prova di provisioning da runtime assente e dei due
account standard: non sono attribuite al nuovo installer. Questa task deve
verificare il nuovo artefatto, i suoi effetti install/uninstall e la coerenza
con il confine security già qualificato; nessuna release agli utenti.

La VM dedicata è ripartita dal checkpoint ripristinato/provato inP1-04:
Win11 Enterprise Evaluation build26300/x86_64, installazione probe e HKLM
assenti. Rete Internet bloccata nel solo guest, SSH locale mantenuto.
Il runtime presente157.0.4341.46 è distinto dal caso assente provato inP1-02.
Script desktop e sentinelle riusano le implementazioni qualificate inP1-02;
solo directory/identità della prova diventanoP1-07. Il driver CSP deriva da608ccdc; la correzione del solo focus Windows è
identificata separatamente nel receipt finale. Nessuna skill globale installata nella VM.

Gli esiti installazione, desktop/security, conservazione e ripristino vengono
registrati solo dopo il completamento effettivo. Nessun PASS presunto.


Primo ciclo non qualificato: il probe era presente, ma il decoder Python host
ha sollevato `UnicodeDecodeError` prima di salvare l’exit code SSH. Non si
ricostruisce un exit0 dall’esistenza del file. Il collector ora acquisisce i
byte prima della decodifica, conserva sempre il return code e gli eventuali
byte originali; PowerShell emette UTF-8. Rimossa soltanto quell’installazione
sintetica e ripetuto il ciclo mirato, senza altre suite o nuove build.


Il primo desktop installato è PASS: account standard/sessione console1,
ACK900×600, finestra visibile/capture ispezionata, chiusura normale e smoke0,
Program Files invariati, sentinella SQLite e profilo per-account presenti.
Il binario installato6f1b32420a8819fcdae6e3b0393f7bad5a5b014b72cb5215ff1bc488f2419703
ha9062912byte: rispetto al file CI cambia soltanto il marcatore Tauri
`UNK`→`NSS` (3byte); tutti gli altri byte sono identici.

Il controllo security aggiuntivo del pacchetto è invece FAIL al primo input:
foreground di un altro processo, nessun Enter inviato, driver/launcher/task1;
zeroTCP/HTTP parziali non sono un PASS. Dopo la pausa la VM è stata trovata
spenta; i risultati persistiti sono stati raccolti senza rieseguire il desktop.
Il retry corregge esclusivamente il driver, mantenendo runtime e installer:
`ActivateOwned` tenta l’attivazione e, se negata, collega temporaneamente la
coda di input al thread foreground con
[AttachThreadInput](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-attachthreadinput),
rilasciandola in `finally`. Un errore SDK o detach fallito impedisce l’input.
Il controllo finale del PID foreground prima di Enter resta obbligatorio,
così come `event.isTrusted`, contatori Rust,18 controlli e0TCP/HTTP.
Nessuna policy OS, timeout, browser flag o requisito CSP viene indebolito.
La modifica non entra nel binario: non giustifica ricompilazione o nuova CI
MSVC/NSIS, che non esegue questo gesto desktop. Sintassi Node PASS; il test
pertinente è il singolo retry nativo, il cui esito resta da acquisire.


## Esiti integrati

Il retry mirato del driver è **PASS** sullo stesso binario installato:
18flag veri,12 eventi CSP,1diniego popup e1navigazione,1recovery del documento,
collettore raggiungibile prima del lancio e **0TCP/0HTTP** durante la prova.
Processo, harness, launcher e task terminano con0. `qualification-complete.json`
lega l’esito al digest del binario e del driver; il primo `desktop-complete`
FAIL resta conservato e non viene riscritto come successo. Lo stderr mantiene
il warning WebView2 Error1412 di uscita già osservato inP1-04.

La preparazione del retry aveva tentato di registrare il task già esistente:
ResourceExists/exit1 prima del reboot, senza nuova prova nativa. Corretto
aggiornando soltanto l’azione del task posseduto, verificato non attivo e con
principal `Interactive/Limited`; segreto LSA già creato riusato, non esportato.
Non si ripetono setup, desktop o reset credenziali per questa correzione.

Disinstallazione offline `/S`: launcher0, attesa della reale rimozione di exe
e HKLM, directory Program Files assente. SQLite e configurazione sintetiche
restano byte-identiche e il profilo WebView per-account esiste ancora. Nessun
DB prodotto è stato aperto: questo qualifica conservazione dell’installer,
non l’integrazione futura della persistenza o la perdita fisica di alimentazione.

Ponytail-review del helper: attivazione/detach circoscritti al gesto di prova,
nessun cambiamento a payload, native guards o condizione zero connessioni.
Sintassi Node e retry Windows PASS; i controlli puri applicativi e la CI MSVC
sono riusati perché la modifica del driver non entra negli input compilati.
La CI43595367523 resta l’unica build del blocco. La matrice core/CLI/Slint,
feature set e golden della CI43078048297 è riusata dopo confronto Git con
3aa0d14: sorgenti, dipendenze, config e fixture pertinenti identici.

Il ripristino finale viene registrato sotto solo dopo verifica degli accessi,
assenza dei residui temporanei e spegnimento effettivo della VM.


Ripristino finale **PASS**: UUID/dischi e stato spento verificati prima del
revert, marker base presente e delta/source/task temporanei assenti,
regola Internet rimossa, probe non installato, autologon disattivato e nessuna
password plaintext. Accessi SSH amministrativo e standard preservati, account
ordinario non elevato; VM effettivamente **shut off**. Il cambio del suffisso
disco durante `snapshot-revert` è registrato, senza operazioni su altre VM.

[Manifest/log/script riproducibili con SHA](T-MIG-P1-07-native/manifest.json).
P1-07 DONE;35/128 (27,3%). Prossima READY **T-MIG-P1-06**. La prova corrente
non dichiara provisioning da runtime assente sul nuovo installer, upgrade del
prodotto, dati clinici, G-REL o distribuzione: sono perimetri distinti già nel piano.
