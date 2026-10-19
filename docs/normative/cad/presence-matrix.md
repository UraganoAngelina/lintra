# CAD — Matrice presenza, assenza e padding (T-P0-03)

## Lettura della matrice e confine della consegna

Trascrizione documentale T-P0-03 del 2026-10-16; nessuna nuova policy runtime.
Prerequisito T-P0-02: 218 campi verificati meccanicamente, review indipendente
approvata (commit `419600d`, collegamenti report `6a4e9ac`); l'autore ha
approvato il campione il 2026-10-16: «Campione verificato: approvo».
`document_status` e testi di attesa nelle tabelle T-P0-02 rappresentano la
consegna precedente, non revocano questa approvazione successiva.
Vedi [verifica T-P0-02](../../verification/T-P0-02.md),
[inventario](../INPUT_INVENTORY.md) e
[report T-P0-03](../../verification/T-P0-03.md).

Ogni riga identifica il campo senza cambiare ID, ordine, posizione one-based,
larghezza fisica o alias sorgente. L'indice `i` è zero-based soltanto per
localizzare `/fields/i` nel JSON ricevuto e nella tabella T-P0-02 collegata.
In quella tabella `/fields/i/regulatory_metadata` conserva integralmente
etichette, note, pagine e annotazioni sorgente; nessuna nota diventa nuova
condizione eseguibile. I nomi restano consultabili lì, senza nuove abbreviazioni.

`R` e `F` sono i conteggi delle liste **ricevute** `required_if` e `forbidden_if`,
non l'esito della valutazione. L'appendice di ciascun trace conserva l'AST
esatto in `/field_policies/i/presence`, insieme a `physical`, `missing_value`,
`constraints`, eventuali `source_id` esterni e `semantic_null_values`.
`trace_context` conserva tutte le altre chiavi del trace, comprese
`named_conditions`, `derived_values`, `joins`, `rules`, gruppi e boundary
`documented_non_executable_rules`; i riferimenti `condition` restano locali
al trace e i riferimenti cross-trace mantengono il join ricevuto.
`flow_metadata` conserva tutte le chiavi del flow eccetto i trace.
Le appendici sono evidenza raw consultabile, **non** schemi da caricare nel runtime.

## Assenza, valore semantico e padding

DET §14–15/§106–107 e ARCH §19 richiedono policy esplicite, senza dedurre
l'assenza dal tipo numerico. Corrispondenze documentali con DET §14:

| `missing_value.kind` ricevuto | Corrispondenza | Significato fisico |
|---|---|---|
| `spaces` | `MissingValuePolicy::Spaces` | Esatta sequenza raw di spazi di larghezza del campo; nella matrice il conteggio `× 0x20` rende visibili gli spazi, l'appendice mantiene la stringa originale. |
| `exact` | `MissingValuePolicy::Exact` | Soltanto la stringa `raw` dichiarata, compresi gli zeri iniziali e l'eventuale `meaning`. |
| `never_missing` | `MissingValuePolicy::NeverMissing` | Nessun raw è classificato come mancante da questa policy; dominio, formato e presenza restano controlli distinti. |

`AnyOf` è previsto dal design ma non compare nei campi di queste fonti.
Una policy di assenza su un campo required indica come riconoscere la
violazione di presenza, non autorizza il valore mancante. Un codice semantico
«dato mancante», «non applicabile» o «deceduto» non diventa automaticamente
assenza fisica: `semantic_null_values`, domini e regole raw restano separati.
Non si elimina whitespace, non si converte il raw in numero e non si applica
un default globale `AllZeros`.

| `physical.alignment.kind` ricevuto | Allineamento e fill dichiarati | Collegamento al modello DET §15 |
|---|---|---|
| `left` | Contenuto a sinistra, padding a destra con il `pad` del campo. | Orientamento `RightPad`; non confondere il nome della posizione del contenuto con il lato del padding. |
| `right` | Contenuto a destra, padding a sinistra con il `pad` del campo. | Orientamento `LeftPad`. |
| `exact` | Rappresentazione esatta; nessun fill/padding aggiuntivo dichiarato. | `Exact`; non è una regola omessa. |

Questa è una corrispondenza descrittiva, non una compilazione Rust:
`ContentBoundaryPolicy` del modello DET non è specificata dai semplici
oggetti `left/right` ricevuti. La relativa traduzione è una boundary del
compiler futuro; non si inventano tolleranze sui byte interni o ai bordi.
Tutti i campi qui hanno sia `missing_value` sia `physical.alignment` espliciti;
nessun caso richiede «nessuna policy, con motivo». La presenza di un oggetto
raw completo non certifica implementazione o conformità del runtime.

## Fonte CAD, colonne sorgente ed eccezioni

Baseline raw: `K/CAD_REVISED_CANDIDATE/`, **RELEASE_CANDIDATE**, DSL `1.0.0`,
revisione `2026-10-16.reviewed-g02-g06`; `K/` è lo starter kit adiacente.
Fonte primaria PDF: `2025-05-14_FLUSSO_ADI.pdf`, intestazione interna
`2024-06`, etichetta data `2025-05-14`: non sono una nuova data di efficacia
dedotta. Il riscontro T-P0-02 delle 83 righe/note era testuale; i metadati
di estrazione e la necessità di revisione dei passaggi sensibili al layout
rimangono nelle tabelle approvate. Questa matrice non ripete l'estrazione PDF.

La colonna matrice `regionale / NSIS / risolta` mantiene separatamente
`source_obligation`, `source_nsis_obligation`, `resolved_obligation`.
SI indica obbligatorietà, NO facoltatività nella colonna pertinente; `NO (1*)`
mantiene il rimando originale, FAC resta FAC, NBB rinvia alle condizioni/note.
CAD-01 del registro ricevuto descrive la risoluzione delle divergenze come NBB;
la risoluzione esistente non viene ricalcolata qui. Nessuna equivalenza globale
tra SI/NO/FAC/NBB e base runtime: la policy configurata è una colonna distinta.

| Colonna regionale | `base: required` ricevuto | `base: optional` ricevuto |
|---|---:|---:|
| SI | 44 | 5 |
| NO | 0 | 17 |
| NO (1*) | 0 | 2 |
| NBB | 0 | 9 |
| FAC | 0 | 6 |

- A `1.4/1.5`: `NO (1*) / NBB / NBB`, required_if `1.3=4`.
  A `1.8/1.9`: `NO / NO / NO` ricevuto ma condizioni `1.3 != 1`;
  CAD-04 le descrive NBB: si conserva il disallineamento documentale.
  A `1.16/1.17`, B1 `2.36`, B2 `3.5`, C `4.7/4.11`, D `5.7`:
  `SI / NO / NBB`; alcune basi restano required (per esempio i progressivi),
  altre optional. B1 `2.6/2.7`: `NBB / SI / NBB`.
- A `1.24` resta optional senza AST d'età; boundary `CAD-AGE-UNDER18`.
  B1 `2.17/2.18` mantengono `SI / SI / SI` e base optional: le soglie
  di tre/sei anni sono documentate in `CAD-AGE-3-6` e nelle note, ancora
  non compilate dalla DSL ricevuta. Non si inventa qui `required_if` d'età.
- B1 `2.6` è optional; `2.7` è richiesto da `has_conclusion`. B1
  `2.5/2.27/2.29` sono richiesti in UCPDOM e vietati altrimenti;
  `2.28` richiede congiuntamente UCPDOM e appartenenza alla rete ed è
  vietato fuori da tale combinazione. `2.30–2.33` sono FAC in UCPDOM,
  vietati altrimenti. Le condizioni complete sono negli AST, senza semplificarle.
- **B1 `2.12`:** alias AN, allineamento left/space ma missing exact `000`,
  presence required. La nota prescrive `000` per patologia concomitante assente;
  `CAD-2.11-12-ICPC` lo riserva a tale significato. Il candidato dichiara
  contemporaneamente quel raw come missing: si registra il disallineamento
  fisico/semantico ricevuto, da chiarire normativamente prima dell'uso runtime,
  senza sostituirlo con spaces/never_missing né dedurre un'esenzione.
- **G03 già risolto:** B1 `2.22`, tipo sorgente T, 70 posizioni: `1–59=1/2`,
  posizione 60 in UCPDOM `1/2`, fuori UCPDOM esattamente un blank ASCII
  `0x20`, posizioni 61–70 esattamente `2`. Questo blank interno è un valore
  posizionale, non padding né assenza dell'intero campo; il missing raw del
  campo resta 70 spazi. `constraints[0].positions["60"].outside_ucpdom`
  contiene già `" "` e il gate report G03 conferma la decisione dell'autore.
  La vecchia `trace_context.freeze_notes` che lo dice «undecided» è preservata
  come metadato storico superato dalla decisione; non riapre G03.
- Tipi T `2.22/2.24/2.25/2.26` sono alphanumeric, left/space; i loro
  bitset/posizioni e fill specifici si conservano nelle constraints, senza
  aggiungere zeri per analogia con N. Le date sono left/space nel candidato,
  non `exact` per deduzione da `D`. B1 `2.10` mantiene nota `GGMMAAA`
  insieme a larghezza 8 e tipo sorgente D: nessuna correzione silenziosa.
- C `4.9` e `4.10` conservano i `forbidden_if` ricevuti (`4.5 != 4` e
  non appartenenza a `60/61`) insieme alle boundary `CAD-4.9-SPECIALIST`
  e `CAD-4.10-OSS`: nessuna risoluzione implicita della prosa.
- Alias fisici: `AN` 19, `N` 50, `D` 10, `T` 4. Tutti i numerici hanno
  missing exact per campo; ciò documenta il candidato, senza certificare
  la validità normativa di ogni convenzione né estenderla globalmente.

Riferimenti ricevuti:
[DECISION_REGISTER](../../../../LINTRA_STARTER_KIT_FINAL_20261016/CAD_REVISED_CANDIDATE/DECISION_REGISTER.md),
[FREEZE_GATE_REPORT G03](../../../../LINTRA_STARTER_KIT_FINAL_20261016/CAD_REVISED_CANDIDATE/FREEZE_GATE_REPORT.md),
[CAD_RUNTIME_PROFILE](../../../../LINTRA_STARTER_KIT_FINAL_20261016/CAD_REVISED_CANDIDATE/CAD_RUNTIME_PROFILE.json).
I punti storici nel registro non annullano la formalizzazione G03 successiva.
I tre dataset CAD nel flow corrente sono `SOURCE_EXTRACTED_REVIEWED`;
eventuali richiami precedenti a dataset CAD pending sono storici e non
descrivono questi file ricevuti. Questo non attesta i lookup nel runtime.
Nessuna promozione FROZEN, validazione Rust o chiusura G0. Le boundary d'età,
temporalità, positional constraint e storia richiedono il compiler/runtime
generico e le relative task, senza nuove decisioni in questa matrice.

## Copertura ricevuta

| Trace | Campi | Byte one-based | Missing per kind | Alignment per kind |
|---|---:|---|---|---|
| A | 24 | 1–218 | `{"spaces":14,"exact":10}` | `{"left":14,"right":10}` |
| B1 | 36 | 1–198 | `{"exact":22,"spaces":14}` | `{"right":21,"left":15}` |
| B2 | 5 | 1–41 | `{"exact":3,"spaces":2}` | `{"right":3,"left":2}` |
| C | 11 | 1–45 | `{"exact":10,"spaces":1}` | `{"right":10,"left":1}` |
| D | 7 | 1–39 | `{"exact":6,"spaces":1}` | `{"right":6,"left":1}` |

Totale **83/83 campi**, tutti con missing/alignment raw espliciti.
Le condizioni e i vincoli dichiarati non sono stati eseguiti.

## Provenienza verificabile

Percorsi sotto `K/`; SHA-256 calcolati sui byte ricevuti.

| Fonte | SHA-256 |
|---|---|
| `CAD_REVISED_CANDIDATE/trace_a.json` | `1b7850af503c0f832f3add306a997e2b3a9f072b590d46a34c6f7783dbf00876` |
| `CAD_REVISED_CANDIDATE/trace_b1.json` | `a6726cf3b2365c7b230c2b1e82b0ba00d8ba18d504c1f3aa3ec9c008674238a3` |
| `CAD_REVISED_CANDIDATE/trace_b2.json` | `daa651271ca210485a9cd4b46c469a8364b807fd78adb6089cb6c76cb86cd148` |
| `CAD_REVISED_CANDIDATE/trace_c.json` | `0ffcf4f83bdb391485c9963125dbcb18cbffc5370d46e26e99449d4e52677b9c` |
| `CAD_REVISED_CANDIDATE/trace_d.json` | `47f6d04d94f63ecdbdcce629615f715e0ec3332ff1b25405256c350f4034db32` |
| `CAD_REVISED_CANDIDATE/flow_schema.json` | `74d92f8e88d1eef70dbc21e0b63fa33823c6d3d1b9bfb23803c671ecd46adb16` |
| `CAD_REVISED_CANDIDATE/manifest.json` | `3b220a34cd317b4bf657a9702d56f841f17fc7435986711c3744a3dcdb5d4561` |

## Metadati flow raw

Tutte le chiavi del flow ricevuto eccetto `/traces`; nessuna riscrittura.

<!-- T-P0-03:flow_metadata -->
```json
{
  "canonical_schema_version": "1.0.0",
  "schema_status": "RELEASE_CANDIDATE",
  "flow_id": "CAD",
  "regulatory_version": "2024-06",
  "source": {
    "title": "2025-05-14_FLUSSO_ADI.pdf",
    "internal_heading_update": "2024-06",
    "authority": "Regione del Veneto",
    "source_label_date": "2025-05-14",
    "pdf_pages": 7
  },
  "encoding": {
    "mode": "runtime_required",
    "position_unit": "byte",
    "line_endings": [
      "LF",
      "CRLF"
    ]
  },
  "general_physical_rules": {
    "alphanumeric": {
      "alignment": "left",
      "padding": "space",
      "default_missing": "spaces"
    },
    "numeric": {
      "alignment": "right",
      "padding": "zero",
      "missing": "field-specific"
    },
    "date": {
      "format": "DDMMYYYY",
      "default_missing_for_optional_dates": "spaces"
    },
    "type_T": {
      "physical_equivalent": "alphanumeric",
      "semantic": "positional_bitset",
      "padding": "space"
    }
  },
  "reference_data": [
    {
      "dataset": "cad-icd9-cm-clinical-signs",
      "status": "SOURCE_EXTRACTED_REVIEWED",
      "file": "datasets/cad_icd9_cm_symptoms_page7.json"
    },
    {
      "dataset": "cad-prestation-types",
      "status": "SOURCE_EXTRACTED_REVIEWED",
      "file": "datasets/cad_prestations_type_5_5_page5.json"
    },
    {
      "dataset": "cad-needs-70-positions",
      "status": "SOURCE_EXTRACTED_REVIEWED",
      "file": "datasets/cad_needs_70_positions_page6.json"
    }
  ],
  "schema_revision": "2026-10-16.reviewed-g02-g06"
}
```

## Trace A

Fonte: `K/CAD_REVISED_CANDIDATE/trace_a.json`; note e metadati sorgente: [A-fields.json](A-fields.json), `/fields/i/regulatory_metadata`.
Policy completa: appendice seguente, `/field_policies/i`; contesto: `/trace_context`.

| Campo (tabella con note) | i | Byte (larghezza) | Alias → tipo fisico | Regionale / NSIS / risolta | Presence configurata | Missing raw | Alignment/padding raw |
|---|---:|---|---|---|---|---|---|
| [1.1](A-fields.json) | 0 | 1–3 (3) | `"AN"` → alphanumeric | `"SI"` / `"SI"` / `"SI"` | required; R=0, F=0 | spaces: 3 × 0x20 | left; pad=`" "` (0x20) |
| [1.2](A-fields.json) | 1 | 4–23 (20) | `"AN"` → alphanumeric | `"SI"` / `"SI"` / `"SI"` | required; R=0, F=0 | spaces: 20 × 0x20 | left; pad=`" "` (0x20) |
| [1.3](A-fields.json) | 2 | 24–24 (1) | `"AN"` → alphanumeric | `"SI"` / `"SI"` / `"SI"` | required; R=0, F=0 | spaces: 1 × 0x20 | left; pad=`" "` (0x20) |
| [1.4](A-fields.json) | 3 | 25–52 (28) | `"AN"` → alphanumeric | `"NO (1*)"` / `"NBB"` / `"NBB"` | optional; R=1, F=0 | spaces: 28 × 0x20 | left; pad=`" "` (0x20) |
| [1.5](A-fields.json) | 4 | 53–60 (8) | `"D"` → date | `"NO (1*)"` / `"NBB"` / `"NBB"` | optional; R=1, F=0 | spaces: 8 × 0x20 | left; pad=`" "` (0x20) |
| [1.6](A-fields.json) | 5 | 61–88 (28) | `"N"` → numeric | `"NO"` / `"NO"` / `"NO"` | optional; R=0, F=0 | exact: `"0000000000000000000000000000"` (28 byte) | right; pad=`"0"` (0x30) |
| [1.7](A-fields.json) | 6 | 89–97 (9) | `"N"` → numeric | `"NO"` / `"NO"` / `"NO"` | optional; R=0, F=0 | exact: `"000000000"` (9 byte) | right; pad=`"0"` (0x30) |
| [1.8](A-fields.json) | 7 | 98–127 (30) | `"AN"` → alphanumeric | `"NO"` / `"NO"` / `"NO"` | optional; R=1, F=0 | spaces: 30 × 0x20 | left; pad=`" "` (0x20) |
| [1.9](A-fields.json) | 8 | 128–147 (20) | `"AN"` → alphanumeric | `"NO"` / `"NO"` / `"NO"` | optional; R=1, F=0 | spaces: 20 × 0x20 | left; pad=`" "` (0x20) |
| [1.10](A-fields.json) | 9 | 148–148 (1) | `"N"` → numeric | `"SI"` / `"SI"` / `"SI"` | required; R=0, F=0 | exact: `"0"` (1 byte) | right; pad=`"0"` (0x30) |
| [1.11](A-fields.json) | 10 | 149–156 (8) | `"D"` → date | `"SI"` / `"SI"` / `"SI"` | required; R=0, F=0 | spaces: 8 × 0x20 | left; pad=`" "` (0x20) |
| [1.12](A-fields.json) | 11 | 157–162 (6) | `"AN"` → alphanumeric | `"SI"` / `"SI"` / `"SI"` | required; R=0, F=0 | spaces: 6 × 0x20 | left; pad=`" "` (0x20) |
| [1.13](A-fields.json) | 12 | 163–165 (3) | `"AN"` → alphanumeric | `"SI"` / `"SI"` / `"SI"` | required; R=0, F=0 | spaces: 3 × 0x20 | left; pad=`" "` (0x20) |
| [1.14](A-fields.json) | 13 | 166–171 (6) | `"AN"` → alphanumeric | `"SI"` / `"SI"` / `"SI"` | required; R=0, F=0 | spaces: 6 × 0x20 | left; pad=`" "` (0x20) |
| [1.15](A-fields.json) | 14 | 172–177 (6) | `"AN"` → alphanumeric | `"SI"` / `"SI"` / `"SI"` | required; R=0, F=0 | spaces: 6 × 0x20 | left; pad=`" "` (0x20) |
| [1.16](A-fields.json) | 15 | 178–183 (6) | `"AN"` → alphanumeric | `"SI"` / `"NO"` / `"NBB"` | optional; R=0, F=0 | spaces: 6 × 0x20 | left; pad=`" "` (0x20) |
| [1.17](A-fields.json) | 16 | 184–184 (1) | `"N"` → numeric | `"SI"` / `"NO"` / `"NBB"` | optional; R=0, F=0 | exact: `"0"` (1 byte) | right; pad=`"0"` (0x30) |
| [1.18](A-fields.json) | 17 | 185–200 (16) | `"AN"` → alphanumeric | `"NO"` / `"NO"` / `"NO"` | optional; R=0, F=0 | spaces: 16 × 0x20 | left; pad=`" "` (0x20) |
| [1.19](A-fields.json) | 18 | 201–201 (1) | `"N"` → numeric | `"SI"` / `"SI"` / `"SI"` | required; R=0, F=0 | exact: `"0"` (1 byte) | right; pad=`"0"` (0x30) |
| [1.20](A-fields.json) | 19 | 202–203 (2) | `"N"` → numeric | `"SI"` / `"SI"` / `"SI"` | required; R=0, F=0 | exact: `"00"` (2 byte) | right; pad=`"0"` (0x30) |
| [1.21](A-fields.json) | 20 | 204–204 (1) | `"N"` → numeric | `"SI"` / `"SI"` / `"SI"` | required; R=0, F=0 | exact: `"0"` (1 byte) | right; pad=`"0"` (0x30) |
| [1.22](A-fields.json) | 21 | 205–205 (1) | `"N"` → numeric | `"NO"` / `"NO"` / `"NO"` | optional; R=0, F=0 | exact: `"0"` (1 byte) | right; pad=`"0"` (0x30) |
| [1.23](A-fields.json) | 22 | 206–217 (12) | `"N"` → numeric | `"SI"` / `"SI"` / `"SI"` | required; R=0, F=0 | exact: `"000000000000"` (12 byte) | right; pad=`"0"` (0x30) |
| [1.24](A-fields.json) | 23 | 218–218 (1) | `"N"` → numeric | `"NBB"` / `"NBB"` / `"NBB"` | optional; R=0, F=0 | exact: `"0"` (1 byte) | right; pad=`"0"` (0x30) |

### Appendice raw trace A

Ogni oggetto di `field_policies` è la proiezione **senza modifica di valori**
delle chiavi di policy del campo ricevuto. Le note originali restano
nella tabella collegata; le policy non sono dedotte dalle note.

<!-- T-P0-03:trace:A -->
```json
{
  "trace_context": {
    "canonical_schema_version": "1.0.0",
    "schema_status": "RELEASE_CANDIDATE",
    "flow_id": "CAD",
    "regulatory_version": "2024-06",
    "trace_id": "A",
    "trace_name": "Dati anagrafici",
    "record_length": 218,
    "value_model": {
      "physical_positions": "one_based",
      "raw_value": "exact fixed-width bytes",
      "logical_value": "raw after declared padding removal"
    },
    "record_key": {
      "fields": [
        "1.1",
        "1.23"
      ],
      "uniqueness_scope": "trace",
      "error_code": "E_DUPLICATE_RECORD_KEY"
    },
    "joins": [],
    "named_conditions": {},
    "derived_values": {},
    "repeated_groups": [],
    "documented_non_executable_rules": [
      {
        "id": "CAD-AGE-UNDER18",
        "fields": [
          "1.24"
        ],
        "policy": "requires_engine_extension_or_decision",
        "reason": "Required when year(2.4)-year(1.11)<=17; cross-trace derived-year expression not present in frozen DSL. Implement via generic extension before executable freeze."
      },
      {
        "id": "CAD-IDENTIFIER-CROSSCHECK",
        "fields": [
          "1.2",
          "1.3"
        ],
        "policy": "requires_engine_extension_or_decision",
        "reason": "Deliberately no semantic cross-field identifier check per user decision."
      }
    ],
    "rules": [],
    "reference_data_dependencies": [],
    "freeze_notes": [
      "Normative release candidate; compiler integration and authoritative sign-off needed to set FROZEN.",
      "CAD_HISTORICAL_PROFILE.md executable contract pending generic runtime support."
    ]
  },
  "field_policies": [
    {"id":"1.1","physical":{"start":1,"length":3,"type":"alphanumeric","source_type":"AN","alignment":{"kind":"left","pad":" "}},"missing_value":{"kind":"spaces","raw":"   "},"presence":{"base":"required","required_if":[],"forbidden_if":[]},"constraints":[{"kind":"allowed_values","values":["501","502","503","504","505","506","507","508","509"]}]},
    {"id":"1.2","physical":{"start":4,"length":20,"type":"alphanumeric","source_type":"AN","alignment":{"kind":"left","pad":" "}},"missing_value":{"kind":"spaces","raw":"                    "},"presence":{"base":"required","required_if":[],"forbidden_if":[]},"constraints":[]},
    {"id":"1.3","physical":{"start":24,"length":1,"type":"alphanumeric","source_type":"AN","alignment":{"kind":"left","pad":" "}},"missing_value":{"kind":"spaces","raw":" "},"presence":{"base":"required","required_if":[],"forbidden_if":[]},"constraints":[{"kind":"allowed_values","values":["1","2","3","4","5","6","9"]}]},
    {"id":"1.4","physical":{"start":25,"length":28,"type":"alphanumeric","source_type":"AN","alignment":{"kind":"left","pad":" "}},"missing_value":{"kind":"spaces","raw":"                            "},"presence":{"base":"optional","required_if":[{"eq":{"ref":{"field":"1.3"},"value":"4"}}],"forbidden_if":[]},"constraints":[]},
    {"id":"1.5","physical":{"start":53,"length":8,"type":"date","source_type":"D","alignment":{"kind":"left","pad":" "}},"missing_value":{"kind":"spaces","raw":"        "},"presence":{"base":"optional","required_if":[{"eq":{"ref":{"field":"1.3"},"value":"4"}}],"forbidden_if":[]},"constraints":[{"kind":"date_if_present","format":"DDMMYYYY"}]},
    {"id":"1.6","physical":{"start":61,"length":28,"type":"numeric","source_type":"N","alignment":{"kind":"right","pad":"0"}},"missing_value":{"kind":"exact","raw":"0000000000000000000000000000"},"presence":{"base":"optional","required_if":[],"forbidden_if":[]},"constraints":[{"kind":"digits_only_if_present"}]},
    {"id":"1.7","physical":{"start":89,"length":9,"type":"numeric","source_type":"N","alignment":{"kind":"right","pad":"0"}},"missing_value":{"kind":"exact","raw":"000000000"},"presence":{"base":"optional","required_if":[],"forbidden_if":[]},"constraints":[{"kind":"digits_only_if_present"}]},
    {"id":"1.8","physical":{"start":98,"length":30,"type":"alphanumeric","source_type":"AN","alignment":{"kind":"left","pad":" "}},"missing_value":{"kind":"spaces","raw":"                              "},"presence":{"base":"optional","required_if":[{"ne":{"ref":{"field":"1.3"},"value":"1"}}],"forbidden_if":[]},"constraints":[]},
    {"id":"1.9","physical":{"start":128,"length":20,"type":"alphanumeric","source_type":"AN","alignment":{"kind":"left","pad":" "}},"missing_value":{"kind":"spaces","raw":"                    "},"presence":{"base":"optional","required_if":[{"ne":{"ref":{"field":"1.3"},"value":"1"}}],"forbidden_if":[]},"constraints":[]},
    {"id":"1.10","physical":{"start":148,"length":1,"type":"numeric","source_type":"N","alignment":{"kind":"right","pad":"0"}},"missing_value":{"kind":"exact","raw":"0"},"presence":{"base":"required","required_if":[],"forbidden_if":[]},"constraints":[{"kind":"allowed_values","values":["1","2"]},{"kind":"digits_only_if_present"}]},
    {"id":"1.11","physical":{"start":149,"length":8,"type":"date","source_type":"D","alignment":{"kind":"left","pad":" "}},"missing_value":{"kind":"spaces","raw":"        "},"presence":{"base":"required","required_if":[],"forbidden_if":[]},"constraints":[{"kind":"date","format":"DDMMYYYY"}]},
    {"id":"1.12","physical":{"start":157,"length":6,"type":"alphanumeric","source_type":"AN","alignment":{"kind":"left","pad":" "}},"missing_value":{"kind":"spaces","raw":"      "},"presence":{"base":"required","required_if":[],"forbidden_if":[]},"constraints":[]},
    {"id":"1.13","physical":{"start":163,"length":3,"type":"alphanumeric","source_type":"AN","alignment":{"kind":"left","pad":" "}},"missing_value":{"kind":"spaces","raw":"   "},"presence":{"base":"required","required_if":[],"forbidden_if":[]},"constraints":[]},
    {"id":"1.14","physical":{"start":166,"length":6,"type":"alphanumeric","source_type":"AN","alignment":{"kind":"left","pad":" "}},"missing_value":{"kind":"spaces","raw":"      "},"presence":{"base":"required","required_if":[],"forbidden_if":[]},"constraints":[]},
    {"id":"1.15","physical":{"start":172,"length":6,"type":"alphanumeric","source_type":"AN","alignment":{"kind":"left","pad":" "}},"missing_value":{"kind":"spaces","raw":"      "},"presence":{"base":"required","required_if":[],"forbidden_if":[]},"constraints":[]},
    {"id":"1.16","physical":{"start":178,"length":6,"type":"alphanumeric","source_type":"AN","alignment":{"kind":"left","pad":" "}},"missing_value":{"kind":"spaces","raw":"      "},"presence":{"base":"optional","required_if":[],"forbidden_if":[]},"constraints":[]},
    {"id":"1.17","physical":{"start":184,"length":1,"type":"numeric","source_type":"N","alignment":{"kind":"right","pad":"0"}},"missing_value":{"kind":"exact","raw":"0"},"presence":{"base":"optional","required_if":[],"forbidden_if":[]},"constraints":[{"kind":"digits_only_if_present"}]},
    {"id":"1.18","physical":{"start":185,"length":16,"type":"alphanumeric","source_type":"AN","alignment":{"kind":"left","pad":" "}},"missing_value":{"kind":"spaces","raw":"                "},"presence":{"base":"optional","required_if":[],"forbidden_if":[]},"constraints":[]},
    {"id":"1.19","physical":{"start":201,"length":1,"type":"numeric","source_type":"N","alignment":{"kind":"right","pad":"0"}},"missing_value":{"kind":"exact","raw":"0"},"presence":{"base":"required","required_if":[],"forbidden_if":[]},"constraints":[{"kind":"allowed_values","values":["1","2","3","4","5","9"]},{"kind":"digits_only_if_present"}]},
    {"id":"1.20","physical":{"start":202,"length":2,"type":"numeric","source_type":"N","alignment":{"kind":"right","pad":"0"}},"missing_value":{"kind":"exact","raw":"00"},"presence":{"base":"required","required_if":[],"forbidden_if":[]},"constraints":[{"kind":"digits_only_if_present"}]},
    {"id":"1.21","physical":{"start":204,"length":1,"type":"numeric","source_type":"N","alignment":{"kind":"right","pad":"0"}},"missing_value":{"kind":"exact","raw":"0"},"presence":{"base":"required","required_if":[],"forbidden_if":[]},"constraints":[{"kind":"allowed_values","values":["1","2","3","4","5","6","7"]},{"kind":"digits_only_if_present"}]},
    {"id":"1.22","physical":{"start":205,"length":1,"type":"numeric","source_type":"N","alignment":{"kind":"right","pad":"0"}},"missing_value":{"kind":"exact","raw":"0"},"presence":{"base":"optional","required_if":[],"forbidden_if":[]},"constraints":[{"kind":"allowed_values","values":["1","2","3","4","5","6","7","8"]},{"kind":"digits_only_if_present"}]},
    {"id":"1.23","physical":{"start":206,"length":12,"type":"numeric","source_type":"N","alignment":{"kind":"right","pad":"0"}},"missing_value":{"kind":"exact","raw":"000000000000"},"presence":{"base":"required","required_if":[],"forbidden_if":[]},"constraints":[{"kind":"digits_only_if_present"}]},
    {"id":"1.24","physical":{"start":218,"length":1,"type":"numeric","source_type":"N","alignment":{"kind":"right","pad":"0"}},"missing_value":{"kind":"exact","raw":"0"},"presence":{"base":"optional","required_if":[],"forbidden_if":[]},"constraints":[{"kind":"allowed_values","values":["1","2","3","4","5","9"]},{"kind":"digits_only_if_present"}]}
  ]
}
```

## Trace B1

Fonte: `K/CAD_REVISED_CANDIDATE/trace_b1.json`; note e metadati sorgente: [B1-fields.json](B1-fields.json), `/fields/i/regulatory_metadata`.
Policy completa: appendice seguente, `/field_policies/i`; contesto: `/trace_context`.

| Campo (tabella con note) | i | Byte (larghezza) | Alias → tipo fisico | Regionale / NSIS / risolta | Presence configurata | Missing raw | Alignment/padding raw |
|---|---:|---|---|---|---|---|---|
| [2.1](B1-fields.json) | 0 | 1–12 (12) | `"N"` → numeric | `"SI"` / `"SI"` / `"SI"` | required; R=0, F=0 | exact: `"000000000000"` (12 byte) | right; pad=`"0"` (0x30) |
| [2.2](B1-fields.json) | 1 | 13–14 (2) | `"N"` → numeric | `"SI"` / `"SI"` / `"SI"` | required; R=0, F=0 | exact: `"00"` (2 byte) | right; pad=`"0"` (0x30) |
| [2.3](B1-fields.json) | 2 | 15–15 (1) | `"N"` → numeric | `"SI"` / `"SI"` / `"SI"` | required; R=0, F=0 | exact: `"0"` (1 byte) | right; pad=`"0"` (0x30) |
| [2.4](B1-fields.json) | 3 | 16–23 (8) | `"D"` → date | `"SI"` / `"SI"` / `"SI"` | required; R=0, F=0 | spaces: 8 × 0x20 | left; pad=`" "` (0x20) |
| [2.5](B1-fields.json) | 4 | 24–24 (1) | `"N"` → numeric | `"NBB"` / `"NBB"` / `"NBB"` | optional; R=1, F=1 | exact: `"0"` (1 byte) | right; pad=`"0"` (0x30) |
| [2.6](B1-fields.json) | 5 | 25–32 (8) | `"D"` → date | `"NBB"` / `"SI"` / `"NBB"` | optional; R=0, F=0 | spaces: 8 × 0x20 | left; pad=`" "` (0x20) |
| [2.7](B1-fields.json) | 6 | 33–34 (2) | `"N"` → numeric | `"NBB"` / `"SI"` / `"NBB"` | optional; R=1, F=0 | exact: `"00"` (2 byte) | right; pad=`"0"` (0x30) |
| [2.8](B1-fields.json) | 7 | 35–35 (1) | `"N"` → numeric | `"SI"` / `"SI"` / `"SI"` | required; R=0, F=0 | exact: `"0"` (1 byte) | right; pad=`"0"` (0x30) |
| [2.9](B1-fields.json) | 8 | 36–43 (8) | `"D"` → date | `"SI"` / `"SI"` / `"SI"` | required; R=0, F=0 | spaces: 8 × 0x20 | left; pad=`" "` (0x20) |
| [2.10](B1-fields.json) | 9 | 44–51 (8) | `"D"` → date | `"NO"` / `"NO"` / `"NO"` | optional; R=0, F=0 | spaces: 8 × 0x20 | left; pad=`" "` (0x20) |
| [2.11](B1-fields.json) | 10 | 52–54 (3) | `"AN"` → alphanumeric | `"SI"` / `"SI"` / `"SI"` | required; R=0, F=0 | spaces: 3 × 0x20 | left; pad=`" "` (0x20) |
| [2.12](B1-fields.json) | 11 | 55–57 (3) | `"AN"` → alphanumeric | `"SI"` / `"SI"` / `"SI"` | required; R=0, F=0 | exact: `"000"` (3 byte) | left; pad=`" "` (0x20) |
| [2.13](B1-fields.json) | 12 | 58–58 (1) | `"N"` → numeric | `"SI"` / `"SI"` / `"SI"` | required; R=0, F=0 | exact: `"0"` (1 byte) | right; pad=`"0"` (0x30) |
| [2.14](B1-fields.json) | 13 | 59–59 (1) | `"N"` → numeric | `"SI"` / `"SI"` / `"SI"` | required; R=0, F=0 | exact: `"0"` (1 byte) | right; pad=`"0"` (0x30) |
| [2.15](B1-fields.json) | 14 | 60–60 (1) | `"N"` → numeric | `"NO"` / `"NO"` / `"NO"` | optional; R=0, F=0 | exact: `"0"` (1 byte) | right; pad=`"0"` (0x30) |
| [2.16](B1-fields.json) | 15 | 61–61 (1) | `"N"` → numeric | `"NO"` / `"NO"` / `"NO"` | optional; R=0, F=0 | exact: `"0"` (1 byte) | right; pad=`"0"` (0x30) |
| [2.17](B1-fields.json) | 16 | 62–62 (1) | `"N"` → numeric | `"SI"` / `"SI"` / `"SI"` | optional; R=0, F=0 | exact: `"0"` (1 byte) | right; pad=`"0"` (0x30) |
| [2.18](B1-fields.json) | 17 | 63–63 (1) | `"N"` → numeric | `"SI"` / `"SI"` / `"SI"` | optional; R=0, F=0 | exact: `"0"` (1 byte) | right; pad=`"0"` (0x30) |
| [2.19](B1-fields.json) | 18 | 64–64 (1) | `"N"` → numeric | `"NO"` / `"NO"` / `"NO"` | optional; R=0, F=0 | exact: `"0"` (1 byte) | right; pad=`"0"` (0x30) |
| [2.20](B1-fields.json) | 19 | 65–65 (1) | `"N"` → numeric | `"SI"` / `"SI"` / `"SI"` | required; R=0, F=0 | exact: `"0"` (1 byte) | right; pad=`"0"` (0x30) |
| [2.21](B1-fields.json) | 20 | 66–66 (1) | `"N"` → numeric | `"SI"` / `"SI"` / `"SI"` | required; R=0, F=0 | exact: `"0"` (1 byte) | right; pad=`"0"` (0x30) |
| [2.22](B1-fields.json) | 21 | 67–136 (70) | `"T"` → alphanumeric | `"SI"` / `"SI"` / `"SI"` | required; R=0, F=0 | spaces: 70 × 0x20 | left; pad=`" "` (0x20) |
| [2.23](B1-fields.json) | 22 | 137–137 (1) | `"N"` → numeric | `"SI"` / `"SI"` / `"SI"` | required; R=0, F=0 | exact: `"0"` (1 byte) | right; pad=`"0"` (0x30) |
| [2.24](B1-fields.json) | 23 | 138–149 (12) | `"T"` → alphanumeric | `"NO"` / `"NO"` / `"NO"` | optional; R=0, F=0 | spaces: 12 × 0x20 | left; pad=`" "` (0x20) |
| [2.25](B1-fields.json) | 24 | 150–153 (4) | `"T"` → alphanumeric | `"NO"` / `"NO"` / `"NO"` | optional; R=0, F=0 | spaces: 4 × 0x20 | left; pad=`" "` (0x20) |
| [2.26](B1-fields.json) | 25 | 154–157 (4) | `"T"` → alphanumeric | `"NO"` / `"NO"` / `"NO"` | optional; R=0, F=0 | spaces: 4 × 0x20 | left; pad=`" "` (0x20) |
| [2.27](B1-fields.json) | 26 | 158–158 (1) | `"N"` → numeric | `"NBB"` / `"NBB"` / `"NBB"` | optional; R=1, F=1 | exact: `"0"` (1 byte) | right; pad=`"0"` (0x30) |
| [2.28](B1-fields.json) | 27 | 159–159 (1) | `"N"` → numeric | `"NBB"` / `"NBB"` / `"NBB"` | optional; R=1, F=1 | exact: `"0"` (1 byte) | right; pad=`"0"` (0x30) |
| [2.29](B1-fields.json) | 28 | 160–164 (5) | `"AN"` → alphanumeric | `"NBB"` / `"NBB"` / `"NBB"` | optional; R=1, F=1 | spaces: 5 × 0x20 | left; pad=`" "` (0x20) |
| [2.30](B1-fields.json) | 29 | 165–169 (5) | `"AN"` → alphanumeric | `"FAC"` / `"FAC"` / `"FAC"` | optional; R=0, F=1 | spaces: 5 × 0x20 | left; pad=`" "` (0x20) |
| [2.31](B1-fields.json) | 30 | 170–174 (5) | `"AN"` → alphanumeric | `"FAC"` / `"FAC"` / `"FAC"` | optional; R=0, F=1 | spaces: 5 × 0x20 | left; pad=`" "` (0x20) |
| [2.32](B1-fields.json) | 31 | 175–179 (5) | `"AN"` → alphanumeric | `"FAC"` / `"FAC"` / `"FAC"` | optional; R=0, F=1 | spaces: 5 × 0x20 | left; pad=`" "` (0x20) |
| [2.33](B1-fields.json) | 32 | 180–184 (5) | `"AN"` → alphanumeric | `"FAC"` / `"FAC"` / `"FAC"` | optional; R=0, F=1 | spaces: 5 × 0x20 | left; pad=`" "` (0x20) |
| [2.34](B1-fields.json) | 33 | 185–185 (1) | `"N"` → numeric | `"NBB"` / `"NBB"` / `"NBB"` | optional; R=1, F=1 | exact: `"0"` (1 byte) | right; pad=`"0"` (0x30) |
| [2.35](B1-fields.json) | 34 | 186–186 (1) | `"N"` → numeric | `"NBB"` / `"NBB"` / `"NBB"` | optional; R=1, F=1 | exact: `"0"` (1 byte) | right; pad=`"0"` (0x30) |
| [2.36](B1-fields.json) | 35 | 187–198 (12) | `"N"` → numeric | `"SI"` / `"NO"` / `"NBB"` | required; R=0, F=0 | exact: `"000000000000"` (12 byte) | right; pad=`"0"` (0x30) |

### Appendice raw trace B1

Ogni oggetto di `field_policies` è la proiezione **senza modifica di valori**
delle chiavi di policy del campo ricevuto. Le note originali restano
nella tabella collegata; le policy non sono dedotte dalle note.

<!-- T-P0-03:trace:B1 -->
```json
{
  "trace_context": {
    "canonical_schema_version": "1.0.0",
    "schema_status": "RELEASE_CANDIDATE",
    "flow_id": "CAD",
    "regulatory_version": "2024-06",
    "trace_id": "B1",
    "trace_name": "Valutazione rivalutazione conclusione",
    "record_length": 198,
    "value_model": {
      "physical_positions": "one_based",
      "raw_value": "exact fixed-width bytes",
      "logical_value": "raw after declared padding removal"
    },
    "record_key": {
      "fields": [
        "2.1",
        "2.36"
      ],
      "uniqueness_scope": "trace",
      "error_code": "E_DUPLICATE_RECORD_KEY"
    },
    "joins": [
      {
        "id": "b1_to_a_event",
        "target_trace": "A",
        "source_fields": [
          "2.1"
        ],
        "target_fields": [
          "1.23"
        ],
        "required": true,
        "cardinality": "exactly_one",
        "context_scope": "same_ulss_submission",
        "note": "A primary key additionally contains ULSS 1.1; unique matching by event alone requires scope/uniqueness invariant."
      }
    ],
    "named_conditions": {
      "is_ucpdom": {
        "eq": {
          "ref": {
            "field": "2.3"
          },
          "value": "2"
        }
      },
      "belongs_to_network": {
        "eq": {
          "ref": {
            "field": "2.27"
          },
          "value": "1"
        }
      },
      "has_conclusion": {
        "is_present": {
          "ref": {
            "field": "2.6"
          }
        }
      }
    },
    "derived_values": {},
    "repeated_groups": [],
    "documented_non_executable_rules": [
      {
        "id": "CAD-AGE-3-6",
        "fields": [
          "2.17",
          "2.18"
        ],
        "policy": "requires_engine_extension_or_decision",
        "reason": "Required if year(2.4)-year(1.11)>=3/6; requires cross-trace age-year derived expression."
      },
      {
        "id": "CAD-17",
        "fields": [
          "2.4",
          "2.6",
          "2.7",
          "3.2",
          "3.3",
          "4.2",
          "5.2"
        ],
        "policy": "requires_engine_extension_or_decision",
        "reason": "Administrative closing after 3 calendar months inactivity, calendar-month threshold + pause duration; longitudinal runtime rule. See CAD_HISTORICAL_PROFILE.md."
      },
      {
        "id": "CAD-2.11-12-ICPC",
        "fields": [
          "2.11",
          "2.12"
        ],
        "policy": "requires_engine_extension_or_decision",
        "reason": "No ICPC lookup per user decision; 2.12=000 reserved for absent concomitant diagnosis."
      }
    ],
    "rules": [
      {
        "id": "CAD-R-2.7-UC-10",
        "when": {
          "eq": {
            "ref": {
              "field": "2.3"
            },
            "value": "2"
          }
        },
        "assertions": [
          {
            "kind": "not_allowed_values",
            "field": "2.7",
            "values": [
              "10"
            ]
          }
        ],
        "error_code": "E_CAD_CONCLUSION_CODE"
      },
      {
        "id": "CAD-R-2.7-CD-13",
        "when": {
          "eq": {
            "ref": {
              "field": "2.3"
            },
            "value": "1"
          }
        },
        "assertions": [
          {
            "kind": "not_allowed_values",
            "field": "2.7",
            "values": [
              "13"
            ]
          }
        ],
        "error_code": "E_CAD_CONCLUSION_CODE"
      }
    ],
    "reference_data_dependencies": [
      "cad-icd9-cm-clinical-signs"
    ],
    "freeze_notes": [
      "Normative release candidate; compiler integration and authoritative sign-off needed to set FROZEN.",
      "CAD_HISTORICAL_PROFILE.md executable contract pending generic runtime support.",
      "2.22 position 60 outside UCPDOM remains undecided; no runtime value imposed."
    ]
  },
  "field_policies": [
    {"id":"2.1","physical":{"start":1,"length":12,"type":"numeric","source_type":"N","alignment":{"kind":"right","pad":"0"}},"missing_value":{"kind":"exact","raw":"000000000000"},"presence":{"base":"required","required_if":[],"forbidden_if":[]},"constraints":[{"kind":"digits_only_if_present"}]},
    {"id":"2.2","physical":{"start":13,"length":2,"type":"numeric","source_type":"N","alignment":{"kind":"right","pad":"0"}},"missing_value":{"kind":"exact","raw":"00"},"presence":{"base":"required","required_if":[],"forbidden_if":[]},"constraints":[{"kind":"allowed_values","values":["01","02","03","04","05","06","07","08","09","10","11","12","13","14"]},{"kind":"digits_only_if_present"}]},
    {"id":"2.3","physical":{"start":15,"length":1,"type":"numeric","source_type":"N","alignment":{"kind":"right","pad":"0"}},"missing_value":{"kind":"exact","raw":"0"},"presence":{"base":"required","required_if":[],"forbidden_if":[]},"constraints":[{"kind":"allowed_values","values":["1","2"]},{"kind":"digits_only_if_present"}]},
    {"id":"2.4","physical":{"start":16,"length":8,"type":"date","source_type":"D","alignment":{"kind":"left","pad":" "}},"missing_value":{"kind":"spaces","raw":"        "},"presence":{"base":"required","required_if":[],"forbidden_if":[]},"constraints":[{"kind":"date","format":"DDMMYYYY"}]},
    {"id":"2.5","physical":{"start":24,"length":1,"type":"numeric","source_type":"N","alignment":{"kind":"right","pad":"0"}},"missing_value":{"kind":"exact","raw":"0"},"presence":{"base":"optional","required_if":[{"condition":"is_ucpdom"}],"forbidden_if":[{"ne":{"ref":{"field":"2.3"},"value":"2"}}]},"constraints":[{"kind":"allowed_values","values":["1","2","9"]},{"kind":"digits_only_if_present"}]},
    {"id":"2.6","physical":{"start":25,"length":8,"type":"date","source_type":"D","alignment":{"kind":"left","pad":" "}},"missing_value":{"kind":"spaces","raw":"        "},"presence":{"base":"optional","required_if":[],"forbidden_if":[]},"constraints":[{"kind":"date_if_present","format":"DDMMYYYY"}]},
    {"id":"2.7","physical":{"start":33,"length":2,"type":"numeric","source_type":"N","alignment":{"kind":"right","pad":"0"}},"missing_value":{"kind":"exact","raw":"00"},"presence":{"base":"optional","required_if":[{"condition":"has_conclusion"}],"forbidden_if":[]},"constraints":[{"kind":"allowed_values","values":["01","02","03","04","05","06","07","08","09","10","11","12","13","80","82","88","97","98","99"]},{"kind":"digits_only_if_present"}]},
    {"id":"2.8","physical":{"start":35,"length":1,"type":"numeric","source_type":"N","alignment":{"kind":"right","pad":"0"}},"missing_value":{"kind":"exact","raw":"0"},"presence":{"base":"required","required_if":[],"forbidden_if":[]},"constraints":[{"kind":"allowed_values","values":["1","2"]},{"kind":"digits_only_if_present"}]},
    {"id":"2.9","physical":{"start":36,"length":8,"type":"date","source_type":"D","alignment":{"kind":"left","pad":" "}},"missing_value":{"kind":"spaces","raw":"        "},"presence":{"base":"required","required_if":[],"forbidden_if":[]},"constraints":[{"kind":"date","format":"DDMMYYYY"}]},
    {"id":"2.10","physical":{"start":44,"length":8,"type":"date","source_type":"D","alignment":{"kind":"left","pad":" "}},"missing_value":{"kind":"spaces","raw":"        "},"presence":{"base":"optional","required_if":[],"forbidden_if":[]},"constraints":[{"kind":"date_if_present","format":"DDMMYYYY"}]},
    {"id":"2.11","physical":{"start":52,"length":3,"type":"alphanumeric","source_type":"AN","alignment":{"kind":"left","pad":" "}},"missing_value":{"kind":"spaces","raw":"   "},"presence":{"base":"required","required_if":[],"forbidden_if":[]},"constraints":[]},
    {"id":"2.12","physical":{"start":55,"length":3,"type":"alphanumeric","source_type":"AN","alignment":{"kind":"left","pad":" "}},"missing_value":{"kind":"exact","raw":"000"},"presence":{"base":"required","required_if":[],"forbidden_if":[]},"constraints":[]},
    {"id":"2.13","physical":{"start":58,"length":1,"type":"numeric","source_type":"N","alignment":{"kind":"right","pad":"0"}},"missing_value":{"kind":"exact","raw":"0"},"presence":{"base":"required","required_if":[],"forbidden_if":[]},"constraints":[{"kind":"allowed_values","values":["1","2","3"]},{"kind":"digits_only_if_present"}]},
    {"id":"2.14","physical":{"start":59,"length":1,"type":"numeric","source_type":"N","alignment":{"kind":"right","pad":"0"}},"missing_value":{"kind":"exact","raw":"0"},"presence":{"base":"required","required_if":[],"forbidden_if":[]},"constraints":[{"kind":"allowed_values","values":["1","2","3"]},{"kind":"digits_only_if_present"}]},
    {"id":"2.15","physical":{"start":60,"length":1,"type":"numeric","source_type":"N","alignment":{"kind":"right","pad":"0"}},"missing_value":{"kind":"exact","raw":"0"},"presence":{"base":"optional","required_if":[],"forbidden_if":[]},"constraints":[{"kind":"allowed_values","values":["1","2","3"]},{"kind":"digits_only_if_present"}]},
    {"id":"2.16","physical":{"start":61,"length":1,"type":"numeric","source_type":"N","alignment":{"kind":"right","pad":"0"}},"missing_value":{"kind":"exact","raw":"0"},"presence":{"base":"optional","required_if":[],"forbidden_if":[]},"constraints":[{"kind":"allowed_values","values":["1","2","3"]},{"kind":"digits_only_if_present"}]},
    {"id":"2.17","physical":{"start":62,"length":1,"type":"numeric","source_type":"N","alignment":{"kind":"right","pad":"0"}},"missing_value":{"kind":"exact","raw":"0"},"presence":{"base":"optional","required_if":[],"forbidden_if":[]},"constraints":[{"kind":"allowed_values","values":["1","2","3"]},{"kind":"digits_only_if_present"}]},
    {"id":"2.18","physical":{"start":63,"length":1,"type":"numeric","source_type":"N","alignment":{"kind":"right","pad":"0"}},"missing_value":{"kind":"exact","raw":"0"},"presence":{"base":"optional","required_if":[],"forbidden_if":[]},"constraints":[{"kind":"allowed_values","values":["1","2","3"]},{"kind":"digits_only_if_present"}]},
    {"id":"2.19","physical":{"start":64,"length":1,"type":"numeric","source_type":"N","alignment":{"kind":"right","pad":"0"}},"missing_value":{"kind":"exact","raw":"0"},"presence":{"base":"optional","required_if":[],"forbidden_if":[]},"constraints":[{"kind":"allowed_values","values":["1","2","3"]},{"kind":"digits_only_if_present"}]},
    {"id":"2.20","physical":{"start":65,"length":1,"type":"numeric","source_type":"N","alignment":{"kind":"right","pad":"0"}},"missing_value":{"kind":"exact","raw":"0"},"presence":{"base":"required","required_if":[],"forbidden_if":[]},"constraints":[{"kind":"allowed_values","values":["1","2","3"]},{"kind":"digits_only_if_present"}]},
    {"id":"2.21","physical":{"start":66,"length":1,"type":"numeric","source_type":"N","alignment":{"kind":"right","pad":"0"}},"missing_value":{"kind":"exact","raw":"0"},"presence":{"base":"required","required_if":[],"forbidden_if":[]},"constraints":[{"kind":"allowed_values","values":["1","2","9"]},{"kind":"digits_only_if_present"}]},
    {"id":"2.22","physical":{"start":67,"length":70,"type":"alphanumeric","source_type":"T","alignment":{"kind":"left","pad":" "}},"missing_value":{"kind":"spaces","raw":"                                                                      "},"presence":{"base":"required","required_if":[],"forbidden_if":[]},"constraints":[{"kind":"positional_bitset","positions":{"1-59":{"values":["1","2"]},"60":{"values":["1","2"],"condition":"is_ucpdom","outside_ucpdom":" "},"61-70":{"fixed":"2"}},"source_type":"T","note":"Position 60: UCPDOM => 1 or 2; non-UCPDOM => exactly one ASCII space (user normative formalization, 2026-10-16). Positions 61–70 fixed 2."}]},
    {"id":"2.23","physical":{"start":137,"length":1,"type":"numeric","source_type":"N","alignment":{"kind":"right","pad":"0"}},"missing_value":{"kind":"exact","raw":"0"},"presence":{"base":"required","required_if":[],"forbidden_if":[]},"constraints":[{"kind":"allowed_values","values":["1","2","9"]},{"kind":"digits_only_if_present"}]},
    {"id":"2.24","physical":{"start":138,"length":12,"type":"alphanumeric","source_type":"T","alignment":{"kind":"left","pad":" "}},"missing_value":{"kind":"spaces","raw":"            "},"presence":{"base":"optional","required_if":[],"forbidden_if":[]},"constraints":[{"kind":"positional_bitset","used_positions":[1,2,3,4,5,6,7,8,9,10],"allowed_values":["1","2"],"unused_positions_padding":"space","source_type":"T"}]},
    {"id":"2.25","physical":{"start":150,"length":4,"type":"alphanumeric","source_type":"T","alignment":{"kind":"left","pad":" "}},"missing_value":{"kind":"spaces","raw":"    "},"presence":{"base":"optional","required_if":[],"forbidden_if":[]},"constraints":[{"kind":"positional_bitset","used_positions":[1,2,3,4],"allowed_values":["1","2"],"unused_positions_padding":"space","source_type":"T"}]},
    {"id":"2.26","physical":{"start":154,"length":4,"type":"alphanumeric","source_type":"T","alignment":{"kind":"left","pad":" "}},"missing_value":{"kind":"spaces","raw":"    "},"presence":{"base":"optional","required_if":[],"forbidden_if":[]},"constraints":[{"kind":"positional_bitset","used_positions":[1,2,3,4],"allowed_values":["1","2"],"unused_positions_padding":"space","source_type":"T"}]},
    {"id":"2.27","physical":{"start":158,"length":1,"type":"numeric","source_type":"N","alignment":{"kind":"right","pad":"0"}},"missing_value":{"kind":"exact","raw":"0"},"presence":{"base":"optional","required_if":[{"condition":"is_ucpdom"}],"forbidden_if":[{"ne":{"ref":{"field":"2.3"},"value":"2"}}]},"constraints":[{"kind":"allowed_values","values":["1","2","9"]},{"kind":"digits_only_if_present"}]},
    {"id":"2.28","physical":{"start":159,"length":1,"type":"numeric","source_type":"N","alignment":{"kind":"right","pad":"0"}},"missing_value":{"kind":"exact","raw":"0"},"presence":{"base":"optional","required_if":[{"and":[{"condition":"is_ucpdom"},{"condition":"belongs_to_network"}]}],"forbidden_if":[{"not":{"and":[{"condition":"is_ucpdom"},{"condition":"belongs_to_network"}]}}]},"constraints":[{"kind":"allowed_values","values":["1","2","3","9"]},{"kind":"digits_only_if_present"}]},
    {"id":"2.29","physical":{"start":160,"length":5,"type":"alphanumeric","source_type":"AN","alignment":{"kind":"left","pad":" "}},"missing_value":{"kind":"spaces","raw":"     "},"presence":{"base":"optional","required_if":[{"condition":"is_ucpdom"}],"forbidden_if":[{"ne":{"ref":{"field":"2.3"},"value":"2"}}]},"constraints":[{"kind":"reference_data_if_present","dataset":"cad-icd9-cm-clinical-signs"}]},
    {"id":"2.30","physical":{"start":165,"length":5,"type":"alphanumeric","source_type":"AN","alignment":{"kind":"left","pad":" "}},"missing_value":{"kind":"spaces","raw":"     "},"presence":{"base":"optional","required_if":[],"forbidden_if":[{"ne":{"ref":{"field":"2.3"},"value":"2"}}]},"constraints":[{"kind":"reference_data_if_present","dataset":"cad-icd9-cm-clinical-signs"}]},
    {"id":"2.31","physical":{"start":170,"length":5,"type":"alphanumeric","source_type":"AN","alignment":{"kind":"left","pad":" "}},"missing_value":{"kind":"spaces","raw":"     "},"presence":{"base":"optional","required_if":[],"forbidden_if":[{"ne":{"ref":{"field":"2.3"},"value":"2"}}]},"constraints":[{"kind":"reference_data_if_present","dataset":"cad-icd9-cm-clinical-signs"}]},
    {"id":"2.32","physical":{"start":175,"length":5,"type":"alphanumeric","source_type":"AN","alignment":{"kind":"left","pad":" "}},"missing_value":{"kind":"spaces","raw":"     "},"presence":{"base":"optional","required_if":[],"forbidden_if":[{"ne":{"ref":{"field":"2.3"},"value":"2"}}]},"constraints":[{"kind":"reference_data_if_present","dataset":"cad-icd9-cm-clinical-signs"}]},
    {"id":"2.33","physical":{"start":180,"length":5,"type":"alphanumeric","source_type":"AN","alignment":{"kind":"left","pad":" "}},"missing_value":{"kind":"spaces","raw":"     "},"presence":{"base":"optional","required_if":[],"forbidden_if":[{"ne":{"ref":{"field":"2.3"},"value":"2"}}]},"constraints":[{"kind":"reference_data_if_present","dataset":"cad-icd9-cm-clinical-signs"}]},
    {"id":"2.34","physical":{"start":185,"length":1,"type":"numeric","source_type":"N","alignment":{"kind":"right","pad":"0"}},"missing_value":{"kind":"exact","raw":"0"},"presence":{"base":"optional","required_if":[{"condition":"is_ucpdom"}],"forbidden_if":[{"ne":{"ref":{"field":"2.3"},"value":"2"}}]},"constraints":[{"kind":"allowed_values","values":["1","2","9"]},{"kind":"digits_only_if_present"}]},
    {"id":"2.35","physical":{"start":186,"length":1,"type":"numeric","source_type":"N","alignment":{"kind":"right","pad":"0"}},"missing_value":{"kind":"exact","raw":"0"},"presence":{"base":"optional","required_if":[{"condition":"is_ucpdom"}],"forbidden_if":[{"ne":{"ref":{"field":"2.3"},"value":"2"}}]},"constraints":[{"kind":"allowed_values","values":["1","2","9"]},{"kind":"digits_only_if_present"}]},
    {"id":"2.36","physical":{"start":187,"length":12,"type":"numeric","source_type":"N","alignment":{"kind":"right","pad":"0"}},"missing_value":{"kind":"exact","raw":"000000000000"},"presence":{"base":"required","required_if":[],"forbidden_if":[]},"constraints":[{"kind":"digits_only"}]}
  ]
}
```

## Trace B2

Fonte: `K/CAD_REVISED_CANDIDATE/trace_b2.json`; note e metadati sorgente: [B2-fields.json](B2-fields.json), `/fields/i/regulatory_metadata`.
Policy completa: appendice seguente, `/field_policies/i`; contesto: `/trace_context`.

| Campo (tabella con note) | i | Byte (larghezza) | Alias → tipo fisico | Regionale / NSIS / risolta | Presence configurata | Missing raw | Alignment/padding raw |
|---|---:|---|---|---|---|---|---|
| [3.1](B2-fields.json) | 0 | 1–12 (12) | `"N"` → numeric | `"SI"` / `"SI"` / `"SI"` | required; R=0, F=0 | exact: `"000000000000"` (12 byte) | right; pad=`"0"` (0x30) |
| [3.2](B2-fields.json) | 1 | 13–20 (8) | `"D"` → date | `"SI"` / `"SI"` / `"SI"` | required; R=0, F=0 | spaces: 8 × 0x20 | left; pad=`" "` (0x20) |
| [3.3](B2-fields.json) | 2 | 21–28 (8) | `"D"` → date | `"FAC"` / `"FAC"` / `"FAC"` | optional; R=0, F=0 | spaces: 8 × 0x20 | left; pad=`" "` (0x20) |
| [3.4](B2-fields.json) | 3 | 29–29 (1) | `"N"` → numeric | `"SI"` / `"SI"` / `"SI"` | required; R=0, F=0 | exact: `"0"` (1 byte) | right; pad=`"0"` (0x30) |
| [3.5](B2-fields.json) | 4 | 30–41 (12) | `"N"` → numeric | `"SI"` / `"NO"` / `"NBB"` | required; R=0, F=0 | exact: `"000000000000"` (12 byte) | right; pad=`"0"` (0x30) |

### Appendice raw trace B2

Ogni oggetto di `field_policies` è la proiezione **senza modifica di valori**
delle chiavi di policy del campo ricevuto. Le note originali restano
nella tabella collegata; le policy non sono dedotte dalle note.

<!-- T-P0-03:trace:B2 -->
```json
{
  "trace_context": {
    "canonical_schema_version": "1.0.0",
    "schema_status": "RELEASE_CANDIDATE",
    "flow_id": "CAD",
    "regulatory_version": "2024-06",
    "trace_id": "B2",
    "trace_name": "Sospensioni",
    "record_length": 41,
    "value_model": {
      "physical_positions": "one_based",
      "raw_value": "exact fixed-width bytes",
      "logical_value": "raw after declared padding removal"
    },
    "record_key": {
      "fields": [
        "3.1",
        "3.5"
      ],
      "uniqueness_scope": "trace",
      "error_code": "E_DUPLICATE_RECORD_KEY"
    },
    "joins": [
      {
        "id": "b2_to_a_event",
        "target_trace": "A",
        "source_fields": [
          "3.1"
        ],
        "target_fields": [
          "1.23"
        ],
        "required": true,
        "cardinality": "exactly_one",
        "context_scope": "same_ulss_submission",
        "note": "A primary key additionally contains ULSS 1.1; unique matching by event alone requires scope/uniqueness invariant."
      }
    ],
    "named_conditions": {},
    "derived_values": {},
    "repeated_groups": [],
    "documented_non_executable_rules": [],
    "rules": [],
    "reference_data_dependencies": [],
    "freeze_notes": [
      "Normative release candidate; compiler integration and authoritative sign-off needed to set FROZEN.",
      "CAD_HISTORICAL_PROFILE.md executable contract pending generic runtime support."
    ]
  },
  "field_policies": [
    {"id":"3.1","physical":{"start":1,"length":12,"type":"numeric","source_type":"N","alignment":{"kind":"right","pad":"0"}},"missing_value":{"kind":"exact","raw":"000000000000"},"presence":{"base":"required","required_if":[],"forbidden_if":[]},"constraints":[{"kind":"digits_only_if_present"}]},
    {"id":"3.2","physical":{"start":13,"length":8,"type":"date","source_type":"D","alignment":{"kind":"left","pad":" "}},"missing_value":{"kind":"spaces","raw":"        "},"presence":{"base":"required","required_if":[],"forbidden_if":[]},"constraints":[{"kind":"date","format":"DDMMYYYY"}]},
    {"id":"3.3","physical":{"start":21,"length":8,"type":"date","source_type":"D","alignment":{"kind":"left","pad":" "}},"missing_value":{"kind":"spaces","raw":"        "},"presence":{"base":"optional","required_if":[],"forbidden_if":[]},"constraints":[{"kind":"date_if_present","format":"DDMMYYYY"}]},
    {"id":"3.4","physical":{"start":29,"length":1,"type":"numeric","source_type":"N","alignment":{"kind":"right","pad":"0"}},"missing_value":{"kind":"exact","raw":"0"},"presence":{"base":"required","required_if":[],"forbidden_if":[]},"constraints":[{"kind":"allowed_values","values":["1","2","3","4","7","9"]},{"kind":"digits_only_if_present"}]},
    {"id":"3.5","physical":{"start":30,"length":12,"type":"numeric","source_type":"N","alignment":{"kind":"right","pad":"0"}},"missing_value":{"kind":"exact","raw":"000000000000"},"presence":{"base":"required","required_if":[],"forbidden_if":[]},"constraints":[{"kind":"digits_only"}]}
  ]
}
```

## Trace C

Fonte: `K/CAD_REVISED_CANDIDATE/trace_c.json`; note e metadati sorgente: [C-fields.json](C-fields.json), `/fields/i/regulatory_metadata`.
Policy completa: appendice seguente, `/field_policies/i`; contesto: `/trace_context`.

| Campo (tabella con note) | i | Byte (larghezza) | Alias → tipo fisico | Regionale / NSIS / risolta | Presence configurata | Missing raw | Alignment/padding raw |
|---|---:|---|---|---|---|---|---|
| [4.1](C-fields.json) | 0 | 1–12 (12) | `"N"` → numeric | `"SI"` / `"SI"` / `"SI"` | required; R=0, F=0 | exact: `"000000000000"` (12 byte) | right; pad=`"0"` (0x30) |
| [4.2](C-fields.json) | 1 | 13–20 (8) | `"D"` → date | `"SI"` / `"SI"` / `"SI"` | required; R=0, F=0 | spaces: 8 × 0x20 | left; pad=`" "` (0x20) |
| [4.3](C-fields.json) | 2 | 21–21 (1) | `"N"` → numeric | `"SI"` / `"SI"` / `"SI"` | required; R=0, F=0 | exact: `"0"` (1 byte) | right; pad=`"0"` (0x30) |
| [4.4](C-fields.json) | 3 | 22–22 (1) | `"N"` → numeric | `"SI"` / `"SI"` / `"SI"` | required; R=0, F=0 | exact: `"0"` (1 byte) | right; pad=`"0"` (0x30) |
| [4.5](C-fields.json) | 4 | 23–24 (2) | `"N"` → numeric | `"SI"` / `"SI"` / `"SI"` | required; R=0, F=0 | exact: `"00"` (2 byte) | right; pad=`"0"` (0x30) |
| [4.6](C-fields.json) | 5 | 25–25 (1) | `"N"` → numeric | `"NO"` / `"NO"` / `"NO"` | optional; R=0, F=0 | exact: `"0"` (1 byte) | right; pad=`"0"` (0x30) |
| [4.7](C-fields.json) | 6 | 26–26 (1) | `"N"` → numeric | `"SI"` / `"NO"` / `"NBB"` | optional; R=0, F=0 | exact: `"0"` (1 byte) | right; pad=`"0"` (0x30) |
| [4.8](C-fields.json) | 7 | 27–27 (1) | `"N"` → numeric | `"NO"` / `"NO"` / `"NO"` | optional; R=0, F=0 | exact: `"0"` (1 byte) | right; pad=`"0"` (0x30) |
| [4.9](C-fields.json) | 8 | 28–30 (3) | `"N"` → numeric | `"NO"` / `"NO"` / `"NO"` | optional; R=0, F=1 | exact: `"000"` (3 byte) | right; pad=`"0"` (0x30) |
| [4.10](C-fields.json) | 9 | 31–33 (3) | `"N"` → numeric | `"NO"` / `"NO"` / `"NO"` | optional; R=0, F=1 | exact: `"000"` (3 byte) | right; pad=`"0"` (0x30) |
| [4.11](C-fields.json) | 10 | 34–45 (12) | `"N"` → numeric | `"SI"` / `"NO"` / `"NBB"` | required; R=0, F=0 | exact: `"000000000000"` (12 byte) | right; pad=`"0"` (0x30) |

### Appendice raw trace C

Ogni oggetto di `field_policies` è la proiezione **senza modifica di valori**
delle chiavi di policy del campo ricevuto. Le note originali restano
nella tabella collegata; le policy non sono dedotte dalle note.

<!-- T-P0-03:trace:C -->
```json
{
  "trace_context": {
    "canonical_schema_version": "1.0.0",
    "schema_status": "RELEASE_CANDIDATE",
    "flow_id": "CAD",
    "regulatory_version": "2024-06",
    "trace_id": "C",
    "trace_name": "Accessi",
    "record_length": 45,
    "value_model": {
      "physical_positions": "one_based",
      "raw_value": "exact fixed-width bytes",
      "logical_value": "raw after declared padding removal"
    },
    "record_key": {
      "fields": [
        "4.1",
        "4.2",
        "4.4",
        "4.5",
        "4.11"
      ],
      "uniqueness_scope": "trace",
      "error_code": "E_DUPLICATE_RECORD_KEY"
    },
    "joins": [
      {
        "id": "c_to_a_event",
        "target_trace": "A",
        "source_fields": [
          "4.1"
        ],
        "target_fields": [
          "1.23"
        ],
        "required": true,
        "cardinality": "exactly_one",
        "context_scope": "same_ulss_submission",
        "note": "A primary key additionally contains ULSS 1.1; unique matching by event alone requires scope/uniqueness invariant."
      }
    ],
    "named_conditions": {},
    "derived_values": {},
    "repeated_groups": [],
    "documented_non_executable_rules": [
      {
        "id": "CAD-14",
        "fields": [
          "3.1",
          "3.2",
          "3.3",
          "4.1",
          "4.2",
          "4.7"
        ],
        "policy": "requires_engine_extension_or_decision",
        "reason": "Sourced B2 suspension disallows home access 4.7=1 on interior days; temporal multi-record join context required. See CAD_HISTORICAL_PROFILE.md."
      },
      {
        "id": "CAD-4.5-15",
        "fields": [
          "4.5"
        ],
        "policy": "requires_engine_extension_or_decision",
        "reason": "Code 15 not valid for care episodes starting in 2025; needs cross-trace source date."
      },
      {
        "id": "CAD-4.9-SPECIALIST",
        "fields": [
          "4.5",
          "4.9"
        ],
        "policy": "requires_engine_extension_or_decision",
        "reason": "Branca specialistica only for professional code 4; no official branch lookup supplied."
      },
      {
        "id": "CAD-4.10-OSS",
        "fields": [
          "4.5",
          "4.10"
        ],
        "policy": "requires_engine_extension_or_decision",
        "reason": "Duration only for OSS; relevant OSS codes need source-specific disambiguation before auto-rule."
      }
    ],
    "rules": [],
    "reference_data_dependencies": [],
    "freeze_notes": [
      "Normative release candidate; compiler integration and authoritative sign-off needed to set FROZEN.",
      "CAD_HISTORICAL_PROFILE.md executable contract pending generic runtime support."
    ]
  },
  "field_policies": [
    {"id":"4.1","physical":{"start":1,"length":12,"type":"numeric","source_type":"N","alignment":{"kind":"right","pad":"0"}},"missing_value":{"kind":"exact","raw":"000000000000"},"presence":{"base":"required","required_if":[],"forbidden_if":[]},"constraints":[{"kind":"digits_only_if_present"}]},
    {"id":"4.2","physical":{"start":13,"length":8,"type":"date","source_type":"D","alignment":{"kind":"left","pad":" "}},"missing_value":{"kind":"spaces","raw":"        "},"presence":{"base":"required","required_if":[],"forbidden_if":[]},"constraints":[{"kind":"date","format":"DDMMYYYY"}]},
    {"id":"4.3","physical":{"start":21,"length":1,"type":"numeric","source_type":"N","alignment":{"kind":"right","pad":"0"}},"missing_value":{"kind":"exact","raw":"0"},"presence":{"base":"required","required_if":[],"forbidden_if":[]},"constraints":[{"kind":"digits_only_if_present"}]},
    {"id":"4.4","physical":{"start":22,"length":1,"type":"numeric","source_type":"N","alignment":{"kind":"right","pad":"0"}},"missing_value":{"kind":"exact","raw":"0"},"presence":{"base":"required","required_if":[],"forbidden_if":[]},"constraints":[{"kind":"allowed_values","values":["1","2","9"]},{"kind":"digits_only_if_present"}]},
    {"id":"4.5","physical":{"start":23,"length":2,"type":"numeric","source_type":"N","alignment":{"kind":"right","pad":"0"}},"missing_value":{"kind":"exact","raw":"00"},"presence":{"base":"required","required_if":[],"forbidden_if":[]},"constraints":[{"kind":"allowed_values","values":["01","02","03","04","05","06","07","08","09","11","12","13","14","15","56","60","61","70","71","99"]},{"kind":"digits_only_if_present"}]},
    {"id":"4.6","physical":{"start":25,"length":1,"type":"numeric","source_type":"N","alignment":{"kind":"right","pad":"0"}},"missing_value":{"kind":"exact","raw":"0"},"presence":{"base":"optional","required_if":[],"forbidden_if":[]},"constraints":[{"kind":"allowed_values","values":["1","2","3","9"]},{"kind":"digits_only_if_present"}]},
    {"id":"4.7","physical":{"start":26,"length":1,"type":"numeric","source_type":"N","alignment":{"kind":"right","pad":"0"}},"missing_value":{"kind":"exact","raw":"0"},"presence":{"base":"optional","required_if":[],"forbidden_if":[]},"constraints":[{"kind":"allowed_values","values":["1","2","3","4","5","6"]},{"kind":"digits_only_if_present"}]},
    {"id":"4.8","physical":{"start":27,"length":1,"type":"numeric","source_type":"N","alignment":{"kind":"right","pad":"0"}},"missing_value":{"kind":"exact","raw":"0"},"presence":{"base":"optional","required_if":[],"forbidden_if":[]},"constraints":[{"kind":"allowed_values","values":["1","2","3"]},{"kind":"digits_only_if_present"}]},
    {"id":"4.9","physical":{"start":28,"length":3,"type":"numeric","source_type":"N","alignment":{"kind":"right","pad":"0"}},"missing_value":{"kind":"exact","raw":"000"},"presence":{"base":"optional","required_if":[],"forbidden_if":[{"ne":{"ref":{"field":"4.5"},"value":"4"}}]},"constraints":[{"kind":"digits_only_if_present"}]},
    {"id":"4.10","physical":{"start":31,"length":3,"type":"numeric","source_type":"N","alignment":{"kind":"right","pad":"0"}},"missing_value":{"kind":"exact","raw":"000"},"presence":{"base":"optional","required_if":[],"forbidden_if":[{"not_in":{"ref":{"field":"4.5"},"values":["60","61"]}}]},"constraints":[{"kind":"digits_only_if_present"}]},
    {"id":"4.11","physical":{"start":34,"length":12,"type":"numeric","source_type":"N","alignment":{"kind":"right","pad":"0"}},"missing_value":{"kind":"exact","raw":"000000000000"},"presence":{"base":"required","required_if":[],"forbidden_if":[]},"constraints":[{"kind":"digits_only"}]}
  ]
}
```

## Trace D

Fonte: `K/CAD_REVISED_CANDIDATE/trace_d.json`; note e metadati sorgente: [D-fields.json](D-fields.json), `/fields/i/regulatory_metadata`.
Policy completa: appendice seguente, `/field_policies/i`; contesto: `/trace_context`.

| Campo (tabella con note) | i | Byte (larghezza) | Alias → tipo fisico | Regionale / NSIS / risolta | Presence configurata | Missing raw | Alignment/padding raw |
|---|---:|---|---|---|---|---|---|
| [5.1](D-fields.json) | 0 | 1–12 (12) | `"N"` → numeric | `"SI"` / `"SI"` / `"SI"` | required; R=0, F=0 | exact: `"000000000000"` (12 byte) | right; pad=`"0"` (0x30) |
| [5.2](D-fields.json) | 1 | 13–20 (8) | `"D"` → date | `"SI"` / `"SI"` / `"SI"` | required; R=0, F=0 | spaces: 8 × 0x20 | left; pad=`" "` (0x20) |
| [5.3](D-fields.json) | 2 | 21–21 (1) | `"N"` → numeric | `"SI"` / `"SI"` / `"SI"` | required; R=0, F=0 | exact: `"0"` (1 byte) | right; pad=`"0"` (0x30) |
| [5.4](D-fields.json) | 3 | 22–23 (2) | `"N"` → numeric | `"SI"` / `"SI"` / `"SI"` | required; R=0, F=0 | exact: `"00"` (2 byte) | right; pad=`"0"` (0x30) |
| [5.5](D-fields.json) | 4 | 24–25 (2) | `"N"` → numeric | `"SI"` / `"SI"` / `"SI"` | required; R=0, F=0 | exact: `"00"` (2 byte) | right; pad=`"0"` (0x30) |
| [5.6](D-fields.json) | 5 | 26–27 (2) | `"N"` → numeric | `"FAC"` / `"FAC"` / `"FAC"` | optional; R=0, F=0 | exact: `"00"` (2 byte) | right; pad=`"0"` (0x30) |
| [5.7](D-fields.json) | 6 | 28–39 (12) | `"N"` → numeric | `"SI"` / `"NO"` / `"NBB"` | required; R=0, F=0 | exact: `"000000000000"` (12 byte) | right; pad=`"0"` (0x30) |

### Appendice raw trace D

Ogni oggetto di `field_policies` è la proiezione **senza modifica di valori**
delle chiavi di policy del campo ricevuto. Le note originali restano
nella tabella collegata; le policy non sono dedotte dalle note.

<!-- T-P0-03:trace:D -->
```json
{
  "trace_context": {
    "canonical_schema_version": "1.0.0",
    "schema_status": "RELEASE_CANDIDATE",
    "flow_id": "CAD",
    "regulatory_version": "2024-06",
    "trace_id": "D",
    "trace_name": "Prestazioni",
    "record_length": 39,
    "value_model": {
      "physical_positions": "one_based",
      "raw_value": "exact fixed-width bytes",
      "logical_value": "raw after declared padding removal"
    },
    "record_key": {
      "fields": [
        "5.1",
        "5.2",
        "5.3",
        "5.4",
        "5.7"
      ],
      "uniqueness_scope": "trace",
      "error_code": "E_DUPLICATE_RECORD_KEY"
    },
    "joins": [
      {
        "id": "d_to_a_event",
        "target_trace": "A",
        "source_fields": [
          "5.1"
        ],
        "target_fields": [
          "1.23"
        ],
        "required": true,
        "cardinality": "exactly_one",
        "context_scope": "same_ulss_submission",
        "note": "A primary key additionally contains ULSS 1.1; unique matching by event alone requires scope/uniqueness invariant."
      },
      {
        "id": "d_to_c_access",
        "target_trace": "C",
        "source_fields": [
          "5.1",
          "5.2",
          "5.3",
          "5.4"
        ],
        "target_fields": [
          "4.1",
          "4.2",
          "4.4",
          "4.5"
        ],
        "required": true,
        "cardinality": "at_least_one",
        "error_code": "E_CAD_MISSING_ACCESS_FOR_PROVISION",
        "note": "4.11 and 5.7 are PK components of their respective traces and are NOT cross-trace join fields."
      }
    ],
    "named_conditions": {},
    "derived_values": {},
    "repeated_groups": [],
    "documented_non_executable_rules": [
      {
        "id": "CAD-5.4-15",
        "fields": [
          "5.4"
        ],
        "policy": "requires_engine_extension_or_decision",
        "reason": "Code 15 not valid for care episodes starting in 2025; needs cross-trace source date."
      }
    ],
    "rules": [],
    "reference_data_dependencies": [
      "cad-prestation-types"
    ],
    "freeze_notes": [
      "Normative release candidate; compiler integration and authoritative sign-off needed to set FROZEN.",
      "CAD_HISTORICAL_PROFILE.md executable contract pending generic runtime support."
    ]
  },
  "field_policies": [
    {"id":"5.1","physical":{"start":1,"length":12,"type":"numeric","source_type":"N","alignment":{"kind":"right","pad":"0"}},"missing_value":{"kind":"exact","raw":"000000000000"},"presence":{"base":"required","required_if":[],"forbidden_if":[]},"constraints":[{"kind":"digits_only_if_present"}]},
    {"id":"5.2","physical":{"start":13,"length":8,"type":"date","source_type":"D","alignment":{"kind":"left","pad":" "}},"missing_value":{"kind":"spaces","raw":"        "},"presence":{"base":"required","required_if":[],"forbidden_if":[]},"constraints":[{"kind":"date","format":"DDMMYYYY"}]},
    {"id":"5.3","physical":{"start":21,"length":1,"type":"numeric","source_type":"N","alignment":{"kind":"right","pad":"0"}},"missing_value":{"kind":"exact","raw":"0"},"presence":{"base":"required","required_if":[],"forbidden_if":[]},"constraints":[{"kind":"allowed_values","values":["1","2","9"]},{"kind":"digits_only_if_present"}]},
    {"id":"5.4","physical":{"start":22,"length":2,"type":"numeric","source_type":"N","alignment":{"kind":"right","pad":"0"}},"missing_value":{"kind":"exact","raw":"00"},"presence":{"base":"required","required_if":[],"forbidden_if":[]},"constraints":[{"kind":"allowed_values","values":["01","02","03","04","05","06","07","08","09","11","12","13","14","15","56","60","61","70","71","99"]},{"kind":"digits_only_if_present"}]},
    {"id":"5.5","physical":{"start":24,"length":2,"type":"numeric","source_type":"N","alignment":{"kind":"right","pad":"0"}},"missing_value":{"kind":"exact","raw":"00"},"presence":{"base":"required","required_if":[],"forbidden_if":[]},"constraints":[{"kind":"reference_data","dataset":"cad-prestation-types"},{"kind":"digits_only_if_present"}]},
    {"id":"5.6","physical":{"start":26,"length":2,"type":"numeric","source_type":"N","alignment":{"kind":"right","pad":"0"}},"missing_value":{"kind":"exact","raw":"00"},"presence":{"base":"optional","required_if":[],"forbidden_if":[]},"constraints":[{"kind":"digits_only_if_present"}]},
    {"id":"5.7","physical":{"start":28,"length":12,"type":"numeric","source_type":"N","alignment":{"kind":"right","pad":"0"}},"missing_value":{"kind":"exact","raw":"000000000000"},"presence":{"base":"required","required_if":[],"forbidden_if":[]},"constraints":[{"kind":"digits_only"}]}
  ]
}
```
