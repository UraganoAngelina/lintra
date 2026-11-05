# Runbook di release Lintra

Procedura T-P18-03, 5 novembre 2026. Contratti: ARCH §39C.4–5, piano N-19/N-23,
[ratifica DEC-RELEASE-SETUP-01 A](verification/DEC-RELEASE-SETUP-01-ratification.md).
Workflow effettivo: `.github/workflows/release.yml`; controlli: `scripts/release.py`.
L’esecuzione della procedura si registra nel dossier di verifica con SHA, URL,
esiti e digest reali. Il semplice avvio di Actions non conclude una release.

## Identità e contenuto

Il repository sorgente è `UraganoAngelina/lintra`; gli eseguibili sono pubblicati
solo dalla CI in `UraganoAngelina/lintra-binaries`, privato e con immutabilità
abilitata. I bundle dati hanno repository e ciclo di vita separati: vedere
[aggiornamento schemi](RUNBOOK_SCHEMA_UPDATE.md). Non cambiare visibilità.

Usare una versione esplicita `MAJOR.MINOR.PATCH[-prerelease]` accettata dal tooling;
per una prova usare un suffisso come `0.1.1-rc.2`. Incrementare il suffisso a ogni
nuova prova. La versione deve coincidere con Cargo.toml, ui/Cargo.toml e le due
voci first-party in Cargo.lock e la dipendenza lintra in xtask/Cargo.lock. Il tag di destinazione è `v<versione>`.
Il commit sorgente completo è registrato in BUILD.json e in `lintra --version`.
Il tag del repository binari identifica il commit di quel repository, non lo SHA
del codice Rust: per la provenienza del codice usare BUILD.json e il link al
commit sorgente nella descrizione della release.

Una release executable contiene esattamente:

- `lintra-<versione>-x86_64-unknown-linux-gnu.tar.gz`;
- `lintra-<versione>-x86_64-pc-windows-msvc.zip`;
- `SHA256SUMS`, con i due digest degli archivi.

Ogni archivio contiene esattamente il binario nativo (`lintra`/`lintra.exe`),
BUILD.json e SHA256SUMS interno. Il workflow corrente pubblica solo la CLI;
Slint ha packaging separato e non è incluso implicitamente. Il bundle incorporato
è SYNTHETIC: nessuna chiave operativa, archivio o dato sanitario accompagna la prova.

## Preparazione del commit

1. Verificare branch e working tree. Preservare le modifiche estranee; non
   pubblicare pazienti, credenziali o file di provisioning. Scrivere note di
   versione in `docs/releases/<versione>.md`, incluse limitazioni e motivo di
   eventuale ripubblicazione. Il commit sorgente linkato dalla release le conserva.
2. Aggiornare i manifest, Cargo.lock e xtask/Cargo.lock coerentemente. Nei golden che includono
   la versione software cambiare solo quel metadata, verificando che ogni altro
   valore resti identico; non rigenerare baseline normative per un bump.
3. Eseguire fmt, Clippy e test pertinenti con `--locked`, compresi i test Python
   `python3 -m unittest discover -s scripts/tests -v`. Verificare anche il workspace separato del tooling con
   `cargo fmt --manifest-path xtask/Cargo.toml -- --check`,
   `cargo clippy --manifest-path xtask/Cargo.toml --all-targets --locked -- -D warnings`
   e `cargo test --manifest-path xtask/Cargo.toml --locked`. Registrare gli esiti.
4. Commit/push sul branch concordato `work/p1-foundations`. Salvare lo SHA esatto
   con `git rev-parse HEAD`. Avviare la CI nativa sullo stesso branch:

   ```sh
   gh workflow run ci.yml --repo UraganoAngelina/lintra --ref work/p1-foundations
   ```

5. Annotare la run restituita e leggerla con `gh run view <run-id> --repo
   UraganoAngelina/lintra --json status,conclusion,headSha,jobs,url`. Rispettare
   AGENTS.md: letture distanziate almeno 60 secondi, non più di tre ordinarie per run;
   diagnosi e lettura finale necessarie restano autorizzate. Richiedere
   completed/success sullo SHA preparato e tutti i job richiesti positivi.
   Il packaging UI opzionale può risultare skipped senza qualificare nuovi asset UI.

## Avvio della pubblicazione

Il secret sorgente `LINTRA_RELEASE_TOKEN` è un PAT dedicato al solo repository
binari, Contents write/Metadata read, scadenza 90 giorni. Lo configura il proprietario
in GitHub. Non copiarlo da credenziali locali, non stamparlo e non passarlo alle
build. `GITHUB_TOKEN` legge il gate del sorgente; il PAT serve negli step di
preflight/pubblicazione. Credenziale scaduta o non disponibile è un gate tecnico,
non autorizza un token alternativo o l’allargamento dei permessi.

Dopo il gate positivo e prima di nuovi push sul branch, avviare:

```sh
gh workflow run release.yml --repo UraganoAngelina/lintra \
  --ref work/p1-foundations -f version=0.1.1-rc.2
```

Sostituire la versione per la release effettiva; non ripetere questo esempio se
rc.2 è già riservata. Annotare subito la run e controllare il suo headSha. La
procedura Python rifiuta esecuzioni fuori Actions del repository sorgente e
versioni diverse dal package. Non simulare variabili Actions sul computer locale.
Il preflight richiede l’ultima CI sullo SHA esatto completed/success: una run
più recente fallita o pendente non viene nascosta da un successo precedente.

La CI verifica che tag e release, inclusi draft, non esistano. Costruisce Linux
su ubuntu-24.04 GNU e Windows su windows-2025 MSVC; applica i controlli nativi,
CRT statico Windows, allowlist dipendenze e due build riproducibili per target.
BUILD.json conserva toolchain, immagine runner, baseline glibc effettiva, SHA,
versione e digest. Linux non promette compatibilità con ogni glibc; Windows non
ha firma Authenticode, secondo ratifica. Non usare una build Arch come release.

## Pubblicazione e comportamento in caso di errore

Il publisher controlla di nuovo gate, versione e due archivi. Riserva atomicamente
il tag nel repository binari, crea un draft, carica tre asset e confronta nome,
dimensione, digest SHA256 e stato uploaded restituiti dal server. Controlla
l’inventario completo prima di pubblicare; dopo draft=false richiede
immutable=true e verifica nuovamente gli asset. Concurrency impedisce release
workflow simultanei; non cancellare un run in pubblicazione per avviarne un altro.

Un timeout o errore dopo una scrittura può avere lasciato tag, draft o release.
Prima di qualsiasi retry leggere lo stato remoto: non dedurre assenza da un
403, timeout o errore di rete. Una versione riservata non si riusa. Conservare
run/log/stato per diagnosi e correggere il problema in un nuovo commit e una
nuova versione, con una nota esplicativa. Non eliminare tag/draft per liberare il
nome; non modificare asset o rendere mutabile una release pubblicata. Se il
fallimento precede qualsiasi scrittura e l’assenza è realmente verificata, si
può riavviare la prova sullo stesso SHA/versione. Il publisher non esegue retry
automatici delle scritture ambigue né resume dei draft.

Una ripubblicazione anche di contenuto equivalente richiede una nuova versione
e note nel commit sorgente che identifichino precedente versione e motivo.
Un rollback operativo seleziona una precedente release verificata in una nuova
cartella, mantenendo identità e bundle dati espliciti. Non implica compatibilità
dello storage né autorizza una migrazione dello storico o sostituzione di chiavi.

## Verifica degli asset pubblicati

Richiedere tutti i job della release completed/success, headSha esatto,
repository privato, release non draft, tag corretto e immutable=true.
Scaricare in una directory nuova, senza sovrascrivere file preesistenti:

```sh
gh api repos/UraganoAngelina/lintra-binaries/releases/tags/v0.1.1-rc.2
mkdir /tmp/lintra-release-rc2-review
gh release download v0.1.1-rc.2 --repo UraganoAngelina/lintra-binaries \
  --dir /tmp/lintra-release-rc2-review
```

Controllare che l’inventario sia esattamente i tre asset sopra descritti. Per
ciascuno confrontare dimensione e SHA256 locale con size/digest/stato uploaded
nell’API. Confrontare SHA256SUMS esterno con i due archivi; su Linux eseguire
`sha256sum --check SHA256SUMS` dalla directory di download. Su Windows usare
`Get-FileHash -Algorithm SHA256` e confrontare i due digest. Il solo hash non è
una firma indipendente: usare repository e run autorizzati per la provenienza.

Prima di estrarre, importare `scripts/release.py` dal checkout sorgente verificato
e chiamare `inspect_archive(path, versione, sha_sorgente, target)` su entrambi gli
archivi. L’inspector rifiuta inventari inattesi, duplicati, link, dimensioni
anomale, identità o checksum diversi; restituisce BUILD.json verificato.
Richiedere anche reproducible=true e artifact_verified=true in entrambi gli audit.
Estrarre soltanto il binario già verificato in una cartella isolata, eseguire
`--version` sul proprio OS e confrontare versione/SHA. Un binario Windows non
eseguito su Linux resta qualificato dalla build/gate Windows, senza dichiarare
uno smoke locale inesistente.

Conservare nel dossier task: URL/SHA CI e release, metadati API senza segreti,
inventario/digest/dimensioni, audit degli archivi e output di --version. Non
versionare gli archivi. DONE richiede questi esiti effettivi, non solo test con
API simulate. La prova non chiude policy reali, normative CAD o milestone M9.

## Collaudo

Prima prova della catena: [T-P15-05](verification/T-P15-05-executable-release.md),
release privata rc.1, immutabile e verificata. Collaudo di questo runbook con nuova
versione: [T-P18-03](verification/T-P18-03-release-runbook.md),
[note rc.2](releases/0.1.1-rc.2.md). Consultare il dossier per lo stato reale;
questo documento non anticipa l’esito della nuova pubblicazione.
