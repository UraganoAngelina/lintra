# T-MIG-P4-02 — Broker file e preflight condiviso

15 novembre 2026. **DONE nel perimetro broker/preflight**, dopo P4-01.
[Receipt](T-MIG-P4-02-checks.json), [dialogo GTK reale](T-MIG-P4-02-native-picker.json).

`ConfigurationError` e `InputPreview` sono spostati senza cambiare algoritmo
da Slint alla library e riesportati nel percorso precedente. Restano le prove
del file regolare aperto, longest BOM reject, campione massimo 65536 byte,
EOF e contatori LF/CRLF/CR senza decoding, normalizzazione o record conservati.
Gli 8 test Slint esistenti PASS mantengono la parità dei chiamanti reali.

`plan_create` lega configurazione, mode esplicito e trace_ref per LocalOnly;
Full richiede trace_ref null e non diventa una serie di run locali. Il piano
è una bozza Rust, non un'autorizzazione ad avviare record validation.
`plan_input_choose` accetta solo plan_ref/trace_ref correnti: prima del dialogo
verifica DTO, owner, piano e trace/mode. Path, owner e namespace arbitrari nel
JSON vengono rifiutati. Il callback del picker viene passato solo dall'handler
nativo; il renderer non può costruire o deserializzare un PathBuf autorizzato.

Linux usa il dialogo GTK già presente nel runtime Tauri, sul thread GUI della
command sincrona; Windows usa IFileOpenDialog con parent HWND nativo, filesystem
esistente, nessun cambio cwd e DONTADDTORECENT. Il buffer COM viene copiato
direttamente in OsString UTF-16 e liberato; nessuna conversione lossy in JS.
GTK conserva PathBuf nativi e non abilita preview del contenuto. Sono API
delle dipendenze già installate/pinnate; nessun plugin shell/fs o nuovo framework.

Il path confermato resta nel piano Rust. Il client riceve soltanto disposizione,
nuovo plan_ref, trace_ref e contatori versionati; nessun nome, path o byte raw.
Se il dialogo viene annullato, il piano resta invariato. Errore di lettura/BOM,
overflow o fallimento della risposta non sostituiscono i binding precedenti.
Ogni input accettato cambia riferimento del piano e invalida i vecchi intenti;
nessun token di input può essere trasferito fra piani o configurazioni.
La chiusura del contesto durante il picker viene ricontrollata nel callback.

Verifiche locali: 19 test Rust nativi, 22 Node, strict TypeScript e 8 regressioni
Slint PASS, exit 0. Coperti stale prima del picker, annullamento, BOM, extra
path/raw nel DTO, assenza di modifica del file e path Unix non UTF-8 conservato.
Il test selettivo del vero dialogo GTK Wayland PASS, file sintetico con spazi/=
e profilo XDG temporaneo. Il primo driver emetteva Accept prima del caricamento
asincrono della selezione: FAIL conservato; corretto il solo driver e nuova
prova PASS in 0,60s. Nessuna capture o file reale. Il test è ignored ordinariamente
perché richiede display: non introduce un nuovo ciclo GUI/VM sistematico.

Windows UI non dichiarata provata qui; build/contratti MSVC al checkpoint del
blocco e interazione UI completa nelle task pertinenti. G-MIG-4 resta aperto.
Segue P4-03: parametri dichiarati e preparazione immutabile della richiesta.
