# T-P10-05 — Regole derivate e diagnostica delle sorgenti reali

**Stato corrente 2026-10-23: DONE.** CI 39919164236, workflow e job
Linux/Windows MSVC completed/success (gate cumulativo per 10-04, spiegato sotto).
P10 estesa ora chiusa con 458 test e CI finale 39920597392. I gate temporaneamente
chiusi o pendenti nel registro seguente descrivono questa specifica consegna;
compiler/presenza/regole pubblici sono stati integrati nelle task successive.

## Registro della consegna e delle verifiche

2026-10-23. Residuo P10 già ratificato T-P0-07/lowering.json; nessuna nuova
norma o formato flow. Prerequisiti 10-04 e 5-02 verificati: 10-04 nel gate
cumulativo 39917228132 su cb0a044, entrambi i job Linux/Windows success;
5-02 già verificata 39ed9b2/CI 39390492669. Prompt atomico/Piano §4/§14,
DET §47/103/140, ARCH §24.7, authoring derivati/assertion/template applicati.
P11-03/CNF-REFERENCE-FORMAT-01 è un blocco distinto, senza dipendenza P10.

## Compiler: piani effettivi, mai payload scartati

compiler::Checks::record conserva la validazione di tutti i derivati, compresi
quelli inutilizzati; rimuove il divieto generico della raccolta ora eseguibile.
compile_record memorizza realmente compile_derived_values(raw) in RecordParts:
stesso ordine canonico BTreeMap già usato per risolvere gli indici negli Expr.
Non conserva vec o risultati vuoti fittizi. Tutti gli altri gate futuri su
join/repeated_groups/reference/persistenza rimangono espliciti.

La allowlist aggiunge le sole tre capability derivate effettivamente eseguite.
Il lowering di required_capabilities conserva ora ogni capability supportata
richiesta, nello stesso ordine: LocalV1, ExpressionsV1 e le tre Derived*V1.
La review ha rilevato il precedente vec fisso LocalV1, che perdeva perfino
ExpressionsV1 già ammessa in preflight: corretto nello stesso percorso per non
perdere le nuove richieste. Un caso non supportato resta errore anche nella
conversione difensiva dopo preflight; nessun filter_map scarta dati.
Il re-export Capability rende nominabile il tipo già restituito dal getter
read-only, senza aggiungere setter/constructor compiled o capability runtime.

rules::compile_assertion abbassa DerivedCompare con binding del nome nel
record, indice esatto, operator_plan condiviso e literal compatibile al tipo
del derivato (Bytes o Integer). Nomi mancanti o literal del tipo sbagliato
impediscono compilation; una when falsa non nasconde un'assertion invalida.

Anche SubstringEqualsDerivedYear, già assertion locale chiusa nell'authoring
ratificato, viene consegnata: ref target locale, source_range one-based comune,
year_digits solo 2/4, lunghezza della slice uguale alle cifre e range entro il
campo fisico. La fonte è Date con formato esplicito verificata da date_ref,
ora condivisa con i piani calendario; date_format_plan riusa la stessa enum.
Non si aggiunge una quarta operazione derivata o una funzione eseguibile per
nome. Questo controllo locale non deve essere confuso con RecordExists/SetExists,
che continuano a richiedere il contesto P13 e restano rifiutati.

## Runtime e frontiere

local_expression_supported resta la verifica strutturale per i contesti puri
che risolvono valori via porta senza possedere lo schema. Ora ammette Expr
derivati locali; DerivedEq richiede ref locale, DerivedCompare literal Bytes/
Integer. I contesti senza quell'operazione conservano Unknown UnsupportedPlan
della porta, indici non disponibili Unknown FieldUnavailable nel contesto locale.

local_expression_supported_in_schema (prima denominato per la sola presence)
riusa il controllo metadata completo di 10-04 quando il record dispone dello
schema: risoluzione indice/ID/tipo di piani e campi prima di leggere input.
derived_comparison_supported è condivisa fra Expr/assertion e prende il literal
per borrow: nessun AST temporaneo o clone dei byte a ogni valutazione di regola.
Il caso senza schema verifica le forme, quello con schema anche tipo e indice.

local_rule_supported riceve schema opzionale. evaluate_local_rule usa il
controllo strutturale per conservare la signature DET §47; RecordValidator usa
Some(schema) una volta alla costruzione e verifica anche tutti gli indici/ID
dei ref locali delle assertion. Nessuna decisione deriva da una factory finta.
SubstringEqualsDerivedYear controlla inoltre dimensione range, larghezza target
e tipo Date della fonte; l'AND short-circuit dei controlli locali assicura che
gli accessi agli indici avvengano solo dopo get+ID validi.

validate_assertion::DerivedCompare risolve il piano dal vero ctx e riusa
compare_derived: True soddisfatta, False violazione con codice della regola,
Unknown conserva il motivo. La riduzione delle assertion resta invariata:
prima violazione certa nell'ordine, altrimenti primo Unknown, nessun pass fittizio.

SubstringEqualsDerivedYear legge il target logical, applica ByteRange::slice
checked e valida la data source logical con DateLiteral::from_ddmmyyyy comune.
Dato corto/missing resta FieldUnavailable, data presente invalida InvalidFieldValue.
L'anno validato è formattato a quattro cifre con zeri, poi confronto byte con
le ultime due o tutte e quattro cifre esplicitamente selezionate. Lo slice 2..
è sicuro perché la data limita l'anno a 0001..9999, dunque output sempre 4 byte.
Nessuna modulo normativa, century window, data host o normalizzazione del record.

## Diagnostica e template

CompiledAssertion::primary_ref aggiunge il vero target di Year; DerivedCompare
e ConditionTrue conservano None, perché non hanno un singolo campo violato.
Non viene inventato un field ID: i template con field_id/field_name sono
rifiutati se un'assertion non offre un target naturale, come già nel contratto.

rule_diagnostics::expression_refs riceve anche lo schema e ritorna Result:
DerivedEq aggiunge il campo confrontato e le fonti del derivato, DerivedCompare
le fonti; gli operatori booleani conservano traversal nell'ordine dichiarato.
derived_refs risolve il piano con get checked e aggiunge source per Substring,
birth poi reference per i piani calendario. Un indice incoerente è InvalidPlan,
mai contesto vuoto inventato. assertion_refs aggiunge le vere fonti dell'assertion
fallita (incluse ConditionTrue/DerivedCompare/Year), non tutte le assertion
indiscriminatamente. I rami futuri non ammessi restituiscono InvalidPlan.

local_rule_error continua a deduplicare per field ID mantenendo la prima
occorrenza e a leggere il raw fisico da RecordView per ogni voce. I valori che
attivano la regola sono così attestati dai byte del record, non dall'esito
derivato preconfezionato. Ordine when poi assertion fallita preservato.

condition_text usa lo schema per DerivedEq/Compare. derived_text descrive
l'operazione con ID delle fonti, target raw/logical e range one-based, oppure
fonti calendario e policy leap esplicita. Non espone derived_index/field_index
o Debug AST. operator_text emette i sei operatori leggibili; literal_text usa
quote JSON per Bytes e segno/magnitude del parser Integer esatto, nessun float.
Il caso Date non compatibile resta InvalidPlan. L'espansione del template è
ancora singola/non ricorsiva, senza valutare espressioni in stringhe.

## Test e regressioni

Otto integration test in derived_rules sul compiler pubblico e i veri
RecordValidator/LocalEvaluationContext/ExpressionEvaluator/JsonlSink:
capability conservate e tre piani memorizzati; guard composto con tutti i
derivati e contesto raw ordinato/deduplicato, target/coordinate/message esatti;
sei operatori, bytes e interi firmati, zero padding e nessun float; Unknown
distinti da guard/assertion e guard falso; ConditionTrue DerivedEq; presenza
pubblica required/conflict; anno a 2/4 cifre compreso 0001, data invalida;
nomi/tipi/range/template incoerenti rifiutati prima dei record.

Il primo run mirato ha rilevato un errore nelle nuove fixture: l'operatore raw
approvato è `==`, non `=` (quest'ultimo è solo testo diagnostico). Corrette le
fixture, senza allargare la grammatica produttiva; tutti otto i test passati.
Il check ha rilevato anche un import rimasto inutilizzato nel refactor: rimosso,
nessuna allow per aggirare Clippy.

Un nuovo test CLI reale esegue schema filesystem e report atomici: violazione
derivata con context/message corretti, caso valido e fonte mancante Unknown.
Quest'ultimo termina tecnicamente e preserva i report precedenti/no part files,
senza pubblicare summary completa o audit inventati. Dati soltanto sintetici.

La regressione schema_compiler non elenca più derived_values fra le raccolte
ineseguibili, perché ora è consegnata; i negativi join/repeated/reference/restanti
rimangono invariati. Nessuna baseline source/normativa/golden riscritta.

## Evidenze e handoff

453 test/55 suite, zero falliti; log /tmp/lintra-t-p10-05-derived-test.log.
fmt/check all-targets/Clippy all-targets -D warnings/test/release/Windows GNU
check all-targets/diff --check exit 0, Cargo --locked. 64 file protetti SHA-256
identici a HEAD. Review QG-3: engine senza application/adapter o norme di flow,
input checked, nessun unwrap produttivo o float nuovo. CI nativa ancora da
eseguire, non equiparata al check Windows GNU locale.

File schema/compiler/derived/rules/compiled plans;
engine/expression_evaluator/record_validator/rule_validator/rule_diagnostics;
lib export, test derived_rules/cli_validate/schema_compiler, dossier/stato.
Nessuna dependency/Cargo/lock nuova. Next residuo T-P10-06 dopo gate nativo:
suite/golden dei derivati e compatibilità completa dello scope ratificato.
P10 resta aperta finché quei test/gate non sono consegnati.

CI [39919164236](https://github.com/UraganoAngelina/lintra/actions/runs/39919164236)
su 213ccb485ab9c8b57bedf820c3e891f858c96385: workflow e job Linux GNU/Windows
MSVC completed/success. Tre chiamate totali, intervalli >45s; nessuna verifica
nativa presunta dal check GNU. T-P10-05 estesa DONE; next residuo 10-06.
