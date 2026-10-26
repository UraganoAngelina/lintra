# T-P15-01 — Profilo release e configurazione

## Contratto e preflight

Task autorizzata dalla governance GOV-2026-10-26-01, applicata e pubblicata nel
commit cbcb750. Prerequisito T-P8-06 DONE, con CI nativa39747270986 verificata
nel dossier precedente. Consultati prompt operativo T-P15-01, ARCH§39B.1/3,
39C.1/4. Nessuna modifica ad ARCH, DET, norme o contratti runtime.

Obiettivo: thin LTO, una codegen unit, strip, CRT statico MSVC, path remapping,
build locked e verifica reale della riproducibilità in CI sui due target nativi.
Non è una distribuzione: baseline Linux DEC-11, artifact verifier15-02,
embedded bundle e packaging restano nelle rispettive task. Benchmark di confronto
rinviato esplicitamente a17-01, come previsto dal prompt.

## Configurazione e ragioni di correttezza

Cargo.toml configura il profilo release con lto="thin", codegen-units=1 e
strip=true. Nessuna modifica ai profili debug o alle dipendenze del prodotto.
.cargo/config.toml mantiene linker/runner GNU locali e aggiunge +crt-static
al target MSVC. L'alias cargo xtask compila il tooling con --locked; il suo
manifest autonomo evita di convertire prematuramente il prodotto in workspace.
Il tooling usa esclusivamente std e ha un lock proprio senza crate esterni.
La scelta Rust, anziché uno script dipendente dalla shell, rispetta ARCH§39B.3.

Il workflow conserva runner, toolchain, test e Clippy esistenti, aggiunge gate
di formato/Clippy/test del tooling e sostituisce la build release con due build
pulite e confronto completo dei byte. Ogni controllo è uno step separato:
un errore non viene mascherato dal comando successivo, anche su PowerShell.

## Spiegazione del tooling

- Options rappresenta target esplicito, offline e verifica ripetuta. options
  richiede build-release e un target supportato; selettori assenti, sconosciuti
  o duplicati sono errori prima di avviare Cargo. Non sceglie target impliciti.
- invalid produce errori tecnici InvalidInput. command propaga errori di spawn
  e qualsiasi exit status non positivo: nessuna build fallita diventa successo.
- inherited_flags preserva la precedenza dell'ambiente Cargo: encoded flags,
  poi RUSTFLAGS, poi flags del target. Non modifica l'ambiente globale.
- add_remap aggiunge forme lessicali e canonical, deduplicandole; questo copre
  alias 8.3 e path estesi Windows. I separatori unitari di Cargo preservano
  gli spazi. Path non Unicode producono un errore tecnico esplicito.
- build invoca Cargo nella root con --release --locked e target esplicito,
  aggiunge --offline solo se richiesto e usa una directory di output propria.
  Richiede che l'eseguibile atteso sia realmente presente.
- run individua root e home del toolchain senza sovrascrivere HOME o variabili
  di sistema. Prepara directory nuove sotto target/release-profile mediante
  create_dir: non cancella né riutilizza lavoro precedente. Il nonce serve solo
  a isolare i percorsi; non viene inserito deliberatamente nel prodotto.
  Remappa root, Cargo home, Rustup home e base degli output verso prefissi
  stabili. Le forme canonical coprono i percorsi reali della piattaforma.
  +crt-static viene aggiunto anche ai flags espliciti MSVC, perché i flags
  encoded hanno precedenza sulla configurazione del target.
- Entrambe le compilazioni ricevono gli stessi flags e sorgenti in directory
  inizialmente vuote. Il confronto fs::read riguarda tutto l'eseguibile: una
  differenza fallisce il gate. Non si confrontano soltanto dimensione o metadati.
  La verifica riguarda due build pulite dello stesso checkout/toolchain/runner;
  non promette identità tra sistemi operativi o toolchain diversi.
- main restituisce gli errori tecnici al chiamante. I test coprono selezione
  positiva/negativa, duplicati e conservazione degli spazi nei remapping.

Il percorso compile-time del manifest appartiene soltanto al tool di sviluppo,
non distribuito. Il prodotto viene compilato con i remapping. L'assenza di
dipendenze native vietate e stringhe host è oggetto del verifier T-P15-02:
non viene dichiarata verificata integralmente da questa sola ricetta.

## Evidenze effettive

26 ottobre2026, HEAD di partenza cbcb750. Due build Linux GNU pulite, offline,
locked, con toolchain1.99.1: prima2m36s, seconda2m41s; confronto byte positivo,
output reale `reproducible=true target=x86_64-unknown-linux-gnu`.
Log locale ignorato: target/verification-p15-01/reproducible-release.log.
Controllo governance PASS:51 prompt, modifiche operative reversibili; piano
fuori§14, ARCH/DET e kit originale invariati.

Gate locali tutti exit0: cargo fmt --all -- --check, fmt del manifest xtask,
cargo check --all-targets --offline --locked, cargo clippy --all-targets
--offline --locked -- -D warnings, cargo test --offline --locked (671 pass,
77 suite,0 fail,2 ignored già previsti), test xtask (2 pass) e check Windows
GNU --offline --locked. Clippy xtask all-targets -D warnings positivo.
Ambiente locale CARGO_HOME=/tmp/lintra-cargo-home, profili dev/test debug=0,
CARGO_INCREMENTAL=0, target di regressione riusato verification-p14-05; log
separati in target/verification-p15-01. La ricetta release usa directory proprie
nuove, non la cache di regressione. Nessun cambio al lock del prodotto.
CI nativa ancora pendente. Nessun DONE/M5 anticipato.

## Handoff provvisorio

Task T-P15-01 IN_PROGRESS. Ricetta e verifica Linux implementate. Restano
CI Linux/MSVC sul commit implementativo. Nessun
blocco contrattuale rilevato. Prossima candidata T-P15-02, dipendente dal gate
di questa task: non READY finché manca la verifica. Prosecuzione AUTO_CONTINUE.
