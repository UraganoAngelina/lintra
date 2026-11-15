# T-MIG-P4-03 — Preparazione immutabile

15 novembre 2026. **DONE nel perimetro preparazione**, dopo P4-01/02.
[Receipt](T-MIG-P4-03-checks.json). G-MIG-4 resta aperto.

`plan_parameter_set` verifica riferimento corrente, tipo e dominio attraverso
`VerifiedRunContext`, mantenendo gli interi come `IntegerLiteral` esatti.
La singola modifica usa il controllo parziale LocalOnly del core; la successiva
preparazione verifica sempre il mode effettivo, compreso Full e i parametri
di namespace richiesti. Nessun avvio Full viene convertito in LocalOnly.
Ogni input/parametro accettato produce un nuovo plan_ref e invalida readiness.

`plan_prepare` costruisce ValidationRequest usando esclusivamente i PathBuf
confermati dal picker. Riusa preflight/preflight_flow_inputs e il contesto
tipizzato prima delle nuove letture dei file. Full produce un'unica Flow con
gli input richiesti, ordinati come lo schema; nessuna lista vuota o trace
obbligatoria mancante viene accettata. Il controllo fisico riusa InputPreview.

Il piano pronto possiede un Arc immutabile di richiesta, parametri, schema raw
congelato e audit del bundle. L'hash è verificato sullo stesso schema acquisito.
La risposta contiene solo riferimenti, mode, hash, contatori e provenienza;
nessun path, valore parametro, dato, schema raw o diagnostico. Un cambio schema
invalida gli handle correnti senza mutare snapshot già catturati. Risposte
fallite non confermano una nuova preparazione. Nessun worker o report allocato.

23 test Rust nativi PASS, 1 ignored perché richiede display; 24 Node, strict
TypeScript/build, Clippy adapter/core/UI e 8 regressioni Slint PASS, exit 0.
Coperti schema cancellato dopo acquisizione, riferimento stale, Full completo
e incompleto, parametri obbligatori prima di I/O, dominio/tipo, interi oltre
u64/limite JS e corsa reale fra selezione e preparazione con mutex/Busy.
Le API TS controllano esattamente i DTO e l'identità della risposta.

Il picker GTK possiede ora destroy-with-parent. Prova selettiva Wayland con
file sintetico e XDG privato PASS, compresa distruzione del parent durante
il dialogo. La prima prova usava close del window manager, inibito dal modale:
FAIL conservato; driver corretto alla distruzione effettiva e PASS in 0,97s.
Smoke nativo React/raw IPC del debug PASS: prova bootstrap, non viste P5.

MSVC e smoke Windows al checkpoint CI del blocco; nessuna equivalenza con
collaudo grafico Windows11. Core normativo, golden e root lock invariati;
nessuna VM o nuova suite workspace. Segue P4-04 worker LocalOnly posseduto.
