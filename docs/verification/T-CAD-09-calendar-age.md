# T-CAD-09 — età cross-trace, 6 novembre 2026

IN_PROGRESS. Gate locali completi in corso in `T-CAD-09-local-gates.json`.
La suite workspace è conclusa: 943 test passati, 0 falliti, 6 ignorati.
Il processo dei test allfeatures della prima esecuzione non aveva un risultato
registrato alla ripresa: log conservato come `gate-3-interrupted-no-result.log`.
Ripresa dal comando 3 con gli stessi input verificati tramite SHA256.
La nuova CI42203885693 ha chiuso CAD03 e CAD11; non contiene il delta09.
La CI42197434407 suaae2e12 è fallita nel checkout dei dataset CAD11 dopo
che i testSQLite Windows sono passati. Nuovo gate42203885693 su6af5f85 con
fix.gitattributes, sempre senza il core09 locale. Non anticipare DONE09 oWindows09.

## Fonti e contratto già autorizzato

Prompt09, CAD05/07, note originali A1.24/B1.2.17/2.18 e authoringT-P0-07 ratificato
confermano delta=anno(inizioB1.2.4)−anno(nascitaA.1.11), soglie<=17/>=3/>=6.
Non si introduce un'età compiuta o una nuova soglia. Le dateDDMMYYYY devono essere
realmente valide. DEC-05-CAD-EVENT-CONTEXT/consensus sono già stati formalizzati
e approvati nell'authoring, come registrato in docs/normative/DECISIONS.md.
Preflight dettagliato in T-CAD-09-preflight.md; nessuna nuova scelta normativa.

La sorgente conserva presenceoptional per i3target; l'obbligo dipendente da dati
su altri trace è una regolaFieldPresent nella seconda passata. Non aggiungiamo
forbidden per età maggiori o minori della soglia. Domini/assenze/coordinate delle
singole field rimangono indipendenti. Tutto il confronto resta schema-driven.

## Compiler: piano tipizzato e chiusura delle proiezioni

src/schema/derived.rs introduce compile_scoped con ReferenceBindings espliciti.
La mappaRaw ordinata determina ancora gli indici canonici. Identificatori vuoti
o padded sono rifiutati. SoloCalendarYearDelta usa il resolver del flow: resolve
verifica proprietà della join, source/target/field e cardinalità scalare oppure
inclusione del campo nella tupla consenso; field deve essereDate. I riferimenti
risolti possiedono indici/ID, non valori record. Substring eAgeInCompletedYears
conservano resolve_local_ref/date_ref e rifiutano sorgenti esterne.

Il wrapperlocale delle unit esistenti usa adesso lo stesso compile_scoped con
ReferenceBindings::local: i test locali non verificano una vecchia implementazione
parallela. Le factory del compiler preflight e le derived_tables finali usano
ReferenceBindings::cross sul trace effettivo. Non si costruisce un compiled falso
rimuovendo capability/policy o proiezioni. Nessuna nuova grammar o capability.

expression_is_local percorre iterativamente tutte le expression. Ogni riferimento
e ogni sorgente del piano derivato deve essere locale; l'indice del piano viene
verificato. Non valuta costanti per nascondere una dipendenza non supportata.
compiler.rs lo applica alle presence dei campi dopo lowering; rules.rs ai predicati
SetExists del target. Un calendar_cross usato lì fallisce con schema error prima
dell'I/O: quei contesti non hanno la seconda join necessaria. Le condizioni di
presenza CAD vengono espresse nelle regolecross, senza indebolire questa boundary.

La RuleDependencies esistente segue già birth/reference e raccoglie join e campi
in ogni trace. Non è stata sostituita con un clone di tutte le righe: la chiusura
seleziona esattamente4fieldA e5fieldB1 nel contenitore sintetico, senza filler.
La passata1 proietta, la passata2 riusa il ProjectedEvaluationContext reale e i
piani del medesimoCompiledFlowSchema. Nessuna terza lettura o passata aggiunta.

## Guardia runtime e valutazione

src/engine/derived_evaluator.rs aggiunge flow_derived_type. Il caso locale conserva
il controllo precedente; l'unica estensione èCalendarYearDelta. Per la Refesterna
verifica join_index/JoinId, sourceID, targettrace, field_index/FieldId, consenso o
cardinalitàsingola, e tipoDate dello stesso format. Restituisce Integer soltanto
con metadata coerenti. Nessuna join eseguita durante questo controllo.

record_validator.rs passa il flow alla guardia metadata: il passaggio locale può
portare definizioni cross da usare più tardi, senza eseguirle né ignorare obblighi
locali. Le guardie locali su rules/presence rimangono locali. LocalOnly conta le
regolecross saltate e conservaPartial; non dichiara una conformità globale.

La matematica DerivedValueEvaluator non cambia: source_bytes conserva Missing e
Unavailable; DateLiteral parsea entrambe le date gregoriane; la sottrazione degli
anni rimane signed/checked. Non usa hostdate, compleanno, zeri o date inventate.
ProjectedEvaluationContext già risolve realmente i piani sui dati proiettati e
sul consenso. Un parent fuori namespace non fornisce dati autorevoli al contesto.

## Unknown causale senza nuova API pubblica

Il precedente evaluator non associava una join all'Unknown proveniente dal derivato.
Le missing policy devono però distinguere targetvuoto verificato, trace assente,
contestoPartial eambiguità: non basta un Unknown generico o una join scelta a caso.

failure_join è helperprivato: considera solo i4motivi legati a join e percorre
le sorgenti nello stesso ordine/target(raw/logical) usato dal derivato. DateMissing
eInvalidFieldValue non diventano parentmissing. Le risoluzioni del contesto sono
immutabili durante il passaggio, perciò l'ispezione ritrova il motivo decisivo
senza I/O o nuovi record posseduti. Non alloca una lista di valori o errori.

expression_evaluator.rs usa un helperinterno con la tabella dei piani del source
schema. DerivedEq/DerivedCompare conservano la causa quando il derivato è decisivo;
la riduzione booleanapreesistente la elimina perFalseANDUnknown/TrueORUnknown.
Il metodo pubblico evaluate e il resolver/context/value pubblici non cambiano.
flow_validator.rs passa i piani effettivi a when eConditionTrue. Gli altri casi
usano il recupero causale delle assertion già presente, che segue i medesimiRef.
Niente nuovo campo DTO, firma pubblica, enumTruthValue o parametro ignorato.

## Componente CAD esplicito

schemas/cad/components/2024-06.calendar-age.1 conserva sourcehash e copie legacy
A/B1. I campi con coordinate originali sono1.1/1.11/1.23/1.24 e2.1/2.4/2.17/2.18/
2.36; PK completeA/B1 e scopeULSS condiviso con la ratificaCAD02. Le forme legacy
digits_only/if_present sono abbassate esplicitamente a regex rawASCII ancorate di
larghezza esatta; datecalendar_valid:true canonico. Non si accetta tacitamente
il costrutto legacy nella grammar. Numericieventi12byte restano exact, zerosolo
missing dove dichiarato; nessuna conversione delle chiavi o normalizzazione nuova.

B1→A è scalare per nascita;A→B1 usa consensus sulla data iniziale, con diagnostica
Unknown in caso di discordanza. Anche due date diverse del medesimo anno restano
confliggenti: non si sceglie la prima né si confrontano solo gli anni per il consenso.
Le correlazioni del componente sono current_submission: non si inventa un valore
storico mancante. Snapshot/temporali e vista storica approvata non sono questa task.

Non è uno schemaCAD completo/FROZEN. Il Full corrente richiede input per tutti i
traceche possiedono controlli sorgente; i test di targetassente usano gruppi di
regole esplicitamente scopiati e un flowdi test distinto. Non rimuovono obblighi
per dichiarare completa la conformitàCAD. Missingtrace operativo/consenso resta05,
integrazione e golden completi12. CNF-DIAGNOSTIC-ORDER-01 rimane separata e aperta:
nessun riordino del prodotto corrente o nuova campagna prestazionale.

## Test osservabili sul servizio reale

I12casi in tests/cad_age.rs usano InMemorySchemaRepository/SchemaRegistry/compiler
eValidationService reali, file sintetici A/B1 e summary/JSONL reali. Non sono outcome
Full costruiti a mano. Filler espliciti fuori scope non vengono validati o proiettati.

- Soglie2/3/5/6 con nascita31dicembre e inizio1gennaio: distinguono deltaanni da
  compleanno. Gli erroriFieldPresent identificano esattamente2.17/2.18.
- Responsabilitàgenitoriale a17/18, facoltativa ma comunque ammessa se presente
  dopo17; nessun forbidden nuovo.
- Chiusura delle proiezioni minima4/5field e3controlli realmente eseguiti inFull.
- Date missing/invalide su entrambi i trace: errore locale primario eUnknown,
  senza età fittizia né ulterioreConditionalRequired.
- B1 concordanti contro due date diverse nello stesso anno: consenso oppure
  Unknown conW_CAD_AGE_CONTEXT_CONFLICT, nessunfirst().
- TargetA assente: Unknown/Partial;A presentevuoto: due violazioniMissingRecord
  previste dalla policy. B1 assente nel gruppoA: Unknown/Partial.
- Namespace errato/parentduplicato: nessuna nascita affidabile, nessuna nuova
  violazioneMissingRecord dedotta da un contesto incompleto o ambiguo.
- LocalOnly esegue i controlli locali, salta esplicitamente le due regolecross,
  ritornaPartial senza provare una conformità globale.
- DominanzaFalse/True preserva la logica approvata anche quando il derivato avrebbe
  una missing policyViolation; nessuna violazione da una join non decisiva.
- Join non dichiarata, setscalare senza consenso, data su fieldnonDate, presence
  locale e predicatoconaltrojoin, anni compiuti esterni: compileerror, non fallback.
- ID di trace/campi rinominati: stessi obblighi suMOBILITY/FUNCTION, JSONLripetuto
  byteidentico eLF. Nessun confronto sui nomi normativi nel core.
- Formato1 generico senza profilo di persistenza/namespace: il calendarjoingenerico
  funziona con le sole join dichiarate. Lo storage non è prerequisito dell'engine.

## Tentativi preliminari e ripresa

Primo check fallito: qualificazione super::derived errata nel modulo rules, che è
figlio del compiler; corretta a crate::schema::derived. Secondo check positivo.
Prime due suite: parser rifiuta correttamente calendar_valid mancante e il costrutto
legacydigits_only. Componente corretto con lowering esplicito canonico, senza
modificare il parser per tollerare il legacy. Terza suite11/11pass, Clippy positivo.

Interruzione di recoverySQLite per il gateWindows03; codice09 preservato con9hash
prima/dopo il trasferimento del fix isolato. Poi affinamento del wrapperlocale,
5unitderivati e31test derivati/presenza positivi; caso genericoformato1 positivo.
Questi tentativi non vengono riscritti come gatefinali. La sequenza finale11comandi
su sorgenti fermi comprende il fixfilesystem pubblicato e tutti12casi09 aggiornati.

## Stato del Quality Gate

Ledger/log in T-CAD-09-local-gates.json eT-CAD-09/gate-N.log. Suitecomplete
workspace/default/allfeatures, regressione senza bundle, release/tooling/Python/
boundary sono distinte dai tentativi preliminari. Finché non terminano e la nuova
CI nativa09 non qualifica il commit corretto,09 restaIN_PROGRESS/WAITING_GATE.
Il gate recovery03/11 aae2e12 non contiene questo core09 e non lo può certificare.
Nessuna release/tag/chiaveoperativa generata; docs/migration preesistente esclusa.


## Provenienza del gate locale durante recovery checkout

La sequenza locale parte daaae2e12 con il delta09. Durante la compilazione è
stato pubblicato6af5f85, che aggiunge soltanto.gitattributes ed evidenze della
conversioneCRLF dei datasetCAD11. Nessun byte di src/schemas/tests/Cargo/UI
è cambiato durante i gate09; il ledger confronta le impronte prima/dopo.
Il fix.gitattributes preserva il checkout futuro Windows e non modifica i dati
letti dai test Linux già in corso. Il diversoHEAD del checkout viene documentato,
non presentato come un'esecuzione CI09 su6af5f85. La CI finale09 deve comunque
qualificare il suo nuovoSHA pubblicato completo, incluse tutte le recovery.


## Pubblicazione del codice durante il completamento dei gate

Il delta funzionale è completo e ha già superato fmt, Clippy workspace/allfeatures,
la suite workspace (943 esiti passati, inclusi sottoprocessi; 0 falliti, 6 ignorati)
e i12casi di servizio. Per sovrapporre la verifica nativa ai restanti controlli
locali, viene pubblicato ora come IN_PROGRESS: nessun DONE e nessun comando
incompleto dichiarato positivo. I comandi3–10 restano obbligatori e proseguono
in foreground sugli stessi input SHA256; il cambiamentoHEAD influisce soltanto
sulla provenienza della build. La nuova CI verifica lo SHA pubblicato completo,
non eredita il risultato CAD03/11 e non sostituisce i gate locali ancora aperti.
