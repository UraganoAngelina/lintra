# T-MIG-P2-04 — Progresso bounded e terminale

14 novembre2026. **DONE per il gate di design**, G-MIG-2 NOT_VERIFIED.
Prerequisito P2-03 DONE. [Contratto](../../migration/ipc-v1/PROGRESS.md) e
reference integrata con codec/lifecycle: un campione latest, una risposta dati
posseduta, controllo separato; nessun push per diagnostico/record o coda storica.

Terminale nel manager autorevole, leggibile con progresso pieno/scartato e
conservato fino all’ack. RunRef/sequence impediscono regressioni UI e dati del
run sbagliato. Counter trace e PASS2 distinti; niente percentuali inventate.
Un guasto del canale progresso è tecnico e non fabbrica un esito del job.

49scenari PASS/exit0, incluse10.000sostituzioni con consumer lento; più53casi
P2-03 rieseguiti per il validatore RunRef condiviso. I84casi envelope sono
riusati dopo verifica hash degli input. [Receipt](T-MIG-P2-04-checks.json)
registra fonti e output reali. Nessuna nuova CI o VM; il prodotto/Rust/Slint,
golden e report non sono cambiati. Non si dichiara un limite globale già
qualificato sui buffer del framework, né un Job Manager nativo già testato.

Acceptance no per-error push/buffer bounded/terminale osservabile soddisfatta
nel design; implementazione e stress nativi restano ai gate previsti.
**40/128 DONE (31,25%)**. Prossima READY **T-MIG-P2-05**, poiP2-06/P2-07
secondo dipendenze. G-MIG-2 resta aperto.
