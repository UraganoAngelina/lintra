# SIOC: validazione degli invii cumulativi e delle PK storiche

Decisione: [DEC-SIOC-HIST-02-A01](normative/DEC-SIOC-HIST-02-A01.md), ratificata
dall'autore il10 novembre2026. Questo documento descrive il nuovo percorso
Rust; la qualifica nativa effettiva è registrata nel dossier U-NORM-03.

## Profilo operativo e perimetro

`schemas/sioc/2026-04-20.cross-run.1/flow_schema.json` deriva dal componente
referenziale già qualificato. Conserva layout,26 campi fisici (20 selezionati
più6 span opachi), sei controlli esistenti e tutte le PK. Aggiunge D11, formato3
e capability `persistence.cross_run.v1`; non attiva i controlli regionali deferred
nel profilo completo structural-minimal.2. Il ledger conserva provenienza e hash.
Non è una qualifica universale di tutti i135 campi del workbook.

| Tracciato | PK ordinata | Confronto storico |
| --- | --- | --- |
| 1 | 1.1,1.17,1.18 | Ricopertura identica ammessa |
| 2 | 2.1,2.49,2.50 | Ricopertura identica ammessa |
| 3 | 3.1,3.2,3.35,3.36 | Ricopertura identica ammessa |
| 4.1 | 4.1,4.7,4.8,4.19.1 | Riuso fra nuovi invii rifiutato anche se identico |
| 4.2 | 4.20,4.22 | D11, senza estensione della ricopertura1–3 |

I parametri sono `organisation_namespace`, `reporting_year`, `reporting_month`.
Il namespace organizzativo non include anno/mese: il cambio d'anno non azzera
l'indice delle identità. C20 delimita invece all'anno di competenza la copertura
cumulativa obbligatoria. Non si deduce l'anno dalla PK o dal nome del file.
Una sessione precedente alla progressione approvata resta indeterminata, salvo
replay esatto di una sessione già approvata.

Il confronto riguarda l'archivio locale selezionato e autenticato. Non dimostra
l'assenza di record in archivi aziendali/altre installazioni non disponibili.
Un archivio appena creato, anche a gennaio, non prova che una PK sia nuova.
L’inizializzazione consiste in un invio effettivamente vuoto di gennaio, con
input controllati, preview, review e commit approvato tramite il servizio reale.
`initial_baseline` rende visibile questa condizione nella summary; qualunque
record osservato prima di tale approvazione resta Unknown e non è acquisibile.
L’inizializzazione attesta il perimetro locale scelto, non l’assenza di invii
aziendali esterni: per archivi preesistenti/incompleti non è una migrazione.
Una prima fase successiva senza baseline pertinente resta Unknown.
Una baseline PRESENT vuota è distinta da una baseline assente/non leggibile.

## Esecuzione e approvazione

`HistoricalValidationService` seleziona il produttore normativo dalla policy
compilata, prima di aprire gli input. La risoluzione audit-only e quella ordinaria
rifiutano questo profilo; non esiste un flag che trasformi un audit in approval.
La CLI `history preview` usa questo stesso servizio e i parametri espliciti.
Per una distribuzione esterna si colloca il profilo in
`<schema-dir>/SIOC/2026-04-20.cross-run.1/flow.json`, con reference root controllata
e chiave comune già configurata. Il profilo non è aggiunto tacitamente al bundle
embedded sintetico o alla UI Slint.

La passata1 produce i fingerprint dai buffer effettivamente validati, senza
seconda scansione e senza conservare righe cliniche. La passata2 emette le
violazioni storiche e i duplicati intra-invio4.1, applicando cancellazione e
max_errors esistenti. I record rifiutati non entrano nell'insieme valido dei join
e non possono essere approvati. Le occorrenze fisiche restano nei conteggi e
negli hash degli input. L'unicità intra-invio degli altri tracciati è preservata.

Solo esito completo e valido produce preview/review/approve. Il costruttore
pubblico di draft rifiuta la promozione diretta di un profilo normativo.
L'approval conserva CAS e revisione osservata; report fallito, cancellazione o
CAS fallito non cambiano la baseline. Il replay esatto mantiene la revisione
approvata; lo stesso submission ID con contenuto differente non è una rettifica.

## Baseline, identità e contenuto

`CrossRunEvaluator::prepare` verifica pagine di al massimo128 metadati, continuità
revisioni, HMAC delle fonti, stato approvato, schema/hash, namespace e binding
mensile canonico nel campo `period` già esistente. Seleziona una sola baseline
PRESENT pertinente per tracciato, non un'unione dei vecchi file cumulativi.
Carica solo gli snapshot selezionati e verifica identità, attestazioni e conteggi.

`observe` distingue PK assente, PK identica e contenuto diverso. Le ricoperture
sono occorrenze fisiche della stessa identità logica; non aumentano `new_records`.
Per una PK assente dalla baseline corrente verifica anche l'ultima occorrenza
esatta conservata, compresi gli anni precedenti. L'assenza nel nuovo anno non
trasforma quindi un'identità già acquisita in un nuovo record. Le policy strict
controllano il riuso anche dopo un invio che abbia omesso il record.

`finish` verifica copertura corrente e omissioni rispetto alla baseline pertinente.
Le omissioni sono anomalie di completezza: summary Partial e nessuna approval,
senza cancellazioni o annullamenti impliciti. Lo stato indeterminato espone una
causa stabile; errori di storage/attestazione restano tecnici. Una lettura di
pagina incompleta non può essere interpretata come archivio vuoto.

Il contratto `fixed_width_record_bytes_without_terminator.v1` usa l'HMAC esistente
su tutta la riga a larghezza fissa, inclusi padding e span opachi; esclude solo
LF/CRLF. Non applica trim, case folding o equivalenze non provate. Il descriptor
compilato include layout, encoding e policy fisiche ordinate per field ID.
Un cambiamento di contesto/fingerprint non è considerato identico. Snapshot
legacy restano leggibili e invariati; senza binding mensile o schema compatibile
producono Unknown nel nuovo controllo. Nessun nuovo algoritmo, chiave, HMAC1,
codec SQLite o migrazione distruttiva.

## Report e limiti applicativi

`full_validation.cross_run_validation` è opzionale e assente nei vecchi report.
Espone anno/mese, inizializzazione vuota, generazione osservata e revisione
baseline per tracciato,
ricoperti, nuovi, mismatch, riusi, mancanti, indeterminatezza e errori non emessi
per limite/cancellazione. Non espone PK protette o righe cliniche. I contatori
sono verificati dal contratto della summary prima di pubblicare un esito.
Il JSONL canonico degli errori mantiene writer e formato esistenti.

La futura React/Tauri deve consumare questi dati Rust, mantenendo il workflow
reale di review e approval; non deve riprodurre la regola in TypeScript. La UI
storica e il gate G-SIOC1 aggregato restano da qualificare nel piano unificato.
I join referenziali del componente restano current-run: l'assenza di un file
amministrativo necessario non è automaticamente sostituita da dati storici e
può rendere Full Partial. La regola cumulativa non autorizza la ricopertura di
4.1 per eludere quel limite. Questo limite è esplicito e non una falsa conformità.
