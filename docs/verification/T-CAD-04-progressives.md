# T-CAD-04 — progressivi ed evento, 6 novembre 2026

DONE. Implementazione finale con15test CAD04. CI42368062138 success sullo SHA
b59a5b905ae9f82baf5ce23bbc4eb8564fa5cd22, tutti7job richiesti positivi.
I risultati locali della base e del successivo affinamento sono distinti. CAD09 è già DONE sulla CI42231182241;134/140 task completate,6aperte.

## Contratto e confine

Fonti e prerequisiti nel preflight04. La norma collega il progressivo all'evento
per ULSS e tracciato, anche dopo annullamento. Gli attributi possono cambiare;
la ricomparsa dello stesso evento è ammessa. Le PK completeC/D restano tali.
Il confronto sul progressivo viene selezionato esplicitamente e lascia invariato
il confronto preesistente sullaPK completa.

La consegna qualifica il componente di proiezione, confronto e approvazione.
Gli outcome nei test sono fixture pubbliche dichiarate, come nella task03;
non attestano un'esecuzione clinica Full del flusso regionale. Compiler, chiavi
canoniche, HMAC, SQLite, fonti, diff, riaperture e transazioni sono reali.
Il compiler ordinario e il produttore storico audit-only continuano a rifiutare
il profiloCAD completo. Nessuna policy viene eliminata per aggirare quel limite.

## Compilazione delle invarianti

`persistence_profile::invariants` risolve trace, campi del progressivo e campi
dell'evento. Le tuple devono essere non vuote, prive di duplicati e riferite a
campi dichiarati; gli identificatori non possono essere vuoti o padded. Il codice
di riassegnazione deve essere un `ValidationCode` valido. LaPK del tracciato viene
registrata senza sostituirla con il progressivo.

Il primo perimetro supporta una sola invariante per trace, stessa versione di
schema e riattivazione dello stesso evento esplicitamente ammessa. Gruppi di
compatibilità non qualificati, flagfalse e invarianti multiple vengono rifiutati
prima dell'input. I piani temporali rimangono unsupported. È richiesta la
capability di confronto persistente già presente nell'authoring ratificato.

`CanonicalKeyField` contiene soltanto ID, range verificato, allineamento e missing
policy. È un metadato di estrazione, non un campo clinico parzialmente validato.
`canonical_fields` converte la dichiarazione fisica in quei dati: numeric eAN
riusano la conversione dell'allineamento; date e decimal usano l'Exact già previsto
dal contratto. `source_range` verifica conversioni e overflow. Le missing policy
restano tipizzate e byte-esatte.

`IdentityProjection` conserva lunghezza della riga,PK e l'eventuale coppia di
proiezioni secondarie. `identity_projections` produce questi metadati solo per
profili che dichiarano invarianti; i profili precedenti mantengono il percorso
abituale. `CompiledPersistenceProfile` ne resta l'unico proprietario compilato.

## Chiavi canoniche e protezione

`build_join_key_from_components` centralizza il protocollo esistente: lunghezza
u64 big-endian e byte originali per ogni componente. Due traversate immutabili
calcolano lo spazio e riempiono il buffer senza clonare i valori. Una lista vuota
è un errore; un valore logico vuoto conserva il proprio frame. Conversioni,
somma delle lunghezze e allocazione rimangono controllate. `build_join_key` usa
lo stesso helper, così i due percorsi non possono divergere nel formato.

`ProjectedHistoricalRecord::project` richiede una riga della larghezza compilata
e una posizione positiva. EstraePK, progressivo ed evento dalla stessa riga
in prestito. La missing policy si applica prima della conversione logica, usando
gli stessi helper dell'engine; niente trim o normalizzazione aggiuntivi.

Si ottengono unaPK protetta, un'identità sul progressivo con fingerprint della
riga e un'identità sul progressivo con fingerprint dell'evento. I key_kind sono
distinti e separati dal contesto HMACv1 esistente. Il digest della definizione è
legato al profilo completo; lo scope contiene namespace, flusso, versione e trace.
Anno e trimestre appartengono alla sessione, non all'identità continuativa.

La struttura non ha costruttore da token, setter o deserializzazione. Conserva
solo valori protetti e origine diagnostica in memoria. I byte della riga non
sopravvivono alla proiezione; Debug è redatto. Il backend riceve gli stessi
`StoredIdentity` opachi già supportati dal codec e dagli indici, senza formato
storage, schemaHMAC o gestione chiavi nuovi.

## Passaggio in Validated e preview

Il metodo precedente `ApprovalDraft::validated` rifiuta i nuovi profili con
invarianti: una lista arbitraria di token non prova che gli indici necessari
siano presenti e coerenti. `validated_with_identity_records` accetta solo le
proiezioni indivisibili dello stesso profilo. Le ordina per trace/linea, rifiuta
origini ripetute, verifica conteggi fisici e numero di token per ruolo. Una trace
assente non può contenere nuove righe proiettate. Gli offset interni delle
evidenze vengono costruiti con somma controllata.

`validated_inner` mantiene i controlli dell'outcome pubblico, parametri, consenso
e metadati. I key_kind secondari sono ammessi solo sul trace che li dichiara;
scope, versione e definizione devono corrispondere al profilo. Ogni token viene
autenticato prima che uno snapshot possa raggiungere lo storage. Le identità
duplicate sono rifiutate, senza scegliere una riga o fonderne gli attributi.

`InvariantValidatedApproval` mantiene un prestito al verificatore e i riferimenti
alle origini; non copia la chiave. La preview verifica prima la chiave pubblicamente
confermata dall'archivio, poi usa l'immutabilità/idempotenza del coordinatore
esistente. Un retry esatto già approvato riusa la ricevuta.

Per un nuovo candidato, la preparazione dell'audit qualifica fonti, protezione,
continuità dei metadati, definizioni e completezza degli indici. Una versione
incompatibile o un indice incompleto non può diventare un reset della storia.
Il numero di generazione osservato deve coincidere con la base della preview.

Ogni binding dell'evento viene cercato tramite il suo indice protetto. La fonte
deve essere approvata, autenticata, nel profilo/versione/hash corretti e non oltre
la generazione fissata. Le definizioni dei fingerprint devono essere confrontabili.
Solo un fingerprint dell'evento differente diventa riassegnazione normativa.
Cancellazione e generazione vengono controllate tra le letture e alla fine.

La prima riassegnazione restituisce `InvariantRejected(Box<ValidationError>)`,
in ordine di origine. Il codice viene dallo schema; il numero di riga viene
dall'input fornito al proiettore. Non si inventa una posizione da un digest e
non si accumulano tutti gli errori in unaVec. Il diagnostico di record non
contiene valori personali o identificativi in chiaro. Gli errori di contesto
non qualificabile usano `HistoricalContext`, separato dagli errori normativi.

La preview restituita mantiene il coordinatore esistente: report prima del
commit, CAS sulla base verificata, gestione del commit ambiguo e annullamento.
Una contesa dopo la verifica dei binding non può pubblicare sulla base vecchia.

## Diff e contatori

`HistoricalAuditService::new` continua a selezionare `cross_run_audit/record_key`.
`for_identity_invariants` seleziona esplicitamente le identità sui progressivi.
Gli elementi diff riportano il loro key_kind; la semantica non è dedotta da
nomiCAD o daID campo. I conteggi inserito/modificato/scomparso/riattivato riusano
l'algoritmo e le fonti storiche già qualificati.

Il controllo degli snapshot separa il numero di righe fisiche dal numero di
token. Per un trace con invariante i tre ruoli devono essere bilanciati; se il
file è presente, ognuno deve corrispondere al conteggio fisico. Per una trace
assente gli eventuali token conservati devono restare bilanciati. Gli alias
hanno definizioni legate al profilo. Il diff filtra il ruolo selezionato e
continua a ignorare l'assenza di un intero file come fonte di annullamenti.

Non viene introdotta qui la riconciliazione cumulativa del contenuto corrente:
la preservazione completa del candidato resta a05 e al gate integrato12.

## Componente e tentativi iniziali

`2024-06.progressives.1` conserva20campi alle coordinate originali, copie legacy,
hash ePK complete. Per gli attributi mutabili usa2.3,3.4,4.7,5.6. Il dominio
clinico5.5 rimane al componente lookup già qualificato. Le regoleGit preservano
i byte dei componenti citati dalla provenienza, verificati anche dai test.

Il primo test ha rifiutato un attributo legacy `raw` superfluo nelle missing
policy: conversione canonica corretta, copie originali conservate. Il secondo
tentativo ha usato open su un archivio non ancora inizializzato: la fixture ora
esegue create esplicito, senza cambiare lo storage. Due test comportamentali
sono poi passati, incluse riaperture e rifiuti dopo annullamento. La prima
compilazione della suite ampliata ha segnalato due parentesi in eccesso nelle
asserzioni; corrette. Tutti i tentativi restano distinti dai gate finali.

I test ampliati coprono anche confrontoPK/progressivo, isolamento ULSS/trace,
continuità fra anni, file assenti, proiezioni mancanti o miste, indici incompleti,
chiave/versione incompatibili, cancellazione, reportfailure, CAS, origine,
JSONL/privacy e limiti dichiarati del compiler. Esiti completi ancora in verifica.


## Affinamenti verificati

La suite ampliata ha richiesto le API effettive ReportError::Write e
Sha256Digest/Display per l'output SHA2 corrente; errori di fixture corretti senza
modificare le librerie. Tutti11casi ampliati sono poi passati. Aggiunti retry
esatto di un vecchio trimestre dopo aggiornamenti e rinomina completa di flusso,
trace, campi e codice di errore: anche questi passati. Regressione complessiva
su cad_progressives/join_key/persistence_profile/session_approval/history_audit:
57pass/0fail/0ignored. Fonte PDF, cinque hashtracce,20definizioni di campo ePK
complete confermati in T-CAD-04-source-check.json. Gate completi ancora necessari.


## Sequenza finale avviata

Ledger T-CAD-04-local-gates.json, log T-CAD-04/gate-N.log. Gli input includono
.gitattributes, authoring, sorgenti/core/UI, schemi, fixture, script, manifest e
lockfile. Ogni comando verifica nuovamente le impronte; al termine si richiede
uguaglianza completa prima/dopo. I primi2comandi sono conclusi con exit0.
La suiteworkspace è in corso. Nessuna modifica di codice o indiceGit durante
la sequenza. I tentativi preliminari restano distinti dai gate, con log preservati.

## Affinamento finale individuato durante i gate di base

Da qualificare prima di pubblicare: il medesimo progressivo su due eventi nello
stesso candidato deve produrre il codice normativo dichiarato. Il percorso
corrente arriva al controllo delle identità secondarie duplicate e restituisce
un errore strutturale. Il test atteso deriva dalla stessa regola progressivo–evento
ed è preparato fuori dall'albero sorgente mentre la suite di base resta congelata.
Dopo il completamento della sequenza: riprodurre il test negativo sulla base,
correggere la verifica dei binding del candidato prima dello snapshot, ripetere
regressioni e gate appropriati sul delta, quindi qualificare l'intero codice
finale in CI Linux/MSVC. I gate della base non verranno attribuiti al sorgente
modificato; nessuna pubblicazione oDONE prima della correzione verificata.

## Risultati della base congelata, in attesa dell'affinamento

Fmt eClippy exit0; workspace957pass/0fail/6ignored; allfeatures957pass/0fail/
6ignored; core senza bundle914pass/0fail/6ignored. Input invariati per ciascun
comando. Release/tooling ancora in corso all'ultima lettura. Questi conteggi
riguardano la base con13casi CAD04: il caso aggiuntivo del candidato corrente
richiede ancora riproduzione, correzione e qualificazione separata sul delta.

## Correzione del binding nel candidato corrente

Il nuovo test è fallito sulla base con zero diagnostici normativi: il conflitto
raggiungeva DuplicateIdentity. La correzione aggiunge `check_current_bindings`
dopo la verifica delle proiezioni e dei conteggi, prima dello snapshot. La mappa
prende in prestito le identità protette dal vettore immutabile e conserva solo
gli indici: non copia righe o identità. Non viene mai iterata per ordinare errori;
l'ordine resta quello delle origini già ordinate. Il suo spazio è rilasciato
prima del controllo di unicità dello snapshot.

Su una collisione autentica entrambi i token con il verificatore prima di
confrontare i fingerprint dell'evento. Eventi diversi producono il codice dello
schema sulla seconda origine; la duplicazione sullo stesso evento resta un
candidato ambiguo rifiutato. `reassignment_error` è l'unica costruzione del
diagnostico, condivisa con il controllo storico. Profilo, namespace, flow e trace
delle proiezioni vengono verificati prima del confronto; i conteggi fisici sono
controllati prima di chiamare l'helper. Nessuna nuova norma, firma pubblica o
forma di storage è stata aggiunta dall'affinamento.

Tutti14casi sono passati dopo la correzione, incluso il caso fallito prima del
fix. Aggiunto un ulteriore controllo per token non autenticabili e conteggi
incoerenti: devono rimanere errori tecnici, prima della classificazione normativa.
Il delta finale viene qualificato con fmt/Clippyworkspace, suitecomplete del core
con/senza bundle, release e boundary. La base locale completa rimane distinta;
la nuova CI sul commit finale comprende nuovamente l'intero workspace Linux/MSVC.

## Stato della qualificazione finale del delta

Il test aggiuntivo è stato effettivamente riprodotto in fallimento prima del fix.
Dopo il fix tutti14casi sono passati; aggiunta la verifica di precedenza per
protezione e conteggi, tutti15casi passati. Fmt eClippyworkspaceallfeatures della
sequenza finale sono già positivi con input invariati. Suitecomplete del core
con/senza bundle, release e boundary sono in corso nel ledger separato
T-CAD-04-final-refinement-gates.json. I gate della base completa rimangono nel
ledger T-CAD-04-local-gates.json, etichettati con il loro perimetro precedente.
La nuova CI completa e lo SHA finale non sono ancora pubblicati.


## Verifica di ripresa — 7 novembre 2026

Entrambi i ledger risultano completi. Verificati tutti i log e i conteggi dei
test contro i ledger, oltre alle impronte delle evidenze della base. Tutti gli
input del ledger finale coincidono con i file correnti; i quattro input diversi
dalla base sono esattamente authoring, historical_identity, session e test04,
modificati dal successivo affinamento documentato.

Base:11comandi exit0,957workspace/default e957allfeatures,914core senza bundle,
18tooling e24Python. Delta finale:6comandi exit0,924core allfeatures e916senza
bundle,0fallimenti/6ignored per suite; fmt, Clippyworkspaceallfeatures, release
e boundary positivi. Quindici test CAD04 compresi in entrambe le suite finali.
I log finali e il loro inventario sono in T-CAD-04-final-refinement/.

I log red/green temporanei citati nella recovery non sono più disponibili in
/tmp alla ripresa: non vengono ricostruiti né dichiarati archiviati. Restano la
registrazione storica della riproduzione e le suite finali effettive conservate.
Nessun gate Windows locale attribuito; nuova CI completa obbligatoria prima DONE.


## Quality Gate finale — 7 novembre 2026

CI42368062138 completed/success sul commit b59a5b905ae9f82baf5ce23bbc4eb8564fa5cd22.
Linux/MSVC, tre confronti JSONL/pixel e due riaperture SQLite crossOS positivi;
packagingUI opzionale skipped, non parte di questo gate. Risultato completo in
T-CAD-04-native-ci.json. Task04DONE;134/140 complessive. Nessuna release nuova.
