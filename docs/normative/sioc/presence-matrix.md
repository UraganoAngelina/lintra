# SIOC — Matrice presenza, assenza e padding (T-P0-03)

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

## Fonte SIOC, alias ed eccezioni

Baseline raw: `K/SCHEMA_INPUTS/SIOC_2026-04-20_FROZEN/`, stato ricevuto
**FROZEN**, DSL `1.0.0`; `K/` è lo starter kit adiacente al repository.
L'XLSX nominato dal flow non è disponibile: questa matrice confronta il
derivato ricevuto con le tabelle approvate, senza attestare le celle originali.
Flow e manifest dichiarano `2026-04-20`; i cinque trace dichiarano
`2025-12-16`. Entrambi i metadati sono preservati qui e nelle tabelle;
la scelta della versione runtime resta fuori scope. Il freeze distingue
baseline 2025 e profilo 2026, senza una nuova ratifica dedotta da questa matrice.

OBB/FAC/NBB sono etichette sorgente distinte dalla policy configurata:
OBB indica obbligatorietà, FAC facoltatività, NBB obbligatorietà dipendente
dalle condizioni/note. Non esiste una sostituzione globale etichetta→base.
Gli AST già ricevuti decidono cosa sia configurato; i casi non formalizzati
restano nelle boundary nominate, senza conversione automatica della prosa.

| Etichetta sorgente | `base: required` ricevuto | `base: optional` ricevuto |
|---|---:|---:|
| OBB | 91 | 2 |
| FAC | 0 | 22 |
| NBB | 9 | 11 |

- `1.2` e `1.6` sono OBB con base optional e condizioni già configurate;
  per `1.6` resta `POLICY-1.6-VENETO-RESIDENCY`, senza inventare una nuova
  condizione Veneto. `1.2` distingue `identifier_unavailable` e anonimato;
  il sentinel raw `AAAAAAAAAAAAAAAAAAAA` è nella regola ricevuta, non assenza.
- `2.8` usa raw `000000` come assenza specifica, pur essendo required:
  il freeze lo dichiara e quindi il placeholder fallisce la presenza.
  `2.27` e `4.27` hanno invece `never_missing` e range da 0 a 999:
  raw `000` rappresenta zero semantico e non assenza fisica.
- `2.13–2.22` hanno assenza fisica di due spazi, dominio che comprende
  `00` e `99`, con `99` annotato «Dato mancante» semanticamente;
  `2.31` usa sei spazi per assenza e `999999` come codice semantico.
- `3.8–3.22`, `3.24–3.32`, `3.34` hanno `never_missing`: i loro zero,
  codici di decesso e placeholder contestuali rimangono nelle regole di trace 3.
  `3.19` conserva il bitset di 20 byte, posizioni 17–20 forzate a `0`,
  e raw tutto zero alla dimissione con decesso. `3.21` è N(2), right/zero,
  con dominio contestuale `00/10/15/25/99` nelle regole ricevute.
- `3.25` NBB è required con raw `0` alla dimissione («Non applicabile»);
  `3.26` richiede raw `0` all'ingresso. `3.27–3.31` NBB sono required con
  raw `00` all'ingresso, `3.32` con raw `0`; i domini alla dimissione
  rimangono differenti. Questi raw non sono dichiarati missing.
- `3.34` conserva NBB sorgente e `base: required`: il freeze registra la
  decisione di compilare ingresso e dimissione, con `999` solo per dimissione
  con decesso e regola d'età distinta. Il disallineamento etichetta/comportamento
  rimane documentato per chiarimento, senza inventare esenzioni.
- `4.5` ha missing exact `0` con meaning «Valore non immesso», dominio `1/2`,
  tipo fisico numeric ma alias sorgente **null**. `4.18/4.19/4.26/4.29`
  e le altre date con `exact` conservano la regola fisica specifica, mentre
  le loro assenze restano spazi. `4.29` può esserci con `4.30` vuoto;
  `4.30` presente richiede `4.29` nella regola ricevuta.
- Alias fisici preservati: `AN` 80, `A` 2, `N` 43, `D` 5, `Data` 4,
  null 1; non si riempie il null dal tipo normalizzato. I riferimenti storici
  `3.48/3.49` nel freeze non cambiano gli ID correnti `3.35/3.36`.

Riferimenti di decisione e limiti:
[FREEZE_REPORT](../../../../LINTRA_STARTER_KIT_FINAL_20261016/SCHEMA_INPUTS/SIOC_2026-04-20_FROZEN/FREEZE_REPORT.md),
[SCHEMA_AUTHORING §§4–8](../../../../LINTRA_STARTER_KIT_FINAL_20261016/SCHEMA_INPUTS/SIOC_2026-04-20_FROZEN/SCHEMA_AUTHORING.md).
I dataset assenti, l'approvazione normativa esterna e il compiler/runtime
non sono convalidati da T-P0-03; G0 resta aperto.

## Copertura ricevuta

| Trace | Campi | Byte one-based | Missing per kind | Alignment per kind |
|---|---:|---|---|---|
| 1 | 18 | 1–206 | `{"spaces":18}` | `{"left":17,"exact":1}` |
| 2 | 50 | 1–271 | `{"spaces":48,"exact":1,"never_missing":1}` | `{"left":45,"exact":3,"right":2}` |
| 3 | 36 | 1–135 | `{"spaces":10,"never_missing":25,"exact":1}` | `{"left":9,"exact":1,"right":26}` |
| 4.1 | 20 | 1–394 | `{"exact":10,"spaces":10}` | `{"right":10,"left":8,"exact":2}` |
| 4.2 | 11 | 1–67 | `{"exact":5,"spaces":5,"never_missing":1}` | `{"right":6,"left":3,"exact":2}` |

Totale **135/135 campi**, tutti con missing/alignment raw espliciti.
Le condizioni e i vincoli dichiarati non sono stati eseguiti.

## Provenienza verificabile

Percorsi sotto `K/`; SHA-256 calcolati sui byte ricevuti.

| Fonte | SHA-256 |
|---|---|
| `SCHEMA_INPUTS/SIOC_2026-04-20_FROZEN/trace_1.json` | `9d10512e716a27387fac12156b94fd2ea5c203de2bb37a1e28e2d0b82cdbb655` |
| `SCHEMA_INPUTS/SIOC_2026-04-20_FROZEN/trace_2.json` | `e0b4b78ec5ac352761a3faf970af9c6b2d315038337a50920e18749cccc41ee1` |
| `SCHEMA_INPUTS/SIOC_2026-04-20_FROZEN/trace_3.json` | `d2e3bcb5d9ed99734a3256a97f1aa05c3c7f6e49f4d3d1205592e2a47683945b` |
| `SCHEMA_INPUTS/SIOC_2026-04-20_FROZEN/trace_4_1.json` | `acbfad7d886fa5c72be6a0e417f7828c228c946ecf000d7ab2591a25de1ad924` |
| `SCHEMA_INPUTS/SIOC_2026-04-20_FROZEN/trace_4_2.json` | `9c3762da808007be9f1433da0e02ed0424f5992a69642de3d4775c38aab8d370` |
| `SCHEMA_INPUTS/SIOC_2026-04-20_FROZEN/flow_schema.json` | `3567bafd3cb9b762becdd46a356454b89df094c7b88c5b1538c97a1bbaf26c45` |
| `SCHEMA_INPUTS/SIOC_2026-04-20_FROZEN/manifest.json` | `9900ab3c0bf9f27abd4bcd6d8289dd0b6ee98a05833605f668a7fb9eb418a0fd` |

## Metadati flow raw

Tutte le chiavi del flow ricevuto eccetto `/traces`; nessuna riscrittura.

<!-- T-P0-03:flow_metadata -->
```json
{
  "canonical_schema_version": "1.0.0",
  "schema_status": "FROZEN",
  "flow_id": "SIOC",
  "regulatory_version": "2026-04-20",
  "source": {
    "title": "SIOC_tracciato_regionale_20260420-definitivo_v05.xlsx",
    "last_update": "2026-04-20",
    "authority": "Regione del Veneto"
  },
  "encoding": {
    "mode": "runtime_required",
    "position_unit": "byte",
    "requirement": "The validator must receive an explicit encoding profile compatible with fixed-width byte slicing. The official XLSX does not declare a file character encoding.",
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
      "missing": "field-specific; never inferred globally when zero is a valid semantic value"
    },
    "date": {
      "format": "DDMMYYYY where declared",
      "default_missing_for_optional_dates": "spaces"
    }
  },
  "reference_data": [
    {
      "dataset": "foreign-country-codes",
      "required": true,
      "status": "PENDING_DATASET"
    },
    {
      "dataset": "health-authority-codes",
      "required": true,
      "status": "PENDING_DATASET"
    },
    {
      "dataset": "hsp11-structure-codes",
      "required": true,
      "status": "PENDING_DATASET"
    },
    {
      "dataset": "icd9-cm-diagnosis-codes",
      "required": true,
      "status": "PENDING_DATASET"
    },
    {
      "dataset": "icd9-cm-procedure-codes",
      "required": true,
      "status": "PENDING_DATASET"
    },
    {
      "dataset": "icpc-codes",
      "required": true,
      "status": "PENDING_DATASET"
    },
    {
      "dataset": "istat-municipality-codes",
      "required": true,
      "status": "PENDING_DATASET"
    },
    {
      "dataset": "italian-region-codes",
      "required": true,
      "status": "PENDING_DATASET"
    },
    {
      "dataset": "regional-entity-manager-codes-dgr-2961-2012",
      "required": true,
      "status": "PENDING_DATASET"
    },
    {
      "dataset": "regional-udo-codes",
      "required": true,
      "status": "PENDING_DATASET"
    },
    {
      "dataset": "ria11-structure-codes",
      "required": true,
      "status": "PENDING_DATASET"
    },
    {
      "dataset": "sts11-structure-codes",
      "required": true,
      "status": "PENDING_DATASET"
    }
  ]
}
```

## Trace 1

Fonte: `K/SCHEMA_INPUTS/SIOC_2026-04-20_FROZEN/trace_1.json`; note e metadati sorgente: [1-fields.json](1-fields.json), `/fields/i/regulatory_metadata`.
Policy completa: appendice seguente, `/field_policies/i`; contesto: `/trace_context`.

| Campo (tabella con note) | i | Byte (larghezza) | Alias → tipo fisico | Sorgente | Presence configurata | Missing raw | Alignment/padding raw |
|---|---:|---|---|---|---|---|---|
| [1.1](1-fields.json) | 0 | 1–20 (20) | `"AN"` → alphanumeric | `"OBB"` | required; R=0, F=0 | spaces: 20 × 0x20 | left; pad=`" "` (0x20) |
| [1.2](1-fields.json) | 1 | 21–40 (20) | `"AN"` → alphanumeric | `"OBB"` | optional; R=1, F=1 | spaces: 20 × 0x20 | left; pad=`" "` (0x20) |
| [1.3](1-fields.json) | 2 | 41–42 (2) | `"AN"` → alphanumeric | `"OBB"` | required; R=0, F=0 | spaces: 2 × 0x20 | left; pad=`" "` (0x20) |
| [1.4](1-fields.json) | 3 | 43–70 (28) | `"AN"` → alphanumeric | `"NBB"` | optional; R=1, F=1 | spaces: 28 × 0x20 | left; pad=`" "` (0x20) |
| [1.5](1-fields.json) | 4 | 71–98 (28) | `"AN"` → alphanumeric | `"FAC"` | optional; R=0, F=1 | spaces: 28 × 0x20 | left; pad=`" "` (0x20) |
| [1.6](1-fields.json) | 5 | 99–107 (9) | `"AN"` → alphanumeric | `"FAC"` | optional; R=0, F=1 | spaces: 9 × 0x20 | left; pad=`" "` (0x20) |
| [1.7](1-fields.json) | 6 | 108–137 (30) | `"AN"` → alphanumeric | `"FAC"` | optional; R=1, F=0 | spaces: 30 × 0x20 | left; pad=`" "` (0x20) |
| [1.8](1-fields.json) | 7 | 138–167 (30) | `"AN"` → alphanumeric | `"FAC"` | optional; R=1, F=0 | spaces: 30 × 0x20 | left; pad=`" "` (0x20) |
| [1.9](1-fields.json) | 8 | 168–175 (8) | `"Data"` → date | `"OBB"` | optional; R=1, F=1 | spaces: 8 × 0x20 | exact |
| [1.10](1-fields.json) | 9 | 176–176 (1) | `"AN"` → alphanumeric | `"OBB"` | required; R=0, F=0 | spaces: 1 × 0x20 | left; pad=`" "` (0x20) |
| [1.11](1-fields.json) | 10 | 177–179 (3) | `"AN"` → alphanumeric | `"OBB"` | required; R=0, F=0 | spaces: 3 × 0x20 | left; pad=`" "` (0x20) |
| [1.12](1-fields.json) | 11 | 180–180 (1) | `"AN"` → alphanumeric | `"OBB"` | required; R=0, F=0 | spaces: 1 × 0x20 | left; pad=`" "` (0x20) |
| [1.13](1-fields.json) | 12 | 181–183 (3) | `"AN"` → alphanumeric | `"OBB"` | required; R=0, F=0 | spaces: 3 × 0x20 | left; pad=`" "` (0x20) |
| [1.14](1-fields.json) | 13 | 184–186 (3) | `"AN"` → alphanumeric | `"OBB"` | required; R=0, F=0 | spaces: 3 × 0x20 | left; pad=`" "` (0x20) |
| [1.15](1-fields.json) | 14 | 187–192 (6) | `"AN"` → alphanumeric | `"OBB"` | required; R=0, F=0 | spaces: 6 × 0x20 | left; pad=`" "` (0x20) |
| [1.16](1-fields.json) | 15 | 193–194 (2) | `"AN"` → alphanumeric | `"OBB"` | required; R=0, F=0 | spaces: 2 × 0x20 | left; pad=`" "` (0x20) |
| [1.17](1-fields.json) | 16 | 195–200 (6) | `"AN"` → alphanumeric | `"OBB"` | required; R=0, F=0 | spaces: 6 × 0x20 | left; pad=`" "` (0x20) |
| [1.18](1-fields.json) | 17 | 201–206 (6) | `"AN"` → alphanumeric | `"OBB"` | required; R=0, F=0 | spaces: 6 × 0x20 | left; pad=`" "` (0x20) |

### Appendice raw trace 1

Ogni oggetto di `field_policies` è la proiezione **senza modifica di valori**
delle chiavi di policy del campo ricevuto. Le note originali restano
nella tabella collegata; le policy non sono dedotte dalle note.

<!-- T-P0-03:trace:1 -->
```json
{
  "trace_context": {
    "canonical_schema_version": "1.0.0",
    "schema_status": "FROZEN",
    "flow_id": "SIOC",
    "regulatory_version": "2025-12-16",
    "trace_id": "1",
    "trace_name": "Dati anagrafici assistito",
    "record_length": 206,
    "value_model": {
      "physical_positions": "one_based",
      "raw_value": "exact fixed-width bytes",
      "logical_value": "raw value after schema-declared padding removal",
      "semantic_comparisons": "logical_value unless assertion explicitly targets raw_value"
    },
    "record_key": {
      "fields": [
        "1.1",
        "1.17",
        "1.18"
      ],
      "uniqueness_scope": "trace",
      "error_code": "E_DUPLICATE_RECORD_KEY"
    },
    "joins": [
      {
        "id": "trace1_to_trace2_episode",
        "target_trace": "2",
        "source_fields": [
          "1.1",
          "1.17",
          "1.18"
        ],
        "target_fields": [
          "2.1",
          "2.49",
          "2.50"
        ],
        "required": false,
        "cardinality": "zero_or_one"
      }
    ],
    "named_conditions": {
      "is_anonymous": {
        "eq": {
          "ref": {
            "field": "1.3"
          },
          "value": "98"
        }
      },
      "identifier_unavailable": {
        "eq": {
          "ref": {
            "field": "1.3"
          },
          "value": "09"
        }
      },
      "is_foreign_resident": {
        "eq": {
          "ref": {
            "field": "1.13"
          },
          "value": "999"
        }
      },
      "is_italian_resident": {
        "ne": {
          "ref": {
            "field": "1.13"
          },
          "value": "999"
        }
      }
    },
    "derived_values": {},
    "repeated_groups": [],
    "documented_non_executable_rules": [
      {
        "id": "POLICY-1.2-CF-SEMANTICS",
        "field": "1.2",
        "policy": "not_enforced_in_v1",
        "reason": "The regional trace defines when CF/ENI/STP/TEAM/other identifiers are used but does not define a confirmed checksum/semantic-validation depth. Canonical v1 validates physical representation and identifier-type-dependent presence only."
      },
      {
        "id": "POLICY-1.6-VENETO-RESIDENCY",
        "field": "1.6",
        "policy": "not_enforced_in_v1",
        "source_note": "Il campo va valorizzato, in combinazione con il codice fiscale, per i soli residenti in Veneto. \nNon compilare nei casi coperti da riservatezza ai sensi di legge",
        "reason": "Operational interpretation of 'in combinazione con il codice fiscale, per i soli residenti in Veneto' was not functionally confirmed; v1 preserves the note but does not infer a resident/identifier-type presence rule."
      }
    ],
    "rules": [
      {
        "id": "R-1.1-ADMISSION-YEAR",
        "when": {
          "is_present": {
            "ref": {
              "field": {
                "trace": "2",
                "field": "2.5",
                "join": "trace1_to_trace2_episode"
              }
            }
          }
        },
        "assertions": [
          {
            "kind": "substring_equals_derived_year",
            "field": "1.1",
            "start": 1,
            "length": 2,
            "source_ref": {
              "field": {
                "trace": "2",
                "field": "2.5",
                "join": "trace1_to_trace2_episode"
              }
            },
            "source_format": "DDMMYYYY",
            "year_digits": 2
          }
        ],
        "error_code": "E010"
      },
      {
        "id": "R-1.2-ANONYMOUS-SENTINEL",
        "when": {
          "condition": "is_anonymous"
        },
        "assertions": [
          {
            "kind": "raw_equals",
            "field": "1.2",
            "value": "AAAAAAAAAAAAAAAAAAAA"
          }
        ],
        "error_code": "E010"
      },
      {
        "id": "R-1.7-ANONYMOUS-NAME",
        "when": {
          "condition": "is_anonymous"
        },
        "assertions": [
          {
            "kind": "logical_equals",
            "field": "1.7",
            "value": "ANONIMO"
          }
        ],
        "error_code": "E010"
      },
      {
        "id": "R-1.8-ANONYMOUS-NAME",
        "when": {
          "condition": "is_anonymous"
        },
        "assertions": [
          {
            "kind": "logical_equals",
            "field": "1.8",
            "value": "ANONIMO"
          }
        ],
        "error_code": "E010"
      },
      {
        "id": "R-1.14-FOREIGN",
        "when": {
          "condition": "is_foreign_resident"
        },
        "assertions": [
          {
            "kind": "logical_equals",
            "field": "1.14",
            "value": "999"
          }
        ],
        "error_code": "E010"
      },
      {
        "id": "R-1.14-ITALIAN",
        "when": {
          "condition": "is_italian_resident"
        },
        "assertions": [
          {
            "kind": "reference_data",
            "field": "1.14",
            "dataset": "health-authority-codes"
          }
        ],
        "error_code": "E011"
      },
      {
        "id": "R-1.15-FOREIGN",
        "when": {
          "and": [
            {
              "condition": "is_foreign_resident"
            },
            {
              "ne": {
                "ref": {
                  "field": "1.15"
                },
                "value": "999999"
              }
            }
          ]
        },
        "assertions": [
          {
            "kind": "composite",
            "field": "1.15",
            "segments": [
              {
                "start": 1,
                "length": 3,
                "equals": "999"
              },
              {
                "start": 4,
                "length": 3,
                "reference_data": "foreign-country-codes"
              }
            ]
          }
        ],
        "error_code": "E010"
      },
      {
        "id": "R-1.15-ITALIAN",
        "when": {
          "and": [
            {
              "condition": "is_italian_resident"
            },
            {
              "ne": {
                "ref": {
                  "field": "1.15"
                },
                "value": "999999"
              }
            }
          ]
        },
        "assertions": [
          {
            "kind": "reference_data",
            "field": "1.15",
            "dataset": "istat-municipality-codes"
          }
        ],
        "error_code": "E011"
      }
    ],
    "reference_data_dependencies": [
      "foreign-country-codes",
      "health-authority-codes",
      "istat-municipality-codes",
      "italian-region-codes",
      "sts11-structure-codes"
    ],
    "freeze_notes": []
  },
  "field_policies": [
    {"id":"1.1","physical":{"start":1,"length":20,"type":"alphanumeric","source_type":"AN","alignment":{"kind":"left","pad":" "}},"missing_value":{"kind":"spaces","raw":"                    "},"presence":{"base":"required","required_if":[],"forbidden_if":[]},"constraints":[{"kind":"logical_length","equals":8},{"kind":"digits_only"},{"kind":"substring_not_equals","target":"logical_value","start":3,"length":6,"value":"000000"}]},
    {"id":"1.2","physical":{"start":21,"length":20,"type":"alphanumeric","source_type":"AN","alignment":{"kind":"left","pad":" "}},"missing_value":{"kind":"spaces","raw":"                    "},"presence":{"base":"optional","required_if":[{"not":{"condition":"identifier_unavailable"}}],"forbidden_if":[{"condition":"identifier_unavailable"}]},"constraints":[]},
    {"id":"1.3","physical":{"start":41,"length":2,"type":"alphanumeric","source_type":"AN","alignment":{"kind":"left","pad":" "}},"missing_value":{"kind":"spaces","raw":"  "},"presence":{"base":"required","required_if":[],"forbidden_if":[]},"constraints":[{"kind":"allowed_values","values":["01","02","03","04","05","09","97","98"]}]},
    {"id":"1.4","physical":{"start":43,"length":28,"type":"alphanumeric","source_type":"AN","alignment":{"kind":"left","pad":" "}},"missing_value":{"kind":"spaces","raw":"                            "},"presence":{"base":"optional","required_if":[{"eq":{"ref":{"field":"1.3"},"value":"04"}}],"forbidden_if":[{"ne":{"ref":{"field":"1.3"},"value":"04"}}]},"constraints":[{"kind":"structure_only"}]},
    {"id":"1.5","physical":{"start":71,"length":28,"type":"alphanumeric","source_type":"AN","alignment":{"kind":"left","pad":" "}},"missing_value":{"kind":"spaces","raw":"                            "},"presence":{"base":"optional","required_if":[],"forbidden_if":[{"condition":"is_anonymous"}]},"constraints":[{"kind":"structure_only"}]},
    {"id":"1.6","physical":{"start":99,"length":9,"type":"alphanumeric","source_type":"AN","alignment":{"kind":"left","pad":" "}},"missing_value":{"kind":"spaces","raw":"         "},"presence":{"base":"optional","required_if":[],"forbidden_if":[{"condition":"is_anonymous"}]},"constraints":[{"kind":"digits_only_if_present"},{"kind":"logical_length_if_present","equals":9}]},
    {"id":"1.7","physical":{"start":108,"length":30,"type":"alphanumeric","source_type":"AN","alignment":{"kind":"left","pad":" "}},"missing_value":{"kind":"spaces","raw":"                              "},"presence":{"base":"optional","required_if":[{"condition":"is_anonymous"}],"forbidden_if":[]},"constraints":[{"kind":"no_lowercase_if_present"}]},
    {"id":"1.8","physical":{"start":138,"length":30,"type":"alphanumeric","source_type":"AN","alignment":{"kind":"left","pad":" "}},"missing_value":{"kind":"spaces","raw":"                              "},"presence":{"base":"optional","required_if":[{"condition":"is_anonymous"}],"forbidden_if":[]},"constraints":[{"kind":"no_lowercase_if_present"}]},
    {"id":"1.9","physical":{"start":168,"length":8,"type":"date","source_type":"Data","alignment":{"kind":"exact"}},"missing_value":{"kind":"spaces","raw":"        "},"presence":{"base":"optional","required_if":[{"not":{"condition":"is_anonymous"}}],"forbidden_if":[{"condition":"is_anonymous"}]},"constraints":[{"kind":"date","format":"DDMMYYYY","calendar_valid":true}]},
    {"id":"1.10","physical":{"start":176,"length":1,"type":"alphanumeric","source_type":"AN","alignment":{"kind":"left","pad":" "}},"missing_value":{"kind":"spaces","raw":" "},"presence":{"base":"required","required_if":[],"forbidden_if":[]},"constraints":[{"kind":"allowed_values","values":["1","2","3"]}]},
    {"id":"1.11","physical":{"start":177,"length":3,"type":"alphanumeric","source_type":"AN","alignment":{"kind":"left","pad":" "}},"missing_value":{"kind":"spaces","raw":"   "},"presence":{"base":"required","required_if":[],"forbidden_if":[]},"constraints":[{"kind":"reference_data_or_values","dataset":"foreign-country-codes","values":["100","999","998"]}]},
    {"id":"1.12","physical":{"start":180,"length":1,"type":"alphanumeric","source_type":"AN","alignment":{"kind":"left","pad":" "}},"missing_value":{"kind":"spaces","raw":" "},"presence":{"base":"required","required_if":[],"forbidden_if":[]},"constraints":[{"kind":"allowed_values","values":["1","2","3","4","5","6","7","9"]}]},
    {"id":"1.13","physical":{"start":181,"length":3,"type":"alphanumeric","source_type":"AN","alignment":{"kind":"left","pad":" "}},"missing_value":{"kind":"spaces","raw":"   "},"presence":{"base":"required","required_if":[],"forbidden_if":[]},"constraints":[{"kind":"reference_data_or_values","dataset":"italian-region-codes","values":["999"]}]},
    {"id":"1.14","physical":{"start":184,"length":3,"type":"alphanumeric","source_type":"AN","alignment":{"kind":"left","pad":" "}},"missing_value":{"kind":"spaces","raw":"   "},"presence":{"base":"required","required_if":[],"forbidden_if":[]},"constraints":[]},
    {"id":"1.15","physical":{"start":187,"length":6,"type":"alphanumeric","source_type":"AN","alignment":{"kind":"left","pad":" "}},"missing_value":{"kind":"spaces","raw":"      "},"presence":{"base":"required","required_if":[],"forbidden_if":[]},"constraints":[]},
    {"id":"1.16","physical":{"start":193,"length":2,"type":"alphanumeric","source_type":"AN","alignment":{"kind":"left","pad":" "}},"missing_value":{"kind":"spaces","raw":"  "},"presence":{"base":"required","required_if":[],"forbidden_if":[]},"constraints":[{"kind":"allowed_values","values":["01","02","03","04","05","06","07","08","09","99"]}]},
    {"id":"1.17","physical":{"start":195,"length":6,"type":"alphanumeric","source_type":"AN","alignment":{"kind":"left","pad":" "}},"missing_value":{"kind":"spaces","raw":"      "},"presence":{"base":"required","required_if":[],"forbidden_if":[]},"constraints":[{"kind":"logical_length","equals":6},{"kind":"digits_only"},{"kind":"composite_segments","segments":[{"name":"region_code","start":1,"length":3,"equals":"050"},{"name":"health_authority_code","start":4,"length":3,"reference_data":"health-authority-codes"}]}]},
    {"id":"1.18","physical":{"start":201,"length":6,"type":"alphanumeric","source_type":"AN","alignment":{"kind":"left","pad":" "}},"missing_value":{"kind":"spaces","raw":"      "},"presence":{"base":"required","required_if":[],"forbidden_if":[]},"constraints":[{"kind":"reference_data","dataset":"sts11-structure-codes"}]}
  ]
}
```

## Trace 2

Fonte: `K/SCHEMA_INPUTS/SIOC_2026-04-20_FROZEN/trace_2.json`; note e metadati sorgente: [2-fields.json](2-fields.json), `/fields/i/regulatory_metadata`.
Policy completa: appendice seguente, `/field_policies/i`; contesto: `/trace_context`.

| Campo (tabella con note) | i | Byte (larghezza) | Alias → tipo fisico | Sorgente | Presence configurata | Missing raw | Alignment/padding raw |
|---|---:|---|---|---|---|---|---|
| [2.1](2-fields.json) | 0 | 1–20 (20) | `"AN"` → alphanumeric | `"OBB"` | required; R=0, F=0 | spaces: 20 × 0x20 | left; pad=`" "` (0x20) |
| [2.2](2-fields.json) | 1 | 21–22 (2) | `"AN"` → alphanumeric | `"OBB"` | required; R=0, F=0 | spaces: 2 × 0x20 | left; pad=`" "` (0x20) |
| [2.3](2-fields.json) | 2 | 23–23 (1) | `"AN"` → alphanumeric | `"OBB"` | required; R=0, F=0 | spaces: 1 × 0x20 | left; pad=`" "` (0x20) |
| [2.4](2-fields.json) | 3 | 24–31 (8) | `"Data"` → date | `"OBB"` | required; R=0, F=0 | spaces: 8 × 0x20 | exact |
| [2.5](2-fields.json) | 4 | 32–39 (8) | `"Data"` → date | `"OBB"` | required; R=0, F=0 | spaces: 8 × 0x20 | exact |
| [2.6](2-fields.json) | 5 | 40–47 (8) | `"Data"` → date | `"OBB"` | required; R=0, F=0 | spaces: 8 × 0x20 | exact |
| [2.7](2-fields.json) | 6 | 48–48 (1) | `"AN"` → alphanumeric | `"OBB"` | required; R=0, F=0 | spaces: 1 × 0x20 | left; pad=`" "` (0x20) |
| [2.8](2-fields.json) | 7 | 49–54 (6) | `"N"` → numeric | `"OBB"` | required; R=0, F=0 | exact: `"000000"` (6 byte) | right; pad=`"0"` (0x30) |
| [2.9](2-fields.json) | 8 | 55–69 (15) | `"AN"` → alphanumeric | `"FAC"` | optional; R=0, F=0 | spaces: 15 × 0x20 | left; pad=`" "` (0x20) |
| [2.10](2-fields.json) | 9 | 70–75 (6) | `"AN"` → alphanumeric | `"NBB"` | optional; R=1, F=0 | spaces: 6 × 0x20 | left; pad=`" "` (0x20) |
| [2.11](2-fields.json) | 10 | 76–76 (1) | `"AN"` → alphanumeric | `"NBB"` | optional; R=1, F=1 | spaces: 1 × 0x20 | left; pad=`" "` (0x20) |
| [2.12](2-fields.json) | 11 | 77–77 (1) | `"AN"` → alphanumeric | `"OBB"` | required; R=0, F=0 | spaces: 1 × 0x20 | left; pad=`" "` (0x20) |
| [2.13](2-fields.json) | 12 | 78–79 (2) | `"AN"` → alphanumeric | `"OBB"` | required; R=0, F=0 | spaces: 2 × 0x20 | left; pad=`" "` (0x20) |
| [2.14](2-fields.json) | 13 | 80–81 (2) | `"AN"` → alphanumeric | `"OBB"` | required; R=0, F=0 | spaces: 2 × 0x20 | left; pad=`" "` (0x20) |
| [2.15](2-fields.json) | 14 | 82–83 (2) | `"AN"` → alphanumeric | `"OBB"` | required; R=0, F=0 | spaces: 2 × 0x20 | left; pad=`" "` (0x20) |
| [2.16](2-fields.json) | 15 | 84–85 (2) | `"AN"` → alphanumeric | `"OBB"` | required; R=0, F=0 | spaces: 2 × 0x20 | left; pad=`" "` (0x20) |
| [2.17](2-fields.json) | 16 | 86–87 (2) | `"AN"` → alphanumeric | `"OBB"` | required; R=0, F=0 | spaces: 2 × 0x20 | left; pad=`" "` (0x20) |
| [2.18](2-fields.json) | 17 | 88–89 (2) | `"AN"` → alphanumeric | `"OBB"` | required; R=0, F=0 | spaces: 2 × 0x20 | left; pad=`" "` (0x20) |
| [2.19](2-fields.json) | 18 | 90–91 (2) | `"AN"` → alphanumeric | `"OBB"` | required; R=0, F=0 | spaces: 2 × 0x20 | left; pad=`" "` (0x20) |
| [2.20](2-fields.json) | 19 | 92–93 (2) | `"AN"` → alphanumeric | `"OBB"` | required; R=0, F=0 | spaces: 2 × 0x20 | left; pad=`" "` (0x20) |
| [2.21](2-fields.json) | 20 | 94–95 (2) | `"AN"` → alphanumeric | `"OBB"` | required; R=0, F=0 | spaces: 2 × 0x20 | left; pad=`" "` (0x20) |
| [2.22](2-fields.json) | 21 | 96–97 (2) | `"AN"` → alphanumeric | `"OBB"` | required; R=0, F=0 | spaces: 2 × 0x20 | left; pad=`" "` (0x20) |
| [2.23](2-fields.json) | 22 | 98–98 (1) | `"AN"` → alphanumeric | `"OBB"` | required; R=0, F=0 | spaces: 1 × 0x20 | left; pad=`" "` (0x20) |
| [2.24](2-fields.json) | 23 | 99–99 (1) | `"AN"` → alphanumeric | `"FAC"` | optional; R=0, F=0 | spaces: 1 × 0x20 | left; pad=`" "` (0x20) |
| [2.25](2-fields.json) | 24 | 100–100 (1) | `"AN"` → alphanumeric | `"OBB"` | required; R=0, F=0 | spaces: 1 × 0x20 | left; pad=`" "` (0x20) |
| [2.26](2-fields.json) | 25 | 101–112 (12) | `"AN"` → alphanumeric | `"FAC"` | optional; R=1, F=1 | spaces: 12 × 0x20 | left; pad=`" "` (0x20) |
| [2.27](2-fields.json) | 26 | 113–115 (3) | `"N"` → numeric | `"OBB"` | required; R=0, F=0 | never_missing | right; pad=`"0"` (0x30) |
| [2.28](2-fields.json) | 27 | 116–117 (2) | `"AN"` → alphanumeric | `"OBB"` | required; R=0, F=0 | spaces: 2 × 0x20 | left; pad=`" "` (0x20) |
| [2.29](2-fields.json) | 28 | 118–120 (3) | `"AN"` → alphanumeric | `"NBB"` | optional; R=1, F=0 | spaces: 3 × 0x20 | left; pad=`" "` (0x20) |
| [2.30](2-fields.json) | 29 | 121–123 (3) | `"AN"` → alphanumeric | `"NBB"` | optional; R=1, F=0 | spaces: 3 × 0x20 | left; pad=`" "` (0x20) |
| [2.31](2-fields.json) | 30 | 124–129 (6) | `"AN"` → alphanumeric | `"NBB"` | optional; R=1, F=0 | spaces: 6 × 0x20 | left; pad=`" "` (0x20) |
| [2.32](2-fields.json) | 31 | 130–136 (7) | `"AN"` → alphanumeric | `"OBB"` | required; R=0, F=0 | spaces: 7 × 0x20 | left; pad=`" "` (0x20) |
| [2.33](2-fields.json) | 32 | 137–143 (7) | `"AN"` → alphanumeric | `"NBB"` | optional; R=0, F=0 | spaces: 7 × 0x20 | left; pad=`" "` (0x20) |
| [2.34](2-fields.json) | 33 | 144–150 (7) | `"AN"` → alphanumeric | `"NBB"` | optional; R=0, F=0 | spaces: 7 × 0x20 | left; pad=`" "` (0x20) |
| [2.35](2-fields.json) | 34 | 151–157 (7) | `"AN"` → alphanumeric | `"NBB"` | optional; R=0, F=0 | spaces: 7 × 0x20 | left; pad=`" "` (0x20) |
| [2.36](2-fields.json) | 35 | 158–164 (7) | `"AN"` → alphanumeric | `"NBB"` | optional; R=0, F=0 | spaces: 7 × 0x20 | left; pad=`" "` (0x20) |
| [2.37](2-fields.json) | 36 | 165–171 (7) | `"AN"` → alphanumeric | `"NBB"` | optional; R=0, F=0 | spaces: 7 × 0x20 | left; pad=`" "` (0x20) |
| [2.38](2-fields.json) | 37 | 172–179 (8) | `"AN"` → alphanumeric | `"FAC"` | optional; R=0, F=0 | spaces: 8 × 0x20 | left; pad=`" "` (0x20) |
| [2.39](2-fields.json) | 38 | 180–187 (8) | `"AN"` → alphanumeric | `"FAC"` | optional; R=0, F=0 | spaces: 8 × 0x20 | left; pad=`" "` (0x20) |
| [2.40](2-fields.json) | 39 | 188–195 (8) | `"AN"` → alphanumeric | `"FAC"` | optional; R=0, F=0 | spaces: 8 × 0x20 | left; pad=`" "` (0x20) |
| [2.41](2-fields.json) | 40 | 196–203 (8) | `"AN"` → alphanumeric | `"FAC"` | optional; R=0, F=0 | spaces: 8 × 0x20 | left; pad=`" "` (0x20) |
| [2.42](2-fields.json) | 41 | 204–211 (8) | `"AN"` → alphanumeric | `"FAC"` | optional; R=0, F=0 | spaces: 8 × 0x20 | left; pad=`" "` (0x20) |
| [2.43](2-fields.json) | 42 | 212–219 (8) | `"AN"` → alphanumeric | `"FAC"` | optional; R=0, F=0 | spaces: 8 × 0x20 | left; pad=`" "` (0x20) |
| [2.44](2-fields.json) | 43 | 220–227 (8) | `"AN"` → alphanumeric | `"FAC"` | optional; R=0, F=0 | spaces: 8 × 0x20 | left; pad=`" "` (0x20) |
| [2.45](2-fields.json) | 44 | 228–235 (8) | `"AN"` → alphanumeric | `"FAC"` | optional; R=0, F=0 | spaces: 8 × 0x20 | left; pad=`" "` (0x20) |
| [2.46](2-fields.json) | 45 | 236–243 (8) | `"AN"` → alphanumeric | `"FAC"` | optional; R=0, F=0 | spaces: 8 × 0x20 | left; pad=`" "` (0x20) |
| [2.47](2-fields.json) | 46 | 244–251 (8) | `"AN"` → alphanumeric | `"FAC"` | optional; R=0, F=0 | spaces: 8 × 0x20 | left; pad=`" "` (0x20) |
| [2.48](2-fields.json) | 47 | 252–259 (8) | `"AN"` → alphanumeric | `"FAC"` | optional; R=0, F=0 | spaces: 8 × 0x20 | left; pad=`" "` (0x20) |
| [2.49](2-fields.json) | 48 | 260–265 (6) | `"AN"` → alphanumeric | `"OBB"` | required; R=0, F=0 | spaces: 6 × 0x20 | left; pad=`" "` (0x20) |
| [2.50](2-fields.json) | 49 | 266–271 (6) | `"AN"` → alphanumeric | `"OBB"` | required; R=0, F=0 | spaces: 6 × 0x20 | left; pad=`" "` (0x20) |

### Appendice raw trace 2

Ogni oggetto di `field_policies` è la proiezione **senza modifica di valori**
delle chiavi di policy del campo ricevuto. Le note originali restano
nella tabella collegata; le policy non sono dedotte dalle note.

<!-- T-P0-03:trace:2 -->
```json
{
  "trace_context": {
    "canonical_schema_version": "1.0.0",
    "schema_status": "FROZEN",
    "flow_id": "SIOC",
    "regulatory_version": "2025-12-16",
    "trace_id": "2",
    "trace_name": "ADT",
    "record_length": 271,
    "value_model": {
      "physical_positions": "one_based",
      "raw_value": "exact fixed-width bytes",
      "logical_value": "raw value after schema-declared padding removal",
      "semantic_comparisons": "logical_value unless assertion explicitly targets raw_value"
    },
    "record_key": {
      "fields": [
        "2.1",
        "2.49",
        "2.50"
      ],
      "uniqueness_scope": "trace",
      "error_code": "E_DUPLICATE_RECORD_KEY"
    },
    "joins": [
      {
        "id": "trace2_to_trace1",
        "target_trace": "1",
        "source_fields": [
          "2.1",
          "2.49",
          "2.50"
        ],
        "target_fields": [
          "1.1",
          "1.17",
          "1.18"
        ],
        "required": true,
        "cardinality": "exactly_one",
        "error_code": "E_MISSING_PARENT_RECORD"
      }
    ],
    "named_conditions": {
      "ssr_fully_pays": {
        "eq": {
          "ref": {
            "field": "2.12"
          },
          "value": "1"
        }
      },
      "self_paying": {
        "eq": {
          "ref": {
            "field": "2.12"
          },
          "value": "2"
        }
      },
      "cot_mediated": {
        "eq": {
          "ref": {
            "field": "2.25"
          },
          "value": "1"
        }
      },
      "cot_not_mediated": {
        "eq": {
          "ref": {
            "field": "2.25"
          },
          "value": "2"
        }
      }
    },
    "derived_values": {
      "sending_region": {
        "kind": "substring",
        "source_ref": {
          "field": {
            "trace": "1",
            "field": "1.17",
            "join": "trace2_to_trace1"
          }
        },
        "start": 1,
        "length": 3
      }
    },
    "repeated_groups": [
      {
        "id": "socio_family_problems",
        "fields": [
          "2.13",
          "2.14",
          "2.15",
          "2.16",
          "2.17",
          "2.18",
          "2.19",
          "2.20",
          "2.21",
          "2.22"
        ],
        "required_slots": [
          "2.13",
          "2.14",
          "2.15",
          "2.16",
          "2.17",
          "2.18",
          "2.19",
          "2.20",
          "2.21",
          "2.22"
        ],
        "optional_slots": [],
        "contiguity_required": false,
        "unique_non_missing": false,
        "unused_slot_value": "99"
      },
      {
        "id": "diagnoses",
        "fields": [
          "2.32",
          "2.33",
          "2.34",
          "2.35",
          "2.36",
          "2.37"
        ],
        "required_slots": [
          "2.32"
        ],
        "optional_slots": [
          "2.33",
          "2.34",
          "2.35",
          "2.36",
          "2.37"
        ],
        "contiguity_required": false,
        "unique_non_missing": true
      },
      {
        "id": "procedures",
        "fields": [
          "2.38",
          "2.39",
          "2.40",
          "2.41",
          "2.42",
          "2.43",
          "2.44",
          "2.45",
          "2.46",
          "2.47",
          "2.48"
        ],
        "required_slots": [],
        "optional_slots": [
          "2.38",
          "2.39",
          "2.40",
          "2.41",
          "2.42",
          "2.43",
          "2.44",
          "2.45",
          "2.46",
          "2.47",
          "2.48"
        ],
        "contiguity_required": false,
        "unique_non_missing": false
      }
    ],
    "documented_non_executable_rules": [],
    "rules": [
      {
        "id": "R-2.1-ADMISSION-YEAR",
        "when": {
          "const": true
        },
        "assertions": [
          {
            "kind": "substring_equals_derived_year",
            "field": "2.1",
            "start": 1,
            "length": 2,
            "source_ref": {
              "field": "2.5"
            },
            "source_format": "DDMMYYYY",
            "year_digits": 2
          }
        ],
        "error_code": "E010"
      },
      {
        "id": "R-2.10-SSR-PAYER",
        "when": {
          "condition": "ssr_fully_pays"
        },
        "assertions": [
          {
            "kind": "equals_concat",
            "field": "2.10",
            "parts": [
              {
                "field": {
                  "trace": "1",
                  "field": "1.13",
                  "join": "trace2_to_trace1"
                }
              },
              {
                "field": {
                  "trace": "1",
                  "field": "1.14",
                  "join": "trace2_to_trace1"
                }
              }
            ]
          }
        ],
        "error_code": "E010"
      },
      {
        "id": "R-2.4-LE-2.5",
        "when": {
          "const": true
        },
        "assertions": [
          {
            "kind": "field_compare",
            "left": {
              "field": "2.4"
            },
            "operator": "<=",
            "right": {
              "field": "2.5"
            },
            "value_type": "date"
          }
        ],
        "error_code": "E_TEMPORAL_ORDER"
      },
      {
        "id": "R-2.5-LE-2.6",
        "when": {
          "const": true
        },
        "assertions": [
          {
            "kind": "field_compare",
            "left": {
              "field": "2.5"
            },
            "operator": "<=",
            "right": {
              "field": "2.6"
            },
            "value_type": "date"
          }
        ],
        "error_code": "E_TEMPORAL_ORDER"
      }
    ],
    "reference_data_dependencies": [
      "health-authority-codes",
      "hsp11-structure-codes",
      "icd9-cm-diagnosis-codes",
      "icd9-cm-procedure-codes",
      "italian-region-codes",
      "regional-udo-codes",
      "ria11-structure-codes",
      "sts11-structure-codes"
    ],
    "freeze_notes": [
      "Field 2.8 uses raw '000000' as the field-specific physical-missing representation, following the regional general numeric rule; because the field is required, it fails presence validation."
    ]
  },
  "field_policies": [
    {"id":"2.1","physical":{"start":1,"length":20,"type":"alphanumeric","source_type":"AN","alignment":{"kind":"left","pad":" "}},"missing_value":{"kind":"spaces","raw":"                    "},"presence":{"base":"required","required_if":[],"forbidden_if":[]},"constraints":[{"kind":"logical_length","equals":8},{"kind":"digits_only"},{"kind":"substring_not_equals","target":"logical_value","start":3,"length":6,"value":"000000"}]},
    {"id":"2.2","physical":{"start":21,"length":2,"type":"alphanumeric","source_type":"AN","alignment":{"kind":"left","pad":" "}},"missing_value":{"kind":"spaces","raw":"  "},"presence":{"base":"required","required_if":[],"forbidden_if":[]},"constraints":[{"kind":"allowed_values","values":["01","02","03","04","05","06","07","08","09","10","11","12","13"]}]},
    {"id":"2.3","physical":{"start":23,"length":1,"type":"alphanumeric","source_type":"AN","alignment":{"kind":"left","pad":" "}},"missing_value":{"kind":"spaces","raw":" "},"presence":{"base":"required","required_if":[],"forbidden_if":[]},"constraints":[{"kind":"allowed_values","values":["1"]}]},
    {"id":"2.4","physical":{"start":24,"length":8,"type":"date","source_type":"Data","alignment":{"kind":"exact"}},"missing_value":{"kind":"spaces","raw":"        "},"presence":{"base":"required","required_if":[],"forbidden_if":[]},"constraints":[{"kind":"date","format":"DDMMYYYY","calendar_valid":true}]},
    {"id":"2.5","physical":{"start":32,"length":8,"type":"date","source_type":"Data","alignment":{"kind":"exact"}},"missing_value":{"kind":"spaces","raw":"        "},"presence":{"base":"required","required_if":[],"forbidden_if":[]},"constraints":[{"kind":"date","format":"DDMMYYYY","calendar_valid":true}]},
    {"id":"2.6","physical":{"start":40,"length":8,"type":"date","source_type":"Data","alignment":{"kind":"exact"}},"missing_value":{"kind":"spaces","raw":"        "},"presence":{"base":"required","required_if":[],"forbidden_if":[]},"constraints":[{"kind":"date","format":"DDMMYYYY","calendar_valid":true}]},
    {"id":"2.7","physical":{"start":48,"length":1,"type":"alphanumeric","source_type":"AN","alignment":{"kind":"left","pad":" "}},"missing_value":{"kind":"spaces","raw":" "},"presence":{"base":"required","required_if":[],"forbidden_if":[]},"constraints":[{"kind":"allowed_values","values":["0","1","2","3","4","5","6","7","8","9"]}]},
    {"id":"2.8","physical":{"start":49,"length":6,"type":"numeric","source_type":"N","alignment":{"kind":"right","pad":"0"}},"missing_value":{"kind":"exact","raw":"000000"},"presence":{"base":"required","required_if":[],"forbidden_if":[]},"constraints":[{"kind":"reference_data","dataset":"regional-udo-codes"}]},
    {"id":"2.9","physical":{"start":55,"length":15,"type":"alphanumeric","source_type":"AN","alignment":{"kind":"left","pad":" "}},"missing_value":{"kind":"spaces","raw":"               "},"presence":{"base":"optional","required_if":[],"forbidden_if":[]},"constraints":[{"kind":"structure_only"}]},
    {"id":"2.10","physical":{"start":70,"length":6,"type":"alphanumeric","source_type":"AN","alignment":{"kind":"left","pad":" "}},"missing_value":{"kind":"spaces","raw":"      "},"presence":{"base":"optional","required_if":[{"condition":"ssr_fully_pays"}],"forbidden_if":[]},"constraints":[]},
    {"id":"2.11","physical":{"start":76,"length":1,"type":"alphanumeric","source_type":"AN","alignment":{"kind":"left","pad":" "}},"missing_value":{"kind":"spaces","raw":" "},"presence":{"base":"optional","required_if":[{"condition":"self_paying"}],"forbidden_if":[{"not":{"condition":"self_paying"}}]},"constraints":[{"kind":"allowed_values","values":["1","2","3","4"]}]},
    {"id":"2.12","physical":{"start":77,"length":1,"type":"alphanumeric","source_type":"AN","alignment":{"kind":"left","pad":" "}},"missing_value":{"kind":"spaces","raw":" "},"presence":{"base":"required","required_if":[],"forbidden_if":[]},"constraints":[{"kind":"allowed_values","values":["1","2"]}]},
    {"id":"2.13","physical":{"start":78,"length":2,"type":"alphanumeric","source_type":"AN","alignment":{"kind":"left","pad":" "}},"missing_value":{"kind":"spaces","raw":"  "},"presence":{"base":"required","required_if":[],"forbidden_if":[]},"constraints":[{"kind":"allowed_values","values":["00","01","02","03","04","05","06","07","08","09","10","99"]}],"semantic_null_values":[{"value":"99","meaning":"Dato mancante"}]},
    {"id":"2.14","physical":{"start":80,"length":2,"type":"alphanumeric","source_type":"AN","alignment":{"kind":"left","pad":" "}},"missing_value":{"kind":"spaces","raw":"  "},"presence":{"base":"required","required_if":[],"forbidden_if":[]},"constraints":[{"kind":"allowed_values","values":["00","01","02","03","04","05","06","07","08","09","10","99"]}],"semantic_null_values":[{"value":"99","meaning":"Dato mancante"}]},
    {"id":"2.15","physical":{"start":82,"length":2,"type":"alphanumeric","source_type":"AN","alignment":{"kind":"left","pad":" "}},"missing_value":{"kind":"spaces","raw":"  "},"presence":{"base":"required","required_if":[],"forbidden_if":[]},"constraints":[{"kind":"allowed_values","values":["00","01","02","03","04","05","06","07","08","09","10","99"]}],"semantic_null_values":[{"value":"99","meaning":"Dato mancante"}]},
    {"id":"2.16","physical":{"start":84,"length":2,"type":"alphanumeric","source_type":"AN","alignment":{"kind":"left","pad":" "}},"missing_value":{"kind":"spaces","raw":"  "},"presence":{"base":"required","required_if":[],"forbidden_if":[]},"constraints":[{"kind":"allowed_values","values":["00","01","02","03","04","05","06","07","08","09","10","99"]}],"semantic_null_values":[{"value":"99","meaning":"Dato mancante"}]},
    {"id":"2.17","physical":{"start":86,"length":2,"type":"alphanumeric","source_type":"AN","alignment":{"kind":"left","pad":" "}},"missing_value":{"kind":"spaces","raw":"  "},"presence":{"base":"required","required_if":[],"forbidden_if":[]},"constraints":[{"kind":"allowed_values","values":["00","01","02","03","04","05","06","07","08","09","10","99"]}],"semantic_null_values":[{"value":"99","meaning":"Dato mancante"}]},
    {"id":"2.18","physical":{"start":88,"length":2,"type":"alphanumeric","source_type":"AN","alignment":{"kind":"left","pad":" "}},"missing_value":{"kind":"spaces","raw":"  "},"presence":{"base":"required","required_if":[],"forbidden_if":[]},"constraints":[{"kind":"allowed_values","values":["00","01","02","03","04","05","06","07","08","09","10","99"]}],"semantic_null_values":[{"value":"99","meaning":"Dato mancante"}]},
    {"id":"2.19","physical":{"start":90,"length":2,"type":"alphanumeric","source_type":"AN","alignment":{"kind":"left","pad":" "}},"missing_value":{"kind":"spaces","raw":"  "},"presence":{"base":"required","required_if":[],"forbidden_if":[]},"constraints":[{"kind":"allowed_values","values":["00","01","02","03","04","05","06","07","08","09","10","99"]}],"semantic_null_values":[{"value":"99","meaning":"Dato mancante"}]},
    {"id":"2.20","physical":{"start":92,"length":2,"type":"alphanumeric","source_type":"AN","alignment":{"kind":"left","pad":" "}},"missing_value":{"kind":"spaces","raw":"  "},"presence":{"base":"required","required_if":[],"forbidden_if":[]},"constraints":[{"kind":"allowed_values","values":["00","01","02","03","04","05","06","07","08","09","10","99"]}],"semantic_null_values":[{"value":"99","meaning":"Dato mancante"}]},
    {"id":"2.21","physical":{"start":94,"length":2,"type":"alphanumeric","source_type":"AN","alignment":{"kind":"left","pad":" "}},"missing_value":{"kind":"spaces","raw":"  "},"presence":{"base":"required","required_if":[],"forbidden_if":[]},"constraints":[{"kind":"allowed_values","values":["00","01","02","03","04","05","06","07","08","09","10","99"]}],"semantic_null_values":[{"value":"99","meaning":"Dato mancante"}]},
    {"id":"2.22","physical":{"start":96,"length":2,"type":"alphanumeric","source_type":"AN","alignment":{"kind":"left","pad":" "}},"missing_value":{"kind":"spaces","raw":"  "},"presence":{"base":"required","required_if":[],"forbidden_if":[]},"constraints":[{"kind":"allowed_values","values":["00","01","02","03","04","05","06","07","08","09","10","99"]}],"semantic_null_values":[{"value":"99","meaning":"Dato mancante"}]},
    {"id":"2.23","physical":{"start":98,"length":1,"type":"alphanumeric","source_type":"AN","alignment":{"kind":"left","pad":" "}},"missing_value":{"kind":"spaces","raw":" "},"presence":{"base":"required","required_if":[],"forbidden_if":[]},"constraints":[{"kind":"allowed_values","values":["1","2","3","4","5"]}]},
    {"id":"2.24","physical":{"start":99,"length":1,"type":"alphanumeric","source_type":"AN","alignment":{"kind":"left","pad":" "}},"missing_value":{"kind":"spaces","raw":" "},"presence":{"base":"optional","required_if":[],"forbidden_if":[]},"constraints":[{"kind":"allowed_values","values":["1","2","3","4","5","9"]}]},
    {"id":"2.25","physical":{"start":100,"length":1,"type":"alphanumeric","source_type":"AN","alignment":{"kind":"left","pad":" "}},"missing_value":{"kind":"spaces","raw":" "},"presence":{"base":"required","required_if":[],"forbidden_if":[]},"constraints":[{"kind":"allowed_values","values":["1","2"]}]},
    {"id":"2.26","physical":{"start":101,"length":12,"type":"alphanumeric","source_type":"AN","alignment":{"kind":"left","pad":" "}},"missing_value":{"kind":"spaces","raw":"            "},"presence":{"base":"optional","required_if":[{"condition":"cot_mediated"}],"forbidden_if":[{"condition":"cot_not_mediated"}]},"constraints":[{"kind":"composite_segments_if_present","segments":[{"name":"region","start":1,"length":3,"reference_data":"italian-region-codes"},{"name":"health_authority","start":4,"length":3,"reference_data":"health-authority-codes"},{"name":"sts11","start":7,"length":6,"reference_data":"sts11-structure-codes"}]}]},
    {"id":"2.27","physical":{"start":113,"length":3,"type":"numeric","source_type":"N","alignment":{"kind":"right","pad":"0"}},"missing_value":{"kind":"never_missing"},"presence":{"base":"required","required_if":[],"forbidden_if":[]},"constraints":[{"kind":"digits_only"},{"kind":"numeric_range","min":0,"max":999}]},
    {"id":"2.28","physical":{"start":116,"length":2,"type":"alphanumeric","source_type":"AN","alignment":{"kind":"left","pad":" "}},"missing_value":{"kind":"spaces","raw":"  "},"presence":{"base":"required","required_if":[],"forbidden_if":[]},"constraints":[{"kind":"allowed_values","values":["01","02","03","04","05","06","07","08","99"]}]},
    {"id":"2.29","physical":{"start":118,"length":3,"type":"alphanumeric","source_type":"AN","alignment":{"kind":"left","pad":" "}},"missing_value":{"kind":"spaces","raw":"   "},"presence":{"base":"optional","required_if":[{"not_in":{"ref":{"field":"2.2"},"values":["01","02"]}}],"forbidden_if":[]},"constraints":[{"kind":"reference_data_or_values","dataset":"italian-region-codes","values":["999"]}]},
    {"id":"2.30","physical":{"start":121,"length":3,"type":"alphanumeric","source_type":"AN","alignment":{"kind":"left","pad":" "}},"missing_value":{"kind":"spaces","raw":"   "},"presence":{"base":"optional","required_if":[{"and":[{"ne":{"ref":{"field":"2.29"},"value":"999"}},{"not_in":{"ref":{"field":"2.2"},"values":["01","02"]}},{"derived_eq":{"ref":{"field":"2.29"},"derived_value":"sending_region"}}]}],"forbidden_if":[]},"constraints":[{"kind":"reference_data","dataset":"health-authority-codes"}]},
    {"id":"2.31","physical":{"start":124,"length":6,"type":"alphanumeric","source_type":"AN","alignment":{"kind":"left","pad":" "}},"missing_value":{"kind":"spaces","raw":"      "},"presence":{"base":"optional","required_if":[{"and":[{"ne":{"ref":{"field":"2.29"},"value":"999"}},{"not_in":{"ref":{"field":"2.2"},"values":["01","02","04","09"]}},{"derived_eq":{"ref":{"field":"2.29"},"derived_value":"sending_region"}}]}],"forbidden_if":[]},"constraints":[{"kind":"reference_data_or_values","datasets":["hsp11-structure-codes","ria11-structure-codes","sts11-structure-codes"],"values":["999999"]}],"semantic_null_values":[{"value":"999999","meaning":"Dato mancante"}]},
    {"id":"2.32","physical":{"start":130,"length":7,"type":"alphanumeric","source_type":"AN","alignment":{"kind":"left","pad":" "}},"missing_value":{"kind":"spaces","raw":"       "},"presence":{"base":"required","required_if":[],"forbidden_if":[]},"constraints":[{"kind":"reference_data","dataset":"icd9-cm-diagnosis-codes","metadata_checks":{"sex":true,"age":true}}]},
    {"id":"2.33","physical":{"start":137,"length":7,"type":"alphanumeric","source_type":"AN","alignment":{"kind":"left","pad":" "}},"missing_value":{"kind":"spaces","raw":"       "},"presence":{"base":"optional","required_if":[],"forbidden_if":[]},"constraints":[{"kind":"reference_data_if_present","dataset":"icd9-cm-diagnosis-codes","metadata_checks":{"sex":true,"age":true}}]},
    {"id":"2.34","physical":{"start":144,"length":7,"type":"alphanumeric","source_type":"AN","alignment":{"kind":"left","pad":" "}},"missing_value":{"kind":"spaces","raw":"       "},"presence":{"base":"optional","required_if":[],"forbidden_if":[]},"constraints":[{"kind":"reference_data_if_present","dataset":"icd9-cm-diagnosis-codes","metadata_checks":{"sex":true,"age":true}}]},
    {"id":"2.35","physical":{"start":151,"length":7,"type":"alphanumeric","source_type":"AN","alignment":{"kind":"left","pad":" "}},"missing_value":{"kind":"spaces","raw":"       "},"presence":{"base":"optional","required_if":[],"forbidden_if":[]},"constraints":[{"kind":"reference_data_if_present","dataset":"icd9-cm-diagnosis-codes","metadata_checks":{"sex":true,"age":true}}]},
    {"id":"2.36","physical":{"start":158,"length":7,"type":"alphanumeric","source_type":"AN","alignment":{"kind":"left","pad":" "}},"missing_value":{"kind":"spaces","raw":"       "},"presence":{"base":"optional","required_if":[],"forbidden_if":[]},"constraints":[{"kind":"reference_data_if_present","dataset":"icd9-cm-diagnosis-codes","metadata_checks":{"sex":true,"age":true}}]},
    {"id":"2.37","physical":{"start":165,"length":7,"type":"alphanumeric","source_type":"AN","alignment":{"kind":"left","pad":" "}},"missing_value":{"kind":"spaces","raw":"       "},"presence":{"base":"optional","required_if":[],"forbidden_if":[]},"constraints":[{"kind":"reference_data_if_present","dataset":"icd9-cm-diagnosis-codes","metadata_checks":{"sex":true,"age":true}}]},
    {"id":"2.38","physical":{"start":172,"length":8,"type":"alphanumeric","source_type":"AN","alignment":{"kind":"left","pad":" "}},"missing_value":{"kind":"spaces","raw":"        "},"presence":{"base":"optional","required_if":[],"forbidden_if":[]},"constraints":[{"kind":"reference_data_if_present","dataset":"icd9-cm-procedure-codes"}]},
    {"id":"2.39","physical":{"start":180,"length":8,"type":"alphanumeric","source_type":"AN","alignment":{"kind":"left","pad":" "}},"missing_value":{"kind":"spaces","raw":"        "},"presence":{"base":"optional","required_if":[],"forbidden_if":[]},"constraints":[{"kind":"reference_data_if_present","dataset":"icd9-cm-procedure-codes"}]},
    {"id":"2.40","physical":{"start":188,"length":8,"type":"alphanumeric","source_type":"AN","alignment":{"kind":"left","pad":" "}},"missing_value":{"kind":"spaces","raw":"        "},"presence":{"base":"optional","required_if":[],"forbidden_if":[]},"constraints":[{"kind":"reference_data_if_present","dataset":"icd9-cm-procedure-codes"}]},
    {"id":"2.41","physical":{"start":196,"length":8,"type":"alphanumeric","source_type":"AN","alignment":{"kind":"left","pad":" "}},"missing_value":{"kind":"spaces","raw":"        "},"presence":{"base":"optional","required_if":[],"forbidden_if":[]},"constraints":[{"kind":"reference_data_if_present","dataset":"icd9-cm-procedure-codes"}]},
    {"id":"2.42","physical":{"start":204,"length":8,"type":"alphanumeric","source_type":"AN","alignment":{"kind":"left","pad":" "}},"missing_value":{"kind":"spaces","raw":"        "},"presence":{"base":"optional","required_if":[],"forbidden_if":[]},"constraints":[{"kind":"reference_data_if_present","dataset":"icd9-cm-procedure-codes"}]},
    {"id":"2.43","physical":{"start":212,"length":8,"type":"alphanumeric","source_type":"AN","alignment":{"kind":"left","pad":" "}},"missing_value":{"kind":"spaces","raw":"        "},"presence":{"base":"optional","required_if":[],"forbidden_if":[]},"constraints":[{"kind":"reference_data_if_present","dataset":"icd9-cm-procedure-codes"}]},
    {"id":"2.44","physical":{"start":220,"length":8,"type":"alphanumeric","source_type":"AN","alignment":{"kind":"left","pad":" "}},"missing_value":{"kind":"spaces","raw":"        "},"presence":{"base":"optional","required_if":[],"forbidden_if":[]},"constraints":[{"kind":"reference_data_if_present","dataset":"icd9-cm-procedure-codes"}]},
    {"id":"2.45","physical":{"start":228,"length":8,"type":"alphanumeric","source_type":"AN","alignment":{"kind":"left","pad":" "}},"missing_value":{"kind":"spaces","raw":"        "},"presence":{"base":"optional","required_if":[],"forbidden_if":[]},"constraints":[{"kind":"reference_data_if_present","dataset":"icd9-cm-procedure-codes"}]},
    {"id":"2.46","physical":{"start":236,"length":8,"type":"alphanumeric","source_type":"AN","alignment":{"kind":"left","pad":" "}},"missing_value":{"kind":"spaces","raw":"        "},"presence":{"base":"optional","required_if":[],"forbidden_if":[]},"constraints":[{"kind":"reference_data_if_present","dataset":"icd9-cm-procedure-codes"}]},
    {"id":"2.47","physical":{"start":244,"length":8,"type":"alphanumeric","source_type":"AN","alignment":{"kind":"left","pad":" "}},"missing_value":{"kind":"spaces","raw":"        "},"presence":{"base":"optional","required_if":[],"forbidden_if":[]},"constraints":[{"kind":"reference_data_if_present","dataset":"icd9-cm-procedure-codes"}]},
    {"id":"2.48","physical":{"start":252,"length":8,"type":"alphanumeric","source_type":"AN","alignment":{"kind":"left","pad":" "}},"missing_value":{"kind":"spaces","raw":"        "},"presence":{"base":"optional","required_if":[],"forbidden_if":[]},"constraints":[{"kind":"reference_data_if_present","dataset":"icd9-cm-procedure-codes"}]},
    {"id":"2.49","physical":{"start":260,"length":6,"type":"alphanumeric","source_type":"AN","alignment":{"kind":"left","pad":" "}},"missing_value":{"kind":"spaces","raw":"      "},"presence":{"base":"required","required_if":[],"forbidden_if":[]},"constraints":[{"kind":"logical_length","equals":6},{"kind":"digits_only"},{"kind":"composite_segments","segments":[{"name":"region_code","start":1,"length":3,"equals":"050"},{"name":"health_authority_code","start":4,"length":3,"reference_data":"health-authority-codes"}]}]},
    {"id":"2.50","physical":{"start":266,"length":6,"type":"alphanumeric","source_type":"AN","alignment":{"kind":"left","pad":" "}},"missing_value":{"kind":"spaces","raw":"      "},"presence":{"base":"required","required_if":[],"forbidden_if":[]},"constraints":[{"kind":"reference_data","dataset":"sts11-structure-codes"}]}
  ]
}
```

## Trace 3

Fonte: `K/SCHEMA_INPUTS/SIOC_2026-04-20_FROZEN/trace_3.json`; note e metadati sorgente: [3-fields.json](3-fields.json), `/fields/i/regulatory_metadata`.
Policy completa: appendice seguente, `/field_policies/i`; contesto: `/trace_context`.

| Campo (tabella con note) | i | Byte (larghezza) | Alias → tipo fisico | Sorgente | Presence configurata | Missing raw | Alignment/padding raw |
|---|---:|---|---|---|---|---|---|
| [3.1](3-fields.json) | 0 | 1–20 (20) | `"AN"` → alphanumeric | `"OBB"` | required; R=0, F=0 | spaces: 20 × 0x20 | left; pad=`" "` (0x20) |
| [3.2](3-fields.json) | 1 | 21–40 (20) | `"AN"` → alphanumeric | `"OBB"` | required; R=0, F=0 | spaces: 20 × 0x20 | left; pad=`" "` (0x20) |
| [3.3](3-fields.json) | 2 | 41–41 (1) | `"AN"` → alphanumeric | `"OBB"` | required; R=0, F=0 | spaces: 1 × 0x20 | left; pad=`" "` (0x20) |
| [3.4](3-fields.json) | 3 | 42–49 (8) | `"D"` → date | `"OBB"` | required; R=0, F=0 | spaces: 8 × 0x20 | exact |
| [3.5](3-fields.json) | 4 | 50–52 (3) | `"AN"` → alphanumeric | `"OBB"` | required; R=0, F=0 | spaces: 3 × 0x20 | left; pad=`" "` (0x20) |
| [3.6](3-fields.json) | 5 | 53–55 (3) | `"AN"` → alphanumeric | `"FAC"` | optional; R=0, F=0 | spaces: 3 × 0x20 | left; pad=`" "` (0x20) |
| [3.7](3-fields.json) | 6 | 56–58 (3) | `"AN"` → alphanumeric | `"FAC"` | optional; R=0, F=0 | spaces: 3 × 0x20 | left; pad=`" "` (0x20) |
| [3.8](3-fields.json) | 7 | 59–60 (2) | `"N"` → numeric | `"OBB"` | required; R=0, F=0 | never_missing | right; pad=`"0"` (0x30) |
| [3.9](3-fields.json) | 8 | 61–61 (1) | `"N"` → numeric | `"OBB"` | required; R=0, F=0 | never_missing | right; pad=`"0"` (0x30) |
| [3.10](3-fields.json) | 9 | 62–62 (1) | `"N"` → numeric | `"OBB"` | required; R=0, F=0 | never_missing | right; pad=`"0"` (0x30) |
| [3.11](3-fields.json) | 10 | 63–64 (2) | `"N"` → numeric | `"OBB"` | required; R=0, F=0 | never_missing | right; pad=`"0"` (0x30) |
| [3.12](3-fields.json) | 11 | 65–66 (2) | `"N"` → numeric | `"OBB"` | required; R=0, F=0 | never_missing | right; pad=`"0"` (0x30) |
| [3.13](3-fields.json) | 12 | 67–68 (2) | `"N"` → numeric | `"OBB"` | required; R=0, F=0 | never_missing | right; pad=`"0"` (0x30) |
| [3.14](3-fields.json) | 13 | 69–70 (2) | `"N"` → numeric | `"OBB"` | required; R=0, F=0 | never_missing | right; pad=`"0"` (0x30) |
| [3.15](3-fields.json) | 14 | 71–72 (2) | `"N"` → numeric | `"OBB"` | required; R=0, F=0 | never_missing | right; pad=`"0"` (0x30) |
| [3.16](3-fields.json) | 15 | 73–74 (2) | `"N"` → numeric | `"OBB"` | required; R=0, F=0 | never_missing | right; pad=`"0"` (0x30) |
| [3.17](3-fields.json) | 16 | 75–75 (1) | `"N"` → numeric | `"OBB"` | required; R=0, F=0 | never_missing | right; pad=`"0"` (0x30) |
| [3.18](3-fields.json) | 17 | 76–77 (2) | `"N"` → numeric | `"OBB"` | required; R=0, F=0 | never_missing | right; pad=`"0"` (0x30) |
| [3.19](3-fields.json) | 18 | 78–97 (20) | `"N"` → numeric | `"OBB"` | required; R=0, F=0 | never_missing | right; pad=`"0"` (0x30) |
| [3.20](3-fields.json) | 19 | 98–99 (2) | `"N"` → numeric | `"OBB"` | required; R=0, F=0 | never_missing | right; pad=`"0"` (0x30) |
| [3.21](3-fields.json) | 20 | 100–101 (2) | `"N"` → numeric | `"OBB"` | required; R=0, F=0 | never_missing | right; pad=`"0"` (0x30) |
| [3.22](3-fields.json) | 21 | 102–103 (2) | `"N"` → numeric | `"OBB"` | required; R=0, F=0 | never_missing | right; pad=`"0"` (0x30) |
| [3.23](3-fields.json) | 22 | 104–104 (1) | `"N"` → numeric | `"OBB"` | required; R=0, F=0 | exact: `"0"` (1 byte) | right; pad=`"0"` (0x30) |
| [3.24](3-fields.json) | 23 | 105–106 (2) | `"N"` → numeric | `"OBB"` | required; R=0, F=0 | never_missing | right; pad=`"0"` (0x30) |
| [3.25](3-fields.json) | 24 | 107–107 (1) | `"N"` → numeric | `"NBB"` | required; R=0, F=0 | never_missing | right; pad=`"0"` (0x30) |
| [3.26](3-fields.json) | 25 | 108–108 (1) | `"N"` → numeric | `"NBB"` | required; R=0, F=0 | never_missing | right; pad=`"0"` (0x30) |
| [3.27](3-fields.json) | 26 | 109–110 (2) | `"N"` → numeric | `"NBB"` | required; R=0, F=0 | never_missing | right; pad=`"0"` (0x30) |
| [3.28](3-fields.json) | 27 | 111–112 (2) | `"N"` → numeric | `"NBB"` | required; R=0, F=0 | never_missing | right; pad=`"0"` (0x30) |
| [3.29](3-fields.json) | 28 | 113–114 (2) | `"N"` → numeric | `"NBB"` | required; R=0, F=0 | never_missing | right; pad=`"0"` (0x30) |
| [3.30](3-fields.json) | 29 | 115–116 (2) | `"N"` → numeric | `"NBB"` | required; R=0, F=0 | never_missing | right; pad=`"0"` (0x30) |
| [3.31](3-fields.json) | 30 | 117–118 (2) | `"N"` → numeric | `"NBB"` | required; R=0, F=0 | never_missing | right; pad=`"0"` (0x30) |
| [3.32](3-fields.json) | 31 | 119–119 (1) | `"N"` → numeric | `"NBB"` | required; R=0, F=0 | never_missing | right; pad=`"0"` (0x30) |
| [3.33](3-fields.json) | 32 | 120–120 (1) | `"AN"` → alphanumeric | `"OBB"` | required; R=0, F=0 | spaces: 1 × 0x20 | left; pad=`" "` (0x20) |
| [3.34](3-fields.json) | 33 | 121–123 (3) | `"N"` → numeric | `"NBB"` | required; R=0, F=0 | never_missing | right; pad=`"0"` (0x30) |
| [3.35](3-fields.json) | 34 | 124–129 (6) | `"AN"` → alphanumeric | `"OBB"` | required; R=0, F=0 | spaces: 6 × 0x20 | left; pad=`" "` (0x20) |
| [3.36](3-fields.json) | 35 | 130–135 (6) | `"AN"` → alphanumeric | `"OBB"` | required; R=0, F=0 | spaces: 6 × 0x20 | left; pad=`" "` (0x20) |

### Appendice raw trace 3

Ogni oggetto di `field_policies` è la proiezione **senza modifica di valori**
delle chiavi di policy del campo ricevuto. Le note originali restano
nella tabella collegata; le policy non sono dedotte dalle note.

<!-- T-P0-03:trace:3 -->
```json
{
  "trace_context": {
    "canonical_schema_version": "1.0.0",
    "schema_status": "FROZEN",
    "flow_id": "SIOC",
    "regulatory_version": "2025-12-16",
    "trace_id": "3",
    "trace_name": "Valutazione",
    "record_length": 135,
    "value_model": {
      "physical_positions": "one_based",
      "raw_value": "exact fixed-width bytes",
      "logical_value": "raw value after schema-declared padding removal",
      "semantic_comparisons": "logical_value unless assertion explicitly targets raw_value"
    },
    "record_key": {
      "fields": [
        "3.1",
        "3.2",
        "3.35",
        "3.36"
      ],
      "uniqueness_scope": "trace",
      "error_code": "E_DUPLICATE_RECORD_KEY"
    },
    "joins": [
      {
        "id": "trace3_to_trace2",
        "target_trace": "2",
        "source_fields": [
          "3.1",
          "3.35",
          "3.36"
        ],
        "target_fields": [
          "2.1",
          "2.49",
          "2.50"
        ],
        "required": true,
        "cardinality": "exactly_one",
        "error_code": "E_MISSING_PARENT_RECORD"
      },
      {
        "id": "trace3_to_trace1",
        "target_trace": "1",
        "source_fields": [
          "3.1",
          "3.35",
          "3.36"
        ],
        "target_fields": [
          "1.1",
          "1.17",
          "1.18"
        ],
        "required": true,
        "cardinality": "exactly_one",
        "error_code": "E_MISSING_PARENT_RECORD"
      }
    ],
    "named_conditions": {
      "is_admission_evaluation": {
        "eq": {
          "ref": {
            "field": "3.3"
          },
          "value": "1"
        }
      },
      "is_discharge_evaluation": {
        "eq": {
          "ref": {
            "field": "3.3"
          },
          "value": "2"
        }
      },
      "is_deceased_episode": {
        "eq": {
          "ref": {
            "field": {
              "trace": "2",
              "field": "2.7",
              "join": "trace3_to_trace2"
            }
          },
          "value": "8"
        }
      },
      "is_death_discharge_evaluation": {
        "and": [
          {
            "condition": "is_discharge_evaluation"
          },
          {
            "condition": "is_deceased_episode"
          }
        ]
      }
    },
    "derived_values": {
      "age_at_admission_years": {
        "kind": "age_in_completed_years",
        "birth_date_ref": {
          "field": {
            "trace": "1",
            "field": "1.9",
            "join": "trace3_to_trace1"
          }
        },
        "reference_date_ref": {
          "field": {
            "trace": "2",
            "field": "2.5",
            "join": "trace3_to_trace2"
          }
        },
        "date_format": "DDMMYYYY"
      }
    },
    "repeated_groups": [],
    "documented_non_executable_rules": [
      {
        "id": "POLICY-3.5-COVID-R83",
        "field": "3.5",
        "policy": "not_enforced_in_v1",
        "source_note": "Vedi valore del codice ICPC, assegnato alla patologia prevalente, in scheda SVaMA aggiornata (DGR 96/2025, Allegato A). Per pazienti COVID utilizzare obbligatoriamente nella valutazione all'ingresso il codice ICPC: R83-Altre infezioni del sistema respiratorio",
        "reason": "COVID status is an internal clinical semantic convention and is not reliably derivable from the fixed-width SIOC flow."
      }
    ],
    "rules": [
      {
        "id": "R-3.1-ADMISSION-YEAR",
        "when": {
          "const": true
        },
        "assertions": [
          {
            "kind": "substring_equals_derived_year",
            "field": "3.1",
            "start": 1,
            "length": 2,
            "source_ref": {
              "field": {
                "trace": "2",
                "field": "2.5",
                "join": "trace3_to_trace2"
              }
            },
            "source_format": "DDMMYYYY",
            "year_digits": 2
          }
        ],
        "error_code": "E010"
      },
      {
        "id": "R-3.8-DEATH",
        "when": {
          "condition": "is_death_discharge_evaluation"
        },
        "assertions": [
          {
            "kind": "raw_equals",
            "field": "3.8",
            "value": "99"
          }
        ],
        "error_code": "E010"
      },
      {
        "id": "R-3.8-NORMAL",
        "when": {
          "not": {
            "condition": "is_death_discharge_evaluation"
          }
        },
        "assertions": [
          {
            "kind": "allowed_values",
            "field": "3.8",
            "values": [
              "00",
              "02",
              "05",
              "08",
              "10"
            ]
          }
        ],
        "error_code": "E010"
      },
      {
        "id": "R-3.9-DEATH",
        "when": {
          "condition": "is_death_discharge_evaluation"
        },
        "assertions": [
          {
            "kind": "raw_equals",
            "field": "3.9",
            "value": "9"
          }
        ],
        "error_code": "E010"
      },
      {
        "id": "R-3.9-NORMAL",
        "when": {
          "not": {
            "condition": "is_death_discharge_evaluation"
          }
        },
        "assertions": [
          {
            "kind": "allowed_values",
            "field": "3.9",
            "values": [
              "0",
              "1",
              "2",
              "4",
              "5"
            ]
          }
        ],
        "error_code": "E010"
      },
      {
        "id": "R-3.10-DEATH",
        "when": {
          "condition": "is_death_discharge_evaluation"
        },
        "assertions": [
          {
            "kind": "raw_equals",
            "field": "3.10",
            "value": "9"
          }
        ],
        "error_code": "E010"
      },
      {
        "id": "R-3.10-NORMAL",
        "when": {
          "not": {
            "condition": "is_death_discharge_evaluation"
          }
        },
        "assertions": [
          {
            "kind": "allowed_values",
            "field": "3.10",
            "values": [
              "0",
              "1",
              "2",
              "4",
              "5"
            ]
          }
        ],
        "error_code": "E010"
      },
      {
        "id": "R-3.11-DEATH",
        "when": {
          "condition": "is_death_discharge_evaluation"
        },
        "assertions": [
          {
            "kind": "raw_equals",
            "field": "3.11",
            "value": "99"
          }
        ],
        "error_code": "E010"
      },
      {
        "id": "R-3.11-NORMAL",
        "when": {
          "not": {
            "condition": "is_death_discharge_evaluation"
          }
        },
        "assertions": [
          {
            "kind": "allowed_values",
            "field": "3.11",
            "values": [
              "00",
              "02",
              "05",
              "08",
              "10"
            ]
          }
        ],
        "error_code": "E010"
      },
      {
        "id": "R-3.12-DEATH",
        "when": {
          "condition": "is_death_discharge_evaluation"
        },
        "assertions": [
          {
            "kind": "raw_equals",
            "field": "3.12",
            "value": "99"
          }
        ],
        "error_code": "E010"
      },
      {
        "id": "R-3.12-NORMAL",
        "when": {
          "not": {
            "condition": "is_death_discharge_evaluation"
          }
        },
        "assertions": [
          {
            "kind": "allowed_values",
            "field": "3.12",
            "values": [
              "00",
              "02",
              "05",
              "08",
              "10"
            ]
          }
        ],
        "error_code": "E010"
      },
      {
        "id": "R-3.13-DEATH",
        "when": {
          "condition": "is_death_discharge_evaluation"
        },
        "assertions": [
          {
            "kind": "raw_equals",
            "field": "3.13",
            "value": "99"
          }
        ],
        "error_code": "E010"
      },
      {
        "id": "R-3.13-NORMAL",
        "when": {
          "not": {
            "condition": "is_death_discharge_evaluation"
          }
        },
        "assertions": [
          {
            "kind": "allowed_values",
            "field": "3.13",
            "values": [
              "00",
              "02",
              "05",
              "08",
              "10"
            ]
          }
        ],
        "error_code": "E010"
      },
      {
        "id": "R-3.14-DEATH",
        "when": {
          "condition": "is_death_discharge_evaluation"
        },
        "assertions": [
          {
            "kind": "raw_equals",
            "field": "3.14",
            "value": "99"
          }
        ],
        "error_code": "E010"
      },
      {
        "id": "R-3.14-NORMAL",
        "when": {
          "not": {
            "condition": "is_death_discharge_evaluation"
          }
        },
        "assertions": [
          {
            "kind": "allowed_values",
            "field": "3.14",
            "values": [
              "00",
              "02",
              "05",
              "08",
              "10"
            ]
          }
        ],
        "error_code": "E010"
      },
      {
        "id": "R-3.15-DEATH",
        "when": {
          "condition": "is_death_discharge_evaluation"
        },
        "assertions": [
          {
            "kind": "raw_equals",
            "field": "3.15",
            "value": "99"
          }
        ],
        "error_code": "E010"
      },
      {
        "id": "R-3.15-NORMAL",
        "when": {
          "not": {
            "condition": "is_death_discharge_evaluation"
          }
        },
        "assertions": [
          {
            "kind": "allowed_values",
            "field": "3.15",
            "values": [
              "00",
              "03",
              "07",
              "12",
              "15"
            ]
          }
        ],
        "error_code": "E010"
      },
      {
        "id": "R-3.16-DEATH",
        "when": {
          "condition": "is_death_discharge_evaluation"
        },
        "assertions": [
          {
            "kind": "raw_equals",
            "field": "3.16",
            "value": "99"
          }
        ],
        "error_code": "E010"
      },
      {
        "id": "R-3.16-NORMAL",
        "when": {
          "not": {
            "condition": "is_death_discharge_evaluation"
          }
        },
        "assertions": [
          {
            "kind": "allowed_values",
            "field": "3.16",
            "values": [
              "00",
              "03",
              "07",
              "10",
              "11",
              "12",
              "14",
              "15"
            ]
          }
        ],
        "error_code": "E010"
      },
      {
        "id": "R-3.18-DEATH",
        "when": {
          "condition": "is_death_discharge_evaluation"
        },
        "assertions": [
          {
            "kind": "raw_equals",
            "field": "3.18",
            "value": "99"
          }
        ],
        "error_code": "E010"
      },
      {
        "id": "R-3.18-NORMAL",
        "when": {
          "not": {
            "condition": "is_death_discharge_evaluation"
          }
        },
        "assertions": [
          {
            "kind": "allowed_values",
            "field": "3.18",
            "values": [
              "00",
              "02",
              "05",
              "08",
              "10"
            ]
          }
        ],
        "error_code": "E010"
      },
      {
        "id": "R-3.20-DEATH",
        "when": {
          "condition": "is_death_discharge_evaluation"
        },
        "assertions": [
          {
            "kind": "raw_equals",
            "field": "3.20",
            "value": "99"
          }
        ],
        "error_code": "E010"
      },
      {
        "id": "R-3.20-NORMAL",
        "when": {
          "not": {
            "condition": "is_death_discharge_evaluation"
          }
        },
        "assertions": [
          {
            "kind": "allowed_values",
            "field": "3.20",
            "values": [
              "00",
              "05",
              "10",
              "15",
              "20",
              "25",
              "30",
              "35",
              "40",
              "45",
              "50",
              "55",
              "60",
              "65",
              "70",
              "75",
              "80"
            ]
          }
        ],
        "error_code": "E010"
      },
      {
        "id": "R-3.21-DEATH",
        "when": {
          "condition": "is_death_discharge_evaluation"
        },
        "assertions": [
          {
            "kind": "raw_equals",
            "field": "3.21",
            "value": "99"
          }
        ],
        "error_code": "E010"
      },
      {
        "id": "R-3.21-NORMAL",
        "when": {
          "not": {
            "condition": "is_death_discharge_evaluation"
          }
        },
        "assertions": [
          {
            "kind": "allowed_values",
            "field": "3.21",
            "values": [
              "00",
              "10",
              "15",
              "25"
            ]
          }
        ],
        "error_code": "E010"
      },
      {
        "id": "R-3.23-DEATH",
        "when": {
          "condition": "is_death_discharge_evaluation"
        },
        "assertions": [
          {
            "kind": "raw_equals",
            "field": "3.23",
            "value": "9"
          }
        ],
        "error_code": "E010"
      },
      {
        "id": "R-3.23-NORMAL",
        "when": {
          "not": {
            "condition": "is_death_discharge_evaluation"
          }
        },
        "assertions": [
          {
            "kind": "allowed_values",
            "field": "3.23",
            "values": [
              "1",
              "2",
              "3"
            ]
          }
        ],
        "error_code": "E010"
      },
      {
        "id": "R-3.19-DEATH",
        "when": {
          "condition": "is_death_discharge_evaluation"
        },
        "assertions": [
          {
            "kind": "raw_equals",
            "field": "3.19",
            "value": "00000000000000000000"
          }
        ],
        "error_code": "E010"
      },
      {
        "id": "R-3.22-DEATH",
        "when": {
          "condition": "is_death_discharge_evaluation"
        },
        "assertions": [
          {
            "kind": "raw_equals",
            "field": "3.22",
            "value": "99"
          }
        ],
        "error_code": "E010"
      },
      {
        "id": "R-3.22-NORMAL",
        "when": {
          "not": {
            "condition": "is_death_discharge_evaluation"
          }
        },
        "assertions": [
          {
            "kind": "numeric_range",
            "field": "3.22",
            "min": 0,
            "max": 98
          }
        ],
        "error_code": "E010"
      },
      {
        "id": "R-3.24-DEATH-TO-00",
        "when": {
          "condition": "is_death_discharge_evaluation"
        },
        "assertions": [
          {
            "kind": "raw_equals",
            "field": "3.24",
            "value": "00"
          }
        ],
        "error_code": "E010"
      },
      {
        "id": "R-3.24-00-TO-DEATH",
        "when": {
          "eq": {
            "ref": {
              "field": "3.24"
            },
            "value": "00"
          }
        },
        "assertions": [
          {
            "kind": "condition_true",
            "condition": "is_death_discharge_evaluation"
          }
        ],
        "error_code": "E010"
      },
      {
        "id": "R-3.24-NORMAL-RANGE",
        "when": {
          "not": {
            "condition": "is_death_discharge_evaluation"
          }
        },
        "assertions": [
          {
            "kind": "numeric_range",
            "field": "3.24",
            "min": 1,
            "max": 90
          }
        ],
        "error_code": "E010"
      },
      {
        "id": "R-3.25-ADMISSION",
        "when": {
          "condition": "is_admission_evaluation"
        },
        "assertions": [
          {
            "kind": "allowed_values",
            "field": "3.25",
            "values": [
              "1",
              "2",
              "3"
            ]
          }
        ],
        "error_code": "E010"
      },
      {
        "id": "R-3.25-DISCHARGE",
        "when": {
          "condition": "is_discharge_evaluation"
        },
        "assertions": [
          {
            "kind": "raw_equals",
            "field": "3.25",
            "value": "0"
          }
        ],
        "error_code": "E010"
      },
      {
        "id": "R-3.26-ADMISSION",
        "when": {
          "condition": "is_admission_evaluation"
        },
        "assertions": [
          {
            "kind": "raw_equals",
            "field": "3.26",
            "value": "0"
          }
        ],
        "error_code": "E010"
      },
      {
        "id": "R-3.26-DISCHARGE",
        "when": {
          "condition": "is_discharge_evaluation"
        },
        "assertions": [
          {
            "kind": "allowed_values",
            "field": "3.26",
            "values": [
              "1",
              "2",
              "3",
              "4"
            ]
          }
        ],
        "error_code": "E010"
      },
      {
        "id": "R-3.26-DEATH-TO-4",
        "when": {
          "condition": "is_death_discharge_evaluation"
        },
        "assertions": [
          {
            "kind": "raw_equals",
            "field": "3.26",
            "value": "4"
          }
        ],
        "error_code": "E010"
      },
      {
        "id": "R-3.26-4-TO-DEATH",
        "when": {
          "eq": {
            "ref": {
              "field": "3.26"
            },
            "value": "4"
          }
        },
        "assertions": [
          {
            "kind": "condition_true",
            "condition": "is_death_discharge_evaluation"
          }
        ],
        "error_code": "E010"
      },
      {
        "id": "R-3.27-ADMISSION",
        "when": {
          "condition": "is_admission_evaluation"
        },
        "assertions": [
          {
            "kind": "raw_equals",
            "field": "3.27",
            "value": "00"
          }
        ],
        "error_code": "E010"
      },
      {
        "id": "R-3.27-DISCHARGE",
        "when": {
          "condition": "is_discharge_evaluation"
        },
        "assertions": [
          {
            "kind": "numeric_range",
            "field": "3.27",
            "min": 0,
            "max": 99
          }
        ],
        "error_code": "E010"
      },
      {
        "id": "R-3.28-ADMISSION",
        "when": {
          "condition": "is_admission_evaluation"
        },
        "assertions": [
          {
            "kind": "raw_equals",
            "field": "3.28",
            "value": "00"
          }
        ],
        "error_code": "E010"
      },
      {
        "id": "R-3.28-DISCHARGE",
        "when": {
          "condition": "is_discharge_evaluation"
        },
        "assertions": [
          {
            "kind": "numeric_range",
            "field": "3.28",
            "min": 0,
            "max": 99
          }
        ],
        "error_code": "E010"
      },
      {
        "id": "R-3.29-ADMISSION",
        "when": {
          "condition": "is_admission_evaluation"
        },
        "assertions": [
          {
            "kind": "raw_equals",
            "field": "3.29",
            "value": "00"
          }
        ],
        "error_code": "E010"
      },
      {
        "id": "R-3.29-DISCHARGE",
        "when": {
          "condition": "is_discharge_evaluation"
        },
        "assertions": [
          {
            "kind": "numeric_range",
            "field": "3.29",
            "min": 0,
            "max": 99
          }
        ],
        "error_code": "E010"
      },
      {
        "id": "R-3.30-ADMISSION",
        "when": {
          "condition": "is_admission_evaluation"
        },
        "assertions": [
          {
            "kind": "raw_equals",
            "field": "3.30",
            "value": "00"
          }
        ],
        "error_code": "E010"
      },
      {
        "id": "R-3.30-DISCHARGE",
        "when": {
          "condition": "is_discharge_evaluation"
        },
        "assertions": [
          {
            "kind": "numeric_range",
            "field": "3.30",
            "min": 0,
            "max": 99
          }
        ],
        "error_code": "E010"
      },
      {
        "id": "R-3.31-ADMISSION",
        "when": {
          "condition": "is_admission_evaluation"
        },
        "assertions": [
          {
            "kind": "raw_equals",
            "field": "3.31",
            "value": "00"
          }
        ],
        "error_code": "E010"
      },
      {
        "id": "R-3.31-DISCHARGE",
        "when": {
          "condition": "is_discharge_evaluation"
        },
        "assertions": [
          {
            "kind": "numeric_range",
            "field": "3.31",
            "min": 0,
            "max": 99
          }
        ],
        "error_code": "E010"
      },
      {
        "id": "R-3.32-ADMISSION",
        "when": {
          "condition": "is_admission_evaluation"
        },
        "assertions": [
          {
            "kind": "raw_equals",
            "field": "3.32",
            "value": "0"
          }
        ],
        "error_code": "E010"
      },
      {
        "id": "R-3.32-DISCHARGE",
        "when": {
          "condition": "is_discharge_evaluation"
        },
        "assertions": [
          {
            "kind": "allowed_values",
            "field": "3.32",
            "values": [
              "1",
              "2",
              "3",
              "4"
            ]
          }
        ],
        "error_code": "E010"
      },
      {
        "id": "R-3.33-DEATH",
        "when": {
          "condition": "is_death_discharge_evaluation"
        },
        "assertions": [
          {
            "kind": "raw_equals",
            "field": "3.33",
            "value": "9"
          }
        ],
        "error_code": "E010"
      },
      {
        "id": "R-3.33-NORMAL",
        "when": {
          "not": {
            "condition": "is_death_discharge_evaluation"
          }
        },
        "assertions": [
          {
            "kind": "allowed_values",
            "field": "3.33",
            "values": [
              "1",
              "2",
              "3"
            ]
          }
        ],
        "error_code": "E010"
      },
      {
        "id": "R-3.34-AGE",
        "when": {
          "const": true
        },
        "assertions": [
          {
            "kind": "derived_compare",
            "derived_value": "age_at_admission_years",
            "operator": ">=",
            "value": 19
          }
        ],
        "error_code": "E010"
      },
      {
        "id": "R-3.34-DEATH",
        "when": {
          "condition": "is_death_discharge_evaluation"
        },
        "assertions": [
          {
            "kind": "raw_equals",
            "field": "3.34",
            "value": "999"
          }
        ],
        "error_code": "E010"
      },
      {
        "id": "R-3.34-NORMAL",
        "when": {
          "not": {
            "condition": "is_death_discharge_evaluation"
          }
        },
        "assertions": [
          {
            "kind": "numeric_range",
            "field": "3.34",
            "min": 0,
            "max": 100
          }
        ],
        "error_code": "E010"
      },
      {
        "id": "R-3.34-999-TO-DEATH",
        "when": {
          "eq": {
            "ref": {
              "field": "3.34"
            },
            "value": "999"
          }
        },
        "assertions": [
          {
            "kind": "condition_true",
            "condition": "is_death_discharge_evaluation"
          }
        ],
        "error_code": "E010"
      }
    ],
    "reference_data_dependencies": [
      "health-authority-codes",
      "icpc-codes",
      "sts11-structure-codes"
    ],
    "freeze_notes": [
      "Official XLSX source IDs 3.48 and 3.49 are normalized to canonical IDs 3.35 and 3.36; source IDs remain preserved in regulatory metadata."
    ]
  },
  "field_policies": [
    {"id":"3.1","physical":{"start":1,"length":20,"type":"alphanumeric","source_type":"AN","alignment":{"kind":"left","pad":" "}},"missing_value":{"kind":"spaces","raw":"                    "},"presence":{"base":"required","required_if":[],"forbidden_if":[]},"constraints":[{"kind":"logical_length","equals":8},{"kind":"digits_only"},{"kind":"substring_not_equals","target":"logical_value","start":3,"length":6,"value":"000000"}]},
    {"id":"3.2","physical":{"start":21,"length":20,"type":"alphanumeric","source_type":"AN","alignment":{"kind":"left","pad":" "}},"missing_value":{"kind":"spaces","raw":"                    "},"presence":{"base":"required","required_if":[],"forbidden_if":[]},"constraints":[{"kind":"structure_only"}]},
    {"id":"3.3","physical":{"start":41,"length":1,"type":"alphanumeric","source_type":"AN","alignment":{"kind":"left","pad":" "}},"missing_value":{"kind":"spaces","raw":" "},"presence":{"base":"required","required_if":[],"forbidden_if":[]},"constraints":[{"kind":"allowed_values","values":["1","2"]}]},
    {"id":"3.4","physical":{"start":42,"length":8,"type":"date","source_type":"D","alignment":{"kind":"exact"}},"missing_value":{"kind":"spaces","raw":"        "},"presence":{"base":"required","required_if":[],"forbidden_if":[]},"constraints":[{"kind":"date","format":"DDMMYYYY","calendar_valid":true}]},
    {"id":"3.5","physical":{"start":50,"length":3,"type":"alphanumeric","source_type":"AN","alignment":{"kind":"left","pad":" "}},"missing_value":{"kind":"spaces","raw":"   "},"presence":{"base":"required","required_if":[],"forbidden_if":[]},"constraints":[{"kind":"reference_data","dataset":"icpc-codes"}]},
    {"id":"3.6","physical":{"start":53,"length":3,"type":"alphanumeric","source_type":"AN","alignment":{"kind":"left","pad":" "}},"missing_value":{"kind":"spaces","raw":"   "},"presence":{"base":"optional","required_if":[],"forbidden_if":[]},"constraints":[{"kind":"reference_data_if_present","dataset":"icpc-codes"}]},
    {"id":"3.7","physical":{"start":56,"length":3,"type":"alphanumeric","source_type":"AN","alignment":{"kind":"left","pad":" "}},"missing_value":{"kind":"spaces","raw":"   "},"presence":{"base":"optional","required_if":[],"forbidden_if":[]},"constraints":[{"kind":"reference_data_if_present","dataset":"icpc-codes"}]},
    {"id":"3.8","physical":{"start":59,"length":2,"type":"numeric","source_type":"N","alignment":{"kind":"right","pad":"0"}},"missing_value":{"kind":"never_missing"},"presence":{"base":"required","required_if":[],"forbidden_if":[]},"constraints":[]},
    {"id":"3.9","physical":{"start":61,"length":1,"type":"numeric","source_type":"N","alignment":{"kind":"right","pad":"0"}},"missing_value":{"kind":"never_missing"},"presence":{"base":"required","required_if":[],"forbidden_if":[]},"constraints":[]},
    {"id":"3.10","physical":{"start":62,"length":1,"type":"numeric","source_type":"N","alignment":{"kind":"right","pad":"0"}},"missing_value":{"kind":"never_missing"},"presence":{"base":"required","required_if":[],"forbidden_if":[]},"constraints":[]},
    {"id":"3.11","physical":{"start":63,"length":2,"type":"numeric","source_type":"N","alignment":{"kind":"right","pad":"0"}},"missing_value":{"kind":"never_missing"},"presence":{"base":"required","required_if":[],"forbidden_if":[]},"constraints":[]},
    {"id":"3.12","physical":{"start":65,"length":2,"type":"numeric","source_type":"N","alignment":{"kind":"right","pad":"0"}},"missing_value":{"kind":"never_missing"},"presence":{"base":"required","required_if":[],"forbidden_if":[]},"constraints":[]},
    {"id":"3.13","physical":{"start":67,"length":2,"type":"numeric","source_type":"N","alignment":{"kind":"right","pad":"0"}},"missing_value":{"kind":"never_missing"},"presence":{"base":"required","required_if":[],"forbidden_if":[]},"constraints":[]},
    {"id":"3.14","physical":{"start":69,"length":2,"type":"numeric","source_type":"N","alignment":{"kind":"right","pad":"0"}},"missing_value":{"kind":"never_missing"},"presence":{"base":"required","required_if":[],"forbidden_if":[]},"constraints":[]},
    {"id":"3.15","physical":{"start":71,"length":2,"type":"numeric","source_type":"N","alignment":{"kind":"right","pad":"0"}},"missing_value":{"kind":"never_missing"},"presence":{"base":"required","required_if":[],"forbidden_if":[]},"constraints":[]},
    {"id":"3.16","physical":{"start":73,"length":2,"type":"numeric","source_type":"N","alignment":{"kind":"right","pad":"0"}},"missing_value":{"kind":"never_missing"},"presence":{"base":"required","required_if":[],"forbidden_if":[]},"constraints":[]},
    {"id":"3.17","physical":{"start":75,"length":1,"type":"numeric","source_type":"N","alignment":{"kind":"right","pad":"0"}},"missing_value":{"kind":"never_missing"},"presence":{"base":"required","required_if":[],"forbidden_if":[]},"constraints":[{"kind":"allowed_values","values":["0","1"]}]},
    {"id":"3.18","physical":{"start":76,"length":2,"type":"numeric","source_type":"N","alignment":{"kind":"right","pad":"0"}},"missing_value":{"kind":"never_missing"},"presence":{"base":"required","required_if":[],"forbidden_if":[]},"constraints":[]},
    {"id":"3.19","physical":{"start":78,"length":20,"type":"numeric","source_type":"N","alignment":{"kind":"right","pad":"0"}},"missing_value":{"kind":"never_missing"},"presence":{"base":"required","required_if":[],"forbidden_if":[]},"constraints":[{"kind":"positional_bitset","length":20,"allowed_chars":["0","1"],"forced_positions":{"17":"0","18":"0","19":"0","20":"0"}}]},
    {"id":"3.20","physical":{"start":98,"length":2,"type":"numeric","source_type":"N","alignment":{"kind":"right","pad":"0"}},"missing_value":{"kind":"never_missing"},"presence":{"base":"required","required_if":[],"forbidden_if":[]},"constraints":[]},
    {"id":"3.21","physical":{"start":100,"length":2,"type":"numeric","source_type":"N","alignment":{"kind":"right","pad":"0"}},"missing_value":{"kind":"never_missing"},"presence":{"base":"required","required_if":[],"forbidden_if":[]},"constraints":[]},
    {"id":"3.22","physical":{"start":102,"length":2,"type":"numeric","source_type":"N","alignment":{"kind":"right","pad":"0"}},"missing_value":{"kind":"never_missing"},"presence":{"base":"required","required_if":[],"forbidden_if":[]},"constraints":[],"semantic_null_values":[{"value":"99","meaning":"Deceduto","valid_if":"is_death_discharge_evaluation"}]},
    {"id":"3.23","physical":{"start":104,"length":1,"type":"numeric","source_type":"N","alignment":{"kind":"right","pad":"0"}},"missing_value":{"kind":"exact","raw":"0"},"presence":{"base":"required","required_if":[],"forbidden_if":[]},"constraints":[]},
    {"id":"3.24","physical":{"start":105,"length":2,"type":"numeric","source_type":"N","alignment":{"kind":"right","pad":"0"}},"missing_value":{"kind":"never_missing"},"presence":{"base":"required","required_if":[],"forbidden_if":[]},"constraints":[]},
    {"id":"3.25","physical":{"start":107,"length":1,"type":"numeric","source_type":"N","alignment":{"kind":"right","pad":"0"}},"missing_value":{"kind":"never_missing"},"presence":{"base":"required","required_if":[],"forbidden_if":[]},"constraints":[],"semantic_null_values":[{"value":"0","meaning":"Non applicabile alla dimissione","valid_if":"is_discharge_evaluation"}]},
    {"id":"3.26","physical":{"start":108,"length":1,"type":"numeric","source_type":"N","alignment":{"kind":"right","pad":"0"}},"missing_value":{"kind":"never_missing"},"presence":{"base":"required","required_if":[],"forbidden_if":[]},"constraints":[]},
    {"id":"3.27","physical":{"start":109,"length":2,"type":"numeric","source_type":"N","alignment":{"kind":"right","pad":"0"}},"missing_value":{"kind":"never_missing"},"presence":{"base":"required","required_if":[],"forbidden_if":[]},"constraints":[]},
    {"id":"3.28","physical":{"start":111,"length":2,"type":"numeric","source_type":"N","alignment":{"kind":"right","pad":"0"}},"missing_value":{"kind":"never_missing"},"presence":{"base":"required","required_if":[],"forbidden_if":[]},"constraints":[]},
    {"id":"3.29","physical":{"start":113,"length":2,"type":"numeric","source_type":"N","alignment":{"kind":"right","pad":"0"}},"missing_value":{"kind":"never_missing"},"presence":{"base":"required","required_if":[],"forbidden_if":[]},"constraints":[]},
    {"id":"3.30","physical":{"start":115,"length":2,"type":"numeric","source_type":"N","alignment":{"kind":"right","pad":"0"}},"missing_value":{"kind":"never_missing"},"presence":{"base":"required","required_if":[],"forbidden_if":[]},"constraints":[]},
    {"id":"3.31","physical":{"start":117,"length":2,"type":"numeric","source_type":"N","alignment":{"kind":"right","pad":"0"}},"missing_value":{"kind":"never_missing"},"presence":{"base":"required","required_if":[],"forbidden_if":[]},"constraints":[]},
    {"id":"3.32","physical":{"start":119,"length":1,"type":"numeric","source_type":"N","alignment":{"kind":"right","pad":"0"}},"missing_value":{"kind":"never_missing"},"presence":{"base":"required","required_if":[],"forbidden_if":[]},"constraints":[]},
    {"id":"3.33","physical":{"start":120,"length":1,"type":"alphanumeric","source_type":"AN","alignment":{"kind":"left","pad":" "}},"missing_value":{"kind":"spaces","raw":" "},"presence":{"base":"required","required_if":[],"forbidden_if":[]},"constraints":[]},
    {"id":"3.34","physical":{"start":121,"length":3,"type":"numeric","source_type":"N","alignment":{"kind":"right","pad":"0"}},"missing_value":{"kind":"never_missing"},"presence":{"base":"required","required_if":[],"forbidden_if":[]},"constraints":[]},
    {"id":"3.35","source_id":"3.35","physical":{"start":124,"length":6,"type":"alphanumeric","source_type":"AN","alignment":{"kind":"left","pad":" "}},"missing_value":{"kind":"spaces","raw":"      "},"presence":{"base":"required","required_if":[],"forbidden_if":[]},"constraints":[{"kind":"logical_length","equals":6},{"kind":"digits_only"},{"kind":"composite_segments","segments":[{"name":"region_code","start":1,"length":3,"equals":"050"},{"name":"health_authority_code","start":4,"length":3,"reference_data":"health-authority-codes"}]}]},
    {"id":"3.36","source_id":"3.36","physical":{"start":130,"length":6,"type":"alphanumeric","source_type":"AN","alignment":{"kind":"left","pad":" "}},"missing_value":{"kind":"spaces","raw":"      "},"presence":{"base":"required","required_if":[],"forbidden_if":[]},"constraints":[{"kind":"reference_data","dataset":"sts11-structure-codes"}]}
  ]
}
```

## Trace 4.1

Fonte: `K/SCHEMA_INPUTS/SIOC_2026-04-20_FROZEN/trace_4_1.json`; note e metadati sorgente: [4.1-fields.json](4.1-fields.json), `/fields/i/regulatory_metadata`.
Policy completa: appendice seguente, `/field_policies/i`; contesto: `/trace_context`.

| Campo (tabella con note) | i | Byte (larghezza) | Alias → tipo fisico | Sorgente | Presence configurata | Missing raw | Alignment/padding raw |
|---|---:|---|---|---|---|---|---|
| [4.1](4.1-fields.json) | 0 | 1–6 (6) | `"N"` → numeric | `"OBB"` | required; R=0, F=0 | exact: `"000000"` (6 byte) | right; pad=`"0"` (0x30) |
| [4.2](4.1-fields.json) | 1 | 7–21 (15) | `"AN"` → alphanumeric | `"OBB"` | required; R=0, F=0 | spaces: 15 × 0x20 | left; pad=`" "` (0x20) |
| [4.3](4.1-fields.json) | 2 | 22–22 (1) | `"N"` → numeric | `"OBB"` | required; R=0, F=0 | exact: `"0"` (1 byte) | right; pad=`"0"` (0x30) |
| [4.4](4.1-fields.json) | 3 | 23–23 (1) | `"N"` → numeric | `"OBB"` | required; R=0, F=0 | exact: `"0"` (1 byte) | right; pad=`"0"` (0x30) |
| [4.5](4.1-fields.json) | 4 | 24–24 (1) | `null` → numeric | `"OBB"` | required; R=0, F=0 | exact: `"0"` (1 byte) | right; pad=`"0"` (0x30) |
| [4.6](4.1-fields.json) | 5 | 25–144 (120) | `"A"` → alphanumeric | `"OBB"` | required; R=0, F=0 | spaces: 120 × 0x20 | left; pad=`" "` (0x20) |
| [4.7](4.1-fields.json) | 6 | 145–147 (3) | `"AN"` → alphanumeric | `"OBB"` | required; R=0, F=0 | spaces: 3 × 0x20 | left; pad=`" "` (0x20) |
| [4.8](4.1-fields.json) | 7 | 148–150 (3) | `"AN"` → alphanumeric | `"OBB"` | required; R=0, F=0 | spaces: 3 × 0x20 | left; pad=`" "` (0x20) |
| [4.9](4.1-fields.json) | 8 | 151–156 (6) | `"N"` → numeric | `"OBB"` | required; R=0, F=0 | exact: `"000000"` (6 byte) | right; pad=`"0"` (0x30) |
| [4.10](4.1-fields.json) | 9 | 157–186 (30) | `"A"` → alphanumeric | `"OBB"` | required; R=0, F=0 | spaces: 30 × 0x20 | left; pad=`" "` (0x20) |
| [4.11](4.1-fields.json) | 10 | 187–192 (6) | `"N"` → numeric | `"OBB"` | required; R=0, F=0 | exact: `"000000"` (6 byte) | right; pad=`"0"` (0x30) |
| [4.12](4.1-fields.json) | 11 | 193–312 (120) | `"AN"` → alphanumeric | `"OBB"` | required; R=0, F=0 | spaces: 120 × 0x20 | left; pad=`" "` (0x20) |
| [4.13](4.1-fields.json) | 12 | 313–314 (2) | `"N"` → numeric | `"OBB"` | required; R=0, F=0 | exact: `"00"` (2 byte) | right; pad=`"0"` (0x30) |
| [4.14](4.1-fields.json) | 13 | 315–320 (6) | `"N"` → numeric | `"OBB"` | required; R=0, F=0 | exact: `"000000"` (6 byte) | right; pad=`"0"` (0x30) |
| [4.15](4.1-fields.json) | 14 | 321–350 (30) | `"AN"` → alphanumeric | `"OBB"` | required; R=0, F=0 | spaces: 30 × 0x20 | left; pad=`" "` (0x20) |
| [4.16](4.1-fields.json) | 15 | 351–361 (11) | `"N"` → numeric | `"OBB"` | required; R=0, F=0 | exact: `"00000000000"` (11 byte) | right; pad=`"0"` (0x30) |
| [4.17](4.1-fields.json) | 16 | 362–372 (11) | `"N"` → numeric | `"OBB"` | required; R=0, F=0 | exact: `"00000000000"` (11 byte) | right; pad=`"0"` (0x30) |
| [4.18](4.1-fields.json) | 17 | 373–380 (8) | `"D"` → date | `"OBB"` | required; R=0, F=0 | spaces: 8 × 0x20 | exact |
| [4.19](4.1-fields.json) | 18 | 381–388 (8) | `"D"` → date | `"OBB"` | required; R=0, F=0 | spaces: 8 × 0x20 | exact |
| [4.19.1](4.1-fields.json) | 19 | 389–394 (6) | `"AN"` → alphanumeric | `"OBB"` | required; R=0, F=0 | spaces: 6 × 0x20 | left; pad=`" "` (0x20) |

### Appendice raw trace 4.1

Ogni oggetto di `field_policies` è la proiezione **senza modifica di valori**
delle chiavi di policy del campo ricevuto. Le note originali restano
nella tabella collegata; le policy non sono dedotte dalle note.

<!-- T-P0-03:trace:4.1 -->
```json
{
  "trace_context": {
    "canonical_schema_version": "1.0.0",
    "schema_status": "FROZEN",
    "flow_id": "SIOC",
    "regulatory_version": "2025-12-16",
    "trace_id": "4.1",
    "trace_name": "Tracciato regionale UDO",
    "record_length": 394,
    "value_model": {
      "physical_positions": "one_based",
      "raw_value": "exact fixed-width bytes",
      "logical_value": "raw value after schema-declared padding removal",
      "semantic_comparisons": "logical_value unless assertion explicitly targets raw_value"
    },
    "record_key": {
      "fields": [
        "4.1",
        "4.7",
        "4.8",
        "4.19.1"
      ],
      "uniqueness_scope": "trace",
      "error_code": "E_DUPLICATE_RECORD_KEY"
    },
    "joins": [],
    "named_conditions": {},
    "derived_values": {},
    "repeated_groups": [],
    "documented_non_executable_rules": [],
    "rules": [
      {
        "id": "R-4.18-LE-4.19",
        "when": {
          "const": true
        },
        "assertions": [
          {
            "kind": "field_compare",
            "left": {
              "field": "4.18"
            },
            "operator": "<=",
            "right": {
              "field": "4.19"
            },
            "value_type": "date"
          }
        ],
        "error_code": "E_TEMPORAL_ORDER"
      }
    ],
    "reference_data_dependencies": [
      "health-authority-codes",
      "istat-municipality-codes",
      "italian-region-codes",
      "regional-entity-manager-codes-dgr-2961-2012",
      "regional-udo-codes",
      "sts11-structure-codes"
    ],
    "freeze_notes": []
  },
  "field_policies": [
    {"id":"4.1","physical":{"start":1,"length":6,"type":"numeric","source_type":"N","alignment":{"kind":"right","pad":"0"}},"missing_value":{"kind":"exact","raw":"000000"},"presence":{"base":"required","required_if":[],"forbidden_if":[]},"constraints":[{"kind":"reference_data","dataset":"regional-udo-codes"}]},
    {"id":"4.2","physical":{"start":7,"length":15,"type":"alphanumeric","source_type":"AN","alignment":{"kind":"left","pad":" "}},"missing_value":{"kind":"spaces","raw":"               "},"presence":{"base":"required","required_if":[],"forbidden_if":[]},"constraints":[{"kind":"structure_only"}]},
    {"id":"4.3","physical":{"start":22,"length":1,"type":"numeric","source_type":"N","alignment":{"kind":"right","pad":"0"}},"missing_value":{"kind":"exact","raw":"0"},"presence":{"base":"required","required_if":[],"forbidden_if":[]},"constraints":[{"kind":"allowed_values","values":["1"]}]},
    {"id":"4.4","physical":{"start":23,"length":1,"type":"numeric","source_type":"N","alignment":{"kind":"right","pad":"0"}},"missing_value":{"kind":"exact","raw":"0"},"presence":{"base":"required","required_if":[],"forbidden_if":[]},"constraints":[{"kind":"allowed_values","values":["1","2","3","4"]}]},
    {"id":"4.5","physical":{"start":24,"length":1,"type":"numeric","source_type":null,"alignment":{"kind":"right","pad":"0"}},"missing_value":{"kind":"exact","raw":"0","meaning":"Valore non immesso"},"presence":{"base":"required","required_if":[],"forbidden_if":[]},"constraints":[{"kind":"allowed_values","values":["1","2"]}]},
    {"id":"4.6","physical":{"start":25,"length":120,"type":"alphanumeric","source_type":"A","alignment":{"kind":"left","pad":" "}},"missing_value":{"kind":"spaces","raw":"                                                                                                                        "},"presence":{"base":"required","required_if":[],"forbidden_if":[]},"constraints":[{"kind":"non_empty"}]},
    {"id":"4.7","physical":{"start":145,"length":3,"type":"alphanumeric","source_type":"AN","alignment":{"kind":"left","pad":" "}},"missing_value":{"kind":"spaces","raw":"   "},"presence":{"base":"required","required_if":[],"forbidden_if":[]},"constraints":[{"kind":"reference_data","dataset":"italian-region-codes"}]},
    {"id":"4.8","physical":{"start":148,"length":3,"type":"alphanumeric","source_type":"AN","alignment":{"kind":"left","pad":" "}},"missing_value":{"kind":"spaces","raw":"   "},"presence":{"base":"required","required_if":[],"forbidden_if":[]},"constraints":[{"kind":"reference_data","dataset":"health-authority-codes"}]},
    {"id":"4.9","physical":{"start":151,"length":6,"type":"numeric","source_type":"N","alignment":{"kind":"right","pad":"0"}},"missing_value":{"kind":"exact","raw":"000000"},"presence":{"base":"required","required_if":[],"forbidden_if":[]},"constraints":[{"kind":"reference_data","dataset":"istat-municipality-codes"}]},
    {"id":"4.10","physical":{"start":157,"length":30,"type":"alphanumeric","source_type":"A","alignment":{"kind":"left","pad":" "}},"missing_value":{"kind":"spaces","raw":"                              "},"presence":{"base":"required","required_if":[],"forbidden_if":[]},"constraints":[{"kind":"non_empty"}]},
    {"id":"4.11","physical":{"start":187,"length":6,"type":"numeric","source_type":"N","alignment":{"kind":"right","pad":"0"}},"missing_value":{"kind":"exact","raw":"000000"},"presence":{"base":"required","required_if":[],"forbidden_if":[]},"constraints":[{"kind":"reference_data","dataset":"regional-entity-manager-codes-dgr-2961-2012"}]},
    {"id":"4.12","physical":{"start":193,"length":120,"type":"alphanumeric","source_type":"AN","alignment":{"kind":"left","pad":" "}},"missing_value":{"kind":"spaces","raw":"                                                                                                                        "},"presence":{"base":"required","required_if":[],"forbidden_if":[]},"constraints":[{"kind":"non_empty"}]},
    {"id":"4.13","physical":{"start":313,"length":2,"type":"numeric","source_type":"N","alignment":{"kind":"right","pad":"0"}},"missing_value":{"kind":"exact","raw":"00"},"presence":{"base":"required","required_if":[],"forbidden_if":[]},"constraints":[{"kind":"allowed_values","values":["01","02","03","04","05","06","07","08","09","10"]}]},
    {"id":"4.14","physical":{"start":315,"length":6,"type":"numeric","source_type":"N","alignment":{"kind":"right","pad":"0"}},"missing_value":{"kind":"exact","raw":"000000"},"presence":{"base":"required","required_if":[],"forbidden_if":[]},"constraints":[{"kind":"reference_data","dataset":"istat-municipality-codes"}]},
    {"id":"4.15","physical":{"start":321,"length":30,"type":"alphanumeric","source_type":"AN","alignment":{"kind":"left","pad":" "}},"missing_value":{"kind":"spaces","raw":"                              "},"presence":{"base":"required","required_if":[],"forbidden_if":[]},"constraints":[{"kind":"non_empty"}]},
    {"id":"4.16","physical":{"start":351,"length":11,"type":"numeric","source_type":"N","alignment":{"kind":"right","pad":"0"}},"missing_value":{"kind":"exact","raw":"00000000000"},"presence":{"base":"required","required_if":[],"forbidden_if":[]},"constraints":[{"kind":"digits_only"},{"kind":"logical_length","equals":11}]},
    {"id":"4.17","physical":{"start":362,"length":11,"type":"numeric","source_type":"N","alignment":{"kind":"right","pad":"0"}},"missing_value":{"kind":"exact","raw":"00000000000"},"presence":{"base":"required","required_if":[],"forbidden_if":[]},"constraints":[{"kind":"digits_only"},{"kind":"logical_length","equals":11}]},
    {"id":"4.18","physical":{"start":373,"length":8,"type":"date","source_type":"D","alignment":{"kind":"exact"}},"missing_value":{"kind":"spaces","raw":"        "},"presence":{"base":"required","required_if":[],"forbidden_if":[]},"constraints":[{"kind":"date","format":"DDMMYYYY","calendar_valid":true}]},
    {"id":"4.19","physical":{"start":381,"length":8,"type":"date","source_type":"D","alignment":{"kind":"exact"}},"missing_value":{"kind":"spaces","raw":"        "},"presence":{"base":"required","required_if":[],"forbidden_if":[]},"constraints":[{"kind":"date","format":"DDMMYYYY","calendar_valid":true}]},
    {"id":"4.19.1","physical":{"start":389,"length":6,"type":"alphanumeric","source_type":"AN","alignment":{"kind":"left","pad":" "}},"missing_value":{"kind":"spaces","raw":"      "},"presence":{"base":"required","required_if":[],"forbidden_if":[]},"constraints":[{"kind":"reference_data","dataset":"sts11-structure-codes"}]}
  ]
}
```

## Trace 4.2

Fonte: `K/SCHEMA_INPUTS/SIOC_2026-04-20_FROZEN/trace_4_2.json`; note e metadati sorgente: [4.2-fields.json](4.2-fields.json), `/fields/i/regulatory_metadata`.
Policy completa: appendice seguente, `/field_policies/i`; contesto: `/trace_context`.

| Campo (tabella con note) | i | Byte (larghezza) | Alias → tipo fisico | Sorgente | Presence configurata | Missing raw | Alignment/padding raw |
|---|---:|---|---|---|---|---|---|
| [4.20](4.2-fields.json) | 0 | 1–6 (6) | `"N"` → numeric | `"OBB"` | required; R=0, F=0 | exact: `"000000"` (6 byte) | right; pad=`"0"` (0x30) |
| [4.21](4.2-fields.json) | 1 | 7–21 (15) | `"AN"` → alphanumeric | `"OBB"` | required; R=0, F=0 | spaces: 15 × 0x20 | left; pad=`" "` (0x20) |
| [4.22](4.2-fields.json) | 2 | 22–31 (10) | `"AN"` → alphanumeric | `"OBB"` | required; R=0, F=0 | spaces: 10 × 0x20 | left; pad=`" "` (0x20) |
| [4.23](4.2-fields.json) | 3 | 32–32 (1) | `"N"` → numeric | `"OBB"` | required; R=0, F=0 | exact: `"0"` (1 byte) | right; pad=`"0"` (0x30) |
| [4.24](4.2-fields.json) | 4 | 33–33 (1) | `"N"` → numeric | `"OBB"` | required; R=0, F=0 | exact: `"0"` (1 byte) | right; pad=`"0"` (0x30) |
| [4.25](4.2-fields.json) | 5 | 34–37 (4) | `"N"` → numeric | `"OBB"` | required; R=0, F=0 | exact: `"0000"` (4 byte) | right; pad=`"0"` (0x30) |
| [4.26](4.2-fields.json) | 6 | 38–45 (8) | `"D"` → date | `"OBB"` | required; R=0, F=0 | spaces: 8 × 0x20 | exact |
| [4.27](4.2-fields.json) | 7 | 46–48 (3) | `"N"` → numeric | `"OBB"` | required; R=0, F=0 | never_missing | right; pad=`"0"` (0x30) |
| [4.28](4.2-fields.json) | 8 | 49–49 (1) | `"N"` → numeric | `"OBB"` | required; R=0, F=0 | exact: `"0"` (1 byte) | right; pad=`"0"` (0x30) |
| [4.29](4.2-fields.json) | 9 | 50–57 (8) | `"D"` → date | `"FAC"` | optional; R=0, F=0 | spaces: 8 × 0x20 | exact |
| [4.30](4.2-fields.json) | 10 | 58–67 (10) | `"AN"` → alphanumeric | `"FAC"` | optional; R=0, F=0 | spaces: 10 × 0x20 | left; pad=`" "` (0x20) |

### Appendice raw trace 4.2

Ogni oggetto di `field_policies` è la proiezione **senza modifica di valori**
delle chiavi di policy del campo ricevuto. Le note originali restano
nella tabella collegata; le policy non sono dedotte dalle note.

<!-- T-P0-03:trace:4.2 -->
```json
{
  "trace_context": {
    "canonical_schema_version": "1.0.0",
    "schema_status": "FROZEN",
    "flow_id": "SIOC",
    "regulatory_version": "2025-12-16",
    "trace_id": "4.2",
    "trace_name": "Tracciato regionale Atti LR 22/2002",
    "record_length": 67,
    "value_model": {
      "physical_positions": "one_based",
      "raw_value": "exact fixed-width bytes",
      "logical_value": "raw value after schema-declared padding removal",
      "semantic_comparisons": "logical_value unless assertion explicitly targets raw_value"
    },
    "record_key": {
      "fields": [
        "4.20",
        "4.22"
      ],
      "uniqueness_scope": "trace",
      "error_code": "E_DUPLICATE_RECORD_KEY"
    },
    "joins": [
      {
        "id": "trace42_to_trace41",
        "target_trace": "4.1",
        "source_fields": [
          "4.20",
          "4.21"
        ],
        "target_fields": [
          "4.1",
          "4.2"
        ],
        "required": true,
        "cardinality": "at_least_one",
        "error_code": "E_MISSING_PARENT_RECORD"
      }
    ],
    "named_conditions": {},
    "derived_values": {},
    "repeated_groups": [],
    "documented_non_executable_rules": [
      {
        "id": "POLICY-4.28-PREVIOUS-ACT",
        "field": "4.28",
        "policy": "not_enforced_in_v1",
        "reason": "Values 2-5 refer to a previous act, but the trace does not expose a predecessor-act identifier. No cross-record relation is invented."
      }
    ],
    "rules": [
      {
        "id": "R-4.30-REQUIRES-4.29",
        "when": {
          "is_present": {
            "ref": {
              "field": "4.30"
            }
          }
        },
        "assertions": [
          {
            "kind": "field_present",
            "field": "4.29"
          }
        ],
        "error_code": "E003",
        "note": "A revoking act necessarily terminates validity; however a cessation date may exist without a revoking act."
      },
      {
        "id": "R-4.30-REFERENCES-4.22",
        "when": {
          "is_present": {
            "ref": {
              "field": "4.30"
            }
          }
        },
        "assertions": [
          {
            "kind": "record_exists",
            "trace": "4.2",
            "match": [
              {
                "target_field": "4.22",
                "source_field": "4.30"
              }
            ]
          }
        ],
        "error_code": "E012"
      }
    ],
    "reference_data_dependencies": [
      "regional-udo-codes"
    ],
    "freeze_notes": [
      "Field 4.29 may be populated while 4.30 is blank (e.g. cessation/expiry without revocation). Field 4.30 populated requires 4.29 populated."
    ]
  },
  "field_policies": [
    {"id":"4.20","physical":{"start":1,"length":6,"type":"numeric","source_type":"N","alignment":{"kind":"right","pad":"0"}},"missing_value":{"kind":"exact","raw":"000000"},"presence":{"base":"required","required_if":[],"forbidden_if":[]},"constraints":[{"kind":"reference_data","dataset":"regional-udo-codes"}]},
    {"id":"4.21","physical":{"start":7,"length":15,"type":"alphanumeric","source_type":"AN","alignment":{"kind":"left","pad":" "}},"missing_value":{"kind":"spaces","raw":"               "},"presence":{"base":"required","required_if":[],"forbidden_if":[]},"constraints":[{"kind":"structure_only"}]},
    {"id":"4.22","physical":{"start":22,"length":10,"type":"alphanumeric","source_type":"AN","alignment":{"kind":"left","pad":" "}},"missing_value":{"kind":"spaces","raw":"          "},"presence":{"base":"required","required_if":[],"forbidden_if":[]},"constraints":[{"kind":"structure_only"}]},
    {"id":"4.23","physical":{"start":32,"length":1,"type":"numeric","source_type":"N","alignment":{"kind":"right","pad":"0"}},"missing_value":{"kind":"exact","raw":"0"},"presence":{"base":"required","required_if":[],"forbidden_if":[]},"constraints":[{"kind":"allowed_values","values":["1","2"]}]},
    {"id":"4.24","physical":{"start":33,"length":1,"type":"numeric","source_type":"N","alignment":{"kind":"right","pad":"0"}},"missing_value":{"kind":"exact","raw":"0"},"presence":{"base":"required","required_if":[],"forbidden_if":[]},"constraints":[{"kind":"allowed_values","values":["1","2"]}]},
    {"id":"4.25","physical":{"start":34,"length":4,"type":"numeric","source_type":"N","alignment":{"kind":"right","pad":"0"}},"missing_value":{"kind":"exact","raw":"0000"},"presence":{"base":"required","required_if":[],"forbidden_if":[]},"constraints":[{"kind":"digits_only"}]},
    {"id":"4.26","physical":{"start":38,"length":8,"type":"date","source_type":"D","alignment":{"kind":"exact"}},"missing_value":{"kind":"spaces","raw":"        "},"presence":{"base":"required","required_if":[],"forbidden_if":[]},"constraints":[{"kind":"date","format":"DDMMYYYY","calendar_valid":true}]},
    {"id":"4.27","physical":{"start":46,"length":3,"type":"numeric","source_type":"N","alignment":{"kind":"right","pad":"0"}},"missing_value":{"kind":"never_missing"},"presence":{"base":"required","required_if":[],"forbidden_if":[]},"constraints":[{"kind":"digits_only"},{"kind":"numeric_range","min":0,"max":999}]},
    {"id":"4.28","physical":{"start":49,"length":1,"type":"numeric","source_type":"N","alignment":{"kind":"right","pad":"0"}},"missing_value":{"kind":"exact","raw":"0"},"presence":{"base":"required","required_if":[],"forbidden_if":[]},"constraints":[{"kind":"allowed_values","values":["1","2","3","4","5"]}]},
    {"id":"4.29","physical":{"start":50,"length":8,"type":"date","source_type":"D","alignment":{"kind":"exact"}},"missing_value":{"kind":"spaces","raw":"        "},"presence":{"base":"optional","required_if":[],"forbidden_if":[]},"constraints":[{"kind":"date_if_present","format":"DDMMYYYY","calendar_valid":true}]},
    {"id":"4.30","physical":{"start":58,"length":10,"type":"alphanumeric","source_type":"AN","alignment":{"kind":"left","pad":" "}},"missing_value":{"kind":"spaces","raw":"          "},"presence":{"base":"optional","required_if":[],"forbidden_if":[]},"constraints":[{"kind":"structure_only_if_present"}]}
  ]
}
```
