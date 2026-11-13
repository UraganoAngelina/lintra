# T-MIG-P1-05 — Profilo WebView, font e DPI

13 novembre 2026, Europe/Rome. **DONE — acceptance della task PASS**; G-MIG-1 aggregato
NOT_VERIFIED. Prerequisiti P1-01/P1-02 DONE, base d9d0f1a. Riprese le
modifiche già presenti nella working tree, senza riavviare il piano o P1-04.
P1-04 resta BLOCKED sul criterio zero connessioni Windows; questa task ha
dipendenze indipendenti. Nessuna nuova milestone o prova Windows ricorrente.

## Delta e ragioni di correttezza

Lo spike, separato dal core/CLI/Slint, aggiunge soltanto `--profile-test`.
`ProbeOptions` abilita l'inizializzazione del renderer e il controllo nativo;
la modalità è esclusiva rispetto a `--security-test`. Le due capability e i
comandi rimangono gli stessi. Nessuna modifica di CSS/font, dati normativi,
golden, persistenza, dipendenze o policy di rete.

`measureProfile` viene chiamata dopo `document.fonts.ready` e due animation
frame. Controlla le due FontFace locali IBM Plex Sans, Regular400 e
SemiBold600, effettivamente caricate, lo stile computato del corpo/titolo,
14/28 CSSpx, overflow orizzontale e titolo nel viewport. Il DTO ha soli numeri
e booleani, senza testo arbitrario, path, user agent o dati di lavoro.
FontFace caricata e famiglia CSS non dimostrano la copertura di qualsiasi
glifo futuro: la prova riguarda il documento sintetico dello spike.

`ProfileReport::matches_native` confronta il report con `inner_size` e
`scale_factor` letti da Tauri, indipendentemente dal renderer. Richiede stesso
DPR e dimensioni fisiche coerenti su entrambi gli assi, tollerando solo2pixel
di arrotondamento. Zoom CSS/browser non viene dichiarato DPI del monitor.
Il test negativo verifica scala/dimensioni incoerenti, font non caricato,
titolo fuori viewport, dimensione del font errata e campi wire sconosciuti.
Il caso numerico150% del test unitario **non è una misura nativa150%**.

`native_profile.cjs` possiede e attende il processo GUI, richiede un unico ACK
e un unico record nativo con exit0, registra hash del binario, OS/architettura,
versione Node/WebView e scala. Verifica l'esistenza effettiva del profilo e
il percorso previsto, non soltanto il valore stampato dal resolver.
Su Linux usa root XDG temporanee private, rimosse dopo l'uscita. Su Windows
usa il profilo reale per-account; non legge contenuti della cache.

`windows-profile-desktop.ps1` è un launcher mirato: rifiuta token elevato,
sessione0 e sessione diversa dalla console attiva. Esegue la sonda e acquisisce
soltanto la finestra figlia verificata per titolo, sessione e foreground;
confronta anche `GetDpiForWindow` con la scala Tauri/WebView. Chiude normalmente
la finestra e registra exit code, capture e risultati reali. Non aggiunge
una matrice CI o un requisito Windows dopo le altre task.

## Profilo runtime e limiti

Il manager Tauri2.12.5, sorgente locale `manager/webview.rs`, imposta
`data_directory` al resolver `app_local_data_dir` quando non esplicitato
su Linux/Windows; Wry riceve quel percorso come chiave del contesto.
Lo spike usa l'identificatore `it.lintra.runtime-spike`, quindi il suo profilo
non è ancora la root prodotto `%LOCALAPPDATA%/Lintra` di ADR-MIG-002.
L'integrazione e i controlli dei percorsi prodotto restano U-OPS-03/MIG;
nessuna migrazione o cancellazione di profili esistenti in questa task.
La semplice presenza della directory non qualifica ACL fra account, assenza
PHI/cache/dump, cancellazione fisica o tutta la privacy del futuro prodotto:
quelle prove appartengono alle task già previste, inclusa P7-03.

## Verifiche effettive

- TypeScript strict/Vite: PASS, Node26.13.0; font TTF locali inclusi, senza CDN.
- Linux Rust1.99.1:4 test PASS, build custom-protocol e Clippy `-D warnings` PASS.
- Wayland reale: PASS con WebKitGTK2.54.2, scala100%, corpo14/titolo28 CSSpx,
  entrambi i font caricati, titolo nel viewport e nessun overflow orizzontale.
  Finestra873×1018 fisici/CSS sotto il tiling personale; non viene falsamente
  dichiarata900×600 solo perché quelle sono le dimensioni richieste da Tauri.
- Tentativo dentro sandbox: GTK non inizializzabile, exit101, conservato FAIL;
  la prova desktop effettiva è distinta, con lo stesso hash del binario.
- Windows MSVC/desktop e CI del delta: PASS; risultati e SHA nelle sezioni successive.

Binario Linux: SHA256
`065eb1949603b7987e687b0f337fcb54165e5a92a67083a206417fff9b5e105c`.
Gli input Windows sono identificati dal manifest di39file: sorgenti locali
compresi i delta non committati, font e `dist` prodotto dal Node host fissato.
Niente `target`, `node_modules`, chiavi o dati sanitari nella sincronizzazione.
Rust/MSVC1.99.1 e dipendenze/cache già presenti vengono riutilizzati, senza
setup o aggiornamento degli strumenti. Il launcher desktop è identificato
separatamente dal proprio SHA, non modifica gli asset compilati.

La VM era **spenta** e viene avviata per il requisito WebView2/font/DPI di
questa task. Nessun utente desktop autenticato o operazione preesistente da
interrompere. Il checkpoint corrente è
`LINTRA-P1-04-BEFORE-DESKTOP-20261113`, UUID/disco verificati prima di qualsiasi
ripristino. L'eventuale accesso console standard temporaneo viene rimosso
ripristinando quel checkpoint dopo raccolta delle evidenze sull'host;
spegnimento finale ordinato obbligatorio perché la VM è stata avviata qui.
L'accesso temporaneo usa l'API documentata
[LSA di Winlogon](https://learn.microsoft.com/en-us/windows/win32/secauthn/protecting-the-automatic-logon-password),
senza password plaintext in file, registry o argomenti di processi esterni;
nessun helper aggiuntivo viene installato.
L'esito del ripristino e dello spegnimento si registra soltanto dopo verifica.

CI43451372328 SUCCESS su d9d0f1a qualifica il delta precedente P1-04,
**non** questo codice P1-05. Non risolve il blocco nativo3TCP di P1-04.
Le prove100% osservate non autorizzano pixel parity fra WebKitGTK e WebView2,
altri DPI/monitor o intera UI futura; P7-06 resta prevista dal piano invariato.

## Aggiornamento delle prove Windows/CI

CI43494464261 **SUCCESS** sul commit runtime
`413b7e41da0fda75f29bb68fcefcea3999a6ec57`: TypeScript strict,4test Rust,
Clippy MSVC `-D warnings`, build release e NSIS offline perMachine. Non è
una prova di installazione o di desktop Windows11.

Il primo comando di compilazione VM non ha restituito un risultato verificabile:
SSH255, nessun binario P1-05 prodotto e nessun processo remoto rimasto.
Il client appeso è stato chiuso soltanto dopo tale riscontro. Il log parziale
resta un esito non verificato, non un PASS. Ripresa la sola build, senza
ripetere test: exit0 e binario SHA256
`d532f913b71a08002633ea8204dda61aa723e9d1d9cf433048670c27ff324d84`,
8912384byte. Log e risultato scritti anche sul disco guest per non dipendere
dal solo trasporto SSH. La prova desktop e il relativo hash vengono registrati
separatamente; nessuna qualifica viene ereditata da un eseguibile diverso.

## Prova desktop Windows effettiva

**PASS** sul binario esatto CI43494464261, SHA256
`0706a715b928d619c056b547cc4eac7532378892442eac7213faea2426f89d05`,
8913408byte. Windows11 build26300, WebView2 `157.0.4341.46`, Node del solo
harness `26.13.1`; frontend compilato dalla CI con Node26.13.0 fissato, senza
ricompilarlo con la versione guest. Account standard `lintra-agent`, token non
elevato, vera console/sessione1. Viewport900×600 CSS/fisici, scala100%, DPI
Win32 della finestra96, corpo14/titolo28 CSSpx, Regular/SemiBold caricati,
famiglie corrette, titolo nel viewport e nessun overflow orizzontale. Profilo
realmente presente sotto `%LOCALAPPDATA%/it.lintra.runtime-spike/EBWebView`.
ACK unici, processi con exit0 e cattura della finestra ispezionata con vision.

Il primo launcher ha rilevato correttamente le misure, ma ha respinto la cattura
per il titolo: Windows PowerShell5 interpreta uno script UTF-8 senza BOM con
la code page ANSI. Il launcher corretto costruisce il solo separatore del titolo
con `[char]0x00B7`, conserva il confronto esatto e registra titolo/visibilità.
SHA launcher finale `8ca7d1486a5ad655145a058d42525942d8670ca744f0a743a206caba982bda54`;
medesimo binario CI, nessuna modifica Rust/frontend o ricompilazione della app.
Il primo FAIL e i suoi dati rimangono separati dalla prova finale PASS.
Anche il collettore SSH ha incontrato la code page OEM nel titolo stampato:
per l'evidenza autorevole sono stati raccolti con SCP i JSON originali UTF-8,
non ricostruiti da un testo console decodificato con sostituzioni.

WebView2 scrive su stderr, in uscita, `Failed to unregister class
Chrome_WidgetWin_0`, errore1412. L'output è conservato: non si dichiara assenza
di diagnostici runtime. ACK, chiusura normale e exit0 sono osservati; nessun
errore di misura o processo GUI residuo è stato rilevato. La prova non misura
traffico generale OS/browser e non risolve il finding3TCP di P1-04.

La correzione finale riguarda soltanto il launcher e i documenti. Gli input
compilati Rust/frontend/font/config/lock sono byte-identici a413b7e4, come
verificato con `git diff --exit-code` sui percorsi runtime. La CI di quel commit
qualifica dunque l'app effettivamente provata; il launcher corretto è qualificato
dalla sua esecuzione desktop reale. Non serve duplicare build/installer per
questo delta di sola raccolta della prova.

Il primo rifiuto della revisione automatica dell'accesso console è stato
riesaminato alla luce del mandato amministrativo esplicito in AGENTS.md e della
procedura già eseguita in P1-04; il secondo tentativo è stato consentito.
Nessun aggiramento del controllo, password in chiaro o intervento manuale.

## Ripristino e chiusura

Ripristino del checkpoint esistente **PASS**, dopo raccolta dei risultati e
spegnimento guest ordinato: marker base presente, marker P1-05 e sorgenti
temporanei assenti,0task P1-05, autologon disabilitato e nessuna password
plaintext Winlogon. Accessi SSH admin elevato e standard non elevato conservati.
Nessun utente o build preesistente è stato interrotto. Il ripristino elimina
le modifiche di accesso console e le cache della prova; i binari e le evidenze
necessari restano raccolti sull'host. Spegnimento finale richiesto con
`virsh -c qemu:///system shutdown Windows11-Lintra-Test`: stato finale
**shut off** verificato, coerente con quello iniziale.

Evidenze raw compresse, screenshot e hash nell'[indice](T-MIG-P1-05-native/manifest.json).
I JSON originali UTF-8/BOM sono compressi senza conversioni di fine riga.
Il manifest Windows registra gli input prima del commit413b7e4: soltanto
README e launcher/documentazione sono successivi alla raccolta; la loro
identità non viene confusa con quella del runtime compilato e della app CI.

Prossima frontiera: P1-04 resta BLOCKED su3TCP senza payload Windows;
P1-07 dipende daP1-04, P1-06 daP1-04/05/07 e P2 daP1-06. U-OPS-03 e
U-NORM-04 attendono integrazione reale nelle task già previste. Nessuna altra
task NOT_STARTED soddisfa tutte le dipendenze DONE. **NO_READY_TASK**.
Nessun nuovo quality gate e nessuna modifica di milestone/requisiti del piano.
