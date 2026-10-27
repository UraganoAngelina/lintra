# T-P2-03 — completamento del residuo applicativo

## Prerequisiti e decisioni

CNF-DOMAIN-01 proposta A ratificata il2026-10-19, registro decisioni §CNF-DOMAIN.
Parte autonoma source70cdb5e/CI39235805213 positiva; T-P2-01/02 native già
verificate. P7/P11/P13 ora hanno payload reali, regressioni root qualificate
anche in CI40510757138/ceb90b7. Non si riapre il riordino o la ratifica.

## Verifica del residuo

ValidationRunError esiste in application/service.rs; payload reali SchemaLoad,
SchemaCompilation, Canonical, InvalidRequest, Input, Record, Trace, Summary,
Report, ReferencePreparation e Flow. Il tipo ContextError è ora reale in
context/store.rs, ma mancava la variante applicativa esplicita DET61.
Gli errori normativi rimangono eventi ValidationError/sink e CompletedInvalid,
mai payload di un Err tecnico. RunMetadata/summary/outcome/hash/truncated della
parte autonoma restano implementati e verificati, senza placeholder aggiuntivi.

## Modifiche e correttezza

Aggiunta Context(ContextError), Display tecnico stabile e source sulla causa
reale, più From<ContextError>. Le conversioni esistenti vengono completate:
TraceValidationError::Collection recupera esclusivamente un box contenente
ContextError con downcast, mantenendo ogni altro box nel precedente Trace.
FlowTrace delega alla stessa conversione; FlowInputs(Store) e
Cross(Context(Store)) espongono la medesima categoria canonica. Errori di
binding RelatedContext non-store conservano la categoria Flow e la causa.
Nessuna nuova norma, nuova interpretazione Unknown/NotApplicable o modifica
all’engine. Le conversioni canoniche conservano il payload originale del guasto;
i wrapper di esecuzione sono tolti soltanto per la categoria tecnica riconosciuta,
come avviene già per Input/Report/Summary. Non si convertono cause in String.
L’enum pubblico era già non_exhaustive, API additiva con match interno completo.

## Test

tests/run_context_errors.rs verifica le cinque provenienze Context (diretto,
Trace collection, Full trace collection, Full input Store, Cross context Store)
con payload e downcast della source identici, includendo trace/posizione reali
sintetiche. Il negativo controlla BrokenPipe eterogeneo conservato nella chain e
binding namespace non reinterpretato come errore store. Doctest compile_fail
impedisce conversione diretta ValidationError→ValidationRunError. Test esistenti
service/Full/reference/robustness coprono gli altri guasti e successo normativo.
Due nuovi test mirati pass; fmt e regressione completa in verifica.

## Handoff

IN_PROGRESS: nessun DONE2-03/M1 prima di gate locali e CI del nuovo commit.
File service.rs, test integrazione, dossier/status/evidence. Nessun crate,
backend, algoritmo privacy o PHI. UI16-01 rimane indipendente in attesa
CI40513168784; next16-02 solo dopo quel gate, PERSIST-02 dopo2-03 e01 reali.
AUTO_CONTINUE, nessuna conferma ordinaria richiesta.

## Gate locali completati

Fmt/Clippy workspace default/all-features positivi;701/709test,83suite,0fail/2ignored,
11doctest inclusi. Guard reale core/CLI senza Slint positivo. Nessuna modifica
a fixture/golden, normativi e report invariati. CI nuova da avviare; non DONE/M1.
