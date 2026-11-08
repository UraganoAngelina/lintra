# CNF-DIAGNOSTIC-ORDER-01 — ratifica A condizionata, 8 novembre 2026

Fonte: risposta esplicita dell'autore nella sessione dell'8 novembre 2026,
«Approvo CNF-DIAGNOSTIC-ORDER-01, alternativa A, con le precisazioni sopra riportate».
Il messaggio integrale dell'autore prevale su questa registrazione operativa.

## Contratto e autorizzazione

Il criterio esterno Full è la passata: locali/PK e altri controlli contrattualmente
PASS1 di tutti i trace prima delle cross/dipendenti PASS2. Preservare schema,
regole, streaming, ContextStore effimero separato dalla persistenza, output,
max_errors, troncamento, Cancelled/Partial, provenienza, sink e memoria controllata.
Nessun ordinamento globale per trace, buffering/spool, rilettura o terza passata.
Runtime sequenziale/DEC-17 D invariati; nessuna adozione implicita di parallelismo.

Prima di modificare DET79/115 verificare effettivamente trace, linee, fieldpipeline,
stage, regolelocali, PK, cross/eventi e diagnostiche prive di campo o di run.
Non dichiarare trace→line→field→stage→rule applicata globalmente dentro una passata
se il codice non la realizza. Documentare differenze con esempio riproducibile;
la ratifica permette la formulazione documentale minima fedele al determinismo
esistente. Nessuna coordinata inventata. Nuova decisione soltanto per conflitto
materiale non coperto, mai modifica automatica del prodotto.

## Evidenze obbligatorie

Fixture sintetica con trace S0/T1 in ordine schema: localeT1 prima di crossS0,
con asserzione indipendente dell'ordine. JSONL ripetuto byteidentico senza
normalizzazione; budget con prefisso corretto e summary/troncamento reali,
separando PASS1/PASS2 quando necessario. Preservare e verificare checkpoint di
cancellazione effettivi senza imporre diagnostiche di regole non eseguite.
Regressioni: golden/JSONL/summary/codici/provenienza/PK/cross/CAD/SIOC/sinkfailure/
errori tecnici. Soli dati sintetici, Quality Gate pertinenti e CI secondoprotocollo.

## Sequenza di chiusura

Registrare ratifica → auditcodice → testi documentali → test mirati → gate →
evidenze → RESOLVED A. Ratifica umana non equivale alla chiusura tecnica.
DET79 deve esplicitare le precedenze verificate; DET115 mantiene sequenzialità,
due passate/DEC17 D e determinismo. ARCH/MASTER soltanto riferimenti direttamente
necessari, non chiusura generaleP18-04. Preservare storia e tracciabilità.

Solo documenti e test di regressione: vietate modifiche funzionali a engine,
compiler/DSL, join, PK, proiezioni/store, sink/JSONL/summary, budget/cancellazione,
repository/SQLite/snapshot/HMAC/API/CLI/UI/formati/capability per questa ratifica.
CAD08restaDONE; CAD12segue dipendenze effettive; nessunFROZEN/M9 implicito.
Single-agent, no delega/background, commit/push/CI autorizzati. Attività CAD
indipendenti non fermate dalla sola risoluzione documentale.
