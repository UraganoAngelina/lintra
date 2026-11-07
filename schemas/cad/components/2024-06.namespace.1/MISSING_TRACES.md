# CAD — tracciati mancanti e consenso storico (T-CAD-05)

La policy è quella già qualificata di component.json/snapshot, riusata dal
componente progressives.1: nessun nuovo driver o valore di default.

Ogni trace dichiarato ha esattamente Present(record_count) o Absent. Present(0)
è un file completo ricevuto senza righe e conserva SHA256 del contenuto vuoto;
Absent non ha hashinput e non prova che i record precedenti siano scomparsi.
Un percorso fornito ma illeggibile è un errore tecnico, non una scelta Absent.

Il client può completare la validazione ordinaria dei file presenti. Prima di
revisionare lo storico crea ApprovalDraft dalla medesima evidenza, legge
missing_trace_warning_code e absent_traces e presenta W_CAD_MISSING_TRACE
con la lista effettiva. Il getter non consulta il repository.

ABORT scarta la preparazione oppure cancella la preview: nessun nuovo invio,
consenso durevole o aggiornamento dello storico viene creato. PROCEED crea
MissingTraceProceedConsent con tutti i trace assenti e timestamp esplicito.
Non è consenso implicito: lista vuota/duplicata, trace sconosciuto o presente
sono rifiutati; un sottoinsieme non sblocca tutti gli assenti.

Validated richiede anche l'outcome coerente e completo previsto dal core,
le proiezioni protette e la copertura del consenso. Il consenso da solo non
prova validità clinica né autorizza un commit. Solo la preview revisionata e
l'approvazione esplicita possono pubblicare: report prima del commit, CAS,
cancellazione e sessione immutabile mantengono i contratti esistenti.

Nel sorgente approvato vengono persistiti presenza/hash per ogni trace e
missing_trace_consent con decision=proceed, trace e recorded_at, protetti
dalla stessa prova HMAC. Il JSON pubblico conserva i dati ma omette la prova
privata: deserializzarlo non ripristina capacità di approvazione/protezione.
La riapertura SQLite usa invece il codec protetto e verifica la fonte.

Il diff sui progressivi cerca l'ultima baseline PRESENT. Per T1tutto presente,
T2senza B2 e T3B2restituito uguale: nessun annullamento inT2 e nessuna falsa
riattivazione inT3. Per T2con B2PRESENTvuoto invece la baseline è ricevuta e
completa: i precedenti B2possono essere annullati e riattivati se ricompaiono.

La qualifica05 usa i port core, HMAC, providerfile e SQLite reali su materiale
pubblico sintetico e outcome espliciti di componente. Non è la pipeline CAD
clinica Full né una schermata Slint storica: l'integrazione completa resta
ai gate dedicati/CAD12. Nessuna policy viene rimossa per fingere tale esito.
