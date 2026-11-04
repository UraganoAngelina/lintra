# T-PERSIST-11 — concorrenza, recovery e file fra sistemi operativi

31 ottobre 2026. Base codice09 `4313f22`, documentazione `c49631b`.
Task09 sospesa su CI41194045486; task10 sospesa sulla scelta del provider,
proposta PERSIST-KEY-PROVISIONING-01 sottoposta all'autore. Nessuna risposta
presunta. Task11 è indipendente da entrambe.

Prerequisiti verificati dai dossier nativi:05 `3712ed6`, CI40844540287 success;
07 `1e92b1f`, CI40877202683 success;08 `bc7c448`, CI41069605562 success.
Fonti: prompt11, ADR001/ADR002, ARCH45, DET166–174 e implementazione adapter/
test/coordinatore/audit. Nessun nuovo override operativo, single-agent.

## Coverage esistente e aggiunte dichiarate

PERSIST05/07/08 già verificano rollback journal reale, uscita nel commit hook
con pagine versate sul disco, uscita dopo commit, risposta persa, CAS, due
processi, lock SHARED/RESERVED, corruzione di source/record/indici, metadati e
immutabilità. Non duplicare queste prove con equivalenti artificiali.

Aggiungere un writer figlio fermato dopo stage oppure dopo INSERT non committati;
un reader indipendente deve vedere soltanto la baseline completa o un errore
di lock esplicito. Uccidere il figlio e verificare recovery, snapshot, generazione,
submission e lookup. Guardia RAII del figlio e deadline finite evitano processi
orfani anche quando un'asserzione fallisce.

Aggiungere fixture di trasferimento **solo sintetica**, con provider pubblico
fisso, due namespace e più revisioni/identità conservate. Esportare DB chiuso
e manifest con hash; ciascun runner deve riaprire il DB prodotto dall'altro.
L'import test usa un nuovo contenitore privato creato dal backend, così i permessi
del download non attenuano i controlli dell'archivio. Non è una API di restore.
Input malformato/hash errato/chiave errata devono fallire senza reset dei file.

File candidati: `src/persistence/sqlite/tests.rs`, nuovo modulo di qualificazione
privato dei test e `.github/workflows/ci.yml`; documentazione/status/indice.
Hook eventuali solo `cfg(test)`. Nessuna modifica SQL produttiva, schema, chiave
reale, nuova dipendenza, contratto normativo o UI Slint.

Gate previsti: test mirati e helper reali, fmt/Clippy/workspace due assetti/core/
boundary; actionlint e CI nativa con scambio in entrambe le direzioni. La nuova
CI attende il termine di09 per non cancellarla o duplicare pipeline. Nessun DONE
prima delle evidenze effettive, nessun busy polling remoto.
