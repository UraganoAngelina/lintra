# Piano fixture SIOC — T-P0-08

Stato: **DRAFT_STRUCTURAL_MINIMAL**, 2026-10-19. Il contratto T-P0-07 è
approvato. Il profilo iniziale è ridotto con DEC-SIOC-MINIMAL-01; non sono
ancora prodotti record sintetici o golden. Le sezioni cliniche del catalogo
restano backlog per un futuro profilo completo, non controlli attivi.

Fonti: ARCH §12; DET §107 punti 12–14 e §111; prompt T-P0-08;
[authoring approvato](../SCHEMA_AUTHORING.md),
[decisioni umane](DECISIONS.md), [note](sioc/notes-matrix.md),
[presenza](sioc/presence-matrix.md), [join](sioc/rules-and-joins.md).
Il [catalogo verificabile](sioc/fixtures-catalog.json) copre tutti i 135 campi
e le 72 regole dei cinque trace; hash e indici legano i casi alle fonti.

## Prerequisiti e boundary

T-P0-07 è DONE. Non sono invece disponibili i dodici dataset ufficiali SIOC
con versione/hash; la fonte li dichiara tutti PENDING_DATASET. Nessun binding
viene popolato con hash fittizi, dati sintetici spacciati per ufficiali o
versioni dedotte dalla data del flow. [Schede e lacune](reference-datasets.md).

Il profilo minimale non usa lookup o no_lowercase: non occorre inferire
un repertorio dai file reali. Encoding comunque esplicito a runtime; il 29
febbraio usa **mar01** per il futuro profilo completo. ICD9/età/ordinamenti
sono rinviati, ciascuno tracciato nel ledger. I dodici dataset restano
necessari alla futura conformità semantica completa.

Il [profilo canonico](../../schemas/sioc/2026-04-20.structural-minimal.1/README.md)
contiene flow, cinque trace e ledger. Nel catalogo ogni caso ha
`minimal_profile_disposition`: active, physical_only, partial_050_only,
deferred oppure runtime_encoding_required. Un caso rinviato non genera un
errore atteso nel profilo minimale. I template di constraint usano il vincolo
canonico attivo, non la parte sorgente esplicitamente rinviata.

## Layout previsto

La revisione operativa di bozza proposta è `2026-04-20.structural-minimal.1`,
distinta dalla versione normativa 2026-04-20 e dalla DSL legacy 1.0.0.
Il metadato 2025-12-16 dei trace ricevuti resta provenienza, non viene corretto.
La bozza non è una release, né un’alterazione del FROZEN ricevuto.

```text
schemas/sioc/2026-04-20.structural-minimal.1/
  flow_schema.json
  trace-1.json
  trace-2.json
  trace-3.json
  trace-4.1.json
  trace-4.2.json

tests/golden/sioc/2026-04-20.structural-minimal.1/
  trace-1/{valid,required,datatype,domain}/<scenario>/
  trace-2/{valid,required,datatype,domain}/<scenario>/
  trace-3/{valid,required,datatype,domain}/<scenario>/
  trace-4.1/{valid,required,datatype,domain}/<scenario>/
  trace-4.2/{valid,required,datatype,domain}/<scenario>/
  cross-trace/<scenario>/
```

Ogni scenario da produrre avrà input.txt (o un file per trace coinvolto),
expected.errors.jsonl e expected.summary.json, come DET §111. Includere
provenienza normativa, schema/versione/hash, dataset/versioni e parametri
necessari; orologio e metadati variabili vanno controllati nell’harness.
I fixture file di dati sono esclusivamente sintetici, preservati byte per
byte dagli attributi Git esistenti. Nessuna generazione da export reali.

## Oracolo e criteri degli output attesi

I predicati attesi del catalogo derivano dalla fonte e dalle decisioni umane,
non dall’esito del motore. La fonte JSON originale è citata per hash; non
viene eseguito il validator Python del kit come oracolo normativo Rust.

- Caso positivo completo: tutti i vincoli e i parent richiesti soddisfatti,
  nessuna violazione; eventuali non applicabilità deliberate restano visibili.
- Mutazione singola: esplicitare la condizione di attivazione, mantenere
  gli altri campi/context validi e dichiarare la violazione bersaglio.
  Non inventare il numero totale di errori quando più regole reagiscono.
- RequiredField/InvalidType/InvalidDate/InvalidDomain/OutOfRange/ForbiddenField
  sono classi di ValidationCode (DET §51); la forma serializzata completa
  viene fissata in T-P2-02/T-P5, non dedotta dai codici numerici legacy.
- E_CONDITIONAL_REQUIRED, E_RELATED_RECORD_MISSING e E_DUPLICATE_RECORD_KEY
  sono simboli già espliciti del design; E010/E003/E012 non sono l’API JSONL.
- Un dataset mancante o binding impossibile produce schema/preflight/run
  error, non CompletedInvalid per tutti i record. Un test di dominio non
  può iniziare con il provider mancante e dirsi verificato.
- Campo/context indisponibile→Unknown con motivo, salvo la policy deliberata
  NotApplicable del parent opzionale. Nessuno dei due è Satisfied.

Gli output JSONL e summary esatti saranno prodotti da specifiche del caso e
revisionati prima della regressione; qui sono pianificati, non già esistenti.

## Copertura per campo e regola

Il catalogo contiene per ogni campo trace/id/indice/hash e:

| Categoria | Testimone e risultato atteso |
|---|---|
| required | Campo con base required: raw missing esatto della fonte → RequiredField. NeverMissing: nessun RequiredField basato sul solo raw di width corretto, senza promettere che datatype/domain siano validi |
| required condizionale | Condizione required_if vera e raw missing → ConditionalRequired; forbidden_if vera e dato presente → ForbiddenField; conflitti e Unknown separati |
| datatype numeric | Byte X seguito da zeri fino al width, non missing → InvalidType |
| datatype date | 31022026, otto byte, non missing → InvalidDate; completare positivi di calendario e bisestili; indipendenti dalla futura regola di età |
| datatype AN | Testimone illecito solo dopo profilo lessicale; non dichiarare invalidi accenti o punteggiatura per deduzione |
| domain | Ogni constraint della fonte ha indice/kind e casi positivo/negativo o boundary rappresentabile; lookup richiede dati ufficiali/versionati |

Tutte le 72 regole sono indicizzate con when, assertion kind ed error code
ricevuto come provenienza. Pianificati when true/satisfied, true/violated
se rappresentabile, false e context mancante; non è affermata l’esistenza
di quattro record isolati validi per ogni formula prima del controllo delle
interazioni. Repeated_groups: unicità diagnosi non mancanti; problemi
socio-familiari con sentinel 99; nessuna contiguità o unicità procedure
inventata. Le non esecuzioni v1 deliberate restano metadata/decisioni.

## Scenari trasversali e modifiche approvate

Nel minimale sono attivi VALID (senza dataset), ID-LEGACY e non-tutti-zeri,
JOIN, REVOCATION, CESSATION, SEMANTIC-ZERO per la sola presenza, PK e il
contratto futuro CROSS-RUN. ANON conserva solo i divieti/obblighi di presenza.
ID-2026, sentinelle/nome ANON, DEATH-SCORES, AGE-BOUNDARY, DATES-ORDER e
UNION-LOOKUP sono backlog esplicito del profilo normativo completo.

Scenari aggiuntivi attivi, con contesto sintetico altrimenti valido:

| ID | Mutazione | Output bersaglio da produrre |
|---|---|---|
| WIDTH-1/2/3/41/42 | Lunghezza esatta / un byte in meno / uno in più | Accettazione del layout / errore lunghezza record; terminatore escluso |
| PAD-ID | ID logico sintetico seguito da spazi fino a 20 byte | Nessun errore di presenza/padding; spazi finali preservati nel file |
| ZERO-ID | Da uno a venti zeri con padding coerente | Violazione del regex non-tutti-zeri, anche per ricovero 2025 |
| VENETO-SENDER | Prefisso 050 / 030 in 1.17, 2.49, 3.35 | Constraint segmento soddisfatta / violata; ultime tre cifre senza lookup |
| VENETO-UDO | 4.7 uguale 050 / 030 | AllowedValues soddisfatta / violata |
| OUTSIDE-REGION | 1.13 o 2.29 uguale 030, contesto condizionale coerente | Nessun errore per il solo fatto di essere fuori Veneto |
| MINIMAL-DOMAIN | Codice fisicamente valido ma non verificato da dataset | Nessun lookup eseguito; non significa codice normativamente corretto |
| BITSET | 3.19 posizioni 17–20 tutte zero / una a 1 | Rappresentazione conforme / constraint violata |

I valori 030 sono solo controesempi sintetici per un confronto con 050,
non un dataset di regioni. Gli esiti si riferiscono al controllo bersaglio,
non promettono un conteggio globale di errori già misurato.


| ID | Preparazione sintetica futura | Predicato atteso / fonte |
|---|---|---|
| VALID-1/2/3/41/42 | Un baseline per trace, parent e dataset necessari presenti | Nessuna violazione del profilo, dopo risoluzione dei binding; FROZEN come fonte, non prova già eseguita |
| ANON-98 | 1.3=98, sentinella identificativo di venti A, cognome/nome ANONIMO e divieti delle field presence | Regole R-1.2/R-1.7/R-1.8 e condizioni ricevute; controesempi con sentinel/nome errato |
| ID-2026 | Ingresso 2026, ID logico di otto cifre con prefisso anno e progressivo non nullo | Regole nuove dal 2026: errori mirati su anno/progressivo/width logico senza cambiare il width fisico 20 |
| ID-LEGACY-1/2/3 | Inizio 2025, fine 2026, ID tecnico sintetico LEGACY2025A seguito da nove spazi | Nessun errore per mancato formato due+sei, da decisione autore; charset e padding vanno applicati dal profilo effettivo |
| ID-LEGACY-ZERO | Stesso episodio, logical composto solo da zeri con padding a destra | Vincolo non-tutti-zeri violato; non confondere con progressivo nullo del ramo 2026 |
| ID-LEGACY-MISSING | Stesso episodio, venti spazi | RequiredField, non deroga alla presenza |
| JOIN-1-OPTIONAL | Anagrafica trace 1 presente, ricovero trace 2 assente | NotApplicable ed esplicito «Anagrafica presente; dati del ricovero correlato assenti»; nessun parent error |
| JOIN-REQUIRED-2/31/32 | Trace 2 senza 1; trace 3 senza 2 oppure senza 1, universo disponibile | E_RELATED_RECORD_MISSING secondo le join ricevute; non primo record arbitrario |
| JOIN-AMBIGUOUS | Due parent con stessa chiave in un trace che deve essere univoco | E_DUPLICATE_RECORD_KEY e correlazione ambigua; non un confronto con first() |
| JOIN-42-MULTI | Più UDO 4.1 con stessa chiave a due campi della join ma PK complete diverse | Almeno un parent soddisfa la cardinalità one_or_many, nessuna falsa unicità della join |
| REVOCATION | 4.30 presente e target 4.22 esistente/assente nell’intero trace 4.2 | 4.29 richiesto; ricerca per solo ID, nessun filtro UDO introdotto; assenza target viola record_exists |
| CESSATION | 4.29 presente con 4.30 mancante | Nessuna imposizione del verso inverso; nota e decisione conservate |
| DEATH-SCORES | Valutazione dimissione e parent 2.7=8, sentinel delle scale | R-3.*-DEATH/NORMAL, inclusi versi inversi Karnofsky/esito; non trattare sentinel come assenza fisica |
| AGE-BOUNDARY | Date che distinguono età compiuta da differenza anni e compleanno 29 febbraio | R-3.34-AGE, boundary 18/19; convenzione bisestile mar01 approvata; regola rinviata nel minimale |
| DATES-ORDER | 2.4≤2.5≤2.6 e 4.18≤4.19, uguaglianza inclusa | Regole temporali locali ricevute, confronto fra date valide |
| SEMANTIC-ZERO | Campo con NeverMissing e valore zero valido da dominio del caso | Non emettere required automaticamente; ammissibilità valutata dalla constraint specifica |
| UNION-LOOKUP | 2.31 valido in una sola delle tre tabelle oppure sentinel 999999 | Unione hsp11/ria11/sts11 e valore esplicito; tabelle non inventate né concatenate nel loop |
| PK-1/2/3/41/42 | Doppia PK identica dentro ciascun trace | E_DUPLICATE_RECORD_KEY; non omettere componenti della PK |
| SIOC-CROSS-RUN | PK già vista in invio differente, stesso namespace | Solo audit/confronto, nessun errore normativo automatico (DEC-SIOC-HIST-01); implementazione PERSIST futura |

LF/CRLF e spazi finali vanno preservati nelle fixture; i casi di BOM/riga vuota
seguono i contratti P4 e le decisioni ancora esplicite, senza scegliere
silenziosamente una policy per preparare un golden. Nessuna regola CAD è
anticipata in questo piano SIOC.

## Ricontrolli obbligatori

1. Bozza minimale prodotta: flow e cinque trace, con ledger dei rinvii.
   Il futuro profilo completo richiede dataset ufficiali e repertorio minuscole.
2. Lint manuale rispetto a SCHEMA_AUTHORING, JSON Schema strutturale e
   tracciabilità completa di campi/regole/decisioni.
3. Appena **T-P3-04** esiste, rieseguire il linter automatico Rust sui file
   effettivi: questo obbligo è registrato anche nel catalogo e nello stato.
4. Prima dei golden eseguibili: completare record sintetici, expected JSONL
   e summary, revisionare l’oracolo e poi eseguire su Linux e Windows.

Evidenze e problemi: [report T-P0-08](../verification/T-P0-08.md).
Nessuno schema completo o test normativo già passato è dedotto dal piano.
