# Istruzioni operative Lintra

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
