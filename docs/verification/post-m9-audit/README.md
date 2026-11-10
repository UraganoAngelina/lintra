# Evidenze dell'audit post-M9 — 10 novembre 2026

Solo sorgenti, documenti e dati sintetici. Nessun archivio operativo o chiave
operativa è stato acquisito. I log sono versionati come `.log.gz`, con byte
originali preservati e digest verificabili; non contengono dati paziente.
La compressione evita di mescolare il whitespace grezzo dei processi con i
controlli dei sorgenti. I `.log` locali sono cache di lettura ignorate da Git.

Artefatti decisionali nella root: LINTRA_RETROSPECTIVE_AUDIT_REPORT.md,
LINTRA_POST_M9_GAP_REGISTER.md e LINTRA_POST_M9_ENGINEERING_ROADMAP.md.

## Provenienza e integrità

- baseline.json: Git/ambiente iniziali, confronto con d45078e, diff originale
  4e44703; working tree pulita prima della creazione delle evidenze.
- mandate-source.json: digest del mandato incollato dall'autore.
- qualified-inputs.json: 498 hash dei sorgenti/test/tooling/configurazioni,
  schema/bundle/build script; quelli aggiunti dopo il primo inventario sono
  confrontati anche con i byte Git del commit af3bccc.
- dependencies.json e tooling-dependencies.json: lock workspace e lock xtask
  separati; sqlite-amalgamation.json identifica la sorgente bundled effettiva.
- profile-inventory.json: capability, trace/campi/PK/regole, rinvii SIOC e
  dichiarazioni reference, senza inventare dataset mancanti.
- storage-fix.patch: delta SQLite/test; la correzione README è nel commit af3bccc.
- published-releases-live.json: lettura API delle release private effettive;
  nessuna nuova release o tag creati.

## Gate

gates.json collega comando, exit code, ambiente/commit, archivio e digest dei log. I
conteggi aggregano i riepiloghi Rust, inclusi helper, e non sono test unici.
Ignored rimane separato. storage-red è una riproduzione con due fallimenti
attesi sul codice originale: non è un gate positivo.

Sono conservati anche i tentativi non qualificati. interrupted-attempts.json
registra i comandi interrotti senza promuoverli a PASS. Il primo raccoglitore
CAD83 ha letto un prefisso errato: cad83-performance.log/JSON e
perf-cad83-100-attempt1.log conservano il fallimento dello strumento, distinto
dal test CAD83 positivo. cad83-performance-final e performance.json sono la
misura completa successiva, dopo la correzione del parser. Il primo boundary
probe aveva un import inutilizzato; boundary-probes-final esegue la stessa
sonda con rustc -Dwarnings ed è il riferimento finale.

Comandi riproducibili, dalla root, con un'etichetta nuova per non sovrascrivere:

```sh
python3 docs/verification/post-m9-audit/run_gate.py NEW-LABEL cargo test -p lintra --lib --locked --offline persistence::sqlite::tests:: -- --test-threads=1
python3 docs/verification/post-m9-audit/boundary_probes.py
python3 docs/verification/post-m9-audit/summarize_evidence.py
```

boundary_probes usa la library Rust già costruita, un temporaneo privato e
sentinelle pubbliche; misura permessi Unix, byte CSV e buffer di una riga da
8 MiB, senza aprire spreadsheet o provocare OOM. Il suo script Linux non è una
verifica ACL Windows. L'output serve a descrivere condizioni operative,
non a presentare come conforme una protezione assente.

Per riprodurre le misure CAD83 occorre prima il comando release-cad83-build
nel ledger. measure_cad83.py riconosce l'eseguibile dal JSON Cargo e misura
direttamente quel processo con wait4. Non include la compilazione nel RSS;
riporta invece fixture e cleanup nella metrica di processo. Rinomina o conserva
prima i precedenti output: la creazione esclusiva evita di perdere evidenze.
Nessun valore osservato diventa una soglia DEC-14.

## CI nativa

Baseline: run42641969728 su d45078e, riletta dal servizio. Correzioni:
run42789338814 su af3bcccb6246a7ff00463c595f4c780a9f5c5195.
Le letture intermedie sono conservate come tali; native-ci-final.json riporta
completed/success, sette job richiesti positivi. I log completi compressi ed
estratti sono verificati da native-log-verification.json; native-imports
conserva i due risultati effettivi con 4 snapshot/2 namespace, batch1/2 e nullable2
in ciascuna direzione. UI packaging opzionale skipped non
equivale a un nuovo archivio UI prodotto/qualificato.

La matrice CI comprende test nativi standard/all-features/core senza bundle,
tooling, release riproducibile/artifact, quickstart/bundle, JSONL/pixel e due
import dei DB del runner opposto. Non sono prove di power-loss hardware,
rotazione, backup live o autorizzazione sanitaria. Le evidenze di sintesi non
duplicano i database e le grandi immagini già conservati negli artifact CI.
