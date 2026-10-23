# CNF-REFERENCE-FORMAT-01 — Ratifica aggiornata dell'autore

2026-10-23. Fonte: risposta dell'autore alla domanda sul profilo filesystem.
**APPROVATA CON REVISIONE VINCOLANTE**, sostituisce qualsiasi risposta precedente
alla medesima CNF. La proposta originaria rimane storica, non fonte di precedenza.

## Contratto ratificato e differenze

Membership v1: JSON UTF-8 senza BOM, oggetto chiuso con dataset_format_version=1,
dataset_id/version espliciti e keys_hex lowercase di lunghezza pari. Byte esatti,
zero-byte e dataset vuoto ammessi, nessuna normalizzazione o latest/path dedotto.
Binding tramite ReferenceDependency, root esplicita, digest sui byte originali
prima del parsing, identità/versione controllate. Nessun dataset normativo inventato.

La precedente proposta di errore indiscriminato su hash mismatch è superata:
file conforme e identità affidabile ma digest diverso → **Degraded**. Non è
automaticamente malformato. Si conservano hash atteso e calcolato separatamente.
**Verified** richiede formato, identità e integrità verificati; **Unavailable**
copre file assente/inaccessibile/non interpretabile o identità non affidabile.
Ogni stato deve avere evidenza e ragioni tipizzate; assente non è set vuoto.

ContinueWithAvailableEvidence utilizza evidenze sicure, continua sulle dipendenze
indipendenti, non promuove risultati degradati a conclusioni normative. Strict
può fermare la preparazione se manca una garanzia. Sicurezza dei path è sempre
obbligatoria e non derogabile. Non dichiarare protezioni TOCTOU non realizzate.
JSON ambiguo o parser permissivo non autorizzati: recupero selettivo richiede
strategia circoscritta, testabile e proposta esplicita; non inventarla in 11-03.

## Semantica approvata per integrazioni future

- VerifiedPresent: presenza in almeno una sorgente verificata, anche se altre
  dipendenze dell'unione mancano; l'assenza delle altre resta in audit.
- VerifiedAbsent: assenza solo quando TUTTE le dipendenze richieste sono verificate.
- ProvisionalPresent: presenza solo in sorgenti degradate, nessuna violazione
  normativa, ma esito non definitivo e completezza Partial.
- Inconclusive: nessuna presenza disponibile e almeno una dipendenza assente o
  degradata; warning separato da ValidationError, nessun incremento total_errors.
- Unione logica di lookup preparati, senza copie/Union-Find; attendibilità e
  provenienza preservate. HashMap<DatasetKey,Arc<dyn LookupSet>>, HashSet<Vec<u8>>.
- Completamento esecuzione, esiti normativi e completezza sono dimensioni distinte.
  CompletedValid non significa conformità completa se la completezza è Partial.
- Contatori con unità esplicite per valutazioni definitive/saltate/provvisorie/
  inconcludenti; warning aggregati deterministicamente, nessuna crescita illimitata
  per record. Provenienza rule/flow/trace/field/dataset/versioni/anomalie/policy.
- Summary/audit finali conservano disponibilità, digest effettivi/attesi, versioni,
  policy e completezza. Report parziali regolarmente completati pubblicabili;
  fatalità tecniche conservano pubblicazione atomica. Repository storico non anticipato.
- Futuro Slint «Controlli saltati» distingue non eseguiti/inconcludenti/provvisori;
  contratti applicativi tipizzati, nessuna reinterpretazione nella UI.
- Attributi/sesso/età/temporalità/lookup per data NON aggiunti dalla ratifica.
  Unsupported primitive/schema scorretto non diventano dataset indisponibile.

## Perimetro autorizzato ora e stop

§18 autorizza T-P11-03: formato/provider filesystem reale, hash/identità/versione,
preparazione/HashSet, stati tecnici e anomalie rappresentabili, test e ledger.
Semantica globale membership/unione, completezza/summary/audit, prosecuzione
applicativa/DSL, UI e recupero selettivo possono essere consegne successive.
Esplicitare l'evoluzione delle API pubbliche con impatto/verifiche e preservare
le API esistenti ove possibile. Nessuna modifica a baseline/fonti/grammar.

§19: gate locali + Linux/Windows, stato/handoff/contratti/ledger aggiornati;
**FERMARSI DOPO T-P11-03**, nessuna task successiva senza autorizzazione.

Test di provider/stati/identità/hash/formato/path obbligatori in 11-03;
test end-to-end di summary/DSL/unione e controlli aggregati assegnati nel ledger
alle integrazioni responsabili, non dichiarati eseguiti da soli test del provider.
