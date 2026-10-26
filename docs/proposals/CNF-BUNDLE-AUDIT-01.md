# CNF-BUNDLE-AUDIT-01 — Manifest e audit dei bundle esterni preesistenti

2026-10-26. **PROPOSTA, NON RATIFICATA.** T-P15-03, wiring ancora non modificato.

## Punto da decidere, evidenza effettiva

ARCH§39C.2 e DET A.1 richiedono bundle_source e bundle_version in report/audit.
Il manifest authoring ratificato contiene bundle_version, schema_format_version
e files con SHA-256. Per lo snapshot incorporato la versione è disponibile.

Il contratto CLI già consegnato in8-01/8-03 accetta invece due root esplicite
anche indipendenti e prive di MANIFEST.json. src/cli/paths.rs controlla le due
directory e le priorità, non impone un parent comune né un manifest. I test
cli_validate/cli_full e il test di cancellazione in src/cli/command.rs installano
schemi e directory reference reali senza manifest e oggi eseguono la validazione.
RunMetadata, ValidationSummary e il loro writer attuali non hanno provenienza
bundle/versione; gli utenti diretti di ValidationService possono iniettare anche
repository in-memory. Non esiste una versione bundle ricavabile affidabilmente
da queste configurazioni. Nessuna ratifica applicabile trovata sul caso senza
manifest o sulla sua localizzazione quando le root hanno parent diversi.

Imporre un manifest obbligatorio cambia l'ammissibilità delle configurazioni
precedenti. Scrivere una versione derivata da path/schema/latest o una versione
fittizia violerebbe la tracciabilità. L'opzionalità esplicita è una precisazione
del contratto audit, non la semplice scelta di un helper Rust.

## Vincolo già risolto, da non riaprire

CNF-REFERENCE-FORMAT-01 ratificata prevale sulle formulazioni generiche di bundle
completo: ContinueWithAvailableEvidence conserva Degraded e Unavailable, Strict
mantiene i propri requisiti, nessun fallback a snapshot e nessun set vuoto fittizio.
Un eventuale controllo manifest non deve intercettare tutti i file reference e
trasformare indiscriminatamente hash mismatch/assenza in errore fatale, aggirando
le evidenze del provider. Questa semantica non è rimessa in approvazione.

## Proposta A — raccomandata, compatibile e additiva

1. Lo snapshot embedded ha sempre il proprio manifest verificato e una versione
   esplicita; nessuna versione mancante o dedotta.
2. Per External il manifest è opzionale. Si può indicare con --bundle-manifest
   PATH; in sua assenza si considera soltanto MANIFEST.json del parent comune
   quando le due root selezionate sono esattamente schemas/ e reference-data/.
   Nessuna ricerca ricorsiva, cwd/home o ulteriore fallback dei dati. Root con
   parent diversi restano ammesse; il manifest può essere fornito esplicitamente.
3. Senza manifest: bundle_source="external", bundle_version=null e indicazione
   esplicita manifest assente. Non si inventa un'identità. Con manifest presente:
   versione dichiarata e stato esplicito; un JSON ambiguo/strutturalmente invalido,
   path non sicuro o manifest esplicitamente richiesto ma assente è errore tecnico.
   La versione dichiarata non certifica da sola tutti i file del bundle.
4. Manifest valido: verifica del file schema selezionato prima della compilazione.
   Le dichiarazioni reference devono essere coerenti con la sua mappa path/digest;
   un conflitto delle dichiarazioni è errore di authoring. Le anomalie dei byte
   effettivi rimangono gestite dai provider secondo Strict/Continue già ratificati:
   file assente → Unavailable; hash mismatch con contenuto affidabile → Degraded.
   Nessuna verifica globale preventiva che annulli la modalità parziale.
5. Audit applicato al confine adapter/report con contesto bundle tipizzato e
   writer additivo, preservando le firme e gli struct pubblici preesistenti di
   ValidationService/ValidationRequest/RunMetadata/ValidationSummary. La CLI usa
   sempre il writer con provenienza; le API generiche senza bundle non inventano
   un'origine per repository opachi. JSONL errori e semantica engine invariati.
6. Confronto External/Embedded: stesso schema hash ed errori byte-identici a
   contenuto uguale; metadata source distinti e versione coerente con i manifest.
   I pacchetti prodotti in15-04 avranno sempre un manifest, non null.

Campi additivi proposti per l'audit JSON della CLI:

```json
{"bundle_source":"external","bundle_version":null,"bundle_manifest_status":"absent"}
```

```json
{"bundle_source":"embedded","bundle_version":"synthetic-embedded-v1","bundle_manifest_status":"present"}
```

Per External con manifest disponibile, source resta external, version è quella
dichiarata e status è present. present descrive la presenza di metadata validi,
non availability=Verified di tutti i dataset. Il riferimento a uno schema/dataset
assente non attiva mai fallback a snapshot o a un'altra versione.

## Alternativa B — manifest obbligatorio per la CLI di validazione

Stessa risoluzione deterministica e policy reference, ma l'assenza del manifest
è errore tecnico anche per configurazioni prima ammesse. Ogni installazione
esterna e relativa fixture/client deve essere aggiornata esplicitamente; audit
sempre con versione non nulla. Nessun fallback a dati embedded dopo errore.
Questa rottura del contratto di configurazione richiede scelta dell'autore.

## Stato e residui

Backend incorporati/manifest sintetico già implementati e in verifica locale;
nessuna delle due proposte applicata al wiring CLI, al resolver o all'audit.
Feature presente ma non predefinita, non ancora attivata nelle release. T-P15-03
non è DONE; rimangono integrazione, test dei quattro livelli, output equivalenti
e CI nativa. ARCH/DET, norme e ratifiche storiche non sono modificati.

Decisione richiesta: approvare A oppure scegliere B. Dopo la decisione ripresa
immediata della stessa task, senza riavvio dei backend già verificati.
