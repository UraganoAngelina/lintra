# T-P16-08 — tracciabilità accettazione

Revisione candidata: aa7be4f2fc76cf78d0f03fdcaf6d2b653c130829.
CI: [41036453232](https://github.com/UraganoAngelina/lintra/actions/runs/41036453232).
Stato: DONE. Tutti i 7 job sono completed/success, inclusi il gate generale
Windows e i confronti finali JSONL/pixel fra runner. M8 raggiunta nello scope UI.

| Requisito | Evidenza effettiva |
|---|---|
| Preservazione del refactoring dell'autore | Preflight d5b0940; check UI e visual1×/2×; manifest di 12 file stile/font riconfermato su aa7be4f, nessun nuovo riferimento generato |
| Eseguibile UI separato, bundle incorporato | `xtask --product ui`, package/bin espliciti; entrambi gli archivi reali scaricati e ispezionati |
| Riproducibilità | Due compilazioni pulite per OS e confronto integrale dei byte, job123998468330/123998468372 success |
| Policy Linux esplicita | ELF DT_NEEDED e librerie effettivamente caricate dal probe X11 confrontate con allowlist; percorsi risolti sotto root OS |
| Policy Windows | PE import table qualificato sul runner MSVC; sole DLL OS, nessuna SQLite/CRT/OpenSSL esterna |
| Isolamento della policy CLI | Test tooling di accettazione/rifiuto prodotto; default CLI preservato, nessuna eccezione UI applicata alla CLI |
| Nessun percorso privato di build | Scanner ELF/PE con radici esplicite; sola eccezione delimitata per il prefisso socket OS X11, con casi negativi e controllo di tutte le occorrenze |
| Archive deterministico e verificabile | Inventario esatto di 5 voci, metadata fissi, SHA256SUMS, provenance/source/target; no-clobber e casi negativi nei 23 test Python |
| Licenze integrali e portabili | Testi IBM Plex/Slint con eol=lf esplicito; checkout core.autocrlf=true byte-identico; ZIP Windows finale ispezionato da Linux con checker esatto invariato |
| Risorse senza strumenti di sviluppo | Versione e rendering software dal vero archivio estratto, PATH vuoto, catalogo incorporato e frame di 307200 pixel su entrambi gli OS |
| Avvio desktop nativo | Finestra/controller/event loop normali e arresto controllato su Xvfb Linux e desktop Windows; il probe software resta una prova distinta |
| Provenienza e ambienti | Versione/commit esatti verificati dopo estrazione; BUILD.json identifica Rust, Slint, Python e immagine runner; Linux Ubuntu 24.04/glibc 2.39, Windows 2025/MSVC |
| Distribuzione controllata | Artifact CI privati, nessuna nuova release/tag e nessuna sovrascrittura della release immutabile0.1.1-rc.1 |
| Regressioni e confronti fra OS | Linux/MSVC generali e confronti JSONL/pixel tutti completed/success; evidenza T-P16-08-native-ci.json |

Checksum degli archivi, SHA degli eseguibili, install probe e osservazioni loader
sono in [T-P16-08-qualified-archives.json](T-P16-08-qualified-archives.json).
Implementazione, ragioni dei controlli e diagnosi dei tentativi precedenti in
[T-P16-08-packaging.md](T-P16-08-packaging.md). Gli artifact di diagnosi delle
run fallite non sono archivi qualificati e non vengono usati per dichiarare M8.

Il test di contesa SQLite corretto in 4e44703 distingue sicurezza CAS da progresso
del singolo tentativo con timeout zero. Verifica anche due pubblicazioni rifiutate
sotto un vero lock SHARED e una nuova transazione esplicita soltanto dopo assenza
accertata. Nessuna modifica all'adapter o retry automatico dei commit ambigui.

Restano esterni alla task: integrazione storica nella UI, nuove norme, backend
remoti, dati reali, nuova pubblicazione e qualificazione di OS precedenti a quelli
effettivamente provati. La milestone M8 non certifica tali capacità.
