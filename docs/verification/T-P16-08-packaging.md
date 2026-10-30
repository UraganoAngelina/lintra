# T-P16-08 — packaging UI (in corso)

Preflight UI dell'autore ee33d64 qualificato da check e visual1×/2×; baseline
eseguibili ricollocate senza cambiare pixel. Design/documentazione utente
preservati. UI07 e15-05 prerequisiti realmente DONE;116/140, M8 ancora aperta.

## Implementazione e ragioni di correttezza

`Product` nel tooling seleziona esplicitamente cli/ui; default CLI preservato,
selettori ignoti/duplicati rifiutati prima del build. Build con package/bin
espliciti, stessa ricetta embedded/remapping/CRT e doppia compilazione pulita.
Il verifier distingue le policy: Linux UI aggiunge solo Fontconfig ai DT_NEEDED
base, già dipendenza dello stack Slint qualificato; Windows aggiunge esclusivamente
DLL OS per finestra/input/font. Nessun wildcard Linux e nessuna SQLite/SSL/CRT
esterna ammessa. I test verificano che la policy UI non attenui quella CLI.

`release.pack/inspect_archive` hanno un parametro prodotto additivo, default CLI.
Gli archivi CLI restano invariati (3entry); UI ha5entry: eseguibile, BUILD.json,
SHA256SUMS e testi originali delle licenze IBM Plex OFL e Slint royalty-free2.0.
Licenza Slint copiata dal crate1.18.1 già qualificato; nessuna nuova dipendenza.
Timestamp/ordine/mode fissi e no-clobber, inventario e digest verificati; i test
usano payload sintetici solo per il protocollo, non per certificare eseguibili.

Il build.rs di provenienza è condiviso con UI (solo visibilità del main cambiata),
così --version identifica versione e commit senza Git/Cargo runtime. La UI espone
--packaging-check: piattaforma software reale, frame e catalogo incorporato, senza
validare record. --packaging-startup-check usa invece il normale backend nativo,
finestra/event loop/controller, poi chiude con un timer. Nessun cambio a stile,
regole o percorso dell'utente normale. Windows release usa sottosistema GUI.

`ui_package.py` è solo CI e non pubblica: build riproducibile, gate artifact,
archivio estratto in installazione temporanea e prove con PATH vuoto. Il rendering
headless non viene scambiato per avvio X11: il workflow Linux usa Xvfb per il
probe nativo e LD_DEBUG=libs per verificare i basename caricati dinamicamente
contro ui/linux-runtime-libraries.list. La lista include solo runtime OS X11/input,
Fontconfig e dipendenze di sistema. Windows richiede runner MSVC nativo.

## Verifiche iniziali e limiti al primo avvio della qualifica

Preflight e test dell'eseguibile debug su host Linux passati con PATH vuoto;
22test Python positivi, Clippy tooling positivo. Test completi/sicurezza nuova
policy, build release native, due build pulite, scansioni import/path e archivi
finali ancora da qualificare. Xvfb non disponibile sull'host: startup reale va
eseguito sul runner, non dichiarato superato localmente.

Workflow dedicato UI native packaging non usa segreti di pubblicazione, non
modifica tag/release immutabile0.1.1-rc.1. Gli archivi saranno artifact CI con SHA
esatto; una nuova pubblicazione richiede il protocollo/versione già concordati.
Non dichiarare M8 o task DONE prima delle evidenze Linux/MSVC e regressioni.

Hardening runtime: il probe rimuove LD_PRELOAD/LD_LIBRARY_PATH e verifica anche
il percorso risolto delle librerie osservate sotto root OS /lib*/usr/lib*, non
solo il basename. Una libreria dal nome ammesso ma caricata da /tmp è rifiutata.
23test Python passati, compresi trace assente/libreria extra/root non OS; nessun
trace vuoto viene considerato prova di autosufficienza.

## Diagnosi del primo gate —41025660338 su d073c4b

Linux sviluppo: tutte le suite passano; negative-font fallisce prima del controllo
perché il fixture isolato non copiava il nuovo ../build.rs condiviso. Corretto il
fixture mantenendo invariato il marker di errore richiesto e i font originali.

Linux release: due build reali, stessa dimensione22999840byte, primo scarto0x32c;
riproducibilità NON superata. La ricetta mappava solo la directory padre
release-profile, lasciando il nome univoco first/second nei path del codice
Slint generato sotto OUT_DIR. Aggiunto remapping del singolo build root allo
stesso alias prima della compilazione. Prova locale con rustc su directory diverse
conferma che il remapping può produrre byte identici; conferma sull'app completa
ancora richiesta dal nuovo runner. Confronto binario integrale invariato: nessuna
normalizzazione degli header/build-id o accettazione di byte diversi.

Aggiunto upload separato dei soli eseguibili di build fallite per diagnosi;
questi artifact non sono archivi qualificati e non soddisfano M8. Le build
fallite non pubblicano l'artifact finale ui-package-TARGET.

Conferma aggiuntiva locale: anche un progetto Cargo minimo con build.rs e
include! di codice generato sotto OUT_DIR produce metadati e binari identici
su due target directory rimappate; non solo rustc diretto. Non sostituisce il
gate dell'app completa. Windows del primo run mostra la stessa classe di errore:
21744128byte per build, primo scarto0xf8. Entrambi i negative-font falliscono
sul modulo condiviso mancante nel fixture, non sullo stile o sul renderer.

Regressioni locali sul codice prodotto d073c4b:816esiti positivi/107riepiloghi
per ciascuno degli assetti workspace default/all-features (includono helper e
doctest),0fail/4ignored. Nuovo fixture negative-font effettivo positivo, test
remapping/tooling e Clippy positivi. Nuova qualifica nativa ancora richiesta.

## Secondo gate —41028902538 su eaf221b

Riproducibilità superata su ENTRAMBI i runner: confronto integrale e nuova
verifica indipendente sui due eseguibili scaricati, SHA/dimensioni in
T-P16-08-reproducibility-eaf221b.json. Linux sviluppo positivo, Windows ancora
in corso alla terza lettura ordinaria. Nessun archive finale qualificato.

Il checker Linux intercettava /tmp/.X11-unix/X: è il prefisso del socket OS
formattato da x11rb-protocol0.13.2 (src/parse_display/connect_instruction.rs:42),
non un percorso di build. La policy UI GNU ora ammette solo quel letterale UTF-8
terminato/bounded, mai continuazioni di path, UTF16, altre directory /tmp,
radici di build esplicite o policy CLI. Tutte le occorrenze sono controllate,
anche dopo il letterale ammesso. Test positivi/negativi e17test tooling passano;
il verifier aggiornato passa sul vero binario Linux scaricato, OpenSSL assente.

Windows si fermava sull'import comctl32.dll, OS Common Controls richiesto dal
backend: qualificato nella sola allowlist UI. Fonte Microsoft primaria:
https://learn.microsoft.com/en-us/windows/win32/api/commctrl/nf-commctrl-setwindowsubclass
Import table completo registrato; nessuna DLL esterna o CRT dinamica ammessa.
La lettura PE e dei path su Linux non sostituisce l'esecuzione Windows; cargo tree
MSVC offline non disponibile sullo stesso host, quindi il gate resta nativo.

Sul PC Linux, il vero binario CI eaf221b è stato copiato in un'installazione
provvisoria con PATH vuoto: versione e frame software positivi. Il display non
è accessibile dalla sandbox; fuori sandbox la finestra nativa si apre/chiude
correttamente (500ms), tutte le librerie osservate soddisfano la policy OS.
Prova in T-P16-08-native-local-probe.json; nessun dato reale o archivio storico
letto/modificato. Font/stile12file preservati contro il commit dell'autore ee33d64,
manifest in T-P16-08-style-preservation.json. Qualifica completa archive CI ancora
necessaria prima di M8; nessuna nuova release/tag pubblicata.

## Terzo gate —41033360461 su8695b72

Qualifica nativa completa riavviata dopo i due correttivi del verifier. Alla
prima lettura ordinaria tutti4job nativi sono in corso; nessun esito finale
presunto. I precedenti eseguibili Linux/MSVC restano prova di riproducibilità
su eaf221b, non sostituiscono il gate della revisione corrente.

Prova aggiuntiva del protocollo archive locale: il vero eseguibile Linux eaf221b
è stato impacchettato, ispezionato ed estratto in una directory temporanea.
Inventario esatto5entry, digest, versione/source e frame software con PATH vuoto
positivi; dossier T-P16-08-local-archive-probe.json. La prova non pubblica un
archivio e non è presentata come qualifica nativa finale della revisione8695b72.

### Regressione del test di contesa su MSVC

Nel run41033360461 il packaging Linux è completed/success e l'archivio reale
è stato scaricato e ispezionato (inventario, licenze, digest e provenance esatti).
La suite Windows fallisce invece nel test SQLite a due processi: esiti[3,3],
nessuna pubblicazione, contro l'assunzione del test di un vincitore obbligatorio.
Con busy_timeout=0 il lock di lettura della risoluzione può impedire anche
il COMMIT del secondo writer: la sicurezza CAS non garantisce la liveness
del singolo tentativo. Non è una modifica al contratto dell'adapter.

Corretto solo il test: ammessi zero o un vincitore iniziale, mai due. Nel caso
zero si verificano generazione1, assenza di entrambi gli invii e snapshot2
assente, prima di una sola nuova transazione esplicita. Gli esiti ambigui
restano risolti dai figli tramite connessione nuova prima di restituire3.
Aggiunta una seconda esecuzione con vero lock SHARED che forza deterministicamente
il caso[3,3] e verifica la successiva pubblicazione coerente. Nessun retry
automatico nel runtime né nelle scritture dei contendenti.17test SQLite passati
(2helper ignored, sottoprocessi eseguiti); fmt e Clippy workspace/allfeatures
positivi. Necessario nuovo gate regressioni MSVC; packaging Windows ancora
in corso alla terza lettura del run, senza qualifica presunta.

### Ispezione indipendente degli archivi scaricati

Entrambi i job packaging8695b72 sono completed/success: build riproducibili e
probe installati software/nativi con PATH vuoto effettivamente passati anche
su Windows. L'ispezione Linux dell'archivio Windows trova però i testi licenza
convertiti da Git in CRLF: IBM4456byte contro4363LF, Slint3663 contro3620LF;
il solo cambio CRLF→LF produce byte identici. I digest interni sono corretti,
ma l'archivio Windows non supera il confronto licenze rispetto al checkout Linux.
Non dichiarato pienamente qualificato né attenuato il checker. Aggiunte due sole
regole text eol=lf in .gitattributes per i testi distribuiti: nessuna modifica
ai testi, font, renderer o schemi. Un nuovo gate completo include anche4e44703
e sostituisce la run di sole regressioni41036244477, resa superata dal difetto
concreto individuato nell'archivio.
