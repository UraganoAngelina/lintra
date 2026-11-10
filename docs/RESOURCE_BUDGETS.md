# Budget operativi soft

D09 separa avvisi operativi e limiti hard di correttezza/safety. I budget
operativi non cambiano regole, conteggi, max_errors, EOF, SHA o cancellazione.

`validate` e `validate-flow` accettano `--resource-budget PATH`, un file JSON
versionato con questa struttura:

```json
{
  "version": 1,
  "measurement_source": "Percorso del dossier di misura e ambiente qualificato",
  "input_bytes": null,
  "reference_bytes": null
}
```

Sostituire `null` con budget interi positivi motivati dalle proprie misure.
Il progetto non imposta soglie di prodotto arbitrarie: questa struttura senza
valori numerici non è una promessa di capienza o uno SLA. Una fonte vuota,
una versione diversa, soglie zero e proprietà sconosciute vengono rifiutate.
I limiti operativi vanno conservati insieme alle misure che li motivano; la
presenza del solo campo measurement_source non certifica quelle misure.

Il preflight conta tutti i file selezionati e le reference richieste dal vero
schema compilato. Legge in streaming con buffer limitato e acquisisce digest
e identità OS; le reference mancanti hanno un contatore distinto e non sono
dataset verificati vuoti. Non esegue la validazione normativa al loro posto.

Se un budget è superato, la CLI mostra l'avviso e termina prima di aprire gli
output, conservando i report precedenti. Per scegliere **Procedi comunque**,
aggiungere `--proceed-over-budget` a quella richiesta. L'esecuzione registra
solo contatori aggregati e la scelta su stderr; non scrive path, digest o dati
di record nel log del preflight. Senza un file di budget esplicito il
comportamento CLI precedente resta compatibile.

Prima della validazione vengono ricontrollati richiesta, versione esatta dello
schema e file. Una sostituzione/modifica invalida la conferma e richiede un
nuovo preflight. L'override non evita I/O failure, overflow, file non regolari,
confini reference o cancellazione. Il validator esegue comunque il proprio
audit completo e non riutilizza il digest del preflight come prova di EOF.

L'API Rust mantiene la conferma in un oggetto con campi privati. La futura UI
deve conservarlo nel backend e presentare Annulla / Procedi comunque con un
handle opaco, secondo MIG-P5-05. Non esiste ancora un dialogo Tauri qualificato.
La CLI `history preview` conserva il contratto audit-only corrente e non espone
questa nuova opzione; l'integrazione del percorso snapshot resta nel bridge CAD.

Il controllo non è una garanzia contro modifiche ostili concorrenti fra l'ultimo
ricontrollo e l'apertura del validator: la tree locale deve restare controllata
durante il run, come D10. Il preflight aggiunge letture complete dei file quando
abilitato; i suoi tempi vanno misurati separatamente dalla validazione.
