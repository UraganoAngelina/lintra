# Codici di validazione — revisione mapping 1

Fonte vincolante: [ARCH §26](ARCHITECTURE.md#26-codici-errore),
[DET §51](DETAILED_DESIGN.md#51-validationcode), DEC-02/CNF-05 risolte.
Questa tabella implementativa trascrive il vocabolario già approvato;
non modifica le fonti architetturali. Unico mapping eseguibile:
`validation_codes!` in `src/domain/errors.rs`.

| Variante | Simbolo pubblico |
|---|---|
| RecordLength | E_RECORD_LENGTH |
| RequiredField | E_REQUIRED_FIELD |
| ConditionalRequired | E_CONDITIONAL_REQUIRED |
| ForbiddenField | E_FORBIDDEN_FIELD |
| Alignment | E_ALIGNMENT |
| InvalidType | E_INVALID_TYPE |
| InvalidDomain | E_INVALID_DOMAIN |
| OutOfRange | E_OUT_OF_RANGE |
| InvalidDate | E_INVALID_DATE |
| CrossField | E_CROSS_FIELD |
| ReferenceData | E_REFERENCE_DATA |
| CrossRecord | E_CROSS_RECORD |
| RuleConflict | E_RULE_CONFLICT |
| RelatedRecordMissing | E_RELATED_RECORD_MISSING |
| DuplicateRecordKey | E_DUPLICATE_RECORD_KEY |

`ValidationCode::as_symbol()` restituisce il simbolo oppure
`InvalidValidationCode`; `FromStr` risolve il simbolo nella stessa tabella.
Nessun numero/discriminante Rust è un codice esterno. I futuri serializer
JSONL/CSV e i riepiloghi devono delegare a questo mapping; qui non viene
implementato un serializer JSON né aggiunta una dipendenza serde anticipata.

`RegulatoryRule(Arc<str>)` trasporta il simbolo specifico del profilo:
la sintassi è `E_[A-Z][A-Z0-9_]*`, come il contratto authoring formato 1.
`as_symbol` rifiuta payload malformati e alias dei quindici codici core;
FromStr riconosce prima i core. E_SCHEMA_ERROR non è una violazione normativa
ed è rifiutato. Il parsing non dimostra che un codice specifico sia stato
registrato: questo controllo spetta al futuro compiler sul profilo effettivo.
Nessun codice clinico CAD/SIOC è inventato o incorporato nel core.

L’ordine Ord/BTreeMap è quello delle varianti di DET §51, nella sequenza
sopra; RegulatoryRule viene dopo, con ordine lessicografico dei simboli.
Non dipende dall’ordine d’inserimento, né da hash randomizzati.
Qualsiasi futura modifica della tabella/ordine richiede revisione e test
espliciti; la posizione di una variante non cambia il simbolo serializzato.
