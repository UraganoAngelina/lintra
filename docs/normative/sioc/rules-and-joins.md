# SIOC — Regole, join e identità (T-P0-05)

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

| Trace | PK intra-trace ricevuta | Campi di identità diagnostica individuati |
|---|---|---|
| 1 | (1.1, 1.17, 1.18) | ID episodio/scheda 1.1; CF o altro identificativo 1.2; tipo 1.3 |
| 2 | (2.1, 2.49, 2.50) | ID episodio/scheda 2.1 |
| 3 | (3.1, 3.2, 3.35, 3.36) | ID episodio/scheda 3.1; numero valutazione 3.2 |
| 4.1 | (4.1, 4.7, 4.8, 4.19.1) | Identificativi tecnici UDO 4.1, MRA 4.2, STS 4.19.1 |
| 4.2 | (4.20, 4.22) | Identificativi tecnici UDO 4.20, MRA 4.21, atto 4.22 |

Tutte le PK hanno `uniqueness_scope: trace` ed errore
`E_DUPLICATE_RECORD_KEY`. La ripetizione in altri invii rimane audit,
non errore automatico (DET §170, DEC-SIOC-HIST-01). La norma sull’unicità
struttura/anno non è dimostrata da una PK locale.

I campi sopra sono un inventario, non un `RawIdentitySpec` già approvato:
il binding ai target DET §36 sarà formalizzato in T-P0-07. `1.2` non è
sempre un CF; anonimato e identificativo indisponibile vanno conservati,
senza validare un altro identificativo come CF né usarlo come PK. Nessuna
estensione di episodi ai tracciati amministrativi 4.1/4.2 è dedotta.

## Join dichiarati e policy

| ID | Sorgente → target e chiavi ordinate | Cardinalità ricevuta | Target mancante / evidenza |
|---|---|---|---|
| trace1_to_trace2_episode | 1.(1.1,1.17,1.18) → 2.(2.1,2.49,2.50) | zero_or_one | NotApplicable, decisione autore DEC-05-SIOC-OPTIONAL-PARENT; esplicitare anagrafica presente e dati ricovero correlato assenti |
| trace2_to_trace1 | 2.(2.1,2.49,2.50) → 1.(1.1,1.17,1.18) | exactly_one | Violation, required=true + E_MISSING_PARENT_RECORD ricevuti |
| trace3_to_trace2 | 3.(3.1,3.35,3.36) → 2.(2.1,2.49,2.50) | exactly_one | Violation, required=true + E_MISSING_PARENT_RECORD ricevuti |
| trace3_to_trace1 | 3.(3.1,3.35,3.36) → 1.(1.1,1.17,1.18) | exactly_one | Violation, required=true + E_MISSING_PARENT_RECORD ricevuti |
| trace42_to_trace41 | 4.2.(4.20,4.21) → 4.1.(4.1,4.2) | at_least_one | Violation, required=true + E_MISSING_PARENT_RECORD ricevuti |

Un risultato multiplo con cardinalità massima uno non può essere risolto
con `first()`, anche se il target ha una PK: duplicati invalidi restano
possibili e devono essere diagnosticati. La join 4.2→4.1 usa due campi,
non l’intera PK di 4.1; più target sono quindi possibili per definizione.
Le dichiarazioni legacy non contengono il membro esplicito
`MissingRelatedRecordPolicy`: la tabella separa i casi determinati da
required/error_code dal caso opzionale ora risolto dall’autore. La serializzazione
normalizzata deve essere definita in T-P0-07, senza riscrivere il FROZEN.

## Relazioni implicite nelle assertion

`R-4.30-REFERENCES-4.22` è una ricerca di esistenza nel trace 4.2:
`source 4.30 = target 4.22`, quando 4.30 è presente, errore `E012`.
Zero risultati con universo di ricerca disponibile produce la violazione
dichiarata (MissingRelatedRecordPolicy: Violation). L’esistenza richiede
almeno un risultato (OneOrMany), non esattamente uno. Il match ricevuto
usa il solo ID 4.22 nell’intero trace 4.2; non aggiunge uguaglianza su UDO.
**DEC-05-SIOC-REVOKING-ACT** è chiusa come lettura dell’AST ricevuto: non
si introduce una nuova join exactly_one né una modifica normativa.
`POLICY-4.28-PREVIOUS-ACT` rimane non eseguita v1: non è la stessa relazione.

## Regole tra campi e tracciati

- Trace 1: anonimato, identificativo indisponibile, TEAM, nomi, residenza
  estera/italiana e relativi domini; l’anno di 1.1 dipende da 2.5 attraverso
  trace1_to_trace2_episode. Le presence condizionali sono incluse sotto.
- Trace 2: 2.10 dipende da 2.12 e dalla concatenazione 1.13+1.14 tramite
  trace2_to_trace1; `sending_region` deriva da 1.17 e condiziona 2.30/2.31.
  Ordinamento date 2.4≤2.5≤2.6; soggetto pagante e COT sono dipendenze locali.
- Trace 3: momento valutazione 3.3 e decesso 2.7 governano i sentinel e i
  domini delle scale; l’età compiuta deriva da 1.9 e 2.5. Tutte le 57 regole
  ricevute, inclusi versi inversi dei sentinel, sono riportate sotto.
- Trace 4.1: ordinamento 4.18≤4.19. Trace 4.2: 4.30 presente richiede 4.29,
  ma la presenza di 4.29 non impone 4.30; ricerca atto revocante come sopra.
- Gruppi trace 2: problemi socio-familiari con valore inutilizzato 99;
  diagnosi con unicità dei non mancanti; procedure senza unicità o
  contiguità imposte. Non confondere slot di uno stesso record con join.

Le regole anno ricevute e i constraint di 1.1/2.1/3.1 sono superati, per
il solo ramo ingressi 2025 conclusi nel 2026, dalla decisione
DEC-05-SIOC-ID-2026: alfanumerico valorizzato, diverso da tutti zeri,
20 byte fisici, padding a destra con spazi. Non applicare a questo ramo
lunghezza logica otto, sole cifre, prefisso anno o progressivo. La scelta
del ramo richiede il contesto di ricovero: un dato non disponibile non
autorizza né a validare automaticamente né a scegliere un altro anno.

Le policy non eseguibili ricevute (semantica CF, residenza Veneto, COVID,
atto precedente) rimangono esplicite nell’inventario; non diventano
regole eseguite con questa consegna. Reference data: [schede](../reference-datasets.md).

## Decisioni residue e casi limite

### DEC-05-SIOC-OPTIONAL-PARENT — RISOLTA dall’autore

Fonte: trace 1, `joins[0]`, R-1.1-ADMISSION-YEAR. Decisione ricevuta
il 2026-10-19: target assente → `NotApplicable`, nessun errore di parent
mancante. Esplicitare nell’esito «Anagrafica presente; dati del ricovero
correlato assenti», con riferimento al join. La proposta Unknown è superata
per questo solo caso. Se il target è presente ma 2.5 è invalido, conservare
l’errore locale e non confrontare un anno fittizio; target multipli non
sono coperti dalla deroga. [Decisione completa](../DECISIONS.md).

### DEC-05-SIOC-REVOKING-ACT — CHIUSA come lettura dell’AST

Fonte: trace 4.2, R-4.30-REFERENCES-4.22: record_exists sul solo target
4.22 = source 4.30, almeno un riscontro; nessun filtro UDO dichiarato.
La nota 4.22 dice «Numero univoco del record del tracciato 4.2» e 4.30
«ID dell'atto di revoca del presente». La PK (4.20,4.22) non autorizza
una nuova uguaglianza con 4.20 o un nuovo vincolo di unicità globale.
Si conserva il match ricevuto senza rafforzarlo. Il caso di file
indispensabile indisponibile resta Unknown (N-10/14), distinto da
zero record in un universo disponibile, che produce E012.

Casi di verifica documentati: zero/uno/più target; stesso ID episodio con
azienda o struttura diversa; stesso atto con UDO diversa; ID legacy alfanumerico
correttamente paddato e ID tutti zeri; anonimato senza CF; più righe UDO
legittime. Non sono fixture approvate né test runtime già eseguiti.

## Inventario ricevuto verificabile

I blocchi sono proiezioni documentali esatte dei JSON ricevuti, non un
nuovo formato di schema. Le correzioni approvate sopra non sono applicate
a questi snapshot. Ogni regola conserva condizione, assertion e codice;
le presence condizionali conservano tutte le dipendenze. Le policy fisiche
e i constraint di campo completi sono nella [matrice presenza](presence-matrix.md).

### Trace 1 — 8 regole ricevute

Fonte: `SCHEMA_INPUTS/SIOC_2026-04-20_FROZEN/trace_1.json` nello starter kit; tabelle: [1-fields.json](1-fields.json).

```json
{
  "trace_id": "1",
  "record_key": {"fields": ["1.1", "1.17", "1.18"], "uniqueness_scope": "trace", "error_code": "E_DUPLICATE_RECORD_KEY"},
  "joins": [{"id": "trace1_to_trace2_episode", "target_trace": "2", "source_fields": ["1.1", "1.17", "1.18"], "target_fields": ["2.1", "2.49", "2.50"], "required": false, "cardinality": "zero_or_one"}],
  "named_conditions": {"is_anonymous": {"eq": {"ref": {"field": "1.3"}, "value": "98"}}, "identifier_unavailable": {"eq": {"ref": {"field": "1.3"}, "value": "09"}}, "is_foreign_resident": {"eq": {"ref": {"field": "1.13"}, "value": "999"}}, "is_italian_resident": {"ne": {"ref": {"field": "1.13"}, "value": "999"}}},
  "derived_values": {},
  "repeated_groups": [],
  "documented_non_executable_rules": [{"id": "POLICY-1.2-CF-SEMANTICS", "field": "1.2", "policy": "not_enforced_in_v1", "reason": "The regional trace defines when CF/ENI/STP/TEAM/other identifiers are used but does not define a confirmed checksum/semantic-validation depth. Canonical v1 validates physical representation and identifier-type-dependent presence only."}, {"id": "POLICY-1.6-VENETO-RESIDENCY", "field": "1.6", "policy": "not_enforced_in_v1", "source_note": "Il campo va valorizzato, in combinazione con il codice fiscale, per i soli residenti in Veneto. \nNon compilare nei casi coperti da riservatezza ai sensi di legge", "reason": "Operational interpretation of 'in combinazione con il codice fiscale, per i soli residenti in Veneto' was not functionally confirmed; v1 preserves the note but does not infer a resident/identifier-type presence rule."}],
  "rules": [{"id": "R-1.1-ADMISSION-YEAR", "when": {"is_present": {"ref": {"field": {"trace": "2", "field": "2.5", "join": "trace1_to_trace2_episode"}}}}, "assertions": [{"kind": "substring_equals_derived_year", "field": "1.1", "start": 1, "length": 2, "source_ref": {"field": {"trace": "2", "field": "2.5", "join": "trace1_to_trace2_episode"}}, "source_format": "DDMMYYYY", "year_digits": 2}], "error_code": "E010"}, {"id": "R-1.2-ANONYMOUS-SENTINEL", "when": {"condition": "is_anonymous"}, "assertions": [{"kind": "raw_equals", "field": "1.2", "value": "AAAAAAAAAAAAAAAAAAAA"}], "error_code": "E010"}, {"id": "R-1.7-ANONYMOUS-NAME", "when": {"condition": "is_anonymous"}, "assertions": [{"kind": "logical_equals", "field": "1.7", "value": "ANONIMO"}], "error_code": "E010"}, {"id": "R-1.8-ANONYMOUS-NAME", "when": {"condition": "is_anonymous"}, "assertions": [{"kind": "logical_equals", "field": "1.8", "value": "ANONIMO"}], "error_code": "E010"}, {"id": "R-1.14-FOREIGN", "when": {"condition": "is_foreign_resident"}, "assertions": [{"kind": "logical_equals", "field": "1.14", "value": "999"}], "error_code": "E010"}, {"id": "R-1.14-ITALIAN", "when": {"condition": "is_italian_resident"}, "assertions": [{"kind": "reference_data", "field": "1.14", "dataset": "health-authority-codes"}], "error_code": "E011"}, {"id": "R-1.15-FOREIGN", "when": {"and": [{"condition": "is_foreign_resident"}, {"ne": {"ref": {"field": "1.15"}, "value": "999999"}}]}, "assertions": [{"kind": "composite", "field": "1.15", "segments": [{"start": 1, "length": 3, "equals": "999"}, {"start": 4, "length": 3, "reference_data": "foreign-country-codes"}]}], "error_code": "E010"}, {"id": "R-1.15-ITALIAN", "when": {"and": [{"condition": "is_italian_resident"}, {"ne": {"ref": {"field": "1.15"}, "value": "999999"}}]}, "assertions": [{"kind": "reference_data", "field": "1.15", "dataset": "istat-municipality-codes"}], "error_code": "E011"}],
  "conditional_presence": [{"field": "1.2", "presence": {"base": "optional", "required_if": [{"not": {"condition": "identifier_unavailable"}}], "forbidden_if": [{"condition": "identifier_unavailable"}]}}, {"field": "1.4", "presence": {"base": "optional", "required_if": [{"eq": {"ref": {"field": "1.3"}, "value": "04"}}], "forbidden_if": [{"ne": {"ref": {"field": "1.3"}, "value": "04"}}]}}, {"field": "1.5", "presence": {"base": "optional", "required_if": [], "forbidden_if": [{"condition": "is_anonymous"}]}}, {"field": "1.6", "presence": {"base": "optional", "required_if": [], "forbidden_if": [{"condition": "is_anonymous"}]}}, {"field": "1.7", "presence": {"base": "optional", "required_if": [{"condition": "is_anonymous"}], "forbidden_if": []}}, {"field": "1.8", "presence": {"base": "optional", "required_if": [{"condition": "is_anonymous"}], "forbidden_if": []}}, {"field": "1.9", "presence": {"base": "optional", "required_if": [{"not": {"condition": "is_anonymous"}}], "forbidden_if": [{"condition": "is_anonymous"}]}}]
}
```

### Trace 2 — 4 regole ricevute

Fonte: `SCHEMA_INPUTS/SIOC_2026-04-20_FROZEN/trace_2.json` nello starter kit; tabelle: [2-fields.json](2-fields.json).

```json
{
  "trace_id": "2",
  "record_key": {"fields": ["2.1", "2.49", "2.50"], "uniqueness_scope": "trace", "error_code": "E_DUPLICATE_RECORD_KEY"},
  "joins": [{"id": "trace2_to_trace1", "target_trace": "1", "source_fields": ["2.1", "2.49", "2.50"], "target_fields": ["1.1", "1.17", "1.18"], "required": true, "cardinality": "exactly_one", "error_code": "E_MISSING_PARENT_RECORD"}],
  "named_conditions": {"ssr_fully_pays": {"eq": {"ref": {"field": "2.12"}, "value": "1"}}, "self_paying": {"eq": {"ref": {"field": "2.12"}, "value": "2"}}, "cot_mediated": {"eq": {"ref": {"field": "2.25"}, "value": "1"}}, "cot_not_mediated": {"eq": {"ref": {"field": "2.25"}, "value": "2"}}},
  "derived_values": {"sending_region": {"kind": "substring", "source_ref": {"field": {"trace": "1", "field": "1.17", "join": "trace2_to_trace1"}}, "start": 1, "length": 3}},
  "repeated_groups": [{"id": "socio_family_problems", "fields": ["2.13", "2.14", "2.15", "2.16", "2.17", "2.18", "2.19", "2.20", "2.21", "2.22"], "required_slots": ["2.13", "2.14", "2.15", "2.16", "2.17", "2.18", "2.19", "2.20", "2.21", "2.22"], "optional_slots": [], "contiguity_required": false, "unique_non_missing": false, "unused_slot_value": "99"}, {"id": "diagnoses", "fields": ["2.32", "2.33", "2.34", "2.35", "2.36", "2.37"], "required_slots": ["2.32"], "optional_slots": ["2.33", "2.34", "2.35", "2.36", "2.37"], "contiguity_required": false, "unique_non_missing": true}, {"id": "procedures", "fields": ["2.38", "2.39", "2.40", "2.41", "2.42", "2.43", "2.44", "2.45", "2.46", "2.47", "2.48"], "required_slots": [], "optional_slots": ["2.38", "2.39", "2.40", "2.41", "2.42", "2.43", "2.44", "2.45", "2.46", "2.47", "2.48"], "contiguity_required": false, "unique_non_missing": false}],
  "documented_non_executable_rules": [],
  "rules": [{"id": "R-2.1-ADMISSION-YEAR", "when": {"const": true}, "assertions": [{"kind": "substring_equals_derived_year", "field": "2.1", "start": 1, "length": 2, "source_ref": {"field": "2.5"}, "source_format": "DDMMYYYY", "year_digits": 2}], "error_code": "E010"}, {"id": "R-2.10-SSR-PAYER", "when": {"condition": "ssr_fully_pays"}, "assertions": [{"kind": "equals_concat", "field": "2.10", "parts": [{"field": {"trace": "1", "field": "1.13", "join": "trace2_to_trace1"}}, {"field": {"trace": "1", "field": "1.14", "join": "trace2_to_trace1"}}]}], "error_code": "E010"}, {"id": "R-2.4-LE-2.5", "when": {"const": true}, "assertions": [{"kind": "field_compare", "left": {"field": "2.4"}, "operator": "<=", "right": {"field": "2.5"}, "value_type": "date"}], "error_code": "E_TEMPORAL_ORDER"}, {"id": "R-2.5-LE-2.6", "when": {"const": true}, "assertions": [{"kind": "field_compare", "left": {"field": "2.5"}, "operator": "<=", "right": {"field": "2.6"}, "value_type": "date"}], "error_code": "E_TEMPORAL_ORDER"}],
  "conditional_presence": [{"field": "2.10", "presence": {"base": "optional", "required_if": [{"condition": "ssr_fully_pays"}], "forbidden_if": []}}, {"field": "2.11", "presence": {"base": "optional", "required_if": [{"condition": "self_paying"}], "forbidden_if": [{"not": {"condition": "self_paying"}}]}}, {"field": "2.26", "presence": {"base": "optional", "required_if": [{"condition": "cot_mediated"}], "forbidden_if": [{"condition": "cot_not_mediated"}]}}, {"field": "2.29", "presence": {"base": "optional", "required_if": [{"not_in": {"ref": {"field": "2.2"}, "values": ["01", "02"]}}], "forbidden_if": []}}, {"field": "2.30", "presence": {"base": "optional", "required_if": [{"and": [{"ne": {"ref": {"field": "2.29"}, "value": "999"}}, {"not_in": {"ref": {"field": "2.2"}, "values": ["01", "02"]}}, {"derived_eq": {"ref": {"field": "2.29"}, "derived_value": "sending_region"}}]}], "forbidden_if": []}}, {"field": "2.31", "presence": {"base": "optional", "required_if": [{"and": [{"ne": {"ref": {"field": "2.29"}, "value": "999"}}, {"not_in": {"ref": {"field": "2.2"}, "values": ["01", "02", "04", "09"]}}, {"derived_eq": {"ref": {"field": "2.29"}, "derived_value": "sending_region"}}]}], "forbidden_if": []}}]
}
```

### Trace 3 — 57 regole ricevute

Fonte: `SCHEMA_INPUTS/SIOC_2026-04-20_FROZEN/trace_3.json` nello starter kit; tabelle: [3-fields.json](3-fields.json).

```json
{
  "trace_id": "3",
  "record_key": {"fields": ["3.1", "3.2", "3.35", "3.36"], "uniqueness_scope": "trace", "error_code": "E_DUPLICATE_RECORD_KEY"},
  "joins": [{"id": "trace3_to_trace2", "target_trace": "2", "source_fields": ["3.1", "3.35", "3.36"], "target_fields": ["2.1", "2.49", "2.50"], "required": true, "cardinality": "exactly_one", "error_code": "E_MISSING_PARENT_RECORD"}, {"id": "trace3_to_trace1", "target_trace": "1", "source_fields": ["3.1", "3.35", "3.36"], "target_fields": ["1.1", "1.17", "1.18"], "required": true, "cardinality": "exactly_one", "error_code": "E_MISSING_PARENT_RECORD"}],
  "named_conditions": {"is_admission_evaluation": {"eq": {"ref": {"field": "3.3"}, "value": "1"}}, "is_discharge_evaluation": {"eq": {"ref": {"field": "3.3"}, "value": "2"}}, "is_deceased_episode": {"eq": {"ref": {"field": {"trace": "2", "field": "2.7", "join": "trace3_to_trace2"}}, "value": "8"}}, "is_death_discharge_evaluation": {"and": [{"condition": "is_discharge_evaluation"}, {"condition": "is_deceased_episode"}]}},
  "derived_values": {"age_at_admission_years": {"kind": "age_in_completed_years", "birth_date_ref": {"field": {"trace": "1", "field": "1.9", "join": "trace3_to_trace1"}}, "reference_date_ref": {"field": {"trace": "2", "field": "2.5", "join": "trace3_to_trace2"}}, "date_format": "DDMMYYYY"}},
  "repeated_groups": [],
  "documented_non_executable_rules": [{"id": "POLICY-3.5-COVID-R83", "field": "3.5", "policy": "not_enforced_in_v1", "source_note": "Vedi valore del codice ICPC, assegnato alla patologia prevalente, in scheda SVaMA aggiornata (DGR 96/2025, Allegato A). Per pazienti COVID utilizzare obbligatoriamente nella valutazione all'ingresso il codice ICPC: R83-Altre infezioni del sistema respiratorio", "reason": "COVID status is an internal clinical semantic convention and is not reliably derivable from the fixed-width SIOC flow."}],
  "rules": [{"id": "R-3.1-ADMISSION-YEAR", "when": {"const": true}, "assertions": [{"kind": "substring_equals_derived_year", "field": "3.1", "start": 1, "length": 2, "source_ref": {"field": {"trace": "2", "field": "2.5", "join": "trace3_to_trace2"}}, "source_format": "DDMMYYYY", "year_digits": 2}], "error_code": "E010"}, {"id": "R-3.8-DEATH", "when": {"condition": "is_death_discharge_evaluation"}, "assertions": [{"kind": "raw_equals", "field": "3.8", "value": "99"}], "error_code": "E010"}, {"id": "R-3.8-NORMAL", "when": {"not": {"condition": "is_death_discharge_evaluation"}}, "assertions": [{"kind": "allowed_values", "field": "3.8", "values": ["00", "02", "05", "08", "10"]}], "error_code": "E010"}, {"id": "R-3.9-DEATH", "when": {"condition": "is_death_discharge_evaluation"}, "assertions": [{"kind": "raw_equals", "field": "3.9", "value": "9"}], "error_code": "E010"}, {"id": "R-3.9-NORMAL", "when": {"not": {"condition": "is_death_discharge_evaluation"}}, "assertions": [{"kind": "allowed_values", "field": "3.9", "values": ["0", "1", "2", "4", "5"]}], "error_code": "E010"}, {"id": "R-3.10-DEATH", "when": {"condition": "is_death_discharge_evaluation"}, "assertions": [{"kind": "raw_equals", "field": "3.10", "value": "9"}], "error_code": "E010"}, {"id": "R-3.10-NORMAL", "when": {"not": {"condition": "is_death_discharge_evaluation"}}, "assertions": [{"kind": "allowed_values", "field": "3.10", "values": ["0", "1", "2", "4", "5"]}], "error_code": "E010"}, {"id": "R-3.11-DEATH", "when": {"condition": "is_death_discharge_evaluation"}, "assertions": [{"kind": "raw_equals", "field": "3.11", "value": "99"}], "error_code": "E010"}, {"id": "R-3.11-NORMAL", "when": {"not": {"condition": "is_death_discharge_evaluation"}}, "assertions": [{"kind": "allowed_values", "field": "3.11", "values": ["00", "02", "05", "08", "10"]}], "error_code": "E010"}, {"id": "R-3.12-DEATH", "when": {"condition": "is_death_discharge_evaluation"}, "assertions": [{"kind": "raw_equals", "field": "3.12", "value": "99"}], "error_code": "E010"}, {"id": "R-3.12-NORMAL", "when": {"not": {"condition": "is_death_discharge_evaluation"}}, "assertions": [{"kind": "allowed_values", "field": "3.12", "values": ["00", "02", "05", "08", "10"]}], "error_code": "E010"}, {"id": "R-3.13-DEATH", "when": {"condition": "is_death_discharge_evaluation"}, "assertions": [{"kind": "raw_equals", "field": "3.13", "value": "99"}], "error_code": "E010"}, {"id": "R-3.13-NORMAL", "when": {"not": {"condition": "is_death_discharge_evaluation"}}, "assertions": [{"kind": "allowed_values", "field": "3.13", "values": ["00", "02", "05", "08", "10"]}], "error_code": "E010"}, {"id": "R-3.14-DEATH", "when": {"condition": "is_death_discharge_evaluation"}, "assertions": [{"kind": "raw_equals", "field": "3.14", "value": "99"}], "error_code": "E010"}, {"id": "R-3.14-NORMAL", "when": {"not": {"condition": "is_death_discharge_evaluation"}}, "assertions": [{"kind": "allowed_values", "field": "3.14", "values": ["00", "02", "05", "08", "10"]}], "error_code": "E010"}, {"id": "R-3.15-DEATH", "when": {"condition": "is_death_discharge_evaluation"}, "assertions": [{"kind": "raw_equals", "field": "3.15", "value": "99"}], "error_code": "E010"}, {"id": "R-3.15-NORMAL", "when": {"not": {"condition": "is_death_discharge_evaluation"}}, "assertions": [{"kind": "allowed_values", "field": "3.15", "values": ["00", "03", "07", "12", "15"]}], "error_code": "E010"}, {"id": "R-3.16-DEATH", "when": {"condition": "is_death_discharge_evaluation"}, "assertions": [{"kind": "raw_equals", "field": "3.16", "value": "99"}], "error_code": "E010"}, {"id": "R-3.16-NORMAL", "when": {"not": {"condition": "is_death_discharge_evaluation"}}, "assertions": [{"kind": "allowed_values", "field": "3.16", "values": ["00", "03", "07", "10", "11", "12", "14", "15"]}], "error_code": "E010"}, {"id": "R-3.18-DEATH", "when": {"condition": "is_death_discharge_evaluation"}, "assertions": [{"kind": "raw_equals", "field": "3.18", "value": "99"}], "error_code": "E010"}, {"id": "R-3.18-NORMAL", "when": {"not": {"condition": "is_death_discharge_evaluation"}}, "assertions": [{"kind": "allowed_values", "field": "3.18", "values": ["00", "02", "05", "08", "10"]}], "error_code": "E010"}, {"id": "R-3.20-DEATH", "when": {"condition": "is_death_discharge_evaluation"}, "assertions": [{"kind": "raw_equals", "field": "3.20", "value": "99"}], "error_code": "E010"}, {"id": "R-3.20-NORMAL", "when": {"not": {"condition": "is_death_discharge_evaluation"}}, "assertions": [{"kind": "allowed_values", "field": "3.20", "values": ["00", "05", "10", "15", "20", "25", "30", "35", "40", "45", "50", "55", "60", "65", "70", "75", "80"]}], "error_code": "E010"}, {"id": "R-3.21-DEATH", "when": {"condition": "is_death_discharge_evaluation"}, "assertions": [{"kind": "raw_equals", "field": "3.21", "value": "99"}], "error_code": "E010"}, {"id": "R-3.21-NORMAL", "when": {"not": {"condition": "is_death_discharge_evaluation"}}, "assertions": [{"kind": "allowed_values", "field": "3.21", "values": ["00", "10", "15", "25"]}], "error_code": "E010"}, {"id": "R-3.23-DEATH", "when": {"condition": "is_death_discharge_evaluation"}, "assertions": [{"kind": "raw_equals", "field": "3.23", "value": "9"}], "error_code": "E010"}, {"id": "R-3.23-NORMAL", "when": {"not": {"condition": "is_death_discharge_evaluation"}}, "assertions": [{"kind": "allowed_values", "field": "3.23", "values": ["1", "2", "3"]}], "error_code": "E010"}, {"id": "R-3.19-DEATH", "when": {"condition": "is_death_discharge_evaluation"}, "assertions": [{"kind": "raw_equals", "field": "3.19", "value": "00000000000000000000"}], "error_code": "E010"}, {"id": "R-3.22-DEATH", "when": {"condition": "is_death_discharge_evaluation"}, "assertions": [{"kind": "raw_equals", "field": "3.22", "value": "99"}], "error_code": "E010"}, {"id": "R-3.22-NORMAL", "when": {"not": {"condition": "is_death_discharge_evaluation"}}, "assertions": [{"kind": "numeric_range", "field": "3.22", "min": 0, "max": 98}], "error_code": "E010"}, {"id": "R-3.24-DEATH-TO-00", "when": {"condition": "is_death_discharge_evaluation"}, "assertions": [{"kind": "raw_equals", "field": "3.24", "value": "00"}], "error_code": "E010"}, {"id": "R-3.24-00-TO-DEATH", "when": {"eq": {"ref": {"field": "3.24"}, "value": "00"}}, "assertions": [{"kind": "condition_true", "condition": "is_death_discharge_evaluation"}], "error_code": "E010"}, {"id": "R-3.24-NORMAL-RANGE", "when": {"not": {"condition": "is_death_discharge_evaluation"}}, "assertions": [{"kind": "numeric_range", "field": "3.24", "min": 1, "max": 90}], "error_code": "E010"}, {"id": "R-3.25-ADMISSION", "when": {"condition": "is_admission_evaluation"}, "assertions": [{"kind": "allowed_values", "field": "3.25", "values": ["1", "2", "3"]}], "error_code": "E010"}, {"id": "R-3.25-DISCHARGE", "when": {"condition": "is_discharge_evaluation"}, "assertions": [{"kind": "raw_equals", "field": "3.25", "value": "0"}], "error_code": "E010"}, {"id": "R-3.26-ADMISSION", "when": {"condition": "is_admission_evaluation"}, "assertions": [{"kind": "raw_equals", "field": "3.26", "value": "0"}], "error_code": "E010"}, {"id": "R-3.26-DISCHARGE", "when": {"condition": "is_discharge_evaluation"}, "assertions": [{"kind": "allowed_values", "field": "3.26", "values": ["1", "2", "3", "4"]}], "error_code": "E010"}, {"id": "R-3.26-DEATH-TO-4", "when": {"condition": "is_death_discharge_evaluation"}, "assertions": [{"kind": "raw_equals", "field": "3.26", "value": "4"}], "error_code": "E010"}, {"id": "R-3.26-4-TO-DEATH", "when": {"eq": {"ref": {"field": "3.26"}, "value": "4"}}, "assertions": [{"kind": "condition_true", "condition": "is_death_discharge_evaluation"}], "error_code": "E010"}, {"id": "R-3.27-ADMISSION", "when": {"condition": "is_admission_evaluation"}, "assertions": [{"kind": "raw_equals", "field": "3.27", "value": "00"}], "error_code": "E010"}, {"id": "R-3.27-DISCHARGE", "when": {"condition": "is_discharge_evaluation"}, "assertions": [{"kind": "numeric_range", "field": "3.27", "min": 0, "max": 99}], "error_code": "E010"}, {"id": "R-3.28-ADMISSION", "when": {"condition": "is_admission_evaluation"}, "assertions": [{"kind": "raw_equals", "field": "3.28", "value": "00"}], "error_code": "E010"}, {"id": "R-3.28-DISCHARGE", "when": {"condition": "is_discharge_evaluation"}, "assertions": [{"kind": "numeric_range", "field": "3.28", "min": 0, "max": 99}], "error_code": "E010"}, {"id": "R-3.29-ADMISSION", "when": {"condition": "is_admission_evaluation"}, "assertions": [{"kind": "raw_equals", "field": "3.29", "value": "00"}], "error_code": "E010"}, {"id": "R-3.29-DISCHARGE", "when": {"condition": "is_discharge_evaluation"}, "assertions": [{"kind": "numeric_range", "field": "3.29", "min": 0, "max": 99}], "error_code": "E010"}, {"id": "R-3.30-ADMISSION", "when": {"condition": "is_admission_evaluation"}, "assertions": [{"kind": "raw_equals", "field": "3.30", "value": "00"}], "error_code": "E010"}, {"id": "R-3.30-DISCHARGE", "when": {"condition": "is_discharge_evaluation"}, "assertions": [{"kind": "numeric_range", "field": "3.30", "min": 0, "max": 99}], "error_code": "E010"}, {"id": "R-3.31-ADMISSION", "when": {"condition": "is_admission_evaluation"}, "assertions": [{"kind": "raw_equals", "field": "3.31", "value": "00"}], "error_code": "E010"}, {"id": "R-3.31-DISCHARGE", "when": {"condition": "is_discharge_evaluation"}, "assertions": [{"kind": "numeric_range", "field": "3.31", "min": 0, "max": 99}], "error_code": "E010"}, {"id": "R-3.32-ADMISSION", "when": {"condition": "is_admission_evaluation"}, "assertions": [{"kind": "raw_equals", "field": "3.32", "value": "0"}], "error_code": "E010"}, {"id": "R-3.32-DISCHARGE", "when": {"condition": "is_discharge_evaluation"}, "assertions": [{"kind": "allowed_values", "field": "3.32", "values": ["1", "2", "3", "4"]}], "error_code": "E010"}, {"id": "R-3.33-DEATH", "when": {"condition": "is_death_discharge_evaluation"}, "assertions": [{"kind": "raw_equals", "field": "3.33", "value": "9"}], "error_code": "E010"}, {"id": "R-3.33-NORMAL", "when": {"not": {"condition": "is_death_discharge_evaluation"}}, "assertions": [{"kind": "allowed_values", "field": "3.33", "values": ["1", "2", "3"]}], "error_code": "E010"}, {"id": "R-3.34-AGE", "when": {"const": true}, "assertions": [{"kind": "derived_compare", "derived_value": "age_at_admission_years", "operator": ">=", "value": 19}], "error_code": "E010"}, {"id": "R-3.34-DEATH", "when": {"condition": "is_death_discharge_evaluation"}, "assertions": [{"kind": "raw_equals", "field": "3.34", "value": "999"}], "error_code": "E010"}, {"id": "R-3.34-NORMAL", "when": {"not": {"condition": "is_death_discharge_evaluation"}}, "assertions": [{"kind": "numeric_range", "field": "3.34", "min": 0, "max": 100}], "error_code": "E010"}, {"id": "R-3.34-999-TO-DEATH", "when": {"eq": {"ref": {"field": "3.34"}, "value": "999"}}, "assertions": [{"kind": "condition_true", "condition": "is_death_discharge_evaluation"}], "error_code": "E010"}],
  "conditional_presence": []
}
```

### Trace 4.1 — 1 regole ricevute

Fonte: `SCHEMA_INPUTS/SIOC_2026-04-20_FROZEN/trace_4_1.json` nello starter kit; tabelle: [4.1-fields.json](4.1-fields.json).

```json
{
  "trace_id": "4.1",
  "record_key": {"fields": ["4.1", "4.7", "4.8", "4.19.1"], "uniqueness_scope": "trace", "error_code": "E_DUPLICATE_RECORD_KEY"},
  "joins": [],
  "named_conditions": {},
  "derived_values": {},
  "repeated_groups": [],
  "documented_non_executable_rules": [],
  "rules": [{"id": "R-4.18-LE-4.19", "when": {"const": true}, "assertions": [{"kind": "field_compare", "left": {"field": "4.18"}, "operator": "<=", "right": {"field": "4.19"}, "value_type": "date"}], "error_code": "E_TEMPORAL_ORDER"}],
  "conditional_presence": []
}
```

### Trace 4.2 — 2 regole ricevute

Fonte: `SCHEMA_INPUTS/SIOC_2026-04-20_FROZEN/trace_4_2.json` nello starter kit; tabelle: [4.2-fields.json](4.2-fields.json).

```json
{
  "trace_id": "4.2",
  "record_key": {"fields": ["4.20", "4.22"], "uniqueness_scope": "trace", "error_code": "E_DUPLICATE_RECORD_KEY"},
  "joins": [{"id": "trace42_to_trace41", "target_trace": "4.1", "source_fields": ["4.20", "4.21"], "target_fields": ["4.1", "4.2"], "required": true, "cardinality": "at_least_one", "error_code": "E_MISSING_PARENT_RECORD"}],
  "named_conditions": {},
  "derived_values": {},
  "repeated_groups": [],
  "documented_non_executable_rules": [{"id": "POLICY-4.28-PREVIOUS-ACT", "field": "4.28", "policy": "not_enforced_in_v1", "reason": "Values 2-5 refer to a previous act, but the trace does not expose a predecessor-act identifier. No cross-record relation is invented."}],
  "rules": [{"id": "R-4.30-REQUIRES-4.29", "when": {"is_present": {"ref": {"field": "4.30"}}}, "assertions": [{"kind": "field_present", "field": "4.29"}], "error_code": "E003", "note": "A revoking act necessarily terminates validity; however a cessation date may exist without a revoking act."}, {"id": "R-4.30-REFERENCES-4.22", "when": {"is_present": {"ref": {"field": "4.30"}}}, "assertions": [{"kind": "record_exists", "trace": "4.2", "match": [{"target_field": "4.22", "source_field": "4.30"}]}], "error_code": "E012"}],
  "conditional_presence": []
}
```

## Evidenze

Controllo riproducibile: `python3 docs/verification/check_rules_and_joins.py`.
Report e limiti: [T-P0-05](../../verification/T-P0-05.md). Nessuna run CI
o prova di compilazione del DSL è dedotta dalla verifica documentale.
