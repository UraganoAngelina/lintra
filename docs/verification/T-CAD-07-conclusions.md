# T-CAD-07 — conclusioni e decesso sui file correnti

DONE, 7 novembre 2026. Undici test nuovi+62regressione=73positivi,
0fail/0ignored; Clippyworkspacealltargetsallfeatures positivo. Undici gate completi
locali conclusi positivamente; CI42390593313 success sullo SHA6477bb2 con sette job richiesti positivi.
Prontezza/fonti/limiti nel preflight07. CAD06 giàDONE sulla CI42372080602.

## Compilazione

Il dispatcher temporal.compile verifica separatamente le capability esistenti
intervals e after_date e abbassa ciascun piano supportato. after_date risolve
owner/join/campi, verifica Date sui due operandi temporali, legge i byte esatti
dei valori vietati ed esige lista non vuota e unica. La cardinalità scalare
o un consenso dichiarato sono obbligatori: una relazione a insieme non viene
scambiata per un singolo record. PolicyUnknown del piano/join devono coincidere.

when/target_when vengono compilati localmente e verificati con il visitatore
esaustivo di località dei derivati. include_if del consenso passa dalla stessa
verifica. RuleDependencies.local_fields raccoglie le dipendenze del predicato
target usando un indice di scope solo compilativo; ogni campo deve essere
contenuto nel consenso insieme alla conclusion_date. Non è una proiezione o
un trace runtime inventato. Il collect ordinario include date/campo ristretto
sorgente e dipendenze target; nessun valore richiesto scompare dalla passata1.

CompiledAssertion.AfterDateRestriction possiede unBox del piano tipizzato
esistente, limitando la dimensione dell'enum. Il compiler genera lo stesso
CompiledRule usato dalla seconda passata, con ID deterministico per indice e
controllo collisioni. Gli ID e i comportamenti intervallo06 rimangono invariati.
I profili con snapshot/storia/invarianti non vengono resi eseguibili eliminando
obblighi: la guardiaread_only preesistente rimane vincolante.

## Resolver target

QualifiedTarget è una vista privata in prestito su contesto, schema target e
join verificato. external accetta solo riferimenti locali coerenti conID/indice
del target e li riassocia all'indice/idjoin già qualificati. resolve_field e
resolve_raw_field delegano ai percorsi distinti del contesto; non simulano
bytefisici dai logici. resolve_derived usa i piani del target sullo stesso
resolver, così un derivato locale non legge accidentalmente il record owner.

Il resolver non possiede righe né sceglie un record, non consulta storage e
non estende laAPIapplicativa. Il consenso preesistente decide se esiste una
tupla qualificata, se è ambigua o se mancano prove; i byte sono presi in prestito
dalle sue proiezioni. I riferimenti non possono sopravvivere ai loro owner.

## Esecuzione e diagnostica

after_date_restriction verifica prima che il valore sorgente appartenga
all'insieme vietato: un valore non soggetto non ha bisogno della conclusione.
Campo mancante/unavailable rimaneUnknown. Lookupassente/nessunrecord correlato
non diventa evento aperto; si applicano missing policy e coverage.

La data di conclusione viene risolta tramite la tupla qualificata. Su evidenza
completa e non vuota, Missing o nessuna tupla dopo include_if descrivono una
conclusione assente; datiinvalidi non vengono confusi con questo caso. Un
conflitto o una coveragePartial rimangonoUnknown con causa/diagnostico effettivi.

Il parserDateLiteral controlla calendario e rappresentazione DDMMYYYY.
Point<=conclusion è fuori dalla restrizione; solo un punto successivo richiede
target_whenTrue sulla tupla. False dàPassed, Unknown conserva la causa deljoin,
True produce il codice dello schema. Il filtro sul motivo quindi non elimina
unB1confliggente prima della qualificazione.

CrossDiagnostic deriva il naturale campo ristretto dall'indice compilato e
crea un riferimento locale transitorio, senza cambiare il contratto del piano.
Il factory esistente ricava nome/range/actual dalle coordinate sorgente, mai
dal target. Nessun evento o dato target viene aggiunto in chiaro. Sink, observer,
contatori, budget, annullamento e ordine trace/linea/regola restano quelli
della seconda passata esistente. Non c'è alcuna mutazione persistente.

## Configurazione e fixture

13campi originali A/B1/C, PK e coordinate/hash verificati nel source-check07;
anche l'unione dei due insiemi di sedi coincide con CAD_RUNTIME_PROFILE.
Codici2.7Ndi due byte conservati esattamente; dominio10/13governato da2.3.
La normalizzazione delle due regole negative usa una guardia di presenza: il
primo test ha riprodotto UndeterminedRule su codice facoltativoMissing; corretto
il componente, non indebolito il valutatore. La required_if data presente
resta separata e invariata. Log preliminari conservati distintamente dai gate.

I helper component/filler/model costruiscono lo schema parziale e dichiarano
i gap, mantenendo lunghezze fisiche originali. compile lega UTF8 e usa il vero
compiler; set converte una coordinata one-based alla slice della riga sintetica.
a/conclusion/access scrivono soltanto valori artificiali alle coordinate
AULSS/evento, B1evento/PIC/data/motivo/progressivo e CPK/data/sede.

Fixture possiede directorytemporanea e opzioni budget/cancel. run distingue
None da filevuoto, prepara repository/registry/service reali, passa il namespace
tipizzato e serializza davvero il summary. I helper errors/control leggono
codice normativo e controllo reale, non deducono Unknown dall'assenza di errori.
JsonlSink.finish codifica e confronta due run; rinomina di flow/trace/campi/join/
codice prova l'assenza di ramiCAD nel motore, incluso il campo diagnostico.

Test:19motivi validi con PICappropriato, quattro decessi/sei sedi/prima/same-day/
dopo, anni diversi e29febbraio, eventi aperti/assenti/vuoti/noncorrelati,
B1multipliuguali/confliggenti/filtrati, partial/dateinvalide, cardinalitàscalare
ambigua, derivati target e guardie dei riferimenti/capability/consenso,
determinismo/budget/pre-cancel. Esiti attesi dalla fonte, non dal comportamento
del software. Gli ultimi undici casi e62regressioni sono passati realmente.

## Limiti

Solo filecorrenti. Il profiloCAD completo e lo storico cumulativo rimangono
aCAD12; inattività aCAD08. Nessun nuovo crate, formato storage/HMAC, schema
FROZEN, credenziale o dato reale. Cancellazione tra record come nel pass2
esistente, nessun worker nuovo. Gate locali completi e CIpropria necessari
primaDONE07; la CI06 non qualifica questo delta.


## Gate intermedi verificati

Fmt/Clippyworkspace exit0, suiteworkspace981pass/0fail/6ignored, impronte
invariate dopo ciascun comando. Allfeatures e gate successivi ancora in corso
nel ledger07; nessunDONE o CI07 già eseguita. CARGO_BUILD_JOBS=2documentata
come sola impostazione di compilazione, senza modificare i test eseguiti.


## Qualificazione locale finale

Undici comandi exit0,468impronte prima/dopo identiche e ricontrollate a fine
sequenza.981workspace standard,981allfeatures e938core senza bundle:
0fail/6ignored ciascuna. Fmt/Clippyworkspace e tooling positivi; release
positiva;18tooling inclusiignored e24Python positivi, boundary positivo.
I conteggi dei log sono stati ricostruiti e confrontati col ledger;16log
con inventarioSHA256 preservano anche i preliminari. Nessun gate locale
Windows attribuito. CARGO_BUILD_JOBS=2e LINTRA_TEST_BIN documentate nel ledger.
Nessuna modifica sorgente durante l'intera sequenza.

Pubblicazione e CIpropria Linux/MSVC ancora necessarie primaDONE07; gli esiti
06non vengono attribuiti a questo delta. Nessuna release/tag nuova.


## Quality Gate finale

CI42390593313completed/success su6477bb28165026236933e9aeef70478350ab97d5.
Sette job richiesti positivi:Linux/MSVC,3confronti e2riapertureSQLite crossOS.
PackagingUI opzionale skipped. Fonte completa T-CAD-07-native-ci.json, ultima
rispostaAPI conservata separatamente. Task07DONE,136/140complessive.
