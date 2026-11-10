# DEC-SIOC-HIST-02 — Non ripetizione PK SIOC fra nuove trasmissioni

Data: 2026-11-10, Europe/Rome. Stato decisione: **USER_APPROVED /
REPOSITORY_RATIFIED**. Implementazione/collaudo: **NOT_VERIFIED**.
ID controllato nel repository: nessuna collisione prima di U-GOV-03.

Fonte della decisione: messaggio utente «PROMPT ZERO AGENTE — LINTRA POST-M9
UNIFICATO · RATIFICA 10/11/2026», D11 e master unificato §1/§3.
[Atto di recepimento](../unified-post-m9/RATIFICATION_2026-11-10.md).

## Supersessione prospettica

DEC-SIOC-HIST-01 resta la policy storica di sola osservazione per i profili
precedenti. Per **nuove versioni del profilo** è superata da questa decisione:
la non ripetizione delle PK fra invii è vincolo da implementare e collaudare,
con diagnostica normativa e blocco di conformità/approvazione quando violato.
La ratifica non riscrive gli schemi FROZEN, golden legacy o snapshot approvati.

Confrontare PK esatte per tracciato e namespace di una **nuova trasmissione**
con le trasmissioni approvate pertinenti. La stessa occorrenza conservata in
più snapshot non è una nuova trasmissione né una nuova duplicazione. Namespace
separati non confliggono. Non dedurre la trasmissione dalla sola presenza in
un indice cumulativo. Conservare replay idempotente e duplicati intra-run.

History assente, incompleto, incongruente o non autentico non prova unicità:
Unknown o errore tecnico secondo la causa, mai falso Valid/Complete. Nessuna
eccezione per rettifica/reinvio/annullamento viene inventata.

## Fonte normativa e limiti di prova

L'autore dichiara provenienza normativa del requisito: ciò costituisce mandato
vincolante di prodotto, **non verifica indipendente della clausola normativa**.
U-NORM-01/02/05 devono citare la fonte locale disponibile, PK per trace,
periodo/namespace e casistiche certe. Soltanto una vera ambiguità circoscritta
può richiedere decisione mirata; non ripristinare audit-only per default.

Attuazione tramite capability/schema/versionamento e persistence **generici**,
senza `if SIOC` nel core. U-NORM-03/04 qualificano due invii duplicati/unici,
namespace distinti, snapshot conservati, replay, intra-run, annullamenti,
Missing/Absent, history incompatibile, restart/SQLite e report cross-OS.
G-SIOC0/G-SIOC1 non sono chiusi da questo documento. G-SIOC1 include servizio
reale, CLI e frontend con la stessa fonte di validazione; nessuna simulazione UI.
