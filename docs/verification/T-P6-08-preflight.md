# T-P6-08 — Preflight / CNF-RECORD-01

2026-10-20. **CNF-RECORD-01 RISOLTA: proposta A APPROVATA con precisazioni
vincolanti dell'autore**. Implementazione T-P6-08 LocalOnly autorizzata.
Nessuna modifica a ARCH/DET/Piano. Le evidenze del blocco sotto sono storiche.

## Ratifica e precisazioni vincolanti

L'autore approva validate_local e LocalRecordValidationArtifacts { identity,
valid }, con valid riferito solo ai controlli locali supportati. Nessun
projected_record/placeholder. RecordValidationError nell'engine conserva i
payload reali senza dipendere dall'application. ProjectionPlan invariato,
anche non vuoto, non impedisce LocalOnly; regole/condizioni locali necessarie
non supportate sono invece rifiutate. Reference data e proiezioni restano P11/P13.
ValidationRunError incrementale autorizzato; T-P2-03/M1 restano aperte.
Chiusura consentita solo come **DONE — LocalOnly scope approved by CNF-RECORD-01**,
con rinvii e test futuri espliciti. Nessuna modifica alle semantiche degli esiti.
Il riordino P7 è ratificato come piano, **non autorizza l'esecuzione**:
STOP dopo T-P6-08 e handoff, attendere nuovo mandato prima di qualsiasi task.
Questa precisazione supera il punto operativo 7 e l'handoff storico sottostanti.

## Evidenze e impatto

Prerequisiti atomici implementati: T-P6-06 65f1eb3 / CI 39437225233
Linux/Windows verde; T-P5-05 85f1ac7 / CI 39399359949 verde.
T-P6-07 c51c019: 233 locali; CI 39439627879 Linux/Windows verde.
Il gate di T-P6-07 è soddisfatto e resta separato da questa decisione.

Il prompt e Piano T-P6-08 citano DET §26/48–49/87–88/134. Dopo lettura e
ricerca in src/tests, i contratti necessari alla firma completa non esistono:

| Contratto richiesto | Fonte | Assegnazione e dipendenza reale |
|---|---|---|
| PreparedReferenceData e relativi provider/errori | DET §48/63–64 | T-P11-01 dipende da T-P7-05, il servizio ancora da costruire. |
| ProjectedRecord, JoinKey, ContextError, project_record | DET §48/66–69/89 | T-P13-01..02 dipendono da P10, che richiede il percorso applicativo successivo a T-P6-08. |
| ValidationRunError applicativo completo | DET §48/61, CNF-DOMAIN-01 | Include InvalidRequestError (T-P7-01), ReferenceDataError (P11), ContextError (P13); residuo T-P2-03/M1 già dichiarato aperto. |

Esiste inoltre un problema di direzione: CNF-DOMAIN-01 assegna ValidationRunError
all'application, mentre DET §48 lo restituisce dall'engine. Un'importazione
engine->application contraddice DET §5/Piano N-02 e crea la dipendenza inversa.
Il wrapper applicativo deve comporre gli errori dell'engine, non viceversa.

Non è lecito restituire sempre projected_record=None come se il piano fosse
vuoto: il compiler corrente include gli indici PK nella ContextProjectionPlan.
T-P3-02 ha esplicitamente sostituito required con la presenza degli indici
(report T-P3-02, sezione piani). Non cambieremo questo fatto di nascosto.
Né introdurremo Option<()>, errori String o tipi segnaposto per soddisfare firme.

La consegna completa letterale genera quindi un ciclo di dipendenze. Non è
una carenza dei controlli locali già verificati, né un nuovo requisito normativo.
Piano T-P7-01 prevede già Full non supportato fino a P13 e T-P7-04 reference
vuoti fino a P11, ma non esplicita come raffinare il contratto record in P6.
Serve una decisione di confine e consegna, oltre alle rifiniture DET §147.

## Proposta A — consegna LocalOnly esplicita per dipendenze reali

Approvare questo perimetro, senza anticipare algoritmi P11/P13:

1. T-P6-08 consegna il percorso `RecordValidator::validate_local` con un
   risultato **LocalRecordValidationArtifacts { identity: Option<RecordIdentity>,
   valid: bool }**. Il tipo non contiene una finta proiezione opzionale.
   Il contratto completo RecordValidationArtifacts/project_record resta
   assegnato all'integrazione P13, con consegna futura esplicita.
2. `validate_local` verifica StrictSkipRecord, slicing checked, identity,
   field pipeline e contatori. Record corto/lungo: solo E_RECORD_LENGTH,
   nessuna identity/field validation. Gli errori normativi restano esiti
   controllati; sink/counter/piano invalido restano errori tecnici tipizzati.
3. Definire nell'engine `RecordValidationError` con i **payload reali** già
   esistenti (IdentityExtractionError, FieldPipelineError, ReportError,
   SummaryBuildError) e varianti tecniche contestuali per invarianti/plan
   non supportati. Nessun ValidationError dentro Err; nessun import application.
4. La firma locale non prende PreparedReferenceData; il compiler continua a
   rifiutare ReferenceData e altri costrutti non implementati. Nessun accesso
   a dataset inventati. La futura integrazione P11 aggiungerà il contesto reale.
5. La modalità locale **non consuma né produce** proiezioni destinate al
   percorso Full. Questo comportamento è esplicito nel nome/contratto locale;
   il piano compiled resta invariato e non viene dichiarato vuoto/soddisfatto.
   Full resta errore esplicito fino all'integrazione P13. Anche local rules
   o condizioni non supportate devono essere rifiutate, mai saltate in silenzio.
6. In P7 l'application mapperà l'errore record con un payload tipizzato.
   ValidationRunError viene consegnato incrementalmente con sole varianti
   concretamente disponibili; ReferenceData/Context saranno aggiunte con i
   moduli competenti, senza messaggi generici al posto dei payload mancanti.
   T-P2-03/M1 restano aperte fino al residuo completo, come già approvato.
7. Autorizzare il riordino limitato su prerequisiti atomici reali: dopo il gate
   della consegna locale T-P6-08, procedere in P7, inclusa T-P7-02 usando la
   parte autonoma T-P2-03 già verificata. Ogni task conserva test e CI nativa.
   Stato T-P6-08: **DONE nello scope LocalOnly approvato**, con contratto Full
   futuro tracciato; nessuna pretesa di aver consegnato il contratto P13.

Questa proposta cambia confini delle firme e ordine di consegna, non gli esiti
CompletedInvalid/Cancelled/Err, il formato normativo o gli obblighi di audit.
Se approvata, registrare ratifica e differenze dal contratto sorgente nei report
senza modificare retroattivamente ARCH/DET/Piano o baseline normative.

## Alternativa B

Mantenere il blocco e ricevere dall'autore un contratto completo e un ordine
alternativo che risolvano il ciclo, compresa la proprietà degli errori e la
consegna di projection/reference data. Nessuna anticipazione automatica P11/P13.

## Handoff

Codice T-P6-08 non scritto. T-P6-07 e la sua CI sono completati. L'avanzamento interessato (T-P6-08/P7/P8 e dipendenti) attende
ratifica; non aggirare il gate passando direttamente alla task seguente.
