# history.existence.v1 — contratto tecnico adottato, 8 novembre 2026

Ratifica implementativa sostitutiva ricevuta: questo contratto precede il codice
e non costituisce una nuova richiesta di approvazione ordinaria. Il modello
esteso HISTORY_EXISTENCE_SPEC.md rimane riferimento di analisi; il precedente
stop alla sola specifica è superato. Nessuna nuova norma, crypto o migrazione.

## 1. Piano e produzione

Primo sottoinsieme: join history_including_current a insieme e assertion set_exists
con Eq su un solo campo target locale e literal codificato secondo il compiled.
Rifiuto anticipato di AST, tipi, scalarizzazione o riferimenti non supportati.
Il compiler lega schema/hash, trace, join/correlazione/namespace, operando/presenza/
validità locale, versione/digest semantico e dipendenze. L'identità del piano
usa la canonicalizzazione JSON/SHA256 esistenti; il nome della regola non basta.
Nessuna conoscenza CAD nel core. Percorsi current-only preesistenti invariati.

Un producer applicativo sul FlowRecordObserver vede le stesse righe di PASS1,
produce assessment temporanei minimi e conserva l'origine reale per diagnosi;
non rilegge input e non duplica righe cliniche. Operandi/correlazioni invalidi
producono indeterminatezza, non falsi negativi. Errori locali non pertinenti non
cancellano un positivo autonomo, ma un run invalido non pubblica fatti approvati.
PASS2 riceve un contesto storico neutro qualificato, distinto dal ContextStore.

## 2. Fatti, copertura e fonte

HistoryFactBatchV1 contiene piano/schema, namespace, trace target, source
fingerprint/revisione/base, assessment riferiti a identità protette e digest di
riga, copertura e commitment dell'insieme. Il batch è un oggetto compilato/
applicativo, non una DTO deserializzata che possa dichiararsi già verificata.

Assessment: PositiveWitness, EvaluatedNonWitness o Indeterminate(causa). Un
NonWitness riguarda una riga, non l'intero storico. Le chiavi di correlazione
usano JoinKey con frame canonici e IdentityProtector::protect_key con scope
esplicito; non evento globale, trim, numerificazione o hashing non keyed.
Nessun valore clinico dell'operando, evento raw o riga completa nel payload.

La provenienza referenzia SourceSubmission approvata e la sua impronta, piano,
compatibilità esatta iniziale, PK e record fingerprint protetti. Il riferimento
persistente a un record è un ordinal del token records, non un numero di riga
clinico inventato. La linea reale originale rimane in staging per i diagnostici;
nessuna coordinata di campo viene derivata casualmente da un witness storico.

Copertura distinta da assessment: UnknownOrigin/Incomplete(cause) oppure
ProvenComplete(anchor, intervallo, sessioni e inventari qualificati). Nessun
costruttore default Complete; prima sessioneDB, indice vuoto o elenco delle
sole sessioni note non provano origine normativa. Traccia Absent, input non
qualificabile o sessione mancante mantengono gap. PROCEED non chiude un gap.
C1 A ammette l’attestazione qualificata d’origine (fonte/hash/responsabilità/
approvazione e binding autenticato allo snapshot). L1, precisata il 9 novembre,
invalida il witness annullato: la vista qualificata del lifecycle è autorevole,
non l’insieme indiscriminato dei positivi di ogni vecchia versione. Senza prove
sufficienti rimane Unknown. Le ratifiche separate conservano le fonti.

## 3. Autenticità con il protocollo già esistente

Non si introduce un nuovo MAC, purpose crittografico, chiave o derivazione.
Ogni payload di fatto/manifest è trattato come una **proiezione canonica
selezionata di un record tecnico**, usando esattamente l'API già ratificata:

```
canonical_payload = canonical_value_bytes(payload tipizzato)
projection = JoinKey a un componente canonical_payload
seal = protect_projection(scope, canonical_key, plan_definition, projection)
```

Gli scope usano namespace/flow/schema/trace e key_kind tecnico distinto. I
key_kind sono metadati dell'IdentityProtectionScope generico già previsto,
non nuovi purpose nel protocollo HMAC: rimangono key/record/record-proof v1 e
le stesse routine/framing. Tutti i byte degli oggetti precedenti restano invariati.
Il payload tecnico include tutti i dati di provenienza/copertura/commitment;
non si presume che un token di identità autentichi altri valori scollegati.

Il lettore verifica prima le prove private del seal con verify_stored_identity,
ricostruisce lo stesso scope, chiave e proiezione dai metadati attesi e payload,
ricalcola protect_projection e confronta identità/fingerprint attesi. Verifica
poi source/revisione/piano/namespace e commitment con numero righe effettivo.
La semplice DTO pubblica o un digest senza seal non sono prove. Alterazione,
rimozione, duplicazione o spostamento fra contesti falliscono tecnicamente.
Il manifest sigillato impegna la lista canonica degli assessment, la copertura
e i conteggi: rimuovere un witness e correggere countSQL non può provare False.

Questo è riuso della primitiva di proiezione già approvata, non l'introduzione
di un protocollo alternativo per firmare byte con il segreto. I test manterranno
le golden HMAC esistenti e proveranno falsificazione/transplant/DTO/omissione.
Se il codice richiedesse realmente un nuovo purpose o una garanzia non ottenibile
con queste API, solo quel punto richiederebbe una decisione crypto separata.

## 4. Versione, rappresentazione e repository

Registro osservato: SQLite FORMAT=2, reader1/2, upgrade esplicito1→2. Nuovo
formato fisico proposto e adottato per archivi nuovi: **3**; nessun formato3
presente nel codice/version registry prima di questo delta. Modello dei fatti
v1 separato dalla versione fisica. Il registro verrà aggiornato nello stesso delta.

I batch saranno associati a HistoricalSnapshot come payload opzionale tipizzato,
con costruttore privato al produttore; vecchio new senza fatti mantiene semantica.
La port transaction stage_snapshot rimane un'unica unità, non una seconda scrittura.
Snapshot normale senza nuova capability non dichiara che l'indice fatti esista.

SQLite3 aggiunge il manifest obbligatorio snapshot_manifests (root ordinata dei
token e root dei batch, fonte/revisione/formato protetti), oltre alla tabella history_batches con FK(ns,revision), payload tipizzato,
seal tramite lo stesso RecordWire/PrivateProof già qualificato e conteggi/root
nel payload autenticato. Eventuali indici secondari sono acceleratori verificati
rispetto al batch sigillato, non verità normativa. Le query si limitano ai batch
compatibili/paginati pertinenti; non caricano righe cliniche o tutto il repository.
Codec con deny_unknown_fields, errori su duplicati/formati/versioni incoerenti,
limiti e conversioni controllati, nessun fallback o flag Complete permissivo.

Archivi1/2: lettura delle funzioni precedenti preservata, storia fatti assente
riconosciuta come LegacyGap/Unknown per le nuove query. Nessuna migrazione o
ricostruzione da fingerprint. Nuovi fatti su formato vecchio richiedono
operazione esplicita futura; il writer rifiuta senza alterare l'archivio. Non
si assegna loro automaticamente formato3 né copertura dall'origine.

## 5. Query e gate applicativo

Repository preflight/audit/chiave/versione tecnicamente validi e generazione
pinnata. Solo fonti approvate compatibili più staging corrente. Verificare
l'affidabilità dei witness effettivamente usati; True da un positivo autonomo
valido anche con gap altrove. Nessun positivo + ProvenComplete sufficiente →
False; nessun positivo + gap/origine ignota → Unknown. Corruzione/prova invalida/
I/O/chiave/codec obbligatorio sconosciuto → errore tecnico, non False/Unknown.

Nel nuovo servizio una regola storica obbligatoria Unknown conserva Partial e
restituisce ValidationOnly, senza capability di approvazione del nuovo percorso;
False produce la violazione dello schema, True rende il run soltanto eleggibile
agli altri gate. Non si modifica ApprovalDraft pubblico né si trasforma un
consenso su omissioni in verità normativa. Source/summary/audit provengono dallo
stesso run e dalla stessa sorgenteRaw, non da fixture di esito o ricarichi diversi.

La validità di witness modificati/annullati non viene decisa dall'ordine degli
indici SQL: dipende dalla casistica ratificata. Finché un caso non è coperto,
nessuna prova interessata viene promossa a True o False per supposizione.

## 6. Commit e recovery

Manifest source-approved/revisione e snapshot devono concordare. Fatti e snapshot
sono inseriti nella stessa transazione con controllo baseGeneration, dopo report
pubblicato e approvazione esplicita. Niente fatti di draft/preview, nessuna scrittura
su errore report/cancellazione/CASfallito. Immutabilità/idempotenza esistenti invariate.

Ricevuta e risoluzione di COMMIT ambiguo verificano anche la presenza e integrità
del batch atteso; non basta trovare la Source se il payload richiesto manca.
Crash/rollback o reopen devono dare stato vecchio o intera unità nuova, mai successo
su unità parziale. SQL rimane nell'adapter generico, nessun driver specifico CAD.

## 7. Accettazione e prosecuzione

La matrice HX01..HX21 del documento esteso viene implementata sul sottoinsieme
reale; test applicativi T1/T2 con veri file, proiezione/approval/reopen/query e
83campi/regole obbligatorie prima di DONE12. Coverage/lifecycle non qualificati
rimangono Unknown e residui visibili, non PASS. Nessun nuovo algoritmo/chiave,
spool normativo, parallelismo o riordino diagnostico. Gate locali/native7job sullo
SHA finale prima di CAD12DONE; poi P18-04 quando tutti i prerequisiti sono reali.
