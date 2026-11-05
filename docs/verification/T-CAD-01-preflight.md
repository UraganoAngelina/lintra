# T-CAD-01 — preflight e registration dichiarativa

5 novembre 2026, task indipendente dalla prova release18-03 in WAITING_GATE.
PERSIST01..12 eP13 DONE con evidenze native; PERSIST12 chiusa con638feec /
CI41857167059 e artifact verificati. G0 approvato con rinvii19ott407f89e;
nessuna promozione implicita CAD aFROZEN o piena eseguibilità.

Letti prompt01/02, piano§4/14/20, ARCH45, DET166–174 ePERSISTENCE_CORE_SPEC;
candidato/manifest/CAD_RUNTIME_PROFILE, audit83note e fixture plan nel kit.
Il candidato conserva DSL1.0.0 e richiede estensioni compiler/runtime. Il contratto
operativo raw1 è approvato separatamente72f9f5e; non deserializzare il candidato
legacy come profilo operativo valido né eliminare requisiti non implementati.

## Perimetro iniziale

Registration generica per chiave flow/version tramite SchemaRegistry e compiled
persistence plan. Nuova resolve_persistence_profile restituisce soltanto
CompiledPersistenceProfile, mai CompiledFlowSchema o prova di esecuzione.
Segue manifest/raw dal repository già selezionato; nessuna nuova forma raw,
repository CAD-specific, backend, storage o ramo per nome del flusso.
I membri di compatibility group vengono caricati per versioni esatte, verificati
contro chiave richiesta e fingerprint, con encoding runtime esplicito.
La sorgente selezionata si carica una sola volta. Lint prima di letture secondarie;
errori restano tecnici. Policy assente rimane disattivata. History/temporal non
supportati continuano a fallire dichiaratamente, non vengono rimossi.

Test nuovi tests/persistence_registry.rs: con etichetta CAD synthetic snapshot on/off,
SIOC reale audit-only, nessuna conversione a schema eseguibile, caricamenti singoli,
versioni ehash, repository che sostituisce flow/version, gruppo invalido/mancante,
history rifiutata. La fixture con etichetta CAD è sintetica per registration e non
rappresenta gli 83 campi o la conformità CAD. Non viene distribuita come schemaCAD.

## Residuo e stato

IN_PROGRESS: accesso registry in sviluppo/test. Il profilo CAD operativo completo
richiede abbassamento dichiarativo tracciato del candidato e capability future;
nessun DONE01 o gate CAD desunto dalla sola API. PK/ULSS/sessioni02 e le regole
storiche/temporali delle task successive rimangono distinte. Nessuna nuova
ambiguità normativa risolta implicitamente. Release rc.2 resta sullo SHA497352c
pubblicato e qualificato separatamente: queste modifiche locali non vi rientrano.


## Precisazione del confine01 rispetto a02 e successive

01 consegna l’accesso di registration nel registry tramite repository/manifest,
versione esatta e compiled plan;02 definisce il profilo CAD con PK/namespace
normativi. La disponibilità di01 non è né certificazione del candidato83campi
né esecuzione di history/temporale: i rifiuti espliciti sono parte del gate01.
Non viene pubblicato un profilo CAD ridotto o una nuova versione normativa.
La nuova prova filesystem usa manifest reale e schema sintetico temporaneo;
verifica il digest prima del compiled plan e rifiuta anche la sola variazione
di whitespace rispetto al manifest. Nessun fallback in caso di mismatch.

Primo tentativo del test manifest: compilazione fallita perché il tipo digest
sha2.11 non implementa LowerHex. Corretto il solo encoder del digest nella
fixture (due cifre esadecimali minuscole per byte), poi avviata la matrice locale
finale. Il tentativo fallito non viene registrato come successo del test.


## Gate locali conclusi — codice di registration

Cinque comandi exit0;894esiti workspace/113riepiloghi,0fail/6ignored,
35test mirati allfeatures (registry7/profile8/schema registry6/SIOC14),
fmt/Clippyworkspace all-targets allfeatures eboundary positivi.405impronte
immutate nel ledger T-CAD-01-local-gates.json. Correzione lock/Python18-03
successiva riguarda tooling/versione, non il metodo registry misurato e verificato.
Nessun testWindows dichiarato locale. Restano pubblicazione del codiceCAD01 e
CI propria: nonDONE01. Nessuna parte diCAD02 avviata senza quel prerequisito.
