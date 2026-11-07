# CAD — approvazione delle sessioni trimestrali (T-CAD-03)

La configurazione eseguibile dal coordinatore di approvazione core è il blocco
`persistence_policy.snapshot` di component.json. Non duplicare questo blocco in
un plugin o driver CAD. Le PK e i nomi dei parametri provengono dallo stesso
componente qualificato inCAD02, il cui fingerprint entra nei metadati firmati.

Lo scope è sending_ulss; year/quarter identificano la sessione, senza separare
l'archivio fra anni. I cinque trace A/B1/B2/C/D hanno presenza esplicita.
`approved_immutable`, `require_explicit_approval` e `atomic_idempotent` sono
vincoli verificati dal compiler del profilo e dal coordinatore core esistente.
Fonti: ARCH45, DET169/171, PERSISTENCE_CORE_SPEC3/4 e profilo CAD storico
ricevuto (hash e decisioni incomponent.json). Nessuna nuova regola regionale.

Il chiamante costruisce ApprovalDraft usando il profilo compilato, i parametri
tipizzati e i metadati della stessa validazione. Prima dell'approvazione può
scartare o sostituire draft/preview senza scrivere l'archivio. Il passaggio a
Validated richiede un outcome Full coerente, non troncato e completo, identità
protette e consenso per ogni trace assente. Questa API pubblica verifica il
contratto dell'outcome; il chiamante è responsabile di fornirne uno reale.

ValidatedApproval.preview legge la generazione e controlla le sessioni precedenti.
ApprovalPreview lega il candidato alla generazione osservata. Il chiamante
presenta quella preview, pubblica i report necessari e invoca approve_commit
solo dopo approvazione esplicita. Errori dei report o cancellazione impediscono
la pubblicazione; una preview concorrente obsoleta fallisce con conflittoCAS.
I dati diventano ApprovedImmutable in una transazione dell'adapter core.

Un invio differente nello stesso ULSS/anno/trimestre approvato è rifiutato con
E_APPROVED_SESSION_IMMUTABLE, anche dopo nuovi trimestri, riapertura SQLite o
cambio di versione schema. Il retry esatto recupera la ricevuta originale e non
ripete report o scritture. ULSS diverse possono approvare lo stesso trimestre;
un nuovo anno continua le generazioni del medesimo namespace.

Qualificazione: tests/cad_sessions.rs compila questa dichiarazione con campi
sintetici della medesima aritàPK e usa SQLite reale per approvazione/restart.
Gli outcome sono fixture di componente esplicite: questi test non attestano
l'esecuzione del servizio Full clinico CAD. Nessuna chiave operativa o dato reale.
Il compiler ordinario e compile_historical_audit continuano a rifiutare il
profilo snapshot incompleto; non si rimuovono policy/capability per aggirarli.

Questa consegna qualifica il confine di approvazione CAD03. Il calcolo cumulativo
dei record attivi/annullati/riattivati e il vincolo progressivo-evento appartengono
aCAD04; la conservazione dei trace assenti aCAD05, temporali alle task pertinenti
e integrazione/golden complete aCAD12. Una preview di candidato03 non deve essere
presentata come una preview di riconciliazione completa o conformitàCAD globale.


T-CAD-05 qualifica warning/consenso e conservazione dei file assenti nella
composizione protetta CAD: vedere [MISSING_TRACES.md](MISSING_TRACES.md).
La distinzione Present(0)/Absent non cambia snapshot/periodo, HMAC o PK.
