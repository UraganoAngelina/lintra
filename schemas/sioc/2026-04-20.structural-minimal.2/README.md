# SIOC — revisione operativa referenziale .2

Derivata versionata, autorizzata da **CNF-SIOC-REFERENTIAL-G0-01**, 2026-10-25.
Non è una nuova baseline regionale né una certificazione Full SIOC.
Il profilo `.1` approvato e la baseline FROZEN rimangono integralmente invariati.

`flow_schema.json` è autorevole; trace-*.json sono proiezioni identiche.
`scope-ledger.json` documenta fonti/puntatori/snapshot/hash, tutte le quattro
regole e le policy della revoca, condizioni, diagnostiche, test e capacità residue.
Il ledger precedente `.1/scope-ledger.json` resta la fonte dei rinvii ereditati.

Le sole differenze semantiche sono quelle autorizzate:

| Sorgente | Regola tecnica | Target / join |
|---|---|---|
| 2 | REL-trace2_to_trace1 | 1 / trace2_to_trace1 |
| 3 | REL-trace3_to_trace2 | 2 / trace3_to_trace2 |
| 3 | REL-trace3_to_trace1 | 1 / trace3_to_trace1 |
| 4.2 | REL-trace42_to_trace41 | 4.1 / trace42_to_trace41 |

Le quattro relazioni risultano required=true e senza guard aggiuntive nella
fonte FROZEN approvata: when const true è la traduzione espressamente ratificata.
Le assertion record_exists usano chiavi/cardinalità originali e codice dichiarato
E_MISSING_PARENT_RECORD. Gli ID REL-* identificano controlli, non nuovi codici regionali.

revoking_act mantiene 4.2→4.2, chiave 4.30→4.22, zero_or_many e guard originale.
Ricerca negativa verificata: Violation E_RELATED_RECORD_MISSING, legacy E012
conservato. Universo indisponibile: Unknown. Full non bypassa i required input.
Nessuno scope UDO aggiunto. Join opzionale anagrafica→ricovero e NotApplicable
sono identici al profilo .1. Nessun controllo implicito generato dall'engine.

**Il profilo completo continua a richiedere capability mancanti**, inclusa
persistence.audit.v1: non sono state rimosse né simulate. Non viene presentato
come compilabile o eseguibile nel runtime attuale; il test verifica il rifiuto.
Layout, campi, vincoli, condizioni, gruppi, persistenza e rinvii sono preservati.

Il componente autorizzato per T-P13-08 è distinto:
[references-golden.1](../../../tests/golden/cross-trace/sioc-referential/v1/README.md).
Esegue le relazioni reali con dati sintetici e ledger esplicito delle esclusioni.
Non sostituisce questa revisione, non chiude G0/PERSIST o l'accettazione SIOC completa.

Riproduzione con kit normativo locale in sola lettura:

```sh
python3 docs/verification/generate_sioc_referential_profiles.py
```

La modalità predefinita verifica, non scrive; --write ricrea soltanto i nuovi
artefatti autorizzati. SHA del profilo .1 e del FROZEN sono verificati prima di
ogni derivazione. Il kit non è una dipendenza di build o runtime.
