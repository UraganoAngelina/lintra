# DEC-18 — Catalogo schemi e reference data

2026-10-26 — PROPOSTA STORICA APPROVATA CON PRECISAZIONI VINCOLANTI:
prevale la [ratifica definitiva](../verification/DEC-18-ratification.md).

## Evidenza e autorizzazione richiesta

Master Plan, registro DEC-18: decisione umana aperta, prerequisito di14-05.
ARCH§30 elenca i comandi senza definire l'enumerazione. DET§10/63 e codice
reale espongono solo SchemaRepository::load_raw e ReferenceDataProvider::load.
Nessuna ratifica DEC-18 reperita.14-01 e8-04 sono DONE.
Proposta additiva: nessuna modifica ad ARCH/DET/piano/baseline/grammatica.

## Catalogo schemi

Porta separata `SchemaCatalog: Send + Sync` con
`list_schemas() -> Result<Vec<SchemaCatalogEntry>, SchemaCatalogError>`,
implementata nei repository filesystem/in-memory. La porta load resta compatibile.
Entry: SchemaKey e ID trace nell'ordine dichiarato, non vuoti e univoci.
Entry ordinate per byte UTF-8 flow/version, nessuna versione latest o casefold.

Filesystem: scansione limitata a root/flow/version/flow.json, layout runtime
già adottato dal loader. Nessuna conversione delle directory authoring che
contengono flow_schema.json. Root vuota esistente → lista vuota; root assente
o non leggibile → errore. File estranei al layout ignorati; componenti flow/version
soggetti agli stessi controlli portabili del loader. Root esplicita canonicalizzata;
symlink nei livelli enumerati e sul flow.json candidato rifiutati. Nessuna
ricorsione arbitraria né promessa contro mutazioni ostili concorrenti dell'albero.

Candidati ordinati prima dei controlli; deserializzazione come load_raw, con
rifiuto proprietà JSON duplicate e identità discordante dal percorso. Errori
I/O, JSON o identità ambigue falliscono l'intero elenco: niente omissioni silenziose.
Enumerabile non significa compilabile: nessuna compilazione/regola eseguita,
nessuna certificazione delle capability. Compiler obbligatorio prima dei run.

CLI: `lintra schema list --schema-dir PATH`, JSON UTF-8 stdout:
`{"schemas":[{"flow_id":"...","schema_version":"...","traces":["..."]}]}`.
Root esplicita come schema lint; nessuna ricerca cwd/home o fallback inventato.

## Reference data da binding espliciti

CLI: `lintra reference-data list --schema-dir PATH --reference-data-dir PATH
--flow ID --schema-version VERSION`.
Inventario delle sole dipendenze dello schema selezionato. Il provider ratificato
non ha catalogo globale: niente secondo manifest o identità dedotte dai filename.
Un inventario globale indipendente dagli schemi resta fuori da questa consegna.

Caricare raw per chiave; condividere i controlli già esistenti di
compile_dependencies mediante piccola API pubblica di tooling con errori tipizzati.
Validare solo le dichiarazioni reference, senza richiedere che l'intero schema
sia eseguibile: nessun aggiramento del compiler nei run. Dichiarazioni duplicate,
path/hash/identità invalidi e metadata non supportati restano errori.

Preparazione reale con ContinueWithAvailableEvidence esplicita: mostrare
Verified/Degraded/Unavailable, digest atteso/effettivo e anomalie, mai le chiavi.
Boundary/symlink restano fatali. Dataset assente/malformato/degradato compare
con stato e cause: successo del listing non significa idoneità per un run Strict.
Root selezionate non valide/non leggibili falliscono prima dell'inventario.
Zero dipendenze → lista vuota. Nessun provider o dataset fittizio.

JSON con selettore schema e array datasets ordinato per ID/versione; ogni elemento
riporta declaration ID, dataset ID/versione, path relativo, policy, availability,
digest e cause tipizzate. Strict e preflight dei run restano invariati.

## Import e lint: wiring esistente

CLI: `lintra schema import --input XLSX --blueprint JSON --profile JSON
--output JSON [--type-aliases JSON]`.
Profilo/blueprint revisionati obbligatori. Alias opzionali tramite TypeAlias e
normalize_import_profile già consegnati; senza alias profilo già canonical.
Nessuna norma dedotta dalle note, nessun flusso selezionato implicitamente.

Importer reale → lint → serializzazione canonical del raw draft. Error di lint
impedisce pubblicazione; warning integralmente su stderr JSON con severity/code/
coordinate permette output draft, senza dichiararlo compilabile o installarlo.
Il completo SIOC resta rifiutato per capability future quando viene compilato.
Conservare note/provenienza/hash/formule cached secondo14-01/02.
Output nuovo atomico senza sovrascrittura implicita; collisioni con input XLSX,
blueprint/profilo/alias rifiutate, input preservati su errore.
Esporre inoltre i warning di schema lint quando la compilazione riesce, senza
duplicarli né cambiare la verifica corrente tramite registry.

## Esiti e verifica dopo approvazione

Exit tooling0 successo,1 errore tecnico/authoring; mai2, riservato alle violazioni
dei record. JSON liste costruito dopo tutti i controlli, senza output parziale
prima di un errore di scansione; errore scrittura stdout rimane tecnico.
Core senza clap, nessuna modifica engine/application o regola normativa.

Test sintetici: lista vuota/mancante, mismatch/duplicate JSON/trace, ordine
deterministico filesystem/in-memory, versioni esplicite, path nativi/symlink,
I/O; reference presente/assente/hash mismatch/malformato/boundary; import XLSX
reale positivo/negativo, warning/error, output esistente e input preservati.
Gate locali e CI Linux/MSVC prima di DONE/M7. Nessun embedded/P15, nuovo flusso,
dataset ufficiale inventato, persistenza storica o UI anticipati.

## Decisione

Approvare questo contratto oppure indicare modifiche, in particolare su catalogo
separato da load, inventario reference vincolato allo schema ed errori senza
omissioni. L'autonomia generale non viene interpretata come ratifica DEC-18.
