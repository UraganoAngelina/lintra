# SIOC — Tabelle campi per revisione, T-P0-02

**DONE — campione verificato e approvato dall'autore il 2026-10-16.**
L'esito corrente è registrato in `IMPLEMENTATION_STATUS.md`; `document_status`
nei JSON e il report iniziale descrivono lo snapshot preparatorio antecedente
all'approvazione. Nessun valore della fonte è stato riscritto.
La fonte ricevuta è il JSON derivato **FROZEN** sotto
`K/SCHEMA_INPUTS/SIOC_2026-04-20_FROZEN/`; l'XLSX ufficiale
`SIOC_tracciato_regionale_20260420-definitivo_v05.xlsx` è assente.
`K/` indica lo starter kit adiacente al repository. Vedi
[inventario](../INPUT_INVENTORY.md) e [report con script e risultati](../../verification/T-P0-02.md).

| Tabella | Campi | Intervallo one-based inclusivo |
|---|---:|---|
| [1](1-fields.json) | 18 | 1–206 |
| [2](2-fields.json) | 50 | 1–271 |
| [3](3-fields.json) | 36 | 1–135 |
| [4.1](4.1-fields.json) | 20 | 1–394 |
| [4.2](4.2-fields.json) | 11 | 1–67 |

Ogni riga conserva esattamente `id`, `name`, l'eventuale `source_id` esterno,
`physical` completo e `regulatory_metadata` completo del trace ricevuto.
Quindi start/lunghezza, tipo fisico e alias sorgente, obbligatorietà, nota,
marcatori di chiave ed eventuali interpretazioni già ricevute rimangono distinti.
Nessun trim o riscrittura delle stringhe; `\n` nelle stringhe JSON rappresenta
il newline originario. Non sono tabelle runtime né una nuova formalizzazione
di tipi/presenza; non includono constraints, presence o regole eseguibili.

`source_manifest_metadata` conserva l'intero manifest ricevuto;
`source_trace_metadata` conserva versioni, stato, identificazione/lunghezza,
`value_model` e `freeze_notes` del trace. `provenance` identifica trace, manifest,
flow e freeze report con percorso nel kit e SHA-256. Tutti gli hash/dimensioni
sono stati confrontati col manifest del kit. I cinque split coincidono
integralmente con i tracciati annidati nel flow ricevuto.

Il manifest e il flow esterno dichiarano `regulatory_version: 2026-04-20`,
i cinque trace e quelli annidati nel flow dichiarano `2025-12-16`.
`FREEZE_REPORT.md:4–6` distingue baseline 2025, profilo 2026 e DSL `1.0.0`;
`SCHEMA_AUTHORING.md:11` dichiara il flow autorevole, mentre l'esempio a riga 22
usa ancora 2025. Sono metadati ricevuti conservati, senza correzione o decisione
sulla versione runtime. Non è stata trovata una ratifica esplicita del mantenimento
della data 2025 nei metadati dei trace. Il freeze stesso precisa che compilatore
Rust, runtime e approvazione normativa esterna non sono verificati.

La verifica documentale ha confrontato tutti i **135 campi** senza differenze,
ID duplicati, gap o overlap; ogni ultimo byte coincide con `record_length`.
L'XLSX assente impedisce il confronto con le celle ufficiali. Nessun dato
sanitario o export è stato letto; nessuna chiusura G0 o approvazione implicita.

Esempi consegnati per il campionamento umano, approvato dall'autore:

- `1.1`: nota multilinea del 2026, inclusi refusi, e lunghezza fisica 20.
- `1.9`, `2.4–2.6`: alias `Data`; `4.6`/`4.10`: alias `A`, senza normalizzarli.
- `3.35`/`3.36`: entrambi i `source_id` attuali restano identici; gli alias
  storici `3.48`/`3.49` restano nel freeze report e nelle decisioni del manifest.
- `3.34`: etichetta sorgente `NBB` conservata; il disallineamento con l'obbligo
  descritto nel freeze resta una questione documentata, non una nuova regola.
- `4.5`: `physical.source_type` è **null** nel JSON ricevuto, pur essendo
  `physical.type: numeric` e lunghezza 1; il manifest descrive `N(1)`.
  Il null resta null: non è sostituito con `N`.
