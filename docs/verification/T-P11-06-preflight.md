# T-P11-06 — Preflight circoscritto dei contratti applicativi

2026-10-23. Prerequisito 11-05: locale verificato, CI nativa in attesa.
Nessun codice 11-06 iniziato prima del gate. Autorità: CNF-REFERENCE-FORMAT-01
aggiornata §§9–12/17–18 e workflow automatico successivo dell'autore.
Il vecchio fail-fast indiscriminato del prompt è superato per Continue.

## Proposta minima di evoluzione

1. Port additivo di preparazione con evidenze, distinto dal vecchio load bool:
   riceve le dipendenze compilate e una sola policy tipizzata. Adapter filesystem
   con root esplicita; l'application non costruisce dataset fittizi. Costruttori
   esistenti del service preservati, configurazione reference aggiunta esplicitamente.
   Flow che richiede reference senza configurazione continua a fallire prima
   degli input. Strict fallisce sulla dipendenza non verificata, Continue conserva
   disponibilità e consente gli altri controlli; confini FS sempre fatali.
2. Modelli audit nel dominio neutro: disponibilità/policy, ID/versioni richiesti
   e dichiarati, digest atteso ed effettivo, motivi tipizzati. Nessuna causa OS,
   path assoluto o messaggio parser grezzo nei report; nessuna dipendenza domain
   verso il provider filesystem. Conservare API dei tipi già pubblicati con re-export.
3. Estensione del ValidationSummary con envelope reference opzionale e una
   completezza tipizzata. Assenza di envelope nei vecchi run senza dipendenze
   conserva il JSON canonico precedente; run reference pubblica esplicitamente
   Complete/Partial e le evidenze anche dopo fine sessione. RunStatus invariato:
   CompletedValid significa assenza di violazioni rilevate, non prova di piena
   conformità quando la completezza è Partial. Console deve mostrarlo chiaramente.
4. Finalizzazione esplicita nel builder con il nuovo envelope verificato;
   il vecchio finish conserva la guardia UnreportedReferenceEvidence e non
   può cancellare il requisito ignorando gli artefatti. RunMetadata non necessita
   sostituzione: il summary/audit finale può ricevere le evidenze separatamente.
5. Aggregazione deterministica bounded per controllo compilato (field constraint
   o reference assertion di rule), mai per valore paziente/riga: valutazioni
   definitive soddisfatte/violate, provvisorie e inconcludenti disgiunte, overflow
   checked. Unità dichiarata: valutazione del predicato reference, non numero
   di regole distinte né numero di ValidationError. skipped_rules esistente non
   duplicato o incrementato per warning. Cause e sorgenti nello snapshot audit;
   controlli non applicabili per presence/stage precedenti non contati come
   tentativi di membership. Cancellazione/troncamento preservano extent reale,
   senza inventare il numero di record non letti.
6. Version map legacy: non registrare degradati come verificati. Envelope con
   DatasetKey completa conserva tutte le versioni, evitando perdita quando lo
   stesso dataset_id ha più versioni dichiarate. Map legacy non è fonte completa
   per il nuovo audit: includere soltanto ID con una sola versione Verified;
   omettere proiezioni ambigue o sole evidenze degradate. Envelope completo
   autorevole conserva tutte le DatasetKey e la relativa attendibilità.
7. Service prepara prima del primo input, poi usa il percorso reference-aware
   reale con gli stessi checkpoint cancel/BOM/audit input; JSONL contiene solo
   violazioni. Report parziale completato pubblicabile con completezza esplicita;
   fatalità tecniche mantengono rollback atomico. CLI usa la root già risolta;
   UI/storico/persistenza rimangono future, non simulate.

## Verifiche prima della consegna

E2E con dataset verificato/degradato/mancante, controlli indipendenti e reali
violazioni; nessun falso Complete, nessun warning in total_errors/errors_by_code;
policy/hash/versioni effettivi nel summary riletto; aggregazione su molti record;
Strict prima di input assente; symlink sempre fatale; cancellation distinta da
parziale completato/fatalità; JSONL e tutti i golden precedenti invariati dove
non esistono reference; gate locali e Linux/Windows nativi.

## Aggregazione e provenienza senza crescita per record

Preallocare le righe dei controlli nell'ordine compilato, indicizzate per field
e constraint index oppure rule ID/assertion index; callback senza copie delle
chiavi paziente. Conteggi checked e categorie mutuamente esclusive per tentativo.
Assenze per presence/stage precedente rendono il predicato non applicabile: non
inventare tentativi di lookup. Il contatore skipped_rules esistente conserva
le vere valutazioni non eseguite; nessun duplicato per un warning di membership.

Le dipendenze della singola Union sono conservate in ordine. Per VerifiedPresent
la ricerca consulta il prefisso fino alla prima presenza Verified; per negativo
o Provisional consulta tutti i set disponibili. Per fallback dichiarato non
consulta set. Questo consente contatori bounded delle sorgenti effettivamente
consultate e della sorgente vincente senza memorizzare record o ripetere lookup.
Le sorgenti Unavailable restano richieste e in audit, ma non sono HashSet consultati.
Completezza Partial deriva da valutazioni provvisorie/inconcludenti o esecuzione
interrotta, non dalla sola presenza di un dataset mancante quando la membership
è già dimostrata da un'altra sorgente verificata.

## Firme e proprietà dei tipi previste

- `ReferenceDataPreparer::prepare(&self, &[ReferenceDependency], policy)` restituisce
  uno snapshot con evidenze oppure un errore tecnico concreto. Non cambia load
  legacy e non promuove un HashSet in-memory senza digest a file Verified.
- Adapter `FileSystemReferenceDataPreparer`: root esplicita, binding del flow
  compilato ricevuti a ogni preparazione, nessun provider vuoto fittizio nel loop.
- `ValidationService::with_reference_data(preparer, policy)` additiva ai
  costruttori attuali; nuova factory con entrambe le root, selezionata dall'adapter.
  La policy è configurazione esplicita del service e audit della preparazione,
  non un secondo default indipendente o una nuova flag normativa nel record.
- `ValidationSummary.reference_validation: Option<ReferenceValidationSummary>`:
  nuovo campo pubblico e aggiornamento necessario dei literal dei consumer Rust;
  nessuna modifica alle altre firme. Il JSON omette l'envelope quando non è
  applicata alcuna preparazione reference, preservando i golden senza dipendenze.
- `ValidationSummary::completeness()` restituisce il tipo Complete/Partial:
  tiene conto di cancellazione/troncamento/controlli saltati ed evidenze non
  definitive. Nessuna sostituzione di RunStatus o nuovo esito tecnico/normativo.
- `ValidationSummaryBuilder::finish_with_reference_data(metadata, audit)`:
  percorso esplicito, validazione di conteggi/disponibilità/digest/proiezione
  legacy e completezza. Il vecchio finish mantiene la guardia di 11-05.

DatasetKey e gli enum puri di disponibilità/policy/formato possono essere
spostati nel dominio neutro conservando gli attuali re-export pubblici. Questo
evita domain→schema/provider o duplicati incompatibili di DatasetKey. Restano
nel provider le cause IO/JSON reali: il dominio riceve soltanto motivazioni
trasportabili tipizzate, nessuna std::io::Error o path host nel summary finale.
Envelope conserva ID/versione richiesti e letti, digest atteso e reale distinti,
path relativo dichiarato, stati/anomalie, policy e contatori per controllo.

La proiezione legacy versioni non è una nuova regola normativa: ID con una sola
versione Verified entra nella map, ID ambiguo o solo degradato non riceve una
versione falsamente verificata. Tutte le versioni, anche assenti/degradate,
rimangono nel nuovo audit autorevole con il loro vero stato.

Consumer console/CLI: nessuna dichiarazione di piena conformità su Partial;
JSONL invariato con soli ValidationError. Exit code segue l'esito di esecuzione
normativa già approvato (0 nessuna violazione rilevata, 2 violazioni, 1 errore
tecnico, 130 cancellazione); la completezza esplicita è una dimensione distinta
nel summary. Nessun successo pienamente conforme dedotto dal solo exit code.

## Invarianti della finalizzazione da verificare

- Verified richiede digest effettivo uguale all'atteso, identità dichiarata
  coerente e assenza di anomalie; Degraded non può essere serializzato Verified.
- Snapshot restituito dal preparatore deve corrispondere alle dipendenze e alla
  policy richieste; nessuna sostituzione con versioni/latest o altri digest.
- Contatori per controllo: somma checked; una valutazione ha un solo esito
  finale. E_REFERENCE_DATA è emesso soltanto da assenza verificata; numero di
  violazioni di predicato e numero di ValidationError sono metriche distinte.
- App e builder verificano la coerenza fra marker non definitivo osservato,
  contatori aggregati e completezza. La nuova finalizzazione non può cancellare
  la guardia di 11-05 limitandosi ad allegare un envelope vuoto.
- Un match Verified in Union parziale resta conclusivo: indisponibilità delle
  altre dipendenze in audit e diagnostica, senza falso Partial sulla sola causa.
- Policy Strict: errore prima dell'apertura dell'input; Continue: input e regole
  indipendenti proseguono. Symlink/escape rimangono errori fatali in entrambi.
- Cancellazione e limite operativo conservano l'audit EOF effettivo e rimangono
  distinti da esecuzione completata con Complete oppure Partial.
- La cardinalità della diagnostica aggregata dipende dal piano e dai dataset,
  non dal numero di record o dai valori trovati/non trovati. Nessun PHI in key.
