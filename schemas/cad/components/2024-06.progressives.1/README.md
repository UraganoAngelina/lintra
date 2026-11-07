# CAD 2024-06.progressives.1 — T-CAD-04

Componente di accettazione del vincolo progressivo–evento e dei diff sui
progressivi B1/B2/C/D. Conserva le PK normative complete, i campi alle coordinate
originali e il namespace ULSS ratificato in CAD02. I quattro progressivi sono
indipendenti fra tracciati e restano associati all'evento anche dopo annullamento;
lo stesso evento può essere riattivato e gli altri attributi possono cambiare.

Il componente contiene venti campi selezionati e copie dei metadati originali,
con hash di PDF, tracciati e componente namespace. I campi aggiuntivi usati per
provare modifiche sono tipo presa in carico, motivo sospensione, sede accesso e
numero prestazioni. I lookup clinici appartengono al componente CAD11.
Le forme legacy dei vincoli sulle cifre e delle missing policy sono convertite
esplicitamente nel contratto canonico; le copie originali restano intatte.

`ProjectedHistoricalRecord` estrae le tre chiavi dalla stessa riga: PK completa,
progressivo e legame del progressivo all'evento. Conserva solo token protetti.
Gli indici usano key_kind distinti nel protocollo HMAC e nello storage generico
già esistenti. Il numero di token memorizzati è distinto dal conteggio dei record
fisici; l'origine diagnostica resta in memoria e non contiene il record grezzo.

`validated_with_identity_records` richiede proiezioni integre dello stesso
profilo e l'outcome della validazione. La preview verifica chiave dell'archivio,
fonti, versione/hash, generazione e binding storico prima di offrire il commit.
La prima riassegnazione restituisce un `ValidationError` con il codice dichiarato
e il numero di riga fornito dal produttore. Il CAS esistente protegge la base
verificata; consenso, immutabilità delle sessioni e callback dei report rimangono
parte del percorso di approvazione.

Il confronto sul progressivo si seleziona esplicitamente con
`HistoricalAuditService::for_identity_invariants`. Il costruttore ordinario
continua a usare la PK indicata da `cross_run_audit`. Così il cambiamento di data
in C/D può essere un aggiornamento del progressivo, mentre il confronto sulla
PK completa mantiene il proprio significato.

La prima implementazione richiede una sola invariante per tracciato,
`same_schema_version` e riattivazione dello stesso evento esplicitamente ammessa.
I casi ulteriori vengono rifiutati dal compilatore. Una proiezione con chiavi
secondarie duplicate è un candidato strutturalmente ambiguo e viene rifiutata;
nessuna riga viene scelta o scartata per risolverla implicitamente.

Questa consegna qualifica il componente di proiezione/approvazione. I test usano
fixture pubbliche di outcome, esplicitamente etichettate, con compiler, HMAC,
SQLite, diff e riaperture reali. La pipeline clinica/storica CAD completa,
riconciliazione cumulativa e golden integrati restano ai rispettivi gate. Il
componente non viene dichiarato schema CAD completo o FROZEN.
