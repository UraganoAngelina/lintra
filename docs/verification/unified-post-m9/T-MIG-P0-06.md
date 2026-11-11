# T-MIG-P0-06 — Policy dati e threat model

Data11 novembre2026. Stato: **DONE per policy documentata/recepita**.
Prerequisito P0-03 DONE; anche ADR OS002 recepita inP0-05. Ingresso4e935d8
clean; codicecb70bee qualificato, nessun frontend/protocollo creato ora.

ADR-MIG-003 registra i confini approvati: Rust possiede file/indici/validazione/
report/chiave; IPC bounded con autorizzazione custom command effettiva, nessun
bulk o shell/fs generici. Threat model con12 minacce, controlli e prove future
per XSS, path, account, late events, DoS, approval, cache/log, rete, export,
supply chain e recovery. Le firme/limiti/CSP concreti non vengono inventati;
sono output e prove dei gateP1/P2/P7, nessun PASS anticipato per la sola policy.

ADR-MIG-004 recepisce D02/DEC-MIG-09: software interno reale, tutti i test e
screenshot sintetici, nessuna autorizzazione aziendale presunta. G-REAL è
limitato ai dati personali specifici e non blocca sviluppo/GUI/package tecnico.
La vecchia frase su ADR/DEC aperte è superata dal recepimento, non diventa una
nuova domanda su cifratura/retention già escluse dall'autore. HMAC non è cipher;
non si promette secure erase o assenza universale di PHI da supporti/swap.

INTERNAL_USE e DET collegano gli ADR. Nessun accesso a chiavi/dati operativi,
nessun cambiamento di core, CLI, storage o golden. Niente approvazione
normativa, CAD UI o pubblicazione abilitata dalla documentazione.

Controlli: fonti autore/master/DEC-MIG lette, confronto confini correnti,
link locali,12 righe threat, check_unified_plan.py e diff check. Delta solo
documentale, nessun nuovo test runtime/CI necessario. GUI WebView, permissions
custom e attacchi rimangono NOT_VERIFIED. G-MIG-0 attendeP0-07/08.
Next READY P0-07 UX/budget; AUTO_CONTINUE single-agent.
