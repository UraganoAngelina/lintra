# CAD-17 — componente calendario/inattività

Versione2024-06.inactivity.1, parziale dichiarata:25campi a coordinate originali,
cinque tracciati con PK complete. Non è lo schema CAD completo né un freeze.
Fonti e SHA256 nel component.json; tre mesi calendario/clamp e pause dei giorni
strettamente interni derivano da G04 ratificata. Nessun dato paziente.

Owner A con quattro insiemi B1/B2/C/D per evento e namespace dichiarato. B1
richiede consenso sull'intera tupla inizio/conclusione/motivo, senza filtrare le
valutazioni aperte. Le tuple C/D sono evento/data/tipo/figura: i progressivi
indipendenti rimangono nelle PK e non vengono confrontati per l'attività.
Ultimo C con almeno una D, altrimenti B1.2.4. Unione delle pause, nessun reset,
proroga a punto fisso e verifica alla fine del trimestre esplicito. D o altro
input necessario assente non equivalgono ad assenza di attività: Unknown.

Il periodo riusa snapshot.period. Il profilo compila come metadati; l'ingresso
eseguibile ordinario resta fail-closed sui profili snapshot combinati. Le prove
private al compiler usano compile_scoped e il vero motore a due passate, con
file sintetici e filler dichiarato per i campi omessi. Non simulano una preview
storica o attestazione clinica Full. L'integrazione completa è T-CAD-12.

Dettaglio di piani, funzioni, condizioni e test:
[report T-CAD-08](../../../../docs/verification/T-CAD-08-inactivity.md).
Nessun nuovo formato raw, storage/HMAC, dipendenza o API applicativa/CLI/UI.
