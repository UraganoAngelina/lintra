# T-P9-03 — Preflight Decimal / BLOCKED

2026-10-22. Nessun codice Decimal scritto. T-P9-02 conclusa: CI 39749927727
completed/success su fa547431e702e37f2cf53203a3286d1f11ae9a32,
Linux GNU e Windows MSVC success verificati, 340 test locali passati.
Letti prompt T-P9-03, DET §137, Piano P9/§4/14, SCHEMA_AUTHORING tabella tipi,
G0/T-P0-07 e compatibilità formato approvata. Non è un nuovo CNF né riapertura
di ratifiche: il rinvio Decimal esiste già ed è vincolante.

## BLOCCO — contratto Decimal non approvato

**Descrizione.** La task chiede DecimalSpec/validator, con separatore, scala e
segno secondo schema. Il formato 1 approvato esclude Decimal; nessun tracciato
ricevuto ne definisce la rappresentazione. Non si può introdurre un nodo raw,
compilare un piano o pubblicizzare supporto Decimal senza aggiornamento approvato.
Una struct isolata con validator non raggiungibile dagli schemi non chiuderebbe
la task e non deve essere presentata come capacità implementata.

**Fonti ed evidenze.**
- TASK_PROMPTS/T-P9-03.md nello starter kit: DecimalSpec/validator, boundary
  separatore/scala/segno secondo schema, prerequisiti T-P6-06 e G0.
- docs/DETAILED_DESIGN.md §137: esempio total_length/scale/decimal_separator/
  sign_policy; «La definizione concreta dipenderà dai tracciati».
- docs/SCHEMA_AUTHORING.md tabella tipi: Decimal/Alphabetic non ammessi dal
  formato; introduzione richiede evidenza, aggiornamento contratto ed esempi.
- IMPLEMENTATION_STATUS.md checklist G0, voce FieldType: esplicito rinvio
  T-P9-03, non accettabile come supportata senza ampliamento approvato.
- docs/verification/T-P0-07.md: rinvio perché nessun campo ricevuto richiede
  Decimal; rifiuto del tipo invece di inventare lessico/separatori.
- docs/schema-authoring/flow.schema.json e src/schema/raw.rs RawPhysical:
  soltanto numeric/alphanumeric/date. src/schema/compiled/plans.rs FieldType:
  stessi tre tipi. Nessun nodo Decimal forgiabile in un compiled verificato.
- CNF-FORMAT-01 è già ratificata: solo formato 1 definito. Non si assegna un
  nuovo formato o una compatibilità motore senza contratto e mappa approvati.

**Impatto.** T-P9-03 BLOCKED. T-P9-05 richiede 9-01..04 e non può considerare
soddisfatta Decimal; quindi la chiusura P9/M4 completa non può essere dichiarata.
T-P9-04 Regex ha prerequisito autonomo T-P6-06, già soddisfatto, e può essere
prossima candidata mantenendo Decimal aperta se l'autore conferma il riordino.
Il mandato corrente consente stop in presenza di blocchi contrattuali sostanziali:
non si presume che autorizzi anche un nuovo formato o il superamento del rinvio.
Le task concluse e la CLI LocalOnly restano utilizzabili; nessun rollback necessario.

**Alternative circoscritte.**
1. Conservare formato 1 e rinvio Decimal, proseguire con T-P9-04; tracciare
   apertamente 9-03/9-05 e non dichiarare P9 completa. È l'opzione che preserva
   integralmente il contratto esistente e non richiede nuove assunzioni normative.
2. Definire e ratificare un'estensione Decimal prima di 9-03: rappresentazione
   fisica esplicita (inclusione separatore/segno nella larghezza), separatori
   ammessi, scala e precisione, segno/posizione, padding/missing, constraint
   compatibili, AST raw/compiled, esempi validi/invalidi e versioning/mappa del
   formato. Tutte le scelte devono essere esplicite; nessun default globale,
   floating point o modifica a SIOC/CAD già ricevuti.

**Decisione richiesta all'autore.** Confermare il rinvio e prosecuzione da
T-P9-04, oppure fornire/approvare il contratto Decimal completo e il relativo
aggiornamento documentale. Nessuna modifica proposta alle baseline normative.
