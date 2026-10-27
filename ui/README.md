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
La configurazione prepara una ValidationRequest e un vero ValidationService;
non esegue validazione. Avvia resta disabilitato fino alla task worker16-04.
Risultati ed export appartengono alle task successive: nessun esito simulato.

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

Palette chiara identica ai token del mockup approvato; scura nera con accenti
rosso/arancione. Il selettore risponde a puntatore e tastiera, con focus visibile
e nomi/stati accessibili. La preferenza resta nello stato della finestra:
nessun file di impostazioni o persistenza implicita introdotto.

`cargo build` continua a costruire la CLI; `--workspace` verifica anche la UI.
CI Linux/MSVC esegue test software a1×/2× e confronta RGB byte per byte per
entrambi i temi. Le capture bootstrap sono diagnostiche: i riferimenti approvati
della schermata Risultati restano in docs/ui/review, senza sovrascritture.
La verifica/distribuzione dell’artefatto UI completo rimane T-P16-08.
