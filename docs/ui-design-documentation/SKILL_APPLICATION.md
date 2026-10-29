# Skill UI — applicazione allo spike nativo

Richieste dall’autore: no-ai-slop, Impeccable e Hallmark; lette dalle rispettive
SKILL.md. Impeccable context eseguito una volta con target del prototipo esterno;
il rilevatore automatico non riconosceva il contenuto Slint, quindi l’autorità
visiva effettiva deriva dal codice e dai PNG inventariati, non da quel flag.
Contesto esistente, non progetto grafico da reinventare; nessuna intervista
ripetuta o proposta di tema estraneo. Materiale/pubblico/uso sono specificati dal
progetto e dall’autore. Nessun subagente, secondo governance esplicita.

## No-ai-slop

Testi del mockup conservati, nessun nuovo slogan, claim o numero inventato.
I numeri125.430/318/411 e gli errori E003/E005 sono fixture del mockup e rimangono
accompagnati da «Dati di esempio a scopo illustrativo». Non certificano output
del motore. La UI di produzione dovrà usare dati reali da ValidationService;
nessuna funzionalità di validazione simulata nello spike o trasferita al core.

## Impeccable

Modalità Operate, riferimenti craft-floor/operate/audit consultati. Verifica
mediante capture software native1×/2×; gerarchia tabella/dettaglio, palette
fornita, contrasto e stati come criteri di audit. Il detector browser/CDP non
qualifica Slint. Il nuovo selettore temi è autorizzato ma appartiene al bootstrap
UI successivo: chiaro fedele, scuro nero con accenti rosso/arancione.
Nessuna inversione automatica o sostituzione della palette chiara.

## Hallmark

App-screen esistente, source preflight cached in .hallmark/preflight.json.
Consultati flow§App screens/Harnesses, gates e interaction-and-states.
Il comando sloplint --scope app --render sul sorgente .slint risponde
«no .html, .css, .jsx, .tsx, .vue, .svelte or .astro files found».
Non è un render pass né un audit CSS superato: non si crea una finta pagina
web per farlo passare. Si trasferiscono invece token/contrasto/stati/focus e
onestà dei dati, con screenshot reali e confronto pixels nativi. Nessun CSS
stamp o theme draw applicabile; design fornito vincolante, niente foto o hero.

## Risultato corrente e limiti

Prima capture del prototipo mostra un layout diverso dal riferimento: tabella
704px invece di800, dettaglio troppo largo e gap fra intestazione/righe.
Correzioni circoscritte ai token di geometria e layout della copia; palette
originale immutata. Controlli filtro/export restano quelli del prototipo, non
azioni applicative implementate; focus/accessibilità e tutti gli stati saranno
verificati nel perimetro delle task UI. Non viene dichiarato un audit finale
senza queste evidenze, né approvato un nuovo riferimento dalla capture corrente.

## Applicazione al bootstrap16-02

Modalità Operate e craft-floor riletta prima dell’editing. Nessun context reload
(una volta per sessione), nessuna nuova visual world: font/colori chiari autorità
invariata. No-ai-slop: solo nomi controlli e versioni vere, nessun claim/metriche
fittizie. Impeccable: rendering visto chiaro/scuro, token unici, contrasto
misurato>=4.84:1, tastiera/Tab/focus/puntatore e viewport minima640×480 provati.
Hallmark: preflight integrato con target prodotto, sloplint sullo Slint restituisce
no supported web files: non è un audit CSS passato. Evidenze native e test reali
sono la verifica; nessuna conversione in HTML o riscrittura della palette fornita.
Riduzione movimento: non introdotte animazioni. Errori/cancel/report/stati analisi
non ancora presenti nel bootstrap, restano nelle task applicative successive.
