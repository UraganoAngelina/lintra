# T-P18-03 — Runbook di release

## Preflight — 2026-11-05

Ripresa esplicita dell’autore: workflow automatico single-agent riattivato dopo
il checkpoint17-04. Prerequisito15-05 DONE: implementazione bbfdb93,
CI40349711927 e pubblicazione40352109303 completed/success, release privata
0.1.1-rc.1 non draft/immutable verificata nel dossier15-05. Ratifica
DEC-RELEASE-SETUP-01 A autorizza prova soltanto dalla CI, destinazione privata e
assenza di sovrascritture; nessun nuovo consenso ordinario necessario.
T-P17-04 chiusa con CI41887090825 success su8045394, sette job richiesti positivi.

Letti piano§4/14, prompt18-03, ARCH39C.4–5, ratifica e implementazione effettiva
release.yml/scripts/release.py. Versione esistente rc.1 già riservata: scelta
ordinaria nuova prerelease0.1.1-rc.2, senza mutare repository o contratto release.
Nessuna PHI, chiave operativa, installer o migrazione. docs/migration preesistente
resta esclusa. DEC-17 non risolta da questa task; CAD non implementato.

## File e correttezza

- docs/RUNBOOK_RELEASE.md: procedura eseguibile, separazione tag binari e SHA
  sorgente, CI esatta, due target nativi, inventario/checksum interni/esterni,
  immutabilità, recupero da errori ambigui e nuova versione con nota esplicativa.
  Distingue packaging CLI da UI e non promette compatibilità glibc universale.
- docs/releases/0.1.1-rc.2.md e README: note della prova e collegamento al runbook.
  Le note nel commit sorgente sono reperibili dal link di provenienza della
  release, senza modificare la descrizione di una release già immutabile.
- Cargo.toml, ui/Cargo.toml, Cargo.lock: solo versioni first-party coerenti.
  Nessuna dipendenza nuova, modifica del runtime o norma.
- Tre expected.summary.json dei golden SIOC: soltanto validator_version passa
  a rc.2. Confronto strutturale contro HEAD richiede tutti gli altri valori
  identici; errori JSONL, input e schema non modificati. I test esercitano la
  versione reale del package e pertanto il metadata segue il bump effettivo.
- ATOMIC_TASKS_INDEX e IMPLEMENTATION_STATUS: avanzamento fedele e handoff.

## Gate e stato

IN_PROGRESS. Gate locali in esecuzione foreground con
/tmp/lintra_release_runbook_gates.py; ledger target/verification-release-runbook.
La matrice locale controlla fmt, Clippy workspace all-features, test workspace
all-features, test Python e boundary dipendenze; conserva hash prima/dopo.
Non certifica Windows. Restano CI propria sul commit esatto, dispatch release,
pubblicazione privata immutabile e verifica degli asset scaricati. Nessun DONE
anticipato e nessuna release locale. Il collaudo15-05 è prerequisito e storia,
non sostituisce la nuova prova rc.2.


## Gate locali conclusi

Cinque comandi exit0,887esiti workspace all-features in112riepiloghi,
0fail/6ignored. Fmt, Clippy all-targets/all-features, test Python e boundary
positivi. Ledger T-P18-03-local-gates.json, impronte sorgenti immutate.
Controllo separato link locali/versioni manifest-lock e confronto strutturale
dei tre golden PASS: cambia solo validator_version; JSONL e schema invariati.
Rust runtime non modificato. Restano CI nativa propria e prova release reale.


## Correzione gate tooling —2026-11-05

CI41996304493 su497352c: Linux ha superato fmt/Clippy, test standard/allfeatures/
core, smoke e prove precedenti; fallisce allo step26Clippy tooling. Windows ancora
in corso all’osservazione. Riprodozione locale sul runtime esatto497352c: exit101,
`cannot update ... xtask/Cargo.lock because --locked was passed`. Il bump aveva
omesso la versione lintra nel lock del workspace separato; nessuna dipendenza
nuova o policy runtime coinvolta. Nessun tentativo di pubblicazione ha riservato
rc.2, che rimane disponibile salvo preflight remoto autoritativo.

Corretto quel solo metadata first-party in xtask/Cargo.lock. Aggiunto un test
Python che confronta versioni dei manifest e dei due lock reali, prima delle
build native costose; l’assenza di una voce o una versione discordante fallisce.
Aggiornato il runbook con tutti i lock e gate standalone necessari. Gate di
correzione effettivi: fmt/Clippy all-targets tooling exit0,17test/1ignored tooling,
24testPython passati. Ledger T-P18-03-tooling-fix.json.
Durante questi gate il file registry CAD è stato preservato fuori dai sorgenti
e il runtime ricondotto esattamente a497352c; successivamente il lavoro CAD è
stato ripristinato con hash identico. Nessuna modifica dell’utente rimossa.
Serve una nuova CI sul commit corretto prima del dispatchrelease; la run fallita
resta evidenza e non viene reinterpretata come gate positivo.
