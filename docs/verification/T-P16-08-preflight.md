# T-P16-08 — preflight packaging dopo Signature

2026-10-29 · IN_PROGRESS, solo preflight; nessun packaging dichiarato verificato.
Prerequisiti reali: UI07/Signature5720748, CI40958452017 tutti5job success;
15-05 DONE con release immutabile0.1.1-rc.1 ed evidenze versionate.
Governance single-agent, nessun processo in background o delega.

Fonti lette: TASK_PROMPTS/T-P16-08, ARCH39D.2, DET AddendumB.1, xtask main/artifact,
scripts/release.py e relativi test, workflow release.yml, DEC15/DEPENDENCIES e
SPIKE_REPORT. Il redesign è una consegna distinta e completa; nessuna modifica
a token, schermate, modelli o core è necessaria per il packaging.

## Stato effettivo e residuo

La ricetta xtask e lo script di release costruiscono/identificano soltanto la
CLI `lintra`/`lintra.exe`. Il verifier Linux consente libc/libm/loader/libgcc,
non una UI. Non riusare una certificazione CLI sul binario Slint.
`readelf -d target/release/lintra-ui`, build Signature qualificata localmente,
rileva DT_NEEDED: libfontconfig.so.1, libgcc_s.so.1, libm.so.6, libc.so.6,
ld-linux-x86-64.so.2. L'assenza di altri DT_NEEDED non certifica librerie caricate
con dlopen (backend X11/input); vanno considerate nel collaudo nativo.

Fontconfig è una dipendenza già presente nello stack Slint qualificato; il setup
runner libfontconfig1-dev non era una precedente allowlist runtime, come dichiarato
in SPIKE_REPORT e DEPENDENCIES. Occorre mantenere separata la policy UI da quella
CLI e testare il rifiuto di librerie extra; nessun rilassamento TLS, CRT o host-path.

L'archivio corrente è rigorosamente di3entry (binario, BUILD.json, SHA256SUMS),
con metadati deterministici/no-clobber e verifica source/target/hash. Estensione
UI da mantenere esplicita/additiva, preservando i consumatori della CLI. Windows
richiede verifica import PE nativo e sole DLL di sistema; nessun risultato da
cross-compilazione o Linux può sostituirla. Riproducibilità deve confrontare due
build pulite, non il binario locale costruito normalmente senza remapping.

Versione corrente0.1.1-rc.1 già rilasciata e immutabile: non riaprire né sovrascrivere
quel tag. Preparare e collaudare il packaging prima dell'eventuale pubblicazione
con nuova versione esplicita secondo il protocollo esistente. Nessun token o
segreto letto/esposto e nessuna release avviata in questo preflight.

## Prossimo passo

Implementazione tooling/workflow e test dedicati al prodotto UI, poi qualificazione
nativa Linux/MSVC e archivio verificato. M8 rimane aperta; nessun gate packaging
passato per il solo successo del redesign. PERSIST08 non anticipata.
