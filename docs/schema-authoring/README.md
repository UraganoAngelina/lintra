# Artefatti di authoring — proposta formato 1

Stato: **REVIEW_REQUIRED**, T-P0-07. Il contratto è in
[SCHEMA_AUTHORING.md](../SCHEMA_AUTHORING.md); l’approvazione della deroga
sui derivati non equivale all’approvazione di questi artefatti.

| Artefatto | Scopo |
|---|---|
| [flow.schema.json](flow.schema.json) | Grammatica JSON Schema 2020-12 chiusa del nuovo formato |
| [manifest.schema.json](manifest.schema.json) | Manifest del bundle distinto dai manifest legacy |
| [lowering.json](lowering.json) | Mapping dei costrutti ricevuti e piani compiled da implementare |
| [local-flow.json](examples/schemas/local-flow.json) | Flow completo sintetico: tipi, constraint, espressioni, assertion e derivati |
| [history-flow.json](examples/schemas/history-flow.json) | Flow completo sintetico: insiemi, consenso, storico e piani temporali |
| [compatible-history-flow.json](examples/schemas/compatible-history-flow.json) | Compatibilità cross-version esplicita su un gruppo dichiarato |
| [MANIFEST.json](examples/MANIFEST.json) | Hash reali dei quattro file dell’esempio di bundle |
| [example-codes.json](examples/reference-data/example-codes.json) | Dataset solo sintetico per mostrare binding/hash/metadati |

Il dataset dimostrativo non definisce il formato ufficiale di tutti i provider
reference data. Le chiavi tecniche e i codici sono inventati per la verifica
del contratto; non sono regole SIOC/CAD o fixture normative approvate.
Nessun record di paziente, CF reale o credenziale. I flow sono esempi di
configurazione strutturale, non risultati di una validazione di record.

Verifica riproducibile (tool Python temporaneo, nessuna dipendenza Rust):

```sh
uv run --cache-dir /tmp/lintra-authoring-uv --no-project --with jsonschema==4.26.0 python docs/verification/check_authoring.py
```

Dopo il primo download si può aggiungere `--offline`. Cache e ambiente
vivono in `/tmp`; nessun pacchetto viene aggiunto al progetto o alla CI.
[Documentazione primaria del validatore](https://python-jsonschema.readthedocs.io/en/stable/validate/)
**[EXT]**: check_schema valida i meta-schemi; validate/is_valid controllano
le istanze. Questa verifica non compila la DSL, non verifica record o esiti
normativi e non sostituisce il linter/SchemaCompiler Rust futuro.
