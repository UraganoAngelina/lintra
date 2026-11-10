# U-FIX-03 — Preflight soft e limiti hard separati

Data: 2026-11-10, Europe/Rome. Prerequisito U-GOV-03 `2cd4bdf`.
Stato: DONE per backend/CLI qualificati. Qualifica locale finale PASS su
`4864c344c06c54bb9cf2863426318ce2dceb255c`; CI nativa nuova qualificata nella chiusura seguente.
Nessun G-HARD PASS.

## Delta attuato

src/application/resource_budget.rs introduce ResourceBudget versionato e
privo di soglie predefinite; measurement_source obbligatoria, valori positivi
oppure null. I valori di prodotto non sono stati dedotti da DEC-14 o da stress
300k/1M. La fonte della configurazione va verificata con le misure effettive;
una stringa non è una prova di misura. Nessun default numerico inventato.

ValidationService::resource_preflight risolve il vero schema compilato e le
sue reference; legge gli input con buffer 64KiB, SHA e identità OS (dev/inode
Unix, volume/file-index Windows), conta byte con checked_add e conserva Missing
separato da un dataset verificato vuoto. Riusa il path checker reference
esistente; cambia soltanto la sua visibilità interna, non il comportamento.

PreparedResourcePreflight ha campi privati; accept consuma il candidato e
richiede una scelta esplicita se il soft budget è superato. L'oggetto accettato
ricontrolla richiesta, hash dello schema, cwd, root e fingerprint dei file prima
di validare. Sostituzioni con byte identici, mutazioni e nuove reference prima
mancanti invalidano la scelta. Contatori/overflow/I/O/file non regolari/symlink
e cancellazione non sono aggirati dall'override.

La CLI validate/validate-flow integra --resource-budget e --proceed-over-budget.
Senza configurazione preserva il percorso precedente. Warning prima di creare
output; errore conserva i report vecchi. La cancellazione resta controllata
exit130, anche durante il preflight o il ricontrollo. La scelta è tracciata con
soli contatori aggregati, senza path, digest, testo della fonte o identità.

La validazione reale conserva EOF/SHA/diagnostiche/Unknown/Partial/max_errors e
hashing propri; non usa il digest di preflight come prova di audit finale.
Non cambia RecordReader, writer JSONL, HMAC, snapshot, norme o golden legacy.
history preview non riceve questa opzione nuova; nessun flag di approval finto.
Il dialogo React si integra in MIG-P5-05, non viene dichiarato già esistente.

## Verifiche e limiti

Smoke finale dopo le correzioni: library all-features 145 pass/4 ignored,
CLI unit 25 pass, cli_full 7 pass, resource_budget 10 pass, zero fail. Nuovi
oracle: rifiuto senza scelta, EOF/SHA intero dopo override, stesse diagnostiche
Full/cross-trace e due audit, file identico sostituito, schema/request mutati,
Missing/reference/symlink, errori hard e cancellazione con vecchi report integri.
Clippy workspace all-targets/all-features positivo prima dell'ultimo raffinamento;
sarà ripetuto sul commit finale. I primi tentativi hanno corretto errori di
compilazione dei test e un'attesa errata di exit0 su fixture normativa invalida:
il test ora richiede exit2 e quattro violazioni, senza modificare il validator.

docs/RESOURCE_BUDGETS e README documentano semantica e costo delle letture
aggiuntive. Il modello è tree locale controllata: nessuna promessa anti-TOCTOU
ostile fra ricontrollo e apertura del validator, nessuna garanzia contro OOM OS.
UI e prove native Windows del nuovo codice non ancora qualificate.

File: resource_budget application/CLI, wiring mod/lib/args/command, visibilità
checker reference, test resource_budget/cli_full, documenti e stato unificato.
SHA del codice e log finali saranno collegati dopo commit; CI4c169cd non viene
attribuita a questo delta. Next: qualifica locale sul nuovo SHA, poi CI.

## Qualifica locale finale sul commit

U-FIX-03-workspace-linux.json e log.gz: workspace all-features exit0,
**1073 esiti passati / 125 riepiloghi / zero fail / 7 ignored**, incluse prove
Slint e doctest. Sono esiti dei riepiloghi (anche helper), non 1073 test unici.
U-FIX-03-fmt e U-FIX-03-clippy JSON/log.gz: exit0, workspace all-targets
all-features, warnings negati. Tutte le prove su 4864c34, 446 input inventariati
e invariati prima/dopo. Nessuna modifica ai golden per ottenere questi esiti.

La task passa in WAITING_GATE per CI nativa. Il dialogo React resta acceptance
della successiva integrazione MIG-P5-05, senza UI dichiarata già qualificata.
Next READY indipendente: U-FIX-05 (ignored e provenienza audit), poi U-FIX-04/OPS.

## Chiusura nativa backend/CLI

CI42829144043 completed/success su539b9bc, sorgenti del delta budget identici
a4864c34 per confronto Git. Sette job richiesti success, UI packaging opzionale
skipped. Metadata U-FIX-03-native-final, log integrale gzip e digest/check JSON.
Nuovi test overflow, sostituzione identica, CLI warning/override e Full
cross-trace effettivamente passati tre volte per OS, nei tre assetti richiesti.
G-HARD aggregato non chiuso: il delta successivo U-FIX-04 richiede la propria
CI. La futura integrazione del dialogo React resta task MIG-P5-05, non una
capacità UI già attestata da questi test. Nessuna pubblicazione artefatto/tag.
