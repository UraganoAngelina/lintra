# LINTRA SIGNATURE UI OVERHAUL REPORT

2026-10-29 · single-agent · branch `work/p1-foundations`.
**Stato: Fasi A/B/C realizzate, review visiva D conclusa; gate nativi e release in corso.**
Questa intestazione sarà aggiornata solo con gli esiti effettivi.

## A. Utilizzo verificato delle skill

Le skill sono state applicate nella successione richiesta; ogni output è
persistito prima di passare alla fase seguente.

1. **UI UX Pro Max**: letta integralmente la copia autorevole
   `.codex/skills/ui-ux-pro-max/SKILL.md`. Eseguito realmente
   `python3 .codex/skills/ui-ux-pro-max/scripts/search.py` con il brief del prompt,
   `--design-system -p Lintra -f markdown`, poi due ricerche mirate `--domain style`
   e `--domain ux`. Output originali in [signature/](signature/), file pro-max-*.
   La ricerca restituisce Data-Dense Dashboard e Drill-Down Analytics, ma anche
   una landing personalizzata inadatta. Sono stati adottati densità, percorso
   riepilogo/dettaglio e focus; esclusi landing, KPI, font nuovi e stack web.
   Direzione, alternative e token sono registrati in
   [LINTRA_DESIGN_DIRECTION.md](LINTRA_DESIGN_DIRECTION.md), input della Fase B.
2. **Impeccable**: letta la versione4.8.0 installata in
   `/home/alberto/.agents/skills/impeccable/SKILL.md`. Eseguito il vero launcher
   `scripts/impeccable context --target ui/ui/main.slint`; consultati new-work,
   operate, craft-floor, harden, polish, init e procedure degradate di review e
   documentazione. Il resolver non riconosce Slint: contesto ricostruito dal codice
   e dalle capture, nessun verdetto HTML inventato. Direzione A mantenuta;
   refactoring e verifiche preliminari in [phase-b.md](signature/phase-b.md), con
   screenshot corretti in phase-b/. Nessuna intervista estetica aggiuntiva,
   subagente o server in background: il mandato dell'autore delega lo stile e
   impone esecuzione diretta in sequenza.
3. **No AI Slop**: letti SKILL.md ed eval.md della versione installata in
   `/home/alberto/.codex/plugins/cache/openai-curated-remote/no-ai-slop/1.0.6/skills/no-ai-slop/`.
   Analizzati i testi dei sei Slint risultanti dalla Fase B e dei controller di
   presentazione. Inventari strings-phase-b.txt / strings-phase-c.txt;
   [phase-c.md](signature/phase-c.md) contiene glossario, dieci esempi prima/dopo,
   verifiche editoriali e semantica preservata. Modifiche concrete: ID campo,
   Cartella padre, Primi valori, conteggio delle diagnostiche trovate e istruzioni
   dirette. Codici, messaggi normativi, raw/logical, domini e provenance invariati.

## B. Risultato estetico e operativo

**Copper Ledger** usa superfici avorio/grafite e rame per azioni e selezione;
scuro nero/rame senza gradienti. IBM Plex Sans28px per titoli,18px per sezioni,
14px corpo; Plex Mono12px per dati tecnici. Bordo1px, focus2px, raggi4/6px.
Nessun font, libreria nativa, raster decorativo o servizio aggiunto.

La firma visiva è composta dal marchio geometrico Slint, dalla barra di prodotto
scura e stabile, dalle azioni primarie piene e dalla griglia di lettura tecnica.
I due passaggi reali Analisi/Risultati sono sempre riconoscibili. I risultati
restano disabilitati senza un report disponibile; non esistono schede storiche
fittizie. Il contesto schema/run rimane visibile e consultabile in Informazioni.

La configurazione occupa60% dello spazio, l'esecuzione40%. I selettori seguono
le dipendenze flusso→versione→tracciato. La tabella cresce in altezza con la
finestra, mantiene etichette di filtro persistenti e allinea a destra le righe.
L'ispettore presenta prima la causa, poi posizione, valori, dominio e contesto.
Su finestre strette pannelli impilati e tabella scorrevole orizzontalmente.

Violazione, errore tecnico e parzialità conservano testi distinti e segnali
rossi/ambra; nessun verde automatico a fine esecuzione. I dati del dominio
restano derivati dal contesto esatto del run, non dalle selezioni successive.

## C. Before/after

Baseline dal commit5197a98:80render software della MainWindow di produzione,
con fixture sintetiche dichiarate e clock fermo. Non sono mockup né prove di
validazione normativa. Stesso harness, contenuti, dimensioni e DPI prima/dopo.
Inventario e hash in [before.json](signature/before.json); confronto finale e
manifest [after.json](signature/after.json) registrano80coppie con dimensioni equivalenti.

| Vista | Prima | Dopo |
|---|---|---|
| Analisi1366×768 | [chiaro](signature/before/analysis-light-1366x768-1x.png) | [chiaro](signature/after/analysis-light-1366x768-1x.png) |
| Risultati1366×768 | [chiaro](signature/before/results-light-1366x768-1x.png) | [chiaro](signature/after/results-light-1366x768-1x.png) |
| Risultati1920×1080 | [scuro](signature/before/results-dark-1920x1080-1x.png) | [scuro](signature/after/results-dark-1920x1080-1x.png) |
| Risultati1920×1200 | [chiaro](signature/before/results-light-1920x1200-1x.png) | [chiaro](signature/after/results-light-1920x1200-1x.png) |
| Export1366×768 | [chiaro](signature/before/export-light-1366x768-1x.png) | [chiaro](signature/after/export-light-1366x768-1x.png) |
| Informazioni1366×768 | [chiaro](signature/before/information-light-1366x768-1x.png) | [chiaro](signature/after/information-light-1366x768-1x.png) |

Ogni coppia dimostra cambiamenti di composizione, gerarchia e uso della
viewport, oltre al colore. L'inventario include anche esecuzione, errore tecnico,
nessuna corrispondenza, browser,1366×768a2× e minimo640×480 nei due temi.
I test separati con il vero servizio provano callback, export e navigazione;
le capture del relativo report sintetico sono disponibili fra le evidenze CI.

## D. Qualità tecnica

Codice UI: `ui/ui/{tokens,controls,main,analysis,results,export}.slint`.
Solo microcopy in `ui/src/{export_controller,worker_controller}.rs`.
Test: signature.rs (nuovo), bootstrap.rs, results_window.rs, visual.rs.
CI: upload delle capture Signature nel workflow ci.yml, controlli esistenti
conservati. Documentazione: DESIGN, direzione, PRODUCT, note delle tre fasi,
inventari, report e aggiornamenti di stato/governance/README.

[IMPLEMENTATION_NOTES.md](signature/IMPLEMENTATION_NOTES.md) spiega componenti,
condizioni, callback, layout, correttezza e ragioni delle modifiche. Non cambia
nessun contratto motore/compiler/persistenza, dipendenza o schema normativo.

Regressioni individuate e corrette durante l'implementazione: collisione del nome
pointer con l'enum Slint, centraggio implicito che sovrapponeva pannelli/titolo,
widget AboutSlint troppo grande, coordinate di test ora sopra i nuovi filtri,
contrasto del testo disabilitato chiaro4,26:1. Non sono state aumentate tolleranze
o rimosse asserzioni per far passare queste prove.

Gate già superati sul nuovo codice: fmt, Clippy workspace all-targets nei due
assetti default/all-features, test workspace default (815 esiti positivi nei
riepiloghi, inclusi helper/doctest;0fail,4ignored). Capture80/80 e regressione
geometrica1×/2× passate; fonte immutata verificata con hash. Evidenze in
[local-gates.json](signature/local-gates.json) e log relativi. Test all-features,
core-no-bundle, release e CI del redesign ancora in corso.

Contrasto misurato: corpo13,76/17,18:1, secondario5,76/9,67:1,
selezione5,67/6,68:1, primaria6,52/7,36:1, disabilitato4,67/7,29:1
(chiaro/scuro, identico a1×/2×). [Misure](signature/contrast.tsv).
Nuovi riferimenti espliciti in signature/reference, nessun aggiornamento dei
riferimenti da parte dei test; prova negativa su un singolo canale di bordo.
La [review D](signature/phase-d-review.md) assegna ship alle correzioni visuali
elencate, con i limiti dichiarati. La CI precedente40908321779 è verde sui
cinque job, ma non sostituisce il gate del redesign.

## E. Valutazione critica

| Dimensione | Evidenza da valutare | Limite da conservare |
|---|---|---|
| Identità estetica | Marchio geometrico, grafite/avorio/rame, scala dei titoli, stessa grammatica fra viste | Giudizio di review dell'agente; non si attribuisce una nuova ratifica visiva all'autore |
| Qualità esecutiva | Componenti condivisi, focus, contrasto misurato, render1×/2× e più dimensioni | Test software headless; non prova percettiva su ogni monitor o lettore di schermo |
| Efficienza esperti | Più righe sulle finestre alte, causa subito leggibile, filtri etichettati, memoria della selezione | Al minimo il dettaglio richiede scorrimento; i messaggi contrattuali lunghi restano integrali |
| Estensibilità | Liste e nomi dai modelli reali, nessun ramo SIOC/CAD o numero di tracciati hardcoded | Nuove capability richiederanno task funzionali; lo storico non è simulato |

La revisione finale deve motivare il verdetto sulle capture effettive e sulle
prove, senza punteggi estetici inventati o dichiarazioni di assenza assoluta di
problemi. Il solo compilatore non soddisfa il requisito visuale.

## F. Handoff

Fonte canonica futura: [DESIGN.md](DESIGN.md), specifica completa in
LINTRA_DESIGN_DIRECTION.md, token eseguibili in Tok. Lo storico originale è
conservato in signature/DESIGN-before.md e docs/ui/review senza sovrascritture.
Stato e prossimo prerequisito reale in IMPLEMENTATION_STATUS.md; UI08 attende il
gate del nuovo assetto. PERSIST08 resta successiva al perimetro UI autorizzato.
