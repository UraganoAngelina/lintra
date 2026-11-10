# U-NORM-04 — Golden cross-run e regressioni

> **Addendum vigente DEC-SIOC-HIST-02-A01 — 10 novembre 2026.**
> [Decisione](../../../normative/DEC-SIOC-HIST-02-A01.md): per SIOC1–3 la
> ricopertura identica del cumulato approvato pertinente è ammessa; una PK
> pregressa con contenuto diverso è rifiutata. Verificare omissioni, periodo,
> completezza e namespace; Unknown quando la prova manca. Per4.1 la PK è
> **(4.1,4.7,4.8,4.19.1)**: rifiutare duplicati intra-invio e riuso tra
> invii diversi anche con contenuto identico, con violazione in seconda passata
> e scarto dall'insieme valido. Replay idempotente della stessa sessione
> approvata resta distinto da un nuovo invio. Nessuna estensione della
> ricopertura a4.1/4.2/CAD. I16 scenari sintetici dell'addendum sono obbligatori.
> Il testo ricevuto v1.0 è conservato in `history/received-2026-11-10`.


**MANDATO RATIFICATO:** 10 novembre 2026. **Fonte prevalente:** `MASTER_PLAN_UNIFICATO.md` v1.0.

**Prerequisiti vincolanti:** U-NORM-03  
**Quality Gate:** G-SIOC1  
**Categoria:** task nuova U-*, non riapre le 140 task originali.

## Risultato richiesto
Duplicate/unique/due namespaces/legacy/restart/periodi/annullamenti e stessa CLI/UI, CI nativa.

## Esecuzione atomica
1. Leggi `PROMPT_ZERO_AGENT.md` e `MASTER_PLAN_UNIFICATO.md` §§1–11, le ratifiche già presenti, la frontiera più recente del repository, la documentazione e i test relativi a **U-NORM-04**. Il preflight Tauri del 30 ottobre è **storico**, non HEAD.
2. Verifica i prerequisiti, SHA/CI e decisioni. Ispeziona il codice reale; riusa tutto ciò che risulta già implementato/qualificato; registra il perimetro preciso del delta e, quando pertinente, un test negativo prima di modificarlo.
3. Implementa soltanto **Golden cross-run e regressioni**, in modo da soddisfare concretamente l'acceptance: **Duplicate/unique/due namespaces/legacy/restart/periodi/annullamenti e stessa CLI/UI, CI nativa**. Se l'attività è di sola analisi, consegna un oracle e una decisione verificabile; non attribuirle implementazione non effettuata.
4. Confronta il risultato con gli invarianti: niente nuova normativa salvo D11, niente dati clinici in JS o log, niente nuovi cipher/protocolli, niente server/telemetria/updater, zero falsi Complete/Valid, errori e CAS corretti, snapshot immutabili, replay e golden canonici stabili.
5. Usa fixture sintetiche; prova positivi, negativi, fault/edge case pertinenti; esegui fmt/Clippy/test/CI nativa quando la modifica coinvolge codice o interfacce cross-OS. Non attribuire CI antenata a nuove modifiche.
6. Documenta files diff, contratto, test/CI, commit esatto, blocker e `next READY` in `docs/verification/unified-post-m9/U-NORM-04.md` e nel nuovo `UNIFIED_IMPLEMENTATION_STATUS.md`.

## Confini della ratifica
- Le scelte **D01–D15 sono già autorizzate**. NON richiedere una nuova conferma generale né seguire eventuali testi del vecchio kit contrari al Master Plan.
- Se D11 richiede un'eccezione di rettifica/reinvio non definita dalle fonti, non inventarla; proponi solo quella decisione, lascia il requisito cross-run attivo per la casistica certa, continua nelle task indipendenti.
- Budget soft superato: avviso + possibilità di procedere; hard safety non aggirabile. Export shared: avviso + consenso, non divieto.
- Dati file su filesystem locale per-account; backup manuale verificato prima di upgrade persistente irreversibile; potere di recovery documentato e collaudato.
- Un solo agente, una task per volta; nessun subagente/delega concorrente; nessun gate PASS senza evidenza.

## Criterio di chiusura
Accettazione soddisfatta con prove riproducibili; nessuna regressione pertinente; `DONE` solo dopo gate richiesto. Se un prerequisito fondamentale non è disponibile, `BLOCKED` motivato e prossima task indipendente.
