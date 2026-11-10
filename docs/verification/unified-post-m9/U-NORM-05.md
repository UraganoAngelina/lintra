# U-NORM-05 — Supersessione e casistica normativa mirata

Data: 2026-11-10, Europe/Rome. Prerequisito U-NORM-01 `a5689ae`.
Stato: **BLOCKED sulla classificazione ricopertura/aggiornamento**.
D11/DEC-SIOC-HIST-02 già ratificate; nessuna nuova ratifica globale necessaria.
G-SIOC0/1 NOT_VERIFIED. Non usare il vecchio audit-only come criterio finale.

## Fonte verificata

Workbook SHA6be8ae60... e celle effettive conservate in source-matrix.json.
Foglio Tracciato_SIOC definitivo: H8/H29/H82, ID episodio unico per struttura
e anno; marker PK esatti in U-NORM-01. Foglio Indicazioni operative C20:
tracciati1–3 a ricopertura, attività dall'inizio dell'anno fino alla fase corrente,
con esempi che includono le fasi precedenti. C15–17: invio mensile dopo dimissione.
C18–19: tracciati4.1/4.2 a ogni aggiornamento. H151 distingue nuovo atto,
modifica/integrazione/rinnovo/revoca attraverso nuovi eventi amministrativi.

## Semantica certa e parte da decidere

| Caso | Contratto attuale / esito determinato | Stato |
|---|---|---|
| Due record con stessa PK nella stessa esecuzione | Violazione intra-run già esistente | Conservare |
| Identità già in snapshot conservato ma non in un nuovo input | Non è una nuova trasmissione, mai nuovo duplicato | D11 esplicita |
| PK uguali in namespace diversi | Universi isolati | D11 esplicita |
| Replay dello stesso invio già approvato | Idempotente, non una seconda pubblicazione | Conservare CAS/replay |
| Nuova attribuzione incompatibile di una PK già assegnata nello stesso scope | Enforcement D11 richiesto, nessun ritorno audit-only | Ratificato; criterio per distinguere attribuzione da riproposizione da precisare sotto |
| File fisico mensile con righe già approvate identiche, richieste da C20 | Deve essere distinto da nuovo record; una deduzione solo dalla presenza sarebbe ambigua | Decisione mirata richiesta |
| Stessa PK UDO4.1 con attributi aggiornati, invio C18 | Potenziale aggiornamento della stessa entità, non prova automatica di nuova attribuzione | Decisione mirata richiesta |
| Rettifica/reinvio non definiti dalla fonte | Nessuna eccezione inventata | Restano fuori dal comportamento approvato finché non deciso |

## Proposta circoscritta e alternative

Per la ricopertura1–3, proporre di classificare le righe identiche a una
occorrenza approvata come conservate, con provenance del precedente invio;
enforcement attivo sulle nuove attribuzioni. Una PK con contenuto diverso non
riceve da questa proposta un'eccezione di rettifica. Alternativa: divieto su ogni
ricomparsa nel nuovo file fisico, che rifiuterebbe anche la ricopertura descritta
in C20; richiede una scelta esplicita dell'autore rispetto a quel contrasto.

Per4.1, decidere separatamente se la stessa UDO possa aggiornare attributi
mantenendo la PK o se D11 richieda rifiuto; non dedurre un nuovo codec/chiave,
né un nuovo identificativo normativo non previsto. Qualunque deroga è mirata,
versionata e testata, mai attivata per default.

Il dossier è la base concreta della domanda all'autore. Il codice legacy non
viene cambiato, né un test audit-only riusato per dichiarare enforcement.
U-NORM-03/04 dipendono dalla chiusura di05: non aggirarne il prerequisito con un
DONE fittizio. Continuare le task indipendenti U-UI-01 e U-OPS applicabili.
Nessun test/CI runtime nuovo per questa analisi; source hash/celle verificati.
File: DEC aggiornata con fonte, source matrix, dossier e stato. SHA in Git.
