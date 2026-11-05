# CNF-CAD-NAMESPACE-BINDING-01 — parametro di invio e campo ULSS

5 novembre 2026. **Proposta da ratificare**, nessun nuovo formato o controllo
implementato. Interessa il lowering del profilo in T-CAD-02 e l’eseguibilità
CAD successiva; non blocca registration T-CAD-01, release18-03 o storage generico.

## Evidenze del conflitto

- CAD_RUNTIME_PROFILE.json `submission_scope.validation` nel candidato ricevuto,
  G06; docs/normative/cad/rules-and-joins.md: ogni A.1.1 deve essere uguale a
  sending_ulss, obbligatorio501–509; i child appartengono allo stesso namespace.
  L’evento da solo non è un’identità globale fra organizzazioni.
- Il contratto raw approvato ha RawRef locale/esterno e RawOperand ref/literal.
  RawExpr confronta un riferimento con un valore letterale; nessun nodo legge
  il valore di un parametro. RawPersistence definisce namespace_parameters, ma
  non un binding campo→parametro. Snapshot/namespace separano i run senza
  dimostrare che il campo A.1.1 corrisponda all’organizzazione dichiarata.
- La PK A include1.1 ma ciò non prova l’uguaglianza col parametro della request.
  Un allowed_values501–509 non dimostra uguaglianza a una ULSS specifica.
- ARCH§3/45, pianoN-01/N-14 eDET168 vietano di mettere il nomeCAD o1.1 nel core,
  eliminare il controllo o inventare nuove proprietà nel formato approvato.

La norma è già approvata; manca la rappresentazione generica eseguibile nel
contratto. Non usare versioni di schema una per ULSS, sostituzione di literal
in memoria o riscrittura tacita dello schema dopo averne calcolato il digest.

## A — binding dichiarativo del namespace (proposta raccomandata)

Definire un’estensione esplicita e versionata dell’authoring per associare
campi a parametri di namespace, per esempio nella persistence_policy:

```json
"namespace_bindings": [{
  "trace": "A", "field": "1.1", "parameter": "sending_ulss",
  "comparison": "logical_text_exact",
  "error_code": "E_NAMESPACE_VALUE_MISMATCH",
  "message_template": "Il codice dell’organizzazione non coincide con quello dell’invio."
}]
```

Il testo è una forma proposta, non JSON operativo già supportato. Adottare un
formato nuovo esplicito (3) per non cambiare tacitamente le grammatiche1/2;
capability `persistence.namespace_binding.v1`. Formati1/2 invariati, nuova
proprietà rifiutata in quei formati; nessuna migrazione automatica dei FROZEN.
Registrare la baseline minima reale solo dopo implementazione/gate, senza
attribuire la nuova capability alla prerelease rc.2 in pubblicazione.

Compiler: risolvere trace/campo/parametro esatti, richiedere parametro stringa
obbligatorio del namespace e confronto testo logico byte-esatto nell’encoding
dichiarato; rifiutare riferimenti sconosciuti, duplicati o tipi incompatibili.
Il binding entra nel fingerprint e nei controlli di compatibilità tra versioni.
Il profilo CAD dichiara sending_ulss stringa con dominio501..509; anno/trimestre
restano nel periodo, non nel namespace. SIOC con namespace intero resta invariato.

Runtime generico: parametro assente/incompatibile è errore tecnico di preflight.
Solo dopo la validazione locale del campo, confrontare il valore logico con
quello della request verificata. Disaccordo: ValidationError normativo con codice
configurato nello schema, CompletedInvalid, ordine deterministico, conteggi e
max_errors ordinari; niente preview approvabile/commit. Un campo già invalido
non riceve un errore namespace derivato. Nessun valore sanitario o nuova chiave
in log/storage, nessuna conversione implicita numero/testo o trim aggiuntivo.

Prove: parametro/campo uguali,501vs502 entrambi di dominio valido, namespace
request mancante, campo già invalido, due organizzazioni con stesso evento,
anno successivo, vecchi formati rifiutano binding, SIOC invariato e nessun ramo
per nomi di flusso/campo nel motore. Convalida e fingerprint deterministici.

## B — operando parametro nella DSL generale

In alternativa approvare un RawOperand::Parameter tipizzato e una capability
per confronto campo↔parametro, sempre in un formato nuovo esplicito. Consente
altri confronti oltre al namespace, ma richiede estendere contesto delle regole,
compiler e valutatori. Stessi esiti e test sostanziali; nessuna adozione implicita
dell’opzioneB o sintassi non ratificata.

## Decisione richiesta

Approvare A oppure B, o precisare un altro contratto. Fino alla ratifica non
implementare la nuova grammatica né dichiarare completo il binding normativo
CAD02. La sola prosecuzione generale del workflow non ratifica questa estensione.
