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

## Verifiche attuali e limiti

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
