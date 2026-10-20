# IMPLEMENTATION_STATUS.md — Lintra

Consuntivo aggiornato al **2026-10-20**, derivato da `IMPLEMENTATION_STATUS_SEED.md`.
Piano v1.1; architettura e detailed design revisione 2026-10-16.

## Posizione corrente e prerequisiti

- P1 / **T-P1-01 — DONE**: scaffold compilabile del package `lintra`.
- Sessione corrente: **T-P0-07 DONE**, contratto del commit 72f9f5e
  approvato dall’autore con «Approvo il contratto T-P0-07». Verifiche
  strutturali e limiti restano registrati nel report.
  **T-P0-08 DONE nello scope minimale autorizzato**: profilo versionato con
  flow e cinque trace, 135 campi, cinque PK, sei join; ledger dei rinvii e
  piano fixture verificati. DEC-SIOC-MINIMAL-01: nessun dataset, semantica
  minima con 050 nell’ambito confermato; encoding runtime esplicito, mar01
  per il futuro controllo età. Ricontrollo strutturale Rust T-P3-04 eseguito: nessuna issue sui cinque trace/135 campi.
  Compiler Rust incrementale locale verificato; nessun validatore normativo runtime implementato.
  **T-P0-09 DONE / G0 APPROVED / M0 REACHED**: l’autore ha approvato
  la checklist del commit 407f89e con i rinvii indicati.
  **T-P1-03 DONE / QG-2 PASS**: [CI Linux/Windows verde](https://github.com/UraganoAngelina/lintra/actions/runs/39232752518)
  sul commit 34e67da; autenticazione GitHub funzionante fuori sandbox.
  **T-P2-01 e T-P2-02 DONE**, entrambe con CI Linux/Windows verde.
  **T-P2-03 IN_PROGRESS**: CNF-DOMAIN-01/proposta A approvata integralmente
  dall’autore; parte autonoma verificata in locale e CI Linux/Windows 39235805213 su 70cdb5e.
  Residuo ValidationRunError applicativo aperto, T-P2-03/M1 non complete;
  parte autonoma, T-P3-01 e T-P3-02 verificate; T-P3-03 verificata in locale e CI Linux/Windows; T-P3-04 verificata in locale e CI Linux/Windows; T-P3-05 verificata in locale e CI Linux/Windows; CNF-COMPILER-01/proposta A approvata; T-P3-06 verificata in locale e CI Linux/Windows; CNF-FORMAT-01 risolto con mappa approvata; T-P3-07 verificata in locale e CI Linux/Windows; T-P4-01 verificata in locale e CI Linux/Windows; T-P4-02..04 DONE: CI 39282115064 Linux/Windows verde su 7168a05; T-P6-01 e T-P4-05 DONE su CI 39389225120; T-P5-01 DONE, CI 39389657166 verde Linux/Windows su a679f40; T-P5-02 DONE, CI 39390492669 Linux/Windows verde su 39ed9b2; T-P5-03 locale verificata (132 test), CI successiva pendente.
- A-02/DEC-01: avvio P1 prima di G0 autorizzato nel kit e dalla richiesta
  dell'autore; DEC-03: nome `lintra` confermato.
- Repository locale `lintra/`, accanto allo starter kit; `origin` configurato
  dall'autore su `https://github.com/UraganoAngelina/lintra.git`.
  Lo starter kit non è parte del repository né una dipendenza di runtime.
- Toolchain corrente: **Rust 1.99.1**, scelta esplicitamente dall'autore,
  installata e fissata; target `x86_64-pc-windows-gnu` disponibile.
- UI: prototipo Slint dichiarato funzionante dall'autore, non verificato,
  non presente in questo scaffold e non considerato MVP.
- `CNF-01..11`, `CNF-LINTRA-SOURCE-01`, `CNF-PERSIST-01` risolti
  documentalmente; ADR-PERSIST-001 approvata; `PCR-CAD-001` superata.

## Autorizzazione corrente — 2026-10-19

Mandato successivo: «non aspettare la mia approvazione per cominciare una task,
cominciala subito se non ci sono problematiche nella task precedente».
Dopo ogni task verificata, recap breve dei file e avvio della successiva se
le dipendenze sono soddisfatte. Questo supera lo stop tra task dei prompt
atomici; restano le decisioni realmente bloccanti e gli altri gate.


L'autore ha revocato l'uso delle skill Superpowers. Esecuzione diretta,
una task alla volta, senza subagenti in questa sessione. Dopo la verifica
si procede senza conferme ordinarie solo se prerequisiti e risultati sono
chiari; le ambiguità richiedono chiarimento prima della task successiva.
Queste istruzioni sono conservate in `AGENTS.md`. Nessuna skill Superpowers,
orchestrazione o nuovo piano del relativo framework utilizzati il 2026-10-19.

## Autorizzazione storica — 2026-10-16

L'autore ha autorizzato esecuzione continua e parallelizzazione delle task,
rispettando le dipendenze funzionali della mappa e dei prompt atomici.
Questo mandato sostituisce lo stop ordinario dopo ogni task, senza derogare
ai gate normativi, alla CI reale o alle decisioni umane ancora aperte.
Ha inoltre autorizzato creazione autonoma del repository GitHub e fornito
identità per i commit locali; nessuna password viene salvata nei file.
DEC-11: scelta Rust 1.99.1 ricevuta. Baseline Linux di release ancora aperta.

Piano storico: [fondamenta](docs/superpowers/plans/2026-10-16-foundations.md).
Skill utilizzate nella sessione precedente: Superpowers pianificazione, esecuzione tramite agenti,
parallelismo e review/verifica; integrazione GitHub suggerita perché assente.

## Integrità e baseline consultata

`VERIFY_KIT.py`: **FAIL, exit 1**, per quattro file extra:
`EXPORT_CAD/A.txt`, `B1.txt`, `C.txt`, `D.txt`. Nessun hash/dimensione errato
o file censito mancante segnalato. Verifica indipendente: tutti i 196 file
del manifest conformi a dimensioni/SHA-256, 140 prompt presenti. Non dichiarare
PASS per il kit completo. Contenuto degli export non letto, copiato o versionato.

Consultati ARCH §3, §31, §39A–D, §45; DET §4–5, §94, §145, §166–174;
Piano §4, §9/P1, §10.3, §14–15; ADR, mappa dipendenze, indice e prompt T-P1-01.
Le sette copie documentali nel repository coincidono byte per byte con le fonti.

SIOC FROZEN: cinque PK nei JSON, `uniqueness_scope: trace`,
`E_DUPLICATE_RECORD_KEY`; dipendenze reference esterne dichiarate, non tutte
fornite come dataset. CAD RELEASE_CANDIDATE: cinque trace, 83 campi, profilo
storico e tre dataset consultati. Nessuna modifica/importazione degli schemi.
Le analisi aggregate preesistenti non sono test Rust o golden approvati.

## Gate e milestone

| Gate | Stato | Evidenza / limite |
|---|---|---|
| G0 — normativa | APPROVED_WITH_DEFERRALS | Autore: «Approvo G0 con i rinvii indicati», checklist commit 407f89e, 2026-10-19 |
| G1 — repository CI | PASS | [Run 39232752518](https://github.com/UraganoAngelina/lintra/actions/runs/39232752518), Linux GNU e Windows MSVC success, commit 34e67da |
| QG-1 — controlli locali | PASS per la frontiera implementata | fmt, build release, check, clippy, 2 integration test + 3 doctest, cross-check GNU |
| QG-2 — CI Linux/Windows | PASS per 39ed9b2 | [Run reale](https://github.com/UraganoAngelina/lintra/actions/runs/39390492669), entrambi i runner verdi; T-P5-03 richiede nuova CI |
| QG-3 — architettura | PASS per dominio/raw/compiled | Moduli privati, compiled con sola lettura e costruzione privata, piani tipizzati senza evaluator; nessuna logica normativa o backend nel core |
| QG-8 — documenti | PASS per lo scaffold | Copie identiche, nessun contratto modificato |
| G-PERSIST | NOT_STARTED | ADR approvata; solo modulo vuoto |
| G-SIOC-HIST | OPEN_POLICY | DEC-SIOC-HIST-01, non blocca scaffold |
| G-CAD-RUNTIME | NOT_STARTED | Richiede compiler, cross-record e persistenza |

## G0 — checklist T-P0-09 approvata

**Approvazione ricevuta il 2026-10-19:** «Approvo G0 con i rinvii indicati».
La risposta si riferisce alla checklist concreta del commit `407f89e`.
T-P0-01..08 verificati su artefatti effettivi: [report T-P0-09](docs/verification/T-P0-09.md).
Contratto formato 1 approvato in T-P0-07; profilo iniziale SIOC minimale
approvato in DEC-SIOC-MINIMAL-01. L’approvazione chiude G0 per la formalizzazione corrente con i rinvii
espliciti sotto; non dichiara implementate le capability o ricevuti i dataset.

| Voce DET §106 | Esito proposto, evidenza e rinvio | Task/accettazioni bloccate dal rinvio |
|---|---|---|
| Forma JSON canonica | CHIUSA per formato 1: SCHEMA_AUTHORING e meta-schema approvati; flow SIOC minimale e cinque trace validi. CAD canonico operativo rinviato alle task CAD: candidato ricevuto non promosso | Esecuzione/certificazione profilo CAD completo, non tipi raw generici |
| FieldType | CHIUSO sottoinsieme numeric/alphanumeric/date con lessico esplicito. Alphabetic/Decimal RINVIATI: assenza di repertorio e specifica separatore/segno/scala | T-P6-04 ramo Alphabetic e T-P9-03 Decimal non accettabili come supportati senza ampliamento approvato; nessun no-op |
| AlignmentRule | CHIUSA: exact/right_pad/left_pad, fill_byte e content_policy per campo; preserve_width per codici significativi. Nessun trim globale | Nessun blocco normativo aggiuntivo per il sottoinsieme corrente |
| Sintassi DSL | CHIUSA: AST tipizzato, operatori/capability chiusi, lowering esaustivo. Deroga limitata substring/età compiuta/delta anni già approvata; implementazione futura | Capability non implementate bloccano esecuzione profili che le richiedono; non la definizione dei contratti |
| Mapping OBB/FAC/NBB | CHIUSO: basi/condizioni approvate e fonte conservata per 218 campi, check_presence positivo; NBB non diventa optional incondizionato | Nessun blocco aggiuntivo; nuove ambiguità normative richiedono decisione |
| Regole cross-trace | CHIUSE strutturali SIOC: parent, PK e revoca con policy esplicite. RINVIATE congruenze cliniche/anno/età/temporali in minimal. CAD storico formalizzato nel contratto, runtime successivo | Golden e accettazione SIOC semantico completo; task runtime CAD e relativo gate finché compiler/persistenza e prove non esistono |
| Identità e join | CHIUSE: cinque PK e sei join canonici SIOC, inventario CAD e scope storico documentati; NotApplicable esplicito. Enforcement cross-run SIOC resta fuori dall’audit | Enforcement storico SIOC oltre audit senza DEC-SIOC-HIST-01 risolta; runtime CAD prima delle task persistenti |
| Formato dataset | CHIUSO contratto id/version/hash/path/normalizzazione e metadata; 15 schede verificate. RINVIATA fornitura dei dodici dataset ufficiali SIOC, esclusi dal minimale | Accettazione T-P11-05/06 su dati reali SIOC e golden di lookup completo; provider generici verificabili con dati sintetici dichiarati |
| Versioning reale tabelle | CHIUSO formato revisioni/hash, no latest; RINVIATE release/efficacia/provenienza reali delle tabelle SIOC mancanti e lacune temporali CAD documentate | Preflight e audit di tali tabelle in T-P11-03/06 e accettazioni normative complete; nessuna versione inventata |

Ulteriori limiti trasversali: encoding obbligatorio runtime, controllo
minuscole rinviato; layout export reali da chiarire prima del loro uso;
ricontrollo strutturale Rust sugli schemi eseguito in T-P3-04 senza issue; fixture/golden sintetiche
pianificate ma non ancora prodotte o approvate. Nessun blocco viene nascosto
come validazione superata. Tutti i dettagli sono nel report T-P0-08.

G0 è chiuso con i rinvii indicati e M0 REACHED. Il solo prerequisito G0
è soddisfatto; restano applicabili tutte le altre dipendenze e i blocchi
puntuali della tabella. La CI reale del commit 34e67da ha ora soddisfatto il prerequisito di T-P2-01.
ARCH/DET/Piano sorgenti non sono modificati; questa checklist registra
contratti approvati e rinvii secondo la procedura di T-P0-09.

## Milestone

| Milestone | Stato | Evidenza |
|---|---|---|
| M0 | REACHED | G0 approvato dall’autore con rinvii espliciti, 2026-10-19 |
| M1 | IN_PROGRESS | P1 e T-P2-01/02 complete; T-P2-03 parte autonoma in verifica, residuo applicativo aperto secondo proposta A approvata |
| M2–M9 | NOT_REACHED | Nessuna implementazione anticipata |

`DONE` riguarda i criteri della singola task. Non dichiara completati P1,
M1 o il gate multipiattaforma; G0 è approvato con i rinvii sopra.

## Task

| Task | Stato | Ultima sessione | Evidenza |
|---|---|---|---|
| T-P0-01 | DONE | 2026-10-16 | Commit 0b13581; inventario 11 categorie/27 righe; integrità e completezza verificate, review approvata |
| T-P0-02 | DONE | 2026-10-16 | Commit 419600d/6a4e9ac; 218 campi/0 gap, review indipendente e campione approvato dall'autore |
| T-P0-03 | DONE | 2026-10-19 | 218/218 campi e policy/AST verificati; fonti integre; conflitto CAD 2.12 esplicitato, nessuna correzione normativa applicata |
| T-P0-04 | DONE | 2026-10-19 | 218 note classificate, hash/riferimenti/DEC-05 verificati; 000, ID legacy SIOC e ambito iniziale CAD risolti |
| T-P0-05 | DONE | 2026-10-19 | Dossier SIOC/CAD: 74 regole, 10 join + ricerca esistenza; PK, identity e policy/DEC verificate; parent opzionale SIOC risolto dall’autore |
| T-P0-06 | DONE | 2026-10-16 | Commit 69dc995/a55a634; 15 schede/150 metadati, gap espliciti, review approvata |
| T-P0-07 | DONE | 2026-10-19 | Commit 72f9f5e: contratto completo/artefatti approvati dall’autore; 33 casi negativi e checker passati, limiti espliciti |
| T-P0-08 | DONE | 2026-10-19 | Profilo structural-minimal.1 autorizzato; flow+5 trace, 135 campi/5 PK/6 join, ledger, lint authoring e piano verificati; ricontrollo strutturale Rust T-P3-04 eseguito senza issue |
| T-P0-09 | DONE | 2026-10-19 | Checklist commit 407f89e approvata dall’autore: «Approvo G0 con i rinvii indicati»; G0 chiuso, M0 raggiunta |
| T-P1-01 | DONE | 2026-10-16 | Build e controlli Linux; 3 doctest; copie e gitignore verificati |
| T-P1-02 | DONE | 2026-10-16 | Commit 0186c0b; pin Rust 1.99.1, attributi/config; STD-VERIFY e cross-check GNU superati; review approvata |
| T-P1-03 | DONE | 2026-10-19 | CI su commit 34e67da: Linux GNU e Windows MSVC verdi, [run 39232752518](https://github.com/UraganoAngelina/lintra/actions/runs/39232752518) |
| T-P2-01 | DONE | 2026-10-19 | 7 newtype + ByteRange; 5 unit/2 integration/5 doctest; CI Linux/Windows verde su 6ac0fb8, run 39233324913 |
| T-P2-02 | DONE | 2026-10-19 | Mapping unico, 9 unit/2 integration/5 doctest e release; [CI 39234593522](https://github.com/UraganoAngelina/lintra/actions/runs/39234593522) verde Linux/Windows su 30af262 |
| T-P2-03 | IN_PROGRESS | 2026-10-19 | Proposta A approvata; identità/chiave/scope/summary/outcome verificati in locale e CI 39235805213, 70cdb5e; errore tecnico applicativo rinviato esplicitamente |
| T-P3-01 | DONE | 2026-10-19 | Raw DTO/AST formato 1, round-trip e negativi; 17 unit/2 integration/5 doctest; [CI 39236782609](https://github.com/UraganoAngelina/lintra/actions/runs/39236782609) verde Linux/Windows su 225419d |
| T-P3-02 | DONE | 2026-10-19 | Compiled immutabile, piani tipizzati e indice derivato; 25 unit/2 integration/10 doctest; [CI 39250930467](https://github.com/UraganoAngelina/lintra/actions/runs/39250930467) verde Linux/Windows su f04bb60 |
| T-P3-03 | DONE | 2026-10-19 | Repository filesystem/memoria e errori tecnici tipizzati; 25 unit/11 integration/10 doctest; [CI 39253152098](https://github.com/UraganoAngelina/lintra/actions/runs/39253152098) verde Linux/Windows su a39924d |
| T-P3-04 | DONE | 2026-10-19 | Linter fisico/tipi con 13 codici testati; 26 unit/25 integration/10 doctest; P0 senza issue; [CI 39257768950](https://github.com/UraganoAngelina/lintra/actions/runs/39257768950) verde Linux/Windows su a1ddabb |
| T-P3-05 | DONE | 2026-10-19 | Proposta A approvata; compiler locale/binding Rust-only e rifiuto costrutti esclusi; 28 unit/36 integration/10 doctest; [CI 39264362959](https://github.com/UraganoAngelina/lintra/actions/runs/39264362959) verde Linux/Windows su b658e05 |
| T-P3-06 | DONE | 2026-10-19 | Mappa ufficiale approvata 1→0.1.0 sviluppo; range/messaggi e 28 unit/42 integration/10 doctest; [CI 39269026995](https://github.com/UraganoAngelina/lintra/actions/runs/39269026995) verde Linux/Windows su 7522345 |
| T-P3-07 | DONE | 2026-10-19 | Registry senza cache, binding esplicito/cause preservate; 28 unit/48 integration/10 doctest; [CI 39272551912](https://github.com/UraganoAngelina/lintra/actions/runs/39272551912) verde Linux/Windows su f671517 |
| T-P4-01 | DONE | 2026-10-19 | Reader byte/tre policy, DEC-12 provvisorio; 29 unit/55 integration/10 doctest; [CI 39276070129](https://github.com/UraganoAngelina/lintra/actions/runs/39276070129) verde Linux/Windows su e18b9ba |
| T-P4-02 | DONE | 2026-10-20 | CI comune [39282115064](https://github.com/UraganoAngelina/lintra/actions/runs/39282115064) verde Linux/Windows su 7168a05; dettagli nel report |
| T-P4-03 | DONE | 2026-10-20 | CI comune [39282115064](https://github.com/UraganoAngelina/lintra/actions/runs/39282115064) verde Linux/Windows su 7168a05; dettagli nel report |
| T-P4-04 | DONE | 2026-10-20 | CI comune [39282115064](https://github.com/UraganoAngelina/lintra/actions/runs/39282115064) verde Linux/Windows su 7168a05; dettagli nel report |
| T-P4-05 | DONE | 2026-10-20 | [CI 39389225120](https://github.com/UraganoAngelina/lintra/actions/runs/39389225120) verde Linux/Windows su 46ab648; 118 test |
| T-P5-01 | DONE | 2026-10-20 | [CI 39389657166](https://github.com/UraganoAngelina/lintra/actions/runs/39389657166) verde Linux/Windows su a679f40; 123 test |
| T-P5-02 | DONE | 2026-10-20 | [CI 39390492669](https://github.com/UraganoAngelina/lintra/actions/runs/39390492669) verde Linux/Windows su 39ed9b2; 127 test |
| T-P5-03 | DONE_UNVERIFIED | 2026-10-20 | ValidationSink/VecSink/CompositeSink, ReportError con cause, fail-fast; 132 test locali, CI pendente |
| T-P6-01 | DONE | 2026-10-20 | [CI 39389225120](https://github.com/UraganoAngelina/lintra/actions/runs/39389225120) verde Linux/Windows su 46ab648; 118 test |
| T-PERSIST-01..12 | NOT_STARTED | — | Nessun contratto o backend |
| T-CAD-01..12 | NOT_STARTED | — | Nessun runtime CAD |

## Registro verifiche corrente — 2026-10-20, Linux locale

T-P5-03: fmt/check/clippy/test/release/diff-check PASS; 132 test
(29 unit, 93 integration, 10 doctest), cinque nuovi integration.
[Report](docs/verification/T-P5-03.md); CI da eseguire dopo factory.


T-P5-02: fmt/check/clippy/test/release/diff-check PASS;
29 unit + 88 integration + 10 doctest = 127.
[Report](docs/verification/T-P5-02.md), CI pendente.
T-P5-01: CI 39389657166 verde Linux/Windows su a679f40.


T-P5-01: fmt/check/clippy/test/release/diff-check PASS, 123 test
(29 unit, 84 integration, 10 doctest). [Report](docs/verification/T-P5-01.md).
T-P4-05/T-P6-01: CI 39389225120 completed/success Linux/Windows
su 46ab648. I pending seguenti sono cronologia superata.


T-P4-05: fmt/check/clippy/test/release/diff-check PASS.
29 unit + 79 integration + 10 doctest = 118, cinque integration nuovi.
Report [T-P4-05](docs/verification/T-P4-05.md); CI con T-P6-01 pendente.
T-P4-02..04: CI 39282115064 completed/success su entrambi i runner,
commit 7168a05. I pending sottostanti sono cronologia superata.


T-P6-01: fmt/check/clippy/test/release/diff-check PASS;
29 unit + 74 integration + 10 doctest = 113.
Report [T-P6-01](docs/verification/T-P6-01.md); CI pendente dopo CI input.
Prerequisito atomico T-P3-02 verde; attività indipendente durante la coda
esterna, nessuna dichiarazione di completamento P4/P5/M1.


T-P4-04: fmt/check/clippy/test/release/diff-check PASS;
29 unit + 70 integration + 10 doctest, sei integration nuovi.
Report [T-P4-04](docs/verification/T-P4-04.md). CI ancora da eseguire.
Run BOM 39278768796: Linux success; Windows tutti gli step success ma stato
aggregato bloccato in_progress. Nuova [CI comune 39282115064](https://github.com/UraganoAngelina/lintra/actions/runs/39282115064) sullo snapshot 7168a05;
nessun gate dichiarato completo prima del relativo esito aggregato.

Evidenze precedenti T-P4-03:

T-P4-03: fmt/check/clippy/test/release/diff-check PASS;
29 unit + 64 integration + 10 doctest; tre integration nuovi.
Report [T-P4-03](docs/verification/T-P4-03.md); CI ancora da eseguire.
T-P4-02 run 39278768796 su f600a4b ancora queued, non dichiarata verde.

Evidenze precedenti T-P4-02:

T-P4-02: fmt/check/clippy/test/release/diff-check PASS;
29 unit + 61 integration + 10 doctest, sei integration nuovi.
Report [T-P4-02](docs/verification/T-P4-02.md); CI pendente.

Evidenze precedenti T-P4-01:

T-P4-01: fmt/check/clippy/test/release/diff-check PASS;
29 unit + 55 integration + 10 doctest; un unit e sette integration nuovi.
Report [T-P4-01](docs/verification/T-P4-01.md); CI 39276070129 verde Linux/Windows su e18b9ba.

Evidenze precedenti T-P3-07:

T-P3-07: fmt/check/clippy/test/release/diff-check PASS;
28 unit + 48 integration + 10 doctest, sei integration nuovi.
Report [T-P3-07](docs/verification/T-P3-07.md); CI 39272551912 verde Linux GNU/Windows MSVC su f671517.

Evidenze precedenti T-P3-06:

T-P3-06: fmt/check/clippy/test/release/diff-check e generatore --check PASS;
28 unit + 42 integration + 10 doctest; sei integration nuovi rispetto a
T-P3-05. Range/formato assente/non conforme e messaggi/mappa verificati.
[CI 39269026995](https://github.com/UraganoAngelina/lintra/actions/runs/39269026995) PASS Linux GNU/Windows MSVC su 7522345; report [T-P3-06](docs/verification/T-P3-06.md).

Evidenze precedenti T-P3-05:

T-P3-05: fmt/check/clippy/test/release/diff-check e generate_raw_model --check
PASS; 28 unit + 36 integration + 10 doctest. Nucleo sintetico positivo;
esempi completi/SIOC esplicitamente rifiutati per costrutti esclusi.
Report [T-P3-05](docs/verification/T-P3-05.md); [CI 39264362959](https://github.com/UraganoAngelina/lintra/actions/runs/39264362959) PASS Linux GNU/Windows MSVC su b658e05.

Evidenze precedenti T-P3-04:

T-P3-04: fmt/check/clippy/test/release e diff-check tutti exit 0;
26 unit + 25 integration + 10 doctest. Generazione raw --check PASS.
Ricontrollo Rust profilo P0 SIOC (5 trace/135 campi) e tre flow sintetici:
nessuna issue strutturale. Report [T-P3-04](docs/verification/T-P3-04.md).
[CI 39257768950](https://github.com/UraganoAngelina/lintra/actions/runs/39257768950) PASS su a1ddabb, Linux GNU/Windows MSVC; nessuna compilabilità normativa presunta.

Evidenze documentali precedenti P0 (storiche):

Evidenze e proposta: [T-P0-03](docs/verification/T-P0-03.md).

| Comando | Esito |
|---|---|
| `python3 docs/verification/check_presence.py` | PASS, exit 0: 135 SIOC + 83 CAD; dieci trace, policy/AST fedeli, quattordici fonti integre |
| `python3 docs/verification/check_notes.py` | PASS, exit 0: 218 note, indici/hash/pagine, constraint e ID regola, conteggi classi e citazioni DEC-05 |
| `python3 docs/verification/check_rules_and_joins.py` | PASS, exit 0: dieci trace, 74 regole, dieci join con cardinalità/policy o DEC, trenta presence, tre gruppi, profilo CAD completo e riferimenti; policy SIOC NotApplicable e spiegazione verificate |
| `python3 docs/verification/check_fixtures_plan.py` | PASS, exit 0: 135 campi/72 regole/376 specifiche di caso, indici e hash delle fonti, template/casi condizionali/constraint, obbligo T-P3-04; nessuna fixture eseguita |
| `uv run … --with jsonschema==4.26.0 python docs/verification/check_authoring.py` | PASS: 2 meta-schemi, 3 flow e manifest; 4 hash, copertura 19 constraint/14 assertion legacy e 13 nodi Expr; 33 casi negativi rifiutati; nessuna compilazione DSL Rust |
| `cargo fmt --all -- --check` | PASS |
| `cargo check --all-targets --locked` | PASS |
| `cargo clippy --all-targets --locked -- -D warnings` | PASS |
| `cargo test --locked` | PASS: due integration test e tre doctest |

Nessun Rust modificato e nessun test normativo inventato. Nessuna nuova
run Windows/GitHub eseguita o presunta. Controlli documentali diretti;
nessun nuovo agente o review tramite framework.

## Registro storico — 2026-10-16, Linux locale

[Trascrizione finale](docs/verification/FOUNDATIONS.md),
[T-P1-02](docs/verification/T-P1-02.md),
[T-P0-01](docs/verification/T-P0-01.md),
[T-P0-02](docs/verification/T-P0-02.md),
[T-P0-06](docs/verification/T-P0-06.md).

| Comando / controllo | Esito reale |
|---|---|
| `rustc --version` | 1.99.1 (58cf8f9c3 2026-10-13), pin attivo |
| `cargo fmt --all -- --check` | PASS, exit 0 |
| `cargo check --all-targets --locked` | PASS, exit 0 |
| `cargo clippy --all-targets --target x86_64-unknown-linux-gnu --locked -- -D warnings` | PASS, exit 0 |
| `cargo test --target x86_64-unknown-linux-gnu --locked` | PASS, exit 0: 2 integration test + 3 doctest |
| `cargo build --release --target x86_64-unknown-linux-gnu --locked` | PASS, exit 0; sola verifica locale |
| `cargo check --all-targets --target x86_64-pc-windows-gnu --locked` | PASS, exit 0; nessun linking/esecuzione Windows |
| `cargo run --locked` | PASS, exit 0, nessun output applicativo |
| Manifest Cargo e moduli | Nessuna dipendenza; undici mod.rs senza codice |
| Copie ARCH/DET/PLAN/ADR/matrice | PASS, ancora identiche byte per byte alle fonti |
| Roundtrip Git attributi | PASS, 66 confronti byte in tre modalità autocrlf |
| Review indipendenti T-P1-02/T-P0-01/T-P1-03 | Approvate, nessun finding implementativo |
| T-P0-02 | 218/218 proiezioni esatte, 0 gap/overlap; CAD 83/83 righe e note riscontrate nel PDF; 16 fonti conformi agli hash |
| Accettazione umana T-P0-02 | Ricevuta: «Campione verificato: approvo»; nessuna approvazione runtime/G0 dedotta |
| T-P0-06 | 15 schede/150 metadati, 12 dataset SIOC mancanti e 3 CAD forniti riconciliati; review approvata |
| `git ls-remote --heads origin` | Prima fallisce per DNS sandbox; rieseguito con escalation, exit 128 `unable to get password from user`: autenticazione non disponibile |
| Secondo `git ls-remote --heads origin` fuori sandbox | Exit 128: `could not read Username for 'https://github.com': No such device or address` |
| GitHub Actions Linux/Windows | Non eseguite dall'agente; nessun push dell'agente effettuato |

## Registro storico bootstrap T-P1-01 — 2026-10-16

Trascrizione finale: [VERIFICATION.md](docs/bootstrap/VERIFICATION.md).

| Comando / controllo | Esito reale |
|---|---|
| `python3 VERIFY_KIT.py` nel kit | FAIL, exit 1: solo i quattro extra elencati |
| Python: SHA-256/dimensioni manifest; confronto copie | PASS: 196 file censiti; 7 copie identiche |
| `cargo fmt --all` | PASS, exit 0; mod.rs contengono solo newline emesso da rustfmt |
| `cargo fmt --all -- --check` | PASS, exit 0 |
| `cargo build` | PASS, exit 0; genera lockfile iniziale |
| `cargo build --locked` | PASS, exit 0 |
| `cargo check --all-targets --locked` | PASS, exit 0 |
| `cargo clippy --all-targets --locked -- -D warnings` | PASS, exit 0 |
| `cargo test --locked` | PASS: 3 doctest (1 positivo, 2 compile-fail), 0 unit test |
| `cargo run --locked` | PASS, exit 0; entrypoint senza output applicativo |
| `cargo metadata --no-deps --locked --format-version 1` | PASS: singolo package, lib e bin, zero dipendenze |
| `git check-ignore --no-index -q -- <path>` su 43 percorsi | PASS: 30 sensibili/artefatti esclusi, 13 sorgenti/fixture sintetiche ammessi |
| Ispezione file | PASS: 11 mod.rs senza codice; nessun .txt/export nel repository |
| `git diff --check`; `git status` | Eseguiti; nuovi file non committati, nessun preesistente modificato |
| Cross-check `x86_64-pc-windows-gnu` | Non eseguito: target non installato; installazione/config in T-P1-02 |
| CI Linux/Windows, release, test normativi | Non eseguiti: task successive, fuori scope |

I doctest verificano importabilità e divieto di importare `engine` e
`persistence` dall'esterno (DET §94). Nessun test di validazione fittizio.

## Problemi noti e decisioni aperte

- **CNF-FORMAT-01 RISOLTO:** autore: «Approvo la mappa con baseline di
  sviluppo 0.1.0». Formato 1 → 0.1.0 sviluppo/compiler b658e05; 0 e
  2..=u32::MAX non definiti/nessun eseguibile compatibile dichiarato.
  [Mappa ufficiale](docs/schema-authoring/FORMAT_COMPATIBILITY.md).
  Nessuna release futura inventata, prima proposta provvisoria superata;
  T-P3-06 verificata in locale e CI Linux/Windows.


- **CNF-COMPILER-01 RISOLTO:** autore: «Approvo A: compiler incrementale con
  rifiuto esplicito dei costrutti esclusi». Gate per costrutti presenti nel solo
  sottoinsieme verificato; nessuna capability completa pubblicizzata dal motore.
  Binding encoding Rust-only attuato senza cambiare il JSON; DEC-13 regionale
  resta aperta. [Proposta approvata](docs/verification/T-P3-05-preflight.md).


- **CNF-DOMAIN-01 risolto con proposta A approvata:** SchemaKey nel dominio,
  ScopeSummary Trace/Flow, parte autonoma prima dell’errore tecnico applicativo.
  [Dossier](docs/verification/T-P2-03-preflight.md). Il precedente rifiuto è
  superato dall’approvazione esplicita integrale. Riordino atomico autorizzato
  dopo verifica della parte autonoma; T-P2-03/M1 restano aperte sul residuo.

- **SIOC completo rinviato:** dodici dataset con versioni/hash/metadata ICD9
  assenti; controlli semantici esclusi dal profilo minimale per decisione
  dell’autore. mar01 approvato, encoding richiesto al chiamante senza dedurlo
  dai campioni. Nessuna domanda precedente resta pendente su questi punti.
- **Export SIOC:** larghezze aggregate diverse dai layout normativi; chiarire
  il formato prima dell’uso come input del futuro validatore. Nessun adattatore
  o stripping dedotto. [Evidenze](docs/verification/T-P0-08.md).
- **Contratto T-P0-07 approvato:** DEC-AUTHORING-FORMAT-01 e forme dei
  piani insiemi/consenso ratificate; restano da implementare con i gate.
- Dossier ed evidenze: [T-P0-05](docs/verification/T-P0-05.md).
- Dettagli e citazioni in [T-P0-04](docs/verification/T-P0-04.md). Le clausole
  SIOC 1.6/4.28 mantengono il non-enforcement v1 già ricevuto; G03, formato
  UVMD ricevuto e sentinel Barthel 99 non sono nuove decisioni aperte.
- Controllo completo kit FAIL per gli export extra, nessun impatto bloccante
  su T-P1-01: documenti censiti integri e dati esclusi dal nuovo repository.
- Accesso GitHub ora verificato tramite gh/keyring fuori sandbox. Push
  non forzato 69dc995→34e67da e CI manuale sul branch preesistente;
  precedenti limiti di autenticazione sono storici, non più bloccanti.
- `DEC-SIOC-HIST-01`: fino ad approvazione solo audit cross-run;
  non dedurre nuovi errori normativi dalla ripetizione di una PK.
- Backend storage, gestione chiavi/retention/protezione prima dell'uso reale:
  decisioni future, nessun database scelto per default.
- G0 CAD e versione runtime della DSL ancora da verificare.
- Metadati SIOC ricevuti: manifest/flow dichiarano 2026-04-20, trace split e
  annidati 2025-12-16; FREEZE_REPORT distingue baseline 2025 e profilo 2026.
  Entrambi conservati e documentati nelle tabelle; nessuna ratifica della
  versione runtime dedotta dalla sola approvazione della trascrizione.
- CAD posizione 60: G03 già risolta (blank fuori UCPDOM); i pending/undecided
  ancora in alcuni artefatti sono metadati storici superati, non nuova norma aperta.

## Decisioni della sessione corrente

- **T-P5-03 / DET §54/§118/§156:** ReportError reale nel report, con cause
  tecniche; nessun anticipo di ValidationRunError applicativo. Composite
  ordinato e terminale dopo il primo Err, nessun rollback o atomicità
  implicita. VecSink esplicitamente test/piccoli casi, non default engine.


- **T-P5-02 / DET §43/§50:** FieldViolation è solo il payload già richiesto
  da field_error; proprietà validators, nessun validatore anticipato.
  Template nel solo engine/error_factory, dati machine-readable preservati.
  Precondizioni metadata preflight/reader mantenute, niente panic o default
  inventati, conversione e rendering esistenti riusati.


- **T-P5-01 / DET §5/§34/§52/§147:** DiagnosticValue come valore neutro
  nel dominio e re-export input invariato: preserva confini e API pubblica,
  nessun rendering nel dominio. FieldDiagnostic::from_range centralizza
  la conversione; validate verifica metadati pubblici prima dell’emissione.
  Non inferisce scope dal codice/testo; ErrorFactory futura nel motore.


- **T-P4-05 / DET §117, task:** InputError esteso con RecordRead e checkpoint
  last_completed_line, causa io::Error intatta, getter per riga one-based.
  Preflight senza riga; overflow esplicito senza saturazione. Nessun cambio
  di read_next o trasformazione di guasti tecnici in violazioni normative.


- **T-P4-04 / DET §33–34:** EncodingSpec esistente riusata senza duplicati.
  Rendering diagnostico esplicito UTF-8/Windows-1252/Latin1; controlli e
  backslash escaped, hex esatto se lossy/escaped o profilo Other sconosciuto.
  Tabella primaria Unicode/Microsoft verificata nel report; nessuna dipendenza.
  Non seleziona codec regionale o amplia il compiler, DEC-13 resta aperta.
- **CI in attesa:** codice indipendente T-P4-03 e T-P4-04 sviluppato e
  verificato una task alla volta mentre GitHub teneva in coda la run BOM.
  Gate di entrambe ancora aperti; suite CI comune sullo snapshot contenente
  entrambi i commit, dopo run BOM, evita annullamenti del concurrency group.


- **T-P4-02 / DET §30, DEC-12:** Reject default provvisorio; strip dei soli
  marker Unicode noti iniziali su richiesta, senza transcodifica o selezione
  encoding. Probe limitato a 4 byte, replay del payload, niente Seek.
  Signatures verificate alla fonte Unicode citata nel report. InputError
  comune estratto in error.rs per due consumer, export pubblico invariato;
  BomRejected è tecnico con rimedio esplicito, nessuna CLI ancora promessa.


- **T-P4-01 / DET §28–29, DEC-12 provvisorio:** righe vuote conservate
  come record di lunghezza zero, EOF senza record fantasma; nessuna norma
  inferita o DEC-12 chiusa. LfOnly conserva CR, CrlfOnly conserva LF isolati
  come dati non terminatori; Auto rimuove LF o CRLF e nulla oltre.
- **T-P4-01 / integrità trasporto:** I/O e overflow sono errori tecnici
  con reader terminale dopo fallimento, per non produrre code troncate dopo
  consumo parziale. Buffer sempre clear e nessuna avanzata di contatore
  su errore/EOF. Nessuna nuova dipendenza o gestione BOM anticipata.


- **T-P3-07 / CNF-DOMAIN-01 già approvato, DET §5/§62/§147:** il registry
  del modulo schema restituisce SchemaResolutionError con i payload reali
  Load/Compilation; application mapperà all’errore di run quando completo.
  Nessuna dipendenza schema→application, nessun placeholder o violazione
  normativa nell’errore tecnico. Rifinitura dichiarata nel report.
- **T-P3-07 / binding già approvato:** encoding esplicito nel costruttore
  Registry e autorevole per il run, nessun locale/default. resolve resta per
  sola chiave, ricarica e ricompila sempre. Identità del raw verificata anche
  per repository custom, nessuna cache o selezione implicita di versione.


- **CNF-FORMAT-01 ratificato:** l’autore ha approvato la mappa dopo
  l’inventario completo. Baseline minima sviluppo 0.1.0 per formato 1,
  compiler b658e05; 0 e 2..=u32::MAX non definiti, senza eseguibile dichiarato.
  Messaggi distinguono minima nota da non applicabilità di formati non
  definiti. La versione corrente del package non è una release futura;
  le entry future richiederanno versioni minime approvate. SCHEMA_AUTHORING
  integrato, nessuna riscrittura di ARCH/DET/Piano o dei JSON normativi.
- **T-P3-06 / DET Addendum A.1:** EngineCapabilities e SchemaFormatVersion
  in modulo compatibility condiviso da lint e compiler; range effettivo 1..=1.
  Nessuna configurazione del chiamante può cambiare il compiler corrente;
  il supporto del formato resta distinto dalle capability normative.


- **CNF-COMPILER-01 / proposta A approvata dall’autore:** compilazione
  incrementale di local.v1 per ogni costrutto effettivamente verificato;
  altri costrutti/capability rifiutati, nessuna capability del motore dichiarata
  completa. Integrazione in SCHEMA_AUTHORING, nessuna modifica ad ARCH/DET/Piano.
- **T-P3-05 / binding encoding e modello:** RawEncodingSpec.resolved escluso
  dal JSON e generato riproducibilmente; UTF-8 esplicito nel solo nucleo attuale,
  nessun default o scelta regionale. Source metadata conservati nel compiled,
  factory sempre private e conversione unica in compiler.rs. DET §11/§157 e
  contratto di authoring; dettagli e limiti nel report T-P3-05.


- **T-P3-04 / DET §12/§141–142 e ARCH §14:** SchemaIssue aggiunge severity
  richiesta da §142; enum non_exhaustive con i soli 13 controlli consegnati.
  Tipi/alias sconosciuti restano errori di parsing raw, parametri fisici sono
  verificati dal lint. Ordine esplicito flow/trace/campo con membership hash
  indipendente dall’emissione. Nessuna issue di compiler futura inventata.
  Coordinate sorgente one-based, conversione runtime rinviata a T-P3-05;
  nessuna modifica alla proprietà dei tipi o alla semantica normativa.


- **T-P0-07 approvata dall’autore il 2026-10-19:** «Approvo il contratto
  T-P0-07», revisione completa del commit 72f9f5e. Formato 1, policy
  persistenti, insiemi/consenso e piani compiled ratificati; non approva
  automaticamente dati mancanti, G0 o runtime.

- **CNF-LINTRA-DSL-01 risolta dall’autore il 2026-10-19:** «Approvo la
  deroga circoscritta (consigliata)». Contratti dei soli substring, età
  compiuta e differenza anni necessari ai profili; implementazione futura
  nelle task pertinenti con gate/test. [Decisione](docs/normative/DECISIONS.md).
  Non approva automaticamente il contratto completo di authoring.
- **Tool documentale:** jsonschema 4.26.0 in ambiente/cache temporanei /tmp
  via uv, download autorizzato tramite escalation; nessuna dipendenza Rust
  o pacchetto aggiunto al progetto. Fonte primaria [EXT] nel README artefatti.

- **DEC-05-SIOC-OPTIONAL-PARENT risolta dall’autore il 2026-10-19:**
  `NotApplicable` per target assente, con evidenza esplicita di anagrafica
  presente e dati del ricovero correlato assenti; nessun errore parent.
  Superata la proposta Unknown per questo caso, senza estendere la policy
  a target ambiguo o invalido. [Decisione registrata](docs/normative/DECISIONS.md).

- **DEC-05-CAD-VALUTAZIONE-INIZIALE risolta dall’autore il 2026-10-19:**
  «Nello storico dell’evento», incluso invio corrente come nella domanda.
  Non richiede un’iniziale nuova in ciascun invio né esattamente una.
  Namespace ULSS conservato, contesto indisponibile distinto da violazione.
  [Decisione registrata](docs/normative/DECISIONS.md).

- **DEC-05-SIOC-ID-2026 risolta dall’autore il 2026-10-19:** per ricoveri
  iniziati nel 2025 e terminati nel 2026, ID alfanumerico valorizzato,
  diverso da tutti zeri, larghezza fisica 20 byte e padding ricevuto.
  Nessun controllo di lunghezza logica otto, sole cifre, prefisso anno o
  progressivo in questo ramo. [Decisione registrata](docs/normative/DECISIONS.md).
  Fonti FROZEN inalterate; applicazione nella task di schema pertinente.

- **DEC-05-CAD-ICPC-000 risolta dall'autore il 2026-10-19:** «Confermo: 000
  valido, spazi mancanti». Campo obbligatorio, nessun lookup ICPC aggiunto.
  [Decisione registrata](docs/normative/DECISIONS.md); correzione della
  configurazione nella task pertinente, senza alterare il kit originale.

## Decisioni della sessione precedente — 2026-10-16

- Esecuzione continua/parallelismo autorizzati: implementer su file disgiunti,
  coordinatore unico per stato e Git; review indipendente per task e finale.
- Branch di lavoro `work/p1-foundations`; identità Git fornita dall'autore;
  nessuna password/token in file, comandi o URL remote.
- Rust 1.99.1 scelta dall'autore, installazione completata; nessuna nuova
  dipendenza Rust o mutazione delle fonti architetturali/normative.
- CI su Ubuntu 24.04/Windows 2025, target nativi espliciti, CRT statica MSVC,
  permessi di sola lettura, checkout fissato a SHA verificato. Nessuna release
  o scelta della baseline Linux distribuibile anticipata.
- T-P2-01 resta bloccata dalla CI reale. P0 documentale procede dove gli
  input consentono inventario/trascrizione, senza approvare norme o G0.

## Decisioni storiche bootstrap T-P1-01

| Decisione | Motivazione e fonte |
|---|---|
| Repository in `lintra/`, branch `main` | `.git` tecnica del workspace vuota e in sola lettura; DET §4 raccomanda `lintra/`; kit/dati separati senza alterare permessi |
| Package 0.1.0, edition 2024, `publish = false` | Metadati iniziali con toolchain disponibile; nessuna scelta MSRV/licenza/pubblicazione, pin in T-P1-02 |
| Moduli library privati, `cli` solo in main | DET §5/§94; nessuna API prematura |
| Undici mod.rs senza codice | T-P1-01, ADR-PERSIST-001: nessun tipo/driver/database anticipato |
| Lockfile iniziale incluso | Generato dalla build necessaria per i controlli `--locked`; non completa T-P1-02 |
| Doctest sul confine pubblico | Casi positivo/negativi pertinenti a DET §94, senza dominio |
| Nessun commit/staging/push | Nuovi file disponibili per revisione; nessuna identità Git inventata |
| Nessuna nuova dipendenza | N-26/DEC-15: lockfile contiene solo `lintra` |

## Documenti da aggiornare

ARCH, DET, PLAN e ADR sorgenti rimangono inalterati. CNF-LINTRA-DSL-01
risolta dalla deroga circoscritta approvata dall’autore; l’allineamento
contrattuale delle task in SCHEMA_AUTHORING.md/lowering.json è stato ratificato
con l’approvazione completa; implementazione solo dopo prerequisiti e gate.
Il contratto completo T-P0-07 è approvato; nessun sorgente architetturale
originale viene riscritto per registrare l’accettazione.
Contratti pubblici Rust consegnati e relativi confini documentati nei report delle task; signature compile(raw) invariata, nessun runtime normativo anticipato.

## Plan change log

- **2026-10-19, autore:** approvato il contratto completo T-P0-07
  (commit 72f9f5e), inclusi formato e allineamenti compiled. Nessuna
  modifica autonoma ai documenti sorgente, nessun avanzamento dei gate runtime.

- **2026-10-19, autore:** approvata la proposta A, deroga circoscritta per
  substring, età compiuta e differenza anni. Implementazione futura con
  prerequisiti/gate/test; nessuna modifica automatica a Piano o DET.
  Allineamento delle task proposto nel contratto T-P0-07 da revisionare.

- **2026-10-16, autore (dal seed):** autorizzato adeguamento ARCH/DET al piano,
  nome Lintra confermato, persistenza promossa a core; ADR-PERSIST-001 approvata.
- **Sessione successiva:** esecuzione continua e parallela autorizzata
  dall'autore; nessuna modifica al Master Implementation Plan. Il piano
  operativo locale organizza la frontiera realmente eseguibile.

## Handoff

HANDOFF — 2026-10-20 — T-P5-03 locale verificata

- **File:** report/sink.rs, report/composite.rs, report/mod.rs, lib.rs,
  tests/validation_sinks.rs, report/stato.
- **Evidenza:** 132 test, fmt/check/clippy/release/diff-check verdi locali.
- **Gate:** CI T-P5-03 da avviare dopo run factory 39390492669.
- **Residui:** il service mapperà ReportError reale a Err(Report),
  T-P2-03/M1 aperte; nessuna transazione di output anticipata.
- **Next:** chiudere CI, T-P5-04 con DEC-02 già risolta.
- **Git:** preservato/escluso il piano storico preesistente dell’autore.


HANDOFF — 2026-10-20 — T-P5-02 locale verificata

- **File:** engine/error_factory.rs, validators/violation.rs, moduli/export,
  tests/error_factory.rs, snapshot messaggi, report e stato.
- **Test:** 127 verdi; fmt/check/clippy/release/diff-check exit 0.
- **Gate:** T-P5-01 DONE in CI 39389657166; nuova CI factory da eseguire.
- **Next:** T-P5-03 dipende solo da T-P5-01 già verde, può procedere
  durante la CI factory secondo il mandato e MIP §14.2.
- **Residui:** T-P2-03/M1 ancora aperte, nessuna semantica normativa nuova.
- **Git:** escluso il piano storico preesistente dell’autore.


HANDOFF — 2026-10-20 — T-P5-01 locale verificata

- **Consegna:** diagnostica completa e conversione coordinate; enum codici
  esistente riusata, DiagnosticValue neutro senza dipendenza domain->input.
- **File:** domain/errors.rs, domain/mod.rs, input/encoding.rs, lib.rs,
  tests/validation_diagnostics.rs, report/stato; report CI T-P4-05/T-P6-01.
- **Verifiche:** 123 test, fmt/check/clippy/release/diff-check verdi locali.
- **Gate:** T-P4-05/T-P6-01 DONE in CI 39389225120; CI T-P5-01 pendente.
- **Next:** CI, quindi T-P5-02 autonomamente; T-P2-03/M1 ancora aperte.
- **Git:** piano storico modificato dall’autore escluso dal commit.


HANDOFF — 2026-10-20 — T-P4-05 locale verificata

- **Consegna:** matrice memoria/file e contesto di riga negli errori I/O.
- **File:** input/error.rs, input/reader.rs, tests/input_matrix.rs,
  tests/record_reader.rs, report/stato e consuntivo CI T-P4-02..04.
- **Evidenza:** 118 test, fmt/check/clippy/release/diff-check tutti exit 0.
- **Gate:** nuova CI necessaria per T-P4-05 e T-P6-01.
- **Git:** file storico di piano preesistente conservato ed escluso.
- **Next:** CI, poi T-P5-01 secondo dipendenze atomiche autorizzate.


HANDOFF — 2026-10-20 — chiusura CI input / avvio T-P4-05

- **DONE:** T-P4-02, T-P4-03 e T-P4-04, CI 39282115064 su 7168a05.
- **Evidenza:** Linux 118158318317 e Windows 118158317939 completed/success.
- **File:** aggiornati i tre report e questo consuntivo.
- **Next:** T-P4-05 iniziata autonomamente, ora tutti i prerequisiti verdi.
- **Residui:** T-P6-01 necessita CI; T-P2-03/M1 ancora aperte come approvato.


HANDOFF — 2026-10-19 — T-P6-01 locale verificata

- **Consegna:** is_missing pura sulle quattro varianti già compiled.
- **File:** validators/presence.rs, validators/mod.rs, lib.rs,
  tests/missing_value.rs, report e stato. Nessun crate o baseline modificato.
- **Test:** 113 test, fmt/check/clippy/release/diff-check verdi in locale.
- **Gate:** CI input 39282115064 in attesa; CI di questa task da eseguire.
- **Decisione:** helper senza controllo width; stage record-length resta
  precedente e obbligatorio. Nessun default per gli zeri.
- **Git:** escluso il file di piano storico già modificato dall’autore.
- **Next:** chiudere CI input, poi T-P4-05 senza conferma ordinaria.


HANDOFF — 2026-10-19 — T-P4-04 / rendering locale verificato

- **Stato:** DONE_UNVERIFIED; 109 test locali verdi e fmt/check/clippy/release/diff.
- **File:** input/encoding.rs, input/mod.rs, lib.rs, diagnostic_rendering.rs,
  report/stato. Nessuna dipendenza nuova o encoding regionale dedotto.
- **Fatto:** rendering esplicito UTF-8/CP1252/Latin1, byte non decodificabili
  preservati con hex, controlli/backslash visibili, Other senza fallback.
- **Gate:** T-P4-02 CI 39278768796 in_progress; poi CI comune T-P4-03/04
  sullo snapshot comprendente i commit e tutti i rispettivi test.
- **Git:** work/p1-foundations; task atomiche in commit distinti, modifica
  storica estranea intatta. Nessuna falsa dichiarazione DONE per CI pendente.
- **Next:** completare gate T-P4-02..04, poi T-P4-05 automaticamente.
  T-P2-03/M1 e DEC-13 restano aperte.


HANDOFF — 2026-10-19 — T-P4-03 / locale verificato, attesa CI BOM

- **Stato:** DONE_UNVERIFIED; 103 test locali verdi e fmt/check/clippy/release/diff.
- **File:** input/record.rs, lib.rs, tests/record_view.rs, report/stato.
- **Fatto:** RecordView/FieldSlice borrowed, ByteRange::slice, nessun trim/copia,
  None fuori range. Lifetime e puntatori testati con compiled reali.
- **Gate:** T-P4-02 CI 39278768796 resta queued; nessun nuovo dispatch
  concorrente per non annullarla. Nessuna di queste due task dichiarata DONE.
- **Avanzamento:** MIP §14.2 e prerequisiti atomici verificati consentono
  T-P4-04 indipendente (solo T-P2-01), una task di codice alla volta.
- **Git:** work/p1-foundations, modifica storica estranea intatta;
  nessuna nuova dipendenza o fonte normativa modificata.


HANDOFF — 2026-10-19 — T-P4-02 / BOM locale verificato

- **Stato:** DONE_UNVERIFIED, CI Linux/Windows pendente; nessun blocco locale.
- **File:** input/source.rs, input/error.rs e tests/input_bom.rs nuovi;
  reader/mod/lib aggiornati; report e stato. Probe 4 byte e replay senza Seek.
- **Evidenze:** fmt/check/clippy/test/release/diff-check PASS;
  29 unit + 61 integration + 10 doctest. Marker/probe/I/O/CRLF verificati.
- **Decisioni:** Reject già previsto come default, strip solo esplicito;
  nessuna transcodifica o scelta encoding; InputError pubblico preservato.
  Fonte primaria Unicode nel report; nessuna nuova dipendenza o baseline modificata.
- **Git:** work/p1-foundations; modifica storica estranea intatta.
- **Next:** CI, poi T-P4-03 automaticamente; T-P2-01/T-P3-02 verdi.
  T-P2-03/M1 restano aperte sul residuo applicativo.


HANDOFF — 2026-10-19 — T-P4-01 completata / avvio automatico T-P4-02

- **Stato:** DONE; CI 39276070129 su e18b9ba, Linux job 118140591245 e
  Windows job 118140590466 completed/success; 94 test locali verdi.
- **File:** input/{reader,record,mod}.rs, lib.rs, tests/record_reader.rs,
  report/stato. Nessun blocco o dipendenza nuova; sorgenti normativi invariati.
- **Git:** codice e18b9ba su origin, chiusura solo documentale, modifica
  storica estranea preservata. Timeout TLS iniziale superato con retry.
- **Next:** T-P4-02 BOM, prerequisito ora verde; avvio senza conferma ordinaria.


HANDOFF — 2026-10-19 — T-P4-01 / lettore locale verificato

- **Stato:** DONE_UNVERIFIED, solo CI Linux/Windows pendente.
- **File:** input/{reader,record,mod}.rs, lib.rs, tests/record_reader.rs,
  report e stato. RawRecord prestato e RecordReader streaming senza trim.
- **Evidenze:** fmt/check/clippy/test/release/diff-check PASS;
  29 unit + 55 integration + 10 doctest. Pointer reuse/lifetime, terminatori,
  vuoti/EOF, I/O parziale e contatore checked verificati con dati sintetici.
- **Decisioni:** DEC-12 provvisorio già autorizzato, nessun record fantasma;
  sole sequenze terminatrici configurate rimosse; terminale dopo guasto I/O.
- **Residui:** InputError reale disponibile ma T-P2-03/M1 restano aperte;
  nessuna gestione BOM, validator o storage anticipata.
- **Git:** work/p1-foundations; modifica storica estranea intatta.
- **Next:** CI della task, poi T-P4-02 automaticamente, prerequisito T-P4-01.


HANDOFF — 2026-10-19 — T-P3-07 completata / avvio automatico T-P4-01

- **Stato:** DONE, CI 39272551912 su f671517 completed/success Linux GNU
  job 118128384953 e Windows MSVC job 118128385000; 86 test locali verdi.
- **File:** registry.rs, tests/schema_registry.rs, mod/lib, AGENTS, report/stato.
- **Decisioni:** errori schema locali preservano payload reali, encoding esplicito;
  nessuna cache o dipendenza applicativa nel modulo schema.
- **Git:** codice f671517 su origin, chiusura solo documentale;
  modifica storica estranea intatta. Nessun blocco; residuo T-P2-03/M1 invariato.
- **Next:** T-P4-01 avviata senza nuova conferma, prerequisito T-P2-01 verde;
  seguire DEC-12 provvisorio già autorizzato nel Piano.


HANDOFF — 2026-10-19 — T-P3-07 / registry locale verificato

- **Stato:** DONE_UNVERIFIED; nessun blocco locale, CI Linux/Windows pendente.
- **File/fatto:** registry.rs e tests/schema_registry.rs nuovi, mod/lib,
  AGENTS.md, report e stato. Carica chiave esatta, binding esplicito, compila
  a ogni resolve e restituisce Arc senza cache; nessuna nuova dipendenza.
- **Verifiche:** fmt/check/clippy/test/release/diff-check PASS;
  28 unit + 48 integration + 10 doctest, sei integration nuovi.
- **Contratti/decisioni:** errore locale con cause Load/Compilation per
  rispettare proprietà applicativa già approvata di ValidationRunError;
  future conversioni nel livello applicativo. Encoding del chiamante obbligatorio.
- **Residui:** T-P2-03/M1 restano aperte; nessuna capability normativa
  aggiuntiva, validator o storage. Fonti originali e baseline invariate.
- **Git:** work/p1-foundations; modifica storica estranea preservata.
- **Next:** completare CI e iniziare T-P4-01 senza attendere conferma,
  come richiesto dall’autore; prerequisito atomico T-P2-01 verificato.


HANDOFF — 2026-10-19 — T-P3-06 completata / CI reale verde

- **Stato:** DONE; QG-2 PASS sul commit codice 7522345. CNF-FORMAT-01
  risolto dalla mappa approvata, nessuna decisione della task ancora pendente.
- **Fatto:** descrittore formato tipizzato/range 1..=1 condiviso da lint/compiler;
  formato obbligatorio senza default; minime e messaggi basati sulla mappa.
- **Decisione:** formato 1→minimo 0.1.0 sviluppo/compiler b658e05; 0 e
  2..=u32::MAX non definiti, nessun eseguibile compatibile dichiarato. Input
  assenti/non conformi rifiutati in caricamento. Nessuna release pubblicata inventata.
- **Evidenze:** [run 39269026995](https://github.com/UraganoAngelina/lintra/actions/runs/39269026995),
  Linux GNU job 118116145345 e Windows MSVC job 118116144743 completed/success,
  SHA 7522345 verificato, fmt/clippy/test/release verdi. Timeout di una lettura
  GitHub superato con retry; nessun fallimento di test o build riscontrato.
- **Locale:** fmt/check/clippy/test/release/diff-check e generatore --check PASS;
  28 unit + 42 integration + 10 doctest, sei integration nuovi. P0 ancora verde.
- **File/contratti:** compatibility.rs, linter, mod/lib, tre file test;
  SCHEMA_AUTHORING, mappa ufficiale, preflight/report e stato. Raw/compiler
  riusati, nessuna signature, dipendenza, grammatica o baseline modificata.
- **Residui:** supporto formato distinto dal supporto local.v1; nessun registry,
  bundle resolver o runtime anticipato. T-P2-03/M1 restano aperte sul residuo applicativo.
- **Git:** work/p1-foundations, codice 81c685e/7522345 su origin;
  consuntivo finale solo documentale. Modifica storica estranea intatta,
  fuori dai commit; nessun merge su main o task successiva avviata.
- **Next:** T-P3-07 SchemaRegistry; T-P3-03 e T-P3-05 verificate Linux/Windows.


HANDOFF — 2026-10-19 — T-P3-06 / mappa approvata e locale verificato

- **Stato:** DONE_UNVERIFIED, solo CI Linux/Windows pendente.
- **Decisione:** «Approvo la mappa con baseline di sviluppo 0.1.0»;
  CNF-FORMAT-01 risolto. Formato 1→0.1.0 sviluppo/compiler b658e05;
  formati non definiti senza eseguibile dichiarato, nessuna release inventata.
- **Fatto/file:** compatibility.rs, tests/schema_compatibility.rs; mod/lib,
  linter e test compiler/repository; contratto, mappa, preflight/report/stato.
  Nessuna modifica raw/compiler necessaria: parser obbligatorio e compile
  tramite lint già esistenti, nessuna implementazione parallela.
- **Verifiche:** fmt/check/clippy/test/release/diff-check e generatore --check
  PASS, 28 unit + 42 integration + 10 doctest. Messaggi verificati, P0 regressione verde.
- **Contratti:** tipi DET Addendum A.1; minima approvata indipendente dalla
  versione corrente del package; nessuna modifica alle signature compiler/raw.
- **Residui:** nessun blocco locale; niente registry, bundle resolver o runtime
  anticipato. T-P2-03/M1 ancora aperte sul residuo applicativo.
- **Git:** work/p1-foundations; parte indipendente 81c685e su origin;
  codice finale/CI da registrare alla chiusura. Modifica storica estranea intatta.
- **Next:** T-P3-07 dopo CI T-P3-06; prerequisiti T-P3-03 e T-P3-05 verificati.


HANDOFF — 2026-10-19 — T-P3-06 / inventario mapping richiesto dall’autore

- **Stato:** IN_PROGRESS; solo parte indipendente implementata, messaggio
  di versione minima e completamento task subordinati alla mappatura ufficiale.
- **Fatto/file:** compatibility.rs e tests/schema_compatibility.rs nuovi;
  lint usa il descrittore corrente, mod/lib esportano i tipi; test compiler e
  repository estesi. Preflight/inventario documentano CNF-FORMAT-01.
- **Verifiche:** fmt/check/clippy/test/release/diff-check e generatore --check
  PASS; 28 unit + 41 integration + 10 doctest. Cinque integration nuovi.
  CI della task non ancora eseguita; nessun DONE o esito remoto presunto.
- **Evidenze mapping:** nessun tag locale; GitHub release list [] exit 0;
  solo formato 1 definito, package 0.1.0 di sviluppo. Prerequisiti T-P3-05/G0 verdi.
- **Decisione richiesta:** ratificare la matrice completa in
  docs/schema-authoring/FORMAT_COMPATIBILITY.md; proposta A del preflight
  non approvata dalla sola richiesta dell’inventario.
- **Git:** work/p1-foundations; modifica storica estranea preservata.
  Nessun cambiamento ad ARCH/DET/Piano, schema JSON o baseline normative.
- **Next:** completare T-P3-06 dopo decisione; nessuna task successiva avviata.


HANDOFF — 2026-10-19 — T-P3-05 completata / CI reale verde

- **Stato:** DONE; QG-2 PASS sul commit codice b658e05. CNF-COMPILER-01
  risolto con approvazione A, registrata nel contratto e nel dossier.
- **Fatto:** SchemaCompiler del sottoinsieme Numeric/AN, Required/Optional,
  padding/missing/lessico, PK/projection e metadati; conversione unica;
  binding encoding Rust-only obbligatorio. Costrutti esclusi sempre Error.
- **Evidenze:** [run 39264362959](https://github.com/UraganoAngelina/lintra/actions/runs/39264362959)
  Linux GNU job 118099968302 e Windows MSVC job 118099968669 entrambi
  completed/success, SHA b658e05 verificato; fmt/clippy/test/release verdi.
- **Locale:** fmt/check/clippy/test/release/diff-check e generatore --check
  PASS; 28 unit + 36 integration + 10 doctest; Rust 1.99.1 verificato.
- **File:** compiler.rs e tests/schema_compiler.rs nuovi; compiled/test,
  raw/generatore, linter, mod/export; SCHEMA_AUTHORING, preflight/report e stato.
  Nessuna nuova dipendenza, grammatica o baseline modificata.
- **Contratti/decisioni:** API DET §11 e distinzione esiti invariati;
  local.v1 non dichiarata completa dal motore, encoding regionale non scelto.
  Dettagli e perimetro effettivo nel report T-P3-05.
- **Blocchi/residui:** nessun blocco della task; SIOC richiede ancora costrutti
  esclusi e fallisce esplicitamente. T-P2-03/M1 restano aperte sul residuo
  applicativo, pur con SchemaCompilationError ora disponibile come payload reale.
- **Git:** work/p1-foundations, codice b658e05 su origin; chiusura successiva
  solo documentale. Working tree con modifica storica estranea intatta,
  fuori dai commit. Nessun merge su main o task successiva avviata.
- **Next:** T-P3-06, compatibilità formato/EngineCapabilities; T-P3-05 e
  G0 verificati. Mantenere distinta la deroga compiler dalle capability runtime.


HANDOFF — 2026-10-19 — T-P3-05 / compiler locale verificato, CI pendente

- **Stato:** DONE_UNVERIFIED; CNF-COMPILER-01 risolto dall’approvazione A.
- **Fatto:** compile(raw) con lint/preflight aggregato e conversione unica,
  binding in memoria obbligatorio, PK/projection risolte e metadati conservati.
  Costruttori compiled privati, nessun no-op di costrutti obbligatori esclusi.
- **File:** compiler.rs e tests/schema_compiler.rs nuovi; mod/export lib,
  raw/generatore, compiled/test, codici issue; contratto/preflight/report e stato.
- **Evidenze:** fmt/check/clippy/test/release/diff-check e generatore --check
  PASS; 28 unit + 36 integration + 10 doctest (2 unit/11 integration nuovi).
- **Contratti/decisioni:** signature DET §11 invariata; SchemaCompilationError
  reale, dieci nuovi codici; RawResolvedEncoding solo Rust. UTF-8 esplicito
  per il sottoinsieme sintetico; non seleziona encoding SIOC/CAD o default.
- **Blocchi/residui:** nessun blocco locale. CI Linux/Windows pendente.
  T-P2-03/M1 aperte; niente validator/runtime/storage/EngineCapabilities.
- **Documenti:** integrazione approvata in SCHEMA_AUTHORING e preflight;
  ARCH/DET/Piano/grammatica/baseline inalterati. Report con supporto esatto.
- **Git:** work/p1-foundations; modifica storica preesistente estranea
  intatta e fuori dallo staging. Codice e CI da registrare alla chiusura.
- **Next:** T-P3-06, dopo CI della task; prerequisito G0 già approvato.


HANDOFF — 2026-10-19 — T-P3-05 preflight / CNF-COMPILER-01

- **Stato:** BLOCKED su contratto capability; prerequisiti T-P3-02 e
  T-P3-04 verificati da CI Linux/Windows, nessuna dipendenza mancante.
- **Fatto/file:** dossier T-P3-05-preflight e stato; nessun codice modificato.
- **Evidenze:** lettura contratti, raw/compiled/linter e script preflight PASS;
  cargo/test/CI non eseguiti perché modifica solo documentale.
- **Decisione richiesta:** proposta A circoscritta nel dossier oppure blocco B.
  Binding encoding Rust-only segue il contratto in memoria già approvato;
  nessun default regionale o modifica dei JSON è proposto.
- **Git:** work/p1-foundations; modifica storica preesistente estranea intatta.
  Nessuna nuova dipendenza, baseline o fonte architetturale modificata.
- **Next:** risolvere CNF-COMPILER-01, poi riprendere T-P3-05. Nessuna task
  successiva avviata; T-P2-03/M1 restano aperte sul residuo applicativo.


HANDOFF — 2026-10-19 — T-P3-04 completata / CI reale verde

- **Stato:** DONE; QG-2 PASS sul commit codice a1ddabb.
- **Evidenze:** [run 39257768950](https://github.com/UraganoAngelina/lintra/actions/runs/39257768950),
  Linux GNU job 118077279746 e Windows MSVC job 118077279423 entrambi
  completed/success: fmt/clippy/test/build release nativi verdi.
- **Locale:** fmt/check/clippy/test/release/diff-check e raw --check PASS;
  26 unit + 25 integration + 10 doctest; 13 codici coperti da test.
- **P0:** ricontrollo Rust strutturale completato senza issue, cinque trace
  e 135 campi SIOC, proiezioni separate confrontate e lintate. Nessuna baseline modificata.
- **File:** src/schema/linter.rs e tests/schema_linter.rs creati;
  src/schema/mod.rs, src/lib.rs aggiornati; report T-P3-04 e stato registrati.
- **Contratti/decisioni:** severity esplicita e codici non_exhaustive;
  ordine flow/trace/campo, coordinate one-based. Dettagli nel report.
- **Problemi/blocchi:** nessuno aperto per la task; prima lettura CI bloccata,
  risolta con richiesta sostitutiva limitata nel tempo e verifica dello SHA.
- **Residui/documenti:** compiler e semantica restano alle task pertinenti;
  T-P2-03/M1 aperte sul residuo applicativo. Fonti ARCH/DET/Piano invariate.
- **Git:** branch work/p1-foundations, codice a1ddabb su origin;
  consuntivo finale solo documentale. Working tree con sola modifica storica
  preesistente estranea alla task, intatta e fuori dai commit; nessun merge.
- **Next:** T-P3-05 SchemaCompiler nucleo; T-P3-02 e T-P3-04 verificate
  su Linux/Windows. Nessuna task successiva avviata in questa consegna.


HANDOFF — 2026-10-19 — T-P3-04 / locale verificato, CI pendente

- **Stato:** DONE_UNVERIFIED, solo CI Linux/Windows da completare.
- **Fatto/file:** linter.rs, tests/schema_linter.rs, mod/export lib,
  report T-P3-04 e stato; nessuna nuova dipendenza o modifica delle baseline.
- **Verifiche:** fmt/check/clippy/test/release/diff-check e raw --check PASS;
  26 unit + 25 integration + 10 doctest. Uno unitario e 14 integration nuovi.
- **P0:** flow SIOC con 5 trace/135 campi e cinque proiezioni sottoposti
  al lint Rust senza issue; tre flow sintetici verdi. Nessun record reale.
- **Contratti/decisioni:** API lint condivisibile con futuro compiler;
  diagnostiche tipizzate/severity, ordine stabile; dettagli nel report.
- **Problemi/blocchi:** nessuno locale; T-P2-03/M1 aperte sul residuo già approvato.
- **Rinvii:** risoluzione ref, compatibilità constraint, encoding/capability,
  compiler/runtime/storage; nessun profilo dichiarato eseguibile.
- **Git:** branch work/p1-foundations; file storico preesistente intatto e
  fuori dallo staging; commit e CI da registrare alla chiusura.
- **Next:** T-P3-05, prerequisiti T-P3-02 e lint T-P3-04 verificato dopo CI.


HANDOFF — 2026-10-19 — T-P3-03 completata / CI reale verde

- **Stato:** T-P3-03 DONE; QG-2 verde anche sul codice corrente.
- **Evidenza:** [run 39253152098](https://github.com/UraganoAngelina/lintra/actions/runs/39253152098)
  sul commit a39924d; Linux GNU job 118061707217 e Windows MSVC job
  118061706768 completed/success, fmt/clippy/test/release nativi verdi.
- **Locale:** fmt/check/clippy/test/release e diff-check tutti exit 0;
  25 unit + 11 integration + 10 doctest. Report T-P3-03 con scope e limiti.
- **Git:** codice e consuntivo iniziale in a39924d su origin;
  chiusura successiva solo documentale. File storico preesistente intatto
  e fuori dai commit; nessun merge su main.
- **Residui:** T-P2-03/M1 ancora aperte sul residuo applicativo;
  nessun lint/compiler/validator anticipato, baseline normative invariate.
- **Next:** T-P3-04 linter strutturale; ricontrollo Rust dei profili P0 obbligatorio.


HANDOFF — 2026-10-19 — T-P3-03 / locale verificato, CI pendente

- **Stato:** repository filesystem e memoria implementati; nessuna task successiva avviata.
- **Locale:** fmt/check/clippy/test/release e diff-check tutti exit 0;
  25 unit + 11 integration + 10 doctest, nove nuovi casi repository.
- **Contratti:** chiave e versione esatte, layout ARCH §12 con flow.json,
  nessun fallback; JSON/identità selezionata verificati, nessun lint semantico.
  Draft SIOC invariato, staging esplicito verificato in directory temporanea.
- **File:** repository.rs, test pubblico, mod/export e
  [report T-P3-03](docs/verification/T-P3-03.md); nessuna nuova dipendenza.
- **Residui:** T-P2-03/M1 aperte; SchemaLoadError disponibile ma gli altri
  payload applicativi non ancora implementati. Nessun compiler o validator.
- **Next:** CI Linux/Windows, poi T-P3-04 con lint obbligatorio dei profili P0.


HANDOFF — 2026-10-19 — T-P3-02 completata / CI reale verde

- **Stato:** T-P3-02 DONE; nessuna task successiva avviata in questa consegna.
- **Evidenza:** [run 39250930467](https://github.com/UraganoAngelina/lintra/actions/runs/39250930467) sul commit
  f04bb60: Linux GNU job 118054251065 e Windows MSVC job 118054251729
  completed/success, fmt/clippy/test/build release nativi verdi.
- **Locale:** fmt/check/clippy/test/release tutti exit 0; 25 unit + 2
  integration + 10 doctest, negativi delle invarianti e della costruzione
  esterna inclusi. Riproducibilità raw e diff-check verificati.
- **Decisioni tecniche:** SchemaKey di dominio unica; Vec per ordine fisico
  e trace, HashMap solo field_index derivato; costruttori privati nel boundary
  compiled e consumer compiler futuro discendente/reexport. Interi literal
  esatti senza limite macchina, programmi regex byte posseduti nel modello.
  Rifiniture DET §147 e limiti sono nel report, nessuna fonte modificata.
- **Residui:** compiler/evaluator/backend non implementati e nessuna nuova
  capability eseguibile; T-P2-03/M1 aperte sul residuo applicativo. Lint Rust
  dei profili P0 ancora obbligatorio in T-P3-04.
- **Git:** codice/lockfile nel commit verificato f04bb60 su origin;
  consuntivo finale solo documentale. File storico preesistente intatto,
  fuori dal commit. Nessun merge su main.
- **Next:** T-P3-03 SchemaRepository, secondo dipendenze atomiche autorizzate.

HANDOFF — 2026-10-19 — T-P3-02 / modello compiled locale verificato

- **Stato:** DONE_UNVERIFIED, CI della modifica da eseguire.
- **File:** schema/compiled.rs, compiled/{plans,tests}.rs, schema/mod.rs,
  export/doctest lib, Cargo.toml/lock, report T-P3-02 e stato.
- **Contratti:** campi privati e accesso condiviso; nessuna costruzione
  pubblica/Deserialize/Default; SchemaKey di dominio riutilizzata. Vec
  ordinate, field_index privato derivato; piani chiusi e ref risolti tipizzati.
- **Evidenze:** fmt/check/clippy/test/release exit 0; 25 unit, 2 integration,
  10 doctest, inclusi confini di costruzione e invarianti di indici/layout.
  Generato raw invariato; diff-check e cargo tree verificati.
- **Dipendenza:** regex 1.13.1 necessaria al programma byte precompilato,
  quattro nuovi package Rust nel lockfile, nessuna libreria di sistema.
- **Limiti:** nessun compiler/evaluator/backend implementato; nessuna
  capability dichiarata eseguibile. Factory privata riservata a T-P3-05;
  le guardie locali non sostituiscono il lint/compile semantico completo.
- **Next:** CI Linux/Windows della task; poi T-P3-03. Residuo T-P2-03/M1 e
  obbligo lint Rust dei profili P0 in T-P3-04 invariati.

HANDOFF — 2026-10-19 — T-P3-01 completata, riordino A applicato

- **Stato:** T-P3-01 DONE; parte autonoma T-P2-03 verificata, residuo
  ValidationRunError applicativo aperto, M1 non dichiarata completa.
- **Evidenza:** [run 39236782609](https://github.com/UraganoAngelina/lintra/actions/runs/39236782609) su 225419d,
  Windows MSVC job 118007665215 e Linux GNU job 118007665508 completed/success.
  17 unit, 2 integration, 5 doctest; fmt/clippy/test/release nativi verdi.
- **Documenti:** report T-P2-03/T-P3-01 e stato aggiornati; approvazione A
  registrata in DECISIONS e dossier. ARCH/DET/Piano e baseline invariati.
- **Git:** codice e lockfile verificati nel commit 225419d già su origin;
  aggiornamento finale solo documentale, nessuna modifica Rust dopo la CI.
  File storico Superpowers preesistente lasciato fuori dallo staging/commit.
- **Next:** T-P3-02: modello compiled riutilizzando SchemaKey del dominio;
  non confondere RawFlowSchema con un artefatto eseguibile. Lint dei profili
  P0 obbligatorio in T-P3-04; residuo T-P2-03 da riprendere con payload reali.

HANDOFF — 2026-10-19 — T-P3-01 / Raw model locale verificato

- **Stato:** DONE_UNVERIFIED; nuova CI da eseguire. Parte autonoma T-P2-03
  verificata Linux/Windows su 70cdb5e, run 39235805213; residuo ancora aperto.
- **File:** schema/raw.rs, raw_tests.rs, schema/mod.rs, export lib;
  generatore DTO documentale, report T-P3-01, Cargo.toml/lock e stato.
- **Contratti:** modello formato 1 tipizzato, metadata aperti separati,
  proprietà operative sconosciute rifiutate, start one-based preservato.
  Raw non è compilato; vincoli scalari/condizionali e risoluzione restano
  esplicitamente al lint/compiler successivo, senza dichiarare esecuzione.
- **Dipendenze:** serde 1.0.229/serde_json 1.0.151, lockfile aggiornato offline;
  derive e arbitrary_precision necessari per DTO/metadata/interi esatti.
- **Evidenze:** fmt/check/clippy/test/release exit 0; 17 unit, 2 integration,
  5 doctest. Riproducibilità del generato verificata contro grammatica invariata.
- **Next:** CI Linux/Windows della modifica, poi T-P3-02; SchemaKey rimane
  nel dominio. Obbligo lint Rust T-P3-04 sui profili P0 non ancora adempiuto.

HANDOFF — 2026-10-19 — T-P2-03 / proposta A approvata, parte autonoma

- **Stato:** IN_PROGRESS; parte autonoma locale verificata, CI da eseguire.
- **Decisione umana:** «approvo la proposta A del dossier», riportata per
  intero dall’autore; supera il precedente blocco CNF-DOMAIN-01.
- **File:** domain/{identity,summary}.rs, Ord DatasetId, export lib/domain,
  registro decisioni, dossier/report e stato. Nessuna dipendenza Cargo nuova.
- **Evidenze:** fmt/check/clippy/test/release exit 0; 12 unit + 2 integration
  + 5 doctest. Tipi neutri senza dipendenze applicative nel dominio.
- **Residuo:** ValidationRunError applicativo con payload tecnici veri,
  nessun placeholder; T-P2-03/M1 non complete fino alla sua verifica.
- **Next:** CI Linux/Windows della parte autonoma; poi T-P3-01 secondo
  riordino limitato autorizzato e soli prerequisiti atomici soddisfatti.

HANDOFF — 2026-10-19 — T-P2-02 completata / T-P2-03 bloccata

- **Stato:** T-P2-02 DONE, T-P2-03 BLOCKED, M1 IN_PROGRESS.
- **CI effettiva:** [run 39234593522](https://github.com/UraganoAngelina/lintra/actions/runs/39234593522) sul commit
  30af262, Linux GNU job 118000501047 e Windows MSVC job 118000500610
  completed/success. fmt, clippy, test e release nativi verificati.
- **Locale:** 9 unit + 2 integration + 5 doctest; check e release verdi.
- **Decisione umana:** «Mantieni il blocco; rivediamo i contratti» per
  CNF-DOMAIN-01. Proposta A non approvata, nessun contratto T-P2-03 scritto,
  nessun documento architetturale o ordine delle task modificato.
- **File:** report T-P2-02, preflight T-P2-03 e questo stato; le modifiche
  Rust sono già nel commit verificato 30af262, nessuna modifica dopo la CI.
- **Next:** rivedere ScopeSummary, proprietà di SchemaKey e collocazione/
  dipendenze di ValidationRunError prima di riprendere T-P2-03 o discendenti.
  Nessuna nuova domanda ordinaria o riapertura di G0.
- **Git:** branch work/p1-foundations, nessuna integrazione/merge su main;
  modifica storica Superpowers preesistente lasciata intatta e non committata.

HANDOFF — 2026-10-19 — T-P2-02 / mapping unico locale verificato

- **Stato:** DONE_UNVERIFIED; CI della modifica da completare.
- **Prerequisito:** T-P2-01 DONE, CI 39233324913 su 6ac0fb8 entrambi success.
- **File:** domain/errors.rs, export lib/domain, docs/VALIDATION_CODES,
  report T-P2-01/02 e questo stato.
- **Evidenze:** fmt/check/clippy/test/release exit 0; 9 unit, 2 integration,
  5 doctest. Quattro nuovi unit per round-trip, simboli attesi, unicità,
  ordine stabile, RegulatoryRule e negativi/alias/legacy.
- **Decisioni:** DEC-02 applicata senza cambiare ARCH/DET; nessun serializer
  JSON anticipato, mapping unico riutilizzabile dai futuri sink.
- **Next:** CI Linux/Windows, poi preflight T-P2-03.

HANDOFF — 2026-10-19 — T-P2-01 / implementazione locale

- **Stato:** DONE_UNVERIFIED, CI della modifica da eseguire.
- **File:** domain/{ids,byte_range,mod}.rs, lib.rs, report T-P2-01 e stato.
- **Evidenze:** fmt/check/clippy/test/build release exit 0; 5 unit + 2
  integration + 5 doctest, inclusi overflow, zero, slice byte senza copie,
  griglia limite 576 combinazioni e impossibilità di scambiare FlowId/TraceId.
- **Contratti:** sette newtype DET §6 e ByteRange a costruzione verificata;
  errori di costruzione tipizzati, nessuna conversione one-based nel dominio.
- **Limiti:** nessun compiler/persistenza/validator normativo; zero dipendenze.
- **Next:** CI Linux/Windows sul nuovo commit prima di T-P2-02.

HANDOFF — 2026-10-19 — T-P1-03 / CI reale completata

- **Stato:** DONE; G1/QG-2 PASS sul commit 34e67da.
- **Evidenza:** [run 39232752518](https://github.com/UraganoAngelina/lintra/actions/runs/39232752518), workflow_dispatch sul branch
  work/p1-foundations. Linux job 117994423636 e Windows job 117994423276:
  entrambi completed/success; fmt, clippy, test e build release eseguiti.
- **Operazioni:** gh auth status fuori sandbox positivo; repository privata,
  push non forzato dei soli commit, nessuna modifica remota estranea sovrascritta.
- **File:** stato, docs/CI.md e report T-P1-03 aggiornati con evidenza corrente.
- **Next:** T-P2-01 autorizzata e prerequisito verificato. Nessuna deroga a CI;
  le modifiche di dominio richiederanno una nuova run su entrambi i runner.

HANDOFF — 2026-10-19 — T-P0-09 / approvazione ricevuta

- **Stato:** DONE; G0 APPROVED_WITH_DEFERRALS, M0 REACHED.
- **Evidenza umana:** «Approvo G0 con i rinvii indicati», riferita al commit
  407f89e. Conservati tutti i nove esiti e i rinvii puntuali approvati.
- **File:** IMPLEMENTATION_STATUS, report T-P0-09, registro DECISIONS.
- **Verifica:** coerenza stato/checklist/report e diff-check; nessun codice
  Rust modificato, test precedenti non dichiarati come nuove esecuzioni.
- **Next:** completare T-P1-03 con CI Linux/Windows reale prima di T-P2-01.
  gh è ora installato; accesso autenticato verificato fuori sandbox.

HANDOFF — 2026-10-19 — T-P0-09 / checklist pronta

- **Stato:** DONE_UNVERIFIED; G0 AWAITING_AUTHOR_APPROVAL, M0 NOT_REACHED.
- **File:** sezione G0 di IMPLEMENTATION_STATUS e docs/verification/T-P0-09.md.
- **Evidenze:** nove voci DET §106 coperte, ciascuna chiusa per contratto
  corrente o rinviata con ragione e accettazioni/task interessate; riscontro
  di 218 proiezioni campo e riesecuzione controllo T-P0-06 (15/150).
  Controlli T-P0-03..08 e Cargo già positivi nella stessa sessione;
  link e diff-check finali. Nessun test/CI normativo dichiarato.
- **Decisione richiesta:** approvazione esplicita della checklist G0 con
  scope minimale e rinvii. Obbligo dal prompt T-P0-09/Piano §9, non da skill.
- **Limiti:** nessun avanzamento delle task dipendenti fino all’approvazione;
  CI Linux/Windows reale resta prerequisito separato per T-P2-01.
- **Git:** nessun push; modifica storica estranea lasciata fuori dal commit.
- **Next:** registrare risposta autore per chiudere G0; poi risolvere QG-2
  prima dello sviluppo dei tipi T-P2-01. Nessuna task successiva iniziata.

HANDOFF — 2026-10-19 — T-P0-08 / profilo minimale autorizzato

- **Stato:** DONE per DEC-SIOC-MINIMAL-01; baseline FROZEN non modificata.
- **File:** schemas/sioc/2026-04-20.structural-minimal.1 (flow, cinque trace,
  ledger, README); fixtures-plan/catalogo, decisioni, report e verificatori.
- **Evidenze effettive, exit 0:** sioc_minimal_profile.py (JSON Schema,
  riproducibilità, 135 layout, riferimenti, 5 PK/6 join, due negativi);
  check_authoring.py (33 negativi), check_fixtures_plan.py (376 casi pianificati),
  check_rules_and_joins.py, check_notes.py, check_presence.py (218 campi).
  cargo fmt --all -- --check; cargo check --locked;
  cargo clippy --all-targets --locked -- -D warnings; cargo test --locked
  (2 integration test e 3 doctest). Nessun test normativo runtime.
- **Verifiche JSON:** uv run --cache-dir /tmp/lintra-authoring-uv --offline
  --no-project --with jsonschema==4.26.0 python docs/verification/<script>.py.
- **Decisioni:** 050 solo tre prefissi inviante e 4.7; niente lookup; mar01
  futuro; regole omesse esplicite. Persistenza PK/audit resta contratto.
- **Limiti:** lint Rust da rieseguire in T-P3-04; niente compiler, golden,
  validazione pazienti o CI reale. Export letti solo per aggregati tecnici;
  larghezze anomale documentate, nessun dato sanitario copiato nel repo.
- **Git:** file storico Superpowers già modificato lasciato intatto e fuori
  dal commit; nessun push. Risultati più recenti prevalgono sui handoff storici.
- **Next:** T-P0-09, checklist concreta e approvazione finale obbligatoria.
  P2 resta separatamente bloccata sulla CI reale.

HANDOFF — 2026-10-19 — T-P0-08 / preflight e fixture-plan

- **Stato:** BLOCKED sulle parti dipendenti da input normativi; prerequisito
  T-P0-07 DONE con accettazione umana completa, commit 2ba2e95.
- **File:** fixtures-plan.md, sioc/fixtures-catalog.json, checker/report
  T-P0-08 e questo stato. Nessun file schema incompleto presentato come valido.
- **Evidenze:** piano copre 135 campi, 72 regole, 376 specifiche di caso
  campo; hash/indici/template/dipendenze tracciati. Checker authoring e
  matrici passati, 218 campi/policy ricevute fedeli.
- **Blocchi:** dataset/versioni e metadata ufficiali, encoding/caratteri
  per no_lowercase, convenzione compleanno 29 febbraio. Domande inviate,
  nessuna decisione inferita. File reali paziente non consultati.
- **Test:** solo controlli documentali/JSON Schema; niente golden o
  record normativi eseguiti. Cargo non rieseguito, nessun Rust cambiato.
- **Ricontrollo obbligatorio:** linter automatico appena T-P3-04 esiste, sui
  canonical schema da produrre; il presente checker non sostituisce il linter.
- **Git:** work/p1-foundations; piano storico preesistente preservato;
  nessun push o nuova CI.
- **Next:** risolvere gli input e riprendere la stessa T-P0-08; T-P0-09
  non avviata, G0 aperto, T-P2-01 continua ad attendere CI reale.

HANDOFF — 2026-10-19 — T-P0-07 / accettazione umana ricevuta

- **Stato:** DONE; autore: «Approvo il contratto T-P0-07» sul commit 72f9f5e.
- **File:** stato/decisioni, intestazioni di authoring/artefatti e report
  aggiornati; nessun cambiamento delle regole o degli esempi approvati.
- **Verifiche:** check_authoring.py rieseguito, 33 casi negativi superati;
  git diff --check superato. Nessun Rust modificato o Cargo rieseguito.
- **Next:** T-P0-08, previo riscontro delle boundary normative; G0 resta aperto.

HANDOFF — 2026-10-19 — T-P0-07 / contratto pronto per revisione

- **Stato:** DONE_UNVERIFIED, attende revisione umana finale del contratto.
- **Decisione:** deroga circoscritta ai tre derivati approvata dall’autore;
  CNF-LINTRA-DSL-01 risolta, registrata senza cambiare i documenti sorgente.
- **File:** SCHEMA_AUTHORING.md, docs/schema-authoring/* (2 meta-schemi,
  lowering, indice, 3 flow, dataset/manifest sintetici), check_authoring.py,
  report, DECISIONS.md, attributi Git limitati agli esempi e questo stato.
- **Evidenze:** JSON Schema reale jsonschema 4.26.0 temporaneo: 3 flow e
  manifest validi, 4 hash verificati, 33 casi negativi rifiutati, 15 confronti byte con filtri Git autocrlf;
  famiglie 19 constraint/14 assertion legacy/13 Expr coperte; checker P0 precedenti
  passati, 218 campi/fonti integri. Nessun Rust modificato o Cargo rieseguito.
- **Limiti:** prove strutturali, non compilazione DSL Rust né test normativi;
  boundary lessicali/dataset e policy 29 febbraio del profilo esplicite.
  Formato/payload compiled/persistenza richiedono la revisione completa.
- **Git:** work/p1-foundations; piano storico preesistente preservato;
  nessun push o nuova CI. Nessuna dipendenza Cargo aggiunta.
- **Next:** ricevere revisione T-P0-07 e solo poi esaminare T-P0-08; G0
  rimane aperto, P2 continua ad attendere la CI reale Linux/Windows.

HANDOFF — 2026-10-19 — T-P0-07 / riconciliazione bloccata

- **Stato:** BLOCKED; T-P0-03/04/05 verificati, nessun problema di accesso
  ai documenti. Decisione SIOC parent opzionale applicata nei documenti
  e nel checker; T-P0-05 resta DONE (commit 20ab9de).
- **File:** docs/SCHEMA_AUTHORING.md (bozza), report T-P0-07 e stato.
- **Evidenze:** tre checker documentali superati; due JSON sintetici della
  bozza parsabili, digest e link verificati; fonti integre. Cargo non
  rieseguito per soli documenti, nessuna validazione JSON Schema dichiarata.
- **Conflitto:** CNF-LINTRA-DSL-01, valori derivati necessari ai profili ma
  rinviati dal Piano §13.5/DET §135; scelta autore sulla proposta A/B.
- **Preparato:** proposta di formato numerico distinta dalla DSL legacy,
  manifest di bundle, vincoli e checklist authoring ancora da completare.
- **Git:** branch work/p1-foundations; piano storico preesistente inalterato.
  Nessun nuovo push, CI o dipendenza. ARCH/DET/PLAN non modificati.
- **Next:** riprendere T-P0-07 dopo decisione di scope; completare contratto
  ed esempi e ottenere revisione umana. T-P0-08 non avviata; P2 attende CI.

HANDOFF — 2026-10-19 — esecuzione diretta / T-P0-05

- **Stato:** DONE, criteri documentali verificati; G0 ancora aperto.
- **File:** rules-and-joins.md SIOC/CAD, checker e report T-P0-05;
  DECISIONS.md, matrice note CAD, report T-P0-04 e stato aggiornati.
- **Evidenze:** checker regole/join, note e presenza superati; 74 regole,
  dieci join, trenta presence condizionali; fonti ricevute integre.
- **Decisione:** iniziale CAD nello storico; ricerca atto SIOC conserva
  match sull’ID ricevuto (esistenza), senza introdurre filtro UDO.
- **Limiti:** policy parent opzionale SIOC da chiarire; piani CAD da
  formalizzare in T-P0-07. Nessun Rust, runtime normativo o schema mutato.
- **Verifiche:** git diff --check superato; Cargo non rieseguito per soli
  documenti. Nessuna nuova CI/Linux-Windows o push.
- **Git:** work/p1-foundations; modifiche preesistenti al piano storico
  lasciate inalterate.
- **Next:** chiarimento SIOC e quindi T-P0-07; P2 continua ad attendere CI.

HANDOFF — 2026-10-19 — chiarimento SIOC-ID-2026

- **Decisione:** registrata la deroga autorizzata per ingressi 2025 conclusi
  nel 2026; aggiornati registro decisioni, matrice note e report T-P0-04.
- **Verifica:** check_notes.py, check_presence.py e git diff --check superati;
  nessun Rust o schema ricevuto modificato, nessuna nuova esecuzione Cargo.
- **Next:** T-P0-05 resta non avviata, in attesa della sola decisione CAD
  sull’ambito della valutazione iniziale (storico evento o singolo invio).

HANDOFF — 2026-10-19 — esecuzione diretta / T-P0-04

- **Stato:** DONE per classificazione e registro delle ambiguità; avanzamento
  a T-P0-05 sospeso per due decisioni normative ancora aperte.
- **File:** matrici note SIOC/CAD, DECISIONS.md, report e checker T-P0-04;
  aggiornamento del report T-P0-03 e di questo stato.
- **Evidenze:** 218 note con hash/indici/riferimenti e citazioni verificate;
  26 metadata, 186 candidate, sei note ambigue (incluse clausole differite v1).
- **Decisioni:** ricevuta e registrata la risposta CAD 2.12; nessuna modifica
  delle fonti. Nessuna nuova esecuzione Cargo rivendicata per soli documenti.
- **Limiti:** definire vincoli ID SIOC 2025 e ambito iniziale CAD. Nessun
  codice Rust, compiler, schema normativo o dipendenza modificato.
- **Git:** branch work/p1-foundations; vecchio piano preesistente lasciato
  invariato; nessun push o nuova evidenza CI in questa sessione.
- **Next:** chiarire i due punti e poi procedere a T-P0-05, senza Superpowers.

HANDOFF — 2026-10-19 — esecuzione diretta / T-P0-03

- **Stato:** DONE per la consegna documentale; avanzamento sospeso per ambiguità.
- **File:** matrici presenza SIOC/CAD, report T-P0-03, checker documentale,
  AGENTS.md e questo stato. Le bozze note e il vecchio piano sono conservati.
- **Evidenze:** 218/218 campi, contesti e fonti verificati; fmt/check/clippy
  e test Rust superati, due integration test e tre doctest.
- **Decisioni:** niente Superpowers o nuovi subagenti; nessuna modifica
  a policy, contratti pubblici, dipendenze o documenti architetturali.
- **Problema:** DEC-05-CAD-ICPC-000, proposta e domanda inviate all'autore.
- **Git:** branch work/p1-foundations; nessun push in questa sessione;
  bozze T-P0-04 e piano precedente preesistenti non alterati da questa task.
- **Next:** registrare il chiarimento, poi verificare T-P0-04; nessuna
  task successiva avviata autonomamente in presenza dell'ambiguità.

HANDOFF — 2026-10-16 — Superpowers / T-P0-02 e T-P0-06

- **Stato:** entrambe DONE. T-P0-02: review indipendente approvata e
  approvazione umana testuale «Campione verificato: approvo» ricevuta.
- **File:** dieci tabelle JSON e README SIOC/CAD; reference-datasets.md;
  report riproducibili versionati in docs/verification/.
- **Evidenze:** 218 campi identici ai JSON, zero gap/overlap; CAD 83 righe e
  83 note corroborate dal PDF; 16 fonti conformi agli hash. Dataset: 15 schede,
  150 metadati con lacune, controlli e review superati.
- **Limiti:** fonte primaria XLSX SIOC assente e metadati ereditati dichiarati;
  nessuna norma cambiata. I JSON `document_status: DONE_UNVERIFIED` e i report
  iniziali sono snapshot antecedenti all'approvazione; lo stato corrente è
  registrato qui e nei README. G0 e gate runtime rimangono aperti.
- **Next:** T-P0-03 e T-P0-04 avviate in parallelo su file disgiunti.

HANDOFF — 2026-10-16 — Superpowers / T-P1-03

- **Stato:** DONE_UNVERIFIED, commit 4f71663; review indipendente approvata.
- **File:** .github/workflows/ci.yml e tests/platform_paths.rs; guida in docs/CI.md.
- **Evidenze:** controlli locali e GNU verdi, 2 test di I/O su percorsi con
  spazi e 3 doctest superati; nessuna dipendenza nuova o codice di dominio.
- **Limite bloccante:** GitHub non autenticato, nessun push o run reale.
  QG-2 resta aperto; T-P2-01 non avviata. Repo creato/configurato dall'autore,
  accesso tramite integrazione o credenziali Git del computer ancora necessario.
- **Next:** autenticare accesso, caricare il codice revisionato e verificare
  entrambi i job; solo poi avviare T-P2-01 e le task dipendenti.

HANDOFF — 2026-10-16 — Superpowers / T-P0-01

- **Stato:** DONE; commit 0b13581, review indipendente approvata.
- **File:** [INPUT_INVENTORY.md](docs/normative/INPUT_INVENTORY.md), matrice
  completa DET §154: undici categorie, 27 sottovoci con stato e motivo.
- **Evidenze:** controllo di completezza/tracciabilità superato; 196/196 file
  censiti integri; kit completo FAIL solo per quattro extra non letti.
- **Mancanti consegnati:** sorgente XLSX SIOC, dodici reference dataset SIOC,
  fixture/golden sintetiche approvate e dettagli di efficacia/versioning
  non desumibili dai materiali ricevuti. Elenco puntuale nell'inventario.
- **Limiti:** G0 aperto; nessuna norma, source document, API o Rust modificati.
- **Next:** T-P0-06 (schede con lacune ammesse) indipendente da CI;
  T-P0-02 richiede fonti e revisione umana a campione, non presunte.

HANDOFF — 2026-10-16 — Superpowers / T-P1-02

- **Stato:** DONE. File: rust-toolchain.toml, .gitattributes, .cargo/config.toml,
  docs/DEVELOPMENT.md. Lockfile già committato in c7ee1cd, invariato.
- **Evidenze:** Rust 1.99.1 installata con componenti e target GNU; fmt,
  check, clippy con warning negati, test (3 doctest), build e cross-check GNU
  tutti exit 0; 66 confronti byte Git su fixture/golden sintetici superati.
- **Review indipendente:** approvata, nessun finding implementativo.
- **Decisioni:** Rust 1.99.1 scelta umana; baseline Linux di release aperta;
  Windows 10+/Server 2016+ secondo architettura e documentazione toolchain.
- **Limiti:** MinGW/Wine non installati, nessuna build collegata/test Wine;
  nessuna CI reale dichiarata. Nessun contratto pubblico/dependency aggiunto.
- **Git:** branch work/p1-foundations, commit 0186c0b; documenti di sessione
  ancora in aggiornamento, nessun remote autenticato.
- **Next:** T-P1-03, prerequisito locale verificato; nessuna deroga a QG-2.

HANDOFF — 2026-10-16 16:30 CEST — Codex / bootstrap Lintra

- **Fase/Task attiva:** P1 / T-P1-01; **stato: DONE**.
- **Fatto:** Cargo.toml/lock, lib/main, undici mod.rs vuoti, directory dati/test
  riservate, README, gitignore, sette documenti approvati, piano, evidenze e stato.
- **Evidenze:** registro sopra e trascrizione collegata; controlli Linux
  effettivamente eseguiti; nessuna CI dichiarata.
- **Test aggiunti/eseguiti:** tre doctest tutti superati; nessun tipo/regola
  normativa ancora implementato; test Windows non eseguiti.
- **Contratti pubblici aggiunti/modificati:** nessuno.
- **Decisioni:** tabella sopra; nessun database o policy normativa scelti.
- **Documenti da aggiornare:** nessuno per lo scope implementato.
- **Problemi/blocchi:** nessun blocco T-P1-01; kit con quattro extra esclusi;
  CI e gate successivi aperti.
- **Elementi rinviati:** toolchain/config Windows/attributi in T-P1-02;
  CI in T-P1-03; dominio, compiler, persistenza, validatori, report e Slint
  nelle rispettive task.
- **Stato Git:** branch `main`; nessun commit; working tree non pulito
  (nuovi file untracked); target ignorato; nessun remote.
- **Prossima task:** T-P1-02, prerequisito scaffold verificato;
  attende autorizzazione dell'autore. Non avviata.
