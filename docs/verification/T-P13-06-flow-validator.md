# T-P13-06 — FlowValidator a due passate

2026-10-24. Dossier tecnico e spiegazione del codice richiesta da AGENTS.
Prerequisiti: T-P13-05 commit 37dff96, CI 40089317003 Linux/MSVC success;
T-P7-02 CI 39569282614. Preflight e contratti applicabili nel documento
T-P13-06-preflight.md. Questo dossier riguarda il core engine: il servizio
applicativo resta LocalOnly fino a T-P13-07. Nessun repository storico introdotto.

## Orchestrazione e confini

`FlowValidator` in engine/flow_execution.rs conserva soltanto schema compilato,
newline policy e snapshot reference immutabile opzionale. `new` rifiuta flussi
con dipendenze reference senza snapshot; `with_reference_data` controlla numero
delle evidenze e binding DatasetKey/digest usando il preflight reale del record
validator. Non costruisce dataset, observer NoOp o parametri runtime predefiniti.
Non importa application e non conosce CAD/SIOC.

`FlowTraceInput` prende in prestito TraceId e Path nativi. `prepare` verifica
trace ignoti, duplicati, path vuoti e required_traces del piano compilato prima
di qualunque lettura. Non interpreta file assenti come file vuoti. Verifica tutti
i parametri forniti e la presenza di quelli namespace richiesti anche con input
vuoti. Restituisce le coppie indice schema/path nell'ordine dichiarato dallo
schema, indipendentemente dall'ordine del bundle o di HashMap. Non canonicalizza
né ricodifica i path; la lettura e l'errore filesystem restano dell'input adapter.

`FlowExecutionPorts` raggruppa sink, builder, audit session, controllo, observer
reference e cross reali. `FlowReferenceObservers::for_trace` consegna il collector
appropriato a ciascun trace; il preflight lo richiede prima dell'input quando
esistono reference. Un observer restituito per l'intero bundle attribuirebbe
erroneamente controlli a un singolo trace: questo port evita tale assunzione.
Gli errori dei port conservano source tecniche, senza diventare ValidationError.

`FlowValidator::validate` prepara tutto il bundle e verifica che il builder sia
nuovo e appartenga allo schema/scope Flow ordinato. Crea uno store in memoria
locale all'invocazione. Per ogni trace crea un collector e TraceValidator reale,
passa attraverso la stessa apertura audit/BOM/framing di LocalOnly e costruisce
l'indice. Prima della seconda passata ricontrolla la cancellazione. La passata
cross usa esclusivamente ContextStore e bytes proiettati: nessuna riapertura dei
file. FlowValidator può essere riutilizzato con un nuovo builder e nuovi port;
contesto e indici PK non sopravvivono al run.

`FlowControl` usa callback sincrone neutrali, senza token applicativi/thread.
`FlowEvent` avvolge gli eventi trace con TraceId e aggiunge inizio/progresso della
passata cross e completamento/annullamento. La cancellazione viene controllata
prima di ogni trace, nel loop comune prima di BOM/ogni record, prima della passata
cross e fra i record cross. Un record acquisito e le sue regole sono processati
atomicamente rispetto alla cancellazione cooperativa. Nessun drain a EOF per
completare un digest. Un errore tecnico restituisce Err, senza evento Completed
né pubblicazione di summary; Completed e Cancelled sono risultati controllati.

## Riutilizzo del ciclo trace e della validazione locale

`RecordPhase` distingue LocalOnly e FullPassOne. `validate_full_pass_one` riusa
`validate_record`: stessi campi, stage, identity, contatori, errori tecnici e stato
terminale. Solo LocalOnly conta le regole cross come saltate durante la lettura;
FullPassOne le rimanda alla passata cross. ReferenceRecordValidator.validate_phase
riusa lo stesso ReferenceExecution, compresi observer e marcatura dei risultati
non definitivi. Le API LocalOnly esistenti mantengono firma e comportamento.

`FullRecordCollector` è un hook engine interno, sincrono, chiamato dopo che il
record ha prodotto veri artifacts locali e contatori/errori sono stati accettati.
Riceve in prestito i byte originali e può copiarne soltanto le proiezioni. Il
metodo interno TraceValidator.validate_full_path usa ancora InputAuditSession.open
una sola volta e lo stesso loop run. Il collector opzionale seleziona la fase
Full; la sua assenza mantiene il vecchio percorso LocalOnly. Un errore hook è
TraceValidationError::Collection con causa preservata e rende il trace terminale.
Il consumer applicativo del vecchio errore è aggiornato esaustivamente, senza
attivare Full nel servizio in questa task.

## Proiezione, chiavi e memoria

`FlowState` contiene InMemoryContextStore, posizioni già invalide, evidenze per
trace, contatori PK, aggregati cross e totali. Non contiene un elenco di errori
né copie delle righe complete. L'ordine pubblico delle evidenze è quello dello
schema e delle regole compilate. HashMap/HashSet servono a lookup, non a decidere
l'ordine degli output. Gli indici sono effimeri e distinti da PersistentContextRepository.

`ProjectionCollector::collect` registra la posizione processata e l'esito locale.
Se la larghezza è errata non tenta recuperi: incrementa unprojectable_records e
le verifiche PK inconcludenti; l'errore di lunghezza è già stato emesso dalla
pipeline locale. Se la larghezza è corretta, usa build_join_key sul RecordKeyPlan.
La prima posizione di ciascuna chiave resta nell'indice per quel trace. Una
ripetizione emette il codice dichiarato dal piano, riferito alla riga successiva;
lo stesso valore in un altro trace non è duplicato. Una componente Missing è
inconcludente, non una chiave vuota inventata; altri errori di estrazione sono
tecnici. I contatori PK/righe sono limitati dal line counter checked del lettore.

`ErrorFactory::duplicate_record_key_error` centralizza la diagnostica record-level:
identità reale, codice da schema, riga corrente e prima posizione nel messaggio;
nessun campo/valore grezzo inventato. L'errore viene contato dopo sink.on_error.
La riga duplicata non viene rimossa dal contesto: eventuali join mantengono tutte
le candidature e possono rilevare ambiguità. Non ci sono confronti inter-invio.

Infine il collector chiama project_record solo quando il piano lo richiede e
inserisce il risultato reale nello store. Un record di lunghezza corretta ma
normativamente invalido contribuisce comunque alle verifiche cross indipendenti.
Memoria O(proiezioni selezionate + indici join/PK + posizioni invalide); nessuna
promessa di memoria costante per il contesto Full. Finito il run viene rilasciato.

## Copertura del contesto separata dall'audit

`TraceContextCoverage` distingue Complete, Partial e Unavailable.
`TraceCoverageEvidence` conserva TraceId, record processati e record non
proiettabili. Complete viene assegnato soltanto dopo il completamento del loop e
con zero righe non proiettabili. Un trace che ha elaborato record e viene annullato resta Partial; un
trace senza contesto elaborato resta Unavailable. La copertura riguarda il contesto, non il
digest: l'audit può essere Complete con EOF e SHA corretto mentre l'indice è Partial.

`CoveredCrossRecordInputs` aggiunge una mappa esplicita senza cambiare la firma
precedente CrossRecordInputs. `ProjectedEvaluationContext::with_coverage` e la
funzione validate_trace_coverage richiedono una voce per ogni trace dichiarato:
nessuna omissione può implicare Complete, né Unavailable può contraddire la lista
di input disponibili. Il vecchio constructor new continua a essere usato per
store standalone dichiarati completi. lookup_coverage conserva la provenienza
per ogni join senza cambiare RelatedLookup pubblico.

`scalar_resolution` conserva le ambiguità osservate (più record per un join
scalare); se il contesto è Partial non afferma unicità o consenso completo e
restituisce UnknownReason::IncompleteRelatedContext. Anche un lookup vuoto
parziale usa quel motivo, distinto da RelatedRecordMissing. L'evaluator e la
risoluzione delle cause preservano JoinId: le policy on_missing_record/trace non
possono trasformarlo in una violazione normativa o in NotApplicable.

`existence` può provare una presenza reale anche con copertura parziale. Una
ricerca vuota o un SetExists senza predicato soddisfatto non è conclusiva su
indice parziale. Un match positivo del predicato è ancora conclusivo. Gli esiti
Unknown non vengono scritti nel sink normativo. Primitive non implementate e
piani incoerenti restano errori tecnici/compiler, non casi di contesto degradato.

## Contatori, aggregazione e finalizzazione

`CrossPassControl` aggiunge coverage e checkpoint/progresso al runner comune.
La vecchia validate_cross_record_rules mantiene la firma, passando contesto
completo e nessuna richiesta di cancellazione. Il runner conta ogni coppia
record-sorgente/regola una volta, in ordine trace/riga/regola. Calcola i nuovi
contatori prima dell'emissione, notifica l'observer dopo sink.on_error e conserva
la causa di ogni fallimento. Non apre input, non materializza unioni di record.

`FullCrossObserver` aggiorna il builder per le sole violazioni già emesse. La
posizione (TraceId, linea) viene inserita nell'insieme invalid: soltanto una nuova
posizione prima valida richiede riclassificazione valid→invalid. Errori locali,
duplicati PK e più errori cross sullo stesso record non raddoppiano invalid_records.
Gli errori normativi continuano a essere contati singolarmente per codice.

`ValidationSummaryBuilder::invalidate_previously_valid_record` verifica underflow
e overflow prima della mutazione; non incrementa processed. counters restituisce
ValidationCounters dalla sola fonte autorevole. rules_skipped aggiunge in batch
un numero checked di valutazioni note omesse. FlowState.finish calcola per ogni
regola le righe processate meno le valutazioni realmente eseguite: comprende
righe non proiettabili e righe rimaste in attesa alla cancellazione, mai righe
non lette né il numero di ID regola distinti. Questa quantità aggiorna anche lo
skipped_rules esistente; non c'è un contatore alternativo con unità incompatibili.

CrossControlEvidence contiene i cinque contatori disgiunti del pass summary,
gli skip e cause aggregate. CrossEvidenceCause distingue Unknown(reason) e
NotApplicable con JoinId e diagnostica compilata, conteggiando valutazioni.
Le cause sono limitate dalle regole/join/motivi del piano, senza valori sanitari,
warning per riga o crescita proporzionale al numero di ripetizioni dello stesso
problema. Il callback esterno rimane disponibile per consumer reali.
NotApplicable indica scope non applicabile: non va confuso con un Unknown.
Il nome del contenitore tecnico non lo trasforma in violazione o risultato True.

FlowValidationArtifacts restituisce completion, snapshot dei contatori, coperture,
verifiche PK e aggregati cross reali. Non è un ValidationSummary persistente.
begin_full marca il builder; finish e finish_with_reference_data legacy non
possono finalizzare perdendo l'envelope Full (UnreportedFullEvidence). Questo vale
anche dopo errore o annullamento. Non viene prodotto un falso Complete mentre il
contratto finale aspetta T-P13-07. Reference evidenze continuano nei collector
reali; T-P13-07 dovrà comporle con queste evidenze nel summary/audit.

## Residui espliciti

T-P13-07: servizio Full, scope/parametri runtime, envelope pubblico persistente,
completezza e serializzazione, conversione delle evidenze engine verso dominio,
finalizzazione esplicita del builder con validazione dell'envelope, aggregazione
reference per più trace, pubblicazione atomica report. Considerare anche PK
inconcludenti e trace non completati nella completezza. NonApplicable conserva
la sua semantica di scope e non equivale automaticamente a controllo inconcludente.
Non modificare ValidationRequest/baseline normative senza il preflight minimo.

PERSIST: storico/sessioni/pseudonimizzazione restano obblighi futuri distinti.
P16: UI dei controlli saltati/provvisori/inconcludenti; nessuna UI qui.
Restano esclusi max_errors, riferimento temporale/metadata non implementati,
recupero di righe malformate e normative hardcoded. Le API pubbliche aggiunte sono
additive; l'aggiunta di varianti enum richiede aggiornamento dei match esaustivi
(consumer interni aggiornati), senza riscrivere la cronologia delle ratifiche.

## Verifiche e handoff

Test integration tests/flow_validator.rs: bundle e namespace preflight prima di
I/O, ordine schema con input invertiti e path Unicode, SHA indipendente sui byte
originali CRLF/LF, due passate reali senza skip fittizi, riclassificazione multipla
e record già localmente invalido, PK trace-local con parent non deduplicati,
indice parziale/EOF completo senza falso parent mancante, presenza positiva e
SetExists su contesto parziale, cancellazione prima/durante entrambe le passate,
1024 record senza drain, reference Verified/Degraded/Unavailable e observer per
trace, errori observer/sink e guard contro finalizzazione legacy, coverage
incoerente rifiutata. Regressioni cross/trace/reference incluse nel gate completo.

Esiti effettivi e commit/CI sono registrati dopo l'esecuzione nel registro di
avanzamento. Nessuna verifica nativa dichiarata sulla sola base del cross-check GNU.

### Gate locale effettivamente eseguito

2026-10-24: cargo fmt --all -- --check; cargo check --all-targets; cargo clippy
--all-targets -- -D warnings; cargo test (589 passati, 68 suite, 0 fail,
1 probe memoria ignored); cargo build --release; cargo check --all-targets
--target x86_64-pc-windows-gnu. Cargo con --offline --locked e cache isolata
/tmp/lintra-cargo-home; nessun nuovo crate. git diff --check passato.
Tutti i 90 file protetti confrontati byte per byte con 6e4b411 sono invariati.
Il cross-check GNU non è il gate nativo MSVC, ancora da eseguire.

### Gate nativo e chiusura

Codice 134b593edcd02c055e0037fe3e65c6cd12152d1c pubblicato. CI 40091666797:
Linux GNU job 120865434899 e Windows MSVC job 120865434680 completed/success,
verificati direttamente circa 11:59 UTC del 2026-10-24. Tre chiamate Actions
complessive, separate da almeno 45s. T-P13-06 DONE, nessuna capacità del servizio
Full o dell'envelope persistente dichiarata prima di 13-07. Handoff e recap in
IMPLEMENTATION_STATUS; prosecuzione automatica autorizzata verso 13-07.
