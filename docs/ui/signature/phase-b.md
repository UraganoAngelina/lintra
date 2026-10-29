# Fase B — Impeccable: registro di implementazione

Input vincolante: LINTRA_DESIGN_DIRECTION.md (Fase A), baseline before/,
inventario dei sei file Slint e controller, PRODUCT.md derivato dal brief.
Skill letta: /home/alberto/.agents/skills/impeccable/SKILL.md (4.8.0).
Eseguito il launcher reale `scripts/impeccable context --target ui/ui/main.slint`.
Il resolver non riconosce Slint né il design sotto docs/ui: consultati direttamente.
Riferimenti consultati: new-work, operate, craft-floor, harden, polish,
degraded/finish-reviewer e init. Applicate composizione Operate, densità,
gerarchia, stati e resistenza dei contenuti.

Adattamenti espliciti al mandato: nessuna nuova intervista o scelta di direzione
perché il prompt delega queste decisioni e vieta di sostituire la Fase A; nessun
subagente/documenter/reviewer parallelo o server in background. Review diretta
single-agent. Nessun detector HTML/CSS su Slint; non avrebbe un verdetto valido.
Nessun raster decorativo o comp artificiale; esclusivamente render di produzione.

Implementazione: token Copper Ledger, ActionButton con primaria e navigazione,
barra di prodotto stabile, form asimmetrico60/40 e ordine delle dipendenze,
tabella/ispettore ad altezza disponibile, filtri con etichette, colonne tecniche
in viewport orizzontale al minimo; causa diagnostica prima dei metadati;
export delimitato e Informazioni scrollabile con widget ufficiale.
Proprietà/callback Rust e logica core invariati. Gli stati non disponibili non
sono trasformati in successo; i messaggi tecnici e normativi restano dati.

Prima verifica: compilazione Slint reale riuscita dopo correzione della collisione
fra ID TouchArea `pointer` e valore enum mouse-cursor. Le capture di questa fase
sono generate in target/ui-signature-phase-b; Fase C attende la review di questo
render e usa le stringhe risultanti. Gate finale e limiti nel report conclusivo.

Chiusura di Fase B prima della revisione editoriale: screenshot corretti da test
reali in signature/phase-b/ (analisi, risultati, export, Informazioni). Bootstrap
1test e export_window1test passati. Nel test risultati hanno superato selezione
puntatore, tastiera, filtri e conservazione dettaglio; completamento della misura
300k/1M ancora in corso. Il primo tentativo ruotava sul nuovo filtro, non sulla
lista: coordinate corrette senza rilassare cache/limiti/assert. Capture di review
iniziale sovrapposte non sono adottate come riferimento.

Input alla Fase C: sei file Slint risultanti, controller di presentazione, questa
nota, direzione Copper Ledger, screenshot corretti e termini/contratti esistenti.

Esito successivo della verifica preliminare completa: results_window PASS,
138,68s debug,300k/1M a1×/2× con eventi rotella e cache≤128; signature PASS.
Non è una misura release. Dettagli finali e revisione D nel report Signature.
