# Prestazioni di Lintra

Stato al 30 ottobre 2026: **soglie prestazionali di prodotto non definite**.
DEC-14 resta aperta. Questa registrazione completa l'alternativa documentale
prevista da T-P17-03 e dalla DoD di P17; non approva valori numerici.

## Motivo e ambito della decisione

Mancano dimensioni reali aggregate dei flussi e un carico operativo rappresentativo
concordato: record e byte per trace/invio, ampiezza delle righe, quantità di
errori, dimensione delle reference, crescita dello storico, hardware e filesystem
supportati. Le fixture sintetiche servono a misurare percorsi riproducibili;
le loro misure non stabiliscono automaticamente un requisito per il prodotto.

DET §114 richiede di definire i target dopo aver conosciuto questi volumi.
Il piano §10/T-P17-03 consente esplicitamente di registrare «non definite» con
il motivo. Pertanto non vengono fissati limiti generali di durata, throughput,
RAM, latenza di lookup o commit, riapertura dello storage o tempo di export.
Nessun percentile osservato viene trasformato in soglia di accettazione.

Restano validi i contratti già ratificati: streaming dei record/report, projection
minima, cache bounded, indici di lookup, integrità degli esiti e scorrimento UI
qualificato. Dimensioni di buffer/cache e vincoli funzionali esistenti conservano
il proprio significato; questa task non li modifica e non li converte in SLA.

## Evidenze disponibili

| Ambito | Evidenza verificabile | Interpretazione |
|---|---|---|
| Validazione e projection | [Harness T-P17-01](verification/T-P17-01-benchmark.md), [run1](verification/T-P17-01-benchmark-run1.json), [run2](verification/T-P17-01-benchmark-run2.json) | 10k/100k/1M record sintetici, valid-heavy/error-heavy/projection, profilo ottimizzato; heap Rust richiesto addizionale e tempi misurati |
| Qualifica del harness | [CI nativa T-P17-01](verification/T-P17-01-native-ci.json) | Linux/MSVC e smoke con conteggi verificati; lo smoke usa un volume ridotto |
| SQLite locale | [Linux](verification/T-PERSIST-05-native-linux-benchmark.json), [Windows](verification/T-PERSIST-05-native-windows-benchmark.json), [accettazione](verification/T-PERSIST-05-acceptance.md) | Adapter reale con 10k/100k/300k identità, lookup indicizzato, commit, riapertura/process restart, picco del processo e contesa |
| Modello risultati UI | [Misure T-P16-05](verification/T-P16-05-local-scrolling.json), [qualifica](verification/T-P16-05-results.md) | Dataset JSONL sintetici grandi, scorrimento/filtri/cache; resta applicabile la successiva qualifica Signature documentata nello stato di progetto |
| Audit storico | [Contratto T-PERSIST-08](verification/T-PERSIST-08-audit.md) | Costi delle API owned e letture puntuali documentati; la qualifica funzionale in corso non costituisce un benchmark esteso dell'audit |

Le misure appartengono alle revisioni, agli ambienti e agli scope riportati nei
rispettivi JSON. Non sono nuove misure della revisione corrente e non promettono
ripetibilità esatta dei tempi su un altro PC. Il conteggio con sink diagnostico
minimo, la generazione di JSONL e l'export XLSX hanno scope differenti.

La memoria del harness di validazione è il massimo addizionale dei byte heap
richiesti a System, con l'overhead di strumentazione documentato. Le misure di
picco del processo SQLite includono componenti C/Rust, fixture e risorse del
processo secondo la metrica OS descritta. Le due metriche vanno riportate con
il proprio nome e non sommate o confrontate come se fossero intercambiabili.

## Riproduzione e interpretazione

Dalla root del repository, scegliere ogni volta un file di output nuovo:

```sh
cargo bench --bench validation --locked -- --output target/performance-validation-new.json
cargo test --bench validation --locked -- --output target/performance-smoke-new.json --smoke
```

Il harness rifiuta un output già esistente, verifica risultati/stati/conteggi ed
espone toolchain, versione, hash dei sorgenti, profilo e scope della misura.
Per lo storage e la UI seguire i comandi dei dossier collegati: il fake in-memory
non misura prestazioni o durabilità SQLite. Un comando fallito, un output
incompleto o un run cancellato non forniscono una misura valida.

## Dati necessari per definire soglie

Prima di proporre numeri occorrono aggregati privi di dati personali:

- record/byte per trace e invio, con volumi tipici e massimi attesi;
- distribuzione della larghezza dei record, reference e chiavi/projection;
- quantità di errori e formati di report richiesti;
- numero di sessioni e identità nello storico, crescita e contesa prevista;
- hardware, RAM disponibile, sistema operativo e filesystem di destinazione;
- operazione misurata, condizioni del test e metrica di memoria/tempo concordate.

Su quella base si ripetono misure rappresentative, si propone un insieme di
soglie con razionale e si acquisisce l'approvazione dell'autore in DEC-14.
Fino ad allora i campi relativi alle soglie restano esplicitamente non definiti.
I gate di correttezza, determinismo e sicurezza rimangono obbligatori.
