# Registro dei formati persistenti

Aggiornamento qualificato 10 novembre2026, ADR-HISTORY-EXISTENCE-001 sostitutiva.
Le versioni fisiche SQLite sono distinte da schema normative, HMAC e DTO pubbliche.

| Versione | Stato e fonte |
|---|---|
| SQLite1 | Legacy qualificato T-PERSIST-05; fixture immutabile storage1, codec record/source originale |
| SQLite2 | Prima versione corrente osservata al commit a644145, constFORMAT=2; consenso omissioni, upgrade1→2 esplicito T-PERSIST-06/07 |
| SQLite3 | Corrente per archivi nuovi: batch fact-only e manifest snapshot protetti, nella stessa transazione dello snapshot; nessuna migrazione automatica1/2→3 |
| HistoryFactBatch1 | Payload tecnico per piano/snapshot, versione logica interna aSQLite3; reader qualificato, byte precedenti preservati |
| HistoryFactBatch2 | Writer corrente; correlazione nullable soltanto se non qualificata; reader1/2, null rifiutato nel formato1 |
| HMAC1 | Invariato: stesse primitive/prove di identità, record/proiezione e source; nessun nuovo algoritmo/purpose/chiave |

Il numero3 deriva dal registro/codice effettivo1/2 e dalla ricerca di versioni già
assegnate, non dalla versione dello schema clinico. Gli archivi1/2 rimangono leggibili
nei loro modi precedenti; writer di fatti deve rifiutarli prima di una mutazione.
L'operazione esistente upgrade_storage_v1_to_v2 continua a significare solo1→2.
Non esiste qui un'autorizzazione a migrare archivi approvati o rigenerare fatti.
Nessuna versione diventa qualificata per sola presenza in questa tabella.

Qualifica storage: CI42592330836 su c5266d0. Qualifica runtime completa:
[CI42641969728](https://github.com/UraganoAngelina/lintra/actions/runs/42641969728)
su d45078e, sette job richiesti positivi. Entrambi gli import nativi verificano
4snapshot/2namespace, manifest e batch logici1/2 con nullable2. Prove e risultati:
[dossier CAD12](verification/T-CAD-12-runtime-native-ci.json).
SQLite3/HMAC1 invariati dall'integrazione; nessuna migrazione implicita.
