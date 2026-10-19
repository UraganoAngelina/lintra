# T-P2-03 — Preflight e CNF-DOMAIN-01

2026-10-19. **BLOCKED prima dell’implementazione dei contratti interessati.**
Prerequisito T-P2-01 verificato nella run 39233324913. Nessuna modifica Rust
T-P2-03 eseguita; non è un blocco normativo SIOC o una riapertura di G0.

## Evidenze del conflitto

1. DET §57 richiede `ValidationSummary.scope_summary: ScopeSummary`, ma
   nessuna definizione di ScopeSummary è presente nelle fonti ricevute.
2. Lo stesso summary richiede SchemaKey, assegnata a T-P3-02 in
   `src/schema/compiled.rs`; P3 richiede P2 a livello di fase. Importarla
   nel dominio come dettaglio compiled inverte la dipendenza di DET §5.
3. DET §61 e T-P2-03 richiedono ValidationRunError con payload tecnici:
   SchemaLoadError, SchemaCompilationError, InputError, ReferenceDataError,
   ReportError, InvalidRequestError, ContextError. I contratti concreti sono
   assegnati a fasi successive; InvalidRequestError è applicativo (T-P7-01).
   Un dominio che dipenda dall’application contraddice esplicitamente DET §5.
4. DET §94 già presenta ValidationRunError fra gli export dell’application,
   mentre il prompt T-P2-03 indica file domain/{identity,summary}.rs.

Non si risolve aggiungendo String/Option o tipi vuoti ai payload, eliminando
campi dal summary, inventando dati normativi o simulando l’implementazione
delle task successive. Nessun documento sorgente modificato autonomamente.

## Risposta dell’autore — 2026-10-19

**«Mantieni il blocco; rivediamo i contratti».** La proposta A seguente è
NON APPROVATA. Non implementare nuovi contratti né applicare il riordino.
T-P2-03 resta BLOCKED; M1 non raggiunta. Documenti ARCH/DET/Piano invariati.
La proposta rimane solo materiale per la revisione, senza forza contrattuale.

## Proposta A non approvata: separazione e consegna per dipendenze reali

- Spostare la proprietà del solo valore neutro `SchemaKey { flow_id: FlowId,
  version: SchemaVersion }` nel dominio; il modulo schema lo riutilizzerà.
  Nessuna struttura compiled o compiler viene anticipata.
- Definire il riepilogo di scope minimo coerente con DET §23:

  ```rust
  pub enum ScopeSummary {
      Trace { trace_id: TraceId },
      Flow { trace_ids: Vec<TraceId> },
  }
  ```

  Solo gli ID nello scope, con ordine preservato. Nessun path, record, dato
  paziente o informazione di storage. Validità/duplicati della richiesta
  rimangono al futuro preflight; nessun obbligo normativo nuovo.
- Consegnare prima la parte autonoma T-P2-03: RecordIdentity, SchemaKey,
  ScopeSummary, RunStatus, RunMetadata, ValidationSummary e outcome neutro,
  includendo schema_hash/truncated e il mapping codici già implementato.
- Collocare ValidationRunError nel confine applicativo, con i payload
  tecnici dei moduli competenti; completarlo quando i relativi contratti
  esistono. Non sostituire nel frattempo i payload con messaggi generici.
- Autorizzare espressamente il riordino limitato: dopo la parte autonoma
  verificata di T-P2-03, proseguire nelle task che hanno prerequisiti atomici
  soddisfatti (iniziando T-P3-01), pur con la consegna dell’errore tecnico
  applicativo ancora aperta. T-P2-03/M1 non vengono dichiarate complete finché
  anche il residuo non è verificato. Ogni task conserva gate e CI previsti.

Questa proposta modifica proprietà dei tipi e ordine di consegna, non la
semantica degli esiti: violazione normativa rimane CompletedInvalid,
Cancelled controllato, guasto tecnico rimane Err senza ValidationError dentro.
Se approvata, registrare la decisione e collegare gli adeguamenti dei contratti
in documentazione; non cambiare retroattivamente le baseline normative.

## Alternativa B

Rinviare T-P2-03 finché l’autore fornisce una diversa definizione completa
ed un ordine compatibile per ScopeSummary, SchemaKey ed errori tecnici.
Le attività dipendenti restano ferme; T-P2-01/02 non sono invalidate.
