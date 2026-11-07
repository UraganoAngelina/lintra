# T-CAD-06 — sospensioni su input correnti

WAITING_GATE, 7 novembre 2026. Prerequisiti/fonti/perimetro nel preflight06.
Undici test CAD06 e51test di regressione positivi,0fallimenti/0ignored.
Gli11gate locali sono conclusi positivamente su input fermi.
Nessun gate nativo06 eseguito né DONE anticipato.

## Compilazione e guardie

schema/temporal.rs introduce read_only, date e compile. read_only verifica
che una policy temporale non abbia obblighi di snapshot/audit/identità; non
riscrive lo schema. date risolve un indice e richiede tipo fisico Date, non
un campo testuale con nome promettente. compile verifica capability, owner,
join da quello stesso owner, zero_or_many senza consensus, condizioni locali
e policy Unknown coerente col join. Riferimenti/diagnostici malformati e
altri piani temporali sono errori di schema.

compile_persistence_profile rifiuta prima le combinazioni non eseguibili;
la policy protetta contiene soltanto piani effettivamente compilati. La normale
compile ammette il perimetro read_only oltre ai binding qualificati; il profilo
CAD con snapshot e storico completo continua a essere rifiutato. Il servizio
storico e l'approvazione non vengono indeboliti: il primo richiede audit-only;
la seconda richiede snapshot o audit, assenti nel nuovo profilo di lettura.

CompiledAssertion::IntervalExclusion è solo un nodo compiled, non un nuovo
costrutto raw. Il piano esistente conserva when e gli indici risolti. Il
compiler lo abbassa in CompiledRule della seconda passata, con ID stabile per
indice e controllo delle collisioni con regole dichiarate. RuleDependencies
aggiunge le date sorgente/target e il join; la visita ordinaria di when include
le dipendenze delle condizioni e dei valori derivati locali.

## Esecuzione

interval_exclusion riusa ProjectedEvaluationContext e il lookup del join.
La data sorgente viene interpretata con DateLiteral::from_ddmmyyyy. Missing
o byte non interpretabili sono cause Unknown, mai date zero. Il target è un
insieme in prestito: nessuna selezione di un rappresentante, Vec di intervalli
o copia di record. Start Missing è incerto; solo End Missing apre l'intervallo.
Start maggiore di End è incerto; intervallo di durata nulla non ha interni.

La scansione verifica start < point e point < end quando end esiste. Una
violazione certa termina la ricerca; se non esiste, una data incerta prevale
sul semplice esito pulito. Anche senza date incerte, coveragePartial dà Unknown
perché una riga non proiettabile potrebbe contenere il controesempio mancante.
File assente conserva TraceInputUnavailable e diagnostico del join; file
presente vuoto completo dà Passed. L'ordine dei record non influisce su
violazione/soddisfazione e il codice normativo è letto dalla dichiarazione.

L'asserzione passa dal loop ordinario di run_cross_record_rules: stesso sink,
observer, contatori, budget, cancellazione e avanzamento. Il diagnostico usa
la trace e la riga sorgente reali. Nessun campo o valore sensibile viene
inventato per un intervallo; CrossDiagnostic omette i riferimenti scalari
del set ma conserva la condizione locale. LocalOnly non esegue il piano cross.

## Configurazione e test

Componente versionato A/B2/C con13campi originali, fonti/hash e PK complete.
I dati di fixture rispettano coordinate fisiche della fonte e dichiarano i
restanti byte come filler non validato clinicamente. Prime prove:6motivi,
5sedi non domiciliarie, confini stretti, aperti, sovrapposti/eventi distinti,
input assenti/vuoti, date invalide e prevalenza del controesempio certo.

I tentativi preliminari sono distinti dai gate finali: correzioni delle chiamate
API del test, calendar_valid obbligatorio, regex generate e larghezza/offset C
errati della fixture. Nessuna norma, source_fields originale o baseline
FROZEN modificata per far passare i test.

## Limiti espliciti

Solo input correnti selezionati per la validazione. Nessun confronto con date
storiche persistenti, periodo ricostruito dall'orologio o profiloCAD completo.
Il percorso cumulativo e la riconciliazione rimangono a CAD12; after-date e
inattività alle task dedicate. Cancellazione al confine dei record come la
seconda passata esistente; la scansione del singolo set non introduce worker.


## Guardia sulle dipendenze indirette di when

Il test aggiuntivo ha riprodotto l'accettazione indebita di un when che leggeva
un calendar_year_delta con operando esterno. Il solo compile_local_expr risolve
il nome derivato ma non ne prova la località. Il fix riusa compile_scoped dei
derivati ed expression_is_local, che visita esaustivamente ogni dipendenza.
Il test ha fallito prima della correzione e passato dopo; i log sono conservati
separatamente dalle suite finali. Anche una dipendenza dietro una guardia
costante non può oltrepassare il confine locale dichiarato.

Regressione finale:11CAD06,12età,14cross-record,17namespace,8profili persistenti,
62esiti positivi totali. Ordine/JSONL deterministici, causeUnknown, rinomina di
trace/campi/join/codice errore, budget e pre-cancellazione verificati realmente.
FontePDF e13campi originali/PK/coordinate verificati in T-CAD-06-source-check.json.


## Gate locali intermedi

Fmt/Clippyworkspacealltargetsallfeatures exit0; workspace standard e allfeatures
970pass/0fail/6ignored ciascuna, con impronte sorgenti invariate dopo ogni
comando. Core senza bundle, release, tooling e controlli finali ancora in corso.
Questi esiti non completano il gate06 e non sostituiscono la futuraCI nativa.
CAD04 nel frattempo qualificata realmente su CI42368062138:134/140DONE.


## Helper di fixture e motivi di correttezza

component legge il JSON versionato incluso nel test; filler dichiara soltanto
le colonne sintetiche non validate. model costruisce un RawFlowSchema di test
con i tre tracciati A/B2/C, copia campi/PK/policy del componente e riempie solo
i gap fisici: il record_length e l'avanzamento next rendono espliciti gli
offset, senza spostare campi normativi. compile applica il binding UTF8 esplicito
e invoca il compilatore reale, senza simulazioni di piani.

set converte una coordinata one-based in slice zero-based della riga sintetica.
a costruisce il recordA con ULSS/evento; suspension costruisce41byte B2 con
date/motivo/evento/progressivo; access costruisce45byte C con data, sede e
PK completa, progressivo all'offset34 della fonte. Gli altri byte restano
filler dichiarato. I valori usati sono artificiali, non copie di input reali.

Fixture possiede una directory temporanea sotto target e opzioni di budget/
cancellazione. new crea un contenitore privato riusando tempfile già adottato;
run scrive i soli file ricevuti, distinguendo None da un file senza righe,
registra lo schema nel repository reale, costruisce ValidationService e
passa il namespace tipizzato. Full seleziona tutti gli input presenti; la
variante Trace resta esplicita nel helper. Lo stesso max_errors e token di
cancellazione arrivano alla request/service; il summary viene realmente
serializzato e verificato. Alla fine la directory è rilasciata dal suo owner.

Il helper run esterno sceglie Full con A501, B2 opzionale e C ricevuto. errors
conta soltanto il codice normativo della sospensione, distinguendolo dagli
errori locali deliberati delle fixture negative. control trova l'ID compilato
stabile e legge l'oggetto di summary reale: Unknown e Violated non vengono
dedotti dalla sola assenza di errori nel sink. La codifica JSONL usa JsonlSink
e finish reali; i due run confrontano i byte, senza normalizzare i diagnostici.

Le condizioni dei test sono derivate dai confini normativi: gli accessi sulle
date di start/end sono ammessi; un giorno interno a qualsiasi intervallo è
bloccante per sede1; nessun filtro elimina motivi B2 validi. Le prove di
coverage e delle guardie confermano che una mancanza di evidenza non diventa
un falso successo, mentre il test di rinomina verifica l'assenza di dipendenze
da nomi CAD nel codice core. Nessun output del software viene usato per
riscrivere un valore normativo atteso.


## Qualificazione locale finale

Undici comandi finali exit0;464impronte prima/dopo identiche e ricontrollate
alla conclusione. Fmt/Clippyworkspace e tooling,970workspace standard,
970workspaceallfeatures,927core senza bundle:0fail/6ignored per suite.
Release positiva;18tooling inclusiignored e24Python positivi, boundary positivo.
Conteggi ricostruiti dai log e confrontati con il ledger. Inventario dei25log
in T-CAD-06/evidence-hashes.json, inclusi i tentativi falliti.

Il primo comando8tooling ha fallito un solo test perché il runner ometteva
LINTRA_TEST_BIN. Conservati ledger originale e logfailed-missing-binary-env.
Senza cambiare sorgenti sono stati ripetuti il comando8con il vero binario
release appena costruito e completati9/10;18testtooling tutti verdi. I comandi
0–7 già positivi e a impronte immutate non sono stati ripetuti inutilmente.
Il ledger corrente conserva il motivo del retry e l'ambiente esplicito.

Ancora necessaria CI completa sul commit pubblicato prima di DONE06.
Nessun Windows locale eseguito, nessuna attribuzione aCI04 del nuovo codice.
