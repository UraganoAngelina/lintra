## Mandato successivo — 2026-10-23

L'autore autorizza nuovamente task successive automatiche e chiamate GitHub.
Lo stop dopo 11-03 sotto riportato è storico e superato. T-P11-04 DONE, CI cumulativa 39936894049 Linux/Windows success:
policy tecnica Exact/AsciiUppercase, nessuna proprietà authoring aggiunta.

# Ledger — Evidenze reference data, CNF-REFERENCE-FORMAT-01 aggiornata

**T-P11-03 DONE:** CI 39930225299 su 1b40c0e, workflow/job Linux e Windows
completed/success, 471 test locali. Le righe future NON IMPLEMENTATE rimangono
aperte. Stop §19 rispettato alla chiusura originaria, poi revocato dall’autore: task
11-04 DONE e 11-05 implementata con gate Windows ancora non verificato.

2026-10-23. Autorità: ratifica aggiornata dell'autore, §§1–19, registrata in
[CNF-REFERENCE-FORMAT-01-ratification.md](CNF-REFERENCE-FORMAT-01-ratification.md).
Assegnazioni dai prompt/Piano esistenti, senza nuovi task ID o modifica delle
fonti. Questo ledger esplicita i requisiti aggiuntivi approvati, non dichiara
già implementate le loro integrazioni. STOP dopo T-P11-03.

| Capacità | Responsabile / prerequisiti | Stato e accettazione |
|---|---|---|
| Formato membership v1/hash originali/identità/versione | T-P11-03; 11-01 + 0-06 | Implementata; 23 payload sintetici con digest Python indipendenti, gate nel dossier 11-03 |
| Provider/preparazione Strict o Continue, stati e anomalie tipizzate | T-P11-03 | Implementata; snapshot con dipendenze assenti e degradate, mai set vuoto sostitutivo, load legacy solo Strict |
| Osservazioni di membership con provenienza | T-P11-03 | Implementate come misura opzionale + evidenza read-only. NON un risultato normativo di regola o completezza del run |
| Normalizzazione esplicita | T-P11-04; 11-03 | Implementata API tecnica Exact/AsciiUppercase; CI 39936894049 Linux/Windows success. Query trasformata solo su scelta esplicita, dataset/evidenze invariati. Authoring attuale resta esatto; nuova proprietà JSON richiederebbe estensione approvata |
| VerifiedPresent/VerifiedAbsent/ProvisionalPresent/Inconclusive | T-P11-05; 11-04 + 9-05 | Implementati nel percorso engine esplicito, gate nativo pendente; nessun errore normativo da sole informazioni degradate o negative incomplete |
| Union lookup con provenienza | T-P11-05; stessi prerequisiti | Implementata, gate nativo pendente. Composizione di lookup preparati senza copie/Union-Find: un match Verified è sufficiente; match solo Degraded provvisorio; negativo definitivo soltanto con tutte le dipendenze Verified |
| Constraint ReferenceData compilata/eseguita | T-P11-05 | Implementata con dichiarazioni/indici verificati e ultimo stage; percorso legacy senza preparazione rifiutato, percorso reference esplicito con evidenze. Test presenza/assenza/ref non dichiarata e source set parziale, CI pendente |
| Propagazione DSL dei risultati reference non conclusivi | T-P11-05 locale, T-P13-04/05 per estensioni esterne; 11-05 e prerequisiti P13 | Locale implementata nel percorso ReferenceRecordValidator/rule: Unknown reference non diventa False o successo definitivo; controlli indipendenti continuano e Unknown tecnico resta fatale. CI 39939546620 Linux success, Windows ultima lettura in_progress. Service 11-06 e DSL esterna P13 future |
| Policy applicativa ContinueWithAvailableEvidence | T-P11-06; 11-05 + 7-04 | NON IMPLEMENTATA nel service. Collegare esplicitamente provider/preparazione prima degli input, conservare Strict selezionabile e sicurezza fatale. La parte «dataset mancante = Err» del prompt storico 11-06 è deliberatamente superata dalla nuova ratifica per Continue |
| Completezza Complete/Partial e stato di esecuzione distinto | T-P11-06 | NON IMPLEMENTATA. Proporre l'estensione minima prima di modificare RunStatus/Summary/Metadata. No sostituzione anticipata del contratto: esecuzione completata con zero violazioni e Partial non significa piena conformità |
| Contatori e warning aggregati | T-P11-05 produce esiti, T-P11-06 aggrega/persistenza artefatti | NON IMPLEMENTATI. Mai ValidationError per warning, mai incremento total_errors/errors_by_code per Provisional/Inconclusive. Partizione di valutazioni documentata sotto, niente emissione illimitata per record |
| Audit/summary finali delle evidenze | T-P11-06; stessi prerequisiti | NON IMPLEMENTATI. Copiare i veri expected/actual digest, versioni richieste/dichiarate, disponibilità/policy/anomalie; non serializzare direttamente cause OS o path assoluti. Nessuna dipendenza inversa domain → adapter |
| Pubblicazione atomica di run parziali regolarmente completate | T-P11-06; report atomico 5-06 | NON IMPLEMENTATA per reference. Completezza esplicita nei report definitivi; fatalità tecniche/cancellazione restano distinte, audit input EOF invariato |
| Adattamento consumer console/export ai risultati parziali | T-P11-06 + task P12 pertinenti; contratti 11-06 verificati | NON IMPLEMENTATO. Aggiornare DTO/consumer dopo proposta compatibile; non aggiungere warning al JSONL canonico degli errori fingendo che siano ValidationError |
| UI Slint «Controlli saltati» e attendibilità | T-P16-04/05; relativi prerequisiti + contratti 11-06 | NON IMPLEMENTATA. Distinguere non eseguiti/inconcludenti/provvisori, filtri/sezioni, assente/degradato, necessità di ripetere; usare contratti applicativi tipizzati, nessuna reinterpretazione nella UI |
| Recupero selettivo di documenti malformati | Prima proposta separata circoscritta e ratifica; poi task assegnata esplicitamente | NON AUTORIZZATO come algoritmo in 11-03. Oggi il documento ambiguo/non conforme è Unavailable, nessuna voce recuperata o parser permissivo |
| Attributi/sesso/età/tempo/lookup per data | Consegne future da specificare prima di accettare il relativo AST; non incluse nella ratifica | NON IMPLEMENTATI. Metadata binding non vuoto è errore di configurazione, non dataset assente da tollerare; nessun appiattimento a membership |

## Unità dei conteggi e contratti da proporre in T-P11-06

Una valutazione è un tentativo di esecuzione di un controllo applicabile su
un record (oppure su un contesto cross-record quando sarà supportato), con
identità di controllo compilata e scope espliciti. Ripetere la stessa regola
su un milione di record conta un milione di valutazioni, non un milione di
regole distinte. Numero di regole distinte è una metrica separata, bounded
dal piano compilato, non il nome ambiguo di un contatore di tentativi.

Prima di cambiare API/summary, formalizzare una sola categoria finale per
valutazione: definitiva soddisfatta, definitiva violata, non eseguita,
provvisoria, inconcludente. Riutilizzare skipped_rules per i controlli non
eseguiti senza aggiungere un duplicato; registrare esplicitamente il rapporto
con il suo uso storico per Unknown terminale. Non riscrivere i risultati
storici o cambiare oggi la semantica del builder. Un conteggio di ValidationError
emessi è distinto dal conteggio di valutazioni e non va sommato a esso.

Aggregare warning per chiave stabile di controllo/flow/trace/field, dipendenze
richieste, stati/anomalie e policy; ordine deterministico, conteggi checked,
nessuna chiave per valore paziente che faccia crescere la mappa con ogni record.
Un eventuale campionamento/indice di dettaglio per consultazione richiederà un
contratto esplicito: non promettere oggi una lista illimitata di esempi in RAM.
Summary/audit devono rendere visibili conteggi e attendibilità anche a sessione
terminata. L'attuale DatasetEvidence contiene causa e identità della dipendenza;
rule/field/flow/trace e contatori appartengono alle integrazioni 11-05/06.

## Prove future obbligatorie, non già eseguite

T-P11-05: matrice Union §§6/16 completa (tutti Verified, Verified+Unavailable,
Verified+Degraded, match solo degradato, tutti assenti), nessun falso
VerifiedAbsent; DSL e violazioni reali indipendenti continuano correttamente.
T-P11-06: end-to-end input→service→report con assenti/degradati, warning distinti,
conteggi disgiunti, completezza Partial, politica/audit effettivi, no falso
CompletedValid=Complete; Cancelled/fatalità e pubblicazione atomica preservati.
Prestazioni: nessun I/O nel loop e nessuna copia ripetuta delle unioni; verificare
anche aggregazione bounded su molti record. Queste prove non vengono sostituite
dai test provider 11-03 che consultano i singoli snapshot.

Nessuna task successiva è avviata da questo ledger. Il prossimo avvio richiede
l'autorizzazione esplicita imposta dall'ultimo mandato dell'autore.
