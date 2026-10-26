# LINTRA — Ripresa operativa

1. Verifica root Git, branch, HEAD, working tree e istruzioni/override applicabili.
2. Leggi AGENTS.md GLOBAL AGENT EXECUTION POLICY e MASTER_AGENT_PROMPT.md correnti.
3. Recupera solo la frontiera di IMPLEMENTATION_STATUS, ultimo handoff e decisioni
   pertinenti; non caricare indiscriminatamente l'intera documentazione storica.
4. Confronta prompt corrente TASK_PROMPTS/<ID>.md, codice e criteri ancora aperti.
5. Non duplicare una task DONE. Riprendi implementazione/gate/handoff mancanti;
   se l'ultima è conclusa, individua e avvia automaticamente la prossima READY.
6. Una CI pendente o DONE_UNVERIFIED non soddisfa un gate richiesto. Puoi passare
   sequenzialmente a una task indipendente registrando lo stato della sospesa.
7. Produci un report breve: repository, task conclusa/corrente, gate, decisioni,
   blocker, next READY e stato della prosecuzione; continua subito quando pronto.

Vietati Superpowers, subagenti e delega. Commit/push/CI già autorizzati. Stop solo
per decisioni reali non ratificate o assenza di task autorizzate e pronte. Errori
implementativi ordinari si risolvono autonomamente. Kit, bootstrap e vecchi
handoff non possono ripristinare stop generici o workflow paralleli storici.
