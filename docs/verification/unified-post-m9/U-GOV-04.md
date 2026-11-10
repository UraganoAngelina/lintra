# U-GOV-04 — Grafo, stato separato e regole dei gate

Data: 2026-11-10, Europe/Rome. Prerequisito U-GOV-03 `2cd4bdf`.
Stato: DONE dopo controlli documentali. **G-U0 PASS documentale**: baseline,
CI audit verificata, riconciliazione, ratifica e grafo; nessun gate runtime PASS.

## Implementazione

DEPENDENCY_GRAPH.json mantiene 39 U-* e 89 T-MIG con tutti gli archi originari
dal master §9/10. L'arco d'ingresso U-UI-01 → MIG-P0-01 rende esplicita la
sequenza del mandato, senza sostituire prerequisiti. DEPENDENCY_MAP.md descrive
il flusso e distingue MM9/cutover da consegna G-REL. Nessun ciclo CAD→MIG:
le prove CAD/SIOC finali convergono in G-REL dopo i gate separati.

UNIFIED_IMPLEMENTATION_STATUS.md contiene tutti gli ID, prerequisiti, stati,
prompt/dossier, SHA noti, verifiche e gate. Nessun dossier vuoto per task futura.
Il consuntivo storico 140/140 e il suo SHA-256 restano identici alla baseline.

scripts/check_unified_plan.py visita ogni nodo mantenendo due insiemi: visiting
rileva cicli sul cammino attivo, visited evita visite duplicate. Confronta gli
archi col master e l'unico arco integrativo approvato, controlla prompt e digest
originali (invertendo solo la correzione testuale delle 89 copie MIG), richiede
dossier e prerequisiti DONE per gli stati DONE. Verifica la congiunzione G-REL.
Il risultato è un controllo di governance, non attestazione tecnica del prodotto.

## Controlli

`python3 scripts/check_unified_plan.py`: PASS, 128 nodi aciclici, 140 fonti,
4 task GOV con dossier. `git diff --check`: PASS. Il checker non introduce
dipendenze ed è indipendente da OS/toolchain UI. Runtime/CI nativa nuovi non
richiesti per questo delta, nessuna qualifica CI ereditata su nuovi sorgenti.
File modificati: grafo, mappa, script, stato unificato e questo dossier.
SHA task: cronologia del dossier. Nessun blocker G-U0.
Next READY U-FIX-01, AUTO_CONTINUE; MIG resta soggetta a G-MIG-0..9.
