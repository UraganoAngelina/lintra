# Adapter desktop

Membro separato `lintra-ui`, dipendente dall’API pubblica `lintra`; core e CLI
restano nel package originale. T-P16-01 stabilisce solo il confine workspace:
il binario segnala esplicitamente che l’interfaccia non è ancora disponibile.
Bootstrap Slint, font e temi appartengono a T-P16-02; nessuna finestra simulata.

`cargo build` continua a costruire la CLI; `cargo build --workspace` include
l’adapter. CI usa `--workspace` per Clippy/test e controlla il grafo effettivo
con `scripts/check_workspace.py`. `xtask` e il prototipo qualificato rimangono
workspace indipendenti. I riferimenti chiari approvati sono in docs/ui/review.
