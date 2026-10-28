# DEC-15 — Registro dipendenze aggiuntive

T-P5-05, 2026-10-20; CNF-AUDIT-01 approvata con precisazioni dall’autore.
Registrazione prima dell’aggiunta a Cargo.toml; verifica concreta del lock
e dei target Linux GNU / Windows MSVC nelle evidenze della task.

| Crate | Versione fissata | Scopo | Licenza | Pure Rust / configurazione |
|---|---|---|---|---|
| sha2 (default-features=false) | 0.11.1 | SHA-256 incrementale dei byte originali dell’input | MIT OR Apache-2.0 | RustCrypto, nessuna libreria crypto dell’host; backend software disponibile su entrambi i target |
| chrono | 0.4.45 | Solo formattazione timestamp UTC RFC3339 | MIT OR Apache-2.0 | default-features=false, std; niente clock locale, timezone database, libc o dipendenze Windows abilitate per il clock |

Fonti primarie consultate: [sha2 0.11.1](https://docs.rs/crate/sha2/0.11.1),
[API incrementale](https://docs.rs/sha2/0.11.1/sha2/),
[chrono 0.4.45](https://docs.rs/crate/chrono/0.4.45); per chrono verificati
anche Cargo.toml e licenza della copia locale del crate.
Nessun algoritmo crypto o calendario riscritto nel core.

Cargo.toml sha2 locale: MSRV 1.88, compatibile con Rust 1.99.1,
nessun build script. Default oid/alloc disabilitati perché non necessari
a SHA-256 incrementale. cpufeatures è Rust; la presenza di libc nel lock
non implica una dipendenza attiva sui target x86_64: verificare con cargo tree.

Verifica locale effettiva: cargo tree --locked su x86_64-unknown-linux-gnu
e x86_64-pc-windows-msvc non abilita libc; chrono ha solo std/alloc.
Controllati anche MIT OR Apache-2.0 e MSRV di cpufeatures, digest,
block-buffer, crypto-common, hybrid-array, typenum, num-traits, autocfg, cfg-if.
Tutti compatibili con la toolchain 1.99.1; esito native CI nel report T-P5-05.

Compatibilità nativa effettivamente verificata dalla [CI 39399359949](https://github.com/UraganoAngelina/lintra/actions/runs/39399359949)
su 85f1ac7: Linux x86_64 GNU e Windows x86_64 MSVC entrambi success,
con test/build release; oltre al check locale Windows GNU all-targets.

## T-P8-01 — clap, dipendenza nominata dal piano

clap =4.6.7, MIT OR Apache-2.0, MSRV 1.85: compatibile con pin 1.99.1.
Fonti primarie: https://docs.rs/clap/4.6.7/clap/ e Cargo.toml locale della
versione scaricata con cargo info clap@4.6.7 (licenza/MSRV/feature verificati).
Solo std/help/usage/error-context, default disabilitati: niente derive, color,
env o suggestions. EnvProvider rimane una porta esplicita testabile.
Nessun import clap nel core: è usato esclusivamente dal target binario CLI.
Lock e dipendenze transitive/target verificati nei gate T-P8-01. Nessun nuovo
crate di test. clap è già nominato da ARCH §4/Piano T-P8-01, nessuna nuova
scelta di framework o deroga DEC-15 richiesta.

Lock T-P8-01: clap_builder 4.6.7 (MIT OR Apache-2.0, MSRV 1.85),
clap_lex 1.1.1 (MIT OR Apache-2.0, MSRV 1.85), anstyle 1.0.14
(MIT OR Apache-2.0, MSRV 1.66). Metadati verificati dai Cargo.toml scaricati.
Cargo tree --locked su Linux GNU e Windows MSVC: stesso ramo clap →
clap_builder → anstyle/clap_lex, nessuna nuova dipendenza nativa o build script.

## T-P8-05 — ctrlc, registrazione prima dell'aggiunta

ctrlc =3.5.2, MIT OR Apache-2.0, MSRV 1.69.0, nessuna feature termination.
Scopo: handler Ctrl+C del solo binario, che imposta CancellationToken;
nessuna dipendenza del codice engine da segnali o runtime GUI/async.
Fonte primaria verificata: [manifest ctrlc 3.5.2](https://docs.rs/crate/ctrlc/3.5.2/source/Cargo.toml),
[API](https://docs.rs/ctrlc/3.5.2/ctrlc/). Registrazione DEC-15 prima di Cargo.toml.
Implementazione Rust senza compilazione C/C++: usa API OS attraverso nix
(signal, default-features=false) su Unix e windows-sys su Windows; queste
chiamate di sistema sono quelle necessarie all'handler prescritto da DET §119.
Nessuna libreria esterna da installare, OpenSSL, GUI o framework aggiunti.
Su Apple il crate dichiara dispatch2, non attivo sui due target supportati.
Licenze/transitive effettive e tree saranno verificati dopo il download.
Il crate avvia un thread per consegnare le notifiche: è parte dell'eseguibile
previsto dal contratto Ctrl+C, non un processo agente in background.

Verifiche effettive T-P8-05: tree Linux ctrlc → nix 0.31.3 → bitflags 2.13.2,
cfg-if 1.0.5, libc 0.2.191; build cfg_aliases 0.2.2. Tree Windows MSVC ctrlc →
windows-sys 0.61.2 → windows-link 0.2.1. Manifest locali: nix MIT/MSRV 1.69,
libc MIT OR Apache-2.0/MSRV 1.65, bitflags MIT OR Apache-2.0/MSRV 1.56,
cfg_aliases MIT (nessun MSRV dichiarato), windows-sys e windows-link
MIT OR Apache-2.0/MSRV 1.71. Tutti compilati con il pin del progetto nel check
Linux/Windows GNU; gate Windows MSVC da CI. I build script nix/libc sono Rust,
nessun cc/cmake/OpenSSL nell'albero Linux né dipendenza Apple attiva sui target
supportati; i pacchetti Apple nel lock non significano dipendenza runtime Linux.
Download iniziali bloccati da DNS sandbox, ripetuti con accesso esterno riuscito.

## T-P9-04 — regex, dipendenza già presente

regex 1.13.1 (MIT OR Apache-2.0, MSRV 1.65), già in Cargo.toml/lock:
nessuna aggiunta o cambio feature. API regex::bytes::Regex nel compiler/piano.
Verificati manifest delle copie locali e cargo tree --locked -p regex per
Linux GNU e Windows MSVC: identico ramo pure Rust, nessun build script o FFI.
regex-automata 0.4.18 / regex-syntax 0.8.11: MIT OR Apache-2.0, MSRV 1.65;
aho-corasick 1.1.5: Unlicense OR MIT, MSRV 1.60;
memchr 2.8.3: Unlicense OR MIT, MSRV 1.61. build=false per tutti.
Toolchain 1.99.1 compatibile. Nessuna dipendenza PCRE/OpenSSL/host.
La libreria rifiuta backreference/lookaround anziché ignorarli; i limiti di
compilazione standard del crate restano attivi e producono InvalidRegex.
Gate nativo e spiegazione completa nel dossier T-P9-04.

## T-P15-01 — tooling first-party prima dell'introduzione

`xtask`, package locale separato, non pubblicabile, sola libreria standard Rust.
Nessuna dipendenza esterna o nuova licenza di terze parti. Lock autonomo con il
solo package locale, Cargo.lock del motore invariato. Serve alla ricetta release
portabile richiesta da ARCH§39B.3; non è distribuito come prodotto e non introduce
un workspace/GUI o un requisito runtime. Verifica artefatti15-02 ancora futura.

## T-P15-02 — parser artefatti, registrazione prima dell'aggiunta

goblin =0.10.7, MIT, MSRV1.85/edition2024: parser ELF/PE portabile pure Rust,
solo tooling xtask. Default disabilitati; std, elf32/elf64, pe32/pe64, endian_fd.
Fonte primaria: [manifest0.10.7](https://docs.rs/crate/goblin/0.10.7/source/Cargo.toml).
Non si riscrive un parser binario e non si richiedono dumpbin/readelf sull'OS
del runner. Nessuna dipendenza aggiunta al prodotto; suo Cargo.lock invariato.
Lock tooling verificato: log0.4.34 MIT/Apache2 MSRV1.71; plain0.2.3 MIT/Apache2
MSRV non dichiarato; scroll0.13.0 e scroll_derive0.13.2 MIT/MSRV1.85;
proc-macro2 1.0.107, quote1.0.47 e syn3.0.9 MIT/Apache2/MSRV1.71;
unicode-ident1.0.27 (MIT OR Apache2) AND Unicode3.0/MSRV1.71. Manifest locali
consultati; toolchain1.99.1 compatibile. Tree Linux GNU e Windows MSVC identici,
nessuna libreria nativa o cc/cmake/OpenSSL. I build script di proc-macro2/quote
sono tooling Rust del compilatore, nessun requisito runtime del prodotto.
Nessuna selezione di nuove librerie runtime Linux autorizzate.

## T-P15-04 — riuso nel solo xtask,2026-10-26

Il packager riusa `lintra` via path (default-features=false) per il contratto
BundleManifest già ratificato, senza duplicarne parser o grammatica. Nel tooling
si riusano versioni già qualificate dal prodotto: sha2=0.11.1 per checksum,
zip=9.0.5 con default-features=false e archivi Stored (nessun codec nativo),
tempfile=3.27.0 senza default per staging RAII e pubblicazione no-clobber.
serde_json=1.0.151 resta solo dev-dependency del tooling per fixture negative.
Licenze/API verificate sulle copie sorgente Cargo già qualificate; nessun nuovo
crate/versione o dipendenza runtime del prodotto introdotti. Cargo.lock del
prodotto invariato; il lock separato di xtask registra il grafo effettivo.

## T-P16-00 — spike Slint, registrazione prima dell’aggiunta

Solo crate separato del collaudo, non workspace/prodotto: slint=1.18.1 e
slint-build=1.18.1, stessi package del lock del prototipo fornito. Licenze
GPL-3.0-only OR LicenseRef-Slint-Royalty-free-2.0 OR commerciale; per desktop
si adotta royalty-free con attribuzione prevista nella UI finale. Verificati
LICENSES/manifest locali della versione esatta e
[fonte primaria](https://slint.dev/agreements/slint-royalty-free-license.pdf).
MSRV1.92, toolchain1.99.1 compatibile. Framework implementato in Rust ma non
si presume che ogni backend/transitiva sia pure-Rust: spike software,
default disabilitati, std/compat-1-2/renderer-software/backend-winit-x11;
winit è incluso per il supporto immagini/SVG richiesto dai widget e l’eventuale
lancio nativo. I test installano una piattaforma headless prima di creare
finestre, senza display/GPU. Grafico transitive da verificare, non cambia la allowlist
CLI né autorizza librerie di sistema per il packaging UI futuro.

png=0.17.16, MIT OR Apache-2.0, pure Rust, MSRV1.57: sole fixture/capture PNG
del test, già nel lock del prototipo. Fonte manifest locale e
[manifest primario](https://docs.rs/crate/png/0.17.16/source/Cargo.toml).
IBM Plex Sans/Mono forniti, famiglie e licenza SIL OFL1.1 verificati nei font;
[testo primario IBM](https://github.com/IBM/plex/blob/master/LICENSE.txt).
Font incorporati invariati, copia del testo OFL con copyright inclusa prima
dell’eventuale ridistribuzione. Nessuna dipendenza nuova nel core/CLI/lock root.

## T-P16-02 — promozione controllata nel solo adapter desktop

Prima dell’aggiunta: slint/slint-build=1.18.1 e png=0.17.16 sono gli stessi
package qualificati nello spike16-00, CI40485215039. Nel membro ui soltanto:
software renderer, std/compat-1-2/backend-winit-x11 senza default GPU.
Licenze/font/MSRV già verificati sopra; attribuzione con il vero AboutSlint
accessibile dal comando Informazioni (royalty-free2.0 §2a), non testo sostitutivo.
Risorse/fonts EmbedFiles, assenza font blocca build. tempfile=3.27.0 solo test
font, package già presente nel core, senza nuova versione. Palette chiara
originale e scuro nero/rosso/arancione. Nessuna dipendenza GUI nel core/CLI.
La allowlist di packaging UI rimane16-08; libfontconfig1-dev è setup runner
Linux, non nuova dipendenza CLI né promessa di UI già distribuibile.

## T-P16-03 — test metadata

Riutilizzata serde_json1.0.151/arbitrary_precision già qualificata nel core,
solo dev-dependency UI per fixture/schema reali dei test. Nessuna nuova versione
o libreria di dialogo/OS; browser costruito con Slint e metadata std::fs.

## T-P16-04 — spool temporaneo del worker UI

Promossa tempfile=3.27.0 (default-features=false) da dev-dependency a runtime
nel solo adapter UI: directory nuova per JSONL atomico, cleanup con ownership,
nessun filename paziente o path condiviso fisso. Package/versione/checksum già
qualificati nel core e nei test UI, MIT/Apache-2.0, implementazione Rust con API
filesystem native. Nessun nuovo crate/versione o dipendenza GUI nel core.
La protezione della persistenza storica resta PERSIST04, non questa spool report.

## T-PERSIST-04 — primitive protezione e memoria segreti

Prima della modifica manifest, risolte e lette le fonti originali:
- hmac=0.13.0, MIT/Apache2.0, MSRV1.85, RustCrypto/MACs, pure Rust/no librerie
  C runtime. Default off, featurezeroize. Digest0.11.3 esistente compatibile.
- zeroize=1.9.1, MIT/Apache2.0, MSRV1.85, RustCrypto/utils, pure Rust, niente
  FFI; default off/alloc per buffer SecretKey owned. Nessun derive/serde segreti.
- ctutils0.4.6 (digest/mac), MIT/Apache2.0, MSRV1.87, RustCrypto/utils; cmov0.5.4,
  MIT/Apache2.0, MSRV1.85, stesso repository: operazioni constant-time in Rust,
  eventuali intrinseci/assembly della piattaforma, nessun nuovo driver C/DB/TLS.
- sha2=0.11.1 resta pin esistente; abilitata featurezeroize per stati hash.

MetadataCargo isolata in /tmp/lintra-protection-deps prima del prodotto,
manifest ufficialehmac tag verificato. Protocollo e fonti nel report04.
Nessuna scelta di backend o storage reale/dati paziente in questa task.

## T-P16-05 — lettura canonica nel client desktop

serde/serde_json già qualificati nel core (stessi pin/checksum) diventano runtime
nel solo adapter UI per decodificare le righe JSONL richieste dalla tabella.
serde_json passa da dev a runtime; serde derive riusa il package esistente.
Nessun nuovo crate/versione/driver o libreria di cache; offset, cache bounded,
thread e file usano std. Licenze MIT/Apache-2.0, stessa toolchain qualificata.
