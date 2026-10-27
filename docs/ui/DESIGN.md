# Lintra UI — contratto visivo fornito dall’autore

Fonte: mockup_slint_UI, inventario/hash in MOCKUP_INVENTORY.json.
Superficie Operate: desktop per analisti che correggono errori di flussi regionali.
Schermate Nuova analisi e Risultati; il client chiama ValidationService, nessuna
norma nel codice UI. Riferimenti e numeri del mockup sono esempi, non risultati
realmente calcolati dal motore. Nessun nuovo claim o dato aziendale inventato.

## Tema chiaro — autorità visiva

Palette esatta tokens.slint: bg#F4F6F8, surface#FFFFFF, ink#14202B,
muted#4A5B6B, muted-on-dark#9FB0C0, border#D9DEE5, row-line#E4E9EE,
header-bg#EEF1F4, accent#0F766E, accent-tint#E6F4F1,
error#B42318, error-tint#FEE4E2, warn#B54708.
IBM Plex Sans Regular/SemiBold/Bold e IBM Plex Mono Regular/SemiBold forniti,
OFL1.1 incorporabili con licenza/copyright; nessun fallback di sistema.
Riferimento Risultati2560×1560 = scala2× del canvas1280×780.
Nuova analisi2560×1440 = canvas1280×720 a2×. Nessun PNG1× originale reperito.
Token/grafia/gerarchia esistenti prevalgono sui default delle skill di design.

## Temi — mandato aggiuntivo2026-10-27

Selettore chiaro/scuro obbligatorio nel modulo UI. Chiaro copia fedele della
palette sopra. Scuro nero con accenti rossi/arancioni; vietato il gradiente
nero/viola. La variante scura non riscrive i riferimenti chiari. Colori/typography/
spaziature/raggi tramite token unici; stati tastiera, contrasto, lettura dei dati
reali e riduzione delle animazioni verificati nel perimetro delle task UI.
Lo spike16-00 qualifica prima il prototipo chiaro; nessun nuovo tema implementato
in anticipo sul bootstrap16-02 né baseline generata dall’output corrente.

## Skill applicate

No-ai-slop per testi e controlli concreti; Impeccable Operate per gerarchia/stati/
leggibilità, Hallmark app-screen per token, contrasto/focus, assenza di effetti
ornamentali e metriche inventate. Slint nativo non è HTML/CSS: CDP e detector
browser non provano questo rendering; il collaudo usa software renderer e
capture PNG reali. Niente nuova pagina web o cambio di palette per seguire un
tema estraneo. Review in-thread, un solo agente come governance del repository.

## Bootstrap16-02

Token di prodotto in ui/ui/tokens.slint: stessi13colori chiari del sorgente,
font originali. Scuro bg#0B0D0F, surface#14181C, ink#F4F6F8, muted#B1BBC5,
border#64717D, header-bg#1D2329, accent#FF8A4C, accent-tint#372117,
error#FF7F70/error-tint#3B1C1A, warn#FFBA69. Nessun gradiente viola.
Header chiaro#16232E come il mockup; nel tema scuro usa il nero del fondo.
Selettore con primitive, stati selezionato/hover/focus, ruolo radio e tastiera;
AboutSlint ufficiale riservato a Informazioni per l’attribuzione richiesta.
Finestra vuota di base: nessuna nuova gerarchia dei risultati introdotta.
Confronto corpo/secondario/selezionato minimo4.84:1 nei due temi e scale.
Capture diagnostiche in bootstrap, non sostituiscono i riferimenti approvati.
