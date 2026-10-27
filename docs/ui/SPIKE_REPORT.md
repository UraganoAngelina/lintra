# T-P16-00 — qualificazione del prototipo Slint

Stato corrente IN_PROGRESS: collaudo locale e gate nativo in preparazione.
La seguente assenza di input è storica, superata dal materiale fornito.

## Storico — preflight iniziale

2026-10-27. Prerequisito CI T-P1-03 verificato dal ledger/run39232752518.
Il prompt richiede un prototipo esistente e PNG di riferimento esportati dal
mockup a1×/2×. Ricerca mirata nel workspace LINTRA (inclusi repository e kit):
nessun sorgente .slint, prototipo, mockup o PNG di riferimento trovato.

Stato BLOCKED sui soli input del collaudo. Geometria, scorrimento300k/1M e
congruenza Linux/Windows NON misurati. Nessun criterio dichiarato fallito:
l’assenza di input non autorizza il piano B egui/DEC-10. Nessun nuovo design o
riferimento generato per sostituire quello richiesto. Non è avviata P16-01.

Per sbloccare serve il percorso del prototipo e dei riferimenti1×/2×, oppure una
successiva decisione esplicita dell’autore sul perimetro UI. Nessuna richiesta
ordinaria di consenso per proseguire task indipendenti: T-P17-01 avviata su13-08
verificata durante l’attesa del gate15-07. ARCH/DET/piano restano invariati.

## Input forniti successivamente dall’autore —2026-10-27

Blocco iniziale sull’assenza del prototipo superato: origine
/home/alberto/Desktop/LINTRA/mockup_slint_UI. Inventario1188directory,19file
sorgente/asset in MOCKUP_INVENTORY.json; cache build censita ma non importata.
Consultati sorgenti Rust/Slint, font, README/test, PDF del board e diagramma SVG.
Risultati.png2560×1560 è riferimento2× del canvas1280×780; Nuova analisi
2560×1440 è l’altra schermata. Non reperito un export originale1×.
La fonte è conservata intatta; copia isolata in spikes/ui-prototype, senza
workspace/core/CLI Slint. Palette chiara e font invariati, tema scuro futuro
nero/rosso/arancione e selettore autorizzati dall’autore in AGENTS/DESIGN.

## Qualificazione in corso

Slint1.18.1 fissato come lock originale; test software con piattaforma headless
e orologio deterministico, nessuna GPU/display. Vec globale di errori non
introdotto: modello lazy sintetico esplicito con quattro seed e contatore di
row_data, non un reader JSONL di produzione.300k/1M righe misurate senza
generarle tutte. Capture reali PNG/RGB a1×/2×, originali mai usate per
autoapprovare la regressione. Il confronto CI nativo è senza tolleranza sui
pixel dei due OS; i tempi non sono confrontati come immagini.

Prima prova debug: circa323/324 richieste su300k/1M, ma latenze frame elevate;
non è prova di fluidità release. Misure ottimizzate ancora in corso.
Capture prima del fix rivela tabella704px anziché800, dettaglio troppo largo,
spazio spurio fra header e righe; corretto nella sola copia di collaudo tramite
token dimensioni/layout, senza cambiare palette. PNG del mockup conserva hash
originale, nessuna baseline ricavata dall’output corrente.

Build iniziale con feature minime fallita sulle icone SVG dei widget; il solo
image-default-formats non attivava il decoder richiesto e chiedeva EXR assente
in cache. Corretto selezionando backend-winit-x11 insieme al renderer software,
che porta il supporto SVG/immagini necessario, senza default GPU/system-tray.
Il test headless installa comunque esplicitamente la propria piattaforma.
Nessun errore di questi tentativi è registrato come gate superato.

STATO IN_PROGRESS: geometria e deviazioni note da verificare, tempi release e
CI Linux/MSVC ancora pendenti. Nessun16-01 prima dei criteri reali16-00.

## Gate locali effettivi —2026-10-27

Fmt e Clippy --all-targets --release --offline --locked -D warnings positivi.
Test qualification ottimizzato positivo, comparatore Python con inventario
vuoto/mancante, dimensione errata e differenza di un singolo canale positivo.
Actionlint1.7.12 positivo (shellcheck/pyflakes esterni non disponibili).
Tree del prodotto verificato senza Slint; Cargo.toml/lock/src del prodotto
invariati.19asset originali ricalcolati: tutti gli hash preservati.
Tutti i color token chiari confrontati letteralmente con tokens.slint originale:
identici. OFL/font incorporati verificati, nessuna trasformazione degli originali.

Otto ancoraggi dei bordi esterni a2×:0/1/1/0/0/1/0/2pixel fisici di scarto,
cioè massimo1pixel logico. Il riconoscimento del colore tollera solo la
miscelazione AA del bordo con la superficie (es.e2e6ec), senza cambiare la
misura della posizione. Nessuna soglia globale3% che nasconde lo spostamento
di un pannello. Il test ora richiede massimo2pixel fisici agli otto ancoraggi.
Questo non sostituisce il controllo visivo di tutti i glifi/controlli.

Corrette spaziature del dettaglio e interlinea: hhea dei due font forniti
1025/-275/1000 dà1.3em naturali; line-height-factor1.5/1.3 mantiene i glifi14px
con paragrafo1.5em. Label/value allineati inline come nel mockup.
Rimane una differenza osservabile di glifi/ritorni a capo fra PNG e renderer:
non viene attribuita automaticamente alla sola AA, né nascosta nei gate.
I font usati sono esattamente quelli del prototipo fornito, non sostituiti
per approssimare a occhio il PNG. Riferimento di regressione da approvare.

Misure release con buffer persistente,30eventi/frame a ogni dimensione:
300k/1M richiedono323/324righe e avanzano fino agli indici313/613; non basta
il solo contatore per dimostrare lo scrolling. P95 locale circa12–13ms a1×,
16–17ms a2×, inclusi dispatch/layout/render headless. Nessuna garanzia sul
compositore desktop dedotta da questi numeri. NewBuffer precedente resta
misura worst-case di ridisegno completo(~14/25ms). Nel cambio1×→2× era necessario
il primo repaint completo del nuovo buffer: corretta una capture parziale
intercettata dal gate geometrico, senza sopprimere il controllo.

Il PNG1× originale non era presente: la vista mockup-derived-1x.png è derivata
nel test dal PNG2× dell’autore con media intera2×2 e provenienza esplicita; non
è un export originale né una baseline autoapprovata dal render corrente.
Capture e dati locali in target/ui-prototype-visual, evidenza persistente
[local-gates](../verification/T-P16-00-local-gates.json).

## Residui per la chiusura

CI ui-spike.yml Linux/MSVC e confronto RGB1×/2× ancora da avviare/verificare.
Prima approvazione visiva prevista da ARCH39D.5 ancora pendente: nessun PNG
prodotto dall’app promosso a golden. Rivedere differenze tipografiche, scrollbar
fluent-light e controlli filtro/export del prototipo (non ancora azioni
applicative). Nessun pass della UI di produzione o16-01 implicito.
Skill application dettagliata in SKILL_APPLICATION.md; selector/dark theme
richiesti sono registrati e pianificati nel bootstrap successivo, non ignorati.
