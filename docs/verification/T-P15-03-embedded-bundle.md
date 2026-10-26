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
