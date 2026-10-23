# Lintra — stato di avanzamento al checkpoint del 2026-10-23

**52,9% del piano completo verificato: 74 task DONE su 140.**
Il core locale e la CLI sono operativi nel perimetro ratificato; il prodotto
completo con Full, persistenza, CAD, UI e release non è ancora consegnato.
Workflow sospeso su richiesta dell'autore per questo allineamento.

## Come è calcolata la percentuale

Fonte del denominatore: ATOMIC_TASKS_INDEX.md dello starter kit 20261016 e
Master Implementation Plan §20: 116 task P0–P18, 12 PERSIST core e 12 CAD.
Conteggio unico per ID, basato esclusivamente sulla tabella `## Task` dello
stato corrente, non sui nomi dei file, sui commit o sulle tabelle storiche.
Snapshot di tutti gli ID/stati: PROJECT_PROGRESS_2026-10-23.json.

| Stato effettivo | Task | Incidenza sul piano |
| --- | ---: | ---: |
| DONE, nello scope ratificato e con evidenze | 74 | 52,9% |
| Codice e gate locali completati, CI nativa non verificata | 2 | 1,4% |
| Parzialmente consegnata con residuo esplicito | 1 | 0,7% |
| Non iniziate | 63 | 45,0% |
| Totale | 140 | 100% |

Se si includono le due task locali in attesa di gate, l'implementato arriva a
76/140 = 54,3%; **non è la percentuale verificata**. Non attribuisco una frazione
arbitraria alla task parziale. Ogni task pesa uno: questa misura non indica
percentuale di ore, costo o tempo rimasto. Full/persistenza/UI hanno un peso
tecnico rilevante anche se occupano relativamente pochi ID. Le task DONE con
scope ridotto approvato contano nello scope ratificato, non come capacità future.

## Avanzamento per area

| Area | Verificate / totale | Stato e residuo |
| --- | ---: | --- |
| P0 — contratti/formalizzazione iniziale | 9/9 | G0 APPROVED_WITH_DEFERRALS; non certifica il profilo normativo completo |
| P1 — repository e CI | 3/3 | Linux GNU e Windows MSVC reali |
| P2 — dominio condiviso | 2/3 | T-P2-03 parziale: errore applicativo completo rinviato a contratti futuri |
| P3 — schema/compiler/registry | 7/7 | Compiler incrementale fidato; costrutti non supportati rifiutati |
| P4 — input byte/streaming | 5/5 | Framing, BOM Reject, encoding esplicito e rendering |
| P5 — diagnostica/report/audit | 7/7 | JSONL canonico, SHA-256 input, summary e guard atomico |
| P6 — validazione locale | 8/8 | Pipeline, presence, datatype, identity; LocalOnly ratificato |
| P7 — application service | 6/6 | Trace/LocalOnly, progress/cancellazione/DI; non Full |
| P8 — CLI | 6/6 | Validate/lint, exit code e Ctrl+C nativo |
| P9 — tipi/vincoli semantici | 6/6 | Date/range/regex e Decimal monetario unsigned con zero ammesso |
| P10 — DSL locale | 6/6 | Espressioni/derivati/presenza/regole nel sottoinsieme consegnato |
| P11 — reference data | 5/6 | 11-06 implementata localmente, CI nativa non letta |
| P12 — CSV/XLSX | 2/5 | 12-03 locale verificata; restano wiring CLI 12-04 e test 12-05 |
| P13 — cross-record/cross-trace | 1/8 | JoinKey e piani; mancano projection/store/evaluator/pass 2/Full |
| P14 — schema tooling | 1/5 | Hash canonico consegnato; importer/diff/list ancora futuri |
| P15 — packaging/release | 0/7 | Bundle, verifica autosufficienza e distribuzione da realizzare |
| P16 — UI Slint | 0/9 | Nessuna UI o spike consegnati |
| P17 — hardening | 0/6 | Limiti operativi, benchmark generali, property test e altri gate futuri |
| P18 — documentazione operativa | 0/4 | I dossier tecnici esistenti non sostituiscono queste task di prodotto |
| PERSIST core | 0/12 | Contratto definitivo approvato; repository persistente non implementato |
| Integrazione CAD | 0/12 | Profilo/runtime completo ancora da consegnare |

## Cosa funziona oggi

- Schemi versionati raw → lint/compiler → handle compilati immutabili; hash
  canonico e rifiuto esplicito di feature non supportate. Nessuna norma nel core.
- Validazione Trace/LocalOnly byte-oriented in streaming; lunghezza, presenza,
  logical dichiarato, tipi, domini, date, range, importi, regex, regole locali e
  derivati supportati. Errori normativi separati dai guasti tecnici.
- Input audit SHA-256 dei byte originali, EOF/cancellazione corretti, JSONL e
  summary; CLI reale con output atomico, progress e Ctrl+C su Linux/Windows.
- Provider reference filesystem/memoria, membership/Union con Verified,
  Degraded e Unavailable; quattro esiti senza falsi errori da dataset parziali.
- CSV RFC 4180 e colonne condivise; backend XLSX qualificato nativamente.

Le due estensioni seguenti sono reali ma il gate nativo finale è ancora aperto:
- **11-06, commit 4cb1a65 pubblicato:** preparazione prima degli input,
  Continue/Strict, summary/audit delle evidenze, Complete/Partial, warning bounded
  e root/policy CLI reali. Non simula metadata o storico.
- **12-03, commit ee3036b solo locale:** export post-analisi da JSONL,
  Errori/Riepilogo, celle testuali, freeze/larghezze/filtri, rollover e audit
  completo nel Riepilogo. API di libreria, non ancora opzione CLI XLSX.

## Verifiche più recenti

524 test passati in 61 suite, zero falliti. Un probe ignorato nella suite
ordinaria è stato eseguito esplicitamente due volte. Tutti i gate locali
fmt/check/Clippy -D warnings/test/build release/Windows GNU/diff passati.
Il check GNU è compilazione locale, non esecuzione Windows nativa.

| Prova memoria writer XLSX | 10.000 record | 100.000 record |
| --- | ---: | ---: |
| JSONL | 3.418.894 byte | 34.288.895 byte |
| Workbook | 282.340 byte | 2.786.008 byte |
| Picco RSS Linux | 15.572 kB | 15.840 kB |

Circa 10x dati e +268 kB di picco: low-memory verificato nel range misurato.
Nessuna promessa quantitativa per tutti i workload o per fogli illimitati.
90 artefatti protetti invariati rispetto a 6e4b411: baseline, golden, grammar e
fonti architetturali non adattati al codice.

Ultima evidenza nativa letta: CI 39959838976 sul commit
8fc31f5788a810de7d0309f1eab54adc9646dfa4, Linux job 120434767242 e Windows
MSVC job 120434767095 entrambi completed/success. Chiude 11-05, 12-02 e 13-01.
Non copre i commit successivi 4cb1a65/ee3036b.

11-06: due timeout di connessione, terzo dispatch accettato; nessuna quarta
chiamata per leggere il risultato. Non presumo successo o fallimento della CI.
12-03: push non eseguito perché il controllo automatico è scaduto prima della
creazione del processo, anche nell'unico retry consentito. Nessun dispatch 12-03.
È un blocco operativo del controllo, non un test fallito o un conflitto Rust.

## Milestone e maturità

M0/G0 raggiunta con rinvii espliciti. M3 locale DET §151 raggiunta nello scope
ratificato, documentata dalla chiusura nativa di 8-06. M1 conserva il residuo
approvato di T-P2-03; il core P3–P6 è verificato ma ciò non chiude quel residuo.
M4 resta aperta (gate 11-06, 12-03 e consegne 12-04/05). M5/release, M6/Full,
M7/tooling completo, M8/UI e M9/1.0 non sono raggiunte.

Rilevata e riallineata una tabella milestone storica nello stato: riportava
ancora M2–M9 NOT_REACHED e M1 limitata alle prime task, mentre il dossier 8-06
registra esplicitamente M3 locale raggiunta. Non è una nuova ratifica: sono
state recuperate le evidenze già presenti, preservando il residuo M1.

Non attribuisco una percentuale di conformità sanitaria a SIOC/CAD: G0 e i test
sintetici non certificano tutte le normative. Restano i rinvii del profilo SIOC
minimale, dodici dataset ufficiali SIOC da fornire/verificare, metadata lookup e
funzionalità temporali non implementati, Full e persistenza core obbligatoria.
Nessuna baseline CAD promossa a FROZEN o completamento normativo simulato.

## Punto di ripresa dopo l'allineamento

1. Superare il blocco operativo del controllo push per ee3036b, senza aggirarlo.
2. Pubblicare e avviare un gate nativo cumulativo, massimo tre chiamate Actions
   distanziate almeno 45 secondi; verificare esplicitamente Linux/Windows.
3. Solo con evidenza positiva chiudere 11-06 e 12-03. I vecchi gate esauriti
   non vengono riletti né vengono attribuiti loro esiti inesistenti.
4. Alla ripresa autorizzata, prossima candidata 12-04; poi 12-05 per chiudere
   l'area report. P13/PERSIST/packaging/UI seguono dipendenze e contratti, non
   una deduzione dal solo numero percentuale.

Nessuna nuova task implementativa avviata al checkpoint. Commit e GitHub
rimangono autonomi nel workflow concordato; questa pausa risponde alla richiesta
successiva dell'autore di ricevere prima il resoconto completo.
