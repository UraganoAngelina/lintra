# U-REL-03 — Durabilità e recovery

11 novembre2026. **WAITING_CI**. Prerequisiti U-OPS-01 e U-FIX-01 DONE;
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
