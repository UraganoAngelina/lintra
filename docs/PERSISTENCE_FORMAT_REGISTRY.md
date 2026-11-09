# Registro dei formati persistenti

Aggiornamento attuativo 8 novembre2026, ADR-HISTORY-EXISTENCE-001 sostitutiva.
Le versioni fisiche SQLite sono distinte da schema normative, HMAC e DTO pubbliche.

| Versione | Stato e fonte |
|---|---|
| SQLite1 | Legacy qualificato T-PERSIST-05; fixture immutabile storage1, codec record/source originale |
| SQLite2 | Prima versione corrente osservata al commit a644145, constFORMAT=2; consenso omissioni, upgrade1→2 esplicito T-PERSIST-06/07 |
| SQLite3 | In implementazione CAD12: batch fact-only e manifest snapshot protetti, collegati alla stessa transazione dello snapshot; nessuna migrazione automatica1/2→3 |
| HistoryFactBatch1 | Nuovo payload tecnico per piano/snapshot, versione logica interna aSQLite3, qualifica ancora aperta |
| HMAC1 | Invariato: stesse primitive/prove di identità, record/proiezione e source; nessun nuovo algoritmo/purpose/chiave |

Il numero3 deriva dal registro/codice effettivo1/2 e dalla ricerca di versioni già
assegnate, non dalla versione dello schema clinico. Gli archivi1/2 rimangono leggibili
nei loro modi precedenti; writer di fatti deve rifiutarli prima di una mutazione.
L'operazione esistente upgrade_storage_v1_to_v2 continua a significare solo1→2.
Non esiste qui un'autorizzazione a migrare archivi approvati o rigenerare fatti.
Nessuna versione diventa qualificata per sola presenza in questa tabella.
