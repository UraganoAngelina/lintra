# Progresso bounded e terminale affidabile

T-MIG-P2-04,14 novembre2026. Design dipendente dal [lifecycle](LIFECYCLE.md)
e dal [wire ratificato](CONTRACT.md); nessun handler Tauri è già implementato.
[Reference integrata](progress_reference.py), [49scenari](verify_progress.py).

## Flusso e limiti applicativi

Il worker non invia un evento IPC per record o diagnostico. Scrive il report
canonico in Rust e aggiorna un solo campione di progresso corrente. Le letture
sono su richiesta: `get_run_progress` restituisce un DTO fisso del run richiesto.
Nessuna coda di campioni arretrati, lista di errori, JSONL o indice è in JS.
Il producer non serializza JSON per ogni record e non attende il consumatore.

Budget applicativo esplicito:1campione latest + al massimo1risposta dati
posseduta in lavorazione per contesto;1posto di controllo separato per stato,
cancel e ack. Niente coda di richieste dati; un secondo accesso pendente è
IPC_BUSY. Il lease della risposta è privato Rust, non un token wire o C1;
conclusione/abbandono del lavoro lo rilascia e un vecchio completamento non
può liberare il posto di una richiesta nuova. Il campione restituito è una
copia immutabile: nuovi aggiornamenti non lo modificano sotto al lettore.

Il lease delimita gli oggetti posseduti dall’adapter. Non dimostra che i buffer
interni di Tauri/WebView o un renderer ostile abbiano un tetto globale di
memoria: non si assume un’API Tauri di conferma consegna non verificata. Intake,
lifetime reale delle risposte, disconnessioni e saturazione vanno qualificati
inP2-08/P3/P4/P7. Anche una risposta tecnica rimane entro1MiB; nessun messaggio
sovradimensionato viene trasmesso in parte. Non è un nuovo quality gate VM/CI.

## DTO minimo e semantica dei contatori

```json
{"schemaVersion":1,"run":{"schemaVersion":1,"instance_id":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","run_id":"1"},"sequence":"3","phase":"local_validation","trace_ref":"trace_a","trace_records":"10","cross_records":"0"}
```

Esattamente questi campi. RunRef e schemaVersion vengono validati con gli
stessi controlli diP2-03; i contatori e sequence sono u64 in testo decimale
canonico, con zero ammesso e nessuna perdita Number JS. Overflow non fa wrap.
`trace_ref` è opaco1..64ASCII, risolto Rust verso il TraceId catturato nel piano.
Non si invia un nome file o una label schema arbitrariamente grande nel frame.

Phase corrisponde ai reali ValidationPhase: preflight, local_validation,
cross_record_validation, finalizing_report; null è ammesso prima del primo
campione. La fase non regredisce all’interno del run. Non è una prova di esito.
`trace_records` è cumulativo per il trace nominato, non totale Full. Un nuovo
TraceStarted cambia identità e riparte da0. PASS2 ha `trace_ref = null`,
trace_records0 e il proprio cross_records cumulativo. TraceCompleted aggiorna
l’ultimo count reale; nessuna percentuale, totale o ETA è inferita da byte,
conteggi parziali o trace già conclusi. Le fasi possono non produrre un campione
visualizzato: la UI riconcilia sempre con l’esito finale autorevole.

Il producer consuma i ProgressEvent reali, non diagnostici normativi. Un evento
vecchio non modifica il run corrente né il suo flag errore. Dopo RunCompleted
servizio/terminale i nuovi campioni vengono ignorati. Contatore regressivo,
trace incoerente, tipo impossibile o overflow sigillano il solo canale progresso
con errore tecnico, senza propagare una falsa violazione normativa al core.
La lettura di stato rimane indipendente: un errore RPC di progresso non è
l’esito del job e non sostituisce il risultato reale della validazione.

## Terminale e consumatore lento

get_run_state usa il posto di controllo e legge lo stato posseduto dal manager:
il terminale non viene inserito nel buffer del progresso, coalesciato o eliminato
quando la UI è lenta. Rimane osservabile dopo ripetute letture fino all’ack;
un nuovo start è rifiutato prima dell’ack secondoP2-03. Il client legge lo stato
anche quando non arrivano nuovi campioni, dopo errore/risposta persa o reload.
Non dipende dall’arrivo di una notifica progress per scoprire un terminale.

Il client verifica DTO, RunRef e sequence; ignora un run diverso e sequence
ripetute o inferiori. Una volta osservato il terminale di quel run, nessun
campione tardivo riapre Running o sostituisce l’outcome. L’ack terminale non è
approval clinica/storica. Il canale resta distinto dal successo/fallimento di
flush, pubblicazione e join: RunCompleted da solo non è il terminale adapter.

Con UI scollegata il worker continua senza accumulare notifiche. Su chiusura
esplicita il supervisor richiede cancel e attende il lavoro posseduto prima
del cleanup. Non si promette recupero in memoria dopo kill del processo:
il nuovo instance_id invalida gli handle e i protocolli persistenti conservano
le proprie regole di recovery. Nessun file/store viene nascosto da un PASS UI.

## Evidenze di design

49scenari PASS/exit0, integrati conP2-02/03:10.000aggiornamenti con lettura
ferma, snapshot non mutato, singolo posto dati, lease vecchio, trace multipli
e PASS2, u64 esatto, DTO sotto1KiB anche al massimo dei contatori, terminale
leggibile a posto dati pieno, restart/run tardivo, reducer monotono e terminale
sticky, campi bulk vietati, errori/categorie e overflow. Le combinazioni di
schema/versione/RunRef sono verificate prima di applicare il campione.

L’estrazione del validatore RunRef condiviso ha richiesto la regressione mirata
P2-03:53casi ancora PASS. I84vettori di envelope restano validi sugli input
immutati; nessuna suite core o CI ripetuta. Sono prove della reference, non
misure di throughput, test con veri thread o qualificazione nativa Tauri.
