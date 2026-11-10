# DEC-SIOC-HIST-02-A01 — Ricopertura cumulativa e PK UDO completa

Data: **2026-11-10, Europe/Rome**. Stato: **USER_APPROVED / REPOSITORY_RATIFIED**.
Fonte: ratifica integrativa dell'autore in questa chat, risposte mirate sui
nuovi invii4.1 e conferma finale «anche4.19.1 fa parte della PK». Nessuna
collisione dell'ID rilevata. Integra DEC-SIOC-HIST-02 senza cancellarla.
Implementazione e qualifica: **NOT_VERIFIED**.

## Decisione vigente

Per SIOC1–3, Indicazioni operative C20 del workbook ricevuto impone la
ricopertura dall'inizio dell'anno fino alla fine della fase. La presenza in
più file mensili non equivale a una nuova acquisizione. Confrontare l'invio
con la baseline cumulativa **approvata, pertinente e qualificata**, separando
namespace organizzativo, tracciato, periodo e identità normativa della PK.
Non sommare tutte le occorrenze fisiche negli snapshot precedenti.

| Classificazione | Condizione | Esito |
| --- | --- | --- |
| CARRY_FORWARD_IDENTICAL | PK pregressa, contenuto canonico identico | Ammesso; zero nuovi record/eventi logici; occorrenza fisica auditata |
| NEW_RECORD | PK dimostrata assente dall'universo cumulativo pertinente | Ammesso subordinatamente agli altri controlli |
| HISTORICAL_KEY_CONTENT_MISMATCH | PK pregressa, contenuto diverso | Violazione/rifiuto; nessuna rettifica implicita |
| MISSING_PREVIOUS_RECORD | PK pregressa omessa quando ricopertura completa obbligatoria | Anomalia di completezza; nessuna cancellazione/annullamento implicito |
| DUPLICATE_WITHIN_CURRENT_SUBMISSION | Stessa PK ripetuta nello stesso tracciato/invio | Errore intra-invio anche nel regime cumulativo |

Baseline mancante, incompleta, incompatibile o non qualificabile: Unknown o
errore tecnico secondo la causa, senza falso NEW_RECORD o falsa identità.
Un archivio appena inizializzato deve provare la propria condizione iniziale;
l'assenza di un file storico non costituisce tale prova. Nessuna sessione non
approvata può diventare baseline. La progressione usa sessioni approvate, non
nomi dei file. C20 delimita la ricopertura all'anno di competenza; questo non
autorizza cancellazioni/reset dello storico, né deduzioni dall'anno codificato
nella PK: H8/H29/H82 conservano anche ID2025 per episodi conclusi nel2026.
Se un confine temporale non è qualificabile conservare Unknown.

Riutilizzare i fingerprint protetti esistenti soltanto dopo aver verificato
la rappresentazione: PK e contenuto sono prove differenti. Rendere espliciti
framing, valori inclusi e compatibilità delle proiezioni. Nessuna riga clinica
persistita in chiaro, nuova chiave o algoritmo introdotti per comodità.

## SIOC4.1 — risposta mirata definitiva

La PK è **(4.1,4.7,4.8,4.19.1)**, come già nel workbook e nei profili legacy.
La proposta intermedia a tre campi è superata dalla conferma finale dell'autore;
non richiede modifica di chiavi, HMAC1 o codec.

L'autore conferma il rifiuto **sia intra-invio sia fra invii diversi**, anche
per ricomparsa identica o attributi aggiornati della stessa UDO. La violazione
è normativa, emessa nella **seconda passata**; il record duplicato è escluso
dall'insieme dei record validi e non acquisito/approvato. Conservare le prove
fisiche dell'input e la diagnostica. L'identità conservata in snapshot e il
replay idempotente della stessa sessione approvata restano distinti da un nuovo
invio. Nessun aggiornamento/annullamento speciale inventato.

La ricopertura1–3 non si estende a4.1,4.2 o CAD. Per4.2 resta la D11 già
ratificata, senza eccezioni cumulative aggiunte da questo addendum.

## Oracle minimo di accettazione

1. Gennaio:1001/1002 validi.
2. Febbraio:1001/1002 identici +1003 valido, nuovi=1.
3. Marzo:1001/1002/1003 identici +1004 valido, nuovi=1.
4. PK pregressa identica: nessun falso errore cross-run.
5. PK pregressa modificata: rifiuto secondo contratto.
6. Duplicato intra-file: errore.
7. Omissione pregressa con ricopertura completa obbligatoria: anomalia.
8. Baseline mancante/incompleta: Unknown o errore tecnico secondo causa.
9. Namespace distinti con PK uguale: isolamento.
10. Tracciati distinti con PK numericamente uguale: isolamento.
11. SQLite reopen e replay idempotente.
12. Più ricoperture storiche senza moltiplicare i nuovi record.
13. Commit/CAS fallito: nessuna modifica della baseline.
14. Ripetizione della stessa sessione approvata: idempotenza contrattuale.
15. Snapshot/fingerprint legacy: compatibilità provata o limite esplicito.
16. Diagnostiche deterministiche Linux/Windows, confrontate su SHA pertinente.

Aggiungere4.1: variazione del solo4.19.1 distingue le PK; uguaglianza di tutti
i quattro campi genera errore in seconda passata, intra-invio e cross-run;
un solo diverso campo non costituisce duplicato. Preservare phase-major,
max_errors, cancellazione, atomicità, CAS e originali golden legacy.

## Integrazione autorizzata

L'autore autorizza l'implementazione completa, senza altra ratifica ordinaria
per le ricoperture identiche. Capability e profilo versionati, core generico,
servizio Rust effettivo comune a CLI e futura React/Tauri2; nessuna norma JS.
La UI distingue quando utile ricoperti, nuovi, mismatch, mancanti e Unknown;
non dichiara conformità globale se la verifica è incompleta. Rispettare
preview/review/approve e snapshot approvati. G-SIOC1 richiede prove effettive.

Fonte locale e citazioni integrali verificate:
[matrice celle](../verification/unified-post-m9/U-NORM-05-source-matrix.json),
workbook SHA256 `6be8ae60a2c236e5a25b731365b42f8e7bca849066d68747abfc78b92ce273e3`.
La scelta4.1 è decisione esplicita dell'autore; C18 da sola non dimostra il
divieto di aggiornamento. Nessuna attribuzione normativa indipendente inventata.
