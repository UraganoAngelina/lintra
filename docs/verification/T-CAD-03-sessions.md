# T-CAD-03 — sessioni CAD immutabili, 6 novembre 2026

DONE. Gate finale CI42203885693 positivo su6af5f85: Linux/MSVC e tutti
i5job di confronto/riapertura crossOS passati. Evidenza integrale condivisa in
T-CAD-11-checkout-recovery-native-ci.json. Le precedenti CI fallite restano
preservate; la recovery journal e il fix checkout sono inclusi nel commit qualificato.
Prerequisiti, fonti e confine della consegna nel preflight03.

## Configurazione e confine della consegna

Il componente versionato2024-06.namespace.1 già contiene l'intera policy snapshot
necessaria: scope sending_ulss, year/quarter, approve esplicito, immutabilità,
commit atomic_idempotent e missing policy. T-CAD-03 collega e qualifica questa
configurazione sul coordinatore core senza creare un secondo schema di storage.
SESSIONS.md descrive l'uso delle API reali e distingue preview del candidato da
reconciliation. La task04 calcola differenze/annullamenti/riattivazioni e vincolo
progressivo-evento;05 conserva i record dei trace assenti;12 integra/golden.
I compiler ordinario/audit-only rifiutano ancora lo snapshotCAD incompleto.

Non si dichiara una validazione clinica Full CAD eseguita: i test della task03
sono component acceptance dell'approvazione e usano outcome pubblici espliciti,
come il confine PERSIST07. L'API ApprovalDraft verifica coerenza del contratto,
non firma un'attestazione del servizio Full. Gli adapter SQLite e tutte le
operazioni di protezione, lettura, preview, approvazione, CAS e riapertura sono
reali. Nessuna nuova API, dipendenza, norma, SQL, HMAC o runtimeengine modificato.

## Test, funzioni e motivi di correttezza

In tests/cad_sessions.rs, declaration legge il JSON versionato; raw assembla un
contenitore sintetico con gli ID/PK/parametri/policy esatti. Solo i campiPK vengono
creati con larghezze sintetiche, eccetto namespace_field fornito dal componente.
Non si imita il layout regionale83campi. profile usa compile_persistence_profile,
perciò typo/riferimenti/capability errati non producono una policy valida.

parameters conserva i tipi stringaULSS e integerperiodo; namespace usa il
costruttore del dominio. parts costruisce metadati Draft con inventario completo,
periodo non inferito, presenza e digest input dichiarati di fixture. outcome
costruisce esplicitamente una fixture del contratto summary: conteggi pertrace,
PKchecked, Complete e parametri corrispondenti; summary.validate la controlla.
Questi digest/outcome non sono descritti come hash/esiti di file reali validati.

rows costruisce tuple sintetiche con le aritàPK del profilo (2/2/2/5/5). Usa un
estrattore separato interamente sintetico e build_join_key, poi protect_projection
nello scope ULSS/flow/schema/trace/record_key. Non rimuove policy al candidatoCAD
per farlo compilare. Le tuple/identità/fingerprint vengono attestate dal core;
nessun valore record in chiaro è salvato negli snapshot. La chiave statica è
materiale pubblico di test, non provisioning operativo.

candidate attraversa ApprovalDraft.new→validated con la fixture coerente e le
identità protette. approve usa preview e approve_commit reali. immutable richiede
esattamente il rifiuto normativoE_APPROVED_SESSION_IMMUTABLE, non un genericoErr:
erroriI/O o di fixture non possono far passare il controllo d'immutabilità.

I sei casi provano:

- PolicyCAD compilata, scope/periodo/PKC-D e warning esatti; compiler ordinario e
  audit-only ancora rifiutano esecuzione del profilo snapshot incompleto.
- T1/T2/T3 approvati in sequenza con riaperturaSQLite fra trimestri; cinque
  identità protette e metadati ApprovedImmutable riletti per ogni revisione;
  tentativi di cambiare tutti i trimestri rifiutati, retry delle tre ricevute
  originali dopoT3, nuova versione schema incapace di sbloccareT1.
- Draft scartato e preview annullata senza versioni nascoste, candidato rivisto
  verificato nel callbackreport e nella sorgente pubblicata; vecchio candidato
  rifiutato dopo approvazione della revisione.
- Stesso trimestre indipendente fra501/502; nuovo anno incrementa la generazione
  dello stesso namespace501 e non permette di riscrivere il vecchio trimestre.
- Errore report e cancellazione dal callback non scrivono; due preview sulla
  stessa base, soltanto la prima approva. La seconda fallisce GenerationConflict;
  dopo riapertura resta una sola sorgente e generazione1.
- ULSS fuori dominio, periodo non valido o namespace divergente rifiutati;
  Cancelled/truncated/Partial/parametri dell'outcome diversi non creano capability.

## Tentativi preliminari e limiti

Log preliminary-first: erroreRust del test dovuto a shadowing fra submissionID e
TraceId; corretto il nome locale senza modificare il prodotto. preliminary-second:
SQLite rifiutava il filesystem temporaneo di sistema e la fixture summary aveva
PKchecked0 con record_count1. Usato tempdir_in(target), come gli altri testSQLite
qualificati, senza indebolire il controllo filesystem; conteggiPK corretti nel
contratto fixture, senza toccare summary.validate. preliminary-third:6/6pass.
I gate finali sono separati su hash fermi. Nessuna suppressionClippy o errore
trasformato in successo. Windows richiede ancora il runCI del nuovo commit.

Questi test non coprono ancora lo stato attivo cumulativo dei record né il
vincolo progressivo-evento, controlli temporali o la pipeline Full/historicalCAD
completa. Le task successive restano aperte con i loro criteri effettivi.

## Gate locali conclusivi e handoff

Sei comandi exit0, source_before/source_after identici nel ledger
T-CAD-03-local-gates.json: fmt, Clippyworkspace all-targets allfeatures,
suiteworkspace922esiti positivi/0falliti/6ignored,38mirati allfeatures/0falliti,
buildrelease e verifica grafo dipendenze. Il conteggio comprende i riepiloghi
pubblicati dai subprocess di test; non include i tentativi preliminari falliti.
Sourcehash e log conservati, nessuna baseline o funzione produttiva modificata.
Test Windows non eseguiti localmente: richiesto nuovo runCI sul commit pubblicato.

Consegna locale: tests/cad_sessions.rs, SESSIONS.md/README del componente e
preflight/report/ledger/log. CAD03 nonDONE prima del gate nativo;130/140DONE,
10task rimanenti invariati. Prossima dipendente04 attende03; CAD11 indipendente
(P11/P10/G0 qualificati) può iniziare dopo dispatchCI, in sequenza. Nessuna nuova
release/tag. docs/migration preesistente preservata/esclusa. Single-agent.
