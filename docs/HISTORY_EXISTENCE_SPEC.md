> Aggiornamento 9 novembre 2026: le limitazioni operative alla sola specifica e
> i punti decisionali di questo documento conservano il contesto della prima
> proposta. La ratifica implementativa sostitutiva, C1 e L1 li superano nei punti
> esplicitamente ratificati. Contratto adottato e codice corrente sono descritti
> in [implementation-contract](verification/ADR-HISTORY-EXISTENCE-001-implementation-contract.md)
> e [report storage](verification/T-CAD-12-history-storage.md). Nessun DONE implicito.

# HISTORY_EXISTENCE_SPEC — contratto attuativo proposto v1

8 novembre2026. ADR-HISTORY-EXISTENCE-001 A ratificata **per la sola specifica**.
Stato: SPECIFICA VERIFICABILE; implementazione e formato attivo NON autorizzati.
Le scelte materiali ancora aperte sono esplicite in§15. Nessun codice produttivo,
codec, schemaSQL, crypto, migrazione o compilerhistory è cambiato per questo documento.

## 1. Fonti e confine

Vincoli: [ratifica](verification/ADR-HISTORY-EXISTENCE-001-ratification.md),
DEC-05-CAD-VALUTAZIONE-INIZIALE in[norme](normative/DECISIONS.md), ADR-PERSIST-001,
ARCH45, DET166–174 e SCHEMA_AUTHORING Join/Temporal. La query considera storico
approvato qualificato+invio corrente. Non richiede un'iniziale per trimestre,
non cambia norma, cardinalità, PK o significato di annullamento.

Tre responsabilità restano distinte:

```text
compiler: dichiarazione → piano chiuso e dipendenze
engine: singolo run streaming / proiezioni effimere / verità a tre valori
application+repository: base approvata, evidenze persistenti, consenso e CAS
```

Il compiler pubblico continua a rifiutare history_including_current. Le prove
CAD12 con filler non qualificano questo contratto e non diventanoCAD83/FROZEN.
La specifica non autorizza la futura implementazione: servono decisioni§15,
autorizzazione attuativa separata e successivi gate completi.

## 2. Ricostruzione del codice corrente

| Aspetto | Stato osservato | Limite per history.existence |
|---|---|---|
| RawJoinSpecScope/HistoryExistenceV1 | presenti nel DTO | non significano supporto runtime |
| schema/joins.rs::compile_joins | rifiuta scope diverso da current_submission | nessun lowering silenzioso a corrente |
| FlowRecordObserver | stessa riga ePK in prestito durantePASS1 | hook neutro, non autorizza scrittura |
| ProjectedHistoricalRecord | PK, progressivo, evento e fingerprint protetti | non recupera2.8 da HMACriga |
| session::verify_outcome | Full effettivo, inventario/hash/counter presenti, stato valido | Complete/Partial distinti; PROCEED non crea True |
| SourceSubmission/HistoricalSnapshot | provenienza approvata e identità opache | nessun oggetto fatto/copertura esistente |
| SQLite FORMAT | 2, lettura1/2; upgrade1→2 esplicito | nessun formato fatti3 attivo |
| protect_key/protect_projection | HMACv1 scopi e definizioni esistenti | non firma automaticamente un nuovo metadato |
| protect_submission/verify_submission | prova privata sui campiSourceSubmission | nuovi fatti o root di fatto non sono nei campi attuali |
| codec::read_record/read_source | ricostruisce prove private e le verifica | una DTO JSONpubblica non è un'attestazione |
| SQLiteTransaction::commit | snapshot/record/latest e generazione nella stessa transazione | non include nuovi fatti o copertura |

Fonti tecniche: src/schema/{raw.rs,joins.rs}, src/engine/flow_execution.rs,
src/application/{historical_identity.rs,session.rs,history_audit.rs},
src/persistence/{mod.rs,protection.rs,sqlite/mod.rs,sqlite/codec.rs}.
Le impronte correnti sono registrate nel gate documentale del dossier ADR.

## 3. Piano compilato minimo, non API Rust pubblica

Nome logico: HistoryExistencePlanV1. È metadato non forgiabile prodotto dal compiler;
nessun costruttore da bool/token pubblici e nessuna deserializzazione come prova.

| Campo | Vincolo e significato |
|---|---|
| contract_version | 1 del modello logico, distinto da versioneSQLite |
| schema_key / schema_hash | stessa sorgenteRaw caricata una sola volta e canonicalizzata |
| plan_definition | descrittore semantico§4 e digest; non soloRuleID |
| owner_trace / target_trace | riferimenti risolti nello stessoCompiledFlowSchema |
| rule / assertion / join | indici+ID verificati; ownership coerente |
| correlation_source / correlation_target | tuple non vuote, uniche, stessa arità/tipi; frameJoinKey esistente |
| namespace_parameters | binding dello stesso run e namespace continuo, mai trimestre implicito |
| predicate | minimo Eq(LocalFieldRef, literalbytes) su target; no coercizioni |
| dependency_projection | campo operando, tuple/PK, policy di presenza/validità e dipendenze locali necessarie |
| scope | history_including_current obbligatorio, nessun fallback |
| compatibility | prima attuazione: stessa schema version/hash e stesso digestpiano |
| diagnostic_policies | Unknown/violazione e testi dichiarati nello schema; nessun codice clinico nel core |

Sottoinsieme iniziale proposto: set_exists con Eq di un campo locale a un literal
secondo formato/encoding/allineamento espliciti; join a insieme senza riduzione
arbitraria. Nessun first(), query libera, confronto di campi storici generico,
reference lookup dentro il predicato, derived/cross/parametri/predicati dinamici.
Altri nodi restano UnsupportedConstruct prima di storage/input access. Le regole
current_submission preesistenti non cambiano. Il sottoinsieme è sufficiente al
predicato iniziale, senza generalizzare a un motore storico di righe cliniche.

## 4. Identità del predicato e canonicalizzazione

PlanDefinitionV1 include: versionecontratto, schema key/hash, scope, owner/target,
ordine e layout dei campi correlati, namespacebindings, AST Eq/literal, tipo e
encoding, missing/presence/allineamento/lexical/constraint e dipendenze necessarie
alla qualificazione dell'operando, compatibilità e modalità lifecycle ratificata.
ID di regola da solo non basta. Indici eseguibili sono verificati contro gli ID;
non sono un fingerprint indipendente da layout o sorgente.

Bytes = canonical_value_bytes del descrittore tipizzato; digest = SHA256 esistente
schema_fingerprint/canonicalizzazione (nessun nuovo algoritmo). Ordine array
significativo preservato, oggetti canonicalizzati, nessun float/default implicito.
Nessun valore paziente entra nel descrittore: il literal è configurazione normativa.
La semantica lifecycle non ratificata impedisce di finalizzare un digest attivo.
Compatibilità fra versioni differenti non viene dedotta dal nome o da un subset
comune: nella prima attuazione è Unknown senza mappa esplicita qualificata.
Hash del piano e hash dello schema sono entrambi verificati, non alternativi.

## 5. Oggetti logici: assessment, copertura, provenienza

Una mappa evento→bool non è questo contratto. Ogni assessment è relativo a una
riga reale e alla propria versione; più iniziali conservano più witness senza
confonderli con duplicatiPK o fondere le righe.

### 5.1 RowAssessmentV1

Campi obbligatori proposti: fact_format=1, plan_digest, schema_key/hash,
namespace, target_trace, correlation_token, witnessPKtoken, recordFingerprint,
sourceSubmissionFingerprint, snapshotRevision, sourceOrdinal, assessment,
reason quandoIndeterminate, proofbinding. Token e fingerprint sono quelli
protetti e qualificati, mai evento/CF/riga/valoreclinico in chiaro.

assessment è uno di PositiveWitness / EvaluatedNonWitness / Indeterminate.
EvaluatedNonWitness descrive solo quel predicato su quella riga: non è False
sull'intero universo. Missing/invalid/unavailable non diventano NonWitness.
Prova privata verificata e binding al batch§5.3 sono necessari per l'uso storico;
la versione corrente può esistere soltanto in staging fino all'approvazione.

### 5.2 EventCoverageV1

Campi proposti: formato/piano/schema/namespace/entitytoken, observedBaseGeneration,
anchorReference, firstRequiredSession, throughSession, previousCoverageCommitment,
orderedSessionReferences, present/absent pertrace richiesto, counted/qualified/
indeterminate/unlocatable rows, coverageStatus e causegap, provenance/proofbinding.
Numeri non negativi con overflow controllato; somme coerenti, intervalli/sessioni
non duplicati o invertiti. I contatori non sostituiscono gli hashinput effettivi.
Non persistere date cliniche che non servono: il confine temporale di copertura
è la sessione/periodo già approvato, non una copia della dataB1.

coverageStatus: QualifiedComplete / QualifiedIncomplete. Completo è possibile
soltanto con anchor normativo verificabile§6 e continuità dimostrata. Stato
mancante non ha defaultComplete. Non basta un contatore di snapshot letti.

### 5.3 FactBatchManifestV1

Un batch per snapshot/plan/targettrace: formato, schema/piano/namespace,
sourceFingerprint, revision e baseGeneration, stato di approvazione, keyID/scheme
quando applicabile, conteggi delle tre classi, commitment degli assessment e delle
coverage, inventario inputeffettivo, provaautenticazione e identificatorecodec.
Ordine canonico: trace compilato→sourceOrdinal→identitàwitness; la riga di origine
rimane un dato reale. Commitment = SHA256 di sequenza canonicalizzata/framed con
il metodo esistente e conteggi inclusi. NON è autenticazione se la root può
essere riscritta insieme alle righe. La prova delmanifest è un gateSEC§11.

Il batch lega sia i witness presenti sia la completezza dell'insieme. Row proof
valida isolata non prova assenza di altre righe, continuità o copertura completa.
Metadati generali già presenti nellaSource sono referenziati e verificati, non
duplicati liberamente in versioni contraddittorie. Source e batchdevono concordare.

## 6. Universo storico e origine della copertura

Universe(event,plan,base,current) è storico approvato compatibile per stesso
namespace/flow/target/correlazione, entro base pinnata, più staging corrente.
La selezione normativa dei witness validi è disciplinata da lifecycle§8.
La continuità della storia non è continuità dei soli numeri di generazioneDB:
generazioni possono contenere invii diversi, omissioni o sessioni non pertinenti.

Per una prova negativa serve un EventUniverseAnchor qualificato che attesti
l'inizio normativo del perimetro e l'applicabilità della storia fornita. La
prima osservazione nel repository o unB1con data non dimostrano da sole che
nessuna storia antecedente sia mancante. Non si crea l'anchor dal min(revision).

Non esiste oggi un produttore ratificato di tale anchor. Il modello lo rende
esplicito, non lo inventa: in assenza di anchor con prova e policy verificabili,
coperturaIncomplete e risultatoUnknown se manca un positivo affidabile.
Un futuro anchor deve riferire origine normativa, ambito organizzativo/evento,
periodo iniziale, fonte verificabile e relativo trust/authenticationbinding.
Una dichiarazione utente libera o un flagComplete non è sufficiente.
OPEN-COVERAGE-01§15 definisce il gate ancora da risolvere prima di abilitareFalse.
Le fixture di modello possono specificare un'anchor esplicitamente ipotetica;
non sono prove di esistenza di una fonteoperativa già approvata.

Dall'anchor alla sessione richiesta ogni sessione normativa applicabile deve
avere source approvata verificata e coverage del target. Salto di trimestre,
tracciaAbsent, batch mancante, schema/predicato incompatibile, righe nonlocatable o
operandiUnknown aprono gap. Un inputPresent(0) qualificato copre la propria
sessione ma non retroattivamente lo storico. Absent non è zero e conserva la
baseline secondo i contrattiCAD, senza dimostrare cosa mancasse nella sessione.
Una sessione assenteDB non si inventa daGenerazione+1; non è Complete.
Snapshot cumulativo può fornire la vista attiva della sessione, non cancellare
implicitamente lacune cronologiche o creare una storia dalla nascita dell'evento.
PROCEED conserva l'inventario/Partial; non autentica né chiude i gap di copertura.

## 7. Query: ordine delle verifiche e tre valori

1. Preflighttecnico: chiave/audit/codec/profilo/namespace; rifiutare predicato non
   supportato prima dell'accesso ai dati. Pinnare generazione e selezione fonti.
2. Preparare staging dal run effettivo; qualificare separatamente tuple e operando.
   L'errore locale indipendente non annulla automaticamente un witness corrente,
   ma il run invalido non può pubblicare nuovi fatti approvati.
3. Verificare integrità e compatibilità delle prove effettivamente usate e validità
   lifecycle. Una prova PositiveWitness autonoma valida, storica o corrente,
   permetteTrue anche con altri gap. Un guasto tecnico osservato non viene mascherato.
4. Senza positivo: False solo se anchor, coverage di tutte le sessioni/target e
   stagingcorrente sonoQualifiedComplete e ogni assessment pertinente è determinato.
5. AltrimentiUnknown con causa qualificata; Error per fallimenti tecnici. Controllare
   la generazione di nuovo prima di completare preview e alla pubblicazioneCAS.

| Evidenza | Copertura | Risultato |
|---|---|---|
| witness autonomo valido | incompleta o completa | True |
| nessun witness, tutte le righe valutate | completa con anchor verificata | False |
| nessun witness | nessuna anchor o gap | Unknown |
| soli witness incompatibili/validità indecisa | non qualificabile | Unknown |
| erroreI/O/corruzione/provaautenticazione invalida/chiave mancante | qualsiasi | errore tecnico |

L'assenza restituita daSQLite è solo un dato di query. Non è il verdetto normativo.
Un predicato obbligatorioUnknown non può attestareComplete, non può essere omesso
né disattivato. Conservare cause ePartial. L'eventuale consenso autorizzato continua
nel coordinatoreesistente, non crea verità; gate di approvazione del futuro percorso
storico obbligatorio deve essere esplicito e non un fallbackaudit-only§10.

## 8. Lifecycle e validità, senza nuova norma implicita

| Transizione | Comportamento definibile oggi |
|---|---|
| nuovo witness corrente qualificato | usabile inpreview, non persistito prima approval |
| nuova rivalutazione senza invalidazione del witness precedente | non cancella un positivo storico valido |
| record invariato / retry approvato esatto | stessa provenienza; nessun doppio fatto o incremento generazione |
| modifica di attributo non pertinente al predicato/correlazione | non scarta automaticamente la prova indipendente; verificare definizione e bindingversione |
| namespace/evento differente per stesso progressivo | rifiuto esistente04; non sposta una prova fraeventi |
| tracciatoAbsent | niente annullamenti dedotti; gapcopertura e baseline preservata |
| tracciatoPresent(0) / progressivo assente da filecompleto | annullamento record secondo04/05; effetto sul predicato storico richiedeLIFECYCLE-01 |
| correzione stesso progressivo che cambia1→2 sull'operando | stato record aggiornabile già ratificato; validità della vecchia prova richiedeLIFECYCLE-01 |
| riattivazione stesso evento | transizione record già consentita; nuova prova derivata dalla versione corrente, non ripristino cieco di un bool |
| sessioneimmutabile / tentativo riscrittura | E_APPROVED_SESSION_IMMUTABLE; nessuna mutazione dei fatti |

I record storici approvati e la loro provenienza restano immutabili; validità di
una prova e conservazione materiale del vecchio snapshot sono concetti distinti.
Fatti non consultabili se evidence invalidata secondo un contratto applicabile.
Non decidere se l'annullamento renda invalida anche la prova di "mai esistita
iniziale": il contratto esistente non lo specifica. OPEN-LIFECYCLE-01 è un reale
punto da risolvere prima del codice, non un nuovo defaultTrue/False delrepository.

## 9. Formato, indici e versioni proposti — nessun codec attivo

Inventario effettivo: sqlite/mod.rs constFORMAT=2; reader accetta1/2 e upgrade1→2
è un'operazione esplicita separata. Domainwire/protezione hanno versioni proprie.
Nessuna versione3 attiva osservata nel registro/codice corrente alcommitbase.

Proposta: fact_format1, plan_definition1, batch_manifest1; SQLite fisico3 è
**candidato non assegnato/attivato**, da ricontrollare al momento dell'eventuale
implementazione autorizzata. Non incrementare FORMAT ora e non riutilizzare2
per payload di significato differente. Lo snapshot/domainwire vigente resta tale
finché un disegno approvato non ne specifichi l'eventuale estensione esplicita.

Codec proposto: JSONUTF8 tipizzato, proprietà sconosciute e duplicate rifiutate,
interi con limiti/overflow controllati, digest32byte in forma canonica verificata,
nessun float/default permissivo. Bytes canonicali definiti dalle funzioni§4;
ordine delle liste esplicito. Una serializzazione pubblica esportata non porta
una prova privata utilizzabile dal repository. Dopo decode verificare tutti
binding e commitment prima di restituire dati al chiamante.

Proposta di layout logico, NON DDL da eseguire:

| Tabella proposta | Chiave / relazione | Indice di query |
|---|---|---|
| history_batches | namespace,revision,plan; FKsnapshot; manifest/proof/counts | snapshot e definizionepiano |
| history_assessments | namespace,revision,plan,sourceOrdinal; FKbatch e originrecord | namespace,plan,entitytoken,revision,sourceOrdinal |
| history_coverage | namespace,revision,plan,entitytoken; FKbatch/anchorref/previouscoverage | namespace,plan,entitytoken,throughSession |

Payload/prove vincolati ai metadati delle chiaviSQL: una chiaveSQL non attestata
non può reindirizzare una prova valida ad altroevento. Nessun valoreclinico in
colonne consultabili. Token/PK di correlazione protetti nel contestoHMAC vigente,
nessuna concatenazione senzaframe o chiaveevento globale. Tabelle vecchie e
nuove appartengono alla medesima transazione; mai sidecar aggiornato separatamente.
Anche il numero righe/insieme deve essere autenticabile§11, non solo FK/countSQL.

Indici persistenti sono acceleratori ricostruibili solo da batch già autentici,
non fonte della verità. Letture pernamespace/plan/entity e revisionepinnata, pagine
limitate senza caricare tutto lo storico o tutte le righe cliniche. Costo query
proporzionale alle evidenze pertinenti, non a tutta la popolazione; nuova qualifica
memoria/performance necessaria, nessun numero benchmark attribuito oggi.

## 10. Applicazione, staging, approvazione e atomicità

Percorso futuro distinto dall'audit-only: compiled prodotto dello stessoRaw,
parametri enamespace verificati, baseGenerationpinnata; input letto una sola volta.
PASS1produce evidenze correnti private; PASS2può usare fatti qualificati della base
senza renderliContextStore persistente o cambiare l'ordine diagnostico ratificato.
Fatti correnti non approvati non sono condivisi con altri run/sessioni/namespace.

Gate proposto per dichiarare conformitàcompleta: nessuna violazione, nessuna
regola obbligatoriaUnknown, niente cancel/truncated, proiezioni/input audit integri,
copertura pertinente completa ove necessaria pernegativi; True indipendente
non richiede chiusura dei gap non necessari alla prova. Complete/Partial del run
rimangono visibili. Non usare il gate audit-only preesistente come scorciatoia;
non cambiare il coordinatore07 o i suoi consensi con questa specifica.

Dopo preview, nuova approvazione scrive SourceApprovedImmutable, snapshot,
record, batch/assessment/coverage e aggiornamento generazione nella stessa
transazioneImmediate conCAS sulla base osservata. Il receipt conferma l'intera
unità, non solo una tabella. Callbackreport deve completare prima della transazione
come nel coordinatorevigente. ABORT/cancel/reportfailure prima delcommit non
pubblicano fatti. ErroreCOMMITambiguo rimaneCommitOutcomeUnknown finché una
riapertura verifica source+snapshot+manifest completi con stesso fingerprint;
mai retry cieco, seconda generazione o receipt su fatto mancante.

Crash primacommit→nessun nuovo stato, dopo commit riuscito→tuttaunitàleggibile,
corruzioneparziale→erroretecnico. Una preview non approvata non è una fonte query.
Un'altra approvazione sulla stessa base invalida ilCAS; cancellazione dopo un
commit realmente avvenuto non finge rollback. Politiche SQLiteDELETE/EXTRA,
ACL/link/ownership/location e keyprovider restano quelle qualificate.

## 11. Integrità: garanzie disponibili e gateSEC ancora aperto

**Disponibile oggi:** protezione scoped delle identità e fingerprintHMACv1;
prove private record/source ricostruite e verificate dalcodec; confrontiCAS,
provenienza Source e garanzie filesystem qualificate. Questo non autentica una
futura tabella, un fattoPositiveWitness, un'originecopertura o un commitment nuovo.
SQLite/chiaviSQL/digestSHA256 da soli non forniscono tale autenticità.

Minacce da coprire nel futuro formato: modifica/aggiunta/eliminazione assessment,
scambioentity/namespace/piano/schema/sessione; replaybatch ad altrarevisione;
rimozione di un witness accompagnata da countSQL cambiato; falsificazioneanchor/
coperturaComplete; sostituzioneroot con le nuove righe; indice che puntaalbatch
sbagliato; deserializzazione diDTOpubblica come prova; firma nonverificabile.

Obbligo tecnico proposto: manifest sigillato deve impegnare tutti i campi§5.3,
rootassessment/rootcoverage/contatori e sorgenteapproved/basegeneration. Le prove
leaf devono legarsi alla root e al contesto esatto, oppure il lettore deve
verificare interamente il manifest prima di usare querynegative. Non basta
verify_identity su un token o verify_submission su una Source senza quelcommitment.
VerificaRoot prima diFalse e prima di dichiarare un witnessstorico attendibile.
Una alterazione rilevata èerroretecnico, non Unknown e nonassenza.

SEC-HIST-01: proposta di riutilizzo **esplicito, da ratificare separatamente**
della primitiva protect_projection esistente per un envelope tipizzato di
manifest/coverage (descrittore canonico+scope/definition binding), mantenendo
identici HMACv1/keyprovider e tutti i byte/protocolli degli oggetti precedenti.
Il lettore dovrebbe ricostruire la proiezione canonica ed eseguire un confronto
verificato alla prova/fingerprint, non fidarsi della sempliceDTOserializzata.
È un nuovo impiego del materiale segreto su oggetti differenti: non si considera
già autorizzato da questa ADR. Nessun nuovoMacpurpose/derivazione/chiave viene
introdotto ora. Se la revisione crittografica non consente tale riuso, altra
soluzione richiede decisione separata, non un fallback di sicurezza.

Fino alla chiusuraSEC-HIST-01: prova futurefacts non disponibile, autenticità del
nuovo formato NON qualificata, nessun encoder/decoder o esecutore attivo. La
specifica indica che cosa deve essereautenticato e comeverificarne ilbinding,
non dichiara implementata una costruzione nonratificata.

## 12. Compatibilità, errori e rollout

| Caso | Trattamento del futuro percorso |
|---|---|
| SQLite1/2 noto, leggibile e coerente, senza fatti richiesti | legacy-gap/Unknown; non ricostruire2.8 dafingerprint; vecchi usi non cambiano |
| versione fisica/fatto sconosciuta doveobbligatoria | errore tecnicoUnsupportedStorageVersion/codec; nessun default |
| schema/predicatocompatibile esatto, proof valida | valutabile secondo copertura/lifecycle |
| significato predicato diverso o mapping nonqualificato | ignorarecomeprova, gapsemantico/Unknown in assenza di positivo autonomo |
| proof invalida, root incoerente, I/O, chiave mancante | erroretecnico; nessuna falsaCompletedValid |
| intervallo/traccia/anchor non qualificati ma archivio coerente | Unknown motivato, Partial e gateapplicativo§10 |

Nessuna migrazione automatica o tentativo di "riparare" metadati. Apertura vecchio
archivio non assegna factsformat1 né coperturaComplete. Ricostruzione da inputoriginali
sarebbe nuovo perimetro esplicito con provenienza e gate, non parte di questaratifica.
Rollout futuro: version registryricontrollato→decisioni§15→autorizzazioneattuativa→
implementazioneisolata→test/crash/privacy/performance→CI Linux/MSVC/crossOS→release
soltanto se separatamente autorizzata. Nessuna nuova chiaveoperativa o PHI nei test.

## 13. Matrice dei test pianificati — non PASS runtime

Ogni riga è un'accettazione futura; gli scenari presuppongono solo ancoraggi
sintetici esplicitamente qualificati nel modello, non proveoperativeinventate.

| ID | Scenario e oracle indipendente | Atteso |
|---|---|---|
| HX01 | T1inizialevalida, T2/T3sole rivalutazioni, witnessT1ancora valido | Trueanche senza nuova iniziale |
| HX02 | solo witness corrente valido, storia incompleta, noncommit | Trueinpreview, nessun nuovofatto sualtroreader |
| HX03 | nessun witness, anchorautentica+sessioni complete+corrente completo | False |
| HX04 | indicevuoto con anchorassente oppure salto sessione | Unknown, nonFalse |
| HX05 | dueiniziali, ordineinserimentoinvertito | Trueidentico; PKfisiche indipendenti |
| HX06 | stesso eventotestuale, ULSS501/502 | isolamento totale |
| HX07 | witnessanno precedente, trimestreanno successivo | True secompatibile/valido |
| HX08 | errore locale non pertinente vsoperando/correlazione invalidi | witnesscorrente indipendente vsUnknown; nessuncommitrunInvalid |
| HX09 | targetPresent(0) controAbsent, PROCEED/ABORT | coperturasololocale vs gap; noFalse daomissione |
| HX10 | correzionepredicato / annullamento / riattivazione | oracleLIFECYCLE-01, gateaperto finoratif. |
| HX11 | schemaversion/hash/predicato/formato diversi | incompatibilità nondiventaNone/False |
| HX12 | indicebatchlegacy assente / codec99 obbligatorio | Unknown vs erroretecnico distinti |
| HX13 | manipolazione root/row/coverage/count/scope/source/replay | erroreintegrità, nessunverdettoFalse |
| HX14 | committutto/reopen e receiptidempotente | stessaunitàapprovata e provenienza |
| HX15 | sink/reportfailure, cancel, ABORT e CASfallito | zero fatti visibili/zero generazioniaggiunte |
| HX16 | crash primadopoCOMMIT e COMMITambiguo | rollback o unitàcompleta, nessun successo inventato |
| HX17 | vecchio snapshot passato per live / sessione non approvata | escluso dallaquery; replayfuori contesto rifiutato |
| HX18 | scannerrighe/CF/operandi/segretinelDB/log/backupcopiaqualificata | niente dati raw o campi nonnecessari |
| HX19 | crescita namespace/eventi/storia, query e cursor paginati | misureRAM/p95/IOreali, niente scansioneclinica integrale |
| HX20 | Linux/MSVC, JSONLbyte/pixel e due trasferimentiSQLite | gate nativo delnuovocodice, non ereditato daCAD08 |
| HX21 | presenza delnuovocontratto senza esecutoreautorizzato | compilerrimaneUnsupportedConstruct |

HX21 possiede già una prova di componente nel groundworkCAD12, che qualifica
il rifiuto attuale, non il futuro supporto. Nessuno degli altri scenari è dichiarato
superato da questa matrice. Documentalgate delmodello e verifiche unitàesistenti
saranno riportati separatamente, senza simulareSQLitefactsautenticati.

## 14. Gate documentale e futura implementazione

Verificare completezza di§1–15, tracciabilità delle condizioni ratificate,
chiavi/counter/binding senza defaultpermissivi, coerenza matriceverità/lifecycle,
registroversionireale, classificazionecause, no dichiarazione diMACnuovogiàesistente,
limiti test e nessunfileprodmodificato. Improntare fonti corrente e verificare
che il compilerhistory sia ancora rifiutato; link e riferimenti locali validi.

Filefuture (nessuno modificato ora perhistory): piano compiler/closure; modello
logico evidence e query application; hookstaging attuale senza rilettura; envelope
codec/storage/index/transazione autorizzati; qualificatoredellacopertura e fonte
anchor; preflight legacy/errori; integrazioneingresso pubblicononforgiabile;
testfactory/copyprivacy/performance/native. Nessuna nuova dipendenza prevista.
Non basta cambiare una capability o passare una policy al compile_scoped privato.

## 15. Decisioni aperte e handoff del limite autorizzato

- **OPEN-LIFECYCLE-01 (normativa materiale):** un'iniziale approvata poi annullata
  per progressivoassente daB1completo conta ancora nello storico? E una correzione
  stesso progressivo2.8=1→2 invalida quelwitness? Gli snapshot restanoimmutabili e
  il record corrente cambia secondo04; queste fonti non decidono il valore del
  predicato sull'intero storico. Prima delcodice occorre scelta esplicita; non
  vengono riaperte "storico vs trimestre" o l'annullamento esistente dei record.
- **OPEN-COVERAGE-01 (origine/trust):** manca una fonte ratificata dell'anchor
  normativo dalla nascita dell'evento. Specificatoilcomportamentofail-closed:
  senzafontequalificata nienteFalse, soloTrue autonomo oUnknown. Nessuna
  ricostruzione/migrazione/importautomatico autorizzati. Origine e autenticazione
  dell'anchor richiedono decisione prima dell'attivazione dei negativi completi.
- **SEC-HIST-01 (uso del segreto):** specificato l'envelope daautenticare;
  nessuna provaattiva. Riuso tipizzato di protect_projection pernuovimanifest
  richiede ratifica separata; HMACv1precedente, provider ebyteprecedenti invariati.
- **IMPLEMENTATION-AUTH-01:** anche risolvendo le questioni sopra, questa ratifica
  termina alla specifica; codec/storage/historyexecutor richiedono autorizzazione
  successiva esplicita e propri gate, non prosecuzione automatica oltre lo scope.

CAD08DONE invariata con i suoi gate effettivi. CAD12resta aperta, nonCAD83/FROZEN;
P18-04nonREADY finché tutti i prerequisiti sono effettivi. Risoluzione CNFordine
separata e autorizzata: può avanzare autonomamente senza implementarehistory.
La specifica dichiara obblighi, limiti e punti decisionali; non assegna garanzie
crittografiche aSQLite o a un tokenidentità oltre quanto realmente qualificato.
