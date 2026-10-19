# CAD — Regole, join e identità (T-P0-05)

Verifica documentale del 2026-10-19. Nessuna implementazione Rust o modifica
dei JSON ricevuti. Le dichiarazioni ricevute non provano eseguibilità: G0,
compiler e test normativi restano da completare. Le decisioni umane correnti
sono in [DECISIONS.md](../DECISIONS.md).

Riferimenti: ARCH §21–22; DET §35–37, §66–72, §91, §107, §149, §166–174;
Master Plan §4, §9/P0, §14; prompt T-P0-05.

## Convenzioni

Le tuple sono ordinate; la costruzione della chiave deve essere collision-safe
(componenti con lunghezza o struttura equivalente), senza separatori ambigui.
Il confronto usa il value model e il padding dichiarati, senza trim globale.
PK intra-trace, chiave di join, identità diagnostica e identità storica sono
nozioni distinte. Il contesto effimero conserva soltanto le proiezioni richieste;
lo storico persistente appartiene al core generico separato.

`Violation`, `NotApplicable` e `Unknown` sono le alternative del contratto
`MissingRelatedRecordPolicy`; `NOT_EVALUABLE` è l’esito documentato per input
CAD indisponibili, non un quarto valore introdotto nell’enum.
`required` nei blocchi sotto è un attributo legacy di join ricevuto, non
un nuovo flag di presenza dei campi. `at_least_one` esprime esistenza su un
insieme (`OneOrMany`), non autorizza a scegliere il primo elemento.

## PK e identità

| Trace | PK intra-trace ricevuta | Identità evento / diagnostica | Identità storica |
|---|---|---|---|
| A | (1.1,1.23) | Evento 1.23; CF o altro ID 1.2, tipo 1.3 | Namespace ULSS 1.1 + evento |
| B1 | (2.1,2.36) | Evento 2.1; progressivo 2.36 | ULSS + B1 + 2.36 → 2.1 |
| B2 | (3.1,3.5) | Evento 3.1; progressivo 3.5 | ULSS + B2 + 3.5 → 3.1 |
| C | (4.1,4.2,4.4,4.5,4.11) | Evento 4.1; progressivo 4.11 | ULSS + C + 4.11 → 4.1 |
| D | (5.1,5.2,5.3,5.4,5.7) | Evento 5.1; progressivo 5.7 | ULSS + D + 5.7 → 5.1 |

Tutte le PK hanno `uniqueness_scope: trace`, `E_DUPLICATE_RECORD_KEY`.
I progressivi C/D fanno parte delle rispettive PK, ma non della join D→C.
L’associazione storica progressivo→evento è immutabile nel namespace ULSS,
anche dopo annullamento; correzione di data o figura non equivale a cambio
evento. Il core persiste solo proiezioni protette (DET §172), senza CF o
righe sanitarie in chiaro per default. I binding diagnostici sono inventariati,
non una nuova serializzazione `RawIdentitySpec`; 1.2 può essere un altro ID.

## Namespace e join dichiarati

`sending_ulss` obbligatorio (501–509) delimita l’invio; ogni A.1.1 deve
corrispondere. L’evento da solo non è globalmente univoco: i child non
contengono ULSS. Fonte vincolante: `CAD_RUNTIME_PROFILE.json`,
`submission_scope` / G06; DET §171.

| ID | Sorgente → target e chiavi ordinate | Cardinalità ricevuta | MissingRelatedRecordPolicy / contesto |
|---|---|---|---|
| b1_to_a_event | B1.(2.1) → A.(1.23) | exactly_one | Violation se A disponibile senza parent; Unknown/NOT_EVALUABLE se A assente e PROCEED; stessa ULSS |
| b2_to_a_event | B2.(3.1) → A.(1.23) | exactly_one | Violation se A disponibile senza parent; Unknown/NOT_EVALUABLE se A assente e PROCEED; stessa ULSS |
| c_to_a_event | C.(4.1) → A.(1.23) | exactly_one | Violation se A disponibile senza parent; Unknown/NOT_EVALUABLE se A assente e PROCEED; stessa ULSS |
| d_to_a_event | D.(5.1) → A.(1.23) | exactly_one | Violation se A disponibile senza parent; Unknown/NOT_EVALUABLE se A assente e PROCEED; stessa ULSS |
| d_to_c_access | D.(5.1,5.2,5.3,5.4) → C.(4.1,4.2,4.4,4.5) | at_least_one | Violation se C disponibile senza riscontro, E_CAD_MISSING_ACCESS_FOR_PROVISION; Unknown/NOT_EVALUABLE se C assente |

Le policy sono ricostruite da `required=true`, errore dove presente e
contratto esplicito sui file mancanti; nessun campo JSON normalizzato
è aggiunto qui. Cardinalità massima uno violata non si risolve con first().
Più C con progressivi diversi possono soddisfare una D: non imporre
uguaglianza fra 4.11 e 5.7 o unicità su soli quattro campi.

## Relazioni richieste dal profilo e dalle note

Le righe seguenti sono requisiti da compilare successivamente, non join
già contenuti nell’array legacy. Sono sempre delimitate da ULSS ed evento.

| Relazione | Campi minimi / cardinalità | Target mancante e limiti |
|---|---|---|
| Età CAD-05/07 | B1.(2.1,2.4) → A.(1.23,1.11), exactly_one parent A; anno(2.4)−anno(1.11) | Come b1_to_a_event; non età compiuta. A.1.24 richiede il contesto delle B1 dell’evento: DEC-05-CAD-EVENT-CONTEXT per valori confliggenti |
| Iniziale evento | Evento → B1.(2.1,2.8), almeno una con 2.8=1 nello storico incluso corrente | Violation se universo disponibile e nessuna iniziale; Unknown se mancano input necessari; decisione autore DEC-05-CAD-VALUTAZIONE-INIZIALE |
| Sospensioni CAD-14 | C.(4.1,4.2,4.7) → tutte B2.(3.1,3.2,3.3); zero o più sospensioni | Nessuna sospensione in input disponibile: NotApplicable; B2 indisponibile: Unknown; relazione su insieme, DEC-05-CAD-SET-CONTRACT per forma tipizzata |
| Conclusione CAD-14/T-01/T-03 | C.(4.1,4.2,4.7) → B1.(2.1,2.6,2.7); insieme di valutazioni | Nessuna conclusione valorizzata non prova una violazione; B1 indisponibile: Unknown; conflitti: DEC-05-CAD-EVENT-CONTEXT |
| Professione dal 2025 | C.4.5 / D.5.4 → inizio B1.2.4 nello stesso evento | Codice 15 vietato per inizio dal 2025; input mancante: Unknown; conflitti B1: DEC-05-CAD-EVENT-CONTEXT |
| Attività CAD-17 | Evento → tutte le coppie C/D sulla chiave a quattro campi; B1.2.4 fallback; intervalli B2 | Insieme vuoto verificato: fallback a inizio evento; D o altro input indispensabile assente: Unknown/NOT_EVALUABLE; DEC-05-CAD-SET-CONTRACT |

Il valore minimo «almeno una iniziale» non impone esattamente una e non
richiede un’iniziale nuova a ogni trimestre: risposta umana già registrata.
Le relazioni zero-o-più non vengono forzate in `OneOrMany`, perché DET §72
non definisce ancora un enum per insiemi opzionali: boundary contrattuale
esplicita da chiudere in T-P0-07.

## Cross-field locali e storico

- A: TEAM condiziona 1.4/1.5; tipo identificativo condiziona 1.8/1.9.
  Nessuna coerenza semantica 1.2/1.3 (CAD-03), nessun checksum CF dedotto.
- B1: UCPDOM condiziona presenza e divieto dei campi, rete 2.27 condiziona
  2.28, conclusione 2.6 richiede motivo 2.7; i due domini condizionati
  2.7/2.3 sono nelle regole ricevute. Sottoposizione 60 di 2.22: blank fuori
  UCPDOM già approvato G03; non riaprire metadati storici pending.
- C: branca 4.9 solo specialista (4.5=4); durata 4.10 solo figure OSS
  previste dal profilo ricevuto. Somme di durata e prevalenza gestione
  pubblica in 4.8 richiedono dati elementari non presenti: non inventare
  aggregazioni né riscrittura dei record. C.4.3 e D.5.6 sono totali,
  senza fonte elementare per certificarli.
- Patologia concomitante B1.2.12 obbligatoria: 000 valido, tre spazi
  mancanti; decisione umana supera missing exact000 ricevuto, senza lookup ICPC.
- CAD-14: sospensioni di ogni motivo, solo giorni strettamente interni;
  fine assente = aperta. Domicilio vietato dopo conclusione per qualunque
  motivo; dopo decesso vietate sedi 1,2,4,5,6, ammessa sede 3. Stesso giorno
  di confine tollerato; nessuna scelta casuale fra conclusioni confliggenti.
- CAD-17: fine trimestre, tre mesi di calendario, ultimo accesso C con D
  corrispondente o data inizio; sospensioni unite mettono in pausa. G04
  nel runtime profile specifica clamp a fine mese e proroga fino a punto
  fisso: il testo più vecchio del decision register non riapre tale scelta.
- Storico: continuità fra anni; sessione approvata immutabile; progressivo
  stesso evento aggiornabile/riattivabile; altro evento bloccante. File
  assente non annulla record; file presente e vuoto è un caso distinto.
  Missing trace: warning e PROCEED/ABORT prima della revisione storica;
  preview e approvazione precedono commit atomico, non il join effimero.

Il profilo richiede sette estensioni del compiler: inventariate integralmente
sotto. `requires_engine_extension_or_decision` non significa disabilitazione
autorizzata: requisiti non supportati producono schema error. La vecchia
frase «outside generic core» sul registry è superata da ADR-PERSIST-001 e
DET §166–174: core persistente generico, policy CAD esterna.

## Boundary contrattuali (nessuna nuova scelta normativa)

### DEC-05-CAD-EVENT-CONTEXT — DA FORMALIZZARE

B1 ammette più valutazioni nello stesso evento. Il contratto storico impone
diagnostica in caso di conclusioni confliggenti, non first(). Anche età e
anno inizio devono usare un contesto non ambiguo. Il nome/codice diagnostico,
la riduzione deterministica e il lowering degli insiemi saranno formalizzati
in T-P0-07; non attribuire ad A.1.24 un’unica B1 scelta casualmente.

### DEC-05-CAD-SET-CONTRACT — DA FORMALIZZARE

DET §72 enumera ZeroOrOne/ExactlyOne/OneOrMany; il profilo CAD richiede anche
insiemi vuoti leciti, union/max/filter e confronto storico. Non esiste ancora
una serializzazione approvata per questi piani. T-P0-07 deve proporla e farla
revisionare, senza trasformare «zero sospensioni» in errore o input assente
in insieme vuoto. Persistenza e primitive temporali restano task future.

Casi limite da coprire nelle task runtime: stesso evento in ULSS diverse;
più A nella stessa ULSS; più C per D senza uguagliare progressivi; iniziale
solo nello storico; iniziale assente con universo noto o incompleto; B1
confliggenti; B2 aperta/sovrapposta; reinvio e riattivazione; file vuoto
contro assente. Nessun test normativo eseguito con questa documentazione.

## Inventario ricevuto verificabile

I blocchi sono proiezioni documentali esatte dei JSON ricevuti, non un
nuovo formato di schema. Le correzioni approvate sopra non sono applicate
a questi snapshot. Ogni regola conserva condizione, assertion e codice;
le presence condizionali conservano tutte le dipendenze. Le policy fisiche
e i constraint di campo completi sono nella [matrice presenza](presence-matrix.md).

### Trace A — 0 regole ricevute

Fonte: `CAD_REVISED_CANDIDATE/trace_a.json` nello starter kit; tabelle: [A-fields.json](A-fields.json).

```json
{
  "trace_id": "A",
  "record_key": {"fields": ["1.1", "1.23"], "uniqueness_scope": "trace", "error_code": "E_DUPLICATE_RECORD_KEY"},
  "joins": [],
  "named_conditions": {},
  "derived_values": {},
  "repeated_groups": [],
  "documented_non_executable_rules": [{"id": "CAD-AGE-UNDER18", "fields": ["1.24"], "policy": "requires_engine_extension_or_decision", "reason": "Required when year(2.4)-year(1.11)<=17; cross-trace derived-year expression not present in frozen DSL. Implement via generic extension before executable freeze."}, {"id": "CAD-IDENTIFIER-CROSSCHECK", "fields": ["1.2", "1.3"], "policy": "requires_engine_extension_or_decision", "reason": "Deliberately no semantic cross-field identifier check per user decision."}],
  "rules": [],
  "conditional_presence": [{"field": "1.4", "presence": {"base": "optional", "required_if": [{"eq": {"ref": {"field": "1.3"}, "value": "4"}}], "forbidden_if": []}}, {"field": "1.5", "presence": {"base": "optional", "required_if": [{"eq": {"ref": {"field": "1.3"}, "value": "4"}}], "forbidden_if": []}}, {"field": "1.8", "presence": {"base": "optional", "required_if": [{"ne": {"ref": {"field": "1.3"}, "value": "1"}}], "forbidden_if": []}}, {"field": "1.9", "presence": {"base": "optional", "required_if": [{"ne": {"ref": {"field": "1.3"}, "value": "1"}}], "forbidden_if": []}}]
}
```

### Trace B1 — 2 regole ricevute

Fonte: `CAD_REVISED_CANDIDATE/trace_b1.json` nello starter kit; tabelle: [B1-fields.json](B1-fields.json).

```json
{
  "trace_id": "B1",
  "record_key": {"fields": ["2.1", "2.36"], "uniqueness_scope": "trace", "error_code": "E_DUPLICATE_RECORD_KEY"},
  "joins": [{"id": "b1_to_a_event", "target_trace": "A", "source_fields": ["2.1"], "target_fields": ["1.23"], "required": true, "cardinality": "exactly_one", "context_scope": "same_ulss_submission", "note": "A primary key additionally contains ULSS 1.1; unique matching by event alone requires scope/uniqueness invariant."}],
  "named_conditions": {"is_ucpdom": {"eq": {"ref": {"field": "2.3"}, "value": "2"}}, "belongs_to_network": {"eq": {"ref": {"field": "2.27"}, "value": "1"}}, "has_conclusion": {"is_present": {"ref": {"field": "2.6"}}}},
  "derived_values": {},
  "repeated_groups": [],
  "documented_non_executable_rules": [{"id": "CAD-AGE-3-6", "fields": ["2.17", "2.18"], "policy": "requires_engine_extension_or_decision", "reason": "Required if year(2.4)-year(1.11)>=3/6; requires cross-trace age-year derived expression."}, {"id": "CAD-17", "fields": ["2.4", "2.6", "2.7", "3.2", "3.3", "4.2", "5.2"], "policy": "requires_engine_extension_or_decision", "reason": "Administrative closing after 3 calendar months inactivity, calendar-month threshold + pause duration; longitudinal runtime rule. See CAD_HISTORICAL_PROFILE.md."}, {"id": "CAD-2.11-12-ICPC", "fields": ["2.11", "2.12"], "policy": "requires_engine_extension_or_decision", "reason": "No ICPC lookup per user decision; 2.12=000 reserved for absent concomitant diagnosis."}],
  "rules": [{"id": "CAD-R-2.7-UC-10", "when": {"eq": {"ref": {"field": "2.3"}, "value": "2"}}, "assertions": [{"kind": "not_allowed_values", "field": "2.7", "values": ["10"]}], "error_code": "E_CAD_CONCLUSION_CODE"}, {"id": "CAD-R-2.7-CD-13", "when": {"eq": {"ref": {"field": "2.3"}, "value": "1"}}, "assertions": [{"kind": "not_allowed_values", "field": "2.7", "values": ["13"]}], "error_code": "E_CAD_CONCLUSION_CODE"}],
  "conditional_presence": [{"field": "2.5", "presence": {"base": "optional", "required_if": [{"condition": "is_ucpdom"}], "forbidden_if": [{"ne": {"ref": {"field": "2.3"}, "value": "2"}}]}}, {"field": "2.7", "presence": {"base": "optional", "required_if": [{"condition": "has_conclusion"}], "forbidden_if": []}}, {"field": "2.27", "presence": {"base": "optional", "required_if": [{"condition": "is_ucpdom"}], "forbidden_if": [{"ne": {"ref": {"field": "2.3"}, "value": "2"}}]}}, {"field": "2.28", "presence": {"base": "optional", "required_if": [{"and": [{"condition": "is_ucpdom"}, {"condition": "belongs_to_network"}]}], "forbidden_if": [{"not": {"and": [{"condition": "is_ucpdom"}, {"condition": "belongs_to_network"}]}}]}}, {"field": "2.29", "presence": {"base": "optional", "required_if": [{"condition": "is_ucpdom"}], "forbidden_if": [{"ne": {"ref": {"field": "2.3"}, "value": "2"}}]}}, {"field": "2.30", "presence": {"base": "optional", "required_if": [], "forbidden_if": [{"ne": {"ref": {"field": "2.3"}, "value": "2"}}]}}, {"field": "2.31", "presence": {"base": "optional", "required_if": [], "forbidden_if": [{"ne": {"ref": {"field": "2.3"}, "value": "2"}}]}}, {"field": "2.32", "presence": {"base": "optional", "required_if": [], "forbidden_if": [{"ne": {"ref": {"field": "2.3"}, "value": "2"}}]}}, {"field": "2.33", "presence": {"base": "optional", "required_if": [], "forbidden_if": [{"ne": {"ref": {"field": "2.3"}, "value": "2"}}]}}, {"field": "2.34", "presence": {"base": "optional", "required_if": [{"condition": "is_ucpdom"}], "forbidden_if": [{"ne": {"ref": {"field": "2.3"}, "value": "2"}}]}}, {"field": "2.35", "presence": {"base": "optional", "required_if": [{"condition": "is_ucpdom"}], "forbidden_if": [{"ne": {"ref": {"field": "2.3"}, "value": "2"}}]}}]
}
```

### Trace B2 — 0 regole ricevute

Fonte: `CAD_REVISED_CANDIDATE/trace_b2.json` nello starter kit; tabelle: [B2-fields.json](B2-fields.json).

```json
{
  "trace_id": "B2",
  "record_key": {"fields": ["3.1", "3.5"], "uniqueness_scope": "trace", "error_code": "E_DUPLICATE_RECORD_KEY"},
  "joins": [{"id": "b2_to_a_event", "target_trace": "A", "source_fields": ["3.1"], "target_fields": ["1.23"], "required": true, "cardinality": "exactly_one", "context_scope": "same_ulss_submission", "note": "A primary key additionally contains ULSS 1.1; unique matching by event alone requires scope/uniqueness invariant."}],
  "named_conditions": {},
  "derived_values": {},
  "repeated_groups": [],
  "documented_non_executable_rules": [],
  "rules": [],
  "conditional_presence": []
}
```

### Trace C — 0 regole ricevute

Fonte: `CAD_REVISED_CANDIDATE/trace_c.json` nello starter kit; tabelle: [C-fields.json](C-fields.json).

```json
{
  "trace_id": "C",
  "record_key": {"fields": ["4.1", "4.2", "4.4", "4.5", "4.11"], "uniqueness_scope": "trace", "error_code": "E_DUPLICATE_RECORD_KEY"},
  "joins": [{"id": "c_to_a_event", "target_trace": "A", "source_fields": ["4.1"], "target_fields": ["1.23"], "required": true, "cardinality": "exactly_one", "context_scope": "same_ulss_submission", "note": "A primary key additionally contains ULSS 1.1; unique matching by event alone requires scope/uniqueness invariant."}],
  "named_conditions": {},
  "derived_values": {},
  "repeated_groups": [],
  "documented_non_executable_rules": [{"id": "CAD-14", "fields": ["3.1", "3.2", "3.3", "4.1", "4.2", "4.7"], "policy": "requires_engine_extension_or_decision", "reason": "Sourced B2 suspension disallows home access 4.7=1 on interior days; temporal multi-record join context required. See CAD_HISTORICAL_PROFILE.md."}, {"id": "CAD-4.5-15", "fields": ["4.5"], "policy": "requires_engine_extension_or_decision", "reason": "Code 15 not valid for care episodes starting in 2025; needs cross-trace source date."}, {"id": "CAD-4.9-SPECIALIST", "fields": ["4.5", "4.9"], "policy": "requires_engine_extension_or_decision", "reason": "Branca specialistica only for professional code 4; no official branch lookup supplied."}, {"id": "CAD-4.10-OSS", "fields": ["4.5", "4.10"], "policy": "requires_engine_extension_or_decision", "reason": "Duration only for OSS; relevant OSS codes need source-specific disambiguation before auto-rule."}],
  "rules": [],
  "conditional_presence": [{"field": "4.9", "presence": {"base": "optional", "required_if": [], "forbidden_if": [{"ne": {"ref": {"field": "4.5"}, "value": "4"}}]}}, {"field": "4.10", "presence": {"base": "optional", "required_if": [], "forbidden_if": [{"not_in": {"ref": {"field": "4.5"}, "values": ["60", "61"]}}]}}]
}
```

### Trace D — 0 regole ricevute

Fonte: `CAD_REVISED_CANDIDATE/trace_d.json` nello starter kit; tabelle: [D-fields.json](D-fields.json).

```json
{
  "trace_id": "D",
  "record_key": {"fields": ["5.1", "5.2", "5.3", "5.4", "5.7"], "uniqueness_scope": "trace", "error_code": "E_DUPLICATE_RECORD_KEY"},
  "joins": [{"id": "d_to_a_event", "target_trace": "A", "source_fields": ["5.1"], "target_fields": ["1.23"], "required": true, "cardinality": "exactly_one", "context_scope": "same_ulss_submission", "note": "A primary key additionally contains ULSS 1.1; unique matching by event alone requires scope/uniqueness invariant."}, {"id": "d_to_c_access", "target_trace": "C", "source_fields": ["5.1", "5.2", "5.3", "5.4"], "target_fields": ["4.1", "4.2", "4.4", "4.5"], "required": true, "cardinality": "at_least_one", "error_code": "E_CAD_MISSING_ACCESS_FOR_PROVISION", "note": "4.11 and 5.7 are PK components of their respective traces and are NOT cross-trace join fields."}],
  "named_conditions": {},
  "derived_values": {},
  "repeated_groups": [],
  "documented_non_executable_rules": [{"id": "CAD-5.4-15", "fields": ["5.4"], "policy": "requires_engine_extension_or_decision", "reason": "Code 15 not valid for care episodes starting in 2025; needs cross-trace source date."}],
  "rules": [],
  "conditional_presence": []
}
```

### Profilo runtime ricevuto

Fonte: `CAD_REVISED_CANDIDATE/CAD_RUNTIME_PROFILE.json`.

```json
{
  "profile": "CAD",
  "status": "RELEASE_CANDIDATE",
  "schema_dsl_version": "1.0.0",
  "extension_contract_version": "0.1.0",
  "historical": {
    "namespace": [
      "ULSS"
    ],
    "snapshot_period": [
      "year",
      "quarter"
    ],
    "immutable_approved_sessions": true,
    "on_missing_trace": "warn_and_ask_proceed_or_abort",
    "store_trace_presence": true,
    "missing_trace_cannot_cancel": true,
    "progressive_identity": {
      "B1": [
        "2.36",
        "2.1"
      ],
      "B2": [
        "3.5",
        "3.1"
      ],
      "C": [
        "4.11",
        "4.1"
      ],
      "D": [
        "5.7",
        "5.1"
      ]
    },
    "reject_reassigned_progressives": true,
    "allow_reactivation_same_event": true
  },
  "temporal": {
    "suspension_start": "3.2",
    "suspension_end": "3.3",
    "suspension_event": "3.1",
    "access_event": "4.1",
    "access_date": "4.2",
    "access_venue": "4.7",
    "suspension_forbid_home_access_inside_only": true,
    "allow_border_dates": true,
    "after_conclusion_forbid_venue": [
      "1"
    ],
    "after_death_forbid_venue": [
      "1",
      "2",
      "4",
      "5",
      "6"
    ],
    "condolence_venue": "3",
    "admin_closure": {
      "code": "12",
      "threshold": {
        "calendar_months": 3
      },
      "subtract_suspension_days": true,
      "last_activity": "last_C_date_with_matching_D",
      "if_no_activity": "B1.2.4",
      "evaluate_at": "end_of_quarter",
      "if_D_missing": "NOT_EVALUABLE",
      "pause_boundary_policy": "suspension days strictly between start and end dates; merge overlapping intervals; open end extends to quarter end; distinct from access tolerance",
      "calendar_month_semantics": "last serviced access date plus three calendar months (clamp invalid day to month end), then extend for paused interior suspension days until fixed point"
    }
  },
  "submission_scope": {
    "required_parameter": "sending_ulss",
    "allowed_ulss": [
      "501",
      "502",
      "503",
      "504",
      "505",
      "506",
      "507",
      "508",
      "509"
    ],
    "trace_A_composite_primary_key": [
      "1.1",
      "1.23"
    ],
    "validation": "Each A.1.1 must equal sending_ulss. All B1/B2/C/D inputs belong to exactly the declared sending_ulss; child event_id must match exactly one A.1.23 within same submitting ULSS; no global join on event_id across ULSS namespaces.",
    "absent_A": "If A is missing and user proceeds, A-parent joins are NOT_EVALUABLE; do not pretend missing parent violations imply orphan records."
  },
  "source_decisions": {
    "G-03": "position 60 of field 2.22 outside UCPDOM = exactly ASCII space",
    "G-04": "calendar months, pause day boundaries excluded",
    "G-05": "ICD9 lookup restricted to PDF annex dataset; no arbitrary unspecified codes",
    "G-06": "ULSS-scoped A parent key and mandatory submission scope"
  },
  "compiler_extensions_required": [
    {
      "name": "calendar_year_delta",
      "purpose": "A.1.24; B1.2.17 / B1.2.18",
      "sources": [
        "A.1.11",
        "B1.2.4"
      ],
      "rule": "year(B1.2.4)-year(A.1.11); not completed age; A.1.24 requires event context across multiple B1 rows"
    },
    {
      "name": "conditional_positional_bitset",
      "purpose": "B1.2.22 position 60 blank iff B1.2.3 != 2; positions 61-70 fixed 2"
    },
    {
      "name": "cross_trace_conditional_domains",
      "purpose": "C.4.5 and D.5.4 forbid code 15 for care start year >= 2025; C.4.7 temporal contextual restrictions"
    },
    {
      "name": "calendar_months_with_paused_intervals",
      "purpose": "CAD-17 three calendar months inactive, pause on union of B2 suspension intervals"
    },
    {
      "name": "historical_session_registry",
      "purpose": "CAD snapshots + ULSS scope + immutable approved quarters + progressive recycling checks; outside generic core"
    },
    {
      "name": "field_presence_logic",
      "purpose": "A.1.24 threshold <=17; B1.2.17 threshold >=3; B1.2.18 threshold >=6 using calendar_year_delta"
    },
    {
      "name": "contextual_temporal_constraints",
      "purpose": "C.4.7 suspension, death and post-conclusion temporal validation"
    }
  ],
  "schema_revision": "2026-10-16.reviewed-g02-g06"
}
```

## Evidenze

Controllo riproducibile: `python3 docs/verification/check_rules_and_joins.py`.
Report e limiti: [T-P0-05](../../verification/T-P0-05.md). Nessuna run CI
o prova di compilazione del DSL è dedotta dalla verifica documentale.
