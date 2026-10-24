# T-P13-06 — Preflight FlowValidator e due passate

2026-10-24. Nessun codice avviato: 13-05 commit 37dff96 pubblicato, CI
40089317003 avviata alle 11:17 UTC. Prima lettura Linux success/Windows in_progress;
una sola chiamata Actions residua, da usare dopo un intervallo ampio. Prerequisito
7-02 già verificato CI 39569282614. Gate locali 13-05: 575 test/67 suite, tutti
passati. Prompt 13-06, DET §24/§75–79/§83/§133–134, Piano §4/§9/§14,
ARCH §45, DET §166–174 e PERSISTENCE_CORE_SPEC letti. Nessun subagente/skill
Superpowers/background. Baseline/fonti/grammar non modificabili.

## Stato effettivo

TraceValidator ha un unico loop BOM/framing/audit/cancellazione e conta tramite
RecordValidator. Il loop oggi scarta gli artifacts dopo validate_local. La
projection esiste ma nessun record è conservato dal servizio LocalOnly. La
pass 2 13-05 è reale, senza accessi input, ma non aggiorna un summary Full né
compone progress/cancellazione. Application continua a rifiutare Full.

RecordValidator.validate_local conta skip cross per record, come DET §24.
Riutilizzarlo senza una fase Full esplicita produrrebbe skip permanenti anche
quando la pass 2 è eseguita. ValidationSummaryBuilder non offre ancora una
riclassificazione valid→invalid dopo una nuova violazione cross, né un guard
per impedire finalizzazione senza gli esiti Full. Occorre correggere entrambi
senza sottrarre/doppiare record processati o errori già emessi.

## Modifica minima proposta prima del codice

Conservare costruttori e signature LocalOnly. Aggiungere un percorso pass1 Full
esplicito che riusa il medesimo loop TraceValidator e comunica i veri artifacts
al collector di projection, senza contare skip per regole che devono ancora
eseguirsi. Nessun bool normativo mutabile nei record: posizione, identity e
un esito locale tipizzato bastano per ricordare se il record era già invalido.
Schema/dependency/invarianti verificati prima di aprire qualsiasi input.

Preflight bundle restituisce riferimenti read-only in ordine schema e rifiuta
trace ignoti, duplicati, percorsi vuoti e required_traces mancanti. In Full i
trace indispensabili sono quelli del piano compilato: DET §24/§74 e prompt
richiedono errore prima dell’input, non una conversione implicita in run parziale.
L’Unknown TraceInputUnavailable del resolver standalone rimane distinto;
nessuna modifica silenziosa di questa precedenza o del contratto storico CAD.

FlowValidator engine-owned apre ogni path una sola volta tramite InputAuditSession,
usa il framing comune e costruisce InMemoryContextStore per il singolo run.
Reference snapshot/observer reali quando richiesti, nessun NoOp o parametro
ignorato. Cancellation via callback neutra come TraceControl, niente import
application; controlli fra trace/record e prima della pass 2, progress ordinato.
La pass 2 deve poter arrestarsi fra record senza essere trasformata in Err.

SummaryBuilder: aggiungere un metodo checked per riclassificare un record
prima valido, senza alterare processed o contare due volte posizioni già
invalide. Un getter readonly dei contatori consente progress/test senza una
seconda fonte di conteggi. Il collector mantiene le sole posizioni necessarie
per riconciliare i risultati; nessun Vec di errori o record originali.

Per preservare scope e compatibilità, non anticipare qui un nuovo campo
obbligatorio in ValidationSummary o ValidationRequest. Il core restituisce
artifacts Full reali (copertura trace, esiti aggregati, completamento/annullamento),
e marca il builder come evidenza Full non ancora riportata. La finalizzazione
legacy deve rifiutare quel marker, così non può pubblicare Complete o perdere
Unknown/NotApplicable. Finalizzazione con envelope persistente e service Full
appartengono a 13-07, da proporre prima di alterare i contratti consolidati.
Nessun builder o metadato fake usato per far passare un test di summary.

## Copertura dell’indice — caso concreto da preservare

Un file target può essere letto fino a EOF ma contenere righe fuori lunghezza:
InputAudit Complete riguarda hash/EOF, non prova che tutte le righe siano
proiettabili. Se il projector scarta una riga, un lookup negativo nell’indice
restante non dimostra un parent assente. Non emettere E_RELATED_RECORD_MISSING
da una ricerca non esaustiva e non fare recupero arbitrario di byte parziali.

Proposta tecnica circoscritta: copertura trace tipizzata Complete/Partial/
Unavailable, distinta dalla presenza del file e dall’audit hash. Mantenere il
vecchio constructor projected per indici dichiarati completi e aggiungere un
percorso esplicito con coverage per il FlowValidator reale. In un indice
parziale, una prova positiva di esistenza/SetExists è ancora conclusiva;
una ricerca negativa o la richiesta di scalar unicità/consensus senza
copertura esaustiva è Unknown con motivo IncompleteRelatedContext e JoinId.
Non usare on_missing_record/trace Violation per chiamare certa quell’assenza.
Nuovo motivo è indisponibilità tecnica dei dati, non nuova norma hardcoded.
L’aggiunta enum va verificata su tutti i consumer; niente cambio del bool DSL.

## Verifiche previste

Preflight prima di I/O e ordine schema con input riordinati, trace ignoto/
duplicato/mancante e path Unicode; una sola lettura/hash originale per input;
projection/minima memoria; valid→invalid una sola volta con più errori cross,
record già invalido non ricontato; nessuno skip LocalOnly falso in Full;
record target fuori lunghezza senza falso errore parent missing; positivo su
indice parziale ancora utilizzabile; cancellazione prima/durante pass1 e
prima/durante pass2, hash incompleto quando EOF non osservato; progress stabile;
reference Verified/Degraded/Unavailable con observer reale e indipendenti
normali; errori tecnici terminali e cause preservate; builder legacy non può
finalizzare ignorando artifacts Full. Gate locali e CI Linux/MSVC.

## Ledger

13-07 integra ValidationService Full, parametri runtime per run senza default,
RunMetadata/ValidationSummary con evidenze Full persistenti e completezza;
report atomici mantengono le regole su Err/Cancelled. Eventuale evoluzione
pubblica deve essere esplicitata prima del codice, mantenendo API LocalOnly e
golden tecnici precedenti. PERSIST e storico/sessioni restano obblighi distinti.
