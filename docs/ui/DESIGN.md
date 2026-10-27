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

## Nuova analisi16-03

Fonte visiva: `lintra_ui_html/Nuova analisi (in esecuzione).png`,2×,
canvas1280×720. Card configurazione (24,80,604,420), esecuzione
(652,80,604,516), padding24/gap24. Il layout sotto1100px impila i pannelli
in un’area scorrevole; minimo640×480. Cinque font e palette del prototipo
restano la fonte; header#16232E ereditato dal sorgente Slint, mentre il PNG
HTML mostra#14202B. Nessun nuovo riferimento approvato sostituito.

Il PNG rappresenta un’esecuzione con numeri illustrativi. In16-03 il pannello
mostra stato reale della configurazione e avvio indisponibile: non riproduce
percentuali, contatori o animazioni fittizie. Anche Windows1252 illustrativo
non è proposto: il motore corrente espone UTF8. File, schema e encoding
rimangono selezioni esplicite. La scelta del flusso azzera versione/tracciato;
quella della versione azzera il tracciato. Editing del percorso invalida il
vecchio campione. Nessuna etichetta «ultima versione» o certificazione implicita.
I checkbox sono intenzioni di export, non file già prodotti.

Controlli nativi personalizzati con focus, tastiera e semantica accessibile;
selettore cartelle integrato senza runtime GTK/portal o dipendenze nuove.
Impeccable Operate applicata alla gerarchia fornita, Hallmark in-thread a
palette/token/focus/contrasto e capture native; no-ai-slop ai testi aderenti
alle capacità. I detector HTML non costituiscono un gate Slint.

### Skill UI aggiornata

ui-ux-pro-max aggiunta su mandato autore. Eseguiti `--design-system` e ricerca UX
focus/keyboard/errori. Suggerimenti generici lead-magnet, font Figtree/Noto e
palette blu non applicabili al mockup vincolante; nessuna sostituzione del design
approvato. Applicati focus indipendente dal valore nei checkbox, verifica reale
puntatore/Space e testi/stati senza claim di esecuzione. Guide web/375px non
attestano supporto nativo: il contratto desktop resta640×480 minimo.

## Worker16-04

Stesso layout/token/font e due temi, nessuna palette sostitutiva dal generatore
ui-ux-pro-max. Avvia/Annulla nello stesso punto, annullamento nei token error
rosso; form bloccato durante l’analisi, temi/Informazioni ancora utilizzabili.
Fasi e contatori sono eventi reali, senza percentuale o totale inventati.
«Annullamento richiesto» non diventa Cancelled prima del risultato del servizio.
Violazioni e errore tecnico hanno testi distinti; completezza parziale resta
visibile, nessun verde implicito per un risultato non conclusivo. Un errore
invalida la descrizione del precedente campione e indica come ripetere il controllo.

no-ai-slop applicata ai messaggi aderenti agli esiti; Impeccable Operate e craft
floor a feedback/cancel/focus, ui-ux-pro-max a progress/error recovery. Riferimenti
visivi forniti prevalgono su suggerimenti generici landing/font/palette.
Capture native di running/errore/cancel sono diagnostiche, non nuove baseline
approvate. Test Slint con proxy event-loop headless e barriera controllano il worker
senza eseguire validazione sul thread UI né usare numeri dimostrativi.
