# T-CAD-11 — lookup e condizioni cliniche, 6 novembre 2026

READY → PREFLIGHT → IN_PROGRESS. Task indipendente daCAD03, sospesa dopo
pubblicazione42f92ed e dispatchCI42190202683. Una sola task implementativa attiva.
P11 DONE:39959838976/8fc31f5, provider/preparazione/reference/summary reali;
P10 DONE estesa39920597392/c2e0d1d; G0 approvato407f89e con rinvii espliciti,
authoring72f9f5e. Gate successivi di prodotto ripetuti fino aCAD02.

Letti prompt11, governance/piano4/14/20, ARCH45, DET166–174, specifica persistente,
CAD_NOTES_AUDIT, CAD_FIXTURE_PLAN09/10 e decisioniCAD10/16/11/G05; candidato B1/D,
dataset reviewed pp5/6/7 e PDF estratto localmente. Tutte le fonti restano intatte.

Disponibili63codiciICD9 distinti con7prefissiV,33prestazioni e70posizioni bisogno.
Normalizzazione offline dichiarata: punto rimosso dai soli codici sorgente,
prefissoV e zeri prestazione conservati. Provider membership v1 già ratificato;
nessuna nuova API/grammar/normalizzazione runtime.70posizioni è catalogo sorgente,
non nuovo lookup sui valori del bitset: policyT70 già qualificata nella task10.

B1 coordinate reali2.3,2.27–2.33;2.28 required se UCPDOM e rete1, vietato altrimenti;
2.29 required inUCPDOM;2.30–33FAC inUCPDOM e vietati fuori. D5.5 appartiene alle
33prestazioni. Riuso condizioni dichiarate, dominio field e reference_data;
nessun driver/storia/temporale o lookupISTAT/ICPC aggiunto. Componente distinto
con sorgenti/hash e dataset membership versionati, non schemaCAD completoFROZEN.

File: schemas/cad/components/2024-06.clinical-lookups.1, tests/cad_lookups.rs,
report/ledger/indice/status. Test sul servizio/provider reali e record sintetici
alle coordinate reali, inventario/normalizzazione/63+33domini, FAC/forbidden/
required, assenze/valori invalidi e unavailable/degraded, JSONL deterministico.
Gate locali pertinenti, CI Linux/MSVC necessaria prima diDONE11; nessuna release.
