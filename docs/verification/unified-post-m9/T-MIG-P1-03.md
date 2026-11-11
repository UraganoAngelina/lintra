# T-MIG-P1-03 — Runtime CachyOS e dipendenze

11 novembre 2026. **DONE nel perimetro build/smoke personale D14.**
Prerequisito P1-01 DONE su b1f14ce; preflight bdca1cf. Nessun GO Windows o
gate aggregato G-MIG-1 implicito.

[Profilo operativo](../../unified-post-m9/CACHYOS_GUI_RUNTIME_PROFILE.md) e
[manifest macchina](T-MIG-P1-03-runtime-profile.json): chiusura di 151 ELF,
99 package provider con versioni/licenze OS, inclusi processi WebKit e sandbox
helper. Nessun missing, nessun RPATH/RUNPATH. Non è una distribuzione universale
di `.so`: conservare i pacchetti completi e le risorse sul profilo personale.
GTK/WebKit dinamici restano un'eccezione GUI-only ratificata, senza cambiare N-20.

Tre script first-party in `scripts/` spiegano e riproducono la qualifica:
`tauri_runtime_inventory.py` segue i link ELF fidati e registra SHA, provider e
sezione dinamica; `tauri_runtime_relocated_smoke.py` copia l'eseguibile in una
cartella privata, cambia CWD e invoca il vero smoke con cache effimere;
`tauri_runtime_missing_library.py` maschera WebKit solo in un mount namespace
readonly e verifica errore mirato, exit 127, nessun ACK e hash host invariato.
Nessuna dipendenza nuova, algoritmo crittografico, cambio al prodotto o regola JS.

Prova positiva [relocated-v2](T-MIG-P1-03-relocated-v2.json): render/IPC Wayland
PASS, exit 0, 1,754s. Sorgenti del harness definitivi in scripts/ registrati
nell'inventario working tree. Il primo relocated-v1 era già positivo, poi lo
script è stato collocato fra i tool Linux esterni alla build Windows; riprovato
il nuovo riferimento al solo harness, senza riscrivere l'evidenza precedente.

Prova negativa [missing-library-v2](T-MIG-P1-03-missing-library-v2.json):
loader exit 127 con `libwebkit2gtk-4.1.so.0: ... Permission denied`; nessun
avvio, SHA della libreria host invariato. Il v1 richiedeva troppo precisamente
`file too short`, mentre il dispositivo montato in questo namespace produce
EACCES. Fallimento del harness conservato; aspettativa corretta al contratto
effettivo (loader, dipendenza esatta, 127, no ACK, host invariato), senza cambiare
binario, protezioni, golden o accettare errori generici di bubblewrap.

Binario qualificato SHA-256
`c53fd4ba3d5c7a1409dd4a62a9fc2a5c46cdfd0f271e0159fe7bd0e1c71e6883`,
lo stesso P1-01. Configurazione/runtime Linux dell'app invariati; i nuovi file
P1-02 sono overlay Windows e relativo ICO. Non si attribuisce la CI Server
Windows a questi tool Linux: prove native locali pertinenti, hash e log gzip
archiviati. Controlli original520/fonti/status e diff restano qualificati.

Profilo debug soltanto; qualifica release, plugin/driver di altri host, nuovi
frontend/servizi, rete/CSP/DPI e funzionalità normative non sono dedotti dal
render di un probe sintetico. Nessuna release Linux GUI ufficiale o decommission
Slint. Aggiornamenti dell'host richiedono profilo/smoke nuovi, non un freeze OS.

P1-02 rimane BLOCKED_NATIVE_ENVIRONMENT: autore senza Win11. P1-04/05/06/07
e P2/P3 non sono READY. Next indipendente READY **U-OPS-04**, quindi U-REL-03
e U-PERF-01 secondo i prerequisiti. AUTO_CONTINUE single-agent.
