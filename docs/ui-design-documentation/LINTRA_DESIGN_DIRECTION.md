# Lintra — direzione Signature «Copper Ledger»

2026-10-29 · Fase A completata · riferimento canonico per il redesign autorizzato.
Il prompt Signature supera soltanto le precedenti indicazioni estetiche. Font
incorporati, architettura, callback, disponibilità e significato dei risultati
restano vincolanti. DESIGN.md è l'indice canonico; i token eseguibili sono Tok.

## Ricerca eseguita e scelta

Skill autorevole: `.codex/skills/ui-ux-pro-max/SKILL.md`, letta integralmente.
Script reale `scripts/search.py`: ricerca design-system del brief, poi style
`data dense dashboard minimalism brutalism` e UX `keyboard focus table loading
error accessibility`. Output originali in `signature/pro-max-*`.
Nessuno stack web forzato: Slint 1.18.1, renderer software, cinque font già
incorporati. La landing personalizzata proposta dalla ricerca è incompatibile
con il prodotto. Nessuna analitica, grafico o capability aggiunta per seguirla.

| Alternativa | Identità / operatività | Decisione |
|---|---|---|
| Dashboard blu con KPI | Leggibile ma generica; KPI assenti nel prodotto | Scartata |
| Minimalismo editoriale estremo | Riconoscibile ma troppo vuoto per diagnostici | Scartata |
| Copper Ledger: banco di lavoro grafite/avorio/rame | Navigazione numerata, griglia densa e ispettore, contrasto netto; fattibile con primitive native | Scelta |

La densità e il percorso riepilogo→tabella→dettaglio derivano dalle ricerche.
Il rame e la composizione sono una scelta di prodotto motivata dal brief: un
segnale caldo riconoscibile su superfici sobrie, senza estetica clinica o SaaS.
Nessun font nuovo: IBM Plex collega titoli, dati e strumenti ingegneristici.

## Baseline e inventario

Commit iniziale `5197a98`, branch `work/p1-foundations`, Git pulito.
Sei file Slint in `ui/ui`: main, tokens, controls, analysis, results, export.
Controller Rust: configurazione/catalogo/browser; worker/cancellazione;
risultati/filtri/indice lazy/dominio; export/Informazioni. Nessuna UI storica,
nessuna scelta Full: non aggiungerle. I nomi di flusso/tracciato vengono dai
modelli. I report e la provenienza dei risultati rimangono legati al run visto.

Harness `ui/tests/signature.rs`: componenti di produzione con input di
presentazione sintetici, clock fermo, due temi, 1366×768 / 1920×1080 /1920×1200,
1366×768 a2× e minimo640×480. Non è un mockup né prova di validazione normativa.
Test esistenti separati verificano servizio e callback reali.

Difetti osservati nelle capture iniziali analysis1366 e results1920:
- Estetica: pannelli omogenei, azioni tutte con lo stesso peso.
- Gerarchia: esito piccolo, titoli dei pannelli quasi equivalenti ai dati.
- Layout/densità: tabella e dettaglio ad altezza fissa anche a1080px; molto
  spazio inutilizzato, messaggio completo sotto la viewport del dettaglio.
- Navigazione: la destinazione Risultati compare soltanto dopo un run;
  manca un riferimento stabile alle due attività reali.
- Stati: export dichiarato non disponibile nel form benché implementato.
- Accessibilità: etichette dei filtri affidate ai placeholder; ordine visivo
  flusso/tracciato/versione contrario alle dipendenze versione→tracciato.
Questi rilievi sono limitati alle schermate effettivamente osservate.

## Token e grammatica

| Ruolo | Chiaro | Scuro |
|---|---|---|
| Fondo | #F5F4F0 | #101112 |
| Superficie | #FFFFFF | #1B1D1F |
| Superficie secondaria | #ECECE7 | #25282B |
| Testo | #20272B | #F5F4F0 |
| Secondario | #56616A | #B6BABC |
| Bordo | #CBCFCF | #60676B |
| Riga | #E3E5E2 | #34383A |
| Azione / focus | #A13E1A | #FF9C66 |
| Selezione | #F9EDE4 | #392A21 |
| Violazione / errore | #AE2925 | #FF9288 |
| Fondo errore | #FCEDEA | #3B2423 |
| Inconcludente / parziale | #855408 | #EFC177 |
| Navigazione | #20272B | #101112 |

Nero/grafite e rame; nessun gradiente viola. Il rame identifica azioni e
selezione, mai certificazione. Rosso accompagnato dal testo dell'esito; ambra
per parzialità. Nessun verde associato automaticamente a fine esecuzione.
Raggi4px controlli,6px pannelli, bordi1px, focus2px. Niente ombre o blur.
Spazi4/8/12/16/24/32. Titoli28px/600, sezioni18px/600, corpo14px,
metadati12px; codice e valori IBM Plex Mono12px, numeri allineati a destra.
Titoli diretti; nessun soprattitolo ornamentale (rifinitura Impeccable). Righe36px, header36px.

## Composizione e componenti

Barra di prodotto64px: marchio geometrico nativo, Lintra, navigazione stabile
01 Analisi /02 Risultati; risultati disabilitati senza report. Temi e
Informazioni a destra. Su640px nomi compatti e controlli ancora raggiungibili.
Footer28px per versione/ambito. Nessuna sidebar ornamentale.

Analisi: titolo e contesto; colonna configurazione circa62%, esecuzione38%.
Sezioni numerate File /Schema /Report; selettori in ordine flusso→versione→
tracciato. Il riepilogo usa le selezioni effettive. Stato esecuzione distinto
con bordo semantico, testo e contatori reali, nessuna percentuale inventata.

Risultati: riepilogo alto con heading forte, ambito esplicito e export vicino
all'esito. Tabella e ispettore affiancati; altezza cresce con la finestra.
Filtri con etichette permanenti. Colonne tecniche monospace, messaggio espanso
nel dettaglio; nessun ordinamento nuovo simulato. Righe selezionate con fondo
e marcatore laterale. Dettaglio: codice, causa, campo/posizione, actual/expected,
dominio paginato e contesto. Dominio sempre dal controller esistente.
A larghezza ridotta pannelli impilati con scorrimento; tabella in viewport
orizzontale per non comprimere colonne a larghezze illeggibili.

Export e Informazioni: superficie di lettura delimitata, larghezza contenuta,
stessa gerarchia titolo/contesto/azione. Conferma overwrite esplicita invariata;
nessun automatismo. Browser: cartella visibile, lista nativa e azioni in fondo.
Stati vuoti: spiegare quale dato manca o come applicare una nuova ricerca.

ActionButton: primaria piena, secondaria bordata, navigazione su grafite;
hover senza spostamenti; disabilitato con testo leggibile. FieldEdit/ChoiceBox:
bordi leggibili, focus2px, popup con tastiera; ReportCheck conserva semantica.
Nessuna animazione obbligatoria: feedback immediato e nessun movimento superfluo.
Conservare Tab/Space/Return, frecce/Home/End/PageUp/PageDown e focus visibile.
Non duplicare cache/righe, scansioni o domini sul thread UI.

## Consegna vincolante a Impeccable

Applicare questa direzione ai sei file Slint e componenti condivisi. Conservare
proprietà/callback pubblici e logica core. Migliorare layout e resistenza dei
contenuti lunghi, verificando capture prima/dopo identiche. Ogni correzione ai
token va registrata qui. Non inventare severity per righe prive di quel dato.
La Fase C revisiona soltanto le stringhe risultanti, senza toccare identificativi,
valori, provenienza o diagnostici contrattuali. Riferimenti PNG precedenti
restano storici e non si sovrascrivono. Nuovi riferimenti Signature devono essere
espliciti, inventariati, con controlli geometrici stretti e confronto nativo CI.

Correzione di Fase D: disabled-ink chiaro#606A70, dopo misura4,26:1 della
prima proposta#677077 su#ECECE7. Il nuovo valore deve superare4,5:1 anche
per i controlli disabilitati; nessuna soglia abbassata. Eliminati i token
geometrici del mockup ormai inutilizzati, senza modificare il rendering.
