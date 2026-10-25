# T-P13-07 — Preflight servizio Full, summary e CLI

2026-10-24, prima del codice. Prerequisiti reali: 13-06 commit 134b593, CI
40091666797 Linux/MSVC success; 8-03 commit 10932f9, CI 39743161012 success.
Lette DET §23–25/§83–86/§95–97, Piano §9/§14, prompt atomico 13-07,
CNF-REFERENCE-FORMAT-01 aggiornata, CNF-RUN-SCOPE-01 e dossier 13-05/06.
Il mandato più recente autorizza la prosecuzione automatica e prevale sugli stop
ordinari. Nessun Superpowers, subagente o processo in background.

## Stato e contratto minimo prima delle modifiche pubbliche

Il servizio rifiuta Full/Flow. Il core FlowValidator è reale, ma restituisce
artifacts engine: i finish legacy del builder rifiutano di perderli. Il summary
persistente conserva solo reference audit; result VALID da solo non descrive
completezza. La CLI validate è Trace/LocalOnly e protegge un solo path input.

Aggiungere `ValidationSummary.full_validation: Option<FullValidationSummary>`
con DTO neutri del dominio per trace coverage, verifiche PK, conteggi cross e
cause tipizzate con JoinId/diagnostica. Nessun import dominio→engine/schema.
La conversione dagli artifacts engine è responsabilità application. Conservare
RunStatus e il campo result con la loro semantica normativa; pubblicare in Full
anche completeness esplicita e l'envelope completo. LocalOnly senza envelope
mantiene i precedenti bytes JSON. La nuova Option impone un aggiornamento dei
costruttori Rust che usano struct literal: impatto esplicito, test aggiornati;
nessuna migrazione o sostituzione delle firme LocalOnly.

Un nuovo finish Full del builder consuma envelope/reference reali, verifica
scope, contatori disgiunti, cause, provenienza e copertura, e rimuove il marker
soltanto attraverso questa via. I finish legacy continuano a rifiutarlo. Summary
validate e report controllano l'envelope prima di emettere byte. Partial se
annullato/troncato, skip effettivi, Unknown cross, PK inconcludenti, reference
provvisorie/inconcludenti o copertura insufficiente dei trace nello scope.
NotApplicable resta esito distinto di scope, non una violazione o Unknown;
non rende automaticamente Partial una regola fuori applicabilità.

I contatori cross misurano valutazioni record/regola, non ID regola distinti;
le righe non lette non vengono inventate. Le cause sono aggregate per controllo,
join e motivo, senza valori dei record. SHA EOF, copertura indice, stato di
esecuzione, completezza ed esiti normativi restano dimensioni separate.
Persistenza dell'envelope JSON/XLSX non equivale al repository storico PERSIST.

## Servizio, parametri e scope

Preservare `validate(request, ...)` come API senza parametri espliciti, tramite
wrapper con mappa vuota. Aggiungere una via per parametri runtime per run,
senza alterare la struttura DET §25 ValidationRequest. La mappa in ingresso
contiene testo esplicito; il servizio risolve il tipo dalla ParameterDefinition
compilata, non dalla forma del valore (un codice "001" resta stringa se dichiarato
stringa). Interi tramite IntegerLiteral esatto, domini/range e namespace verificati
prima dell'input. Nessun default di anno/ente o valore scelto dal dominio ammesso.
Registrare i binding effettivi nell'audit Full con tipo, in ordine deterministico.

Eseguire Full soltanto con scope Flow; errore esplicito di compatibilità per
Full/Trace. LocalOnly/Trace resta identico; non ampliare LocalOnly/Flow per questa
consegna. Preflight Flow e parametri prima di caricare reference e aprire input,
riusando la preparazione engine per evitare due interpretazioni dei required.
Costruire observer reference per ogni trace reale, aggregando dataset una volta
e controlli in ordine trace/campo/regola. Mappare errori engine verso categorie
applicative mantenendo le source. Progress usa le fasi esistenti e nessun evento
finto per passate non eseguite. Report atomici conservano Err/Cancelled.

## CLI e verifiche

Aggiungere validate-flow --trace-input ID=PATH ripetibile e --parameter ID=VALUE
esplicito. Path nativi con spazi, Unicode, '=' e, su Unix, byte non UTF-8 restano
PathBuf: separazione soltanto al primo '=' con API native sicure. Nessuna chiamata
CLI diretta al motore: passa dal servizio. Proteggere tutti i path input dalle
destinazioni finali e .part; preservare bundle dirs/JSONL/XLSX/summary e rollback.
Esporre nel riepilogo Full completezza, skip/Unknown/NotApplicable aggregati,
senza trasformare warning in errori normativi. Futuri consumer UI usano i DTO.

Test: servizio e processo reale Full valido/invalido/parziale, più trace ordinati,
preflight missing/duplicate/unknown, parametri richiesti/invalidi, reference
Verified/Degraded/Unavailable, summary persistente/riclassificazione/contatori,
nessun falso Complete e warning nel sink normativo, output atomici e protezione
di ogni input, regressioni LocalOnly e paths. fmt/check/clippy/test/release,
Windows GNU e CI nativa Linux/MSVC; baseline90 intatta.

Nessun conflitto nuovo da sottoporre: sono integrazioni minime necessarie per
consegnare il servizio Full e i requisiti già ratificati, con impatto pubblico
esplicitato prima del codice. T-P13-08/G0 reale, PERSIST e P16 restano task distinte.

### Port observer cross opzionale

Il core 13-06 aggrega già le evidenze cross reali dentro FullCrossObserver.
Rendere opzionale il port observer esterno del FlowExecutionPorts: l'applicazione
può consumare gli artifacts senza inventare un NoOp o duplicare aggregati.
I consumer con callback reale passano Some(observer), comportamento invariato.
È una variazione del campo pubblico appena introdotto, dichiarata qui prima
della modifica; test core aggiornati e integrazione servizio esercitata.
