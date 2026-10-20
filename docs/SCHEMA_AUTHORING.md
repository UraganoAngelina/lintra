# Schema authoring Lintra — contratto di formato 1

Stato: **APPROVATO / T-P0-07 DONE**, 2026-10-19.
L’autore ha approvato il contratto completo del commit 72f9f5e: formato 1,
JSON Schema, esempi, mapping, policy persistenti ed estensioni compiled.
I riferimenti a «proposta» nel seguito descrivono il testo ora approvato.
Nessun compiler Rust, storage o validatore normativo è implementato qui.

## Fonti, autorità e perimetro

Riferimenti: [ARCH](ARCHITECTURE.md) §7–12, §21–22, §39C.2, §45;
[DET](DETAILED_DESIGN.md) §8–21, §35–37, §66–72, §91, §106, §155,
Addendum A.1 e §166–174; [Piano](MASTER_IMPLEMENTATION_PLAN.md) §4,
§9/P0, §14; [persistenza core](PERSISTENCE_CORE_SPEC.md).
T-P0-03/04/05 sono stati verificati contro le fonti con i checker documentali.
Le [decisioni umane](normative/DECISIONS.md) e i dossier
[SIOC](normative/sioc/rules-and-joins.md) / [CAD](normative/cad/rules-and-joins.md)
restano la fonte delle regole dei rispettivi profili.

**CNF-LINTRA-DSL-01 risolta dall’autore:** approvata la proposta A limitata
alle operazioni richieste: substring, età compiuta, differenza fra anni.
Il rinvio Piano §13.5/DET §135 è derogato solo per queste necessità.
Nessuna funzione arbitraria o implementazione anticipata; le task future
mantengono prerequisiti, quality gate e test. I documenti sorgente restano
inalterati; la decisione e il necessario allineamento delle task sono nel
registro, non una modifica silenziosa al piano.

Gli artefatti normativi ricevuti sono materiale di import, non esempi già
conformi a questo nuovo formato. Il flow aggregato SIOC è autorevole per
quel profilo; la discrepanza 2026-04-20 del flow e 2025-12-16 dei trace si
conserva nella provenienza. Non si sceglie implicitamente una versione.

## Artefatti e grammatica autoritativa della proposta

- [flow.schema.json](schema-authoring/flow.schema.json): JSON Schema 2020-12,
  oggetti operativi chiusi e operatori enumerati; `$defs` contiene ogni nodo.
- [manifest.schema.json](schema-authoring/manifest.schema.json): manifest di bundle.
- [lowering.json](schema-authoring/lowering.json): mapping raw→compiled e import legacy.
- [Indice ed esempi completi](schema-authoring/README.md): tre flow sintetici,
  dataset dimostrativo e manifest con hash reali.

Per dettagli di proprietà obbligatorie, union e tipi fa fede il JSON Schema
collegato. Campi sconosciuti e operatori arbitrari sono invalidi. Solo
`source` e `regulatory_metadata` sono oggetti metadata aperti: il runtime
non esegue né interpreta come istruzioni il loro contenuto.
JSON Schema controlla la grammatica; riferimenti, byte codificati,
compatibilità e semantica richiedono i controlli del compiler elencati sotto.

## Identità del flow e versioni

Ogni flow dichiara obbligatoriamente schema_format_version, flow_id,
schema_version, regulatory_version, source, encoding, required_capabilities,
parameters, reference_data, traces e persistence_policy (null se non attiva).

| Valore | Significato |
|---|---|
| schema_format_version = 1 | Intero u32: versione del contratto dati motore |
| schema_version | Revisione operativa del profilo; mappata a SchemaKey.version |
| regulatory_version | Versione della fonte normativa; non sostituisce la revisione operativa |
| bundle_version | Revisione dell’insieme di file distribuito |
| canonical_schema_version legacy | Metadato DSL della fonte ricevuta, da conservare nella provenienza |

DEC-AUTHORING-FORMAT-01 propone il primo formato numerico 1: nessuna
conversione implicita da "1.0.0", nessun default se manca il formato.
La selezione usa flow_id + schema_version, poi trace id; niente latest.
Qualsiasi cambiamento semantico produce una nuova revisione del profilo.
Un formato strutturale incompatibile richiede un nuovo numero di formato;
un operatore nuovo richiede una capability versionata riconosciuta.
La prima release compatibile definirà la versione minima del binario;
non si inventa oggi un numero di release non implementata.

### Mappa formato / eseguibile CNF-FORMAT-01 — approvata

L’autore ha approvato «Approvo la mappa con baseline di sviluppo 0.1.0».
La [mappa ufficiale](schema-authoring/FORMAT_COMPATIBILITY.md) assegna al
formato 1 la baseline minima di sviluppo 0.1.0, compiler verificato b658e05;
non la presenta come release pubblicata. Formato 0 e numeri 2..=u32::MAX
non hanno una specifica o un eseguibile compatibile dichiarato: la versione
minima non è applicabile a un formato non definito. Input assenti/non conformi
sono errori di caricamento senza default. Ogni futuro formato avrà un’entry
con versione minima concreta quando viene definito, senza deduzioni da semver.
Questa mappa non dichiara implementata local.v1 completa e non modifica i JSON.

## Import, mapping e preservazione della fonte

L’import è esplicito e revisionato, mai un fallback durante la validazione:

1. Conservare artefatto originale/hash e metadati di fonte, incluso source_id.
2. Produrre una nuova revisione con formato numerico, binding e policy esplicite.
3. Normalizzare gli alias confermati: AN e A SIOC→alphanumeric; N→numeric;
   D/Data→date; T CAD→alphanumeric con constraint posizionale. Gli altri
   alias richiedono mapping dichiarato, non un’euristica sul nome.
4. Trasformare i costrutti secondo lowering.json; conservare le clausole
   non eseguibili deliberate con decision_reference e motivazione.
5. Trasferire le decisioni umane correnti senza cambiare gli originali.
6. Lint, revisione del diff e fixture normative nelle task successive.

La rinomina trace_id→id/trace_name→name è un mapping esplicito, non una
lettura ambigua di due proprietà concorrenti. Nessuna equivalenza automatica
fra regole di versioni diverse o fra PK diverse. T-P0-08 produrrà i profili
canonici solo dopo l’accettazione di questo contratto e i propri prerequisiti.

Le fonti legacy hanno 19 kind di constraint e 14 kind di assertion: tutti
sono tracciati in lowering.json. `structure_only` significa controlli fisici
senza ulteriore semantica: si conserva in metadata, non si crea un executor
fittizio. `_if_present` passa dallo stage comune missing/presence, senza
saltare il controllo di obbligatorietà. Le formule named sono risolte e
inlined con rilevamento dei cicli.

## Trace, campi e range fisici

Un trace contiene id, name, record_length, fields ordinati, record_key,
joins, identity, named_conditions, derived_values, repeated_groups, rules
ordinate e documented_non_executable_rules. Una lista vuota è esplicita.
Identificativi non vuoti, unici nel rispettivo scope, case-sensitive;
il compiler rifiuta spazi esterni e identificativi ambigui, non li corregge.

Ogni campo ha id, name, physical, missing_value, presence e constraints;
regulatory_metadata è opzionale. Posizioni start one-based e length in byte,
interi positivi entro u32. Conversione unica in [start-1, start-1+length),
con overflow, gap, overlap e record_length verificati prima dell’esecuzione.
L’ordine dei campi non è derivato da HashMap.

Tipi ammessi in formato 1:

| Tipo | Lessico e significato |
|---|---|
| numeric | Cifre ASCII 0–9 del logical value, almeno una; range e confronti numerici su interi decimali senza floating point. Nessun segno o separatore implicito |
| alphanumeric | Testo compatibile con encoding/profilo esplicito; nessun dominio automatico A–Z/0–9 dedotto dal nome AN |
| date | Esattamente 8 byte ASCII DDMMYYYY; giorno/mese/anno calendario validi, anni 0001–9999; niente timezone o ordinamento lessicografico delle date |

Alphabetic e Decimal esistono nel design concettuale ma non sono richiesti
dai campi ricevuti: non sono ammessi da questo formato e non si inventano
separatore decimale, segno, scala o repertorio alfabetico. La loro introduzione
richiederà evidenza, aggiornamento del contratto ed esempi dedicati.

`encoding.mode = runtime_required`, posizione byte. Nessuna code page di
sistema. Prima di compile, il servizio lega il profilo di encoding esplicito
fornito dall’adapter al RawEncodingSpec in memoria, conservando configurazione
originale/hash e scelta nei metadati del run. SchemaCompiler::compile(raw)
non cambia signature: deve ricevere encoding risolto per verificare literal
e range e costruire CompiledFlowSchema immutabile. Se manca il binding, il
preflight fallisce; nessun compiled schema fidato con literal non verificati.
DEC-13 resta aperta sui profili concreti dei dati regionali, non sulla scelta
di usare un encoding esplicito. Nessun default viene dedotto da questi esempi.

`lexical.kind = encoded_text` verifica rappresentazione valida nell’encoding
selezionato. `byte_set` dichiara allowed_bytes e lowercase_bytes: insiemi di
byte 0–255, senza locale; lowercase_bytes dev’essere sottoinsieme dei byte
ammessi. La codifica resta comunque da verificare. `no_lowercase` richiede
una classe di minuscole esplicita; encoded_text da solo non la fornisce e
non autorizza un case-folding scelto dal motore.

## Raw, logical, padding e assenza

Raw è lo slice esatto. Il logical si ricava soltanto dall’allineamento
esplicito. Decisione successiva dell'autore per T-P6-03 / DET §44: il controllo
fisico del padding è fuori ambito per il formato 1, senza residuo aperto e
senza E_ALIGNMENT; gli errori sui valori emergono dagli stage successivi.
Larghezza record, presenza e chiavi rimangono controlli distinti. Uno spazio
iniziale nel testo libero resta contenuto lecito, non padding implicito.
Le policy di estrazione sono:

- right_pad: fill_byte e content_policy strip_trailing_fill o preserve_width;
- left_pad: fill_byte e content_policy strip_leading_fill o preserve_width;
- exact: nessuna rimozione di padding.

Nessun trim globale, rimozione di zeri o inferenza dal tipo. Se gli zeri
sono significativi nei codici numerici, preserve_width conserva "01";
un confronto numerico può leggere 01 come intero 1 senza riscrivere i byte
usati dai confronti di dominio. Se stripping produce valore vuoto, non si
reinventa uno zero. L’import deve scegliere la policy dal significato
confermato del campo; non basta applicare una rimozione indiscriminata.

Missing si valuta sul raw prima delle constraint:

| kind | Semantica |
|---|---|
| spaces | Esattamente width spazi ASCII |
| exact | Byte codificati del raw literal, di larghezza esatta |
| any_of | Una delle rappresentazioni raw esplicite, tutte di larghezza esatta |
| never_missing | Nessuna rappresentazione è fisicamente mancante |

Un literal JSON si codifica nel profilo risolto, non si misura in code point.
I literal raw devono avere la larghezza del campo; domini logical non vengono
paddati di nascosto. Nessun default AllZeros: `000` CAD è semantico valido
per B1.2.12, mentre tre spazi sono mancanti come deciso dall’autore.

## Presenza e DSL delle condizioni

Presenza normalizzata: base required/optional, required_if[], forbidden_if[].
Le condizioni si valutano su contesto disponibile indipendentemente
all’ordine dei campi. OBB→required e FAC→optional sono import dichiarati;
NBB conserva la base e le condizioni approvate del campo, non diventa
un required globale. Conflitti richiesto/vietato sono visibili; Unknown
propaga secondo DET §19–22, non false e non validazione superata.

Expr è la union chiusa di eq, ne, in, not_in, is_present, is_missing, and,
or, not, condition, const, derived_eq, derived_compare. and/or sono liste non vuote; const
rappresenta true/false senza sfruttare una convenzione implicita sulle liste
vuote. Eq/Ne/In/NotIn confrontano literal string su byte logical codificati.
Non si trasforma un campo mancante in stringa vuota per farlo confrontare.

Ref locale: `{ "field": "ID" }`; esterna: trace + field + join, come negli
esempi completi. Le esterne risolvono tramite join dichiarato; un set non
ridotto non può essere usato arbitrariamente come singolo record.
DerivedCompare confronta un derivato con un literal tipizzato, necessario
per soglie di presenza o rami per anno. DerivedEq confronta con il tipo del piano derivato: byte oppure intero
(parsing numerico del campo quando il piano produce intero).

Il compiler risolve campi, condizioni e dipendenze, rileva cicli, valida
compatibilità dei tipi e produce field_index/JoinId. L’evaluator non esegue
nomi di funzioni o codice proveniente da stringhe. FALSE AND UNKNOWN=FALSE,
TRUE AND UNKNOWN=UNKNOWN, TRUE OR UNKNOWN=TRUE, FALSE OR UNKNOWN=UNKNOWN,
NOT UNKNOWN=UNKNOWN; le altre combinazioni sono quelle Boolean ordinarie.

## Valori derivati approvati e casi limite

| kind | Parametri e risultato |
|---|---|
| substring | source_ref, target raw/logical, start one-based e length in byte; risultato byte string, range controllato, niente slice fuori limite |
| age_in_completed_years | birth_date_ref, reference_date_ref, DDMMYYYY, leap_day_anniversary esplicita feb28 oppure mar01; intero di anni compiuti |
| calendar_year_delta | Stessi riferimenti data e formato; year(reference)−year(birth), senza confronto del compleanno |

Nessun piano muta i campi sorgente. Input mancante/invalido o join ambiguo
produce Unknown con motivo; mai data fittizia. La scelta per il compleanno
29 febbraio nei non bisestili è un parametro obbligatorio, senza default
normativo: l’esempio usa mar01 soltanto a titolo sintetico; il profilo reale
richiede conferma prima del freeze. La data resta gregoriana valida.
Età negativa non fa wrap di un unsigned; le constraint del profilo decidono
il suo significato. SIOC usa anni compiuti, CAD differenza degli anni.

La deroga ID legacy si esprime con condizioni sui derivati substring
dell’anno di inizio e fine, confrontati con "2025"/"2026"; l’esempio
legacy_example mostra la forma del guard, senza assegnare norme al flow
sintetico. Nell’import reale i vincoli otto cifre/prefisso/progressivo
devono diventare assertion condizionali nel ramo pertinente, lasciando
comuni presenza, caratteri ammessi, non-tutti-zeri e padding. Un contesto
ignoto non sceglie automaticamente il ramo più restrittivo.

## Constraint e assertion

Ogni constraint è tipizzata; il compiler rifiuta applicazioni incompatibili.
I nodi sono esemplificati nel flow local e definiti in $defs/constraint:

| kind | Controllo |
|---|---|
| allowed_values | Appartenenza a valori raw oppure logical espliciti |
| numeric_range | Limiti interi inclusivi, min≤max |
| regex | Pattern regolare su raw/logical, match required/forbidden; niente backreference, lookaround o scripting; pattern non supportato è SchemaIssue |
| reference_data | Dataset versionati dichiarati, fallback_values espliciti, eventuali metadata_bindings tipizzati |
| logical_length | Numero esatto di byte logical |
| substring_not_equals | Slice byte valido e target esplicito, confronto con literal |
| no_lowercase | Nessun byte della classe minuscole esplicita del profilo |
| date | DDMMYYYY e validità calendario; nessuna disattivazione implicita |
| composite_segments | Segmenti one-based con equals o lookup dichiarato |
| positional_bitset | Posizioni start/length e valori di un byte; when/otherwise espliciti, inclusi spazi inutilizzati o fissi |
| non_empty | Logical non vuoto dopo padding esplicito |

Positional_bitset lavora sulle posizioni raw, non su una stringa con coda
rimossa. Il compiler richiede copertura/non sovrapposizione entro il campo
per la struttura posizionale dichiarata; le classi di valori sono encodeable
in un byte. Il T70 CAD resta un constraint, non un tipo nuovo.

Le rule hanno id, when, assertions ordinate, error_code simbolico e
message_template. Le assertion chiuse sono constraint su Ref, field_present,
field_compare, equals_concat, condition_true, derived_compare, record_exists,
set_exists, substring_equals_derived_year. Operandi dei confronti sono ref
o literal, con value_type bytes/integer/date esplicito. Confronti data
usano date valide; concat preserva l’ordine; year_digits ammette 2 o 4.
SetExists applica il predicate a ciascun record del target del join:
i ref locali del predicate appartengono al trace target, non al sorgente.

Gli esempi legacy raw_equals/logical_equals diventano allowed_values singleton
col target corretto; composite diventa composite_segments; not_allowed_values
diventa condition_true con not(in(...)). Nessuna regex costruita interpolando
valori normativi. Le formule non supportate sono errori di schema.

## Identità, PK, gruppi e diagnostica

Record_key dichiara fields ordinati, uniqueness_scope trace ed error_code;
non è l’identità diagnostica o il solo progressivo. Identity è una lista di
binding target/field: sioc_record_id, fiscal_code, episode_id o technical;
solo technical richiede technical_name. Estrarre dopo controllo lunghezza,
senza dedurre che un campo CF/altro identificativo contenga sempre un CF.
I dati persistenti non coincidono con il contenuto diagnostico in chiaro.

Repeated_groups dichiara fields, required_slots/optional_slots, contiguity,
unique_non_missing e opzionale unused_slot_value. I ref sono locali;
non si crea una join fra slot. Nessuna contiguità/unicità implicita.

Errori simbolici stabili E_* (DET §51), mai E010/E003/E012 in output.
Le numerazioni ricevute restano metadata; mapping versionato e centralizzato
in T-P2-02/T-P5: ad esempio assenza correlata→E_RELATED_RECORD_MISSING,
constraint cross-field→E_CROSS_FIELD. Non si cambia la condizione che causa
la violazione per rinominare un codice. I codici specifici approvati del
profilo sono registrati come RegulatoryRule. Gli E_EXAMPLE_* degli esempi
non sono codici normativi assegnati a SIOC/CAD.

I template sono testo con placeholder consentiti e verificati dal compiler,
non espressioni. Ordine errori trace→riga→campo→stage→regola secondo il
compiled plan; no PHI in log, path o file di fixture.

## Join scalari, insiemi e policy

Join dichiara id, target_trace, source_fields e target_fields ordinate,
cardinality, scope, namespace_parameters, on_missing_record e on_missing_trace.
Tuple collision-safe, non concat con separatore ambiguo. Scope esplicito
current_submission oppure history_including_current; parametri di namespace
richiesti e verificati prima di qualsiasi join/storico.

| cardinality | Insieme di riscontri ammesso |
|---|---|
| zero_or_one | 0 o 1 |
| exactly_one | 1 |
| one_or_many | Almeno 1, nessun first() |
| zero_or_many | Anche insieme vuoto lecito: estensione contrattuale richiesta da CAD |

Le cardinalità minime e le missing policy devono essere coerenti: min=1
con nessun record disponibile richiede violation, oppure unknown se universo
non disponibile. zero_or_one può dichiarare not_applicable. Il massimo è
verificato prima di accedere a un valore scalare. Duplicati non sono eliminati
silenziosamente per far rispettare la cardinalità.

Ogni policy contiene policy, diagnostic.code e message_template; error_code
è obbligatorio solo per violation e vietato negli altri casi. Policy ammesse:
violation (violazione accertata), not_applicable (regola fuori ambito,
spiegazione esplicita), unknown (non valutabile, motivo conservato).
Un input non fornito è distinto da un insieme verificato senza riscontri.

**Decisione SIOC vincolante:** anagrafica presente ma ricovero correlato
assente → NotApplicable, senza errore di parent mancante, con testo esplicito
«Anagrafica presente; dati del ricovero correlato assenti». Non diventa
Satisfied e non si applica a target multiplo o data invalida di target presente.
L’esito deve conservare uno stato di non applicabilità con motivo e JoinId:
il futuro RuleValidationResult/diagnostic outcome va allineato a questa
semantica senza riusare Satisfied o Unknown per nasconderla.

Riduzione facoltativa consensus: include_if sul target, fields come tupla,
una sola tupla distinta→proiezione scalare; più tuple→Unknown con diagnostica;
nessuna tupla→policy esplicita del join. Non prendere una data da un record e
il motivo da un altro. Questo chiude la proposta DEC-05-CAD-EVENT-CONTEXT
nel contratto da revisionare, senza assegnare nuove violazioni normative.

## Reference bindings e capability

Ogni reference binding ha id locale, dataset_id/version, SHA-256 dei byte,
path relativo al bundle e metadata_attributes dichiarati. Tutte le dependency
usate devono esistere; mancanza/corruzione/incompatibilità è errore di
preparazione, non valore paziente fuori dominio. Lookup su chiave codificata
senza rimozione di punti/zeri non dichiarata. MetadataBindings specifica
attributo, operand (record ref oppure derivato locale), operatore e tipo.
Il valore del record/derivato è l’operando sinistro; l’attributo del dataset
è il destro. lt/le/gt/ge esprimono soglie, in/not_in richiedono un insieme
compatibile; tipi o attributi incompatibili sono errori di preparazione. Un generico sex:true/age:true legacy
non basta: occorrono binding e semantica della tabella, senza inventarli.
Il formato dei provider e le versioni reali restano in REFERENCE_DATA_SPEC
quando i dati saranno disponibili; il dataset d’esempio non è quel contratto.

Le capability sono identificatori di operazioni chiuse, non plugin di codice:

| Capability | Piano richiesto |
|---|---|
| local.v1 | Tipi fisici, padding/missing, scalar constraint e chiavi locali |
| expressions.v1 | DSL/presenza/rule e assertion locali |
| joins.v1 | Join espliciti, cardinalità e missing policy |
| joins.sets.v1 / joins.consensus.v1 | Insiemi / riduzione deterministica |
| derived.substring.v1 | Sottostringa byte |
| derived.age_in_completed_years.v1 | Età compiuta |
| derived.calendar_year_delta.v1 | Differenza anni |
| reference_metadata.v1 | Lookup con binding metadata |
| persistence.audit.v1 / persistence.snapshot.v1 | Audit / snapshot e invarianti storici |
| history.existence.v1 | Esistenza nell’insieme storico incluso invio corrente |
| temporal.intervals.v1 / temporal.after_date.v1 / temporal.inactivity.v1 | Piani temporali definiti sotto |

Il compiler inferisce anche le capability dall’AST e verifica che siano
dichiarate. Capability sconosciuta, mancante o non implementata blocca la
compilazione; un motore non pubblicizza una capability implementata solo
parzialmente. Non è sufficiente che il numero di formato sia riconosciuto.
Nessuna di queste capability è dichiarata implementata nello scaffold attuale.

### Deroga di sviluppo CNF-COMPILER-01 — approvata

L’autore ha approvato la proposta A del [dossier T-P3-05](verification/T-P3-05-preflight.md):
«Approvo A: compiler incrementale con rifiuto esplicito dei costrutti esclusi».
Durante la consegna incrementale, il compiler può accettare local.v1 solo
per schemi i cui costrutti presenti siano tutti supportati e verificati;
la dichiarazione local.v1 rimane obbligatoria. Ogni costrutto escluso e ogni
altra capability non supportata causa Error, senza no-op o cancellazioni.
Questa deroga riguarda il gate del compiler, non autorizza il motore a
pubblicizzare local.v1 completa. Non cambia formato JSON, baseline normative,
encoding regionale o la signature compile(raw). Il binding di encoding
previsto sopra è una proprietà solo Rust in memoria, esclusa dal JSON.
Lo scope esatto di ogni consegna rimane nel relativo report di verifica.

## Persistenza e piani temporali

Persistence_policy null significa nessuna policy persistente attiva; non
attivarne una perché un backend è disponibile. Altrimenti contiene:

- namespace_parameters, tutti obbligatori; version_compatibility same_schema_version
  oppure declared_group con elenco di versioni e id gruppo espliciti;
- run_uniqueness e cross_run_audit, ciascuna con trace e record_key;
- history_identity_invariants con progressive_fields/event_fields e policy
  di riattivazione/errore di riassegnazione;
- snapshot null oppure scope, parametri anno/trimestre, approvazione immutabile,
  require_explicit_approval, commit atomic_idempotent e missing_trace_policy;
- temporal_rules esplicite e storage_protection external_secure_configuration_required.

Una compatibility group non converte automaticamente PK diverse: il compiler
verifica compatibilità delle proiezioni e blocca equivalenze non dichiarabili.
Nessuna versione condivisa dedotta dalla sola identità flow_id. Scope ULSS,
tracciato e chiavi pseudonimizzate restano separati; chiavi/credenziali/driver
si configurano fuori dallo schema. Minime proiezioni protette, niente CF o
righe originali in chiaro nello storage default.

Snapshot: trace assente conserva lo storico; trace presente è una vista
completa dichiarata, anche se vuota, e richiede il percorso di consenso
previsto. Warning più PROCEED/ABORT prima della revisione storica quando
mancano trace. Preview non mutante, approvazione esplicita, commit atomico
idempotente con controllo generazione; sessione approvata immutabile. Non si
implementano qui storage, cifratura o recovery.

Tre piani temporali tipizzati, tutti esemplificati in history-flow:

| kind | Semantica |
|---|---|
| interval_exclusion | owner point_field contro intervalli dal join target; inizio/fine aperti, fine mancante estesa all’orizzonte; when locale, input assente con policy esplicita |
| after_date_restriction | Data del record strettamente successiva alla conclusione risolta; target_when sul target, forbidden_values sul campo sorgente; confini stessa data tollerati |
| inactivity_with_pauses | Quattro insiemi dichiarati (valutazioni, sospensioni, accessi, prestazioni), correlazione access_fields/provision_fields senza progressivi, ultima attività abbinata o inizio evento, tre mesi/parametro calendar_months, unione pause e proroga a punto fisso |

Per inattività: addition mesi con clamp al fine mese, giorni strettamente
interni dell’unione delle sospensioni, sospensione aperta fino a fine trimestre,
valutazione a fine trimestre. Evento già concluso legittimamente non richiede
una seconda chiusura; input indispensabile assente→Unknown. I mapping CAD
sono configurazione esterna; nessun nome campo o codice CAD nel core.
Le tuple di inizio/conclusione confliggenti generano indeterminatezza con
diagnostica, non selezione first(); il compiler richiede il piano di consenso
necessario, anche quando più valutazioni sono disponibili.

L’iniziale CAD è set_exists su B1 nello storico dell’evento incluso corrente:
almeno una 2.8=1; non esattamente una né una per trimestre. Zero input non
è zero record accertati. Le PK C/D mantengono i progressivi; la correlazione
tra accesso e prestazione li esclude. Le sette estensioni runtime CAD ricevute
restano requisiti da compilare/testare, non policy disabilitate. La frase
legacy «outside generic core» è superata da ADR-PERSIST-001.

## Controlli richiesti al futuro compiler

JSON Schema non prova da solo la compilabilità. Il compiler deve:

1. Verificare formato, versioni, id, capability e binding di encoding/parametri.
2. Verificare layout byte, overflow, gap/overlap, literal e tipo/constraint.
3. Risolvere ogni ref locale/esterna, dataset, condizione, derivato, PK e identity;
   controllare unicità, cicli, enum chiusi e tipi degli operatori.
4. Verificare chiavi source/target di pari arità, namespace/scope, cardinalità,
   missing policy coerenti e compatibilità di ogni riduzione/scalar ref.
5. Verificare min≤max, pattern supportati, substring e bitset nei range,
   classi lessicali, width raw e lunghezze codificate dei valori posizionali.
6. Verificare parametri storici, gruppi di compatibilità e tutti i field role
   dei piani temporali; nessun piano obbligatorio non supportato viene ignorato.
7. Classificare regole locali/cross-record, risolvere i piani compiled tipizzati,
   derivare proiezioni minime ed ordine deterministico, congelare il risultato.

La verifica di questi casi appartiene a P3/P10/P13/PERSIST/CAD; il checker
Python documentale non è un compiler alternativo. Esempi: cicli, ref inesistenti,
regex non valida, PK diverse in gruppo, valore raw non codificabile, B1
confliggenti e target scalare da set non ridotto restano test runtime futuri.

## MANIFEST.json, hash e bundle

Formato proposto: bundle_version, schema_format_version numerico e mappa
files da path relativo portabile a SHA-256 dei byte esatti. Niente assoluti,
drive/UNC, backslash, caratteri di controllo, segmenti . o .. o componenti
vuote. Il preflight verifica anche l’ammissibilità del path sul filesystem
target e non effettua espansioni di home o variabili d’ambiente. Duplicate chiavi
JSON, anche se un parser ordinario le sovrascriverebbe, devono essere rifiutate
in lettura. Hash file distinti dal futuro schema_hash canonico T-P14-03.
Il manifest non include se stesso; SHA256SUMS/archivio/release sono P15.

Cartella esterna selezionata usa esclusivamente quel bundle: schemi,
reference-data e dipendenze devono essere presenti e verificati; niente
fallback a singolo file. Embedded usa lo stesso manifest. Runtime registra
bundle_source, bundle_version, schema_hash e dataset/versioni; non usa path
della macchina di build o della cwd come default.
L’esempio contiene hash reali dei file sintetici ma non è una release né
uno snapshot normativo approvato per la produzione.

## Adeguamenti delle task e accettazione

Proposta di allineamento per le task future, senza cambiare i documenti sorgente:
P3 raw/compiled/capabilities e piani tipizzati; P10 i tre derivati approvati e
lowering di const/derived_eq/derived_compare; P13 cardinalità ZeroOrMany, consenso e policy
NotApplicable con diagnostica; PERSIST core generico e profili CAD dopo i gate.
Riferimenti e payload richiesti sono in lowering.json/compiled_contracts.
Sono estensioni esplicite dei placeholder DET §16–18/§72, ratificate nella
revisione umana di T-P0-07; non executor Builtin con payload arbitrario.

Verifica: [check_authoring.py](verification/check_authoring.py) usa un validatore
JSON Schema reale per meta-schemi, esempi e casi strutturali negativi; controlla
hash, copertura di ogni famiglia, mapping delle fonti e riferimenti degli esempi.
[Report T-P0-07](verification/T-P0-07.md) registra comandi e limiti.
Non dimostra che i profili normativi legacy siano compilati o validi in Rust.

- [x] Deroga circoscritta ai derivati approvata e registrata.
- [x] JSON Schema flow/manifest, tre flow completi e manifest dimostrativo.
- [x] DSL/constraint, identità, presence, join, reference, versioning e persistenza definiti.
- [x] Mapping delle fonti e piani raw→compiled, scope delle task future esplicito.
- [x] Revisione umana finale ricevuta: «Approvo il contratto T-P0-07».

T-P0-07 è DONE dopo la revisione umana del contratto completo. È autorizzata
la verifica della task successiva; G0 e l’eseguibilità dei profili restano
gate separati. Le boundary normative indicate non sono colmate per deduzione.
