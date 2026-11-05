# CAD 2024-06.namespace.1 — componente T-CAD-02

Lowering esplicito di namespace ULSS, periodo trimestrale e PK normative da
CAD_RUNTIME_PROFILE.json/G-06 e cinque trace legacy del candidato, preservati
nel kit; digest sorgenti in component.json. Contratto generico ratificato in
CNF-CAD-NAMESPACE-BINDING-01. Nessuna modifica dei file ricevuti o FROZEN.

Non è un flow schema eseguibile completo. I test lo assemblano con campi
sintetici per verificare riferimenti, PK e piano; non dichiarano validazione
CAD clinica o riconciliazione snapshot già eseguita. La policy snapshot è una
dichiarazione già compilabile; attivazione operativa in T-CAD-03 e successive.
Il warning W_CAD_MISSING_TRACE è l'etichetta dichiarata del consenso esistente.

sending_ulss è stringa 501–509; anno e trimestre non entrano nel namespace.
Anno 1–9999 è il limite rappresentativo calendario del contratto di periodo
esistente, non una nuova finestra normativa degli invii; trimestre 1–4.
1.1 usa tre byte, estrazione logica del testo con padding destro dichiarato,
dominio locale e binding dopo il successo del campo. Altri campi/regole CAD,
join, storia progressivo-evento e temporali non sono aggiunti implicitamente.
C/D conservano il progressivo nella PK; nessuna nuova uguaglianza D→C.
