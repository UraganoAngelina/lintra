# CNF-SIOC-REFERENTIAL-G0-01 — Preflight circoscritto per il golden reale

2026-10-25. Proposta, non ratificata. Nessun file protetto modificato.
Non riapre G0, DEC-SIOC-MINIMAL-01 o la policy opzionale anagrafica→ricovero.
T-P13-07 ha gate locale passato e CI nativa in corso; il problema riguarda la
traduzione operativa G0 necessaria alla successiva T-P13-08, non le prove sintetiche.

## Fonti in conflitto

1. DEC-SIOC-MINIMAL-01, docs/normative/DECISIONS.md, e README del profilo
   schemas/sioc/2026-04-20.structural-minimal.1: le relazioni strutturali sono
   attive, congruenze cliniche/anno/età/temporali rinviate.
2. docs/normative/sioc/rules-and-joins.md: trace2→trace1, trace3→trace2,
   trace3→trace1 e trace42→trace41 richiedono parent; zero riscontri disponibili
   per R-4.30-REFERENCES-4.22 è violazione della disposizione sorgente E012.
3. Il flow_schema.json minimale approvato contiene i join min=1 con policy
   violation, ma nessuna regola che li interroghi nei trace 2/3. Nel trace 4.2
   la regola R-4.30-REFERENCES-4.22 usa record_exists(revoking_act), ma il join
   zero_or_many ha on_missing_record=not_applicable e on_missing_trace=not_applicable.
4. Runtime/compiler attuali eseguono le regole esplicite del piano cross.
   Un join dichiara il binding, la cardinalità e le policy; non viene trasformato
   implicitamente in una nuova regola normativa. RecordExists applica la policy
   della relazione assente, preservando N/A anziché convertirlo in violazione.
   Le prove sintetiche precedenti verificano questa semantica.
5. L'unica regola referenziale esplicita attiva nel minimale è un self-join
   (4.2→4.2), mentre T-P13-08/DET §152 richiedono una regola cross-trace reale.

SCHEMA_AUTHORING distingue gli insiemi ammessi, richiede coerenza min=1/policy e
il rispetto di NotApplicable. Non è stata trovata una ratifica che scelga fra
sintesi implicita di controlli da tutti i join e regole esplicite nel profilo.
Non è autorizzato inventare un golden clinico rinviato per aggirare il problema.

## Riproduzione minima, con soli dati sintetici

Eseguito il binario reale con schema SYNTHETIC, file sorgente ABCDEF e target
vuoto disponibile; stesso compiler/service/FlowValidator/report di 13-07:

| Dichiarazione | Esito osservato |
|---|---|
| Join exactly_one, policy violation, nessuna regola | VALID, complete, total_errors=0, controls=[] |
| Regola record_exists, join zero_or_many, policy not_applicable | VALID, complete, 1 valutazione NotApplicable, total_errors=0 |

I risultati sono coerenti col modello di esecuzione esplicito corrente, ma non
realizzano da soli gli obblighi strutturali dichiarati attivi nel profilo G0.
Nessuna riga sanitaria reale, baseline riscritta o engine hardcoded.

## Soluzione proposta — nuova revisione operativa derivata

Richiesta di autorizzazione documentale specifica, non di nuove regole cliniche.
Conservare integralmente il profilo approvato .1 e la baseline FROZEN. Creare una
nuova revisione operativa SIOC con provenienza/hash e ledger espliciti, che renda
eseguibili gli obblighi già approvati senza introdurre controlli impliciti engine.
Prima dell'esecuzione tale revisione deve mantenere il rifiuto delle capability
ancora non implementate; questa proposta non autorizza a rimuoverle o fingere
persistence.audit. Il golden può consegnare un sottoinsieme operativo tracciato
che comprende la sola relazione strutturale scelta, dichiarando ogni esclusione;
non certifica l'intero profilo SIOC o chiude i rinvii G0.

Aggiungere regole esplicite, when const true, record_exists sul join indicato:

| Trace | ID tecnico proposto | Assertion | Codice già dichiarato |
|---|---|---|---|
| 2 | REL-trace2_to_trace1 | record_exists(trace2_to_trace1) | E_MISSING_PARENT_RECORD |
| 3 | REL-trace3_to_trace2 | record_exists(trace3_to_trace2) | E_MISSING_PARENT_RECORD |
| 3 | REL-trace3_to_trace1 | record_exists(trace3_to_trace1) | E_MISSING_PARENT_RECORD |
| 4.2 | REL-trace42_to_trace41 | record_exists(trace42_to_trace41) | E_MISSING_PARENT_RECORD |

Gli ID sono identificativi tecnici dei controlli già richiesti, non nuovi codici
normativi. Messaggio da diagnostic.message_template del join; fonte/obbligo e
codice legacy restano nel ledger. Nessun first() o collasso dei parent duplicati.

Per revoking_act: mantenere zero_or_many, source 4.30→target 4.22, guard della
regola R-4.30-REFERENCES-4.22 e nessuno scope UDO aggiunto. Correggere la sola
traduzione della ricerca negativa verificata:

```json
{
  "on_missing_record": {
    "policy": "violation",
    "error_code": "E_RELATED_RECORD_MISSING",
    "diagnostic": {
      "code": "RELATED_RECORD_ABSENT",
      "message_template": "Atto revocante indicato non presente tra gli atti disponibili"
    }
  },
  "on_missing_trace": {
    "policy": "unknown",
    "diagnostic": {
      "code": "RELATED_TRACE_UNAVAILABLE",
      "message_template": "Verifica dell'atto revocante inconcludente: universo non disponibile"
    }
  }
}
```

E012 resta il codice legacy di provenienza. Full continua a rifiutare input
indispensabili mancanti in preflight; la policy unknown non lo aggira e non
trasforma un universo non disponibile in una ricerca negativa certa.

La policy di trace1_to_trace2_episode resta esattamente quella ratificata:
anagrafica presente senza ricovero correlato → NotApplicable, testo esplicito,
nessuna violazione normativa inventata. Nessuna nuova condizione su anno/età,
metadata, dataset ufficiali, codici clinici o relazioni storiche.

## Alternativa da non implementare senza decisione

Se l'autore intende che la mera dichiarazione di qualsiasi join sia sempre un
controllo normativo autonomo, occorre definire prima il contratto di lowering:
ID/provenienza del controllo implicito, conteggi senza duplicazioni quando una
regola lo interroga già, scope dei join opzionali, cardinalità massima e codici.
Non introdurre tale comportamento per far passare un golden senza approvazione.

## Attività indipendenti

Il gate nativo di 13-07 può essere chiuso con le evidenze effettive. La porzione
memoria di 13-08 è verificabile senza cambiare contratti normativi; non può però
chiudere 13-08/M6 senza il golden cross-trace reale richiesto. Nessuna task
successiva deve essere dichiarata completata aggirando questo prerequisito.
