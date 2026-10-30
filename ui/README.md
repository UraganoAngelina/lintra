# Adapter desktop Lintra

Il membro `lintra-ui` dipende dall’API pubblica `lintra`; core e CLI restano
nel package originale e non dipendono da Slint. Bootstrap T-P16-02: finestra
base, selettore chiaro/scuro e Informazioni con il vero widget AboutSlint.
T-P16-03 aggiunge Nuova analisi: catalogo reale, versione sempre esplicita,
tracciato, file, UTF-8 e formati di report. Il controllo preliminare legge al
massimo64KiB, rifiuta BOM tramite il core e descrive soltanto il campione di
terminatori, senza modificare il file. Invio conferma un percorso digitato;
Sfoglia apre il selettore integrato (cartelle prima dei file). La sorgente dati
è visibile: stessa risoluzione CLI per directory esterne/ambiente/eseguibile,
poi bundle incorporato. Un bundle esterno incompleto o errato non consente fallback.

L’ambito attuale è LocalOnly sul tracciato selezionato, dichiarato nella schermata.
T-P16-04 collega Avvia al vero ValidationService su un thread dedicato. Il
progresso attraversa due canali da un solo elemento (progresso ed esito separati)
e Slint upgrade_in_event_loop, con throttling50ms dei contatori. Annulla imposta
il CancellationToken; lo stato resta «richiesto» finché arriva l’esito reale.
Violazioni, Err tecnico, Cancelled e completezza parziale restano distinti.
Non sono inventati totali o percentuali quando il servizio non li fornisce.

Gli errori fluiscono nel JsonlSink del core verso una directory temporanea nuova;
solo un run completo pubblica lo spool atomico. Nessun Vec globale di errori,
export fittizio o aggiornamento diretto di widget dal worker. La directory resta
owned finché il consumer usa il report; chiusura/cancel joinano il thread. La
finestra continua a processare eventi mentre la validazione è in corso.
T-P16-05 collega Risultati al JSONL completo: indice degli offset, lettura lazy
e cache massima di128 righe. I cinque filtri (codice, field ID, ID record, CF,
riga) usano confronti esatti combinati: Invio o Applica filtri avviano la ricerca
su un worker owned. Il riepilogo conserva conteggi, ambito e completezza del run;
il dettaglio resta stabile mentre la riga selezionata esce dalla viewport.
Una riga filtrata via cancella la selezione; errore di lettura e nessuna
corrispondenza hanno messaggi distinti. Puntatore e frecce/Home/End/PageUp/PageDown
permettono di navigare la tabella. Il report resta owned anche durante scansione
e lettura lazy, con cleanup dei reader prima della directory temporanea.

Export XLSX/JSONL/CSV tramite writer del core: le scelte formato sono
intenzioni, il salvataggio è esplicito dalla schermata Risultati. Nessuna modalità storica o norma è introdotta nel client.

```sh
cargo run --package lintra-ui --locked
cargo test --package lintra-ui --locked
python3 ui/check_missing_font.py
```

Richiesti un ambiente desktop nativo per l’esecuzione e i prerequisiti di build
del backend winit/software; i test installano una piattaforma headless e non
richiedono display/GPU. Slint e slint-build sono fissati a1.18.1, default GPU
disabilitati; il main seleziona espressamente il renderer software.

Font IBM Plex forniti incorporati con EmbedFiles e licenza OFL in ui/fonts.
La build fallisce se manca una qualsiasi delle cinque facce o se un asset non
è TrueType. Il controllo negativo usa una copia temporanea isolata, senza
rimuovere font o modificare il lock del repository. I font non si caricano da
un percorso della macchina dell’operatore.

Redesign Signature autorizzato il29/10: direzione Copper Ledger, chiaro
avorio/grafite/rame e scuro nero/rame. Il riferimento canonico è
`docs/ui-design-documentation/DESIGN.md`; il mockup precedente resta storico. Il selettore risponde a puntatore e tastiera, con focus visibile
e nomi/stati accessibili. La preferenza resta nello stato della finestra:
nessun file di impostazioni o persistenza implicita introdotto.

`cargo build` continua a costruire la CLI; `--workspace` verifica anche la UI.
CI Linux/MSVC esegue test software a1×/2× e confronta RGB byte per byte per
entrambi i temi. Le capture bootstrap sono diagnostiche: i riferimenti approvati
della schermata Risultati restano nella cronologia Git precedente al refactoring ee33d64.
La verifica/distribuzione dell’artefatto UI completo rimane T-P16-08.

Per qualificare lo scorrimento sul modello JSONL reale (300k/1M,1×/2×), eseguire
`cargo test -p lintra-ui --release --test results_window --locked`. Il test usa
dati sintetici dichiarati e salva tempi/offset/cache e capture diagnostiche in
`target/ui-results-release-visual` (`ui-results-visual` per debug); la versione debug verifica le funzioni e non prova
fluidità release. Workflow nativo esplicito: `UI disk-backed scrolling`.
La distribuzione del binario UI rimane16-08, nessuna baseline approvata cambia.


Il refactoring Signature mantiene callback e controller: navigazione stabile,
form60/40 con ordine flusso/versione/tracciato, tabella e ispettore ad altezza
variabile, filtri con etichette permanenti. Il minimo resta640×480; sotto1100px
le colonne si impilano e la tabella conserva uno scorrimento orizzontale.
Per ripetere le80capture di presentazione sintetiche (due temi, otto viste,
1366×768/1920×1080/1920×1200,1366×768a2× e minimo640×480):

```sh
cargo test -p lintra-ui --test signature --locked
```

Output predefinito `target/ui-signature`; `LINTRA_SIGNATURE_OUTPUT` permette una
directory esplicita (preferire un percorso assoluto, il test ha cwd ui/).
Queste sono capture di componenti di produzione con fixture di presentazione,
non evidenza di un run. I test worker/results/export usano il servizio reale.
Il test visual legge i riferimenti Signature revisionati, senza aggiornarli;
lo storico è consultabile nella cronologia precedente a ee33d64. Report in
`docs/ui-design-documentation/LINTRA_SIGNATURE_UI_OVERHAUL_REPORT.md`.


Packaging UI (tooling T-P16-08, qualifica nativa in corso):

```sh
cargo xtask build-release --product ui --target x86_64-unknown-linux-gnu --verify-reproducible --verify-artifact
```

Il prodotto predefinito resta la CLI. La UI ha una policy di librerie propria;
non usare il gate CLI per certificarla. Il workflow `UI native packaging` produce
archivi con eseguibile, provenienza, checksum e licenze dei font/Slint, verificati
su installazione temporanea. Non pubblica release né sovrascrive tag esistenti.
Il binario estratto non richiede Rust, Cargo, Git, SQLite o font separati.
Richiede il runtime desktop OS documentato: X11/input/Fontconfig su Linux, DLL OS
su Windows. Il gate su queste dipendenze deve essere realmente superato prima
che la task oM8 siano dichiarate completate.
