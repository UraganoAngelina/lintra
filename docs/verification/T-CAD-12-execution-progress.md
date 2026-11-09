# T-CAD-12 — avanzamento attuativo, 9 novembre 2026

Mandato: completare CAD12, poi P18-04/M9 quando READY; C1/L1 approvate.
Single-agent, nessun background/delega. docs/migration preesistente esclusa.

Storage c5266d02985f34b93c5a63a2c610a3dfe919b90f qualificato da CI42592330836,
sette job richiesti e due importSQLite4snapshot/2namespace/batch/manifest.
Evidenza documentale nel commit750b1e1, da pubblicare con il delta runtime.

Implementazione runtime completata nel working tree, ancora senza commit/CI
propri. PASS1 per-field, compiler Eq/deps/digest, query C1/L1, PASS2 qualificata,
servizio pubblico snapshot, review/approval/replay e CAD83 reale collegati.
Descrizione tecnica e matrice HX01..21 in T-CAD-12-runtime-implementation.md.

Ultima suite mirata: /tmp/lintra-history-expanded-all.log, lib142pass/4ignored,
21test applicativi generici e9CAD83 positivi; misurazione esplicita CAD ignorata
nel test ordinario. Nessun fallimento equiparato a PASS. Correzioni preliminari:
API consent con timestamp, rappresentazione Fingerprint non stringa, presenza
source assente con contatori a zero e audit dichiarato tramite invarianti.
Replay dopo attestazione successiva verificato; reader fatti1/2 e null2 testati.

Composizione83campi riprodotta byte-identica con hash7159edc14cbc1198362271a48206f301d22d530a8c8d16b41da598ca0e23ba51.
Nessun filler; schema operativo acquisito dal filesystem e cataloghi Strict reali.
Oracle T1/T2/T3 fisso, riaperturaSQLite, namespace, annullamento, JSONL negativo,
B2/Dassenti, sospensioni, codice15 e date invalide verificati. Fixture native
SQLite ampliate a entrambi i formati logici1/2, mantenendo4snapshot/2namespace.

Prima misura debug100eventi/namespace,6approvazioni,DB40878080byte positiva.
Il piccoRSS del primo invoco includeva la compilazione, quindi NON è una misura
RAM del solo runtime. In corso build e misure release, poi gate complessivi su
input stabili, commit/push e CI Linux/MSVC/JSONL/pixel/import sullo SHA esatto.
Nessuna qualifica runtime nativa ancora eseguita; non usare CI dello storage
come suo sostituto. CAD12IN_PROGRESS,138/140invariato; P18-04nonREADY.
Nessun FROZEN/M9/release/tag e nessun checkpoint ordinario richiesto.

## Campagna stabile in corso

Workspace default positivo:1059esiti nei124riepiloghi,0fallimenti,7ignored;
21testgenerici e9CAD83 inclusi. Fmt eClippy workspace/alltargets/allfeatures
positivi.504input confrontati dopo ogni comando;HEAD/index non modificati.
Assettoallfeatures in chiusura UI; core/tooling/release/misure/native ancora
necessari, non dichiarati PASS. Runner sequenziale foreground
/tmp/lintra-cad12-final-gates.py, logfinal3 e ledger runtime-local-gates.json.

Primi due tentativi completi conservati in runtime-preliminary* e
runtime-preliminary2*: golden root non registrata (registrato ora il runner
CAD versionato senza cambiare attesi), poi codice fail-closed formato1
non più UnsupportedConstruct (ripristinato il guardformato3 all'ingresso join).
Nessun contratto indebolito;8test persistence_profile e13golden mirati positivi
prima del terzo tentativo complessivo. Non equiparare quei tentativi a PASS.

## Gate locali conclusi / pubblicazione

15gatefinali positivi,504input fermi.1059workspace/default e1059allfeatures,
1016core-no-default,0fail. Tooling includeignored18pass dopo invoco corretto
con LINTRA_TEST_BIN esplicito; primo errore di invocazione conservato.
Release riproducibile e artifact gate positivi. Misure release100/300eventi,
6approvazioni ciascuna,0errori: p95empirico759/1920ms, RSS25936/28276KiB.
Ledger runtime-local-gates/runtime-measurements e report runtime-validation.
Ora commit/push/CI runtime autorizzati; CAD12nonDONE finché7job nativi e
import opposti non qualificati sullo SHA esatto. Nessuno stop ordinario.

## Pubblicazione / WAITING_GATE nativo

Codice c59b0def0915d8cca6dae7f59048aa38f50b6807 pubblicato insieme al dossier
storage750b1e1; push51950af..c59b0de riuscito. Unica CI42634997501 avviata
con ui_packaging=false; prima lettura verifica SHA/branch e stato pendente.
Ledger runtime-native-ci.json; non DONE12, P18-04nonREADY. Normali letture
successive distanziate e sobrie; non usare watch/pollingcontinuo.
