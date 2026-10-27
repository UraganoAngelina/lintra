# T-P15-05 — Workflow di release dell’eseguibile

## Preflight e ratifica

Prerequisito tecnico15-02 verificato: CI40329791116 su59b1663 Linux/MSVC success.
15-03/04 DONE suCI40339655671/40343261368. DEC-RELEASE-SETUP-01 A approvata
esplicitamente dall’autore: [ratifica](DEC-RELEASE-SETUP-01-ratification.md).
Sono ora creati e verificati privati UraganoAngelina/lintra-binaries e lintra-data,
con bootstrap README/main, senza modificare il sorgente pubblico. Timeout TCP
risolti verificando l’esistenza prima di riprovare, nessuna creazione duplicata.
Immutabilità GitHub nativa abilitata e verificata su entrambi con GET/PUT
/repos/OWNER/REPO/immutable-releases. La lettura può restituire200/ enabled=false,
non soltanto404: gestito il valore effettivo prima dell’abilitazione.
Nessun PAT locale copiato nei secret; configurazione dedicata richiesta all’autore
con domanda asincrona, mentre prosegue il lavoro indipendente dalla credenziale.

## Perimetro e versione

Workflow release.yml, tooling Python stdlib con test isolati, pipeline CI,
versione Cargo/lock e soli metadata tecnici di tre golden, documentazione/status.
Nessun nuovo crate, provider, norma, formato schema o modifica engine/service.
Prima prerelease prevista **0.1.1-rc.1**: mantiene una versione SemVer successiva
alla baseline compiler0.1.0 già dichiarata, con tag/versione binario coincidenti.
I tre golden SIOC sintetici cambiano esclusivamente validator_version; gli altri
byte e la baseline normativa/schema/digest restano invariati.0.1.0-rc.1 valutata
inizialmente non viene pubblicata o mantenuta perché precedente alla baseline.

## Workflow e codice: comportamento e motivi di correttezza

- workflow_dispatch richiede versione esplicita; niente pubblicazione su push/PR.
  Concurrency globale senza cancellazione impedisce sovrapposizioni dello stesso
  publisher. Input passa tramite env e argomenti, senza interpolazione shell diretta.
- preflight usa solo Ubuntu: controlla la credenziale richiesta, repository privato
  esatto, tag/release/draft assenti e CI sviluppo più recente completed/success sul medesimo SHA.
  Risparmia build native quando manca un prerequisito. GITHUB_TOKEN sorgente ha
  contents/actions read; PAT separato dedicato al solo repository binari.
- job build nativi GNU ubuntu-24.04/MSVC windows-2025 non ricevono PAT. Riutilizzano
  xtask con --verify-reproducible --verify-artifact: due build pulite, allowlist,
  path host/openssl e negative probe restano effettivi. Niente test di sviluppo
  ripetuti nel job Windows release: il gate completo del medesimo SHA è obbligatorio.
- identity valida versione portabile/SHA40hex e uguaglianza con Cargo.toml.
  require_ci verifica il contesto del repository e SHA Actions: è un controllo
  operativo, non una promessa di attestazione crittografica contro env contraffatti.
  L’agente non invoca pubblicazioni locali e non inietta la propria credenziale.
- GitHub incapsula HTTP: HTTPS/host/repository fissati, token solo in memoria/header,
  nessun redirect con Authorization, errori senza body o credenziali. Nessun retry
  automatico di scritture ambigue.403/timeout non diventano disponibilità/successi.
- unused_version controlla tag e tutte le pagine di release, inclusi draft. Il
  publish riserva atomicamente il nuovo ref prima del draft; una collisione fallisce.
  Se una scrittura fallisce, tag/draft restano per diagnosi e non vengono rimossi,
  sovrascritti o ripresi implicitamente. Una nuova versione è necessaria.
- build confronta --version del binario con versione e SHA richiesti dopo i gate
  reali e registra Rust/Python/runner/immagine/glibc effettiva. Non inventa una
  baseline glibc minima universale: uso Linux interno/personale ratificato.
- pack produce Windows ZIP Stored o Linux tar.gz con nomi, timestamp, UID/GID e
  permessi fissi; contiene solo binario, BUILD.json e SHA256SUMS. Output nuovo
  esclusivo; errori non sono caricati dal workflow. Dati incorporati sintetici.
- inspect_archive accetta soltanto inventario esatto/tre file, niente duplicati,
  path extra, link tar/ZIP o file eccessivi; verifica hash binario e metadata,
  versione/SHA/target e checksum interni. Non estrae contenuti non fidati.
- publish ricontrolla preflight dopo le build; accetta esattamente i due archivi,
  verifica audit e crea SHA256SUMS esterno. Crea prima un draft, controlla digest
  SHA256/size/state restituiti per ogni upload e inventario server completo.
  Solo allora draft=false; verifica inoltre immutable=true del risultato con
  letture brevi limitate per l’eventuale propagazione server. Un mancato riscontro
  resta errore, senza cancellare/ripubblicare una versione ormai esistente.
- Actions checkout/setup-python/upload/download sono fissate a SHA ufficiali
  verificati via refs GitHub: checkoutv7.0.1 già qualificata; setup-pythonv7.1.0,
  uploadv7.0.2, downloadv8.0.2. Python3.13 esplicito; nessun pip/runtime extra.
  PAT solo negli step Python che ne hanno bisogno, mai negli step delle action.

## Verifiche e residui correnti

12 test Python passati localmente: contesto/versione, tag/draft, byte deterministici,
checksums/provenienza/inventario, no-clobber, ordine draft-upload-publish,
digest errati/upload fallito senza publish, immutabilità finale obbligatoria,
asset mancanti/extra prima di scritture, boundary HTTP/redirect/no log segreti,
CI sorgente pendente rifiutata, CI più recente fallita non nascosta da un vecchio
successo e collisione della prenotazione tag senza creazione di release. Mock API e payload sintetici sono dichiaratamente
test del protocollo: nessuna release reale creata dai test.

Gate locali Rust/tooling dopo bump tutti superati: 681 test default, 689 con
feature, 15 tooling; fmt/clippy e actionlint superati. Il secret dedicato è
confermato dall’autore e il nome risulta presente via API Actions, senza lettura
del valore. Restano CI Linux/MSVC sul commit finale e prova release solo CI. Nessun DONE15-05 o M5 presunto.

## Gate nativo verificato

2026-10-26 15:43:17UTC: CI40349711927 sul commit
bbfdb93243e8f316940b73d9aec8920a625ce7ee completed/success,
Linux121666661900 e MSVC121666661423, tutti18step per job success.
Evidenza strutturata T-P15-05-native-ci.json. Avviata la prerelease0.1.1-rc.1
via release.yml; pubblicazione effettiva ancora pendente, quindi task non DONE.

## Chiusura verificata — recovery 2026-10-27

Stato **DONE**. Evidenza strutturata: [T-P15-05-published-release.json](T-P15-05-published-release.json).
Pubblicazione CI completata con successo, release privata non draft e immutable=true.
Scaricati tutti gli asset pubblicati; inventario esatto, digest SHA256, dimensioni
e stato uploaded confrontati con API GitHub. SHA256SUMS esterno verificato;
entrambi gli archivi nativi verificati con inspect_archive: inventario, checksum interni, versione/SHA/target/audit; binario Linux scaricato eseguito con --version e risultato corretto. Windows qualificato in CI MSVC, non eseguito su host Linux.
Test protocollo ripetuti: 12/12 positivi. Nessun codice modificato;
Rust non rieseguito, gate già disponibili sugli SHA implementativi invariati.
Nessun gate pendente della task. M5 resta aperta fino a T-P15-07.
Handoff completo in IMPLEMENTATION_STATUS.md; candidata15-07 non avviata,
secondo arresto alla conclusione delle attività aperte richiesto nella recovery.
