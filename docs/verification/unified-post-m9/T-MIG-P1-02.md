# T-MIG-P1-02 — Spike Windows / WebView2

11 novembre 2026. **BLOCKED_NATIVE_ENVIRONMENT; build CI PASS**.
Prerequisito P0-08 DONE; P1-01 Linux DONE su b1f14ce.

Preflight corrente: nessun virsh, QEMU, VBoxManage, vmrun, PowerShell, Wine,
NSIS locale, immagine VM Windows o /dev/kvm disponibile. La ricerca delle
immagini è stata limitata al workspace Lintra; nessuna scansione di dati
personali o modifica di checkout adiacenti. L'autore ha risposto alla domanda
mirata di disponibilità: **«Nessun ambiente Windows 11 al momento»**.
La CI Windows Server 2025 non soddisfa il requisito Win11 clean-room.

Preparati overlay Windows dello spike, ICO dal marchio vettoriale esistente e
workflow nativo MSVC/Clippy/NSIS. Installer configurato perMachine con
offlineInstaller WebView2, ID del probe separato dal prodotto. Nessun package
applicativo nuovo o modifica normativa, core, CLI, Slint, HMAC o SQLite.
Setup-node con SHA ufficiale immutabile serve a fissare Node nella build;
gli altri Actions pin riutilizzano quelli qualificati nel repository.

[Procedura e necessità documentate](../../../spikes/tauri-runtime/WINDOWS_QUALIFICATION.md).
Il workflow produrrà artefatto sintetico e manifest SHA/run/OS senza tag o
release. Il download degli strumenti durante la build non è traffico del
prodotto. Non attribuire a tale build installazione offline, due account,
versioni runtime, DPI o zero-network mai provati.

Esiti locali: JSON overlay sintatticamente valido e diff check PASS; review
manuale del workflow. Parser YAML/PowerShell non installati localmente:
nessun package aggiunto solo per il lint; parsing Tauri/Actions effettivo in CI.
Inventario e digest in T-MIG-P1-02-preflight.json;
CI43014153420 conclusa success su bdca1cf00121717b3c58504127ae4ce45f806e6b: frontend strict, fmt/Clippy MSVC release, build e NSIS offline configurato, upload artefatto sintetico. Metadati e log completi archiviati in native-43014153420.json/log.gz. Non sostituisce la prova Win11. Windows11 install/run NOT_RUN.
G-MIG-1 NOT_VERIFIED. La prova non è sostituita da simulazione o da un flag.

P1-03 è conclusa indipendentemente su Linux; next READY **U-OPS-04**. Nessun ingresso P2/P3 prima del vero gate G-MIG-1. AUTO_CONTINUE.

Artefatto sintetico scaricato nel target locale ignorato da Git e verificato
contro il manifest CI: exe8.722.432byte SHA7927f57a…2b1b0d;
setup219.878.356byte SHAf31b7064…c0897. Digest completi e percorsi nel
T-MIG-P1-02-artifact-receipt.json. Installazione non eseguita: nessuna
consegna del prodotto o PASS Win11 derivato dal solo file disponibile.
