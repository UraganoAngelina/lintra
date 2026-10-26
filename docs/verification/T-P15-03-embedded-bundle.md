# T-P15-03 — Snapshot incorporato e risoluzione completa

2026-10-26, preflight successivo a15-02. Prerequisiti15-01 (CI40307076976)
e8-01 (CI39628560176) verificati;15-02 ora DONE/CI40329791116. HEAD iniziale
7a3371a, codice precedente verificato59b1663. Nessuna implementazione embedded
preesistente: resolver solo tre livelli, provider reali filesystem/in-memory.

## Piano e perimetro corrente

- Feature embedded-bundle, snapshot esplicitamente sintetico derivato dalla
  fixture applicativa già presente; nessuna norma o dataset ufficiale inventato.
- Manifest nel formato authoring ratificato (versione, formato, mappa SHA-256).
  Asset inclusi a compile time; repository embedded legge bytes veri senza
  estrazioni temporanee o dipendenze dai path della macchina di build.
- Provider/preparer embedded riusa la decodifica e le evidenze reference esistenti,
  con Strict/Continue invariati. Niente provider fittizio o dataset vuoto sostitutivo.
- Quarto livello del resolver solo se nessuna coppia esterna è selezionata;
  coppie incomplete, directory inesistenti e chiavi/versioni assenti restano errori.
- Wiring e audit della provenienza; test di precedenza, provider, integrità e
  confronto degli errori External/Embedded a parità di contenuto. I metadati di
  origine restano intenzionalmente distinti, non vengono nascosti per il confronto.

Consultati ARCH§39C.2, DET A.1, contratto manifest SCHEMA_AUTHORING e relativo
meta-schema, contratti repository/registry/preparer/service/report effettivi.
Nessun nuovo crate richiesto: std, serde e sha2 già disponibili.
Stato IN_PROGRESS; nessun gate eseguito o DONE anticipato per questa task.
15-02 resta chiusa sullo SHA verificato; nessuna modifica retroattiva ai suoi gate.

## Handoff di avvio

AUTO_CONTINUE, sviluppo single-agent sequenziale.91 task DONE,15-03 avviata;
M5 resta aperta. Residui: intera implementazione e gate15-03. Nessuna decisione
normativa nuova proposta, nessun dato paziente o fonte protetta modificata.

## Backend implementati, wiring sospeso su contratto audit

Durante l'analisi del wiring è emersa CNF-BUNDLE-AUDIT-01: le configurazioni
esterne precedenti non richiedono manifest né parent comune, mentre ARCH/DET
richiedono bundle_version. La gestione di versione assente e l'eventuale nuovo
requisito manifest non sono ratificati. [Proposta concreta A/B](../proposals/CNF-BUNDLE-AUDIT-01.md).
Le policy reference sono già ratificate e non vengono riaperte: niente abort
globale che annulli ContinueWithAvailableEvidence, niente versione inventata.
Nessuna semantica proposta è applicata a CLI, resolver, service o report.

File e comportamento già implementati:

- Cargo feature embedded-bundle esplicita, default vuoto: nessuna capacità CLI
  non implementata viene annunciata come fallback già disponibile in release.
- bundles/embedded: schema SYNTHETIC/v1 copiato dai byte della fixture applicativa
  esistente; dataset dimostrativo SYNTHETIC_CODES/v1 con001/002 e manifest SHA
  generato sui byte esatti. README dichiara che lo schema non richiede il dataset;
  questo esercita il provider senza simulare regole o tabelle ufficiali.
  .gitattributes -text conserva i digest su Windows. Fonti normative invariate.
- BundleError distingue JSON, manifest invalido, path pericoloso, file assente
  e integrità; errori tecnici, mai ValidationError. SchemaLoadError aggiunge
  una variante bundle al contratto già non_exhaustive senza cambiare load_raw.
- BundleManifest parse riusa il controllo JSON senza duplicati già presente nel
  repository; campi chiusi, versione non vuota, inventario non vuoto, formato1
  del manifest authoring, path relativi portabili e digest lowercase di64 cifre.
  verify_bytes confronta SHA-256 dei byte originali con la voce dichiarata.
  Non viene inventato un manifest formato2 o modificata la grammatica approvata.
- EmbeddedFile conserva path e slice statiche. EmbeddedBundle::from_static
  rifiuta duplicati, file non dichiarati, mancanti e digest errati; i payload
  restano nel binario, senza estrazione temporanea o copia dei loro contenuti.
  Le copie del bundle condividono manifest e mappa tramite Arc. builtin include
  solo i due payload e il manifest sintetici, senza scegliere un flow a runtime.
- EmbeddedSchemaRepository acquisisce JSON reale secondo schemas/flow/version/
  flow.json, verifica identità percorso/contenuto e formato del manifest,
  riusa InMemorySchemaRepository per caricamento a chiave e catalogo. Non
  compila implicitamente, non sceglie versioni, non converte flow_schema.json.
- validate_bindings, initial_evidence, decode_bytes e append_prepared sono
  estrazioni interne minime del provider filesystem per riusare identica logica
  negli adapter: stesso ordine errori, digest originale, parser chiuso,
  disponibilità, cardinalità e Strict. Nessuna configurazione filesystem fittizia
  viene costruita per validare binding embedded; niente mutazioni dell'ambiente.
- EmbeddedReferenceDataProvider conserva bytes e binding reali, seleziona solo
  il path dichiarato e produce evidenze attraverso gli helper comuni. Asset
  assente ha causa tipizzata ReadFile/NotFound con messaggio esplicito embedded,
  nessuna syscall filesystem simulata e nessun set vuoto sostitutivo. Il vecchio
  load resta Strict; prepare conserva Verified/Degraded/Unavailable e digest
  distinti. EmbeddedReferenceDataPreparer lega il bundle una volta e prepara
  le dipendenze compilate del run con la policy richiesta.
- La verifica JSON unica è ora un helper interno condiviso; i contratti
  filesystem/load_raw, compiler, engine e service mantengono firme e comportamento.

5 test mirati con feature passati: manifesto/schema/catalogo/registry reali e
schema hash uguale a filesystem; parità FS/Embedded per Verified, Degraded,
Unavailable e Strict; binding duplicati/traversal/metadata non supportati;
JSON malformato senza recupero fittizio; manifest duplicato/pericoloso/incompleto/
hash errato. Backend new con bytes statici verifica realmente l'integrità.
Test globali conclusi con exit0: default671 pass e all-features676 pass, entrambi
78 suite/0fail/2ignored già previsti. Clippy all-targets default/all-features
-D warnings, formato e check lib GNU all-features positivi; audit governance e
diff check positivi. Log ignorati in target/verification-p15-03, CARGO_HOME
/tmp/lintra-cargo-home, debug dev/test0, incremental0. Lock e dipendenze del
prodotto invariati. Nessuna CI15-03 o DONE dichiarata: il gate nativo completo
verrà avviato sul wiring concluso, evitando pipeline duplicate su una task bloccata.
[Evidenze strutturate](T-P15-03-local-gates.json).

## Handoff corrente

T-P15-02 DONE/CI40329791116;91/140=65,0%, M5 ancora aperta. T-P15-03 ha backend
parziali e gate locali default/features positivi; wiring, provenienza audit, livello4,
attivazione release e test processuali/cross-OS restano incompleti.
Prosecuzione BLOCKED su CNF-BUNDLE-AUDIT-01: attesa decisione A/B. Non è una
richiesta di permesso Git/CI o uno stop ordinario di fine task. Codice valido
preservato; ripresa sul residuo dopo ratifica, senza ricominciare.

## Ripresa autorizzata — CNF-BUNDLE-AUDIT-01 A

L’autore approva A: [ratifica](CNF-BUNDLE-AUDIT-01-ratification.md). Il precedente
blocco è risolto, non si ripetono i backend già verificati. Checkpoint6f8ae70.

### Wiring e audit additivi

- `BundleAudit` ha campi privati e costruttori external/embedded: external senza
  manifest serializza version null/status absent, con manifest versione dichiarata
  e present. Embedded richiede un EmbeddedBundle realmente verificato e conserva
  la sua versione. Present descrive i metadata, non certifica i dataset.
- `ManifestSchemaRepository` avvolge solo la sorgente filesystem esplicita: usa
  schema_path per verificare il selettore, canonicalizza root/file e rifiuta escape,
  legge una volta e verifica gli stessi byte prima di deserializzarli. Identità
  e formato devono corrispondere a selettore e manifest. Nessuna compilazione
  alternativa o fallback: SchemaRegistry/SchemaCompiler restano obbligatori.
  Il containment non promette protezione contro sostituzioni ostili concorrenti.
- `ManifestReferenceDataPreparer` confronta path/digest di tutte le dipendenze
  compilate con il manifest, poi delega al preparer esistente. Non apre alcun file
  prima del provider; mismatch dei byte/missing/malformed restano rispettivamente
  Degraded/Unavailable/Unavailable secondo identità e parse, con Strict invariata.
- Il writer privato condiviso riceve un contesto opzionale e serializza i campi
  additivi solo con contesto. `write_summary`/`write_run_summary` passano None,
  mantenendo il contratto precedente. `write_run_summary_with_bundle` aggiunge
  provenienza effettiva per la CLI. Nessun campo/firma modificato in service,
  request, RunMetadata o ValidationSummary; JSONL errori invariato.
- Il resolver mantiene la selezione esclusiva CLI→env→exe e abilita Embedded
  soltanto dopo assenza di entrambe le directory adiacenti. Pair incompleti,
  directory inesistenti/errate o errori di lettura non attivano fallback.
  Senza feature la precedente assenza rimane errore esplicito.
- `cli/bundle.rs` compone service/registry/preparer reali della sola sorgente
  scelta. Il manifest esterno esplicito deve esistere; discovery implicita solo
  per sibling schemas/reference-data nello stesso parent. JSON invalido, symlink
  del manifest e file non regolari sono errori. Nessun manifest cwd/home.
  Un manifest esterno esplicito senza sorgente esterna non viene ignorato.
- `check_outputs` riceve le root effettive (nessuna root fittizia per Embedded)
  e protegge anche il manifest esplicito/individuato dalle collisioni con file
  finali/part. Restano rollback e cancellazione dei report precedenti.
- La ricetta release xtask seleziona esplicitamente embedded-bundle; il default
  dev rimane senza feature. CI verifica entrambe le configurazioni su Linux/MSVC
  e mantiene build riproducibili/controllo artifact/negative probe.

Stato: implementazione e test in corso, gate completi/CI ancora da eseguire.
Le evidenze backend precedenti non sono attribuite al nuovo wiring.

Revisione del caso di confine: il probe delle directory adiacenti usa
symlink_metadata prima di seguire un link. Un link pendente è una sorgente
esterna presente ma rotta, quindi errore e mai fallback embedded. Test Unix
specifico; link validi a directory mantengono il comportamento precedente.

## Gate completi locali e consegna alla CI

Configurazione finale, dopo il fix symlink: test default 681 pass,
all-features 689 pass, entrambi 79 suite/0fail/2ignored previsti;
fmt, Clippy all-targets default/all-features -D warnings, check lib Windows GNU
all-features positivi. Tooling fmt/Clippy/9 test positivi; governance e diff check
positivi. Le due test run complete includono doctest e golden precedenti. Nuovi
test: quattro livelli e symlink pendente, 11 integrazioni bundle con feature,
writer additivo e propagazione I/O; oltre ai5 test backend del checkpoint.

I processi reali verificano manifest implicito/esplicito e root indipendenti,
versione assente/null, rifiuto di manifest assente esplicito/invalido/duplicato/
pericoloso/symlink, digest schema, mancato fallback per chiave/versione assente,
protezione report/input, parità JSONL e hash external/embedded. Il test CLI reference
con due dipendenze verifica trust e indipendenza, Strict e conflitto di dichiarazioni
fatale, senza chiamare present una certificazione normativa. Il writer legacy
continua a superare i golden byte-per-byte; API service/request/domain invariate.

Stato consegna: DONE_UNVERIFIED, CI Linux/MSVC da avviare sul commit completo.
T-P15-04 dipende dal gate15-03 e non viene iniziata prima del suo esito effettivo.
