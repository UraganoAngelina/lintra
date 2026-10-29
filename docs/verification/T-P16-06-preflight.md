# T-P16-06 — export e Informazioni

Prerequisiti:16-05 DONE (CI40668541764/release40668592173) e12-04 DONE
(CI39974616511). PERSIST07 chiusa su1e92b1f/CI40877202683 tutti4job success.
Priorità autore29/10:06→07→08UI, con requisito diagnostici concordato.

File previsti: UI export worker/controller, controlli Slint e Informazioni,
replay streaming nel core per riusare CsvSink/JsonlSink e il decoder XLSX già
qualificato; test di contenuto/IO, threadUI e catture. Nessuna dipendenza nuova.
Report sempre del run catturato, completi e indipendenti dai filtri di tabella;
nessuna rivalidazione o ricostruzione normativa nella UI. XLSX stesso writer CLI.
JSONL/CSV accompagnati da riepilogo core con audit del bundle del run.
Destinazione esplicita, conferma per file esistenti, protezione file input,
pubblicazione atomica del singolo file e nessuna finta transazione multi-file.

Impeccable context eseguito una volta: launcher non riconosce Slint come visual
implementation, ma l'autorità reale è docs/ui/review e token/font già ratificati.
Estensione Operate dell'interfaccia corrente, non redesign; nessuna intervista o
nuova palette necessaria. Letti operate/craft-floor; no-ai-slop applicata ai testi;
ui-ux-pro-max design-system interrogato, suggerimenti web/palette/font ignorati
quando in conflitto col brief nativo IBM Plex chiaro/scuro approvato. Nessun
subagente nonostante direttive generiche delle skill: mandato autore single-agent.
