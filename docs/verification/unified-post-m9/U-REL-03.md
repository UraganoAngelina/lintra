# U-REL-03 — Durabilità e recovery

Aggiornamento 12 novembre 2026. **DONE; G-DUR software PASS**.
Hardware **NOT_TESTED_HARDWARE**. Prerequisiti U-OPS-01 e U-FIX-01 DONE;
D15 autorizza hardening e prove di interruzione. Preflight da0892e.
U-OPS-04 sospesa in attesa della CI43050943529 sul codice34f5b71.

SQLite configura e verifica già DELETE/EXTRA, foreign_keys e mmap0. Esistono
kill del writer con pagine non committate, reopen, CAS e risoluzione del commit
ambiguo. Riutilizzare questi meccanismi; nessuna riscrittura repository/WAL.
Mancano una matrice esplicita sui manifest/fatti durante crash e prove dei
punti di pubblicazione report. AtomicReportFile sincronizza il file prima del
rename, ma non sincronizza la directory Linux dopo il cambio di nome.

Delta previsto: sync directory Linux; richiesta write-through nel rename
Windows sullo stesso volume senza delete/copy fallback; errore tecnico esplicito
per directory-sync fallita dopo pubblicazione, con file già pubblicato contato
correttamente negli export. Test-only fault injection e processi figli posseduti,
terminati e attesi dal test, senza hook/flag nel prodotto. Nessuna nuova chiave,
crypto, dipendenza, normativa o modifica dei writer canonici.

Un fallimento dopo rename non ripristina magicamente il vecchio file. Il
chiamante deve distinguere file visibile da durabilità non confermata. Report
e SQLite rimangono transazioni distinte. Su filesystem che non supportano il
sync directory il file può essere già salvato, ma l'errore non diventa successo.
Questo non introduce una denylist delle destinazioni né aggira D04.

Fonti tecniche primarie consultate: Linux man-pages fsync(2), Microsoft
MoveFileExW e SQLite PRAGMA synchronous. WRITE_THROUGH non è una prova fisica
universale della persistenza dei metadati, soprattutto su storage remoto.
G-DUR resta NOT_VERIFIED; hardware power-loss NOT_TESTED_HARDWARE.
Backup-B/restore applicativi mantengono le task U-OPS-06/08 e i relativi gate.

## Implementazione e prime verifiche

Implementati sync directory Linux, MoveFileExW con write-through e supporto
path lunghi Windows, variante non-exhaustive PublishedDurabilityUnconfirmed e
conteggio del file già visibile nell'errore export. Contratti e matrice nel
documento docs/DURABILITY_AND_RECOVERY.md; writer e repository invariati.
I nuovi helper dei test esistono solo sotto cfg(test), non nel prodotto.

core-v1 ha fallito nel solo nuovo test manifest: apriva il lettore dopo il
lock del writer e riceveva correttamente StorageBusy. Nessuna modifica al
repository: anticipata l'apertura della connessione come nel test qualificato
esistente. core-v2:156 test libreria PASS,5 ignored; i3esiti di helper figli
nel log sono esecuzioni ausiliarie, non ulteriori casi distinti della suite.
Test manifest include quattro fasi, reopen e risoluzione idempotente;3nuovi
test report comprendono6barriere di kill e4fault di pubblicazione.

export-v1:5test reali PASS, inclusa la prova ignored eseguita esplicitamente
con permessi Linux0300. Dopo rename il file è contato in failure.published,
summary non pubblicato e parti ripulite; la validazione resta CompletedInvalid.
Successivamente i nuovi test report/export sono stati spostati da /tmp(tmpfs)
a directory fixture proprie sotto target sul Btrfs del workspace: necessaria
nuova verifica, non riattribuire quella precedente al filesystem su disco.
I test SQLite già usano il filesystem del workspace.

Suite workspace-v1 ora in corso sul delta completo; Clippy, nuove prove Btrfs
e CI Linux/Windows ancora da completare. Nessun DONE o G-DUR PASS anticipato.
CI43050943529 appartiene all'export34f5b71, non a questo nuovo delta.

## Qualifica locale del delta e invio alla CI

workspace-v1:1109 esiti PASS,0fail,9ignored in554.397s,529 input immutati
durante il check; comprende i casi Btrfs di kill report e recovery manifest.
Le somme del harness includono gli esiti ausiliari dei processi helper;
non sono un conteggio di nomi di test unici. export-v2 esegue davvero tutti
i5casi, incluso il fault di permessi ora su Btrfs: PASS. OS locale Linux
7.3.3-1-cachyos x86_64; workspace Btrfs, /tmp tmpfs.

Clippy-v1 ha rilevato un'allocazione nel confronto del solo hook test.
Corretto PathBuf::from(p) in Path::new(&p), confronto equivalente senza copia;
Clippy-v2 workspace/all-targets/all-features -Dwarnings PASS,529input immutati.
I log negativi restano conservati. Nessun lint è stato disabilitato.

Dopo la suite completa sono cambiati soltanto il confronto borrowed nell'hook,
il blocco cfg(windows) dei path, i relativi2test Windows e il setup CI. I path
brevi Windows conservano la normalizzazione OS, i lunghi il supporto verbatim;
entrambi richiedono prove native, non attribuite alla suite Linux. Casi report
locali rieseguiti sullo stato finale; nessuna nuova modifica funzionale Linux.

La CI Linux precedente non poteva installare fontconfig per DNS del mirror
Azure. Il solo runner usa ora archivi ufficiali Ubuntu HTTPS, mantenendo suite,
componenti, Signed-By, autenticazione apt e TLS; update rigoroso e timeout
espliciti. Nessun pacchetto aggiuntivo o modifica al sistema dell'autore.
Python embedded verificato con ast.parse; validazione effettiva setup/CI sul
nuovo commit ancora necessaria. Nessun fork di SQLite, nuovo schema o golden.

**WAITING_CI** dopo i controlli locali; task e G-DUR non sono DONE/PASS finché
le nuove prove native e i confronti cross-OS non sono positivi. Il collaudo
fisico resta NOT_TESTED_HARDWARE anche in caso di gate software positivo.

Ultimo controllo: report-final-v2 esegue3test e6barriere di kill su Btrfs,
PASS; helper ignored invocato dai parent. export-v3 esegue6test PASS,
incluso il nuovo caso di presentazione di2file pubblicati prima dell'errore:
receipt/failure e messaggio ora usano il conteggio reale e non espongono path.
Questa ultima correzione riguarda la copia UI, non il writer o lo storico.
Clippy finale e commit/CI del delta vengono registrati separatamente.

Clippy-v3 finale PASS con529input identici a export-v3 e report-final-v2.
Il manifest dei tre check e i gzip conservano hash e stato working-tree; la
build locale precede il commit e non viene spacciata per un binario CI sul
nuovo SHA. Fmt/diff-check e grafo128task PASS. M9,otto fonti congelate,
schema/bundle/golden e dipendenze di prodotto invariati. Commit autorizzato
con sorgenti equivalenti ai receipt finali; prossima azione CI nativa nuova.

Codice pubblicato: `7d4c02e39554d618e337145cbb5ba8bad4d693ed`.
CI nuova: `43060623511`, avviata sul branch autorizzato dopo la conclusione
della precedente. Non riusa il PASS Windows34f5b71 come prova del nuovo rename.
Durante l'attesa il grafo non contiene task NOT_STARTED con tutti i prerequisiti
DONE; U-OPS-03/U-NORM-04 attendono il bridge effettivo, G-MIG-1 attende Win11.
Nessuna task indipendente viene dichiarata eseguita per riempire l'attesa.

U-REL-03-source-7d4c02e.json verifica che i529 input dei tre receipt finali
corrispondano al codice pubblicato. È equivalenza dei sorgenti, non attribuzione
di un binario locale da working-tree al nuovo SHA o a un runner diverso.

## Rete dei prerequisiti Linux

CI43060623511: Linux fallisce ancora prima di Rust, questa volta per DNS di
archive.ubuntu.com; timeout8min, log completo linux-43060623511-failed.log.gz
con SHA25683f41b23dbbfafcbb0b09fa09c951cab5038f4d546a697ade798d8498abf1238.
Signed-By/TLS non sono stati indeboliti. Windows prosegue sul7d4c02e.

È preparato un workflow manuale distinto di sola connettività, timeout3min,
per verificare i metadati Ubuntu noble/noble-security sul mirror offerto dai
[Linux Kernel Archives](https://mirrors.kernel.org/). Non installa pacchetti,
modifica DNS o rende PASS alcun gate del prodotto. La firma effettiva dei
pacchetti continuerà a essere verificata da apt nella CI completa. Il workflow
è esclusivamente QA, nessuna nuova dipendenza o rete nel prodotto.

Probe43063034911 su1a044e4 SUCCESS per entrambi i metadati noble e
noble-security; log/metadata connectivity-43063034911 conservati. Log SHA256
d63b6182c0793d91e44bd224945c45e62911cbf6bf81e3e3944461c8c149e0c1.
È prova di trasporto HTTPS soltanto, non di compilazione o corretta firma.
Il setup apt completo seleziona ora quel mirror conservando l'autenticazione
Ubuntu esistente. Nessun resolver/hosts/CA modificato, nessun check saltato.

Windows130661899188 della CI43060623511 ha superato Clippy e i casi iniziali
report, incluso path breve con normalizzazione OS, ma fallisce il setup del
nuovo test path lungo: create_dir_all alla riga442, prima del publisher.
La fixture costruiva componenti con spazio finale. Rimossa solo l'ambiguità
del nome con trim_end; Unicode, quattro componenti, lunghezza>260 e tutte le
assertion del publisher restano. Log completo windows-43060623511-failed.log.gz,
SHA256dafec245bc677ab4910200b37044c46c78698f7b71355531cf81da47b6e3a3e6.
Nessuna modifica al runtime del7d4c02e; serve nuova CI completa sul fix.

Nuova CI completa `43078048297` sul commit
`3aa0d14f1f66b51ce1e8ba9c14a327b3eda26a25`, dopo conclusione della precedente.
Fmt/diff-check e sintassi Python del setup PASS; confronto Git conferma src,
ui/src/ui, dipendenze e norme identici al7d4c02e. Delta: fixture Windows e CI.
Task ancora WAITING_CI, non riattribuire i vecchi risultati al nuovo SHA.


## Quality Gate finale — 12 novembre 2026

**DONE; G-DUR PASS per il perimetro software provato.**
[CI43078048297](https://github.com/UraganoAngelina/lintra/actions/runs/43078048297)
su `3aa0d14f1f66b51ce1e8ba9c14a327b3eda26a25` SUCCESS; sette job
applicabili positivi, packaging UI condizionale skipped e non qualificato.
Metadati e log integrale condivisi con `U-OPS-04-native-43078048297.*`;
receipt specifico `U-REL-03-qualification-43078048297.json` con hash,
righe di ciascun oracle e limiti. Il log Linux preesistente, inizialmente
untracked, è conservato senza modifiche: tutte le8077righe corrispondono al
log appena scaricato dopo la sola normalizzazione della rappresentazione ANSI.

| Prova | Linux130723017375 | Windows130723016884 |
| --- | --- | --- |
| Flush/sync/rename e sync-directory sintetici | PASS3esecuzioni | PASS3esecuzioni |
| Kill reale a6barriere report, residui e retry esplicito | PASS3esecuzioni | PASS3esecuzioni |
| Kill/exit a4fasi SQLite, fatti/manifest, reopen e risoluzione idempotente | PASS3esecuzioni | PASS3esecuzioni |
| Errore permessi Linux reale dopo rename, file pubblicato contato | PASS1esecuzione esplicita ignored | Non applicabile |
| Path Windows lungo Unicode e normalizzazione path breve | Non applicabile | PASS3esecuzioni ciascuno |
| Messaggio export con conteggio reale e senza path | PASS2esecuzioni | PASS2esecuzioni |
| Suite default/all-features/no-default-features, fmt, Clippy e release | PASS | PASS |

Le tre esecuzioni core per OS sono default, all-features e senza bundle;
non vengono sommate come casi distinti. I parent coprono sei barriere report
e quattro fasi SQLite; helper ignored sono subprocessi attesi dai parent,
non casi saltati che si pretendono superati. Riutilizzate le regressioni
corruzione/key mismatch/CAS già presenti. Confronto JSONL, confronto pixel
di produzione/tema e reopen del database sull'altro OS: cinque job PASS.

Ambienti effettivi: CI Ubuntu24.04.5 e Windows Server2025; locale CachyOS
su Btrfs dalle verifiche precedenti conservate. I log CI non identificano
esplicitamente il tipo del filesystem del workspace: non si inventa una
qualifica ext4/NTFS. Gli oracle operano sulle directory fixture del checkout,
non soltanto su tmpfs. Signed-By/apt/TLS e tutti i gate sono rimasti attivi.
Il delta fra SHA CI e HEAD di ripresa è solo documentale; nessun nuovo
runtime da qualificare. Chiusura: dossier, receipt, log Linux e stato; contratti
e spiegazioni di struct/funzioni in `docs/DURABILITY_AND_RECOVERY.md`.

**NOT_TESTED_HARDWARE** resta invariato: kill/exit non simulano lo svuotamento
delle cache hardware né provano ogni storage remoto. Backup-B/restore
applicativi restano U-OPS-06/08; Win11 clean-room, account e installer restano
G-MIG/G-REL. Nessuna promozione di questi gate dalla sola CI Server2025.

## Handoff

U-OPS-04 è chiusa e pubblicata nel commit327e5ec; questa task chiude il gate
software D15 sul sorgente3aa0d14. Branch `work/p1-foundations`, nessuna nuova
modifica di codice, dipendenza, schema, golden, chiave o checkout adiacente.
Test locali della chiusura: grafo/preservazione fonti e diff-check; suite Rust
non ripetuta per le sole evidenze documentali dopo CI nativa positiva.

Prossima task READY: **nessuna**, verificato sui128nodi e sullo stato attuale.
P1-02 richiede Windows11 clean-room, già dichiarato indisponibile dall'autore;
non si riapre una domanda già risolta né si sostituisce con Server2025.
U-OPS-03 e U-NORM-04 restano WAITING_INTEGRATION sul bridge Tauri non ancora
ammesso dal gate piattaforme. U-OPS-06/08 attendono03 e le task successive
hanno prerequisiti incompleti. Dettaglio e conteggi nel receipt finale.
**NO_READY_TASK** per il piano invariato. Ripresa da P1-02 quando disponibile
l'ambiente nativo prescritto; nessun consenso nuovo su D01–D15 richiesto.
