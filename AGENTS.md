# Istruzioni operative Lintra

## GLOBAL AGENT EXECUTION POLICY

Revisione GOV-2026-10-26-01, approvata esplicitamente dall'autore il26 ottobre2026.
Questa è la governance operativa corrente di tutte le task Lintra. Supera gli
stop generici e i checkpoint di sola sessione precedenti, incluso quello dopo
T-P14-05, senza modificare ratifiche normative, contratti runtime o Quality Gate.

1. **Un solo agente principale.** Vietati categoricamente Superpowers, subagenti,
   delega e strumenti multi-agent. Eseguire task atomiche in sequenza; nessuna
   orchestrazione parallela e nessun processo locale in background.
2. **Commit, push e CI autorizzati permanentemente** quando necessari sul repository
   UraganoAngelina/lintra e branch concordato work/p1-foundations. Nessuna conferma
   ordinaria; rispettare branch protection, sicurezza, piattaforma e Quality Gate.
   Niente force-push, perdita di lavoro altrui, segreti o dati paziente versionati.
3. **Prosecuzione automatica.** Dopo i gate effettivi aggiornare stato e handoff,
   scegliere la prossima task autorizzata e pronta e iniziarla nello stesso flusso.
   Fine task e handoff non terminano automaticamente la sessione.
4. **Arresto solo per criticità reali.** Richiedere approvazione per decisioni
   normative, architetturali, contrattuali o di sicurezza non già autorizzate.
   Diagnosticare e correggere autonomamente bug ordinari, errori di test/build,
   problemi implementativi e di rete. Se non esistono task autorizzate e pronte,
   registrare il prerequisito mancante senza inventare progresso o consenso.

Workflow vincolante:
TASK READY → PREFLIGHT → IMPLEMENTAZIONE SINGLE-AGENT → TEST LOCALI →
COMMIT/PUSH E CI QUANDO NECESSARI → QUALITY GATE → AGGIORNAMENTO STATUS →
HANDOFF → PROSSIMA TASK READY → RIPETI.

### Gate e condizioni di prontezza

- READY richiede dipendenze e decisioni applicabili verificate con evidenze.
  DONE_UNVERIFIED, CI pendente/non leggibile o gate fallito non soddisfano un
  prerequisito che richiede quel gate. Nessun DONE o milestone senza criteri reali.
- Durante l'attesa remota si può sospendere la task con stato fedele e lavorare
  sequenzialmente su una task indipendente autorizzata; mai aggirare dipendenze.
- Conservare determinismo, streaming, audit, confini di layer, gestione byte/raw/
  logical/MissingValuePolicy e separazione errori normativi/tecnici. Il motore sa
  COME validare; lo schema sa COSA è valido. Nessuna norma hardcoded o implicita.
- Restano validi i gate umani reali (norme/ADR/CNF/contratti/sicurezza irrisolti).
  Gli stop storici di sola sessione sono superati dalla presente autorizzazione.
- CI sobria: pianificare non più di tre interrogazioni ordinarie per run, con
  almeno60s fra letture. Log necessari, retry motivati e letture finali mancanti
  sono già autorizzati: il vecchio limite non è una barriera di approvazione per
  diagnosi o chiusura. Evitare busy polling e pipeline duplicate. Non trasformare
  timeout in successi e non indebolire controlli TLS o confini di sicurezza.
- Spiegare dettagliatamente codice nuovo/modificato (struct, funzioni, condizioni,
  motivi di correttezza) con commenti/documentazione e report consultabile;
  comunicare per ogni task un breve recap dei file e degli esiti effettivi.

### Fonti operative e memoria storica

- Leggere MASTER_AGENT_PROMPT.md, docs/MASTER_IMPLEMENTATION_PLAN.md §14,
  TASK_DEPENDENCY_MAP.md, ATOMIC_TASKS_INDEX.md e TASK_PROMPTS/<ID>.md del
  repository. Template correnti in docs/governance/templates/.
- Il kit adiacente LINTRA_STARTER_KIT_FINAL_20261016, docs/bootstrap, i vecchi
  piani Superpowers e i report/ratifiche/handoff precedenti sono fonti storiche
  per contratti/evidenze. Non costituiscono un'autorizzazione a delegare o uno
  stop operativo corrente. Non riscriverli retroattivamente.
- Le ratifiche contrattuali presenti nella cronologia seguente continuano a
  valere nel merito; soltanto le clausole operative incompatibili sono superate.
  Specifiche normative, ARCH e DET non sono modificate da questa policy.
- Nessun override annidato può ripristinare uno stop generico o la delega; eventuali
  istruzioni successive dell'autore e le regole della piattaforma mantengono la
  propria priorità. Verificare override reali durante ogni ripresa.

## Registro storico e ratifiche precedenti

Mandato dell'autore aggiornato il 2026-10-19:

- Non usare le skill Superpowers o i relativi workflow.
- Procedere direttamente, una task atomica alla volta, verificandone il risultato.
- Avanzare senza conferme ordinarie quando prerequisiti ed esito sono chiari.
- Fermare l'avanzamento interessato in presenza di ambiguità o decisioni mancanti.
- Per questa modalità lavorare senza subagenti; non avviare processi in background.
- Rispettare ARCH, DET, Master Plan e prompt atomici dello starter kit;
  il nuovo mandato sostituisce il precedente flusso di orchestrazione.
- Registrare stato, evidenze reali e handoff in `IMPLEMENTATION_STATUS.md`.
- I file di pianificazione e orchestrazione precedenti sono materiale storico.
- Mandato successivo dell'autore: dopo una task verificata, iniziare subito
  la successiva se prerequisiti ed esito sono chiari. Questo mandato prevale
  sullo stop tra task contenuto nei prompt atomici; restano i blocchi reali.
- Alla fine di ogni task comunicare un breve recap dei file creati/modificati.

Mandato specifico successivo — CNF-AUDIT-01 approvata:
- Implementare T-P5-05 con le precisazioni dell’autore su EOF, sorgente
  autorevole degli audit, overflow, cancellazione e test.
- Fermarsi al termine di T-P5-05 e attendere autorizzazione prima della
  task successiva; questo mandato supera la progressione autonoma generale.

Mandato successivo dell'autore — 2026-10-20:
- Autorizzata la prossima task, T-P5-06, dopo la consegna di T-P5-05.
- Regola vincolante: «OGNI QUAL VOLTA SCRIVI DEL CODICE DEVI ESPRIMERE
  DETTAGLIATAMENTE COSA FA E COME MAI FUNZIONA, CON DEL CODICE INTENDO
  STRUCT FUNZIONI IMPLEMENTAZIONI E CONDIZIONI».
- Accompagnare quindi il codice nuovo o modificato con spiegazioni del
  comportamento e delle ragioni di correttezza: documentazione/commenti
  pertinenti, spiegazione tecnica consultabile e recap finale dei file.

Mandato più recente — ripresa del workflow autonomo, 2026-10-20:
- Dopo T-P5-07, procedere da T-P6-02 e iniziare autonomamente le task successive
  quando la precedente è verificata e prerequisiti/contratti non presentano blocchi.
- Questo mandato supera gli stop ordinari dei prompt e lo stop storico dopo
  T-P5-05. Restano vincolanti i gate e le decisioni realmente mancanti.
- Restano esclusi Superpowers, subagenti e processi locali in background;
  mantenere spiegazioni dettagliate del codice e recap dei file per ogni task.

Decisione successiva — CNF-ALIGNMENT-01, scelta B:
- L'autore ha scelto «B: definiamo prima il contratto fisico».
- Non attuare la proposta A di consegna parziale/prosecuzione con residuo.
- Definire e sottoporre il contratto fisico completo prima di implementare
  T-P6-03; la scelta B non approva automaticamente i dettagli proposti.

Decisione finale dell'autore — supera la precedente scelta B:
- T-P6-03 consegna solo estrazione logical zero-copy dal lato dichiarato;
  preserve_width/exact invariati, all-fill vuoto solo sotto stripping.
- Autorizzata revisione del prompt T-P6-03 e di DET §44. Nessun validator
  fisico, E_ALIGNMENT o nuovo parametro opposite_edge_fill nel formato 1.
- Nessun residuo fisico aperto: CNF-ALIGNMENT-01 archiviata come estensione
  futura non implementata. Dopo verifica procedere a T-P6-04.
- Lo spazio iniziale in testo libero resta lecito; altri stage verificano
  valori, larghezza, chiavi e date senza garanzia assoluta sugli shift di colonne.

Ratifica CNF-RECORD-01 — T-P6-08 LocalOnly:
- Proposta A approvata con precisazioni dell'autore: validate_local restituisce
  LocalRecordValidationArtifacts { identity, valid }; valid riguarda solo
  controlli locali supportati, mai conformità globale.
- Errori tecnici RecordValidationError nell'engine, senza import application.
- ProjectionPlan invariato e non consumato in LocalOnly: non è un errore se
  non vuoto. Regole locali/condizioni richieste ma non supportate vanno rifiutate.
- Nessun placeholder, reference dataset o proiezione simulata; P11/P13 e
  residuo T-P2-03/M1 restano aperti; fonti architetturali invariate.
- STOP obbligatorio dopo T-P6-08 e handoff: non implementare P7 o altre task
  senza nuova autorizzazione. Il riordino P7 è approvato solo come piano.

Mandato 2026-10-21 — ripresa autonoma dopo T-P6-08:
- Autorizzata la continuazione da T-P7-01 e l'avanzamento automatico dopo ogni
  task verificata, quando prerequisiti/output non presentano ambiguità.
- Superato lo stop storico dopo T-P6-08. Restano gate e blocchi reali,
  esclusione Superpowers/subagenti e spiegazioni dettagliate del codice.

Ratifica CNF-RUN-SCOPE-01 — 2026-10-21:
- Proposta A approvata: ValidationMode neutro nel dominio, re-export compatibili;
  validation_mode esplicita in RunMetadata/ValidationSummary/summary JSON.
- Fonte autorevole metadata dalla request verificata, nessun default indipendente.
- LocalOnly serializzato local_only, Full full ma resta non eseguibile fino P13.
- Autorizzati aggiornamenti dei soli golden tecnici; RunStatus/Cancelled/truncated,
  input audit e baseline normative invariati. Procedere con T-P7-04 e verifiche.

Ratifica CNF-SERVICE-DI-01 — 2026-10-21:
- T-P7-05 autorizzata solo come API LocalOnly: factory esplicita schema_root/encoding,
  repository/registry/SystemClock reali e DI new/with_clock preservata.
- Nessun fallback, provider/repository fittizio o parametro ignorato; Full rifiutato.
- Handle compiled pubblici di sola lettura ratificati, non-forgiabilità obbligatoria.
- P11, P13, PERSIST e T-P2-03/M1 restano aperte secondo il ledger di task.
- La ratifica non sostituisce la CI precedente: verificare i gate Linux/Windows.
- STOP dopo T-P7-05 e handoff; attendere autorizzazione prima di T-P7-06, CLI
  o qualunque altra task. Questo mandato supera la progressione autonoma generale.

Mandato successivo — ripresa dopo T-P7-05, 2026-10-21:
- L’autore revoca lo stop dopo T-P7-05: continuare da T-P7-06 e poi
  automaticamente una task alla volta con dipendenze/gate soddisfatti.
- Fermarsi solo sulle attività con ambiguità o blocchi reali; restano
  esclusi Superpowers/subagenti e obbligatori spiegazioni e recap dei file.

DEC-12 — prima CLI, scelta A ratificata dall’autore:
- Riga vuota = record zero byte; terminatore finale non inventa una riga extra.
- BOM Reject, errore tecnico; nessun --bom strip nella prima CLI.
- Nessuna modifica della request/pipeline per simulare stripping non selezionabile.

Ratifica dell'autore — 2026-10-22 — Decimal importi / formato 2:
- Approvata docs/proposals/decimal-amount-contract.md: procedere T-P9-03,
  inclusi aggiornamenti authoring/meta-schema/lowering/DET §137/mappa/prompt.
- Formato 2 aggiunge Decimal; formato 1 invariato, nessuna migrazione automatica.
- Solo importi monetari diretti: dot al terzultimo byte, due cifre frazionarie,
  unsigned, non negativi con zero ammesso, niente separatori delle migliaia.
- Codici/range di compartecipazione e qualunque campo esistente restano invariati.
  Niente classificazione per nome o ID hardcoded, niente baseline riscritte.
- Exact/preserve width per il nuovo Decimal; CNF-ALIGNMENT-01 invariata per v1.
- Nessun floating point; gate locali/native CI e spiegazioni dettagliate richiesti.

Ribadito dall'autore — 2026-10-22:
- Iniziare automaticamente la task successiva dopo esito e gate non ambigui.
- Un handoff o la fine di una task non sono motivi per chiudere il turno.
- Nessuna conferma ordinaria ulteriore; fermarsi per criticità contrattuali reali
  o nuove istruzioni esplicite dell'autore, preservando tutti i Quality Gate.

Ratifica successiva — CNF-REFERENCE-FORMAT-01 aggiornata, 2026-10-23:
- Membership v1 approvata con Verified/Degraded/Unavailable ed evidenze distinte.
- Hash mismatch non equivale a JSON malformato: file conforme e identità affidabile
  resta consultabile soltanto come degradato con entrambi i digest conservati.
- ContinueWithAvailableEvidence esplicita; Strict compatibile, sicurezza filesystem
  mai rilassata. Nessun recupero permissivo inventato per JSON malformato.
- T-P11-03 consegna provider/preparazione/stati/evidenze e test, non summary/DSL/UI
  o Union completi: mantenere ledger futuro, proporre API minime e additive.
- Baseline CAD/SIOC, flow grammar e fonti architetturali non modificabili qui.
- STOP obbligatorio dopo T-P11-03, gate/handoff; non iniziare task successive
  senza autorizzazione. Questo mandato supera la progressione autonoma generale.

Mandato successivo — ripresa automatizzata, 2026-10-23:
- L'autore autorizza la ripresa delle task e l'avvio automatico della successiva
  se non emergono ambiguità/conflitti; superato lo stop dopo T-P11-03.
- Chiamate GitHub automatiche, massimo tre chiamate Actions per gate con almeno
  45 secondi di intervallo. Restano gate, spiegazioni e divieti precedenti.

Mandato successivo — checkpoint di avanzamento, 2026-10-23:
- L'autore chiede: «appena finisci la prossima task fermati un attimo e fammi
  un resoconto completo [...] voglio capire a che percentuale di progresso siamo».
- Workflow fermo al checkpoint T-P12-03 locale verificata/gate nativo pendente;
  nessuna nuova task dopo il resoconto prima della ripresa dell'autore.
- Consultare IMPLEMENTATION_STATUS e PROJECT_PROGRESS_2026-10-23 nel dossier:
  nessun DONE fittizio, push bloccato dal timeout del controllo automatico.
- Restano valide autonomia commit/GitHub sul repository/branch concordati e
  le regole del workflow alla ripresa; non richiedere nuovamente conferme ordinarie.

Mandato successivo — ripresa dopo l'allineamento, 2026-10-23:
- L'autore ordina: «prosegui con il workflow concordato sulle attività di
  sviluppo a pieno ritmo». Revocata la pausa del checkpoint precedente.
- Riprendere da pubblicazione/gate di 12-03 e 11-06 senza duplicare codice;
  dopo evidenza positiva iniziare automaticamente 12-04 e le task successive
  con prerequisiti verificati e senza conflitti sostanziali.
- Commit, push su UraganoAngelina/lintra work/p1-foundations e CI autonomi
  nel workflow già concordato; massimo tre chiamate Actions per gate, ≥45s.
- Restano vietati Superpowers, subagenti e processi locali in background.

Ratifica CNF-SIOC-REFERENTIAL-G0-01 — 2026-10-25:
- Autorizzata una nuova revisione operativa SIOC derivata da structural-minimal.1,
  preservando integralmente .1, FROZEN e documenti normativi sorgente.
- Quattro regole REL-* esplicite approvate con provenance per relazione e
  condizioni verificate; nessun controllo implicito generato dai join nell'engine.
- revoking_act mantiene self-join 4.30→4.22 zero_or_many e guard; record assente
  verificato Violation E_RELATED_RECORD_MISSING, universo indisponibile Unknown.
- Anagrafica→ricovero opzionale NotApplicable invariata. Niente first(), nuove
  regole cliniche o rimozione delle capability mancanti dal profilo completo.
- Per 13-08 autorizzato sottoinsieme operativo esplicito/tracciato per golden
  con almeno una relazione cross-trace reale, dati sintetici e ledger esclusioni.
- Memoria indipendente; 13-08/M6 solo dopo golden reale e tutti i gate effettivi.
- STOP al termine del perimetro T-P13-08: attendere nuova autorizzazione prima
  di ulteriori task. Questo stop specifico prevale sul workflow automatico generale.

Mandato successivo — ripresa del workflow dopo T-P13-08, 2026-10-25:
- L’autore ordina «procedi con il workflow concordato a sviluppare le task».
- Riprendere dalla verifica del gate13-08 già avviato; se positivo, chiudere
  la task e avanzare automaticamente con prerequisiti/contratti verificati.
- Superato lo stop specifico dopo13-08. Restano gate, spiegazioni dettagliate,
  esclusione Superpowers/subagenti/background e limite tre chiamate Actions
  con intervalli≥45s nella ripresa autorizzata. Nessuna baseline riscritta.

Mandato aggiornato — ripresa autonoma push/CI,2026-10-25:
- L’autore rinnova autonomia nei push e negli avvii CI e autorizza l’avvio
  automatico della task successiva con output/prerequisiti senza criticità.
- Limite aggiornato: tre chiamate CI, distanziate di almeno un minuto.
- Riprendere dalla pubblicazione14-04 senza duplicare codice; poi14-01 usando
  il solo workbook SIOC fornito. Nessun altro flusso o funzionalità fuori piano.

Precisazione dell'autore — prosecuzione indipendente,2026-10-25:
- Procedere con la task successiva se lo sviluppo precedente e i prerequisiti
  non presentano problemi o criticità normative. Il solo gate remoto non letto
  non blocca task indipendenti con prerequisiti soddisfatti.
- T-P14-01 avviata: richiede T-P3-01/G0, non T-P14-04. Tenere14-04 aperta
  sulla verifica MSVC senza dichiararla DONE. Nessuna quarta chiamata al vecchio
  gate; il gate della nuova consegna reale include le regressioni precedenti.

Checkpoint successivo dell’autore —2026-10-25:
- «Completa la prossima task poi fermati e dimmi a che percentuale»:
  checkpoint dichiarato su T-P14-02 dopo verifica della consegna14-01.
- Terminata14-02 e relativi gate, fermarsi con resoconto/progresso verificato.
  Nessuna14-05/P15 o ulteriore task prima di nuova autorizzazione.

Mandato successivo dell'autore — ripresa2026-10-26:
- Autorizzato lo sviluppo delle task del piano dopo il checkpoint14-02.
- Niente Superpowers o task parallele; restano esclusi subagenti/background.
- Autorizzati push sul repository e avvii CI quando necessari.
- Dopo ogni task verificata proseguire automaticamente senza criticità.
- Fermarsi per criticità evidenti sottoponendo proposte risolutive all'autore;
  le decisioni esplicitamente aperte non sono ratificate implicitamente.
- Restano limite tre chiamate CI per gate distanziate almeno un minuto,
  spiegazioni dettagliate del codice, gate reali e recap file.

Ratifica DEC-18 —2026-10-26:
- Proposta catalogo approvata con precisazioni vincolanti; vedere
  docs/verification/DEC-18-ratification.md. Autorizzata T-P14-05.
- Catalogo separato da load; inventory reference schema-scoped e trust effettivo;
  import draft/lint reali, nessuna certificazione normativa implicita.
- STOP dopo T-P14-05 e relativi gate/handoff: nessuna task successiva senza
  nuova autorizzazione. Questo stop prevale sulla progressione generale.

Eccezione dell'autore — log MSVC T-P14-05,2026-10-26:
- Autorizzata una chiamata GitHub aggiuntiva per leggere il log diagnostico del
  job fallito121483043646, run40296287288. Nessuna estensione generale del polling.
