# LINTRA — registro delle lacune post-M9

10 novembre 2026. Baseline 140/140 DONE e M9 preservata. Questo registro non è
un nuovo piano approvato: priorità suggerite per decisione dell'autore, con
completamento verificabile. I bug corretti AUD-M9-001/002/003 restano nel
[report](LINTRA_RETROSPECTIVE_AUDIT_REPORT.md), senza trasformare le task originali
in incomplete. Riferimenti di codice: commit `af3bccc`.

Le categorie distinguono contratto violato, rischio, test mancante, esclusione
ratificata, decisione, opportunità e informazione non verificata. Le priorità
seguenti indicano l'ordine suggerito per l'evoluzione; P1 per readiness non
significa incidente già avvenuto o bug del profilo qualificato.

## Decisioni operative e protezione dei dati

### GAP-01 — autorizzazione e perimetro d'uso con dati reali

- **Categoria / fonte:** DECISIONE_APERTA; frontiera IMPLEMENTATION_STATUS,
  ADR-PERSIST-002, PERSISTENCE_OPERATIONS e ratifica provisioning.
- **Stato:** qualifica sintetica presente, nessuna autorizzazione generale a
  dati reali deducibile da M9 o dal possesso del bundle.
- **Impatto:** manca una readiness operativa end-to-end per uso sanitario reale.
- **Prerequisiti:** GAP-02..06, ruoli, ambienti ammessi e procedure di esercizio.
- **Decisione:** autore/proprietario stabiliscono uso consentito, responsabilità
  e criteri di accettazione; nessuna valutazione legale inventata dall'agente.
- **Priorità / completamento:** P1 prima dell'uso reale; decisione esplicita
  versionata e checklist operativa eseguita con prove non sensibili.

### GAP-02 — cifratura e confine di protezione a riposo

- **Categoria / fonte:** DECISIONE_APERTA; ADR-PERSIST-002 prescrizione 2,
  PERSISTENCE_OPERATIONS, AUD-M9-008.
- **Stato:** HMAC/prove/ACL/minimizzazione qualificati; HMAC non cifra DB,
  journal, copie o report. Namespace non equivale ad autorizzazione multiutente.
- **Impatto:** pseudonimi/metadati e report richiedono protezione nel deployment.
- **Prerequisiti:** classificazione dei contenuti, supporti, utenti, destinatari
  e modello di minaccia; compatibilità con storage/protocollo esistenti.
- **Decisione:** modello e controllo approvati, custodia e recupero dei materiali
  necessari. Nessun nuovo algoritmo/provider scelto da questo audit.
- **Priorità / completamento:** P1; protezione ratificata e verificata su dati,
  journal/temporanei/copie e piattaforme di distribuzione, inclusi errori/recovery.

### GAP-03 — retention, cancellazione e trattamento delle copie

- **Categoria / fonte:** DECISIONE_APERTA; PERSISTENCE_OPERATIONS §conservazione.
- **Stato:** snapshot immutabili e audit; nessun purge o scadenza automatica,
  nessuna prova di cancellazione sicura derivabile da DELETE/unlink.
- **Impatto:** impossibile attestare una politica operativa di conservazione.
- **Prerequisiti:** responsabilità, inventario di copie/report/backup e vincoli
  applicabili determinati dal proprietario, senza cambiare le norme del motore.
- **Decisione:** durata, eccezioni e procedura compatibile con audit/immutabilità.
- **Priorità / completamento:** P1 prima dei dati reali; policy ratificata con
  casi di cancellazione/conservazione e prova di trattamento delle copie.

### GAP-04 — compromissione, rotazione e continuità della chiave comune

- **Categoria / fonte:** DECISIONE_APERTA sul lifecycle futuro;
  PERSIST-KEY-PROVISIONING-01 e PORTABLE_KEY_BUNDLE.
- **Stato:** file esterno esatto 32 byte, key-confirmation anche in archivio
  vuoto, ACL/ownership/link verificati, zeroizing; chiave comune ratificata,
  nessuna rotazione/migrazione automatica. Il rischio condiviso è già accettato.
- **Impatto:** sostituire il file interrompe la continuità; compromissione può
  riguardare tutti gli archivi. I DB restano indipendenti e non sincronizzati.
- **Prerequisiti:** custodia/copia di riferimento e inventario installazioni,
  modello di compromissione, preservazione della leggibilità degli storici.
- **Decisione:** risposta all'incidente e migrazione/rotazione esplicitamente
  autorizzate; non riaprire implicitamente il provisioning comune già ratificato.
- **Priorità / completamento:** P1; procedura approvata e drill sintetico di
  perdita/recupero/compromissione senza reset, perdita o falsa compatibilità.

### GAP-05 — backup e restore operativi completi

- **Categoria / fonte:** DECISIONE_APERTA; PERSISTENCE_OPERATIONS,
  T-PERSIST-11/12 e AUD-M9-008.
- **Stato:** copia chiusa/ripristino isolato sintetici e trasferimento nativo
  qualificati; niente backup live, schedulazione o comando prodotto completo.
  `history check` non inventaria e verifica ogni snapshot/fatto.
- **Impatto:** copia tecnica non dimostra recuperabilità/freschezza in esercizio.
- **Prerequisiti:** destinazioni, frequenza, accessi, custodia chiave separata,
  inventario atteso e obiettivi di recupero scelti dall'autore.
- **Decisione:** procedura operativa e responsabilità; eventuale API/comando
  di backup richiede scope dedicato, senza riusare archivi utente per test.
- **Priorità / completamento:** P1; restore in destinazione isolata, confronto
  di tutte le generazioni/namespace/fatti attesi, chiave errata/danno rifiutati,
  evidenze di freschezza e recupero firmate dal responsabile operativo.

### GAP-06 — accessi, redazione e custodia dei report

- **Categoria / fonte:** RISCHIO_TECNICO; AUD-M9-004, atomic.rs/columns.rs,
  PERSISTENCE_OPERATIONS §review dei dati protetti e visibili.
- **Stato:** export atomico per file, permessi secondo umask/ACL ereditate;
  report possono contenere identità/actual. Spool e directory private UI sono
  distinti dagli export dell'utente. Nessuna esposizione reale dimostrata.
- **Impatto:** directory condivise possono esporre report sensibili.
- **Prerequisiti:** destinatari, scope dei campi visibili, retention dei report,
  destinazioni e comportamento di sovrascrittura/ACL richiesti.
- **Decisione:** policy di accesso/redazione; non alterare in silenzio gli output
  approvati o introdurre nuove garanzie di sicurezza senza ratifica.
- **Priorità / completamento:** P1 prima dell'uso reale; prove Linux/Windows di
  create/replace/staging/errore/cleanup con contenuti sintetici e ACL attese.

## Contratti di consumo, risorse e trust boundary

### GAP-07 — policy di consumo CSV da spreadsheet

- **Categoria / fonte:** RISCHIO_TECNICO; AUD-M9-005, CsvSink e golden RFC4180.
- **Stato:** contenuto esatto, compresi prefissi formula; XLSX usa stringhe.
  Nessuna esecuzione di formule su Excel/LibreOffice dimostrata dall'audit.
- **Impatto:** importazione in spreadsheet potrebbe interpretare contenuti
  esterni; quoting CSV da solo non impone il tipo testuale al consumatore.
- **Prerequisiti:** consumatori target e scelta fra scambio byte-esatto e
  rappresentazione per visualizzazione; preservare JSONL canonico.
- **Decisione:** contratto dell'export destinato a spreadsheet o procedura
  d'importazione supportata; nessuna sanitizzazione tacita nel formato attuale.
- **Priorità / completamento:** P2, anticipare se CSV è la via d'uso reale;
  casi =/+/-/@ e separatori verificati nel consumatore scelto, senza perdita dei
  valori originali, e test del contratto approvato.

### GAP-08 — budget di input, reference e cancellazione in lettura

- **Categoria / fonte:** RISCHIO_TECNICO; AUD-M9-006, RecordReader,
  trace_validator e filesystem reference.
- **Stato:** buffer proporzionale alla singola riga, fs::read per dataset,
  cancellation fra record; max_errors limita diagnostiche, non bytes della riga.
- **Impatto:** memoria/latency sotto input sproporzionati o letture lente.
- **Prerequisiti:** volumi massimi ammessi, fonti regolari/stream supportati,
  comportamento su record eccedente e integrità dell'input audit.
- **Decisione:** budget operativo e risposta tecnica/diagnostica, se necessario;
  non inventare limiti come soglie DEC-14 o troncare silenziosamente i dati.
- **Priorità / completamento:** P2; misure bounded e test EOF/CRLF/hash,
  cancellation, errori I/O e preservazione dei report precedenti.

### GAP-09 — tree reference controllata durante il caricamento

- **Categoria / fonte:** LIMITAZIONE_RATIFICATA; AUD-M9-007,
  CNF-REFERENCE-FORMAT-01 e intestazione filesystem.rs.
- **Stato:** confinement in configurazione stabile; nessuna promessa di open
  atomico per handle contro mutazioni ostili. Dopo preparazione snapshot
  immutabile e digest riferito ai byte effettivamente caricati.
- **Impatto:** un deployment con tree modificabile da soggetti non fidati
  supera il modello qualificato.
- **Prerequisiti:** ownership e accessi del bundle, aggiornamenti controllati.
- **Decisione:** confermare il deployment ammesso o autorizzare hardening mirato.
- **Priorità / completamento:** P2; requisito operativo verificato oppure nuova
  risoluzione per handle qualificata con sostituzioni/symlink native e regressioni.

## Capacità library, interfacce e profili normativi

### GAP-10 — integrazione Full e storico nella GUI

- **Categoria / fonte:** LIMITAZIONE_RATIFICATA; ui/src/analysis.rs richiesta
  Trace/LocalOnly, PORTABLE_KEY_BUNDLE, matrice del report §4.
- **Stato:** GUI corrente locale; nessun workflow Full/cross-run/approval.
  Convenzione di percorso chiave disponibile a livello API, senza schermata.
- **Impatto:** l'utilizzatore GUI non raggiunge tutte le capacità della library.
- **Prerequisiti:** selezione multi-trace/parametri, presentazione Complete/
  Partial/Unknown, key/archive preflight, preview/report/review/cancellation/CAS.
- **Decisione:** scope e condizioni operative dell'integrazione, preservando
  Signature; nessuna implicita migrazione React/Tauri in questo workstream.
- **Priorità / completamento:** P2; percorsi GUI→servizio reale con stessi golden
  e fallimenti CLI/library, senza norme nel frontend o capability sovradichiarate.

### GAP-11 — workflow CAD snapshot nelle interfacce

- **Categoria / fonte:** LIMITAZIONE_RATIFICATA; frontiera M9,
  schemas/operational/CAD/2024-06.runtime-candidate.1/README.md,
  validate_snapshot e HISTORICAL_CLI.
- **Stato:** library CAD83 C1/L1/PASS1/2 e approval atomica qualificata;
  CLI storica audit-only rifiuta il profilo snapshot; GUI non storica.
- **Impatto:** CAD operativo completo non disponibile al solo utente finale.
- **Prerequisiti:** GAP-01..06/10; dati origine C1 con fonte/hash/responsabilità,
  consensi esatti Present/Absent e review legata al candidato.
- **Decisione:** esposizione e responsabilità operative; non richiedere di
  nuovo C1/L1, già ratificate, né trasformare PROCEED in copertura.
- **Priorità / completamento:** P2; golden multi-trimestre e namespace attraverso
  l'interfaccia, con report, rifiuti, annullamenti e replay reali su entrambi OS.

### GAP-12 — domini esterni e profilo SIOC completo

- **Categoria / fonte:** LIMITAZIONE_RATIFICATA e dipendenze esterne;
  DEC-SIOC-MINIMAL-01, CNF-SIOC-REFERENTIAL-G0-01, ledger .1/.2,
  profile-inventory.json, dossier CAD03/11 per le esclusioni CAD.
- **Stato:** SIOC .2 eredita 83 constraint/70 regole/3 gruppi deferred da .1;
  quattro relazioni esplicite e golden operativo separato. Alcune primitive
  esistono nel runtime attuale ma non sono integrate/qualificate nel profilo
  completo. CAD esclude lookup ICPC/ISTAT non forniti e non estende gli elenchi
  ricevuti ICD9/prestazioni a cataloghi universali.
- **Impatto:** nessuna conformità regionale/clinica universale deducibile dal
  profilo minimale, dal nome Full o da CAD83.
- **Prerequisiti:** ricognizione voce-per-voce, fonti ufficiali effettivamente
  fornite e versionate, digest/provenienza, contratti mancanti e oracle normativi.
- **Decisione:** approvare solo le integrazioni con fonti e regole determinate;
  l'esclusione preesistente non diventa automaticamente nuovo obbligo del codice.
- **Priorità / completamento:** P2 per completamento funzionale; tutti i rinvii
  riconciliati in nuovo profilo versionato, dataset verificati, golden positivi/
  negativi/Unknown e rifiuto invariato delle capability non supportate.

### GAP-13 — enforcement SIOC inter-invio

- **Categoria / fonte:** DECISIONE_APERTA; DEC-SIOC-HIST-01, DET §170,
  tests/sioc_persistence.rs.
- **Stato:** confronto audit deterministico, namespace e identità persistenti;
  nessun nuovo divieto su PK ripetuta cross-run. Duplicati intra-run già errori.
- **Impatto:** audit non equivale ad accettazione/rifiuto normativo dell'invio.
- **Prerequisiti:** norma e casi di invio/rettifica/riuso, perimetro storico.
- **Decisione:** autore competente ratifica il comportamento; non risolta qui.
- **Priorità / completamento:** P2 solo se richiesto dal contesto d'uso; decisione
  versionata e test di riapparizione lecita/illecita e contesto indisponibile.

## Release, qualifica e miglioramenti

### GAP-14 — workload rappresentativi, soglie e profiling

- **Categoria / fonte:** DECISIONE_APERTA per soglie; MIGLIORAMENTO_OPZIONALE
  per profiling, AUD-M9-009, DEC-14, PERFORMANCE, DEC-17 finale.
- **Stato:** misure sintetiche disponibili, soglie deliberatamente non definite;
  prototipo parallelo non adottato, produzione sequenziale.
- **Impatto:** non si possono promettere SLA né scegliere ottimizzazioni su
  benchmark estremi; possibili costi owned/history da attribuire prima di agire.
- **Prerequisiti:** aggregati non personali per trace/invio/storia, errori,
  reference, hardware/filesystem; distinguere heap vivo, RSS, I/O e UI.
- **Decisione:** volumi e soglie, eventuale proposta di ottimizzazione separata.
- **Priorità / completamento:** P3, elevabile per esigenze operative dimostrate;
  profiling riproducibile end-to-end/stage, bilanciato e sbilanciato, confronto
  A/B di eventuali miglioramenti e soglie solo dopo ratifica. Nessun parallelismo
  automatico o riapertura di DEC-17.

### GAP-15 — release pubblicata, artefatto e freeze normativo

- **Categoria / fonte:** LIMITAZIONE_RATIFICATA / decisione di distribuzione;
  frontiera, Cargo 0.1.1-rc.3, dossier T-P18-03 release rc.2 e CAD candidato.
- **Stato:** M9/QG-8 non sono release 1.0, pubblicazione rc.3 o FROZEN CAD.
  Le qualifiche del codice non sostituiscono build/provenance dell'artefatto.
- **Impatto:** il pacchetto rc.2 non include automaticamente le correzioni o
  tutte le capacità del checkout rc.3; un bundle privato con chiave è distinto.
- **Prerequisiti:** scope scelto, SHA qualificato, condizioni operative e norma,
  packaging/runbook/manifest/checksum e gestione upgrade compatibile.
- **Decisione:** quale artefatto distribuire e quale gate umano di freeze serve.
- **Priorità / completamento:** P2 quando si decide di distribuire; release
  autorizzata su SHA esatto, artefatti nativi verificati, distinzione pubblicazione/
  freeze/readiness mantenuta. Nessun tag o release creato dall'audit.

### GAP-16 — verifica fisica di durabilità oltre il crash di processo

- **Categoria / fonte:** NON_VERIFICATO; limiti espliciti ADR-PERSIST-002 e
  dossier PERSIST05/11. Non è un requisito di task rimasto fallito.
- **Stato:** DELETE/EXTRA riletti, crash/recovery, lost response e trasferimento
  fra OS qualificati; niente laboratorio di interruzione alimentazione/disco.
- **Impatto:** nessuna garanzia generalizzata su firmware/device/filesystem non
  qualificati o anti-rollback esterno dell'intero archivio.
- **Prerequisiti:** modello operativo e hardware/supporti realmente previsti.
- **Decisione:** necessità di ulteriori prove e livello di garanzia richiesto.
- **Priorità / completamento:** P3, elevabile per esigenza dimostrata; protocollo
  di laboratorio approvato con criteri, recovery e inventario atteso, senza dati reali.

### GAP-17 — piano originale dell'audit separato

- **Categoria / fonte:** NON_VERIFICATO; AUD-M9-010 e mandato dell'autore.
- **Stato:** `lintra-retrospective-engineering-audit.md` non reperito; mandato
  incollato completo disponibile e applicato.
- **Impatto:** solo confronto di provenienza con quel documento non eseguibile.
- **Prerequisiti / decisione:** fonte originale se desiderato dall'autore;
  nessun blocco di correttezza o nuova richiesta ordinaria di conferma.
- **Priorità / completamento:** P3; confrontare la fonte reperita e registrare
  esclusivamente eventuali requisiti aggiuntivi materialmente diversi.

## Chiusura del registro

Nessuna voce è promossa automaticamente a READY. L'ordine proposto è:
correzioni qualificate → decisioni dati reali → interfacce/profili richiesti →
ottimizzazioni giustificate. Le nuove implementazioni richiedono il relativo
perimetro autorizzato; le decisioni già ratificate rimangono valide.
