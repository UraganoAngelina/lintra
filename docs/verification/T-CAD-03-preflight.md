# T-CAD-03 — sessioni CAD sul coordinatore core, 6 novembre 2026

READY → PREFLIGHT → IN_PROGRESS. Base dee2a87, branch work/p1-foundations.
Prerequisiti: CAD02 DONE/CI42067927663 su5dbfc174; PERSIST07 DONE,
CI40877202683 su1e92b1f0ad0a2d2f8adbde80ad8ddb90a1b2352d. G0 con
rinvii espliciti, nessuna promozione del candidato aFROZEN.

Fonti: prompt03, governance/master/piano§4/14/20, ARCH45, DET166–174,
PERSISTENCE_CORE_SPEC§3/4/7, CAD_RUNTIME_PROFILE historical,
CAD_NOTES_AUDIT e rules-and-joins, CAD_FIXTURE_PLAN/CAD-FIX-19. PDF regionale
consultato tramite estrazione locale; ciclo di approvazione Lintra deriva
specificamente dall'ADR approvata e dal profilo, senza attribuirlo al PDF.

Il componente versionato2024-06.namespace.1 già dichiara scope sending_ulss,
year/quarter, explicit approval, immutabilità, atomic_idempotent e missing policy.
Il coordinatore ApprovalDraft/ValidatedApproval/ApprovalPreview implementa questi
stati usando gli adapter core. La task03 configura/qualifica questa policy CAD,
non aggiunge un secondo formato, SQL o un executor clinico. Reconciliation e
vincolo progressivo-evento sono04; missing/preserve operativo05; temporali06–09;
gate completo12. I gate del compiler ordinario e audit-only restano attivi.

File previsti: tests/cad_sessions.rs; documento operativo versionato vicino al
componente dichiarativo; report/gate/indice/status/handoff. Test outcome di
componente espliciti, non output inventato del servizio Full: medesimo confine
pubblico ApprovalDraft già qualificato inPERSIST07, ora con profilo/PK/ULSS CAD.
Prove SQLite reali per T1/T2/T3, scarto/revisione draft, reject sessione approvata,
retry esatto anche dopo nuova generazione/restart, isolamento ULSS e anni,
abort/cancel/report failure/CAS. Proiezioni sintetiche protette, nessunaPHI.
Nessuna API nuova o rimozione di capability per far eseguire il profilo clinico.
