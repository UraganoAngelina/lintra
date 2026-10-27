# Aggiornamento di uno schema senza ricompilare Lintra

Questa procedura separa la revisione del bundle dati dalla release del binario.
Vale quando formato e capability richiesti sono realmente supportati. Un lint
positivo attesta eseguibilità tecnica, non completezza o conformità normativa.
Riferimenti: ARCH §39C.2, DET §107/155, [authoring](SCHEMA_AUTHORING.md).

## Preparare una revisione

1. Acquisire la fonte ufficiale e registrarne versione, data, provenienza e licenza.
   Confrontare campi, posizioni byte, PK, domini, presenza, regole, join e dataset.
   Una nota discorsiva non diventa automaticamente una regola dell’engine.
2. Copiare lo schema in `schemas/<FLOW>/<NUOVA_VERSIONE>/flow.json`, dichiarando
   esattamente flow/versione e versione normativa. Non riscrivere una baseline
   FROZEN, un’identità già distribuita o una decisione storica.
3. Conservare il formato se il contratto resta lo stesso; dichiarare tutte le
   capability effettivamente necessarie. Non rimuoverle per superare il compiler.
   Se un formato/costrutto non è supportato, fermare questa distribuzione dati:
   serve una nuova capacità del motore con propri gate, non un fallback permissivo.
4. Aggiornare i reference dataset tramite il loro formato e identificativi
   versionati. Conservare le dipendenze delle versioni precedenti che si intendono
   mantenere nel bundle. Le cartelle esterne sostituiscono l’intero snapshot:
   non vengono mescolate con i file embedded mancanti.

## Lint e fixture

Eseguire con l’eseguibile destinato agli operatori, indicando percorsi assoluti
oppure percorsi relativi alla shell corrente. Esempio sintetico:

```sh
lintra schema lint --flow SYNTHETIC --schema-version v2 --schema-dir candidate/schemas --encoding utf8
```

Preparare fixture esclusivamente sintetiche: almeno un record valido e uno
invalido per ogni vincolo cambiato, casi byte/encoding e relazioni pertinenti.
Definire risultati attesi indipendentemente dall’output corrente. Per i golden
confrontare JSONL byte per byte; non sostituire gli attesi con il nuovo output
per far passare un test. Per versioni conservate, verificare anche le regressioni.
Full e LocalOnly hanno scope distinti: il successo locale non prova correlazioni
o capacità storiche. Il profilo di persistenza dichiarativo non abilita un backend.

## Manifest e confezionamento

Assegnare un `bundle_version` nuovo, distinto dalla versione dell’eseguibile.
`MANIFEST.json` contiene `bundle_version`, `schema_format_version` e `files`:
mappa completa dei payload verso SHA-256 dei **byte dei file**. Questo digest
non coincide necessariamente con `schema_hash` canonico dei report. Non includere
il manifest stesso nella sua mappa. Calcolare nuovamente gli hash dopo ogni edit.
Percorsi relativi portabili, nessun symlink o segreto; dataset in `reference-data/`.

Dal checkout del tooling qualificato, con directory di destinazione nuova:

```sh
mkdir dist-update
cargo xtask bundle-data --input candidate --output-dir dist-update
```

Il bundler verifica i digest dichiarati e produce `schemas-<bundle_version>.zip`
con manifest e `SHA256SUMS` interno. Verificare inventario completo e hash di
ogni payload prima di installare/distribuire. Un checksum prova integrità dei
byte, non la validità della fonte normativa. Non sovrascrivere versioni/archivi.

## Release e installazione

La pubblicazione reale avviene nel repository privato `UraganoAngelina/lintra-data`:
committare il bundle revisionato e avviare **Release data bundle** con l’esatto
`bundle_version` del manifest. Il workflow usa il bundler qualificato, verifica
l’inventario, pubblica una prerelease immutabile e controlla digest, dimensioni,
stato e asset. Non ricompila né ripubblica l’eseguibile. Conservare SHA sorgente,
run CI, tag e verifiche degli asset come evidenza. Un tag prenotato o un tentativo
fallito richiede una nuova versione; non cancellare o riutilizzare la precedente.

Installare solo dopo verifiche e a validazioni ferme: preparare la nuova cartella
completa e verificata, conservare la precedente per rollback e sostituire l’intero
bundle. Su Windows chiudere l’applicazione prima della sostituzione. Non copiare
pezzi del nuovo bundle sopra una cartella usata da una validazione in corso.
Non è una transazione automatica dell’app: il lifecycle persistente è separato.

La risoluzione è: opzioni esplicite, variabili `LINTRA_SCHEMA_DIR` e
`LINTRA_REFERENCE_DATA_DIR`, cartelle accanto all’eseguibile, snapshot embedded.
La working directory non è una fonte implicita. Per installazione accanto al
binario estrarre il bundle completo, inclusi manifest, `schemas/` e
`reference-data/`. Un esterno presente ma incompleto/errato è errore tecnico:
non si torna silenziosamente all’embedded e non si sceglie una versione “latest”.

Provare `--version`, lint e validazioni della versione esplicita. Nei summary
controllare `bundle_source=external`, `bundle_version` e `schema_hash` attesi.
Exit0 = valido nello scope richiesto,2 = invalidità normativa,1 = errore tecnico,
130 = cancellazione. Non confondere l’errore tecnico con un report conforme.
Usare `--errors`, `--summary` e `--xlsx` per i file di output; evitare `>` in
PowerShell5.1, che può cambiare encoding. I record sono byte-oriented; usare
UTF-8 senza BOM e terminatori LF/CRLF, preservando la larghezza dei record.

Se l’aggiornamento va ritirato, ripristinare un bundle completo precedente
identificato e verificarne audit/versioni; l’embedded non è un canale di rollback
automatico. Nessuna migrazione/riscrittura di eventuali sessioni persistenti.

## Dry-run ripetibile, senza pubblicazione remota

Serve Python3 e Cargo per il tooling; il binario deve essere una release nativa
con feature `embedded-bundle`, contenente il bundle dimostrativo SYNTHETIC/v1.
Il seguente comando funziona dalla root del repository; sostituire il percorso
binario con quello della release scaricata. Su Windows usare `python` e `.exe`.

```sh
python3 scripts/schema_update_dryrun.py --binary /percorso/lintra --work-dir target/schema-update-prova
```

Il percorso di lavoro deve essere nuovo. Lo script crea una copia del binario in
una directory con spazi e usa una cwd diversa, senza override d’ambiente del bundle.
Verifica v1 embedded, prepara un candidato v2 che ammette un secondo valore,
esegue il lint reale, confeziona con il Rust bundler, verifica ZIP/manifest/hash,
installa accanto alla copia e prova v1/v2. Verifica i risultati validi/invalidi,
l’identità dei vecchi JSONL e il determinismo dei nuovi errori; non ricompila la
CLI e ne confronta l’hash prima/dopo. Restano fixture, report ed `evidence.json`.
Il file ZIP è la prova della fase di confezionamento della release dati; il
dry-run non crea tag né pubblica release GitHub e non è una certificazione regionale.

Esecuzione verificata: [evidenza](verification/T-P18-02-dryrun.json). Fonte binario
59ba771, Linux nativo, versione0.1.1-rc.1. Nessuna prova Windows del dry-run
dichiarata; i contratti CLI/packaging restano qualificati dalle CI native del progetto.
