# T-P3-06 — Formati e versione minima: CNF-FORMAT-01

2026-10-19. Proposta A non approvata: l’autore ha richiesto un inventario
completo e una mappatura ufficiale. [Inventario e decisioni proposte](../schema-authoring/FORMAT_COMPATIBILITY.md).
Nessuna versione futura inventata; attività indipendenti implementate e verificate in locale.

## Prerequisiti

G0 approvato sul commit 407f89e, accettazione 34e67da; T-P3-05 verificata
sul commit b658e05 in [CI 39264362959](https://github.com/UraganoAngelina/lintra/actions/runs/39264362959),
Linux GNU e Windows MSVC completed/success. CNF-COMPILER-01/proposta A già
approvata riguarda il gate dei costrutti, non identifica release future.

## CNF-FORMAT-01

ARCH §39C.2.3, DET Addendum A.1 e prompt T-P3-06 richiedono che l’errore per
formato non supportato indichi la versione minima dell’eseguibile richiesta.
SCHEMA_AUTHORING (versioni e identità) precisa invece:
«La prima release compatibile definirà la versione minima del binario;
non si inventa oggi un numero di release non implementata».

Formato 1 è l’unico contratto disponibile. Né flow.schema.json né
manifest.schema.json contengono una proprietà di versione minima del motore;
non esiste un catalogo di release/formati nel repository. Cargo 0.1.0 è la
versione di sviluppo corrente, non prova una release compatibile con
qualsiasi formato futuro. Non si può dedurre che formato 2 richieda motore
2.x, né che una versione inferiore del formato sia automaticamente accettata.

## Proposta A — da approvare

- EngineCapabilities dichiara il range effettivo 1..=1, con SchemaFormatVersion
  tipizzato ed engine_version ricavata dalla versione del package corrente.
  Nessuna capability normativa completa viene pubblicizzata.
- Lint e compiler condividono questo controllo. Formato numerico fuori range
  produce UnsupportedSchemaFormat in SchemaCompilationError; formato assente,
  null, stringa, negativo o oltre u32 resta errore tecnico di caricamento raw.
  Nessun default o fallback al formato corrente, nessuna conversione legacy.
- Il messaggio indica formato richiesto, motore corrente, range supportato e
  **«minimum required executable version: not declared for this format»**
  quando la mappatura ufficiale non è disponibile. Non raccomanda una release
  specifica o la disponibilità di un aggiornamento non verificato.
- Questa politica di informazione assente soddisfa il gate attuale della task;
  la versione minima concreta sarà vincolata a una release compatibile quando
  dichiarata, senza aggiungere oggi campi JSON, cataloghi fittizi o nuove release.
- La decisione integra il contratto di authoring. ARCH/DET/Piano e baseline
  originali non vengono riscritti; nessuna modifica alla deroga local.v1.

API del descrittore (DET Addendum A.1):

```rust
pub struct SchemaFormatVersion(pub u32);
pub struct EngineCapabilities {
    pub supported_schema_formats: RangeInclusive<SchemaFormatVersion>,
    pub engine_version: String,
}
```

Il descrittore dell’eseguibile corrente è costruito dal codice; un descrittore
ipotetico creato dal chiamante non riconfigura il compiler e non autorizza
un formato diverso da quello implementato.

## Alternativa B

Mantenere aperto il requisito della versione minima concreta finché esiste
una fonte ufficiale approvata per il mapping. T-P3-06 non viene dichiarata
completa; i controlli di range indipendenti possono essere verificati senza
scegliere il contenuto del messaggio o anticipare altre task.

## Test previsti

Formato 1 e suoi confini inclusivi; 0, 2, u32::MAX rifiutati; formato assente /
shape numerica errata rifiutati dal repository; messaggio completo verificato
se A approvata; regressione dei test compiler/linter/P0 e CI Linux/Windows.
