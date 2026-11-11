# G-MIG-0 — PASS per governance e ingresso agli spike

Data11 novembre2026. **PASS / APPROVED per l'avvio della faseP1**, non per
runtime/installer/cutover. Verifica eseguita dall'agente sulle condizioni del
mandato autorizzato; decision owner delle scelte: autore Lintra, ratifica10
novembre recepita in U-GOV-03 `2cd4bdf`. Nessuna firma o nuova risposta umana
inventata. L'autore ha già autorizzato la prosecuzione alle task READY.

| Condizione | Evidenza verificata |
| --- | --- |
| Baseline post-audit | cb70bee19f2d470c3ac2e98828343b32e23b3f0a; af3bccc antenato |
| CI corretta per il codice | CI42990775714 success,7 job richiesti; log/digest completi in U-NORM-03-native-42990775714.* |
| Regressione locale |1096pass/7ignored espliciti,fmt/Clippy e24 Python PASS;520 input qualificati immutati |
| Nessun WIP08 riaperto | PERSIST08/CAD12/P18-04 nel consuntivo140/140 byte-identico |
| N-24 e separazione | ADR-MIG-001, ARCH§39D/DET e matrice N-24 aggiornati; Slint conservato |
| OS/eccezioni UI-only | ADR-MIG-002; N-19/N-20/N-23 core/CLI preservati |
| Scope LocalOnly+Full | P0-04 e matrice16 capacità; CAD/SIOC con i rispettivi gate |
| Sicurezza e dati | ADR-MIG-003/004,12 minacce/prove, nessun nuovo crypto/retention/gate enterprise |
| UX e risorse | P0-07, mondo approvato e criteri per OS; limiti strutturali/protocollo misure, nessuna DEC-14 inventata |
| Coerenza del piano |128 nodi/39U+89MIG, archi e140 fonti preservati, dossier e prerequisiti verificati |

Non resta un conflitto decisionale della fase0. DEC-MIG-04 wire concreto,
versioni/lock/licenze, screenshot/performance e policy effettive WebView sono
output dei gate successivi; il loro stato pendente non è un PASS anticipato.
Il registro MIG corrente distingue chiusura policy da prova tecnica.

G-MIG-1 richiede realmente CachyOS/WebKitGTK e Win11 clean-room/NSIS offline,
non la CIWindows2025 o una check-build. Il preflight ha già rilevato
webkit2gtk-4.1 mancante sul PC: da soddisfare nello spikeP1-01. Win11/installer/
WebView2/due account non sono stati provati daP0. I residui U-OPS-03/U-NORM-04
rimangono espliciti; non bloccano lo sviluppo UI sintetico e devono essere
chiusi prima dei gate di distribuzione pertinenti.

L'uscita permette **T-MIG-P1-01**, poi le altre task con i propri prerequisiti.
Nessun cambiamento di core/CLI o norma, rimozione Slint, tag/release/pubblicazione
è autorizzato da questo solo gate. G-MIG-1..9/G-CADUI/G-SIOC1/G-OPS/G-DUR/G-REL
restano NOT_VERIFIED. Prove macchina in T-MIG-P0-08-checks.json.
