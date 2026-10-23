# T-P10-06 — Suite DSL e golden del contratto derivati esteso

2026-10-23. Prerequisiti 10-05 estesa 213ccb4/CI 39919164236, workflow e
job Linux/Windows completed/success; 5-07 già verificata b0e5ce7/CI 39416549983.
Prompt/Piano §4/§14, ARCH §34.2, DET §110, QG-3/4 e authoring ratificato
T-P0-07 applicati. Nessuna nuova operazione o modifica architetturale.
La rettifica P10 non viene chiusa finché anche questi gate non sono verificati.

## Suite del percorso pubblico

tests/derived_rules aggiunge quattro test agli otto della consegna 10-05,
riusando le fixture sintetiche e le vere compilation/validation, senza nuovi
context di test che ritornino valori derivati preconfezionati.

derived_rules_resolve_by_id_across_reversed_physical_layouts dichiara un guard
named che compone DerivedEq intero, DerivedCompare substring e delta anni.
Compila una disposizione fisica e poi quella con tutti i cinque campi invertiti,
ricalcolando start cumulativi dalle larghezze esplicite. Per ogni input ricava
le parti raw usando le coordinate originarie e le ricompone nel nuovo ordine.
Il compiler risolve così nuovi field_index veri, senza modifica dei ref per ID.
Si confrontano artefatti, codice/actual/expected/context/messaggio; le coordinate
del solo target cambiano correttamente da start 1 a 25. Record validi, violazione
e sorgente assente Unknown devono conservare lo stesso esito. Nessuna modifica
dei byte dentro un campo o dei piani di derivazione per far passare il confronto.

known_failed_assertion_dominates_an_earlier_unknown_derived_obligation verifica
la riduzione AND delle assertion: dato nascita mancante rende ignoto il primo
confronto età, ma DETAIL sicuramente mancante viola la seconda assertion.
La violazione certa resta Invalid, non Unknown o Satisfied; il primary/context
è quello della vera assertion fallita, senza inventare un valore per la prima.
È coerente con False AND Unknown = False e con la riduzione già approvata.

static_conflicts_are_proved_without_guessing_the_truth_of_a_derived_condition
assegna lo stesso guard età a required/forbidden. Il compiler non può assumere
che sia sempre vero: ammette lo schema, il record adulto produce Conflict,
quello minorenne permette l'optional mancante. Sostituendo entrambi i guard con
Const(true), il conflitto è invece certo e SchemaIssue PresenceConflict statico.
Non si introduce un solver, una soglia hardcoded o una deduzione normativa.

derived_profile_roundtrips_and_unreachable_bad_definitions_are_rejected controlla
roundtrip raw completo del nuovo profilo, compilation dei tre piani, ciclo di
named_conditions e riferimento derivato assente in una definizione inutilizzata.
Nessun nodo ineseguibile viene nascosto dalla mancata chiamata a runtime.

Le prove precedenti 10-01/03/04/05 restano parte della suite completa:
operatori, Unknown, bounds, tipi, leap policy esplicite, signed age/delta, range
raw/logical, date 0001/9999, century leap, presenza e report CLI atomici.
Non sono ricostruite o dichiarate prove normative di profili reali.

## Nuovo profilo golden e oracolo indipendente

tests/golden/derived/v1/schema.json è un profilo nuovo, non una baseline normativa:
22 byte, DETAIL AN 3, BIRTH/REF Date 8 e AGE Numeric 3. Contiene tutti e tre i
derivati con capability esplicite e regole ordinate NEEDS-DETAIL, AGE-MATCH,
YEAR-LIMIT. La soglia 25 è soltanto un parametro sintetico di schema.

valid/input contiene cinque record validi: febbraio 29/mar01, minorenne senza
DETAIL, anno 0001/età zero, date in ordine inverso con risultato firmato e
AGE assente esclusa da IsPresent, compleanno 01/03. invalid/input contiene
quattro record (uno valido e tre invalidi), tre violazioni specificate:
adulto senza DETAIL, età 021 invece di 020 prima dell'anniversario mar01,
delta 121 oltre il limite esplicito 25. Sono tutti dati artificiali.

generate_derived_goldens.py costruisce una lista ESPLICITA di tre errori,
non implementa un secondo evaluator e non legge alcun report Rust. encoded
serializza JSON compatto LF, context costruisce valori raw e trace/field,
error fissa i DTO del contratto. main usa le decisioni della tabella sopra,
Counter per codici e hashlib per schema/input originali, conteggi dichiarati
5/5 valid e 4/1 valid. Scrive solo i nuovi attesi. Nessuna modalità update del
runner è usata per determinare il risultato corretto del software.

tests/golden registra il quarto profilo esplicitamente, continuando a rifiutare
profili non registrati. Il produttore è quello reale comune di RecordReader,
audit EOF, RecordValidator e sink JSONL/summary. Un nuovo test riapre input e
validator indipendenti due volte, confrontando ogni byte dei due report.
Tempo fisso nel dominio di test prima della serializzazione, nessuna maschera
o normalizzazione dell'output per nascondere differenze.

La suite ora ha dodici scenari su quattro profili. .gitignore aggiunge soltanto
l'eccezione tests/golden/derived/**/input.txt; .gitattributes già conserva tutti
i byte sotto tests/golden. I sette nuovi file schema/input/expected sono
versionati insieme: niente fixture locale ignorata ma assente in CI.
README documenta casi/origine attesi/limiti. Nessun vecchio atteso riscritto.

## Evidenze e handoff

Test mirati: dodici derived_rules e tredici test del runner golden passati.
Primo run dei nuovi golden e prove di ripetizione passati direttamente, senza
correggere gli attesi copiando report Rust. 458 test/55 suite, zero falliti;
log /tmp/lintra-t-p10-06-derived-test.log. fmt/check all-targets/Clippy all-targets
-D warnings/test/release/Windows GNU check all-targets/diff --check exit 0,
Cargo --locked. QG-3 positivo per suite/perimetro invariato; QG-4 locale positivo,
12 scenari raw byte-identici e ripetizione indipendente. CI nativa pendente.
64 artefatti protetti precedenti identici SHA-256 rispetto a 213ccb4 (prima dei
nuovi golden), README escluso dal conteggio. git check-attr conferma text unset
per input nuovi, git check-ignore non li esclude; file aggiunti esplicitamente.
File .gitignore, tests/derived_rules/golden, nuovo profilo derived, README,
oracolo Python e dossier/stato; nessun codice produttivo o Cargo/lock cambiato.
P10 estesa sarà DONE solo dopo gate nativo di questa consegna. M4/M5, Full,
P13/PERSIST e T-P2-03/M1 restano aperti. 11-02 già verificata; prossima 11-03
attende decisione reale CNF-REFERENCE-FORMAT-01, separata dal contratto P10.
