# T-P17-03 — soglie esplicitamente non definite (DONE)

30 ottobre 2026. Task documentale indipendente, eseguita mentre PERSIST08 attende
la CI41069605562 su bc7c448. Un solo agente; nessuna implementazione parallela.

Prerequisito T-P17-01 verificato dal dossier e da native-ci.json: CI40476599897
completed/success, tutti i job success. I due JSON ottimizzati contengono ciascuno
9 scenari10k/100k/1M × valid-heavy/error-heavy/projection, synthetic=true,
optimized=true, thresholds=null. Hash e verifiche in documentation-check.json.

DEC-14 è aperta: mancano volumi operativi e ambiente target concordati. Non è
pronta una proposta numerica. È invece applicabile l'alternativa esplicita del
piano §10/T-P17-03 e DoD P17: registrare «non definite» con il motivo. Questa
consegna attua tale alternativa, senza chiudere DEC-14 né ratificare soglie.
DET114 e piano §4/14/P17 rispettati; nessuna modifica a fonti architetturali.

File principale: docs/PERFORMANCE.md. Distingue scope e metriche dei benchmark,
riporta collegamenti alle evidenze esistenti, spiega perché non costituiscono
SLA e indica quali aggregati privi di dati personali servono per una decisione
futura. Cache/buffer e invarianti funzionali già approvati restano applicabili.
Il documento non inventa misure della revisione corrente e non sostituisce
profiling, backup, integrazione storica o validazione normativa.

Verifica documentale:10 link locali esistenti, struttura/scope dei due dataset,
thresholds=null e prerequisito CI reale controllati automaticamente. Nessun test
Rust aggiunto o ripetuto per un cambiamento soltanto documentale. Nessuna nuova
CI dichiarata verde e nessun nuovo dispatch che cancelli il gate attivo08.

Stato: DONE nell'alternativa autorizzata;118/140 DONE=84,3%, M9 resta aperta.
Handoff: aggiornati indice/status; riprendere PERSIST08 in WAITING_GATE. PERSIST09
attende08 e non viene anticipata. Materiale utente docs/migration preservato.
