# T-MIG-P3-03 — Font, attribuzioni e token locali

15 novembre 2026. **DONE**: copie byte-identiche e baseline WebView distinta.
Font IBM Plex Sans regular/semibold e Mono regular con licenza OFL originale,
già forniti in `ui/ui/fonts`; nessuna conversione, download o nuova dipendenza.
[Manifest di provenienza e hash](../../../apps/lintra-desktop/assets-provenance.json).
Due test Node confrontano sorgenti, copie, licenza, fonti dei token e risorse CSS.

`tokens.css` centralizza font e ruoli, riusando geometrie e colori semantici
del brief/Slint; il marchio segue la ratifica blu/cyan dell'11 novembre.
Cyan è accento sul fondo scuro, blu sul chiaro; slate non viene usato come
testo normale sui fondi chiari. Corpo 14 px, titolo 28 px, spaziature 4/12/24/32.
Le due palette seguono per ora la preferenza del sistema; la selezione esplicita
del tema appartiene alla UI funzionale successiva. Nessuna risorsa remota,
nessuna riscrittura di Slint o promessa di parità pixel tra toolkit.

Build frontend e smoke release React/IPC nativo Wayland PASS.
[Capture nativa scura 873×503](T-MIG-P3-03-native.png) osservata con vision:
font locali, titolo e stato leggibili, nessun overflow o contenuto dimostrativo.
[Receipt](T-MIG-P3-03-native.json): catturata solo la finestra di proprietà,
PID e focus verificati prima di limitare l'immagine alla sua area. Il test
capture termina e attende il processo posseduto (SIGTERM -15); la chiusura
normale exit 0 è provata separatamente dallo smoke release. Nessuna capture
di altre applicazioni. In precedenza il selettore testuale Hyprland non aveva
focalizzato la finestra; il controllo usa ora l'oggetto finestra nativo risolto
dall'indirizzo verificato. Non viene modificata la configurazione del compositor.

Il nome nell'header è una semplice etichetta; non ricostruisce il wordmark
ufficiale con un font. Il logo book resta la fonte approvata in sola lettura,
senza copiare la tavola o inventare asset separati in questa task. Gli artwork
di prodotto/icone restano da preparare nelle task UI e packaging pertinenti.
Questa prova non qualifica tutti i viewport, scale o il tema chiaro nativo.
Controllo meccanico Impeccable sul delta: nessun rilievo; nessun hook aggiunto.
Segue P3-06 per le verifiche aggregate obbligatorie, senza cicli VM.

Prima CI Windows su f482918: test asset FAIL perché autocrlf trasformava in
CRLF la copia della licenza OFL; i 14 test IPC e il controllo CSS PASS. Nessun
Rust eseguito da quel job. Correzione circoscritta `.gitattributes`: LF esplicito
per la copia OFL e per le tre fonti testuali citate con hash. Le sorgenti
mantengono così i byte Git canonici, come già previsto per la licenza Slint.
Checkout temporaneo con `core.autocrlf=true`: tutte le 11 risorse di provenienza
confrontate byte per byte/hash PASS. Nessuna normalizzazione nel test o soglia
indebolita; CI Windows del checkout corretto ancora da completare in P3-06.
