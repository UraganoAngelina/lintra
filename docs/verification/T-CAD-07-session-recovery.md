# Recovery CAD07 — 7 novembre 2026

Repo /home/alberto/Desktop/LINTRA/lintra, branch work/p1-foundations.
Single-agent, niente background/delega, commit/push/CI autorizzati e sviluppo
automatico sequenziale. Docs/migration preesistente untracked da escludere.

CAD04DONE su b59a5b9/CI42368062138. CAD06DONE su3719d14/CI42372080602,
sette job richiesti positivi; chiusuradocumentale a59ae04 pubblicata.
135/140DONE96,4%; aperte05/07/08/12/P18-04. Nessuna run pendente06.

CAD07implementata localmente, non pubblicata néDONE. Schema after_date e
capabilityesistente, condizioni locali anche attraverso derivati, dipendenze
esatte del consenso, resolverprivatoQualifiedTarget su tuple reali, errori
con campo ristretto e coordinate sorgente. Nessun first o filtro prima del
consenso. Scope inputcorrenti, niente snapshot/storia cumulativa abilitata.
Componente2024-06.conclusions.1 A/B1/C con13campi originali, source-check
positivo e unione sedi uguale alla policyfonte.11test nuovi+62regressione
positivi, Clippyworkspaceallfeatures positivo.

Primi test falliti su codicefacoltativoMissing perché una regolanegativa
normalizzata mancava della guardia di presenza: corretto il componente,
non il valutatore. Preliminari copiati in T-CAD-07/preliminary-*.log.
Report T-CAD-07-conclusions.md e authoring/README spiegano tutti gli helper
e i limiti. Nessuna APIapplicativa/CLI/UI, nuovo crate, HMAC/storage o norma.

## Gate finale

Avviare/attendere python3 /tmp/lintra-cad07-local-gates.py in foreground.
Ledger T-CAD-07-local-gates.json, log T-CAD-07/gate-N.log. Undici comandi
standard uguali alle precedenti task; CARGO_BUILD_JOBS=2registrata per ridurre
pressione memoria, tutti i test conservati. Comando8con LINTRA_TEST_BIN vero
eseguibile target/release/lintra appena costruito da5. Inputhashprima/dopo
verificati dopo ogni comando.

NON modificare src/tests/schemas/UI/authoring/.gitattributes/indiceGit/HEAD
durante la sequenza. Build.rs osserva indice/HEAD, evitare gitstatus/staging
finché non termina. Si possono aggiornare soltanto report/status fuorihash.
Nessun altro processo di lavoro avviato dall'agente è attivo. Non fare build
concorrenti. Il runner supporta resume con controllo impronte e preserva
loginterrotto: se sessione/file/tmp spariscono non attribuire esito al comando
interrotto; ricostruire il runner dai comandi e riprendere dopo quelli conclusi.

Dopo11exit0/complete/source_unchanged: verificare hash e conteggi dei log,
aggiornare report/inventario, commit/push07con pathespliciti, escludendo
docs/migration. CI completa Linux/MSVC sullo SHAfinale necessaria primaDONE07.
Durante sola attesa remota07, CAD05èREADY su04;08richiede07DONE. Nessun
consenso ordinario nuovo necessario, niente release/tag/chiavi operative.


Gateavviato in execforeground session82660, python3 /tmp/lintra-cad07-local-gates.py.
Leggere ledger per comando attuale; non modificare input/indice finché attivo.


## Gate07 intermedi

Session82660 ancora attiva: comandi0fmt,1Clippy e2workspace exit0,
source_unchanged true. Workspace981pass/0fail/6ignored. Ora3workspace
allfeatures, poi4core/5release/6–10tooling/Python/boundary. Nessun source
modificato durante i gate. Dopo fine verificare ledger/log/hash prima commit.


## Gate07 completati — session82660 terminata

11comandi tutti exit0/complete/source_unchanged true;468input finali invariati.
981/981workspace,938core,18tooling,24Python. Log/conteggi/inventario16log
verificati. Nessun processo di build attivo. Ora si può modificare indice/HEAD.
Report/status aggiornati, pubblicare07 con pathespliciti ed escludere
docs/migration. DispatchCI Linux/MSVC, registrareSHA/runID; nonDONE07.
Poi preflight05 indipendente. Nessuna release o codice privato/paziente.


## STATO TERMINALE — CAD07 DONE

CI42390593313success su6477bb2 esatto, settejob positivi.136/140DONE97,1%.
Nessun codice07da riprendere.05attiva in gate locale, leggere recovery05.
Chiusuradocumentale07 pronta da committare dopo la sequenza05 per conservare
indice/HEAD osservati da build.rs.08READY dopo05pubblicata/in attesa remota.
