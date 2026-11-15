# Parity e fixture sintetiche — P2-09

15 novembre2026. Contratto del futuro harness, non PASS del bridge inesistente.
parity-fixtures.json congela14scenari già versionati:6synthetic,2semantic,
2DSL,2derived e2Full cross-trace. SHA-256 di schema/input/attesi e documenti/
generatori indipendenti. Nessuna nuova norma, dato reale o rigenerazione attesi.
I generatori derivano da tabelle esplicite, non dagli output dell’adapter.

verify_parity.py verifica hash/file regolari e definisce il confronto byte
per byte di errors.jsonl/summary.json per scenario. Directory di artefatti
opzionale; senza di essa non viene eseguita parity nativa. La prova locale
controlla anche oracolo alterato e un byte LF aggiunto, su copie sintetiche
temporanee: è self-check del comparatore, non esecuzione del motore.

Il harness implementativo dovrà chiamare il servizio/producer Rust reale e
il bridge autorizzato sullo stesso schema/input catturati, usando i fixture
adapter esistenti in tests/support e il clock di test iniettato. Started_at,
validator_version e elapsed sono deterministici **prima** della serializzazione,
come nei golden esistenti. Niente normalizzazione di output, trim, sort,
conversione Unicode o rimozione di audit per fare coincidere gli attesi.
Per artefatti così prodotti entrambi i file devono coincidere interamente.

Una CLI ordinaria ha clock/elapsed/build diversi dai fixture storici. Il suo
confronto black-box conserva JSONL byte-identico; summary verifica tutti i
campi normativi/provenienza/input, distinguendo solo validator_version,
started_at ed elapsed, dichiarati nella receipt. Stessa versione build fra CLI
e bridge; non si dichiara identico l’intero summary ordinario ai golden con
clock fisso. I golden canonici non vengono cambiati o filtrati per questo.

Oltre ai14scenari deterministici, il gate implementativo include i casi già
definiti P2-03..08: cancellazione/interruzione e tecnico, Unknown/Partial,
truncated non esportabile, schema/reference mutati dopo cattura, owner e
run/query stale, filtri/paging che preservano ordine/valori, dettaglio guida/
domini, export tre formati/summary e receipt per-file. Errori trasporto
restano separati dagli esiti del servizio. Le attese di interleaving sono
tabelle indipendenti, non snapshot generati dal frontend in prova.

Paging/detail non contengono l’intero report: il harness osserva i file Rust
privati autorizzati e attraversa intenzionalmente le pagine in un processo di
test, senza introdurre un endpoint bulk o accumulo nascosto nel prodotto.
CSV/XLSX usano gli assert canonici in ui/tests/exports.rs e servizi esistenti;
non si pretende byte-identità di ZIP XLSX con timestamp runtime diversi.
Frontend in sola presentazione, nessuna regola normativa o validatore Python.

Ogni esecuzione qualificante registra commit+delta/lock, fixture SHA, sistema,
toolchain, comando, exit code, artefatti effettivi e tipo di confronto. Il
comparatore da solo non attesta da quale binario provengano i file. Errori,
fixture assenti/suite vuota o versioni incompatibili falliscono; nessun update
automatico degli attesi e nessun falso PASS da output copiati.

Le prove native/parity nuove si eseguono ai task implementativi/gate previsti,
non dopo ogni task documentale. G-MIG-2 resta il gate di design e non prova
parity Rust↔Tauri↔TS, rilascio, sicurezza DOM o durabilità del prodotto.
