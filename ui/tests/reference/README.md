# Baseline visiva eseguibile — Copper Ledger

`light-1x.png` e `light-2x.png` sono le due reference Signature del commit5720748,
qualificato dalla CI40958452017. Spostate senza cambiare alcun byte dopo il
refactoring documentale dell'autore ee33d64; SHA256 e percorsi originali in
`docs/verification/UI_REFACTOR_PREFLIGHT_2026-10-30.json`.

Nessuna nuova acquisizione o modifica di tolleranza. Il test legge le reference,
non le rigenera. I vecchi mockup e le capture storiche restano nella cronologia
Git; non viene ripristinata la cartella documentale rimossa dall'autore.

Design corrente: docs/ui-design-documentation/DESIGN.md e
LINTRA_DESIGN_DIRECTION.md. Renderer software, IBM Plex incorporati, due scale;
CI confronta anche l'intero RGB fra Linux/MSVC senza tolleranza.
