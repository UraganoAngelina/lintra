# T-P16-00 — copia isolata del prototipo Slint

Origine: mockup_slint_UI/lintra-ui-Rust_slint fornito dall’autore. Hash sorgenti,
font e immagini originali in docs/ui/MOCKUP_INVENTORY.json. Gli originali non
sono modificati. Non è il modulo UI di produzione né un nuovo workspace.
Il core/CLI restano senza dipendenze Slint.

## Collaudo

Dalla root Lintra:

```sh
CARGO_TARGET_DIR=target/ui-prototype cargo test --manifest-path spikes/ui-prototype/Cargo.toml --release --locked --test qualification -- --nocapture
python -m unittest discover -s spikes/ui-prototype -p 'test_*.py' -v
```

Il test installa una piattaforma software headless e un orologio deterministico.
Produce in target/ui-prototype-visual due capture reali1×/2×, pixel RGB grezzi,
misure di scorrimento300k/1M con modello lazy sintetico e misure dei bordi
rispetto al PNG2× dell’autore. Il riferimento1× è una vista derivata con media
intera2×2 dell’export2×, esplicitamente distinta da un export originale o dalla
capture corrente. Non si autoapprova una baseline di regressione.

La CI ui-spike.yml qualifica Linux GNU e Windows MSVC e confronta direttamente
RGB1×/2× senza tolleranza; tempi ed encoder PNG non entrano nel confronto.
Latenze headless strumentate non equivalgono alla valutazione percettiva del
compositore desktop. Il buffer persistente misura il redraw incrementale;
misure precedenti NewBuffer restano documentate come ridisegno completo.

## Confini e correzioni

Slint/slint-build1.18.1 fissati dal lock originale, png0.17.16. Backend winit X11
software incluso per immagini/SVG e possibile lancio desktop; test senza display
con piattaforma esplicita, senza renderer GPU o system tray. Font IBM Plex
forniti incorporati; OFL.md contiene copyright/licenza.

Corrette nella copia larghezze pannelli/filtri, contiguità header-righe,
allineamento metadati e altezza righe del paragrafo: i font forniti hanno1.3em
naturali, il mockup richiede1.5em; line-height-factor=1.5/1.3. Palette chiara
invariata. Scuro nero/rosso/arancione e selettore temi sono un requisito della
UI successiva, non implementati in anticipo nello spike.

Demo source main.rs usa le quattro righe illustrative originali; il modello lazy
per stress test è una fixture di qualificazione, non un reader JSONL simulato.
Filtro/export e supporto di tutti gli stati/accessibilità appartengono alle task
UI successive. Nessun numero o errore mostrato è certificato dal motore.
