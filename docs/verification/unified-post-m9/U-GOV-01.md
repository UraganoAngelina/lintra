# U-GOV-01 — Baseline post-audit e CI

Data: 2026-11-10, Europe/Rome. Stato: DONE (acceptance documentale verificata).
Gate aggregato G-U0: NOT_VERIFIED fino a U-GOV-04.

## Evidenze

Repository operativo `lintra`, branch `work/p1-foundations`, HEAD iniziale
`7e2ff15523c8b0dd552c435eb1ebf47f80f82251`. Working tree iniziale pulita.
`af3bcccb6246a7ff00463c595f4c780a9f5c5195` è antenato effettivo; il confronto
README/src/ui/manifest/lock/toolchain/build/xtask/scripts/tests/schemas/bundles/CI
è vuoto. Il commit successivo all'audit contiene documentazione/evidenze.

[CI audit riletta](https://github.com/UraganoAngelina/lintra/actions/runs/42789338814):
completed/success sullo SHA af3bccc, sette job success Linux GNU/Windows MSVC,
import reciproci, JSONL e pixel; Qualify UI archives skipped. Non è CI Tauri,
né qualifica automatica di modifiche successive. La prima lettura live è riuscita;
una successiva acquisizione automatica ha fallito la connessione API. Entrambi
sono registrati senza confondere errore di rete e risultato CI.

Comandi, exit, output CI normalizzato e SHA-256 delle fonti nel
[baseline JSON](U-GOV-01-baseline.json). I 39 prompt U-* e gli 89 MIG sono presenti
nel kit dell'autore, inventariati senza modificarli. Il preflight sotto
`docs/migration` è già tracked ed è preservato. Checkout adiacenti non modificati.

## Delta e controlli

Creati solo questo dossier, baseline JSON e `UNIFIED_IMPLEMENTATION_STATUS.md`.
Nessuna modifica ai sorgenti, golden, formati, chiavi o consuntivo M9 140/140.
Test runtime/CI nuovi: non applicabili alla fotografia documentale; non dichiarati.
Commit della task: `git log --format=%H -- docs/verification/unified-post-m9/U-GOV-01.md`.
Nessun blocker della task. Next READY: U-GOV-02; AUTO_CONTINUE.
