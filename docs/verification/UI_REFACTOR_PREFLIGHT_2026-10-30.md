# Preflight UI dopo ee33d64 —2026-10-30

Richiesta dell'autore: verificare rapidamente il refactoring prima di riprendere
le task, preservando il lavoro. Working tree iniziale pulito, nessun comando
precedente ancora raggiungibile; log delle vecchie suite completati.

Il commit UI REFACTOR sposta8 documenti in docs/ui-design-documentation e rimuove
capture/log storici; non modifica sorgenti Rust/Slint, manifest, lock o workflow.
Copper Ledger resta la fonte stilistica corrente. UI06/07 già DONE con evidenze
native40908321779/40958452017; non vanno ripetute come nuove implementazioni.

Residuo reale: visual.rs cercava due PNG eliminati. Recuperate esclusivamente
le2 reference eseguibili dalla revisione5720748, verificate identiche a HEAD^ e
collocate in ui/tests/reference. Nessuna rigenerazione, nuova soglia, palette o
geometria. La cartella docs/ui rimane rimossa. Aggiornati i puntatori correnti del
modulo; evidenze storiche continuano a essere consultabili nella cronologia Git.

Verifica locale fmt/check all-targets/render headless1×/2× PASS. Riprendere
T-P16-08 già in preflight, mantenendo116/140DONE e M8 aperta fino ai gate packaging.
