# CLI storica e revisione esplicita

T-PERSIST-10,4novembre2026. Il percorso corrente esegue **Full + audit-only** per
schemi che dichiarano le capability supportate. Policy cumulative/temporali CAD,
dataset mancanti e profili regionali completi non vengono attivati implicitamente.
La [ratifica provisioning](verification/PERSIST-KEY-PROVISIONING-01-ratification.md)
e il [contratto portabile](PORTABLE_KEY_BUNDLE.md) descrivono la sorgente della chiave.

## Operazioni e argomenti

`lintra history init --archive PATH --key-id ID --key-file PATH` crea
esplicitamente un archivio nuovo con il provider configurato. Un archivio già
esistente o un file chiave non sicuro sono errori; non viene generato materiale.

`lintra history check --archive PATH --key-id ID --key-file PATH` verifica la
compatibilità dell'archivio esistente e della chiave. Non inizializza, migra,
riconcilia o approva sessioni. La verifica vale anche prima del primo namespace.

`lintra history preview` accetta gli argomenti Full di `validate-flow` e inoltre:

| Argomento | Significato |
|---|---|
| `--archive PATH` | Archivio esistente esplicito; nessun create automatico |
| `--key-id ID`, `--key-file PATH` | Sorgente precisa, senza discovery o chiave nell'argv |
| `--submission-id ID` | Identificativo non sensibile dell'invio |
| `--audit PATH` | Diff storico JSONL streaming, nell'ordine del servizio |
| `--audit-summary PATH` | Envelope versionato con `phase=historical_preview` e summary audit reale |
| `--review` | Abilita una richiesta di revisione dopo l'analisi; non approva |

Restano obbligatori flusso/versione/encoding espliciti, `--trace-input ID=PATH`
e i report normali (`--summary`, almeno uno fra `--errors`/`--xlsx`). I parametri
`--parameter ID=VALUE` sono tipizzati usando lo **stesso** schema compilato del
run: una stringa `001` non diventa un intero per ipotesi del client. I parametri
interi dello storico rispettano il contratto `NamespaceValue::Integer(i64)`;
overflow o tipo errato sono preflight tecnici, senza troncamento.

Nessun default di organizzazione/periodo, modalità Full implicita per altri
comandi, stripping di policy, `max_errors` anticipato o regola inter-run nuova.
Per lo stesso ID il servizio gestisce l'idempotenza secondo il contratto
esistente; l'adapter non forza overwrite o retry di una scrittura ambigua.

## Preview e approvazione

Senza `--review` il comando pubblica report completi di preview e termina senza
commit. Lo stdout JSON dichiara `historical_preview`, copertura soltanto locale,
generazione di base, revisione candidata, fingerprint protetto e conteggi audit;
`new_commit=false` rende esplicita la distinzione. Un secondo evento conferma
la pubblicazione dei report. Lo stderr rimane il canale diagnostico del comando.

Con `--review` gli output rimangono in staging mentre lo stdout presenta il
candidato reale e i suoi conteggi. La conferma richiesta ha questa forma:

```text
APPROVE <revisione> <fingerprint-del-candidato>
```

La stringa esatta viene generata dalla preview, non scelta prima dell'analisi.
Non è una password, un'autenticazione multiutente o un segreto operativo: lega
la decisione umana al candidato presentato. `ABORT`, EOF o cancellazione
interrompono senza commit; un'altra conferma è un errore tecnico. Non vengono
ignorati spazi o normalizzate conferme diverse. Input massimo256byte ASCII.

Chi conferma può leggere anche i report di staging col suffisso `.part`; il
metadata/stdout e il summary audit li identificano come preview. I file finali
vengono pubblicati **nel callback di approvazione**, prima di aprire la
transazione del repository. Se uno fallisce, niente commit. Abort/cancel elimina
gli staging posseduti; un `.part` preesistente estraneo viene preservato.

La pubblicazione resta atomica per file, come nella CLI esistente: non si
promette una transazione unica fra tutti i report e SQLite. Un guasto a una
pubblicazione successiva può lasciare precedenti file completi di preview;
nessun file finale parzialmente scritto o falsa attestazione di commit.
Un conflitto CAS dopo i report resta errore, senza perdita dell'invio concorrente.
Se la cancellazione arriva dopo la pubblicazione dei report e prima del commit,
restano i file completi di preview e lo storico non viene avanzato.

Una precedente approvazione già risolta dal servizio viene indicata con
`already_committed=true`; non è una nuova approvazione o un nuovo commit. In
caso di esito ambiguo il comando effettua una sola riconciliazione in lettura,
emette `commit_outcome_unknown` con la risoluzione effettiva e termina con
errore tecnico. Non ripete la scrittura e non dichiara un rollback inventato.

La cancellazione durante l'attesa è cooperativa con polling50ms. Linux usa
`poll/read`; Windows distingue pipe, file e vera coda input console. Non resta
un thread stdin distaccato. La cancellazione dopo il checkpoint finale di commit
non può annullare una pubblicazione già avvenuta, secondo il contratto esistente.

## Esiti e protezione dei percorsi

| Codice | Esito |
|---|---|
|0| Operazione tecnica/preview completa, oppure approvazione esplicita riuscita; leggere l'evento per distinguere |
|1| Configurazione, chiave, schema, storage, report, conferma errata, CAS, esito ambiguo o evidenze incomplete per l'audit |
|2| Validazione completata con violazioni normative; nessuna approval capability |
|130| Cancellazione, abort o EOF durante revisione; nessun nuovo commit |

Una validazione senza violazioni ma con evidenze incomplete resta tale nel
report; il comando storico termina1 perché non ha completato la preview
richiesta. I report indicano `historical_audit=not_executed`. Non si inventa una
violazione normativa per una copertura mancante.

Chiave, input, archivio, bundle dati, report e nomi `.part` non possono collidere.
Non si può usare la chiave come input di validazione o scrivere report dentro
l'archivio. Gli hard link della chiave sono rifiutati dal provider e i controlli
dei percorsi non si limitano ai nomi passati nell'argv. Nessun segreto è inserito
in errori/Debug/log/report. L'esistenza di una chiave valida non autorizza un
uso clinico reale o una condivisione dello storico fra PC.
