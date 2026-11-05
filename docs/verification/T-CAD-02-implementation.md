# T-CAD-02 — binding namespace e profilo CAD

Stato: WAITING_GATE, implementazione e gate locali completati. CI nativa di
questa consegna ancora da qualificare sul commit pubblicato. Fonte vincolante:
CNF-CAD-NAMESPACE-BINDING-01 A estesa, 5 novembre 2026.

## Problema e comportamento

Le PK e il namespace dello storico non provavano che il campo organizzazione
appartenesse all'invio. Il formato 3 aggiunge il binding dichiarativo del campo
al parametro di namespace stringa. 501/501 passa; 502/501 emette il codice
configurato E_NAMESPACE_VALUE_MISMATCH. Un campo fuori dominio, mancante o non
valutabile conserva la violazione primaria o il conteggio skipped: nessun errore
derivato e nessuna attestazione positiva del namespace.

Full richiede tutti i componenti del namespace dichiarato validi prima di leggere input, anche per file vuoti
o cancellazione già richiesta. LocalOnly può ometterlo: il binding è skipped e
la completezza rimane Partial, anche su file vuoto. Presente-invalido non è
assenza. Non vengono aggiunti stati normativi, opzioni CLI/UI o scheduling.

Il bump di sviluppo a 0.1.1-rc.3 distingue la nuova capability dalla rc.2 già
pubblicata; nessuna release o tag rc.3 viene creato da questa task. Package root,
UI e i due lockfile sono sincronizzati, senza nuove dipendenze.

## Grammatica e compilazione

RawNamespaceBinding contiene trace/field/parameter/comparison/error_code e
message_template. Accetta soltanto logical_text_exact e proprietà note. La
lista opzionale RawPersistence.namespace_bindings è omessa quando assente:
i byte canonici dei formati precedenti rimangono invariati. Null esplicito non
sostituisce la lista. Vecchi formati rifiutano la nuova proprietà/capability,
anche lista vuota. Il meta-schema 3 deriva dalla grammatica 2 preservata; il
compiler verifica anche riferimenti, domini, unicità e capability.

schema::namespace_binding::compile risolve gli indici di trace/campo/parametro;
richiede un parametro stringa del namespace, un campo compatibile con testo
logico e un solo binding per campo. Date/Decimal non sono reinterpretati come
identificativi testuali. Codice diagnostico sintatticamente valido e messaggio
letterale non vuoto, privo di token non supportati. Nessuna DSL Parameter generale.
NamespaceBindingPlan è leggibile ma viene inserito nei compiled soltanto dal
compiler: il loop non cerca nomi raw e non modifica lo schema dopo il fingerprint.

compile_persistence_profile espone i binding anche per l'ispezione del profilo.
La compatibilità di gruppo confronta dichiarazione del binding e semantica del
valore logico/missing, anche quando il campo non fa parte della PK selezionata;
lo spostamento di offset non cambia tale semantica. Il binding entra nella
canonicalizzazione e nel fingerprint; i valori del run ne rimangono esclusi.

Il compiler ordinario ammette la policy limitata al binding/PK; audit storico
resta nel compilatore storico verificato, snapshot e temporali restano rifiutati
come esecuzione fino alle rispettive task. Nessuna capability viene rimossa per
ottenere artificialmente un compiled eseguibile.

Nel formato 3 il piano di bundle consente l'assenza di un target con policy
on_missing_trace Unknown, se il trace non è anche sorgente di una join richiesta.
Il motore di seconda passata esegue la policy Unknown già presente. Nei formati
1/2 resta il precedente requisito di presenza dei trace coinvolti nelle join.
Questo rende eseguibile il caso ratificato A assente senza dedurre orfani; non
abilita automaticamente la riconciliazione snapshot o l'assenza di ogni sorgente.

## Contesto e runtime

VerifiedRunContext possiede una mappa tipizzata immutabile, verificata sui
ParameterDefinition dello stesso compiled e sul costruttore PersistenceNamespace; non ha deserializzazione né setter.
Il costruttore rifiuta parametri sconosciuti, tipi errati, valori fuori dominio e,
in Full, binding obbligatori mancanti. Il servizio aggiunge l'entry point
validate_with_typed_parameters; gli adapter testuali esistenti rimangono espliciti.
L'identità del compiled impedisce di applicare il contesto a un altro artefatto
con una chiave nominale identica. Nessun valore è derivato dall'identità del DB.

TraceValidator.with_run_context inoltra il contesto allo stesso RecordValidator,
anche nel percorso reference. Il validator prepara soltanto binding e valori
risolti del trace; quando inizia un record congela il contesto e rifiuta un
cambio successivo. FieldPipeline esegue il confronto solo dopo Valid, sullo slice
logico secondo padding compilato: nessun trim aggiuntivo, case folding, numero→testo
o locale. 001 con stripping equivale al parametro testuale 1, non a 001.

Una violazione usa ErrorFactory/sink/counter esistenti. valid_records/invalid_records,
JSONL, ordine dei campi e budget max_errors seguono il percorso ordinario; errori
di sink/logica/counter rimangono tecnici e rendono terminale l'esecutore. Il binding
precede ProjectionCollector; nessuna terza passata e nessun controllo cross spostato
nel locale. La seconda passata e il runtime restano sequenziali (DEC-17).

Il risultato interno namespace_compatible distingue un record appartenente al
contesto da uno fuori namespace o non verificabile. Il trait privato del collector
riceve questo bit, senza cambiare LocalRecordValidationArtifacts pubblico. Una
riga non verificata conserva errori e conteggi locali, ma non fornisce proiezioni,
PK definitive del namespace o dati all'observer storico: il collector contabilizza
unprojectable/inconclusive e il contesto rimane Partial. Non viene pubblicato come
Complete. Record con altri errori locali indipendenti continuano a fornire evidenza
quando l'appartenenza è verificata; schemi senza binding conservano il comportamento
precedente. La regressione con parent502/namespace501/child501 prima soddisfaceva
erroneamente la join; ora produce Unknown, senza associazione fra organizzazioni.

Lo storico costruisce namespace e parametri dallo stesso request tipizzato e
profilo risolto prima della validazione. Full usa gli stessi valori per binding,
join e audit del summary. Il coordinatore di approvazione verifica l'uguaglianza
fra parametri Full e namespace dell'approvazione già prima di preparare il commit.
Un mismatch termina in ValidationOnly, senza token di preview o mutazione dello
storico. La prova usa il servizio storico reale e il repository memory di test;
non dichiara eseguito un workflow snapshot CAD completo su SQLite.

## Componente CAD e verifiche

schemas/cad/components/2024-06.namespace.1 conserva fonte/hash e dichiara
sending_ulss stringa501–509, A.1.1, namespace continuo senza anno/trimestre, periodo
trimestrale e tutte cinque PK ricevute. C/D mantengono il progressivo. Il componente
non è uno schema CAD completo/FROZEN. Il test del profilo usa campi sintetici per
verificare gli indici/PK/piano snapshot, senza attribuirgli controlli clinici o
reconciliation già eseguiti. Il limite calendario dell'anno è rappresentativo,
non una nuova soglia operativa degli invii.

namespace_bindings.rs esercita servizi reali LocalOnly/Full/Historical, summary e
JSONL reali, match/mismatch/dominio, tipi/assenza, padding, budget, cancellazione
dinamica, isolamento con stessa chiave evento fra due namespace, nessuna preview
su mismatch, approvazione/commit nel namespace corretto, contesto incompatibile,
parent assente/vuoto/presente, grammatiche/capability/riferimenti e fingerprint.
I test di formato non definito usano ora 4: 3 è l'estensione ratificata.

Gate locali conclusivi in target/cad02-gates-qualified/, ledger versionato in
T-CAD-02-local-gates.json e log in T-CAD-02/local-final/. Tutti dieci i comandi
exit0, hash prima/dopo invariati. Workspace default/allfeatures:915pass/0fail/6ignored
ciascuno; core senza bundle872pass/0fail/6ignored, tooling17pass/0fail.
CARGO_BUILD_JOBS=2 limita le build contemporanee, non il runtime del prodotto. CI Linux/MSVC necessaria prima di DONE.
Non eseguita una validazione esterna JSON Schema: jsonschema non disponibile nel
Python locale; nessuna dipendenza aggiunta per questo. Parsing raw, lint/compiler
e regressioni verificano il contratto implementato. Nessun benchmark nuovo.

Limitazioni preservate: CAD completo e snapshot operativi ancora nelle task
successive; nessun cambio a HMAC, storage, immutabilità, CAS o contratti dei dati
reali. Conflitto ordinamento diagnostiche separato ancora OPEN. docs/migration
preesistente esclusa. Errori di sviluppo e risultati preliminari non equivalgono
ai gate finali; nessuna soppressione di test o trasformazione di condizioni fallite.

## Diagnosi dei gate preliminari

Il primo gate workspace si è fermato sul test schema_linter che usava 3 come
formato indefinito; i casi analoghi registry sono stati aggiornati a4, mantenendo
le asserzioni di rifiuto. Log/ledger iniziali conservati, non trasformati in PASS.
La regressione di isolamento è stata prima eseguita con attesa satisfied=0 e ha
mostrato satisfied=1; stessa asserzione dopo il fix del collector passa. Non è
stato attenuato il test. I gate finali ripartono su sorgenti stabili dopo il fix.

Il secondo gate si è fermato sui tre golden SIOC cross-trace: byte differente
esclusivamente nella versione eseguibile rc.2→rc.3. Aggiornato soltanto quel
metadato con confronto strutturale che verifica uguaglianza di ogni altro campo;
JSONL, hash schema, esiti/conteggi/ordine e dati normativi restano invariati.
Il test continua a confrontare byte-per-byte senza normalizzare la versione.

## Affinamento finale del preflight, fonte namespace unica

Dopo i gate completi è stata provata una divergenza fra validità del parametro
stringa (dominio dello schema) e contratto strutturale del namespace persistente:
stringhe vuote/solo spazi/con controlli potevano essere accettate dal contesto.
La stessa attesa di rifiuto falliva prima del fix; log conservato. Ora
VerifiedRunContext usa PersistenceNamespace::new per ogni componente presente,
senza normalizzare il valore e senza accedere a un repository. Full richiede
tutti i componenti dichiarati, LocalOnly consente l'assenza e verifica quanto
fornito. Gli interi devono anche essere rappresentabili dal NamespaceValue
persistente. Schemi senza binding mantengono il precedente comportamento.

Il delta rispetto ai gate completi riguarda soltanto namespace_context.rs e
namespace_bindings.rs: nessun altro sorgente o golden modificato. Gate mirati
successivi: fmt/Clippy workspace allfeatures,54test schema/servizio standard e
17test binding allfeatures, tutti positivi,330hash stabili durante le verifiche.
Ledger T-CAD-02-refinement-gates.json. Non si attribuiscono retroattivamente i
915/915/872test al sorgente dopo questo affinamento: la CI finale sul commit
pubblicato deve qualificare l'intero insieme prima di DONE.

Tooling:17test ordinari positivi e prova aggiuntiva del bundle estratto con
eseguibile reale rc.3 positiva prima dell'affinamento di dominio. Restano ignorati
per invocazione automatica ordinaria helper di subprocess/trasporto cross-OS,
qualificazione manuale del workbook sorgente e memory probe XLSX; i job nativi
eseguono separatamente i controlli cross-OS previsti. Non è stata avviata una
nuova campagna prestazionale. Non ci sono gate locali falliti ancora irrisolti.
