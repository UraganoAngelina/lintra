# DEC-16 — max_errors: decisione richiesta prima di T-P17-04

4 novembre 2026. **A RATIFICATA** dall’autore; fonte completa in
[ratifica](../verification/DEC-16-MAX-ERRORS-01-ratification.md).
Il testo seguente conserva la proposta originaria. Fonte del gate: piano§13.1 DEC-16
rinviata, task17-04 dipendente dalla decisione; DET§80 ammette None per MVP.
Nessun codice del limite implementato e nessuna modifica runtime autorizzata
per semplice inferenza dalla prosecuzione del workflow.

## Stato effettivo

ValidationOptions.max_errors è Option<u64>, default dei client None. Preflight
LocalOnly e Full rifiuta qualunque Some prima di aprire gli input. RunStatus ha
CompletedValid/CompletedInvalid/Cancelled; summary.truncated e completeness
Partial esistono già. SourceSubmission riutilizza RunStatus: aggiungere uno stato
pubblico può coinvolgere i DTO persistenti. La proposta A evita tale estensione.
Non è un limite di errori attualmente attivo o configurabile implicitamente.

## A — limite opzionale, stati e formati esistenti (raccomandata)

- None conserva integralmente il comportamento attuale, nessun cap nascosto.
- Some(N) accetta solo N>0, zero rifiutato in preflight prima degli input.
- Budget globale per la singola validazione, condiviso fra trace e passate:
  conta solo ValidationError normativi realmente emessi con successo al sink.
  Diagnostiche reference, warning/Unknown, progress e guasti tecnici non consumano
  il budget. Ordine trace/riga/campo/stage/regola identico a oggi.
- Dopo l'N-esimo errore interrompere in modo controllato, nessun errore N+1
  emesso o proseguimento su altre righe/trace. Contatori descrivono il prefisso
  realmente elaborato, non si stimano record/errori non osservati.
- Esito CompletedInvalid, truncated=true, completeness=Partial. Non introdurre
  CompletedWithLimitReached o nuovi valori nei formati persistenti. Il risultato
  segnala violazioni osservate ma NON una validazione completa del file.
- Raggiungere N esattamente sull'ultima riga resta troncamento conservativo:
  non si legge altro solo per trasformare il risultato in Complete. Se l'input
  termina prima di N, valgono i normali esiti e truncated=false.
- LocalOnly e Full supportati; se il contesto della prima passata è incompleto,
  non inventare valutazioni cross-record da un universo incompleto. Evidenze
  e hash input mantengono la distinzione completa/prefisso già ratificata.
- Nessun errore I/O/sink/schema/storage è convertito in limite o successo.
  Un guasto effettivamente osservato resta tecnico. Fra cancellazione utente e
  limite conta la prima causa controllata osservata dall'orchestrazione;
  cancellazione resta Cancelled, non viene usata per simulare il limite.
- Validazione storica troncata: nessuna preview approvabile, nessun commit.
  Report possono descrivere esplicitamente il prefisso, senza falsa completezza.
- Scope17-04: API applicativa e propagazione controllata necessaria nel motore,
  con test molti errori, N=1, N maggiore degli errori, confine EOF, più errori per
  record/trace/passate, sink failure e cancel. Default CLI/UI rimane None;
  un selettore GUI o un nuovo flag non sono anticipati da questa decisione.

Non si cambia normativa, HMAC, namespace, schema o formatoSQLite. Nessun ramo per
flusso. La scelta del meccanismo interno è implementativa; non deve usare finti
errori I/O o nascondere la cancellazione per arrestare il motore.

## B — MVP senza limite, rinvio esplicito della feature

Confermare None come unico valore supportato e rifiuto anticipato di ogni Some.
Documentare e testare streaming di molti errori senza troncamento. Chiusura17-04
nel solo perimetro MVP ammesso da DET80, **se esplicitamente approvata dall'autore**;
limiti positivi e stato dedicato rimangono estensione futura, non funzionalità
consegnate. Nessun cap automatico e nessun DONE della semantica positiva.

Se nessuna alternativa è ratificata,17-04 resta BLOCKED/DEC-16. T-PERSIST-12
prosegue sul proprio gate e18-03 è indipendente. Non è richiesto un permesso
ordinario per commit/push/CI: è una decisione contrattuale esplicitamente rinviata.
