# T-P15-02 — Preflight e handoff contrattuale

Ripresa successiva2026-10-26: l'autore ha ratificato A, motivando Linux per uso
interno/personale e distribuzione ad altri su Windows. Il blocco sotto è storico
e risolto; implementazione ripresa senza nuovo consenso. [Ratifica](CNF-ARTIFACT-LINUX-01-ratification.md).

26 ottobre2026. Branch work/p1-foundations. HEAD iniziale df03feb, chiusura
documentale15-01; codice verificato7c919fb. Nessuna implementazione precedente
del verifier rilevata; xtask attuale contiene soltanto la ricetta release.

## Prerequisiti verificati

T-P15-01 DONE: CI40307076976 Linux121518727583/MSVC121518727360 entrambi
completed/success, lettura diretta alle10:31:01 UTC. Consultati prompt15-02,
ARCH§39C.1, DET Addendum A.3, piano task/dipendenze e governance corrente.
Tooling deve restare Rust/Cargo e girare sui due runner. Il core non cambia.

## Stato effettivo

BLOCKED prima dell'implementazione per CNF-ARTIFACT-LINUX-01: l'artefatto Linux
GNU locale ha un DT_NEEDED libgcc_s.so.1; «libc di base» non ha allowlist o
eccezione ratificata e DEC-11 Linux resta aperta. Non si allarga il gate per
far passare un artefatto né si seleziona implicitamente musl/staticità/panic.

[Proposta concreta A e alternativa B](../proposals/CNF-ARTIFACT-LINUX-01.md).
Una decisione sul criterio è necessaria prima di fissarlo nel verifier e nei
suoi test. Non è un bug ordinario né una richiesta di permesso Git/CI.

Nessun sorgente del verifier, manifest, lock o workflow modificato. Valutata
documentazione primaria del parser Goblin come possibile componente portabile;
nessun crate selezionato/aggiunto, nessuna API fittizia. Lettura readelf del
binario locale reale eseguita; nessuna ispezione remota simulata. Nessun gate
15-02 eseguito o dichiarato superato; nessuna nuova CI duplicata.

## Handoff

- Task appena conclusa: T-P15-01 DONE,90/140=64,3%; M5 non raggiunta.
- Task corrente: T-P15-02 BLOCKED al preflight, implementazione non avviata.
- Decisioni: GOV-2026-10-26-01 applicata; CNF-ARTIFACT-LINUX-01 proposta aperta.
- File: proposta, questo report, matrice append-only, IMPLEMENTATION_STATUS.
- Gate: CI15-01 verificata; audit governance e git diff --check positivi.
- Residui: intero verifier, test positivi/negativi, wiring e CI dopo ratifica.
- Prossima task: riprendere15-02 subito dopo decisione;15-03 è candidata
  indipendente da15-02, non iniziata durante questo arresto contrattuale.
- Prosecuzione: BLOCKED sulla decisione reale, nessun nuovo consenso ordinario.
- Git: chiusura/proposta da pubblicare, nessuna perdita di lavoro precedente.
