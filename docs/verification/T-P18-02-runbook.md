# T-P18-02 — runbook di aggiornamento dati

## Prerequisiti e contratti

M5 generica verificata con15-07/CI40475046094 e release private15-05/06;
nessuna certificazione normativa SIOC/CAD implicita. ARCH39C.2, DET107/155,
authoring/manifest/capability esistenti applicati, nessuna nuova semantica.

## File e comportamento

docs/RUNBOOK_SCHEMA_UPDATE.md copre fonte/revisione, formato/capability,
fixture/golden, manifest/filehash vs fingerprint canonico, bundler, release
privata lintra-data e verifica asset, installazione completa/rollback e audit.
Nessuna sovrascrittura di baseline o promozione automatica; nessuna pubblicazione
GitHub reale nel dry-run. La procedura reale riusa il workflow15-06 qualificato.

scripts/schema_update_dryrun.py è il comando ripetibile di accettazione.
`dryrun` richiede binario esistente e directory nuova, copia il binario con
permessi in una directory con spazi e isola la cwd/variabili bundle. `call`
controlla ogni exit e registra il comando reale; `validate` confronta risultati
con aspettative indipendenti (un record, zero/un errore, valido/invalido e fonte).
La fixture originale rimane invariata: copia solo payload manifestati, crea
v2 con dominio sintetico dichiarato e ricalcola SHA256 dei byte. Il linter reale
verifica la nuova versione, Rust xtask confeziona solo dati; inventario ZIP,
manifest e hash sono verificati prima dell’estrazione della fixture locale.
Confronta v1 JSONL esatto e hash schema, v2 valido/invalido e JSONL ripetuto;
SHA256 della copia e binario sorgente devono restare uguali. `save` scrive UTF-8
senza BOM/newline controllato. Non è un installer per ZIP esterni non fidati.
Nessun comportamento prodotto modificato o PHI introdotto.

## Gate reali e chiusura

Due dry-run end-to-end Linux sul binario release source59ba771/0.1.1-rc.1:
lint/bundler veri, sei validazioni ciascuno, ZIP identico fra run; vecchi errori
identici e nuova versione operativa senza ricompilare CLI. Negativo: workdir
già esistente rifiutata, evidence precedente invariata.19test Python tooling,
py_compile e fmt positivi. JSON di evidenza con comandi/exit/hash archiviato.
Nessun test Windows del dry-run dichiarato: la task richiede il dry-run,
non un nuovo gate di esecuzione nativa su entrambi i sistemi. Prova effettuata
su Linux; CLI e bundler restano coperti dalla CI nativa già qualificata.
Nessun Rust modificato: Clippy/test Rust non ripetuti per documentazione e
script stdlib, restano validi i gate81/82suite delle task precedenti.
T-P18-02 DONE. Nessun gate remoto dedicato richiesto per questa consegna.

## Handoff

Task e script completi, nessuna pubblicazione test remota. Rimane la release
reale di ogni futuro aggiornamento normativo, con proprie approvazioni/gate.
UI16-01 sospesa su CI40513168784, prima lettura in_progress; next16-02 dopo
esito positivo. AUTO_CONTINUE, nessuna conferma ordinaria.
