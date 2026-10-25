# CNF-SIOC-REFERENTIAL-G0-01 — Ratifica definitiva

2026-10-25. Fonte: risposta esplicita dell'autore alla proposta
[sioc-referential-g0-contract](../proposals/sioc-referential-g0-contract.md).
Proposta approvata con precisazioni vincolanti. Questa registrazione aggiunge
la decisione alla cronologia; non riscrive G0, DEC-SIOC-MINIMAL-01 o fonti.

## Perimetro autorizzato

Creare una nuova revisione operativa derivata da
schemas/sioc/2026-04-20.structural-minimal.1, versione distinta, provenienza,
hash, diff e ledger. Profilo .1 e baseline FROZEN integralmente invariati.
Non è nuova baseline regionale né certificazione dell'intero SIOC.

Aggiungere regole esplicite già approvate, verificando fonti/condizioni:
- 2 / REL-trace2_to_trace1 / record_exists(trace2_to_trace1).
- 3 / REL-trace3_to_trace2 / record_exists(trace3_to_trace2).
- 3 / REL-trace3_to_trace1 / record_exists(trace3_to_trace1).
- 4.2 / REL-trace42_to_trace41 / record_exists(trace42_to_trace41).
Codice E_MISSING_PARENT_RECORD. when const true soltanto per relazioni realmente
incondizionate; non inferire nuovi obblighi dal solo min=1. Per ogni regola:
fonte, source/target, join, chiavi, cardinalità, applicabilità, entrambe le policy,
codice stabile/legacy, testo e test nel ledger. Non inventare provenienze.

Correggere solo le policy revoking_act:
- on_missing_record violation, E_RELATED_RECORD_MISSING, RELATED_RECORD_ABSENT,
  «Atto revocante indicato non presente tra gli atti disponibili».
- on_missing_trace unknown, RELATED_TRACE_UNAVAILABLE,
  «Verifica dell'atto revocante inconcludente: universo non disponibile».
Restano 4.2→4.2, 4.30→4.22, zero_or_many, guard, scope, nessun nuovo vincolo UDO.
E012 resta provenienza legacy, mapping stabile preservato.

## Vincoli semantici

Join dichiarato non produce controllo implicito: nessun lowering indiscriminato
nel compiler/engine, nessuna duplicazione o nuova obbligatorietà opzionale.
Il controllo è richiesto dalla regola esplicita. Nessun first(), dedup silenzioso,
conversione zero_or_many→exactly_one o cambio chiavi per ottenere un solo match.
Assenza certa e cardinalità multipla sono distinti. Test unico/assente/molti
ammessi/molti non ammessi/target indisponibile senza duplicare la stessa causa.

Anagrafica senza ricovero conserva NotApplicable e testo già approvato;
nessun nuovo controllo inverso obbligatorio. Unknown non diventa False,
Violation o NotApplicable. Full mantiene il rifiuto degli input indispensabili
mancanti; la policy unknown non bypassa preflight. Completezza e audit distinguono
Satisfied, Violated, N/A, Unknown e non eseguito; zero errori non prova Complete.
RunStatus invariato, niente placeholder o nuovi tipi preventivi.

## Capability e golden

Non rimuovere/neutralizzare capability mancanti dal profilo completo, compresa
persistence.audit. Nessun dataset ufficiale inventato o dominio clinico rinviato
attivato. Profilo non compilabile resta rifiutato. Per 13-08 è autorizzato un
sottoinsieme operativo esplicito e tracciabile con esclusioni/provenienza, senza
spacciarlo per Full SIOC o chiudere residui G0. Dati esclusivamente sintetici;
regola reale approvata esercitata con dati sintetici è golden normativo legittimo.

Percorso reale schema→compiler→service/FlowValidator→store/join→cross evaluation
→error/summary/audit. Almeno una relazione cross-trace; self-join 4.2 non basta.
Expected motivati da semantica normativa, non copiati dall'output corrente.
Provare parent presente/assente, N/A opzionale, revocante assente, Unknown/preflight
con universo indisponibile, cardinalità, diagnostiche/ordine/counts/no duplicati.
Regressione: join min=1 senza regola conserva comportamento; nessun automatismo
normativo introdotto nel compiler/engine. Memoria solo proiezioni, senza record
integrali o nuovi crate; porzione indipendente, non chiude da sola 13-08/M6.

## File, gate e arresto

Autorizzati nuovi schemi derivati, ledger/diff/hash, fixture/golden, decisione,
stato e handoff. Vietate modifiche a FROZEN, .1, norme sorgente, ARCH, DET,
Master Plan e regole/policy estranee. Registrare decisione senza riscrivere
cronologia. Normali decisioni implementative nel perimetro non richiedono altre
approvazioni. Nuovi conflitti sostanziali richiedono preflight circoscritto.

13-07 chiude solo con esiti Linux/Windows effettivi, indipendentemente da questa
ratifica; nessun gate storico riscritto. Verificare gate locali/nativi 13-08.
Handoff: CI13-07, nuova revisione/diff/hash/fonti per regola, test/golden/memoria,
capability escluse/mancanti, stato13-08 e condizioniM6.
**Fermarsi al termine di T-P13-08 e attendere nuova autorizzazione** prima di
ulteriori task. Questo mandato specifico supera l'avanzamento automatico ordinario.
