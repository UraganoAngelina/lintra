# Evidenze T-CAD-03

Preflight e spiegazione dei test sono nei documenti adiacenti
T-CAD-03-preflight.md e T-CAD-03-sessions.md. Il ledger dei gate locali registra
comandi, exit code, durata e hash prima/dopo: T-CAD-03-local-gates.json.

preliminary-first.log conserva l'erroreRust di shadowing della fixture;
preliminary-second.log conserva rifiuti filesystem e summary incoerente;
preliminary-third.log conserva la prima suite mirata positiva dopo i fix.
Nessuno di questi log viene riscritto o presentato come un gate diverso.

gate-N.log appartiene alla sequenza finale su sorgenti fermi. Windows è
qualificato soltanto dal runCI del commit pubblicato. Il gate precedenteCAD02
non qualifica questi nuovi test. Le fixture Full sono input del componente
ApprovalDraft, non prova di esecuzione della pipeline clinicaCAD completa.
