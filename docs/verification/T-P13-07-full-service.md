# T-P13-07 — Servizio Full, audit persistente e validate-flow

Ripresa e consegna in lavorazione 2026-10-25 (data client). Preflight scritto
2026-10-24 prima del codice, poi precisato prima di rendere opzionale l'observer
cross esterno. Prerequisiti verificati: 13-06 codice 134b593, CI 40091666797
Linux/MSVC success; 8-03 codice 10932f9, CI 39743161012 success.
Questo dossier spiega struct, funzioni e condizioni nuove/modificate come
richiesto dall'autore. Nessuna baseline, grammatica o fonte architetturale modificata.

## Contratti pubblici e compatibilità

ValidationRequest mantiene key/scope/mode/options (DET §25). validate mantiene
la firma e chiama validate_with_parameters con mappa vuota. Quest'ultima riceve
binding testuali espliciti per il run, senza cambiare configurazione del servizio
né inferire valori mancanti. Le factory storiche con nome local rimangono
compatibili: il servizio composto ora accetta anche Flow/Full. LocalOnly/Trace
mantiene il percorso precedente; LocalOnly/Flow e Full/Trace sono rifiutati.
L'helper pubblico preflight/PreflightPlan resta compatibile come helper trace-only;
la preparazione Full usa preflight_flow_inputs reale, non il primo trace del bundle.

ValidationSummary aggiunge Option<FullValidationSummary>. I costruttori Rust che
usano struct literal devono indicare full_validation; i consumer interni/test
sono aggiornati esplicitamente. None nei run LocalOnly non produce nuovi campi
JSON: i golden tecnici precedenti restano identici. In Full vengono serializzati
full_validation e completeness, oltre al result normativo già esistente.
RunStatus non viene sostituito né reinterpretato: VALID + partial non è piena
conformità. L'aggiunta di varianti errore/progresso richiede aggiornare eventuali
match esaustivi esterni. Sono evoluzioni minime esplicitate nel preflight.

## DTO neutri del dominio

full_validation.rs nel dominio non importa engine, schema o application.
ContextCoverage distingue Complete/Partial/Unavailable. FlowTraceEvidence
conserva TraceId, copertura, righe processate/non proiettabili, conteggi PK e
cross_rules_declared. Quest'ultimo conta definizioni distinte; tutti gli altri
conteggi cross misurano valutazioni su record fisici, non ID regola.

CrossUncertaintyReason è la rappresentazione persistente neutra dei motivi
consegnati dal motore. CrossCauseKind distingue Unknown(reason) e NotApplicable.
CrossEvaluationCause conserva JoinId opzionale, codice/template diagnostico e
numero di valutazioni; nessun valore del record. CrossControlSummary raccoglie
trace/rule_id, valutazioni, satisfied, violations, not_applicable, unknown, skipped
e cause aggregate. Le quattro categorie valutate sono disgiunte; skipped è
esterno a evaluations. Non vengono creati ValidationError per Unknown o N/A.

RunParameterAudit conserva tipo e valore effettivo: stringa byte/testo invariata,
intero in forma decimale canonica esatta. FullValidationSummary contiene binding
in BTreeMap deterministica, evidenze trace in ordine schema e controlli in ordine
compilato, con completezza esplicita. has_non_definitive_evaluations considera
Unknown/skip; partial considera anche copertura insufficiente e PK inconcludenti
per i soli trace nello scope. N/A non è automaticamente Partial: indica una
regola fuori applicabilità, senza convertirla in un pass o in un Unknown.

FullValidationSummary.valid verifica prima della serializzazione:
- scope Flow e input audit senza duplicati; ogni trace con record ha audit;
  coverage Complete richiede audit EOF/SHA Complete reale;
- TraceId non vuoti/unici, righe non proiettabili <= processate, duplicati PK <=
  PK controllate, checked + inconclusive = righe, inconclusive >= non proiettabili;
- trace fuori scope con zero record e coverage Unavailable; indisponibilità non
  accompagnata da record; coverage Complete senza righe scartate;
- salvo interruzione, trace nello scope effettivamente disponibili e Partial
  giustificato da righe non proiettabili; somma righe uguale al summary;
- numero di controlli presenti per trace uguale alle definizioni dichiarate;
  trace/rule_id validi e unici, evaluations + skipped = righe della sorgente;
- somma delle quattro categorie = evaluations; skip aggregati = skipped_rules;
  violazioni cross + duplicati PK non superiori al totale normativo del summary;
- cause non vuote, non duplicate, con JoinId/codice/template coerenti;
  N/A richiede join/diagnostica e le somme delle cause coincidono con Unknown/N/A;
- completezza esattamente derivata da interruzione, controlli non definitivi,
  copertura, PK e reference; interi audit canonici e ID parametri non vuoti.
Ogni somma usa checked arithmetic: overflow non significa wrap o conformità.

## Builder e finalizzazione autorevole

begin_full ora acquisisce anche gli ID trace/controlli dal piano compilato,
in una FullExecutionExpectation privata. La modifica è interna engine↔dominio:
nessuna dipendenza inversa. Questa attesa è fissata prima dell'input e non può
essere fornita da un consumer del report. finish_full rifiuta envelope con trace
o regole mancanti, aggiunti o riordinati rispetto al piano acquisito.

finish_full richiede marker Full e metadata.mode Full; verifica che le evidenze
reference non definitive corrispondano al marker reference. Determina completezza
reference e Full dalle condizioni reali, poi usa il finish_inner comune.
Quest'ultimo richiede parità fra marker Full ed envelope e invoca summary.validate.
I finish legacy continuano a rifiutare un run Full senza evidenze. Non basta
cancellare controls per ottenere un falso Complete: attesa privata nel builder
e cross_rules_declared nel DTO rendono visibile l'omissione anche dopo il run.
ValidationSummary.completeness combina le dimensioni senza cambiare RunStatus.

## Binding e conversione applicativi

application/full.rs contiene bind_parameters: cerca la ParameterDefinition
compilata per ogni ID fornito. String resta String anche se contiene solo cifre;
Integer usa IntegerLiteral senza floating point. Parametri ignoti o interi non
interpretabili sono errori tecnici di binding. Il preflight engine successivo
controlla domini/range e namespace richiesti prima di leggere input o reference.
Non ci sono default di anno/ente né selezioni dal primo valore ammesso.

FlowReferences.new costruisce un vero ReferenceAuditCollector per ciascun trace
nello scope ordinato; for_trace restituisce quello corretto e fallisce se manca.
into_audit unisce i controlli conservando l'ordine e registra i dataset una volta:
prima di unire verifica uguaglianza di policy/evidenze. Questo evita sia audit
reference duplicati sia attribuzioni di controlli al trace sbagliato. Dataset
Verified/Degraded/Unavailable restano distinti con hash attesi/calcolati.

uncertainty converte esaustivamente i motivi engine verso il DTO neutro;
UnsupportedPlan resta errore tecnico, mai un warning che mascheri una primitiva
assente. into_summary converte gli artifacts reali: abbina trace/PK e ne verifica
l'identità, calcola il numero di regole distinte, copia conteggi e cause bounded,
conserva JoinId e diagnostiche, serializza i binding effettivi. Non inventa
esiti né conserva righe originali. La completezza iniziale del DTO viene derivata
nuovamente e validata dal builder, non accettata sulla fiducia del converter.

FullServicePorts raggruppa sink/progress/cancellazione/timestamp/schema risolto
senza possedere nuovi adapter. ValidationService.validate_full verifica key,
opzioni e scope non vuoto; crea FlowTraceInput nativi, esegue binding e il
preflight ordinato condiviso; solo allora prepara reference e observer.
Costruisce builder Flow con scope ordinato e InputAuditSession autorevole.
Il FlowValidator reale esegue le due passate e riconcilia i contatori esistenti.

Gli eventi trace vengono mappati uno a uno su ProgressEvent; CrossStarted
introduce la fase CrossRecordValidation e CrossRecordsProcessed riporta un
conteggio effettivo. Completed del core non è RunCompleted applicativo:
quest'ultimo arriva soltanto dopo audit, validazione del summary e sink.on_summary.
Un fallimento di input, sink o summary non emette un esito completato.
Cancellazione prima/durante le passate o nel checkpoint finale resta Cancelled,
con audit parziale o EOF completo secondo le letture realmente avvenute.

La conversione From<FlowValidationError> conserva cause: Trace usa il mapping
precedente, errori report cross restano Report, errori builder Summary, problemi
di bundle diventano InvalidRequest tipizzati; binding/piani/cause residue
restano Flow. Errori tecnici non sono ValidationError normativi.

## Piccola evoluzione del core e observer

prepare_flow è la stessa funzione pura di preparazione sia per validate sia per
preflight_flow_inputs. Il port pubblico restituisce gli ID ordinati, non legge
file e non richiede uno snapshot fittizio per controllare il bundle prima della
preparazione reference. La semantica required_traces/parametri resta quella 13-06.

FlowExecutionPorts.cross_observer diventa Option del callback esterno: i consumer
reali passano Some. FullCrossObserver continua sempre a eseguire l'aggregazione
core e i contatori normativi; None evita soltanto un osservatore aggiuntivo.
L'applicazione consuma gli artifacts reali senza NoOp e senza una seconda copia
degli aggregati. Il borrow dell'observer è separato dal lifetime dello stato,
senza clone dello store né trasferimento degli adapter durante il run.

## CLI e pubblicazione

Il builder dei comandi condivide opzioni comuni di validate/validate-flow;
solo validate-flow richiede --trace-input ID=PATH ripetibile e accetta --parameter
ID=VALUE. parse costruisce FlowValidationInput + mode Full e una mappa di binding;
parametri duplicati/ID vuoti sono errori di sintassi, nessun last-write-wins.
Il parser LocalOnly mantiene trace/input espliciti e non accetta binding ignorati.

split_trace_input usa su Unix bytes/OsStringExt e su Windows UTF-16/OsStringExt,
separando esclusivamente il primo '='. L'ID deve essere Unicode valido e ID/path
non vuoti; il path resta nativo, può contenere altri '=' o componenti non UTF-8
su Unix. Nessuna conversione lossy né unsafe. L'input adapter conserva il proprio
contratto sul basename audit, senza rilassarlo per la nuova sintassi CLI.

check_outputs ora risolve tutti gli input Flow/Trace e vieta a ciascuna
destinazione finale/.part di coincidere con qualsiasi input o con i bundle.
Restano i controlli di collisione fra output e i guard atomici originali.
La CLI chiama validate_with_parameters del servizio, mai direttamente il motore.
ErrorSpool, JSONL, summary, XLSX e rollback conservano il percorso già verificato.
Un Full parziale ma completato può pubblicare report esplicitamente parziali;
Err/Cancelled/truncated conservano le regole precedenti, senza un nuovo XLSX
presentato come definitivo. L'atomicità rimane per file, non transazione multi-file.

## Report e test

write_summary serializza l'envelope e completeness soltanto quando esiste il
contratto Full, dopo summary.validate. L'ordine dei campi e delle mappe è stabile.
XLSX riusa già il JSON canonico: il foglio Riepilogo include automaticamente tutti
i campi Full senza un secondo mapper incompleto. HumanReadableSink mostra
completezza e avvertenza Partial anche senza reference, poi i conteggi aggregati
Unknown/N/A/skip per regola. Nessun warning incrementa errors_by_code.

I support full_schema/full_fixture contengono solo authoring e file sintetici;
la fixture authoring è condivisa con i test core 13-06 per evitare divergenze.
Test servizio: errori cross multipli/riclassificazione, mutazioni incoerenti del
summary rifiutate, copertura parziale con EOF hash completo, N/A distinto,
parametri senza default e stringa "001", cancellazione nelle due passate,
reference Verified/Degraded/Unavailable aggregate per trace, guasti input/sink.
Test processo: valide/invalide, missing/duplicate/unknown trace, namespace e
binding duplicati, protezione di tutti gli input final/.part, path Unicode/'='
e directory Unix non UTF-8, summary e XLSX parziali senza errori normativi fittizi.
Le regressioni LocalOnly, JSONL/golden, reference e report rimangono nel gate globale.

## Residui e limiti

T-P13-08 deve consegnare golden con regola cross-trace normativa reale G0 e
misura memoria sulle sole proiezioni, prima di dichiarare M6. Questa task prova
il servizio con dati sintetici: nessuna certificazione normativa CAD/SIOC.
PERSIST storico/sessioni obbligatorio e distinto dall'audit nei report;
P16 UI consumer dei DTO, nessuna implementazione anticipata. Metadata/temporal
lookup e primitive non supportate restano rifiutati dal compiler, non emulati.
Nessun max_errors introdotto o schema normativo riscritto. Gate locali/nativi
ed esiti effettivi sono registrati nel consuntivo prima della chiusura.

### Gate locale effettivo — 2026-10-25, data client

cargo fmt --all -- --check; cargo check --all-targets; cargo clippy --all-targets
-- -D warnings; cargo test (602 passati/70 suite, 0 fail/1 ignored); cargo build
--release; cargo check --all-targets --target x86_64-pc-windows-gnu; git diff
--check. Cargo con --offline --locked, CARGO_HOME isolata /tmp/lintra-cargo-home.
Tutti passati; baseline90 confrontata byte per byte con 6e4b411, invariata.
CI Linux/MSVC da avviare sul commit pubblicato, non inferita dal check GNU.
