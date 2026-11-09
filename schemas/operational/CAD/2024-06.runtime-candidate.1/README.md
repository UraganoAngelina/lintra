# CAD 2024-06 — candidato runtime completo, 83 campi

Layout A218/B1198/B241/C45/D39, PK normative complete, namespace ULSS ed evento.
Il repository filesystem generico si configura con root `schemas/operational`,
chiave `CAD / 2024-06.runtime-candidate.1`. Le due membership sono in `data/`;
il preparatore reference usa questa directory di versione come root.

L'ingresso eseguibile è `HistoricalValidationService::validate_snapshot`, con
Full/Flow, `sending_ulss`, `year`, `quarter`, repository e proteggitore reali.
Restituisce ValidationOnly oppure una preview da rivedere e approvare tramite
`approve_reviewed`. La compilazione/validazione ordinaria e la CLI audit-only
continuano a rifiutare questo profilo: non sono esecutori snapshot.
Esempi completi eseguibili: `cargo test --locked --test cad_history_runtime`.

La composizione preserva ID, nomi, coordinate, note integrali e presenza della
fonte ricevuta; incorpora i componenti CAD02..11 già qualificati e dichiara
esplicitamente parent A, D→C a quattro campi, iniziale storica e codice15.
`source.control_resolutions` associa ogni nota non eseguibile precedente a un
esecutore o alla decisione di esclusione già approvata. Il controllo di branca
è la restrizione alla figura specialistica; non dichiara un catalogo specialistico
mai fornito. ICPC/ISTAT e controlli semantici degli identificativi non vengono
inventati; valgono le decisioni CAD03/CAD11 e G0 già registrate.

`scripts/compose_cad_runtime.py` riproduce il JSON dal kit originale con hash
verificati e dai componenti revisionati. Serve solo allo sviluppo; kit e Python
non sono dipendenze runtime. Il test confronta tutti gli83 campi con le tabelle
documentali ricevute e verifica separatamente PK e chiavi D→C.

Stato candidato: nessuna dichiarazione FROZEN o conformità clinica universale.
`source.full_normative_compliance=false` conserva tale distinzione. I gate di
esecuzione e il loro commit sono registrati nel dossier T-CAD-12, fuori dallo
schema: cambiare metadati di qualifica nel JSON cambierebbe il suo hash semantico.
