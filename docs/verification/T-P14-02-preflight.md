# T-P14-02 — normalizzazione esplicita e lint avanzato

2026-10-25. Checkpoint richiesto dall'autore: completare14-02 e fermarsi con
percentuale. Prerequisiti3-04 verificata,14-01 implementata e gate locale completo
632/74 suite positivo, qualificazione workbook reale positiva. Native14-01
ancora da verificare, registrata separatamente; prosecuzione autorizzata.

Letti ARCH§8/14, DET§142, Piano§4/14, prompt e SCHEMA_AUTHORING§Import/mapping.
Alias già approvati AN/A SIOC→alphanumeric, N→numeric, D/Data→date. Nessun T CAD
implementato o mapping implicito di nuovi flussi: il chiamante fornisce tabella
alias esplicita. Nessuna conversione per nome del campo/importo/compartecipazione.

API additiva di normalizzazione del solo reviewed_physical_type del profilo:
mappe tipizzate esplicite, report delle sostituzioni, preservazione dei token
originali/source_id/note. Blueprint rimane canonical e il costruttore14-01
continua a verificare che il tipo sia quello revisionato. Alias sconosciuti,
duplicati o ridefinizione di un tipo canonical sono errori, non fallback.

Lint: warning per note importate non ancora classificate con revisione esplicita,
nessuna deduzione note→DSL. Una revisione classificata come metadata richiede
motivazione; una nota mappata richiede riferimenti a regole esistenti nello stesso
trace. Binding SHA-256 al testo originale; una revisione stale non vale per testo
cambiato. Nessuna rimozione o generazione di regole automatica. Namespace metadata
owned, non reinterpretazione indiscriminata di regulatory_metadata arbitrari.

Contraddizioni deterministiche: intersezione vuota di più allowed_values sullo
stesso target raw/logical. Error per campo sempre required senza forbidden_if,
Warning se optional/condizionalmente forbidden: non si assume che il campo sia
sempre applicabile. Target diversi non vengono confusi; valori/zeri/spazi esatti.
Le altre incongruenze già controllate dal compiler restano invariate.

Test warning vs error, compiler rifiuta soltanto Error, normalizzazioni/alias
espliciti e assenti, review stale/invalid/rule refs, determinismo, regressioni.
Nessuna nuova dipendenza, baseline, grammatica, engine/application o CLI.
