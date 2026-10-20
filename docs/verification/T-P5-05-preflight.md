# T-P5-05 — Preflight / CNF-AUDIT-01

2026-10-20. **BLOCCO DI CONTRATTO: proposta A non ancora approvata.**
Task letta, nessuna implementazione Rust o dipendenza SHA-256 aggiunta.
Prerequisito T-P5-04 locale verificato su a804956, relativa CI in corso.
Questa analisi non salta il gate né dichiara pronta la task.

## Evidenze e problema circoscritto

- ARCH §35/§39C.3 e DET Addendum A: audit con nome, dimensione e SHA-256
  dell’input, senza path assoluto. T-P5-05 richiede hash durante la lettura.
- DET §57/§101 e i tipi neutri T-P2-03 approvati non hanno un payload per
  l’audit dell’input: RunMetadata/ValidationSummary riportano solo schema,
  software, reference data e tempi/contatori. Va esteso il contratto, non
  sovraccaricato schema_hash o technical_ids/identità di record.
- DET §119: su annullamento chiudere le risorse, non effettuare ulteriori
  pass e finalizzare Cancelled solo se il sink lo permette. Se si interrompe
  prima di EOF non si conosce lo SHA-256 dell’intero file dalla sola lettura
  eseguita. Nessun documento stabilisce l’eccezione all’hash obbligatorio
  per questo caso. Il digest del prefisso non è lo SHA-256 del file.

Serve approvazione dell’autore su un’estensione del risultato e sulla
completezza dell’audit in un run interrotto. Non è una richiesta di conferma
ordinaria per scrivere builder o aggiungere il crate già previsto dalla task.
Non si cambia alcuna regola normativa, baseline o documento sorgente.

## Proposta A — da approvare

1. Aggiungere al dominio i dati neutri:

   ```rust
   pub struct Sha256Digest(pub [u8; 32]);
   pub enum InputAuditCompletion {
       Complete { sha256: Sha256Digest },
       Incomplete,
   }
   pub struct InputAudit {
       pub trace_id: TraceId,
       pub file_name: String,
       pub file_size_bytes: u64,
       pub bytes_read: u64,
       pub completion: InputAuditCompletion,
   }
   ```

   Aggiungere inputs: Vec<InputAudit> a RunMetadata e ValidationSummary.
   L’ordine è quello dei file effettivamente selezionati nello scope, un
   elemento per trace/file. Nessun record, identificativo paziente, path
   assoluto o stato di persistenza. Trace non forniti non vengono inventati.

2. Hash dei byte sorgente esatti, prima di BOM/newline/decoding, inclusi
   marker e terminatori. Wrapper streaming generico; nessun secondo pass
   per l’audit e nessun caricamento globale. Lo schema_hash resta separato.
   file_size_bytes è la dimensione del file all’apertura, bytes_read conta
   byte sorgente effettivamente letti (anche read-ahead, non record validati).
   Un disaccordo fra dimensione iniziale e byte letti a EOF è un errore
   tecnico di audit, non una violazione normativa.

3. Complete è disponibile soltanto dopo EOF osservato e dimensione coerente.
   Un’interruzione prima di EOF produce Incomplete, senza hash del file e
   senza riutilizzare il digest parziale. Ciò vale anche se bytes_read ha
   già raggiunto la dimensione nota ma EOF non è ancora stato osservato.
   Se read-ahead ha effettivamente raggiunto EOF, l’audit può essere completo
   pur con validazione annullata: completezza input e stato del run sono
   distinti. Nessuna lettura aggiuntiva per completare hash dopo cancellazione.

4. Questa è la deroga limitata richiesta: su run annullati/troncati l’audit
   può essere Incomplete. Un run non interrotto deve avere audit completo
   per ogni file. Su guasto I/O resta Err(Input), nessun summary definitivo
   pubblicato; un eventuale snapshot incompleto è diagnostico, non un report
   completo. Non si ridefiniscono RunStatus, Cancelled o truncated.

5. file_name è solo basename UTF-8; basename non rappresentabile in UTF-8
   produce un errore tecnico esplicito, non perdita silenziosa tramite
   to_string_lossy. Le API di apertura continuano a usare Path/PathBuf;
   questa scelta riguarda solo l’etichetta dell’audit nel report.

6. In summary.json il digest completo usa 64 cifre esadecimali minuscole;
   per Incomplete il campo SHA-256 non viene inventato o riempito con zeri.
   started_at esistente rimane timestamp UTC RFC3339 nell’output separato.
   Builder e writer saranno implementati dopo la decisione, con contatori
   controllati, audit esplicito e regressioni Linux/Windows. Crate hash e
   relativa licenza/pure-Rust verranno verificati e registrati in DEC-15
   come già richiesto dalla task; nessuna dipendenza viene anticipata ora.

## Artefatti reviewabili e verifiche

- docs/proposals/input-audit/complete.json: file sintetico ABC + due spazi
  + CRLF (7 byte), SHA-256 calcolato con Python hashlib sui byte espliciti.
- cancelled-before-eof.json: stesso file, 3 byte letti, nessun hash completo.
- Entrambi JSON parseabili, basename senza percorso, contatori coerenti;
  hash completo e assenza di quello incompleto verificati nello script.

Sono esempi di proposta, non golden o contratto già implementato/approvato.
ARCH/DET/Master Plan e contratti Rust attuali rimangono invariati.

## Alternativa / impatto

L’autore può fornire un’altra policy completa per audit e cancellazione.
Non si sceglie autonomamente una scansione integrale dopo Ctrl+C né si
pubblica l’hash del prefisso come file hash. Fino alla decisione T-P5-05
e le task che dipendono dal nuovo summary restano ferme; T-P5-06 o altri
rami indipendenti non sono architetturalmente bloccati da CNF-AUDIT-01.
T-P2-03/M1 rimangono aperte come già approvato.
