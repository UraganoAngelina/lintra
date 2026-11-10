# T-P18-04 — sincronizzazione finale dei documenti

## Preflight — 10 novembre 2026

CAD12 DONE, commit di chiusura f23be67. Prerequisiti restanti già qualificati
nel consuntivo; ultimo prodotto d45078e con CI42641969728, sette job richiesti
success. Verificati API, log dei30casi CAD/history in tre assetti per OS e due
import reali.139/140DONE prima di questa task; nessuna dipendenza aggirata.
Letti piano§4/14/P18/QG-8, prompt18-04, ARCH44/45 e DET166–174.

Scope puramente documentale: allineare descrizioni correnti, compatibilità,
registri di decisioni e riferimenti. Nessun codice, fixture, norma, versione
Cargo, workflow o schema operativo modificato. docs/migration preesistente
esclusa. Nessun test artificiale per la documentazione.

## Tracciabilità delle decisioni implementate

| Contratto/capacità | Fonte corrente e prove |
|---|---|
| API/DI/Full, summary/errori, streaming e report | ARCH/DET; SCHEMA_AUTHORING, INPUT_AUDIT_AND_SUMMARY, REPORT_JSONL e report P2/P7/P13/PERSIST |
| Formati schema1/2/3 e ingressi separati | schema-authoring/FORMAT_COMPATIBILITY; T-CAD-02 e CAD12 |
| SQLite3/fatti1/2/readerlegacy/HMAC1 | PERSISTENCE_FORMAT_REGISTRY; contratto ADR-HISTORY e runtime/storage CAD12 |
| C1/L1, validità per campo, lifecycle, copertura | Contratto ADR-HISTORY adottato e T-CAD-12-runtime-implementation, matrice HX01–21 |
| Preview/review/approval/replay/CAS e report | PERSISTENCE_CORE_SPEC, DET169 e report CAD12; nessuna mutazione durante validation-only |
| Chiave portabile, ACL e limiti operativi | PORTABLE_KEY_BUNDLE e PERSISTENCE_OPERATIONS, ratifiche provisioning/storage |
| Ordine per passata/budget/cancellazione | DET79/115, ARCH17 e CNF-DIAGNOSTIC-ORDER-01, CI42530063288 |
| Slint e confini core/adapter | docs/ui-design-documentation e task P16, boundary e pixel CI qualificati |
| Parallelismo non adottato | DEC-17-final-ratification, produzione sequenziale |
| Misure/soglie | PERFORMANCE, soglie non definite con motivo; CAD12 release100/300eventi documentati |
| Packaging/release/manutenzione schemi | RUNBOOK_RELEASE, RUNBOOK_SCHEMA_UPDATE e P18-01..03, rc.2 immutabile |

Il registro corrente di tutti i CNF è in CONFLICT_RESOLUTION_MATRIX.
Nessuna chiusura è basata sulla sola assenza della stringa OPEN: proposte e
handoff storici restano leggibili, mentre ratifiche sostitutive e gate determinano
lo stato corrente. L'inventario meccanico verifica che tutti gli ID CNF citati
nei documenti siano coperti dal registro corrente.

## Modifiche e motivi

README descrive il servizio snapshot qualificato e conserva i comandi rapidi.
ARCH45, DET174 e PERSISTENCE_CORE_SPEC collegano le decisioni al codice e ai gate.
SCHEMA_AUTHORING distingue i vecchi componenti dall'ingresso snapshot integrato.
Registro formati e mappa compatibilità descrivono l'eseguibile di sviluppo
rc.3 senza estendere la release rc.2. HISTORY_EXISTENCE_SPEC conserva l'analisi
iniziale con un rinvio esplicito al contratto adottato. CI separa stato corrente
e bootstrap; DEVELOPMENT registra la decisione Linux già approvata.
PERFORMANCE aggiunge le misure CAD83 con metriche e limiti già qualificati.
Stato/indice e matrice CNF saranno chiusi soltanto dopo il gate documentale.

## Gate richiesto

Verifica inventario CNF/decisioni, link locali nuovi/modificati, quickstart
README byte-identico, hash degli input prodotto rispetto al commit qualificato,
coerenza formati/versioni, git diff-check. Eseguire fmt/Clippy/test richiesti
per la task e controlli Python/boundary. Ledger T-P18-04-doc-gates.json.
Per nessuna modifica Rust non è necessaria una nuova CI nativa duplicata:
i due OS sono già qualificati sul prodotto identico; il gate della task verifica
la tracciabilità e la documentazione. M9 è la milestone del piano, non una
pubblicazione1.0, un freeze CAD o l'autorizzazione a usare dati reali.

Stato iniziale IN_PROGRESS; risultati effettivi nel ledger prima della chiusura.


## Quality Gate e chiusura — 10 novembre 2026

DONE, QG-8 superato. Inventario completo28ID CNF senza residui correnti,
link nuovi verificati e comandi rapidi README byte-identici. Equivalenza degli
input produttivi verificata contro il gate CAD12; sola variazione precedente
workflow già qualificata da CI42641969728. Nessuna modifica prodotto in18-04.

Fmt e Clippyworkspace/alltargets/allfeatures exit0. Testworkspace--locked exit0:
1059esiti in124riepiloghi,0fail,7ignored; conteggi aggregati anche sui sottoprocessi,
non test unici. Inclusi core/CAD83/history, UI con report fino a1Mrighe,
regressioni visuali e doctest.24testPython e boundary exit0. Ledger con hash
log e comandi effettivi; nessun test ignorato contato come PASS.

Documenti da aggiornare: nessuno nel perimetro ratificato.140/140DONE e
M9 raggiunta come milestone del piano, senza bump/versione/release/freeze.
Nessuna ulteriore task READY; manutenzione o nuove estensioni richiedono un
perimetro distinto. Limiti G0, dati reali, soglie non definite e audit-only SIOC
rimangono espliciti. Git: commit/push sul branch concordato, senza force;
nessuna nuovaCI nativa per documentazione e sorgenti prodotto invariati.
