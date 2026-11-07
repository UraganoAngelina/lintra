# Recovery operativa CAD04 — 6 novembre 2026

Workflow attivo: proseguire automaticamente, single-agent, niente background,
subagenti o conferme ordinarie. Repository /home/alberto/Desktop/LINTRA/lintra,
branch work/p1-foundations, remote UraganoAngelina/lintra. HEADa98d4e7 pubblicato.
La cartella docs/migration è preesistente, untracked, da preservare ed escludere.

133/140DONE(95%),7aperte: CAD04–08/12 eP18-04. CAD03/11 chiuse sulla
CI42203885693/6af5f85; CAD09 chiusa sullaCI42231182241/1740a6d. Tutti7job richiesti
positivi per ciascun gate. Le evidenze e la chiusura09 sono pubblicate ina98d4e7.

CAD04 implementata localmente, nonDONE e non pubblicata. Piani compilati per
invarianti, proiettore opaco dalla stessa riga, wrapper di approvazione con
verificatore in prestito, guardia di contesto/versione/completezza prima degli
indici, rifiuto normativo con origine reale; audit esplicito sul progressivo.
Formato storage/HMAC invariati: riuso dei key_kind e degli StoredIdentity v1.
Componente progressives.1 con20campi originali/PK complete, niente schemaCAD
completo oFROZEN. Dettagli e limiti in T-CAD-04-progressives.md/preflight.md.

13nuovi test e57esiti di regressione mirata passati,0fail/0ignored. Include
SQLite/restart T1/T2/T3, C/D PK mutabile, riassegnazione prima/dopo annullamento,
namespace/trace/anni, fileassenti, chiavi/versioni/indici incompleti, CAS/report/
cancel, origine/JSONL/privacy, retry e rinomina generica. Clippyworkspaceallfeatures
positivo. FontePDF,5tracce,20campi ePK verificati nel source-check04.

## Gate finale corrente

Processo foreground exec_command session56674: python3 /tmp/lintra-cad04-local-gates.py.
Ledger docs/verification/T-CAD-04-local-gates.json; log T-CAD-04/gate-N.log.
Comandi0fmt e1Clippy conclusi exit0/source_unchanged:true; comando2workspace
in compilazione all'ultima lettura. Seguono3workspaceallfeatures,4corecompleto
senza bundle,5release,6fmttooling,7clippytooling,8toolingincludeignored,
9Python,10boundary. Il runner confronta tutte le impronte dopo OGNI comando.

Non modificare sorgenti, .gitattributes, authoring o indiceGit durante i gate.
Evitare git status/commit finché non termina: build.rs osserva l'indice/HEAD.
Sono possibili aggiornamenti dei soli report di verifica/status esclusi dagli hash.
Poll con write_stdin, attese entro60secondi e aggiornamenti utente regolari.
Se la sessione non esiste più dopo una ripresa, leggere ledger/active_command e
rilanciare lo stesso script: conserva il loginterrotto e riparte dai comandi
conclusi, solo se tutti gli input sono identici. Non attribuire un exitstatus
al comando interrotto. Se un gate fallisce o cambiano i sorgenti, preservare
ledger/log della vecchia sequenza e fare una nuova qualificazione dopo il fix.

I log preliminari (inclusi fallimenti di fixture e loro correzioni) sono già
copiati in T-CAD-04/preliminary-*.log. Nessuna regressione è stata nascosta.
Non avviare build concorrenti. Nessun altro processo locale di lavoro è attivo.

## Prossimi passi autorizzati

1. Finire tutti11gate e verificare complete/source_unchanged veri.
2. Aggiornare report/inventariohash e pubblicare codice04 con percorsi espliciti;
   escludere docs/migration e ogni artefatto target. Solo logevidenza in docs.
3. Push e dispatchCIci.yml sul branch concordato, packagingUI opzionale falso.
   Nessuna conferma necessaria. Registrare SHAesatto/runID e leggere gli esiti.
   Pianificare max3letture ordinarie distanziate>=60s; logdiagnostici/letturafinale
   mancante autorizzati daAGENTS. NessunDONE prima dei7job richiesti positivi.
4. Durante la sola attesa remota sono READY le indipendenti CAD06/07 su02/P10/P13.
   CAD05 richiede04DONE: non anticiparne il prerequisito. Dopo04DONE aggiornare
   a134/140,6aperte e proseguire automaticamente secondo dipendenze.
5. Nessuna decisione normativa/criptografica nuova, chiave operativa, release/tag
   o refreeze. Il mandato è continuare: un handoff non chiude la sessione.

## Affinamento obbligatorio successivo ai gate della base

Individuato caso non ancora coperto: stesso progressivo su eventi diversi nello
STESSO candidato. Oggi Snapshot::new lo rifiuta come DuplicateIdentity tecnico;
serve il ValidationError con codice dichiarato prima di costruire lo snapshot.
Non modificare gli input della suite in corso. Il test normativo è pronto in
/tmp/lintra-cad04-current-binding-test.rs; aggiungerlo dopo la chiusura dei gate
base e riprodurne prima il fallimento. Poi correggere validated_with_identity_records:
controllo binding correnti, autentica i token prima di confrontarli, namespace/
profilo coerenti, errore normativo con seconda riga reale se evento differente.
Il caso duplicato sullo stesso evento resta strutturalmente ambiguo e rifiutato.
Riutilizzare la costruzione del diagnostico del wrapper storico, senza valori
sensibili. Il tracking usa solo identità protette e viene rilasciato prima dello
snapshot. Ripetere regressione/core/Clippy/release pertinenti e CI completa sullo
SHA finale. Distinguerne esplicitamente il delta dai gate della base congelata;
non marcare04DONE né pubblicare la base incompleta.13test esistenti/57regressione
sono positivi ma non coprono ancora questo quattordicesimo caso.

## STATO SUCCESSIVO — affinamento applicato, 15 test verdi

La sequenza BASE è completa:11comandi tutti0, hash invariati;957workspace,
957allfeatures,914core,18tooling,24Python. Ledger etichettato come base PRIMA
dell'affinamento. Non applicare nuovamente i draft nel/tmp: sono già stati usati.
Il test aggiuntivo ha fallito prima del fix (current-before-fix.log); poi è stato
corretto il candidato corrente con check_current_bindings e diagnostico condiviso.
La mappa prende in prestito identità esistenti, controlla autenticità su collisione
ed è rilasciata prima dello snapshot. Namespace/flow/trace e conteggi sono verificati
prima della comparazione. Il caso stesso evento duplicato resta rifiutato.

Aggiunta anche la precedenza degli errori tecnici per token non autenticabili e
conteggi incoerenti. Tutti15testcad_progressives passati in
/tmp/lintra-cad04-final-targeted.log. Sourceauthoring e report aggiornati.

Processo foreground CORRENTE: session98567,
python3 /tmp/lintra-cad04-final-refinement-gates.py.
Ledger T-CAD-04-final-refinement-gates.json; log T-CAD-04-final-refinement/gate-N.log.
Sei comandi:0fmt,1Clippyworkspaceallfeatures,2coreinteroallfeatures,
3coreinterosenza bundle,4release,5boundary. Impronteprima/dopo controllate;
non modificare sorgenti/authoring/indiceGit durante questa sequenza. La nuovaCI
sul commit definitivo dovrà ripetere l'intero workspace Linux/MSVC, inclusaUI.

Attendere questi gate e complete=true. Poi copiare i log del red/green e gli
ultimi tentativi dal/tmp nei dossier, inventariohash, report/status fedeli.
Pubblicare codice04 SOLO nella versione corretta, CI nuova obbligatoria.
Ancora133/140DONE,7aperte;04IN_PROGRESS. Nessun altro processo locale attivo.
Il runner di affinamento non ha resume automatico: se la sessione scompare,
leggere il ledger, preservare i log già conclusi e continuare solo i comandi
mancanti con gli stessi hash; non riscrivere come verde un comando interrotto.


## Ripresa 7 novembre — gate finali conclusi

I sei gate finali sono tutti exit0/complete true; tutte le impronte finali
coincidono. Log test e inventario base verificati, inventario finale aggiunto.
924core allfeatures/916senza bundle,0fail/6ignored;15testCAD04. Nessuna build
locale ancora attiva. I temporanei /tmp red/green non sono più disponibili:
non ricostruirli né dichiararli conservati. Pubblicare il codice qualificato,
poi CI completa sullo SHA finale; attendere sette job prima di DONE.


## STATO TERMINALE — CAD04 DONE, 7 novembre 2026

CI42368062138 success sullo SHA b59a5b905ae9f82baf5ce23bbc4eb8564fa5cd22,
sette job richiesti positivi, risultato completo nel native-ci.json.
134/140DONE95,7%. Nessuna nuova implementazione04 da riprendere. CAD06 attiva
in test locale, leggere la sua recovery; documenti04 da committare dopo i gate06
per preservare indice/HEAD durante la qualificazione corrente.
