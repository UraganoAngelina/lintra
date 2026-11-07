# Recovery CAD06 — 7 novembre 2026

Workflow single-agent, nessun background/subagente, commit/push/CI autorizzati
sul branch work/p1-foundations, repo /home/alberto/Desktop/LINTRA/lintra.
Docs/migration untracked preesistente preservata ed esclusa.

CAD04: commit b59a5b905ae9f82baf5ce23bbc4eb8564fa5cd22 pubblicato; CI42368062138
avviata sullo SHA esatto. Tre letture ordinarie e una aggiuntiva motivata da
gateancorapendente. Ultima: Linux buildrelease, Windows core senza bundle;
nessunDONE04. File T-CAD-04-native-ci.json e T-CAD-04-native-ci-latest.json.
Non cancellare questa run dispatchando06 finché04 non è terminale.

CAD06: implementazione current-input in schema/temporal.rs, lowering compiler
in CompiledAssertion::IntervalExclusion nella seconda passata esistente;
dipendenze e diagnostica adeguate. Componente2024-06.suspensions.1 A/B2/C
con13campi originali verificati. Nessun snapshot/audit/storia combinato
abilitato, nessuna norma nuova. Report e authoring aggiornati.

11test06+51regressione=62positivi,0fallimenti/ignored. Red test del when
indirettamente esterno fallito per accettazione indebita, fix via guardia
espressioni/derivati esistente e green finale conservati. Preliminari/log
copiati in docs/verification/T-CAD-06/preliminary-*.log.

## Gate locale in corso

Processo foreground exec session51491: python3 /tmp/lintra-cad06-local-gates.py.
Ledger T-CAD-06-local-gates.json, log T-CAD-06/gate-N.log. Gli input sono
congelati; non modificare src/tests/schema/UI/authoring/.gitattributes o
indiceGit/HEAD e non eseguire gitstatus fino alla fine: build.rs osserva indice.
Fmt/Clippy finali exit0 e source_unchanged; suiteworkspace attiva all'ultima
lettura. Seguono workspaceallfeatures/core senza bundle/release/tooling/Python/
boundary. Non lanciare altre build concorrenti.

Il runner supporta resume se ancora presente in/tmp: preserva loginterrotto,
verifica tutti gli hash e riparte dopo comandi conclusi. Se il file sparisce,
ricostruire solo il runner dai comandi registrati, conservando log e senza
attribuire exitcode a un comando interrotto. NessunDONE prima degli11gate e
dellaCI completaLinux/MSVC sullo SHAfinale. Gate04 indipendente non qualifica06.

Dopo gate: verificare complete/impronte/log, aggiornare report e inventario,
commit06 con path espliciti incluse evidenze, escludere docs/migration.
Push eCI solo dopo04terminale. Dopo04success con settejob: chiudere04,134/140.
06resta nonDONE fino suaCI. Durante sola attesa remota06, CAD07 è indipendente
READY; CAD05 richiede04DONE. Nessuna nuova release/tag o adozione parallelismo.


## Aggiornamento: CAD04 DONE

CI42368062138 completed/success sullo SHA b59a5b9 esatto con7job positivi.
Status/indice/report04 aggiornati documentalmente, non ancora committati per
conservare HEAD/indiceGit durante gate06. Ora134/140DONE; CAD05READY.
LaCI06 non rischia più di annullare04; avviarla dopo gate/commit06.


## Gate06 intermedi conclusi

Comandi0–3 tutti exit0/source_unchanged true: fmt,Clippy e due workspace
970pass/0fail/6ignored ciascuna. Runner foreground session51491 ora al comando4
core senza bundle, poi5release e6–10tooling/Python/boundary. Non mutare indice/
HEAD/sorgenti fino a complete=true. CAD04DONE, i documenti di chiusura attendono
commit separato dopo la sequenza. Nessun dispatchCI06 ancora effettuato.


## Gate06 tutti conclusi — sessione16104 terminata

11comandi tutti exit0/complete true,464hashfinali invariati.970/970workspace,
927core,18tooling,24Python. Primo8fallito per omissione LINTRA_TEST_BIN nel
runner: ledger/log originali preservati, retry8senza cambi prodotto poi9/10
positivi. Nessun processo di build attivo. Ora si può aggiornare indice/HEAD.
Report/status/inventario aggiornati. Pubblicare chiusura04e06con pathespliciti
escludendo docs/migration, dispatchCI06 e registrareSHA/runID. NonDONE06.


## STATO TERMINALE — CAD06 DONE

CI42372080602 success sullo SHA3719d14 esatto, settejob positivi e soli
packagingUI opzionali skipped.135/140DONE96,4%. Nessun codice06da riprendere.
CAD07attiva con compiler/engine/componenti/test modificati; usare sua recovery.
Documenti06da committare dopo il test mirato07 corrente, per non mutare HEAD
mentre build.rs è attivo. Nessun processo06 ancora in esecuzione.
