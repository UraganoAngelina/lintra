# Budget e protocollo prestazionale della UI

T-MIG-P0-07. Il budget è verificabile tramite limiti strutturali e misure
riproducibili. Non attribuisce al progetto una DEC-14 o tempi/FPS non misurati.
D09/D13 e DEC-17 finale mantengono produzione sequenziale e una validazione
attiva per istanza. L'override soft non modifica correctness, EOF/hash o cancel.

| Oggetto | Budget/criterio verificabile | Misura ed evidenza |
| --- | --- | --- |
| Esito e report | Stessi input/schema/opzioni/parametri → stesse evidenze e JSONL canonico rispetto al servizio Rust | Test a clock controllato, senza normalizzazioni che nascondano divergenze |
| Job attivi | Uno per istanza, secondo start rifiutato/coerente | Test state machine e race, terminale reale dopo cancel |
| Input e diagnostici | Streaming Rust, nessun intero input/JSONL/indice in JS | Instrumentation IPC e heap/DOM, sentinelle sintetiche |
| Dati nel renderer | Righe di pagina/viewport+overscan e dettaglio bounded, non proporzionali al totale N | Conteggio oggetti/righe e bytes per messaggio, cap dal contrattoP2 |
| Indice backend | Offset Rust O(N) dichiarati; nel modello corrente payload8×N byte e cache128 righe | Profilare payload indice, cache e memoria processo separatamente; nessuna pretesa RSS8×N |
| Progresso e terminale | Queue bounded; progresso può essere aggregato, terminale affidabile e correlato | Slow consumer/eventi tardivi/cancel/restart; caps e policy wire inP2 |
| Soft preflight | Input/reference oltre il budget configurato → warning e scelta | Config versionata ed evidenza dati; conferma legata alla richiesta/file/schema |
| Override soft | Nessun taglio record/hash/report per rientrare nella soglia | Confronto diretto con Rust, invariati max_errors/cancel/Partial/tecnici |
| Hard bounds | Overflow, integrità e safety non aggirabili | Errore esplicito; nessun pulsante di override né successo parziale mascherato |
| Reattività | Event loop mantiene input/cancel/query e feedback reali mentre Rust lavora | Replay dichiarato, latenze p50/p95/max e intervalli frame misurati per OS |
| Memoria | Renderer bounded rispetto alle finestre richieste; risorse chiuse secondo ownership | Peak/per-process Rust e WebView, stato prima/dopo/cancel; non sommare erroneamente allocator traffic e RSS |
| Durata analisi | Nessun parallelismo o ottimizzazione automatica da un microbenchmark | Misure per fase/local/cross/I-O/reference/storia, colli di bottiglia dimostrati |

I limiti numerici di pagina, payload e queue vengono dal contratto Rust
versionato P2 e dalle prove; non da default arbitrari del renderer. Le soglie
soft sono valori configurati/giustificati, non un tetto occulto ai record.
Se manca una stima attendibile di durata, la UI non la inventa. Un algoritmo
di presentazione bounded non promette un RSS costante del browser o del processo
Rust, che include indice e contesto dichiarati.

## Classi di workload

- Correttezza: fixture sintetiche dei golden, errori locali/cross, Unknown,
  dataset degradato/assente, Partial, cancel e replay. Il volume piccolo non
  sostituisce un carico operativo.
- Misure decisionali: distribuzioni e volumi realistici motivati da aggregati
  autorizzati e profili CAD/SIOC effettivi, raccolti in U-PERF-01. Nessuna lettura
  o copia di record sanitari reali nei test. I volumi non ancora confermati
  restano NOT_MEASURED; non vengono dichiarati realistici per intuizione.
- Stress:300k/1M diagnostici per indice, paging, scroll e memoria della nuova
  WebView. Sono stress e non giustificano parallelismo o soglie cliniche.
- Le misure sintetiche10k/100k già presenti in T-P17 sono riferimenti tecnici
  del backend/profilo usato, non tempi garantiti per Full regionale o per la UI.
  Heap traffic/peak requested heap non equivalgono a RSS/working set.

## Protocollo riproducibile

Per ciascun scenario annotare SHA software/schema, profilo/perimetro, hash
fixture, N record/errori per trace, parametri, reference/cache/storia, opzioni,
modalità e stato terminale. Ambiente: OS/build, CPU/RAM/storage, runtime WebView,
build debug/release, viewport/DPI/tema e condizioni cold/warm. Usare stesso
protocollo/input per confrontare vecchio adapter, chiamata diretta e nuovo bridge.

Misurare separatamente avvio, preflight/hash, passate locali/cross, costruzione
indice, prima pagina, query/dominio/dettaglio, scroll, risposta a cancel e
chiusura risorse. Registrare numero di campioni e distribuzione; non ricavare
un p95 da un unico tempo di test. Dichiarare ripetizioni/variabilità e controlli
sugli esiti: una misura rapida con record omessi non è una misura valida.

Una scheda incompleta, solo headless quando serve runtime reale, solo Linux
per comportamento Windows o solo un microbenchmark non rende PASS il gate.
Misure e anomalie servono a decidere priorità sui volumi realistici. Il rapporto
finale deve mostrare numeri, motivazione e limiti; nessun valore ms/FPS o
cutoff hardware universale viene inventato inP0. Budget strutturali e correttezza
sono obbligatori; accettazione UX/performance effettiva inP7/U-PERF-02.
