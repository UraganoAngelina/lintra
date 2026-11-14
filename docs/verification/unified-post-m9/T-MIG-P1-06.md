# T-MIG-P1-06 — GO tecnico piattaforme, G-MIG-1

14 novembre2026. **DONE — G-MIG-1 GO/PASS nel perimetro degli spike nativi**.
P1-02/03/04/05/07 sono DONE con evidenze attribuite ai rispettivi sorgenti.
Questa decisione tecnica attua D14 e ADR-MIG-001/002/003 già ratificati;
non inventa una nuova ratifica dell’autore né autorizza rilascio o cutover.

## Matrice delle prove

| Requisito | Evidenza effettiva | Identità e limiti |
| --- | --- | --- |
| Win11 x86_64, NSIS offline perMachine, WebView2 | [P1-02](T-MIG-P1-02.md): installazione runtime presente e provisioning da assenza controllata, due desktop standard, isolamento ACL, uninstall e restore PASS | Installer CI43014153420/bdca1cf; runtime157.0.4341.46, build26300. Assenza preparata su VM ripristinabile, non OS appena installato |
| Nuovo installer dopo il contenimento security | [P1-07](T-MIG-P1-07.md): setup/uninstall offline0, HKLM/Program Files, desktop standard, sentinelle conservate, restore/spento PASS | CI43595367523/608ccdc; installerb2bc8462…1eedc1, installed6f1b3242…419703. Runtime già presente; non si attribuisce il vecchio provisioning al nuovo file |
| CachyOS/Hyprland reale e dipendenze | [P1-03](T-MIG-P1-03.md): ELF/provider, relocated smoke PASS e missing-library fail-closed | Build/smoke personale; runtime/profilo qualificati. Nessuna distribuzione Linux universale |
| CSP, text-only, ACL e navigazioni native | [P1-04](T-MIG-P1-04.md):18 controlli,12 eventi CSP, popup trusted/navigation,0TCP/0HTTP su Wayland eWin11, recovery del documento | Runtime608ccdc, WebKitGTK2.54.2/WebView2157.0.4341.46; collettore raggiungibile prima del test, nessuna eccezione ai predicati |
| Font, profilo e scala OS | [P1-05](T-MIG-P1-05.md): font locali caricati, misure native coerenti, profili per-account e capture PASS | Sorgente413b7e4/CI43494464261; scala100% sui desktop osservati.150% è solo test unitario, non misura nativa |
| Build e contratti dello spike corrente | [CI43595367523](https://github.com/UraganoAngelina/lintra/actions/runs/43595367523): TS strict,9test Rust MSVC, fmt/Clippy, release/NSIS PASS | Windows Server2025 solo build; prove desktop Win11 separate. Input compilati ancora identici; successiva correzione solo del driver focus provata nativamente |
| Regressioni core/CLI/Slint, feature set, golden e SQLite cross-OS | [U-REL-03](U-REL-03.md), CI43078048297 su3aa0d14 PASS; riuso verificato per blob/tree Git | Nessun delta nei sorgenti/config/dipendenze/fixture pertinenti. Non si dichiara una nuova esecuzione della matrice completa |

Il delta attuale non cambia CSS/font, resolver per-account o logica di misura
P1-05. Non si estendono le sue misure ad altre scale o a runtime non provati.
P1-04 eP1-07 provano separatamente il documento ordinario e quello security
con gli asset locali e l’IPC effettivo; la pagina di errore non riceve autorità.
Identità complete e hash sono nei manifest dei singoli dossier.

## Decisione e confini

**GO all’ingresso di MIG-P2**, dove congelare architettura, DTO, lifecycle e
sicurezza prima di implementare il prodotto. Nessun ostacolo tecnico aperto
impedisce lo stack ratificato sui due ambienti osservati. I tentativi falliti
precedenti restano nei log; non vengono contati come PASS.

La prova dimostra fattibilità dello spike, non copertura del futuro frontend:
LocalOnly/Full/storico, Job Manager, PHI/cache/dump, broker file, approvazioni,
paging, export e durabilità conservano tutte le task/gate già previsti.
G-MIG-2..9, G-CADUI, G-OPS, G-SIOC1 e G-REL non passano da questo documento.
Slint resta disponibile e nessun artefatto viene distribuito ai colleghi.

Il contenimento Connection-Allowlist è verificato sul WebView2 concreto157;
versioni precedenti che ignorano l’header non sono qualificate. Il filtro SDK
resta attivo ma da solo non aveva eliminato i socket. Il prodotto dovrà
rispettare il runtime qualificato e i gate di sicurezza/distribuzione previsti,
senza inferire compatibilità universale dal build verde. La CSP non certifica
assenza di traffico Microsoft/OS: il finding `/ping` del redistributable inP1-02
rimane distinto dal traffico applicativo osservato dal collettore.

Il reload conserva soltanto l’entry locale e il checkpoint sintetico Rust;
non promette recovery di job o stato React arbitrario. Report normativi,
HMAC/CAS/SQLite e goldens non sono modificati. NOT_TESTED_HARDWARE invariato.

## Verifica e handoff

Consolidamento documentale, verifica dei manifest nativi e dell’identità degli
input compilati; nessuna nuova build, CI, suite workspace o sessione Windows.
Il ciclo obbligatorio07 è terminato con rimozione di task, autologon, firewall
e sorgenti temporanei, accessi standard/admin preservati e VM **shut off**.

P1-06 DONE; **G-MIG-1 PASS**,36/128 (28,1%). Prossima READY **T-MIG-P2-01**.
ID, dipendenze, acceptance e gate del piano restano invariati. AUTO_CONTINUE.
