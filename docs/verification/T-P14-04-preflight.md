# T-P14-04 — preflight diff tra versioni di schema

2026-10-25. Autorizzazione workflow rinnovata; T-P14-03 DONE (da80163,
CI39574640180 Linux/MSVC success, dossier esistente). ARCH§38/Fase5 e§40,
prompt atomico e Piano§4/14 letti. T-P14-01 manca della fonte XLSX esterna;
questa task non ne dipende. Nessun cambio al piano o alle fonti protette.

API additiva diff_schemas(&RawFlowSchema,&RawFlowSchema)→SchemaDiff/report
serializzabile. Hash dalla canonicalizzazione approvata; binding encoding runtime
escluso come nell'hash. Il diff non compila, valida o determina compatibilità
normativa; può servire a revisionare anche un modello raw con layout invalido.

Confronto solo dello stesso flow_id; trace identificati da ID e field da ID
nello stesso trace. ID duplicati rifiutati per evitare accoppiamenti arbitrari.
Rinomina ID=remove/add, nessuna euristica. Modifiche proprietà ricorsive in ordine
lessicografico, altre liste in ordine posizionale. Le liste trace/field preservano
l'ordine significativo: evento OrderChanged quando cambia l'ordine relativo degli
ID condivisi, senza attribuire spostamenti a una semplice aggiunta/rimozione.
I puntatori JSON separati before/after restano verificabili sulle due versioni.
Metadati/source/regole/policy/capability/versioni non sono omessi.

Nessuna nuova dipendenza, CLI o funzionalità di migrazione. Schema compiler/engine/
audit esistente invariati. Test: add/remove/modify, trace/field reorder, scope ID,
metadata/JSON pointer, numeri esatti, encoding runtime, ambiguità e diff delle
revisioni SIOC ratificate. Gate standard Linux/GNU e CI Linux/MSVC richiesti.
