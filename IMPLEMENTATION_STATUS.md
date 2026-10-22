# IMPLEMENTATION_STATUS.md — Lintra

Consuntivo aggiornato al **2026-10-22**, derivato da `IMPLEMENTATION_STATUS_SEED.md`.
Piano v1.1; architettura e detailed design revisione 2026-10-16.

## Frontiera corrente — T-P10-03 DONE / avvio T-P10-04 — 2026-10-22

ExpressionEvaluator e contesto reale locale consegnati, 384 test/48 suite.
fmt/check/Clippy/test/release/Windows GNU check/diff passati; CI 39780196256
completed/success Linux/Windows su a6c71386b708ae89961e5b19a15a01e52e520453.
[Dossier e spiegazioni](docs/verification/T-P10-03.md).

HANDOFF — T-P10-03 DONE
- Eq/Ne/In/NotIn/IsPresent/IsMissing/AND/OR/NOT/Const eseguiti su byte;
  Missing distinto da Present vuoto, width/ref invalidi Unknown esplicito.
- Porta FieldValueResolver realmente consumata e LocalEvaluationContext reale;
  nessun parametro projected/related fittizio, nessun backend futuro simulato.
- Derived/esterne nel contesto locale danno UnsupportedPlan nelle chiamate
  dirette; compiler continua a rifiutarle e a bloccare conditional execution.
- File evaluator/lib, test evaluator/tavole, dossier/stato. Git work/p1-foundations.
- a6c7138 pubblicato, native CI verificata; next 10-04 avviata.
- P11/P13/PERSIST/T-P2-03/M1 aperti; solo LocalOnly eseguibile.

## Registro storico — T-P10-02 conclusa

## Frontiera corrente — T-P10-02 DONE / avvio T-P10-03 — 2026-10-22

TruthValue/UnknownReason e tavole complete verificate, 376 test/47 suite.
fmt/check/Clippy/test/release/Windows GNU check/diff passati; CI 39764219198
completed/success Linux/Windows su e17c33abef954c1b8eb9c6854b760f8a1482319f.
[Dossier e spiegazioni](docs/verification/T-P10-02.md).

HANDOFF — T-P10-02 DONE
- AND/OR/NOT a tre valori, nessuna conversione bool o Unknown promosso a pass.
- Quattro motivi del design, tie Unknown sinistro per ordine diagnostico stabile.
- File engine/expression_evaluator/mod, export lib, test tavole e dossier/stato.
- Git work/p1-foundations; e17c33a pubblicato e gate nativo verificato.
- Prerequisito 10-01 chiuso; next 10-03 dopo gate, avanzamento automatico.
- P11/P13/PERSIST/T-P2-03/M1 aperti; solo LocalOnly eseguibile.

## Registro storico — T-P10-01 conclusa

## Frontiera corrente — T-P10-01 DONE / avvio T-P10-02 — 2026-10-22

Modello espressioni locali risolto, 374 test locali/46 suite; fmt/check/Clippy/
test/release/Windows GNU check/diff passati. CI nativa 39763287298
completed/success Linux GNU/Windows MSVC su b8e1e80ea8595ea3a579af3dd2d1ecb4f093f50d.
[Dossier e spiegazioni](docs/verification/T-P10-01.md).

HANDOFF — T-P10-01 DONE
- Raw/compiled esistenti riusati; lowering locale di foglie/booleani/Const e
  definizioni nominate, ref mancanti/cicli espliciti; niente eval.
- Stack esplicito testato con catena di 4096 nomi; byte/ordine preservati.
- Schemi condizionali restano UnsupportedConstruct fino agli executor P10;
  esterne e derived rifiutati, nessun piano ignorato in uno schema accettato.
- File expression/mod/compiler/linter, unit/integration, dossier/stato.
- Git work/p1-foundations, b8e1e80 pubblicato; gate Linux/Windows verificato.
- Prerequisiti compiler/G0 soddisfatti, next 10-02 dopo gate.
- P11/P13/PERSIST/T-P2-03/M1 aperti; solo LocalOnly eseguibile.

## Registro storico — P9 conclusa

## Frontiera corrente — P9 DONE / avvio T-P10-01 — 2026-10-22

Golden semantici dedicati consegnati: 367 test locali, 46 suite, 8 scenari golden.
fmt/check/Clippy/test/release/Windows GNU check/diff passati; CI 39762338659
completed/success Linux GNU/Windows MSVC su 3fafc4b41740e1fea8f2055b4fb4ec2dce4d3522.
[Dossier e spiegazioni](docs/verification/T-P9-06.md).

HANDOFF — T-P9-06 DONE / P9 chiusa
- Nuovo profilo sintetico 14 byte: range/date/dominio; attesi indipendenti con
  7 errori ordinati, datatype primary senza cascata, summary/audit completi.
- Golden producer ora usa RecordValidator reale; vecchi attesi invariati.
- File runner/support, schema/input/attesi nuovi, README, generatore oracle/stato/report.
- ErrorFactory già corretta, nessun codice produzione o dipendenza modificati.
- 49 file protetti identici; CI nativa verificata, git work/p1-foundations.
- Prerequisiti 9-05/5-07 soddisfatti; next 10-01 dopo chiusura P9.
- P11/P13/PERSIST/T-P2-03/M1 aperti; nessun supporto Full dichiarato.

## Registro storico — T-P9-05 conclusa

## Frontiera corrente — T-P9-05 DONE / avvio T-P9-06 — 2026-10-22

Compatibilità vincolo/tipo verificata, 366 test locali in 46 suite; gate locali
fmt/check/Clippy/test/release/Windows GNU check/diff passati. CI nativa
39761531728 completed/success Linux/Windows su 2f3b36fbab11a7bfe755442675876cc7626df7b2.
[Dossier e spiegazioni](docs/verification/T-P9-05.md).

HANDOFF — T-P9-05 DONE
- ConstraintTypeMismatch per Date/Range su tipo errato; domini/range coerenti
  già verificati mantengono i diagnostici di parametri, nessuna coercizione Decimal.
- File compiler/linter, test matrice e regressioni date/range/Decimal; dossier/stato.
- Git work/p1-foundations, 2f3b36f pubblicato; CI Linux/Windows verificata.
- Prerequisiti 9-01..04 verificati, next 9-06 dopo gate; nessun blocco nuovo.
- P11/P13/PERSIST/T-P2-03/M1 restano aperti, LocalOnly esplicito.

## Registro storico — T-P9-04 conclusa

## Frontiera corrente — T-P9-04 DONE / avvio T-P9-05 — 2026-10-22

Regex compiled su byte implementata, 363 test locali passati (45 suite).
fmt/check/Clippy/test/release/Windows GNU check/diff passati; CI nativa
39760756884 Linux/Windows success su c6b85cd9d15599ab319000b1f7f54b791aebf641.
Nessuna baseline o grammatica modificata. [Dossier](docs/verification/T-P9-04.md).

HANDOFF — T-P9-04 DONE
- Cache privata per compilazione, InvalidRegex contestualizzato, raw/logical e
  required/forbidden espliciti, E_INVALID_DOMAIN senza codici nuovi.
- File: compiler/linter, validator Regex/mod, pipeline/record capability;
  test regex/CLI/lint, registro dipendenze, dossier e stato.
- DEC-15 ramo regex pure Rust confermato su Linux GNU/Windows MSVC; Cargo invariato.
- Git work/p1-foundations, c6b85cd pubblicato, native CI verificata.
- Next T-P9-05 dopo gate; nessun blocco contrattuale nuovo.
- P11/P13/PERSIST/T-P2-03/M1 restano aperti, solo LocalOnly eseguibile.

## Registro storico — T-P9-03 conclusa

## Frontiera corrente — T-P9-03 DONE / avvio T-P9-04 — 2026-10-22

Decimal consegnata su **5866d1dc80f18f8c447ffd70161c592a30d99e3b**,
[CI 39759270541](https://github.com/UraganoAngelina/lintra/actions/runs/39759270541)
completed/success Linux GNU e Windows MSVC, 354 test locali. Formato 1 e 54
file baseline protetti invariati; nessuna conversione di codici/range monetari.
Mappa formato 2 collega la baseline sviluppo 0.1.0 a questo commit verificato.

HANDOFF — 2026-10-22 — T-P9-03 DONE / T-P9-04 autorizzata
- Nuovo Decimal esplicito formato 2 verificato via compiler, pipeline e CLI;
  schema v1 e codici/range preesistenti invariati, zero ammesso, unsigned/dot/2.
- File/spiegazioni in dossier 9-03; autorità nella proposta ratificata e prompt.
- Tutti gate locali e CI nativa superati, confronto baseline SHA-256 positivo.
- Git work/p1-foundations, codice 5866d1d pubblicato, chiusura documentale qui.
- Next 9-04, prerequisito 6-06 già verificato; nessun blocco contrattuale.
- P11/P13/PERSIST/T-P2-03/M1 ancora aperti; nessun supporto globale dichiarato.

## Registro storico — gate locali T-P9-03

Ratifica dell'autore ricevuta: «approvo questa estensione, procedi pure».
Formato 2 implementato con Decimal per importi espliciti, formato 1 invariato.
354 test locali passati e tutti i gate locali verdi; CI nativa pendente.
Il confronto SHA-256 conferma identici 54 file protetti (normative, schemas,
golden preesistenti e meta-schema v1). Nessun campo codice/range convertito.
Spiegazioni e fonti: [dossier T-P9-03](docs/verification/T-P9-03.md),
[prompt ratificato](docs/verification/T-P9-03-prompt.md).

HANDOFF — 2026-10-22 — T-P9-03 gate locali
- Prerequisiti 6-06/G0 e ratifica Decimal soddisfatti; blocco contrattuale rimosso.
- Nuovi DecimalSpec/SignPolicy read-only, raw Decimal esplicito, format 2 gate,
  validazione fisica parametrica senza float; processo reale CLI verificato.
- Source autorizzate aggiornate: authoring, DET §137, lowering, mappa/prompt;
  schema v1 e baseline/codici/range restano invariati.
- 354 test, log /tmp/lintra-t-p9-03-test.log; fmt/check/Clippy/test/release/
  Windows GNU check/diff, riproducibilità raw e authoring v1/v2 passati.
- ABI Rust pubblico ampliato con FieldType::Decimal e metadata read-only;
  nessuna apertura costruttori compiled, variante richiede match consumer aggiornati.
- Nessuna dependency nuova/Cargo.lock cambiato. P11/P13/PERSIST/M1 aperti.
- Git work/p1-foundations, ultimo commit 92013f9, task da pubblicare per CI.
- Next T-P9-04 dopo gate Linux/Windows. Versione minima formato 2: baseline
  sviluppo 0.1.0 della consegna T-P9-03, nessuna release o vecchio supporto inventato.

## Registro storico — prima della ratifica Decimal

Ultimo codice **fa547431e702e37f2cf53203a3286d1f11ae9a32**: T-P9-02 DONE,
[CI 39749927727](https://github.com/UraganoAngelina/lintra/actions/runs/39749927727)
completed/success, Linux GNU e Windows MSVC success verificati. 340 test locali,
fmt/check/Clippy/test/release/Windows GNU check/diff passati. P8/M3 locale già
chiusi; range e date eseguibili dalla CLI. Nessuna baseline o fonte alterata.

**T-P9-03 BLOCKED soltanto sulla ratifica dell'estensione dati/versionamento.**
L'autore ha ora definito il profilo importi: punto al terzultimo byte, due cifre
decimali, nessun raggruppamento o segno, valori non negativi, zero ammesso.
Decimal solo per importi monetari diretti; campi di compartecipazione con
codici/range conservano integralmente tipo/vincoli. Nessuna classificazione
basata sul nome, nessuna conversione di baseline autorizzata o effettuata.

Proposta reviewabile: [contratto Decimal importi](docs/proposals/decimal-amount-contract.md).
Propone nodo esplicito e formato 2 con formato 1 invariato; numero/versionamento
e aggiornamento dei documenti sorgente restano da ratificare. Nessun codice
Decimal scritto, nessuna funzionalità o gate aggiuntivo dichiarati superati.
Il [preflight](docs/verification/T-P9-03-preflight.md) distingue chiarimenti
attuali e origine storica del blocco. T-P9-05/P9 completa restano aperte.

HANDOFF — 2026-10-22 — chiarimento Decimal, proposta dati da ratificare
- Conferme autore: importi diretti soltanto; punto fisico terzultimo, scala due,
  unsigned, zero valido. Codici/range di compartecipazione rigorosamente invariati.
- File modificati: questo registro, preflight 9-03 e nuova proposta Decimal.
- Nessun Rust/schema/source/baseline modificato; soli documenti di avanzamento.
- Nessun test runtime rieseguito per questa modifica documentale. Ultime evidenze
  invariate: 340 test e CI 39749927727 positiva sul codice fa54743.
- Next: ratifica della proposta dati/versionamento; poi implementazione T-P9-03
  già autorizzata nel workflow, senza riaprire le decisioni normative ricevute.

HANDOFF — 2026-10-22 — T-P9-02 DONE / arresto contrattuale T-P9-03
- Sessione: recuperata/chiusa 8-02, completate 8-03..06 e 9-01..02 con CI nativa
  positiva per ciascun codice. M3 locale DET §151 raggiunta, Full non disponibile.
- Ultimi file Rust: validator date/datatype, compiler, pipeline e gate record;
  unit/integration/process test, spiegazioni nel dossier 9-02. Nessuna API
  modello, dipendenza o baseline nuova in 9-02; chrono già presente riusato.
- Ultime evidenze: 340 test locali, tutti gate passati, CI 39749927727 nativa
  success su fa54743. Test console Windows reale introdotto in 8-06 e passato.
- File chiusura: questo registro, docs/verification/T-P9-02.md,
  docs/verification/T-P9-03-preflight.md e README.md; nessun codice aggiuntivo.
- Blocco: contratto Decimal escluso da formato 1, G0 lo rinvia esplicitamente;
  manca ratifica di un'estensione, vietato inventare parametri o fingere supporto.
- Next: decisione sul rinvio/riordino a T-P9-04 oppure contratto Decimal approvato.
  Prerequisito 9-04 T-P6-06 soddisfatto; nessuna implementazione 9-04 avviata.
- Rinvii P11/P13/PERSIST e T-P2-03/M1 invariati; CNF-AUDIT/RECORD/SERVICE-DI/
  RUN-SCOPE e DEC-12 preservate. Nessun Superpowers/subagente usato dopo mandato.
- Git work/p1-foundations, codice fa54743 pubblicato; chiusura documentale
  separata in questa sessione. Nessuna modifica preesistente sovrascritta.

## Registro storico — T-P9-02 / 2026-10-22

T-P9-02 **DONE_UNVERIFIED**: Date DDMMYYYY/0001–9999 calendariale con formato
compiled esistente, compiler/pipeline/record/CLI integrati. 340 test locali e
check/fmt/Clippy/release/Windows GNU check/diff passati; CI nativa pendente.

HANDOFF — 2026-10-22 — T-P9-02 gate locali
- Prerequisiti 6-06/G0 verificati, 9-01 CI 39748653454 Linux/Windows verde.
- File: src/validators/{date,datatype,mod}.rs, src/schema/compiler.rs,
  src/schema/compiled/tests.rs, src/engine/{field_pipeline,record_validator}.rs,
  tests/{calendar_date,cli_validate,schema_compiler}.rs, dossier 9-02 e registro.
- Sette nuovi test, 340 globali passati; log /tmp/lintra-t-p9-02-test.log.
  Gate locali passati, test Windows nativi ancora da CI.
- Date base calendariale con chrono già presente, nessun crate/API/formato nuovo;
  una sola causa primaria, presenza prima di calendario/domains. Fonti invariate.
- Git work/p1-foundations, ultimo commit 885c25d; task da pubblicare per CI.
- Next T-P9-03 dopo gate: rinvio G0 Decimal da risolvere prima di implementare,
  senza ampliare silenziosamente enum/raw/grammar. P11/P13/PERSIST/M1 aperti.

## Chiusura T-P9-01 — 2026-10-22

T-P9-01 **DONE**, CI 39748653454 completed/success su
dc4b6ec79660de65700c19c2a9db9916531d16c8, Linux GNU/Windows MSVC success.

HANDOFF — 2026-10-22 — T-P9-01 DONE / avvio T-P9-02
- 333 test locali e gate nativi confermati; file/spiegazioni nel dossier 9-01.
- Codice dc4b6ec pubblicato, branch work/p1-foundations; chiusura documentale qui.
- Next T-P9-02, prerequisiti 6-06 e G0 verificati. Contratto DDMMYYYY,
  ASCII 8 byte e anni 0001–9999 già approvato; nessuna estensione di formato.
- Decimal resta rinvio contrattuale G0, non autorizzato come nuovo tipo formato 1.

## Registro storico — T-P9-01 / 2026-10-22

T-P9-01 **DONE_UNVERIFIED**, range intero inclusivo esatto con payload compiled
esistente, compiler/type check, pipeline e preflight record integrati. 333 test
locali passati, tutti i gate locali verdi. CI nativa da acquisire.

HANDOFF — 2026-10-22 — T-P9-01 gate locali
- Prerequisito 6-06 verificato; P8/M3 locale chiusi su CI 39747270986.
- File: src/validators/{mod,range}.rs, src/schema/compiler.rs,
  src/schema/compiled/tests.rs, src/engine/{field_pipeline,record_validator}.rs,
  tests/{numeric_range,allowed_values,cli_validate,cli_schema_lint}.rs,
  dossier 9-01 e registro. Nessun crate/API modello/formato/baseline nuovo.
- Un test unitario, sei integration e uno processo nuovi; 333 globali passati,
  log /tmp/lintra-t-p9-01-test.log. fmt/check/Clippy/test/release/Windows GNU
  check/diff passati. Vecchie fixture unsupported ora usano nodi ancora assenti.
- Range solo dopo datatype; niente float o bound di macchina, input raw immutato.
- Git work/p1-foundations, ultimo commit 0fc698a; task da pubblicare per CI.
- Next T-P9-02 dopo gate. Decimal 9-03 resta rinviato dal contratto G0,
  non si amplia formato 1 senza ratifica; altri rinvii core invariati.

## Chiusura P8 / M3 locale — 2026-10-22

T-P8-06 **DONE**, CI 39747270986 completed/success su
90528c4ff48275ab115eb7f2fd10daa63c8223a9, Linux GNU/Windows MSVC success.
Test processo console Windows reale incluso e passato. **M3 locale DET §151
raggiunta nel perimetro ratificato**, checklist nel dossier T-P8-06.
Non equivale a M4/M5, Full, PERSIST o chiusura T-P2-03/M1.

HANDOFF — 2026-10-22 — T-P8-06 DONE / avvio T-P9-01
- 325 test Linux, gate locali e native CI confermati; file nel dossier 8-06.
- Nessuna produzione/dependency/core/baseline modificata in 8-06.
- Branch work/p1-foundations, codice 90528c4 pubblicato, chiusura documentale qui.
- Next T-P9-01 autorizzata, prerequisito T-P6-06 verificato (CI 39437225233).
- NumericRange contratto già definito: limiti interi inclusivi min≤max, logical
  dopo datatype. Decimal rimane fuori formato 1 in attesa di ampliamento approvato.

## Registro storico — T-P8-06 / 2026-10-22

T-P8-06 **DONE_UNVERIFIED**, 325 test Linux e gate locali passati; test Windows
console-event nuovo compilato/lintato GNU ma non ancora eseguito nativamente.
Quattro nuove prove portabili: no opzioni/help, CLI sopra env, coppia env,
eseguibile copiato/bundle adiacente/no fallback. Nessun codice produzione nuovo.

HANDOFF — 2026-10-22 — T-P8-06 gate locali
- Prerequisiti 8-03/04/05 CI nativa verificata e positiva.
- File: tests/cli_validate.rs, tests/cli_windows_signal.rs, dossier 8-06 e registro.
- 325 test locali, fmt/check/Clippy/test/release/Windows GNU check/diff passati;
  anche Clippy Windows GNU all-targets passato. CI nativa da acquisire.
- Windows test console dedicato: Kernel32 ABI test-only, gruppo figlio nonzero
  e CTRL_BREAK_EVENT; non broadcast al runner, nessun crate aggiunto.
- M3 locale DET §151 da confermare dopo CI; non chiude M1/T-P2-03 o capacità future.
- Git work/p1-foundations, ultimo commit 120ba7e, task da pubblicare per CI.
- Next T-P9-01 dopo gate; rinvii e contratti preservati, nessun blocco nuovo.

## Chiusura T-P8-05 — 2026-10-22

T-P8-05 **DONE**, CI 39746072212 completed/success su
6417c408f98951b6f60601d07f169e55ccc3e827, Linux GNU/Windows MSVC success.

HANDOFF — 2026-10-22 — T-P8-05 DONE / avvio T-P8-06
- 321 test locali e gate nativi confermati; file/spiegazioni nel dossier 8-05.
- Windows ha eseguito test callback/token/atomicità; nessun segnale console
  Windows reale dichiarato. SIGINT reale verificato su Linux.
- Branch work/p1-foundations, codice 6417c40 pubblicato; chiusura documentale qui.
- Next T-P8-06, prerequisiti 8-03/04/05 tutti verificati e consegnati; nessun
  nuovo conflitto; rinvii core e baseline inalterati.

## Registro storico — T-P8-05 / 2026-10-22

T-P8-05 **DONE_UNVERIFIED**: Ctrl+C nel binario imposta token condiviso,
321 test locali passati inclusi SIGINT Linux reale, callback e adapter cancel
portabili. Tutti i gate locali passati, CI nativa pendente. Registrazione DEC-15
ctrlc =3.5.2 prima di Cargo.toml, licenze/tree verificati nel registro dipendenze.

HANDOFF — 2026-10-22 — T-P8-05 gate locali
- Prerequisito 8-03 verificato su CI 39743161012 Linux/Windows verde.
- File: Cargo.toml/lock, src/cli/{signal,command,mod}.rs, tests/cli_validate.rs,
  README.md, docs/DEPENDENCIES.md, dossier T-P8-05 e questo registro.
- Test 321 passati, log /tmp/lintra-t-p8-05-test.log; fmt/check/Clippy/release/
  Windows GNU check/diff passati. SIGINT processo reale Linux: 130 e report
  precedenti preservati; Windows segnale console reale non eseguito, test
  callback e token precancellato portabili inclusi per CI nativa.
- Nessuna API core, fonte o baseline modificata; semantiche EOF/audit/late
  cancellation invariate. Rinvii P11/P13/PERSIST e T-P2-03/M1 invariati.
- Git work/p1-foundations, ultimo commit 5f3915d; task da pubblicare per CI.
- Next T-P8-06 dopo gate; nessun blocco contrattuale nuovo.

## Chiusura T-P8-04 — 2026-10-22

T-P8-04 **DONE**: 318 test locali e CI 39744299145 completed/success su
4f7115fcf784c02511dc78b3217fda399f53703d, Linux GNU/Windows MSVC success.

HANDOFF — 2026-10-22 — T-P8-04 DONE / avvio T-P8-05
- File/spiegazioni nel dossier T-P8-04, nessuna modifica compiler o API core.
- Codice 4f7115f pubblicato, branch work/p1-foundations, chiusura documentale qui.
- Next T-P8-05 autorizzata, prerequisito 8-03 CI 39743161012 soddisfatto;
  registry/compiler e report atomici preservati. Rinvii core invariati.

## Registro storico — T-P8-04 / 2026-10-22

T-P8-04 **DONE_UNVERIFIED**: schema lint attraverso registry/compiler reali,
nessun controllo duplicato; 318 test locali e tutti i gate locali passati.
CI nativa da acquisire. Dettagli in [T-P8-04](docs/verification/T-P8-04.md).

HANDOFF — 2026-10-22 — T-P8-04 gate locali
- Prerequisiti 3-04/8-01 verificati, 8-03 chiusa su CI 39743161012 Linux/Windows.
- File: src/cli/{args,command,mod,tests}.rs, tests/{cli_schema_lint,cli_validate}.rs,
  README.md, docs/verification/T-P8-04.md e questo registro.
- Cinque nuovi test processo: valido, strutturale/semantico, unsupported,
  errori tecnici del loader rigoroso, help/sintassi; 318 globali passati.
- fmt/check/Clippy/test/release/Windows GNU check/diff passati; CI nativa pendente.
- Contratti core/dependency/fonti/baseline invariati; P11/P13/PERSIST/M1 aperti.
- Git work/p1-foundations, ultimo commit 9248fe0; task da committare/pubblicare.
- Next T-P8-05 dopo gate, nessun blocco contrattuale rilevato.

## Chiusura T-P8-03 — 2026-10-22

T-P8-03 **DONE**, [CI 39743161012](https://github.com/UraganoAngelina/lintra/actions/runs/39743161012)
completed/success su 10932f9c0345ec60009624940596bad23fed6b77; Linux GNU e
Windows MSVC entrambi success verificati. 313 test locali, gate locali passati.

HANDOFF — 2026-10-22 — T-P8-03 DONE / avvio T-P8-04
- File e spiegazioni nel dossier T-P8-03; API core e baseline/fonti invariate.
- Branch work/p1-foundations, codice 10932f9 pubblicato, chiusura documentale qui.
- Next T-P8-04 autorizzata; prerequisiti T-P3-04 (CI 39257768950) e T-P8-01
  (CI 39628560176) verificati documentalmente, regressioni locali rieseguite.
- Nessun nuovo conflitto; P11/P13/PERSIST e residuo T-P2-03/M1 invariati.

## Registro storico — T-P8-03 / 2026-10-22

T-P8-03 **DONE_UNVERIFIED**: comando validate LocalOnly reale, report atomici
per file, riepilogo human/stdout, eventi applicativi stderr, --version con
provenienza build. Nessun nuovo contratto core o dipendenza. Nove test processo
nuovi Linux/Unix, sette portabili Windows; **313 test locali passati**.
fmt/check/Clippy -D warnings/test/release/Windows GNU check/diff tutti exit 0.
CI nativa da acquisire prima di avviare T-P8-04. Spiegazioni dettagliate:
[report T-P8-03](docs/verification/T-P8-03.md).

HANDOFF — 2026-10-22 — T-P8-03 gate locali
- Prerequisiti 8-01/8-02/5-06 verificati; 8-02 CI 39741868369 Linux/Windows verde.
- File: build.rs, src/cli/{args,mod,command}.rs, tests/cli_validate.rs,
  README.md, dossier T-P8-03 e questo registro.
- Test: 313 passati, nove nuovi processo; log /tmp/lintra-t-p8-03-test.log.
  Tutti i gate locali passati, Windows GNU solo compilazione; CI nativa pendente.
- Errori tecnici abortiscono output provvisori; collisioni con input/bundle o
  output rifiutate. Atomicità per file invariata; secondo rename può fallire
  dopo prima pubblicazione, con exit 1. Nessuna transazione di coppia promessa.
- Contratti pubblici: invariati. Nessuna nuova dipendenza o baseline alterata.
- Rinvii P11/P13/PERSIST/T-P2-03/M1, Ctrl+C 8-05, schema lint 8-04 invariati.
- Git: work/p1-foundations, ultimo commit b4819c2; file di questa task da
  committare/pubblicare per CI. Nessun blocco contrattuale.
- Next T-P8-04 dopo conferma QG-2; avanzamento già autorizzato.

## Registro storico — 2026-10-22

T-P8-02 **DONE**: 304 test locali e [CI 39741868369](https://github.com/UraganoAngelina/lintra/actions/runs/39741868369)
completed/success su 81cee3686eded6dce53c2a5ba1f97f602e4434bb, Linux GNU e
Windows MSVC entrambi completed/success verificati con gh run view.
Mandato successivo: chiamate GitHub autonome e avanzamento task per task dopo
esito chiaro; restano gate e blocchi contrattuali reali. Nessun Superpowers/subagente.

HANDOFF — 2026-10-22 — T-P8-02 DONE / avvio T-P8-03
- Codice 81cee36, gate locali e CI nativa Linux/Windows confermati.
- File di chiusura: questo registro e docs/verification/T-P8-02.md.
- Nessun nuovo codice, API, dipendenza o modifica alle baseline/fonti.
- Prossima task T-P8-03 autorizzata, prerequisiti 8-01/8-02/5-06 soddisfatti.
- Rinvii P11/P13/PERSIST/T-P2-03/M1 invariati.
- Branch work/p1-foundations; working tree con soli documenti di questa sessione.

## Recupero sessione — 2026-10-22 / T-P8-02

**Aggiornamento successivo:** accesso GitHub consentito e ripristinato.
Avviata [CI 39741868369](https://github.com/UraganoAngelina/lintra/actions/runs/39741868369)
su `81cee3686eded6dce53c2a5ba1f97f602e4434bb`. Ultima risposta verificata:
Linux GNU completed/success, Windows MSVC in_progress, run in_progress.
Tre chiamate Actions eseguite (list/dispatch/view), distanziate oltre 45 s;
raggiunto il limite autorizzato, nessuna quarta chiamata. Nessun esito Windows
presunto. T-P8-03 resta autorizzata dopo gate ma non iniziata.

Ultimo codice `81cee3686eded6dce53c2a5ba1f97f602e4434bb`, branch
`work/p1-foundations`, working tree inizialmente pulito. T-P8-01 ultima task
chiusa documentalmente; T-P8-02 implementata, resta **DONE_UNVERIFIED** per CI.
Rieseguiti fmt/check/Clippy -D warnings/test/release/Windows GNU check: tutti
exit 0, **304 test passati**, zero falliti. Dettagli nel dossier T-P8-02.
Accesso GitHub dal sandbox fallito; escalation rifiutata dall'utente.
CI nativa Linux/Windows non verificata in questa sessione, nessuna run avviata.
Autorizzata dall'autore la task successiva dopo conclusione senza ambiguità
della precedente; T-P8-03 non iniziata perché QG-2 ancora da verificare.
Confermato divieto di Superpowers/subagenti. Nessuna modifica di produzione.

HANDOFF — 2026-10-22 — recupero T-P8-02
- Fase/task attiva: P8 / T-P8-02; stato DONE_UNVERIFIED.
- File modificati: questo registro e docs/verification/T-P8-02.md, soltanto
  evidenze di recupero; nessun codice o test aggiunto, nessuna baseline alterata.
- Evidenze: fmt, check all-targets, Clippy all-targets -D warnings,
  test --locked (304 passati), build release e check Windows GNU tutti exit 0.
- Non eseguiti: test Windows nativi locali; CI remota non consultabile dopo
  errore connessione sandbox e rifiuto dell'accesso esterno. Nessuna run avviata.
- Contratti/decisioni: invariati; ratifiche già approvate preservate.
- Documenti da aggiornare: esito CI in questi due documenti quando verificabile.
- Blocco operativo: evidenza QG-2 mancante; nessun conflitto architetturale nuovo.
- Rinvii: P11/P13/PERSIST e residuo T-P2-03/M1 invariati.
- Git: work/p1-foundations, HEAD 81cee36; due file documentali modificati
  in questa sessione, non committati.
- Next: verificare CI su 81cee36; se Linux/Windows positivi, chiudere T-P8-02
  e procedere a T-P8-03 con l'autorizzazione già ricevuta.

HANDOFF — 2026-10-22 — successiva autorizzazione / CI T-P8-02 avviata
- Stato: DONE_UNVERIFIED, manca esito finale Windows; Linux verde verificato.
- Evidenza: run 39741868369 su 81cee36, stato in_progress all'ultima lettura.
- File: solo IMPLEMENTATION_STATUS.md e docs/verification/T-P8-02.md aggiornati;
  nessun codice/test nuovo, contratti invariati; 304 test locali già confermati.
- Vincolo operativo: consumate tre chiamate Actions distanziate oltre 45 s.
- Git: work/p1-foundations, HEAD 81cee36, due file documentali non committati.
- Next: acquisire esito finale della run quando consentito; dopo QG-2 positivo
  chiudere T-P8-02 e implementare T-P8-03 già autorizzata.

## Mandato corrente — 2026-10-21

L’autore revoca lo stop dopo T-P7-05 e autorizza la ripresa da T-P7-06,
con avanzamento automatico task per task dopo gate e in assenza di ambiguità.
Restano esclusi Superpowers/subagenti, fonti invariate e spiegazioni del codice.

## CNF-RUN-SCOPE-01 chiusa — T-P7-04 LocalOnly completata

Autore: «Approvo A: modalità esplicita nei report». ValidationMode neutro nel
dominio, metadata/summary/JSON con modalità obbligatoria, RunStatus invariato.
Il servizio LocalOnly e gli aggiornamenti tecnici sono implementati, 280 test
locali superati; CI 39587379476 Linux/Windows verde su dcb8c59. Fonti e baseline normative invariate.
Started_at già nei metadata è conservato nel summary per il writer del run;
nessun nuovo campo JSON per il timestamp, API precedente ancora compatibile.
T-P2-03/M1, P11/P13 restano aperti. Mandato successivo: ripresa autonoma da T-P7-06.

## Decision log — CNF-SERVICE-DI-01 chiusa / T-P7-05 DONE — LocalOnly API scope approved by CNF-SERVICE-DI-01

Ratifica esplicita dell’autore, 2026-10-21: proposta A con precisazioni §§1–13.
Factory `build_default_local_service(PathBuf, RawResolvedEncoding)` con reali
filesystem repository/registry/SystemClock; `new`/`with_clock` preservati.
Root obbligatoria e encoding invariato; nessun fallback o porta simulata.
Compiled pubblici solo in lettura, costruttori e invarianti protetti.
Porte P11/PERSIST e Full/P13 rinviate, core persistente resta obbligatorio.
Dettagli, audit API e ledger: [T-P7-05](docs/verification/T-P7-05.md).
Gate T-P7-04 ricontrollato via GitHub: CI 39587379476 success Linux/Windows
su dcb8c592fbec4b95db0b466dae83eb37f3dfa3cb.
Ratifica indipendente dalla CI. Nessuna modifica ARCH/DET/Piano/spec persistente.

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
  parte autonoma, T-P3-01 e T-P3-02 verificate; T-P3-03 verificata in locale e CI Linux/Windows; T-P3-04 verificata in locale e CI Linux/Windows; T-P3-05 verificata in locale e CI Linux/Windows; CNF-COMPILER-01/proposta A approvata; T-P3-06 verificata in locale e CI Linux/Windows; CNF-FORMAT-01 risolto con mappa approvata; T-P3-07 verificata in locale e CI Linux/Windows; T-P4-01 verificata in locale e CI Linux/Windows; T-P4-02..04 DONE: CI 39282115064 Linux/Windows verde su 7168a05; T-P6-01 e T-P4-05 DONE su CI 39389225120; T-P5-01 DONE, CI 39389657166 verde Linux/Windows su a679f40; T-P5-02 DONE, CI 39390492669 Linux/Windows verde su 39ed9b2; T-P5-03 DONE su CI 39390829911 Linux/Windows verde; T-P5-04 DONE su CI 39391897709 Linux/Windows verde (137 test); T-P5-05 DONE, CNF-AUDIT-01 risolta; [CI 39399359949](https://github.com/UraganoAngelina/lintra/actions/runs/39399359949) Linux/Windows verde su 85f1ac7, 155 test locali; stop rispettato e successivamente revocato per avviare T-P5-06. **T-P5-06 DONE**: output atomico e spiegazioni dettagliate del codice; 170 test locali, 171 Windows; [CI 39405353679](https://github.com/UraganoAngelina/lintra/actions/runs/39405353679) Linux/Windows verde su 0c48768. **T-P5-07 DONE**: harness golden generico e sei scenari sintetici; 180 test locali, 181 Windows; [CI 39416549983](https://github.com/UraganoAngelina/lintra/actions/runs/39416549983) Linux/Windows verde su b0e5ce7. **T-P6-02 DONE**: presenza Required/Optional e boundary Unknown; 186 test locali, 187 Windows; [CI 39418788154](https://github.com/UraganoAngelina/lintra/actions/runs/39418788154) Linux/Windows verde su c34649b. **T-P6-03 DONE**: sola estrazione logical, nessun controllo fisico/residuo formato 1; CNF-ALIGNMENT-01 archiviata. 193 locali, 194 Windows; [CI 39425268285](https://github.com/UraganoAngelina/lintra/actions/runs/39425268285) verde su bd49791. **T-P6-04 DONE**: datatype Numeric/AN, encoding e ByteSet; 205 locali, [CI 39430943109](https://github.com/UraganoAngelina/lintra/actions/runs/39430943109) Linux/Windows verde su 24b0e02. **T-P6-05 DONE**: AllowedValues raw/logical e compiler, 215 locali; [CI 39435239667](https://github.com/UraganoAngelina/lintra/actions/runs/39435239667) Linux/Windows verde c3caa5e. **T-P6-06 DONE**: pipeline ordinata, 226 locali; [CI 39437225233](https://github.com/UraganoAngelina/lintra/actions/runs/39437225233) Linux/Windows verde 65f1eb3. **T-P6-07 DONE**: identità da binding, 233 locali, [CI 39439627879](https://github.com/UraganoAngelina/lintra/actions/runs/39439627879) Linux/Windows verde c51c019. **T-P6-08 DONE — LocalOnly scope approved by CNF-RECORD-01**: 245 locali, [CI 39449345726](https://github.com/UraganoAngelina/lintra/actions/runs/39449345726) Linux/Windows verde daa9f15; stop successivamente revocato, T-P7-01 DONE (251 locali, CI 39567896281 Linux/Windows verde 20f6835); T-P7-02 DONE, 254 locali, CI 39569282614 Linux/Windows verde bacedba; T-P7-03 DONE, 263 locali, CI 39571479598 Linux/Windows verde 5d2f371; T-P14-03 DONE (268 locali), CI 39574640180 Linux/Windows verde da80163; T-P7-04 DONE LocalOnly (280 locali), CI 39587379476 Linux/Windows verde dcb8c59; T-P7-05 DONE — LocalOnly API scope approved by CNF-SERVICE-DI-01, 285 test locali e CI 39601419396 Linux/Windows verde su 074bbfe.
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

## Decisione finale T-P6-03 — sostituisce A parziale e B fisica

L'autore ha ratificato sola estrazione logical dal lato dichiarato e autorizzato
la revisione del prompt e DET §44. Niente validate_alignment/FieldViolation/
E_ALIGNMENT. Controllo fisico padding fuori ambito formato 1 **senza residuo**.
CNF-ALIGNMENT-01 archiviata come possibile estensione futura non implementata;
nessun parametro opposite_edge_fill. Dopo verifica procedere a T-P6-04.
Mandato completo tracciato in AGENTS.md e report T-P6-03, con fonti revisionate.

## CNF-RECORD-01 ratificata — T-P6-08 LocalOnly completata / STOP

L'autore approva la proposta A con precisazioni vincolanti nel
[dossier](docs/verification/T-P6-08-preflight.md): percorso LocalOnly esplicito,
risultato solo identity/valid locale, errori tecnici engine tipizzati,
ProjectionPlan invariato e non consumato, nessun placeholder/reference data.
P11/P13 e T-P2-03/M1 restano aperte. Fonti architetturali invariate.
Dopo T-P6-08 verificata e handoff, **STOP: nessuna implementazione P7 o altra
 task senza nuova autorizzazione**. Riordino P7 approvato solo come piano.

## Mandato corrente — ripresa autonoma dopo T-P5-07

Autore: «procedi con la prossima task, riprendiamo il workflow che ti permetteva
di cominciare in autonomia una task se non emergono dipendenze bloccanti o punti
aperti nelle task precedenti». Ripristinata progressione task per task dopo
verifica, senza conferme ordinarie, conservando gate e arresto sui blocchi reali.
Superati gli stop operativi storici; AGENTS.md aggiornato. Restano vincolanti
spiegazioni dettagliate del codice, recap dei file, nessun Superpowers/subagente.

## Autorizzazione successiva — T-P5-07

Dopo T-P5-06 DONE l'autore ha chiesto «procedi con la prossima task».
Avviata esclusivamente T-P5-07, con spiegazioni dettagliate di codice e test;
nessun servizio applicativo, validatore normativo o task successiva anticipato.

## Mandato successivo — 2026-10-20

Autore: «PROCEDI CON LA PROSSIMA TASK». Autorizzata T-P5-06 dopo lo stop
rispettato di T-P5-05. Nuova regola vincolante in AGENTS.md: spiegare
**dettagliatamente cosa fa e perché funziona ogni codice scritto**, incluse
struct, funzioni, implementazioni e condizioni. Applicata con commenti,
rustdoc e [spiegazione completa T-P5-06](docs/verification/T-P5-06.md).
Esecuzione senza Superpowers, subagenti o processi locali in background.

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
| QG-1 — controlli locali | PASS per T-P7-04 LocalOnly | fmt/check/clippy/release/GNU, 39 unit + 230 integration + 11 doctest = 280 |
| QG-2 — CI Linux/Windows | PASS per dcb8c59 | [Run reale](https://github.com/UraganoAngelina/lintra/actions/runs/39587379476), Linux 119179544895 e Windows 119179544684 completed/success |
| QG-3 — architettura | PASS nel perimetro LocalOnly ratificato | Moduli privati, compiled con sola lettura e costruzione privata, piani tipizzati senza evaluator; nessuna logica normativa o backend nel core |
| QG-8 — documenti | PASS con revisione autorizzata T-P6-03 | DET §44 e prompt kit aggiornati per mandato esplicito; hash prima/dopo registrati; ARCH/Piano invariati; nessun residuo fisico formato 1 |
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
| FieldType | Numeric/AN/Date formato 1 preservati; Decimal formato 2 ratificata e verificata T-P9-03 (5866d1d/CI 39759270541). Alphabetic ancora RINVIATO per repertorio assente | Nessun blocco Decimal residuo; ramo Alphabetic resta escluso, nessun no-op |
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
| T-P5-03 | DONE | 2026-10-20 | [CI 39390829911](https://github.com/UraganoAngelina/lintra/actions/runs/39390829911) verde Linux/Windows su 0f7b2df; 132 test |
| T-P5-04 | DONE | 2026-10-20 | [CI 39391897709](https://github.com/UraganoAngelina/lintra/actions/runs/39391897709) verde Linux/Windows su a804956; 137 test, fixture JSONL byte identici |
| T-P5-05 | DONE | 2026-10-20 | Audit/builder/summary e 12 casi; 155 test; [CI 39399359949](https://github.com/UraganoAngelina/lintra/actions/runs/39399359949) verde Linux/Windows su 85f1ac7 |
| T-P5-06 | DONE | 2026-10-20 | Output atomico, cleanup e policy diagnostica; 170 test locali, 171 Windows; [CI 39405353679](https://github.com/UraganoAngelina/lintra/actions/runs/39405353679) verde Linux/Windows su 0c48768; [spiegazione](docs/verification/T-P5-06.md) |
| T-P5-07 | DONE | 2026-10-20 | Sei scenari sintetici, confronto byte esatto e opt-in aggiornamenti; 180 test locali, 181 Windows; [CI 39416549983](https://github.com/UraganoAngelina/lintra/actions/runs/39416549983) verde su b0e5ce7; [spiegazione](docs/verification/T-P5-07.md) |
| T-P6-01 | DONE | 2026-10-20 | [CI 39389225120](https://github.com/UraganoAngelina/lintra/actions/runs/39389225120) verde Linux/Windows su 46ab648; 118 test |
| T-P6-02 | DONE | 2026-10-20 | Presenza base, Unknown per condizioni non valutate; 186 locali, 187 Windows; [CI 39418788154](https://github.com/UraganoAngelina/lintra/actions/runs/39418788154) verde su c34649b; [spiegazione](docs/verification/T-P6-02.md) |
| T-P6-03 | DONE | 2026-10-20 | Mandato finale: solo logical, nessun residuo; 193 locali/194 Windows, [CI 39425268285](https://github.com/UraganoAngelina/lintra/actions/runs/39425268285) verde bd49791; [evidenze](docs/verification/T-P6-03.md) |
| T-P6-04 | DONE | 2026-10-20 | Numeric/AN, profili/ByteSet, 205 locali, [CI 39430943109](https://github.com/UraganoAngelina/lintra/actions/runs/39430943109) Linux/Windows success su 24b0e02; [spiegazione](docs/verification/T-P6-04.md) |
| T-P6-05 | DONE | 2026-10-20 | Membership byte esatto, target/width compiler, 215 locali; [CI 39435239667](https://github.com/UraganoAngelina/lintra/actions/runs/39435239667) Linux/Windows verde c3caa5e; [report](docs/verification/T-P6-05.md) |
| T-P6-06 | DONE | 2026-10-20 | Pipeline presenza/logical/datatype/constraint, 226 locali, CI 39437225233 Linux/Windows verde 65f1eb3; [report](docs/verification/T-P6-06.md) |
| T-P6-07 | DONE | 2026-10-20 | IdentityExtractor e lowering binding, 233 locali; CI 39439627879 Linux/Windows verde c51c019; [report](docs/verification/T-P6-07.md) |
| T-P6-08 | DONE — LocalOnly scope approved by CNF-RECORD-01 | 2026-10-20 | validate_local, 245 locali, CI 39449345726 Linux/Windows verde daa9f15; [spiegazioni e rinvii](docs/verification/T-P6-08.md) |
| T-P7-01 | DONE | 2026-10-21 | Request/preflight Trace LocalOnly, 251 locali; CI 39567896281 Linux/Windows verde 20f6835; [report](docs/verification/T-P7-01.md) |
| T-P7-02 | DONE | 2026-10-21 | Token condiviso e progress, 254 locali; CI 39569282614 Linux/Windows verde bacedba; [report](docs/verification/T-P7-02.md) |
| T-P7-03 | DONE | 2026-10-21 | Loop streaming, audit/cancellazione, 263 locali; CI 39571479598 Linux/Windows verde 5d2f371; [report](docs/verification/T-P7-03.md) |
| T-P14-03 | DONE | 2026-10-21 | Canonical JSON/hash e registry, 268 locali; CI 39574640180 Linux/Windows verde da80163; [report](docs/verification/T-P14-03.md) |
| T-P7-04 | DONE — LocalOnly | 2026-10-21 | Servizio LocalOnly/clock/mode, 280 locali; CI 39587379476 Linux/Windows verde dcb8c59; [report](docs/verification/T-P7-04.md) |
| T-P7-05 | DONE — LocalOnly API scope approved by CNF-SERVICE-DI-01 | 2026-10-21 | 285 test locali, [CI Linux/Windows verde](https://github.com/UraganoAngelina/lintra/actions/runs/39601419396) su 074bbfe; [audit API/ledger](docs/verification/T-P7-05.md); stop successivamente revocato |
| T-P7-06 | DONE — LocalOnly | 2026-10-21 | 289 test; CI 39621523482 Linux/Windows verde b402fdf; [report](docs/verification/T-P7-06.md) |
| T-P8-01 | DONE | 2026-10-21 | 297 test, CI 39628560176 Linux/Windows success 98d952b; [report](docs/verification/T-P8-01.md) |
| T-P8-02 | DONE | 2026-10-22 | HumanReadableSink, 304 test locali, CI 39741868369 Linux/Windows completed/success su 81cee36; [report](docs/verification/T-P8-02.md) |
| T-P8-03 | DONE | 2026-10-22 | validate reale LocalOnly, 313 test locali, CI 39743161012 Linux/Windows verde su 10932f9; [report](docs/verification/T-P8-03.md) |
| T-P8-04 | DONE | 2026-10-22 | schema lint, 318 test locali, CI 39744299145 Linux/Windows verde su 4f7115f; [report](docs/verification/T-P8-04.md) |
| T-P8-05 | DONE | 2026-10-22 | Ctrl+C cooperativo, 321 test locali, CI 39746072212 Linux/Windows verde su 6417c40; [report](docs/verification/T-P8-05.md) |
| T-P8-06 | DONE | 2026-10-22 | 325 locali, CI 39747270986 Linux/Windows verde su 90528c4, test console Windows passato; [report](docs/verification/T-P8-06.md) |
| T-P9-01 | DONE | 2026-10-22 | range inclusivo esatto, 333 locali, CI 39748653454 Linux/Windows verde su dc4b6ec; [report](docs/verification/T-P9-01.md) |
| T-P9-02 | DONE | 2026-10-22 | date DDMMYYYY, 340 locali, CI 39749927727 Linux/Windows verde su fa54743; [report](docs/verification/T-P9-02.md) |
| T-P9-03 | DONE | 2026-10-22 | formato 2 Decimal, 354 test, CI 39759270541 Linux/Windows verde su 5866d1d; [preflight](docs/verification/T-P9-03-preflight.md) |
| T-P9-04 | DONE | 2026-10-22 | regex bytes, 363 test locali; CI 39760756884 Linux/Windows verde su c6b85cd; [report](docs/verification/T-P9-04.md) |
| T-P9-05 | DONE | 2026-10-22 | compatibilità, 366 test locali; CI 39761531728 Linux/Windows verde su 2f3b36f; [report](docs/verification/T-P9-05.md) |
| T-P9-06 | DONE | 2026-10-22 | golden semantici, 367 test; CI 39762338659 Linux/Windows verde su 3fafc4b; [report](docs/verification/T-P9-06.md) |
| T-P10-01 | DONE | 2026-10-22 | lowering Expr, 374 test; CI 39763287298 Linux/Windows verde su b8e1e80; [report](docs/verification/T-P10-01.md) |
| T-P10-02 | DONE | 2026-10-22 | TruthValue, 376 test; CI 39764219198 Linux/Windows verde su e17c33a; [report](docs/verification/T-P10-02.md) |
| T-P10-03 | DONE | 2026-10-22 | evaluator, 384 test; CI 39780196256 Linux/Windows verde su a6c7138; [report](docs/verification/T-P10-03.md) |
| T-PERSIST-01..12 | NOT_STARTED | — | Nessun contratto o backend |
| T-CAD-01..12 | NOT_STARTED | — | Nessun runtime CAD |

## Registro verifiche corrente — Linux locale

2026-10-21 T-P7-04: 280 locali (39 unit + 230 integration + 11 doctest),
fmt/check/clippy/test/release/GNU-all-targets/diff PASS; mode approvata,
otto JSON con sola aggiunta validation_mode verificati byte per byte.
CI 39587379476 dcb8c59: Linux 119179544895 / Windows MSVC 119179544684
completed/success. Clock fisso realmente iniettato, acquisito una sola volta;
errori/cancellazioni/audit EOF e serializzazione locale esercitati dal servizio.


2026-10-21: T-P7-01 251 locali / CI 39567896281 su 20f6835,
Linux 119114529989 / Windows 119114530347 success;
T-P7-02 254 locali / CI 39569282614 su bacedba,
Linux 119119128861 / Windows 119119129190 success;
T-P7-03 263 locali / CI 39571479598 su 5d2f371,
Linux 119126471890 / Windows 119126471439 success;
T-P14-03 268 locali / CI 39574640180 su da80163,
Linux 119137007062 / Windows 119137006791 success.
Per ogni task fmt/check/clippy/test/release/GNU/diff PASS. GNU locale è solo
compilazione; Windows eseguito su MSVC CI. Timeout iniziali GitHub/push/dispatch
superati con retry, nessun test fallito in CI o baseline adattata al codice.


T-P6-08 LocalOnly: fmt/check/clippy/test/release/GNU-all-targets/diff-check PASS;
245 locali (39 unit + 195 integration + 11 doctest). Dieci integration e due
unit nuovi. Sink/identity/pipeline/counter realmente esercitati; projection
non vuota consentita e invariata, nodi locali non supportati rifiutati.
CI 39449345726 sul commit daa9f15: Linux 118719672526 e Windows MSVC
118719672356 completed/success. [Codice e handoff](docs/verification/T-P6-08.md).


T-P6-07: fmt/check/clippy/test/release/GNU-all-targets/diff-check PASS;
233 locali (37 unit + 185 integration + 11 doctest). Sette test identity
nuovi e dodici regression compiler verdi; CI 39439627879 su c51c019:
Linux 118685756716 e Windows 118685756618 completed/success.
[Contratti, condizioni e spiegazioni](docs/verification/T-P6-07.md).

T-P6-06: fmt/check/clippy/test/release/GNU-all-targets/diff-check PASS;
226 locali (37 unit, 178 integration, 11 doctest). CI 39437225233 su 65f1eb3:
Linux 118677332148 e Windows 118677331247 completed/success.
Stop al primo errore primario; guasti tecnici tipizzati e terminali.
[Spiegazioni complete](docs/verification/T-P6-06.md).

T-P6-05: fmt/check/clippy/test/release/GNU-all-targets/diff-check PASS;
215 locali, dieci integration nuovi. Regression compiler invariati e verdi.
Prerequisito T-P6-04 CI 39430943109 Linux/Windows success su 24b0e02.
CI 39435239667 su c3caa5e: Linux 118670448108 e Windows 118670447892
completed/success. [Spiegazioni](docs/verification/T-P6-05.md).

T-P6-04: fmt/check/clippy/test/release/GNU-all-targets/diff-check PASS;
205 test locali, 35 unit + 159 integration + 11 doctest. Undici integration
Datatype e un unit tecnico nuovi. Tutti i 256 byte verificati per CP1252/ISO,
nessun cambiamento al binding compiler UTF-8-only. CI 39430943109 su 24b0e02:
Linux 118655543578 e Windows 118655542644 completed/success.
[API, confini e spiegazioni](docs/verification/T-P6-04.md).

T-P6-03: fmt/clippy/test/release/GNU-all-targets PASS, 193 test locali;
sette nuovi integration su zero-copy e policy. Check/diff-check PASS.
Audit delle revisioni autorizzate PASS: prima=manifest originale, dopo=file
corrente, copie byte identiche, nessun cambiamento DET fuori §44. Due file kit
ora intenzionalmente diversi dal manifest; manifest invariato, nessun PASS
inventato per integrità del kit originario. CI 39425268285 su bd49791:
Linux 118636134483 e Windows 118636134182 completed/success;
log Windows 194 test PASS. Nessun residuo fisico.
[Contratto, spiegazioni ed evidenze](docs/verification/T-P6-03.md).

T-P6-02: fmt/check/clippy/test/release/GNU-all-targets/diff-check PASS;
186 test locali (34 unit + 141 integration + 11 doctest), cinque integration
presence e un unit boundary nuovi. Prerequisiti T-P6-01 e T-P5-02 riconfermati
via API, entrambe CI Linux/Windows completed/success. QG-2 PASS su
c34649b0873fa3a8bfc837ec314093c8322f1919: CI 39418788154,
Linux 118614279561 e Windows 118614279377 completed/success.
Log Windows verificato: 187 test PASS, inclusi i nuovi test presence.
[Contratto, adattamento DET §147 e spiegazioni](docs/verification/T-P6-02.md).

T-P5-07: fmt/check/clippy/test/release/GNU-all-targets/diff-check PASS;
180 test locali (33 unit + 136 integration + 11 doctest), incluso runner con
sei scenari. Aggiornamento effettivo LINTRA_UPDATE_GOLDEN=1 PASS senza
cambiamento di byte degli attesi indipendenti; -text attivo sulle fixture.
Prerequisiti T-P5-04/05 riconfermati tramite API: entrambe CI Linux/Windows
verdi sui commit già registrati. QG-2 PASS su b0e5ce79cc949a54c77e1c6948ebe06f40aaa435:
[CI 39416549983](https://github.com/UraganoAngelina/lintra/actions/runs/39416549983),
Linux 118606873792 e Windows 118606874264 completed/success. Log Windows
verificato: 181 test PASS, inclusa suite golden di dieci test.
[Spiegazione completa](docs/verification/T-P5-07.md).

T-P5-06: fmt/check/clippy/test/release/diff-check PASS; 170 test locali.
Cross-check GNU all-targets PASS, nessun test Windows locale rivendicato.
Prerequisito T-P5-04 verificato via API: run 39391897709, commit a8049568,
Linux 118525169500 e Windows 118525169146 completed/success.
QG-2 PASS sul commit 0c487689e5173e06db9cd4ed679519f9d5b0746a:
[CI 39405353679](https://github.com/UraganoAngelina/lintra/actions/runs/39405353679),
Linux 118569837260 e Windows 118569837612 completed/success.
Log Windows verificato: 171 test verdi, incluso lock del definitivo.
[Spiegazione e verifiche](docs/verification/T-P5-06.md).

T-P5-05: fmt/check/clippy/test/release/diff-check PASS.
31 unit + 114 integration + 10 doctest = 155. Sedici integration e
due unit nuovi coprono i dodici casi richiesti e i boundary tecnici.
Check all-targets Windows GNU PASS; non equivale a test MSVC nativi.
[Report](docs/verification/T-P5-05.md); [CI 39399359949](https://github.com/UraganoAngelina/lintra/actions/runs/39399359949)
completed/success Linux/Windows su 85f1ac7. Task DONE; stop allora richiesto dall’autore, rispettato prima della
successiva autorizzazione T-P5-06.


T-P5-04: fmt/check/clippy/test/release/diff-check PASS;
29 unit + 98 integration + 10 doctest = 137.
[Report](docs/verification/T-P5-04.md); CI 39391897709 verde Linux/Windows
su a804956. Criterio byte per byte JSONL soddisfatto; QG-4 globale resta
legato al futuro harness golden end-to-end T-P5-07.
T-P5-03: CI 39390829911 completed/success Linux/Windows su 0f7b2df.


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

- **CNF-RECORD-01 APPROVATA / T-P6-08:** ratifica dell'autore con precisazioni:
  validate_local, risultato solo identity/valid locale, errori engine con payload
  reali, projection invariata fuori scope e non bloccante, fail-closed sui
  requisiti locali non supportati. Nessun import application/placeholder.
  Contratto Full P13, reference data P11 e residuo T-P2-03/M1 restano aperti.
  Riordino P7 approvato solo come piano: **stop dopo T-P6-08**, non autorizzata
  esecuzione di task successive. Fonti architetturali non modificate.
- **T-P6-07 / DET §35–37/147:** encoding esplicito e Result tecnico per la
  precondizione, soli indici compilati; assenza da missing policy raw,
  diagnostica dal renderer condiviso senza trim/validazione normativa.
  Destinazioni identity univoche in compilazione, nessuna sovrascrittura.
  Dettagli e limiti della rappresentazione diagnostica nel report.
- **T-P6-05 / DET §16/46/139:** confronto esatto dei byte sul target scelto
  esplicitamente dal chiamante. Compiler ora abbassa solo AllowedValues, con
  domini non vuoti/unici, lunghezza UTF-8 e coerenza literal logical; nessun
  padding/case folding inserito. Altri constraint ancora UnsupportedConstruct.
  CNF-COMPILER-01 invariata; local.v1 non dichiarata completa. Nuovi export
  di piani già esistenti; niente nuovi formati, encoding o baseline normative.


- **T-P6-04 / DET §45/§147:** validate_datatype riceve field e encoding
  esplicito, indispensabili per il LexicalPlan. Outer Err solo tecnico,
  dato invalido come Ok(Invalid(FieldViolation)); ErrorFactory arricchisce.
  Numeric ASCII non vuoto, AN encoded_text + ByteSet senza dominio implicito.
  Helper encoding riusa la tabella CP1252 esistente. Alphabetic non ammesso
  dal formato 1; Date resta T-P9-02. Nessun ampliamento del compiler UTF-8-only.


- **CNF-ALIGNMENT-01 — ARCHIVIATA per mandato finale:** solo logical per
  formato 1, nessuna verifica fisica e nessun residuo. Autorizzati prompt e
  DET §44; aggiornati anche i richiami authoring/lowering per coerenza con
  la decisione espressa. Nessun nuovo schema/parametro/default. La proposta
  fisica non è implementata; riconsiderazione futura richiede nuova decisione.
- **Audit fonti T-P6-03:** modificati DET §44 nella copia repository e nel kit,
  più prompt T-P6-03 kit; copia prompt e SHA-256 prima/dopo versionati. Manifest
  originale non modificato, hash iniziali riscontrati. ARCH e Master Plan intatti.


- **CNF-ALIGNMENT-01 / scelta B storica, superata dal mandato finale:** «B: definiamo prima il
  contratto fisico». Non approvata A; nessun helper parziale o avanzamento
  con residuo. Proposta completa redatta in
  docs/proposals/alignment-physical-contract.md: controllo esplicito
  opposite_edge_fill allow/forbid, default Allow proposto per compatibilità,
  all-fill separato da presenza/logical, test validi/invalidi, impatto su
  authoring/raw/compiled/compiler e compatibilità formato 1. Dettagli ancora
  DRAFT, non ratificati dalla sola scelta B. ARCH/DET/Piano/schemi/Rust invariati.


- **CNF-ALIGNMENT-01 / T-P6-03 — origine storica, ora archiviata:** Rule + raw non distinguono sempre
  fill di contenuto e padding. Esempio: LeftPad zero, raw 1000 può essere
  valore 1000 senza padding, non necessariamente valore 1 padded a destra.
  SCHEMA_AUTHORING definisce estrazione logical ma non un divieto universale
  di fill interno/al bordo opposto. Vietate euristiche e validator fittizio.
  Proposta A storica non approvata: helper logical e residuo aperto.
  Scelta B ora ricevuta: definire prima i vincoli fisici espliciti. Nessuna modifica
  a ARCH/DET/Piano/schemi e nessun codice della task ancora implementato.


- **T-P6-02 / DET §42/§147:** validate_presence omette il contesto non usato
  nella sola presenza base; nessun EvaluationContext fittizio o evaluator
  anticipato. Condizioni compiled non vuote restituiscono Unknown, mai base
  applicata ignorandole. MissingValuePolicy/FieldViolation/ErrorFactory riusati;
  Required mancante produce solo RequiredField, Optional mancante MissingOptional.
  Nuovi export espliciti; nessun documento architetturale o compiler cambiato.
- **Mandato operativo corrente:** ripresa progressione autonoma, task per task
  dopo verifica; stop solo su dipendenze/contratti o gate realmente aperti.


- **T-P5-07 / DET §111, ARCH §34.4:** harness solo nei test; produttore
  sintetico sostituibile, compilazione schema esterno e controlli di sola
  lunghezza senza anticipare RecordValidator. Nessun oracolo normativo SIOC/CAD.
  Check byte esatto senza riscritture; solo LINTRA_UPDATE_GOLDEN=1 abilita
  Update. Tempo sintetico fissato prima della serializzazione, nessuna
  normalizzazione dei report. Attesi iniziali indipendenti dall'output Rust.
  Nessuna nuova dipendenza o modifica a contratti pubblici.
  `.gitignore` consente espressamente solo gli input.txt golden sotto il
  profilo sintetico: il filtro *.txt altrimenti li escludeva dal commit.
  `.gitattributes` mantiene -text e disabilita il solo whitespace check Git
  sugli input sintetici: padding finale e CRLF devono restare byte di test.


- **CNF-AUDIT-01 — APPROVATA con precisazioni:** decisione integrale
  dell’autore ricevuta dopo 625850b. Audit tecnico generico, fonte unica,
  Complete solo con EOF e dimensione coerente, hash raw inclusi BOM/CRLF;
  Incomplete senza digest su annullamento/troncamento prima di EOF.
  RunMetadata/ValidationSummary ricevono inputs, ordine selezione preservato.
  Overflow e nomi non UTF-8 sono guasti tecnici; nessun locking/snapshot
  dei file sorgente, limite modifiche concorrenti a dimensione invariata
  documentato. Test aggiuntivi 1–12 obbligatori, nessuna revisione dei sorgenti.
- **T-P5-06 / DET §56/117/119:** sostituzione del definitivo solo dopo
  produzione riuscita non cancellata/non troncata, flush/sync/chiusura/rename;
  `.part` esclusivo, default eliminato, conservazione diagnostica esplicita.
  Nessuna rimozione preventiva del definitivo su Windows. Errori cleanup
  tipizzati con causa iniziale conservata. API di report generiche aggiunte;
  nessuna modifica a formati wire, documenti sorgente o dipendenze.
- **Regola autore 2026-10-20:** spiegazioni dettagliate del codice obbligatorie;
  registrata in AGENTS.md e applicata al report tecnico T-P5-06.
- **Mandato operativo specifico:** completare esclusivamente T-P5-05 e
  fermarsi prima della successiva, superando la progressione generale.
- **DEC-15 / T-P5-05:** sha2 =0.11.1 e chrono =0.4.45 (solo std),
  licenza MIT OR Apache-2.0; scopo/fonti/configurazione in
  docs/DEPENDENCIES.md, registrati prima di Cargo.toml.


- **CNF-AUDIT-01 / T-P5-05 — origine storica del conflitto, ora RISOLTA:** audit nome/dimensione/SHA-256
  obbligatorio (ARCH §39C.3) ma payload assente in DET §57/§101;
  annullamento prima di EOF non può produrre hash file dalla sola lettura
  eseguita (DET §119). Proposta A non approvata in
  docs/verification/T-P5-05-preflight.md, due JSON sintetici reviewabili.
  Proposta successivamente approvata con precisazioni e implementata in
  T-P5-05; nessun hash parziale presentato come completo o nuova scansione
  dopo annullamento. Lo stop finale è ora richiesto dall’autore.


- **T-P5-04 / ARCH §25, DET §34/§52–55:** DTO wire con alias dell’esempio
  ARCH e payload completi di diagnostica/context DET, evitando perdita di
  raw_hex o collisioni fra trace. Contratto in docs/REPORT_JSONL.md.
  JSON compatto in prestito, LF esplicito, mappe ordinate, summary separato.
  Guasti serde_json conservati in ReportError::Serialize; sink terminale.


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

HANDOFF — 2026-10-21 — T-P7-04 DONE LocalOnly / T-P7-05 BLOCKED

- **Stato:** T-P7-04 DONE LocalOnly; CNF-RUN-SCOPE-01 chiusa e verificata.
- **Fatto:** servizio reale schema/hash/preflight/trace/audit/status/sink, errori
  tecnici applicativi incrementali; modalità esplicita e timestamp metadata
  preservato, Clock iniettato senza dipendenze temporali globali nei test.
- **File:** service/request/clock/mod, domain/summary/builder/mod, report summary,
  lib, metadata test, otto JSON tecnici, nuova suite servizio, AGENTS, guida,
  ratifica/report, preflight API e stato. Nessuna fonte/baseline normativa alterata.
- **Evidenze:** 280 locali; fmt/check/clippy/release/GNU/diff PASS; CI 39587379476
  dcb8c59 Linux 119179544895 / Windows 119179544684 completed/success.
- **Contratti:** Mode neutra con wire local_only/full, Full ancora rifiutato;
  write_run_summary usa timestamp reale, writer precedente compatibile.
  Source dcb8c59 include il Clock richiesto da DET §166. Nessuna capability storica.
- **Rinvii:** T-P2-03/M1 errori completi, P11 reference data, P13/Full,
  PERSIST repository e storage; nessun placeholder o aggiornamento persistente.
- **Blocco:** CNF-SERVICE-DI-01, costruttori 7-05 richiedono porte future in ciclo;
  proposta API LocalOnly con rinvii espliciti sottoposta all'autore. Factory non scritta.
- **Git:** work/p1-foundations; dcb8c59, dossier cff635b e consuntivo afa177d
  pubblicati. Timeout documentali risolti usando il trasporto Git standard;
  chiusura dello stato Git registrata in commit documentale separato.
- **Next:** ratifica sul contratto 7-05; poi workflow autonomo con gate invariati.

HANDOFF — 2026-10-21 — T-P7-04 locale verificata / CI da eseguire

- **Stato:** DONE_UNVERIFIED. CNF-RUN-SCOPE-01 ratificata, prerequisiti CI verdi.
- **Fatto:** servizio LocalOnly con schema/hash, preflight, trace, audit, metadata,
  status e sink; cause tecniche composte in application, mai errori normativi in Err.
- **File:** service/request/domain/serializer/export, test metadata, otto JSON,
  suite servizio, AGENTS, guida, preflight/report e stato.
- **Evidenze:** 280 locali; fmt/check/clippy/release/GNU/diff PASS. Ogni JSON
  tecnico coincide col precedente togliendo soltanto il nuovo validation_mode.
- **Contratti:** modalità obbligatoria, timestamp metadata conservato in summary
  per write_run_summary; API timestamp esplicita compatibile. Full rifiutato.
- **Rinvii:** T-P2-03/M1 e P11/P13 aperti; nessun provider/context placeholder.
- **Git:** work/p1-foundations; commit da creare, nessun dato sanitario/fonte alterato.
- **Next:** CI Linux/Windows, poi preflight T-P7-05 senza conferme ordinarie.

HANDOFF — 2026-10-21 — T-P7-01..03 e T-P14-03 DONE / T-P7-04 BLOCKED

- **Stato:** quattro task DONE; 7-04 BLOCKED su CNF-RUN-SCOPE-01.
- **Fatto:** request/preflight Trace LocalOnly, token e progress, loop BufRead/path
  con audit/cancellazione/streaming, canonical JSON/hash collegato al registry.
- **File:** application/request/service/cancellation/progress/mod, engine/preflight/
  trace_validator/mod, input/source, schema/canonical/registry/mod, lib,
  quattro suite e due fixture hash, AGENTS, report, dossier 7-04 e stato.
- **Evidenze:** 268 locali; fmt/check/clippy/release/GNU/diff PASS. Ogni task ha
  CI Linux/Windows verde; ultima 39574640180 da80163, Linux 119137007062 e
  Windows MSVC 119137006791 success. Oracolo indipendente Python/sha256sum.
- **Contratti:** preflight applicativo con helper engine neutro, callback trace
  senza import application, errori tecnici propri, fingerprint da un solo load.
  Hash modello normativo, binding runtime separato; niente RFC8785 dichiarato.
- **Decisioni:** ripresa autonoma ratificata; 14-03 eseguita come dipendenza audit
  del servizio con prerequisiti P3/G0 già soddisfatti. Nessuna fonte modificata.
- **Blocco:** report non distingue modalità, proposta metadata da approvare;
  nessun codice/serializer/golden 7-04 cambiato. Domanda inviata all'autore.
- **Rinvii:** T-P2-03/M1 errori completi, P11/P13; nessuna fase intera chiusa.
- **Git:** work/p1-foundations; da80163 pubblicato, chiusura documentale separata;
  working tree inizialmente pulito su b464bec, nessuna modifica utente alterata.
- **Next:** ratifica CNF-RUN-SCOPE-01; poi T-P7-04, senza ulteriori conferme
  ordinarie per il workflow già autorizzato. Nessuna task dipendente avviata.

HANDOFF — 2026-10-21 — T-P14-03 locale verificata / T-P7-04 preflight

- **Stato:** 14-03 DONE_UNVERIFIED; 7-03 DONE nativo; 7-04 BLOCKED su contratto.
- **File:** canonical/hash, registry/export, cinque test e due fixture indipendenti,
  report 14-03, preflight 7-04 e stato.
- **Evidenze:** 268 locali; fmt/check/clippy/release/GNU/diff PASS; oracolo Python
  e sha256sum verificati. CI 14-03 ancora da eseguire.
- **Decisione operativa:** 14-03 ha prerequisiti P3/G0 soddisfatti, eseguita per
  audit reale del futuro servizio secondo ordine funzionale autorizzato.
- **Blocco:** CNF-RUN-SCOPE-01, summary senza modalità; proposta metadata neutri
  da ratificare. Nessun campo JSON/golden del report cambiato per 7-04.
- **Git:** work/p1-foundations, commit da creare; nessuna dipendenza/fonte nuova.
- **Next:** CI 14-03; attesa ratifica sul contratto prima di implementare 7-04.

HANDOFF — 2026-10-21 — T-P7-03 locale verificata

- **Stato:** DONE_UNVERIFIED, CI nativa da eseguire. Prerequisiti verificati.
- **File:** trace_validator/export, nove test, report e stato.
- **Evidenze:** 263 locali; fmt/check/clippy/release/GNU/diff PASS; 100k record
  generati e sink O(1), cancellazione reale, audit EOF e guasti Input/Report.
- **Contratti:** solo LocalOnly; callback neutre nell'engine preservano DET §5/147;
  cause tecniche tipizzate, nessun placeholder o summary definitivo dal loop.
- **Git:** work/p1-foundations; nessuna dipendenza/fonte/baseline modificata.
- **Next:** CI Linux/Windows; preflight T-P7-04 dopo gate.

HANDOFF — 2026-10-21 — T-P7-02 locale verificata

- **Stato:** DONE_UNVERIFIED, CI da eseguire; riordino CNF-RECORD-01 ratificato.
- **File:** cancellation/progress/mod, export, tre test, report e stato.
- **Evidenze:** 254 locali; fmt/check/clippy/release/GNU/diff PASS.
- **Contratti:** DET §81–83; no thread di produzione/async/throttling implicito.
- **Git:** work/p1-foundations; parte autonoma T-P2-03 usata, M1 ancora aperta.
- **Next:** CI Linux/Windows, poi T-P7-03 se gate verde.

HANDOFF — 2026-10-21 — T-P7-01 locale verificata

- **Stato:** DONE_UNVERIFIED, CI nativa da eseguire; prerequisito P6-08 verificato.
- **File:** request/service e preflight neutro, InputSource/export, sei test,
  AGENTS, report e stato; nessuna fonte/dipendenza normativa modificata.
- **Evidenze:** 251 locali; fmt/check/clippy/release/GNU/diff PASS.
- **Decisioni:** P7 Trace/LocalOnly secondo Piano; Full/Flow/limiti non supportati
  rifiutati. Preflight pubblico applicativo con helper neutro engine (DET §147).
- **Git:** work/p1-foundations; working tree inizialmente pulito su b464bec.
- **Next:** CI Linux/Windows poi T-P7-02, secondo mandato autonomo ripristinato.

HANDOFF — 2026-10-20 — T-P6-08 DONE LocalOnly / STOP prima di P7

- **Stato:** DONE — LocalOnly scope approved by CNF-RECORD-01.
- **Fatto:** StrictSkipRecord, slicing checked, identity, field pipeline,
  errori normativi streaming, contatori e cause tecniche tipizzate. valid
  certifica solo controlli locali supportati; nessuna conformità globale.
- **File:** engine/record_validator.rs, engine/mod.rs, lib.rs,
  schema/compiled/tests.rs, tests/record_validator.rs, AGENTS.md,
  preflight ratificato, report T-P6-08 e IMPLEMENTATION_STATUS.md.
- **Evidenze:** 245 locali (39 unit, 195 integration, 11 doctest), fmt/check/
  clippy/release/GNU/diff PASS; CI 39449345726 su daa9f15, Linux 118719672526
  e Windows MSVC 118719672356 completed/success. Nessuna esecuzione Windows locale.
- **Contratti:** firma locale con LocalRecordValidationArtifacts e errore engine;
  encoding dal flow compiled. Nessun PreparedReferenceData/projection placeholder.
  RecordValidator possiede il conteggio record, da non duplicare nel futuro loop.
- **Rinvii:** P11-01/05/06 reference data; P13-01..03/06..07 proiezioni e Full;
  T-P2-03/M1 errori applicativi completi. Test futuri, dipendenze e differenze
  da DET §48 descritti in docs/verification/T-P6-08.md. Fonti invariate.
- **Problemi/blocchi:** CNF-RECORD-01 risolta; nessun blocco della consegna locale.
  Nessuna nuova dipendenza, baseline normativa o dataset modificato.
- **Git:** work/p1-foundations, daa9f15 pubblicato; chiusura documentale separata.
  Working tree non pulito solo per cancellazione storica preesistente esclusa.
- **Next pianificabile:** T-P7-01 dopo T-P6-08; T-P7-02 sulla parte autonoma
  T-P2-03 verificata, secondo riordino ratificato. T-P7-03 richiede entrambe
  e T-P4-01. Nessuna task P7 o altra implementata: **attendere autorizzazione**.

HANDOFF — 2026-10-20 — T-P6-08 LocalOnly locale verificata / CI da eseguire

- **Stato:** DONE_UNVERIFIED, esclusivamente scope CNF-RECORD-01 ratificato.
- **Fatto:** record StrictSkipRecord, slicing checked, identity, field pipeline,
  contatori e propagazione tecnica; nessuna proiezione/reference data simulata.
- **File:** record_validator, export, test integration/compiled, AGENTS,
  preflight ratificato, report dettagliato e stato.
- **Evidenze:** 245 locali; fmt/check/clippy/release/GNU/diff PASS. CI da eseguire.
- **Contratti:** validate_local con LocalRecordValidationArtifacts e errore
  RecordValidationError engine; detailed design completo resta consegna futura.
- **Rinvii:** P11-01/05/06, P13-01..03/06..07, T-P2-03/M1; test futuri e
  differenze firme in docs/verification/T-P6-08.md. Nessuna fonte modificata.
- **Problemi:** nessun blocco locale; due errori iniziali delle fixture corretti
  e verifiche rieseguite. Gate nativo ancora necessario.
- **Git:** work/p1-foundations; commit da creare; cancellazione storica esclusa.
- **Next:** CI Linux/Windows e handoff finale, poi STOP obbligatorio.

HANDOFF — 2026-10-20 — T-P6-07 DONE / T-P6-08 BLOCKED

- **Stato:** T-P6-03..07 DONE; prossima T-P6-08 BLOCKED su CNF-RECORD-01.
- **Fatto:** T-P6-07 estrae identity dai binding compilati, con encoding
  esplicito e missing policy raw; nessuna validazione normativa degli ID.
- **File:** estrattore/export/compiler, due suite, report 07, preflight 08 e stato.
- **Evidenze:** 233 locali, fmt/check/clippy/release/GNU/diff PASS;
  CI 39439627879 c51c019 Linux 118685756716 / Windows 118685756618 success.
- **Contratti:** encoding/Result documentati DET §147; collisioni identity
  rifiutate dal compiler. Nessun source document o schema normativo cambiato.
- **Blocco:** ciclo con PreparedReferenceData/ProjectedRecord/errori P11/P13,
  confine engine/application; proposta A locale esplicita sottoposta all'autore.
- **Rinviati:** nessun codice T-P6-08/P7; T-P2-03/M1 ancora aperte.
- **Git:** work/p1-foundations; c51c019 pubblicato, chiusura documentale separata;
  cancellazione preesistente del piano storico esclusa, working tree non pulito.
- **Next:** ratifica CNF-RECORD-01, poi T-P6-08 con gate propri.

HANDOFF — 2026-10-20 — T-P6-07 locale verificata / CI da eseguire

- **Stato:** DONE_UNVERIFIED; prerequisiti 3-02/4-03 CI verificati.
- **File:** estrattore, export, compiler, integration/regression, report e stato.
- **Evidenze:** 233 locali; fmt/check/clippy/release/GNU/diff PASS.
- **Contratti:** DET §147 encoding esplicito/Result; nessuna norma nuova.
- **Git:** commit da creare, cancellazione storica esclusa.
- **Next:** CI nativa; preflight T-P6-08 dopo gate.

HANDOFF — 2026-10-20 — T-P6-06 DONE / avvio T-P6-07

- **Stato:** DONE; commit 65f1eb3, CI 39437225233 Linux/Windows success.
- **File:** pipeline, export, test interni/integrazione, report e stato.
- **Evidenze:** 226 locali; fmt/check/clippy/release/GNU/diff PASS.
- **Contratti:** esiti DET §41, errori tecnici tipizzati, stop primo errore.
- **Limiti:** DSL P10 ancora esclusa; nessun residuo fisico formato 1.
- **Git:** sorgenti pubblicati, chiusura documentale separata; cancellazione
  storica preesistente esclusa. Nessuna nuova dipendenza o norma.
- **Next:** T-P6-07 IdentityExtractor; prerequisiti 3-02/4-03 CI verificati.

HANDOFF — 2026-10-20 — T-P6-06 locale verificata / CI da eseguire

- **Stato:** DONE_UNVERIFIED; prerequisiti T-P6-02..05 CI nativa verde.
- **File:** pipeline e export, due suite, report e stato; nessuna fonte cambiata.
- **Evidenze:** 226 test locali; fmt/check/clippy/release/GNU/diff PASS.
- **Decisioni:** DET §40–41 e Piano §7; nessun E_ALIGNMENT o evaluator P10.
- **Git:** commit implementativo da creare; cancellazione storica esclusa.
- **Next:** CI Linux/Windows; poi T-P6-07 se gate verde.

HANDOFF — 2026-10-20 — T-P6-05 DONE / avvio T-P6-06

- **Stato:** DONE, nessun blocco o residuo fisico.
- **File:** domain.rs, validators/mod.rs, lib.rs, compiler.rs,
  allowed_values.rs integration, report e stato.
- **Evidenze:** 215 locali; fmt/check/clippy/test/release/GNU/diff PASS;
  CI 39435239667 su c3caa5e, Linux 118670448108 e Windows 118670447892 success.
- **Decisioni:** valori su byte esatti, target esplicito; compiler conserva
  valori/ordine e rifiuta configurazioni incompatibili e nodi non supportati.
  local.v1 ancora parziale; nessuna norma o baseline nuova.
- **Git:** c3caa5e pubblicato; chiusura documentale separata;
  cancellazione storica preesistente esclusa.
- **Next:** T-P6-06 Field pipeline, dopo prerequisiti reali verificati.

HANDOFF — 2026-10-20 — T-P6-05 locale verificata / CI da eseguire

- **Stato:** DONE_UNVERIFIED fino al gate nativo.
- **File:** validators/domain.rs e mod.rs, lib.rs, schema/compiler.rs,
  tests/allowed_values.rs, report tecnico e questo stato.
- **Comportamento:** membership puro; il compiler valida e conserva dominio,
  target e byte. Literal invalidi sono issue di schema, dato fuori dominio
  FieldViolation/InvalidDomain. Nessuna normalizzazione o padding fisico.
- **Spiegazioni:** tutte le funzioni/condizioni e test descritti nel report.
- **Evidenze:** 215 test locali; fmt/check/clippy/release/GNU/diff PASS;
  dieci test nuovi e regressioni compiler ancora verdi. CI da eseguire.
- **Decisioni:** estensione del sottoinsieme compiler già autorizzato,
  nessuna capability completa, norma, campo o dipendenza nuova.
- **Git:** work/p1-foundations; cancellazione preesistente esclusa.
- **Next:** CI nativa, poi T-P6-06 Field pipeline secondo dipendenze reali.

HANDOFF — 2026-10-20 — T-P6-04 DONE / avvio autonomo T-P6-05

- **Stato:** DONE; Numeric/AN e profili espliciti verificati.
- **File:** datatype.rs, input/encoding.rs, validators/mod.rs, lib.rs,
  compiled/tests.rs, datatype.rs integration, report e stato.
- **Evidenze:** 205 locali; fmt/check/clippy/release/GNU/diff PASS;
  CI 39430943109 su 24b0e02: Linux 118655543578 e Windows 118655542644 success.
- **Decisioni:** enum esiti normativi distinta da errori tecnici; classe
  lessicale esplicita e nessun dominio AN o trim. Nessuna dichiarazione di
  compiler completo o di binding aggiuntivi. Spiegazioni in T-P6-04.md.
- **Git:** 24b0e02 pubblicato, chiusura documentale separata;
  cancellazione storica preesistente esclusa.
- **Next:** T-P6-05 AllowedValues, preflight e implementazione se contratti chiari.

HANDOFF — 2026-10-20 — T-P6-04 locale verificata / CI da eseguire

- **Stato:** DONE_UNVERIFIED fino al gate nativo Linux/Windows.
- **File:** datatype.rs, validators/mod.rs, input/encoding.rs, lib.rs,
  compiled/tests.rs, tests/datatype.rs, report tecnico e questo stato.
- **Cosa fa e perché:** encoding esplicito verificato senza replacement,
  Numeric ASCII non vuoto e ByteSet aggiuntivo; nessun parsing/domain/trim.
  Ok Invalid(FieldViolation) resta distinto da Err tecnico con configurazione.
  Fonte CP1252 condivisa col renderer; spiegazioni dettagliate nel report.
- **Evidenze:** 205 test locali e fmt/check/clippy/release/GNU/diff PASS;
  undici integration e un unit nuovi, anche diagnostica ErrorFactory.
- **Limiti:** Date T-P9-02, Alphabetic escluso formato 1; compiler invariato
  nel binding UTF-8-only. Nessun residuo fisico, schema o dipendenza nuova.
- **Git:** work/p1-foundations; cancellazione storica preesistente esclusa.
- **Next:** ottenere CI nativa, poi T-P6-05 se prerequisiti/contratti chiari.

HANDOFF — 2026-10-20 — T-P6-03 DONE / nessun residuo / avvio T-P6-04

- **Stato:** DONE. Il controllo fisico non è un residuo: fuori ambito formato 1
  per decisione finale dell'autore; CNF archiviata come estensione futura.
- **File:** alignment.rs, validators/mod.rs, lib.rs, logical_value.rs;
  DET §44 repo/kit, prompt kit/copia, hash audit, authoring/lowering, AGENTS,
  preflight/proposta archiviati, report di verifica e stato.
- **Evidenze:** 193 locali/194 Windows, fmt/check/clippy/release/GNU/diff PASS.
  CI 39425268285 su bd49791: Linux 118636134483, Windows 118636134182 success.
  Hash autorizzati e copie source verificate senza alterare il manifest.
- **Decisioni:** estrazione zero-copy, errori soltanto tecnici per piani
  incoerenti; nessun E_ALIGNMENT o parametro fisico. Spiegazioni nel report.
- **Git:** bd49791 pubblicato; chiusura documentale separata. Cancellazione
  preesistente del piano storico esclusa. Fonti kit esterne al Git tracciate.
- **Next:** T-P6-04 avviata autonomamente come autorizzato; nessun blocco fisico.

HANDOFF — 2026-10-20 — T-P6-03 logical / locale verificato / CI da eseguire

- **Stato:** DONE_UNVERIFIED fino alla CI nativa. Nessun residuo fisico:
  controllo fuori ambito formato 1 per decisione espressa dell'autore.
- **File codice:** validators/alignment.rs, validators/mod.rs, lib.rs,
  tests/logical_value.rs. Spiegazioni dettagliate nel codice e report tecnico.
- **Documenti:** AGENTS, DET §44 repo/kit, prompt kit e copia versionata,
  authoring/lowering, preflight risolto/proposta fisica archiviata, audit hash,
  report T-P6-03 e stato. Nessuna baseline normativa modificata.
- **Evidenze:** 193 test locali, fmt/clippy/release/GNU PASS, sette test nuovi;
  audit fonti prima/dopo e confronto byte copie PASS. CI da eseguire.
- **Decisioni:** input prestato senza allocazione; strip soltanto lato
  dichiarato, preserve/exact identici, all-fill vuoto sotto stripping.
  Errori di configurazione tecnici distinti; nessun FieldViolation/E_ALIGNMENT.
- **Git:** work/p1-foundations; cancellazione preesistente del piano storico
  esclusa. Fonte kit fuori Git coperta da copia prompt e hash autorizzati.
- **Next:** verificare CI, poi T-P6-04 come autorizzato; nessun residuo fisico.

HANDOFF — 2026-10-20 — T-P6-03 / scelta B registrata / contratto DRAFT

- **Stato:** T-P6-03 BLOCKED fino a ratifica del contratto fisico;
  T-P6-02 resta DONE con evidenze nel proprio report.
- **Mandato:** autore ha scelto B, definizione prima del codice; A non
  autorizzata, nessuna consegna parziale o task successiva avviata.
- **File:** AGENTS.md, dossier preflight T-P6-03, nuova proposta completa
  docs/proposals/alignment-physical-contract.md e questo registro.
- **Proposta:** vincolo esplicito sul fill al bordo opposto, indipendente
  dal logical; default compatibile Allow da approvare; tutti-fill fisicamente
  ammessi con presenza separata; zeri/spazi interni non vietati per euristica.
  Tabella completa di casi, compatibilità additiva formato 1 e cambiamenti
  authoring/API elencati per decisione concreta. Nessuna regola assegnata
  automaticamente ai campi regionali.
- **Evidenze:** verificati documenti, parser con deny_unknown_fields, controllo
  UTF-8 del fill nel compiler e modelli compiled. Attività solo documentale:
  nessun Cargo/CI nuovo rivendicato. git diff --check eseguito prima del commit.
- **Decisione richiesta:** approvare il contratto completo proposto oppure
  indicare i punti da rivedere; la scelta B non ratifica dettagli nuovi.
- **Git:** work/p1-foundations; modifica preesistente al piano storico esclusa;
  nessuna modifica a Rust, baseline, schemi o fonti architetturali approvate.
- **Next:** dopo ratifica, implementare T-P6-03 completa e verificare prima
  di riprendere la progressione autonoma generale.

HANDOFF — 2026-10-20 — T-P6-03 preflight BLOCKED / T-P6-02 DONE

- **Stato:** T-P6-02 DONE; T-P6-03 avviata autonomamente e BLOCKED prima del
  codice per CNF-ALIGNMENT-01, senza dichiarare verifiche di padding inesistenti.
- **File task corrente:** docs/verification/T-P6-03-preflight.md e questo stato.
  Recap T-P6-02 nel relativo handoff e report tecnico.
- **Evidenze:** prerequisito T-P6-01 CI Linux/Windows riconfermato; DET §15/44,
  SCHEMA_AUTHORING e grammatica analizzati. Nessun Cargo aggiuntivo rivendicato.
- **Problema:** lo stesso raw può rappresentare contenuto legittimo o padding
  sul lato sbagliato; la funzione non riceve informazione sufficiente a decidere.
- **Decisione richiesta:** A, estrazione logical autonoma con residuo fisico
  esplicito e prosecuzione atomica, oppure B, completare prima il contratto.
- **Git:** work/p1-foundations; documentazione del blocco separata da codice;
  modifica preesistente al piano storico preservata ed esclusa.
- **Next:** attendere decisione; nessun validator euristico, schema o task
  successiva avviati per aggirare il blocco.

HANDOFF — 2026-10-20 — T-P6-02 DONE / progressione autonoma ripristinata

- **Stato:** DONE; nessun blocco residuo nella task.
- **File:** AGENTS.md; validators/presence.rs; lib.rs; compiled/tests.rs;
  tests/presence.rs; report tecnico T-P6-02 e questo stato.
- **Comportamento:** presenza base sul raw con MissingValuePolicy; Required
  assente -> FieldViolation RequiredField, Optional assente -> MissingOptional,
  valore presente -> Present. Condizioni compiled non valutate -> Unknown.
- **Spiegazioni:** enum, funzione, condizioni e test descritti nel codice e
  in docs/verification/T-P6-02.md; adattamento DET §147 dichiarato.
- **Evidenze:** 186 locali e 187 Windows; fmt/check/clippy/release/GNU PASS.
  CI 39418788154 su c34649b, job Linux 118614279561 e Windows 118614279377
  completed/success. Payload ErrorFactory e boundary condizioni verificati.
- **Decisioni/limiti:** nessun evaluator o pipeline P10 anticipato; niente
  dipendenze/norme nuove. Nuovo mandato supera gli stop ordinari e storici.
- **Git:** c34649b sul branch work/p1-foundations; chiusura documentale
  separata; modifica preesistente al piano storico esclusa.
- **Next:** avvio autonomo T-P6-03, verificando prima il contratto padding.

HANDOFF — 2026-10-20 — T-P6-02 locale verificata / CI da eseguire

- **Stato:** DONE_UNVERIFIED fino al gate nativo Linux/Windows.
- **File:** AGENTS.md, validators/presence.rs, lib.rs, compiled/tests.rs,
  tests/presence.rs, report tecnico e questo stato.
- **Comportamento:** Required/Optional sul raw, policy missing riusata;
  condizioni non valutate producono Unknown. Nessuna cascata di validatori.
- **Evidenze:** 186 test locali, fmt/check/clippy/release/GNU-all-targets PASS;
  integrazione ErrorFactory e boundary condizioni coperti. Spiegazioni di ogni
  struttura/funzione/condizione nel codice e nel report.
- **Decisioni:** adattamento DET §147 documentato; nessuna dipendenza nuova,
  né logica normativa. Nuovo mandato autonomo registrato in AGENTS.md.
- **Git:** work/p1-foundations, piano storico preesistente escluso.
- **Next:** chiudere gate e procedere a preflight T-P6-03 se contratti chiari.

HANDOFF — 2026-10-20 — T-P5-07 DONE / CI nativa verde

- **Stato:** DONE; nessun blocco residuo nella task.
- **File:** tests/golden.rs, tests/support/{mod,golden,golden_tests,synthetic}.rs;
  README, schema e diciotto file scenario sotto tests/golden/; .gitignore,
  .gitattributes; report di verifica e questo stato. Nessun Rust di produzione.
- **Cosa fa e perché funziona:** runner con scoperta ricorsiva e confronto dei
  byte originali di entrambi i report; modalità Check priva di scritture,
  Update soltanto con variabile esatta. Produttore di prova con componenti
  reali già consegnati e tempi fissati prima della serializzazione. Ogni
  struttura/funzione/impl/condizione spiegata nel codice e nel report tecnico.
- **Evidenze:** 180 test locali e fmt/check/clippy/release/GNU-all-targets PASS;
  CI 39416549983 su b0e5ce7: Linux 118606873792 e Windows 118606874264
  completed/success. Log Windows: 181 test PASS, suite golden inclusa.
  Opt-in effettivo verificato senza cambiare gli attesi indipendenti;
  tutti i 19 file schema/scenario nel commit confrontati ai byte sorgente.
- **Problemi risolti:** *.txt escludeva input sintetici: eccezione limitata
  in .gitignore. Diff-check segnalava padding e CRLF intenzionali: attributo
  -whitespace solo per gli input sintetici, mantenendo -text e confronto esatto.
- **Decisioni/limiti:** adattatore sintetico di sola lunghezza, non validazione
  normativa completa. Nessuna dipendenza o API core nuova. Update fixture
  per documento, non transazione globale; revisione del diff necessaria.
- **Git:** b0e5ce7 pubblicato su work/p1-foundations; chiusura documentale
  separata. Modifica preesistente al piano storico preservata ed esclusa.
- **Next:** T-P6-02 — Presenza di base prevista, non avviata in questa consegna.
  Residuo applicativo T-P2-03/M1 invariato.

HANDOFF — 2026-10-20 — T-P5-07 locale verificata / CI da eseguire

- **Stato:** DONE_UNVERIFIED fino alla CI nativa Linux/Windows.
- **File:** tests/golden.rs, tests/support/{mod,golden,golden_tests,synthetic}.rs,
  README/schema/sei scenari golden, report tecnico e questo registro.
- **Comportamento:** scoperta ricorsiva, confronto integrale byte per byte,
  aggiornamento solo con variabile esplicita; errori e fixture mancanti non
  diventano pass. Produttore sintetico collega componenti reali già consegnati.
- **Evidenze:** 180 test locali e fmt/check/clippy/release/GNU PASS;
  update via ambiente PASS senza modificare i byte indipendentemente attesi;
  gitattributes -text verificato. Spiegazioni di struct/funzioni/impl/condizioni
  nei commenti e nel report di verifica.
- **Decisioni/limiti:** tempo di test fisso prima della serializzazione;
  nessuna normalizzazione dei byte, semantica normativa o runtime anticipato.
- **Git:** branch work/p1-foundations; modifica preesistente al piano storico
  esclusa. QG-2 da ottenere sul nuovo commit.
- **Next:** chiudere QG-2; T-P6-02 — Presenza di base prevista, non avviata.

HANDOFF — 2026-10-20 — T-P5-06 DONE / QG-2 verde

- **Stato:** DONE; nessun blocco residuo nella task.
- **File:** AGENTS.md (regola vincolante), src/report/atomic.rs (guard e
  pubblicazione), src/report/mod.rs, src/report/sink.rs (errori tecnici),
  src/lib.rs (export), tests/atomic_report.rs, report tecnico e questo stato.
- **Cosa fa e perché funziona:** apertura esclusiva del `.part`, scrittura
  streaming, controllo stato/troncamento, flush/sync, chiusura e rename;
  il definitivo cambia solo al successo. Abort e cleanup conservano la
  classificazione degli errori e gestiscono esplicitamente la policy dei
  parziali. Struct/funzioni/impl/condizioni spiegate in rustdoc e nel report.
- **Evidenze:** 170 test locali, fmt/check/clippy/release/diff-check e GNU
  all-targets PASS. CI 39405353679 su 0c48768: Linux 118569837260 e Windows
  MSVC 118569837612 completed/success. Log Windows: 171 test PASS, incluso
  definitivo bloccato senza delete sharing.
- **Limiti:** atomicità per file, nessuna transazione multireport o garanzia
  crash-durable della directory. Drop best effort; abort esplicito espone
  errori cleanup. Nessun contratto applicativo o baseline anticipato.
- **Git:** implementazione 0c48768 pubblicata sul branch work/p1-foundations;
  chiusura documentale separata. Piano Superpowers preesistente escluso.
- **Next:** T-P5-07 — Harness golden prevista, non avviata. T-P2-03/M1
  mantengono aperto il residuo applicativo già registrato.

HANDOFF — 2026-10-20 — T-P5-06 locale verificata / CI da eseguire

- **Stato:** DONE_UNVERIFIED fino a QG-2 nativa Linux/Windows.
- **File:** AGENTS.md, report/atomic.rs, report/mod.rs, report/sink.rs,
  lib.rs, tests/atomic_report.rs, report di verifica e questo stato.
- **Comportamento:** `.part` esclusivo; pubblicazione solo per conclusione
  riuscita e non troncata; errore/cancellazione eliminano o conservano
  esplicitamente il parziale. Definitivo precedente preservato fino al rename.
- **Evidenze:** fmt/check/clippy/test/release/diff-check e cross-check GNU PASS;
  170 test Linux locali. Test lock destinazione Windows compilato, non ancora
  eseguito nativamente. Nuove spiegazioni dettagliate nel report tecnico.
- **Decisioni:** nessuna dipendenza o norma nuova; guard per mantenere tipizzati
  gli errori upstream; atomicità per file, nessuna transazione multireport.
- **Git:** branch work/p1-foundations; piano storico preesistente escluso.
- **Next:** ottenere QG-2; poi T-P5-07 prevista, non avviata.

HANDOFF — 2026-10-20 — T-P5-05 DONE / STOP richiesto

- **Consegna:** audit generico autorevole, SHA-256 raw/EOF, builder
  checked e summary deterministico separato con timestamp UTC.
- **Evidenze:** 155 test locali, fmt/check/clippy/release/diff-check verdi;
  Windows GNU all-targets check verde; CI 39399359949 su 85f1ac7,
  Linux 118549909807 e Windows MSVC 118549910365 completed/success.
- **File:** quattro moduli nuovi domain/input_audit, summary_builder,
  input/audit, report/summary; metadata/errori/export e Cargo/lock;
  test/fixture/documenti approvazione/DEC-15/AGENTS. Elenco nel report.
- **Decisioni:** CNF-AUDIT-01 e tutte le precisazioni dell’autore applicate;
  sole copie congelate da fonte unica, nessun hash parziale o drain dopo stop.
- **Limiti:** niente lock/snapshot contro modifiche di pari dimensione,
  nessun service o pubblicazione atomica anticipato. T-P2-03/M1 aperte.
- **Git:** codice e consuntivo committati sul branch di lavoro; piano
  storico preesistente preservato ed escluso. Nessun merge o force.
- **Next:** T-P5-06 NON INIZIATA; attendere autorizzazione dell’autore.
  Arresto operativo richiesto al termine di T-P5-05 rispettato.


HANDOFF — 2026-10-20 — T-P5-05 locale verificata / CI da completare

- **Consegna:** audit tecnico generico con sessione autorevole e hashing
  raw incrementale; Complete solo a EOF coerente; builder e JSON summary.
- **File:** quattro moduli nuovi domain/input_audit, summary_builder,
  input/audit, report/summary; metadata/errori/moduli/export; Cargo/lock;
  test e fixture; DEC-15/documentazione/AGENTS e registro approvazione.
- **Test:** 155 verdi, fmt/check/clippy/release/diff-check; Windows GNU
  all-targets check verde, dipendenze verificate per Linux/MSVC.
- **Gate:** CI nativa da avviare; nessun successo Windows dichiarato prima.
- **Limiti:** niente lock/snapshot sorgente contro modifiche di stessa
  dimensione, niente service o pubblicazione atomica dei file anticipati.
- **Git:** escluso il piano storico già modificato dall’autore.
- **Next:** completare CI e fermarsi, attendere autorizzazione prima
  di T-P5-06. T-P2-03/M1 ancora aperte, nessuna baseline modificata.


HANDOFF — 2026-10-20 — T-P5-04 completata / T-P5-05 bloccata

- **DONE:** T-P4-02..05, T-P6-01 e T-P5-01..04, con evidenze CI
  per ogni task nei rispettivi report. Ultima CI 39391897709 su a804956,
  Windows 118525169146 e Linux 118525169500 completed/success, 137 test.
- **Ultimi file:** protocollo JSONL, sink/wire DTO, fixture e test;
  dossier T-P5-05 e due JSON sintetici per audit completo/incompleto.
- **Blocco reale:** CNF-AUDIT-01, estensione audit dei tipi neutri e
  assenza esplicita dell’hash file prima di EOF nei run interrotti.
- **Decisione richiesta all’autore:** proposta A concreta in
  docs/verification/T-P5-05-preflight.md, domanda già presentata.
- **Limiti:** nessun summary builder/audit approvato o implementato;
  niente pubblicazione atomica dei report, service o residuo T-P2-03.
- **Git:** codice e dossier committati sul branch di lavoro; file di piano
  storico dell’autore preservato/escluso. Nessun merge/main/force.
- **Next:** T-P5-05 dopo decisione; task indipendenti non invalidate.


HANDOFF — 2026-10-20 — preflight T-P5-05 / CNF-AUDIT-01

- **Blocco:** contratto audit su input non interamente letto; serve scelta
  dell’autore prima di estendere RunMetadata/ValidationSummary.
- **Proposta reviewabile:** dossier T-P5-05-preflight.md e due JSON
  docs/proposals/input-audit/, sintesi/hash verificati senza codice Rust.
- **Stato:** T-P5-04 locale verificata/CI in corso; T-P5-05 BLOCKED.
- **Decisione richiesta:** approvare proposta A oppure fornire policy
  alternativa coerente con hash dell’intero input e cancellazione.
- **Indipendenti:** T-P5-06 e rami con prerequisiti verdi non invalidati;
  il blocco non è una richiesta di autorizzazione ordinaria all’avanzamento.


HANDOFF — 2026-10-20 — T-P5-04 locale verificata

- **File:** report/jsonl.rs, report/sink.rs, moduli/export, tests/jsonl_sink.rs,
  fixture JSONL sintetica, docs/REPORT_JSONL.md, report/stato.
- **Evidenza:** 137 test, fmt/check/clippy/release/diff-check verdi locali.
- **Gate:** T-P5-03 DONE in CI 39390829911; nuova CI JSONL da eseguire.
- **Next:** CI QG-2/QG-4, preflight T-P5-05 su audit/summary.
- **Residui:** pubblicazione atomica .part in T-P5-06, niente report parziale
  presentato come definitivo; T-P2-03/M1 ancora aperte.
- **Git:** file storico dell’autore escluso dal commit.


HANDOFF — 2026-10-20 — T-P5-03 locale verificata

- **File:** report/sink.rs, report/composite.rs, report/mod.rs, lib.rs,
  tests/validation_sinks.rs, report/stato.
- **Evidenza:** 132 test, fmt/check/clippy/release/diff-check verdi locali.
- **Gate:** CI T-P5-03 [39390829911](https://github.com/UraganoAngelina/lintra/actions/runs/39390829911) avviata su 0f7b2df; factory già verde.
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


HANDOFF — 2026-10-21 — T-P7-05 gate locali
- **Fase/task:** P7 / T-P7-05; DONE_UNVERIFIED fino alla CI nativa.
- **Fatto:** factory LocalOnly in service.rs e unico nuovo re-export in lib.rs;
  cinque test consumer reali e due observer nel supporto test; API audit e
  spiegazioni dettagliate in docs/verification/T-P7-05.md.
- **Evidenze:** fmt/check/Clippy -D warnings/test/release/diff passati; 285 test
  (39 unit, 235 integration, 11 doctest); check all-targets Windows GNU passato.
- **Decisione:** CNF-SERVICE-DI-01 ratificata; gate 7-04 ricontrollato verde.
- **Contratto:** build_default_local_service(PathBuf, RawResolvedEncoding)
  -> Result<ValidationService, SchemaLoadError>; new/with_clock preservati.
- **Rinvii:** T-P11-01/02/03, T-PERSIST-02/03, P13, factory completa;
  residuo T-P2-03/M1 aperto; nessun placeholder. Ledger nel report.
- **Problemi:** nessun nuovo conflitto; CI nativa ancora necessaria.
- **Git:** work/p1-foundations, modifiche di task da committare e pubblicare.
- **Next:** dopo gate T-P7-06 raccomandata (T-P5-07 DONE), ma STOP obbligatorio
  e nuova autorizzazione per ogni ulteriore task. Nessun avvio automatico.


HANDOFF — 2026-10-21 — T-P7-05 conclusa / STOP
- **Fase/task:** P7 / T-P7-05.
- **Stato:** DONE — LocalOnly API scope approved by CNF-SERVICE-DI-01.
- **Fatto/file:** factory esplicita in src/application/service.rs, export in
  src/lib.rs; cinque test esterni in tests/local_service_factory.rs e observer
  tests/support/progress.rs; ratifica in AGENTS.md, preflight aggiornato,
  spiegazioni del codice/audit export/ledger in docs/verification/T-P7-05.md.
- **Evidenze:** 285 test locali (39 unit, 235 integration, 11 doctest); fmt,
  check all-targets, Clippy -D warnings, build release, Windows GNU check e
  diff --check superati. CI 39601419396 su 074bbfeda74bc1db197f6d4a250961c646f0ad70:
  Linux job 119226605960 e Windows MSVC job 119226606315 success, confermati
  dall’API GitHub dopo errori transitori di gh run view.
- **Test non eseguiti:** nessuna esecuzione Windows locale; test nativi
  Windows eseguiti realmente in CI. Nessun test Full/capacità future dichiarato.
- **Contratto:** build_default_local_service(PathBuf, RawResolvedEncoding)
  -> Result<ValidationService, SchemaLoadError>, nessun nuovo payload generico.
  Root/encoding espliciti, caricamento lazy, new/with_clock invariati.
- **Decisioni:** CNF-SERVICE-DI-01 approvata; compiled read-only e non-forgiabili.
  Nessuna modifica delle fonti architetturali o delle baseline normative.
- **Problemi/blocchi:** nessuno per T-P7-05 LocalOnly. Nessun Full disponibile.
- **Rinvii:** P11/T-P11-01..03; T-PERSIST-02/03; P13; factory completa;
  T-P2-03/M1 restano aperte, persistenza core obbligatoria finale.
- **Git:** branch work/p1-foundations, codice 074bbfe pubblicato; questa
  chiusura documentale in commit successivo, senza ulteriori modifiche Rust.
- **Next:** T-P7-06 (T-P7-05 e T-P5-07 DONE), T-P8-01 (DEC-03 già approvata),
  T-P11-01 sbloccate dalle dipendenze locali secondo la ratifica; nessuna iniziata.
  STOP vincolante: attendere nuova autorizzazione anche per CLI o altre task.


HANDOFF — 2026-10-21 — T-P7-06 gate locali
- Stato DONE_UNVERIFIED, prerequisiti 7-05/5-07 verificati.
- File: tests/integration.rs, tests/integration/synthetic_service.rs,
  tests/fixtures/application/schema.json; AGENTS.md, questo registro e dossier
  docs/verification/T-P7-06.md. Nessuna modifica Rust di produzione.
- Copertura: campi/padding/presence/tipi/domains attraverso servizio e JSONL;
  contatori/ordine deterministico, schema invalido da filesystem, 10.000 righe
  senza raccolta globale, guasto sorgente a metà run senza summary.
- Regressioni: 289 test, fmt/check/Clippy/release/Windows GNU check/diff passati;
  I/O/sink/cancellazione/golden preesistenti riusati e rieseguiti.
- Contratti invariati; Full/P11/P13/PERSIST/T-P2-03/M1 rimangono aperti.
- Git: work/p1-foundations, implementazione da pubblicare per CI nativa.
- Next: T-P8-01 dopo CI e verifica DEC-12/contratti di risoluzione bundle.
  Workflow autonomo ripristinato dal mandato più recente.


HANDOFF — 2026-10-21 — T-P7-06 DONE LocalOnly / avvio T-P8-01
- Task T-P7-06 DONE: 289 test locali e CI 39621523482 su b402fdf completa success.
  Linux job 119295973306, Windows MSVC 119295973105: fmt/Clippy/test/release.
- File e comportamento nel report T-P7-06; contratti produzione invariati.
- Nessun blocco della consegna locale; P11/P13/PERSIST/T-P2-03/M1 restano aperti.
- Git work/p1-foundations, codice b402fdf pubblicato, chiusura documentale qui.
- Decision log DEC-12: autore approva A del preflight T-P8-01; righe vuote
  zero byte, nessun record finale inventato, BOM Reject, nessun --bom strip.
  Nessuna revisione delle fonti ARCH/DET/Piano; scelta registrata come ratifica.
- Next T-P8-01, avvio autonomo dopo gate verificati; DEC-03 già risolta.


HANDOFF — 2026-10-21 — T-P8-01 gate locali
- DONE_UNVERIFIED fino a CI nativa. DEC-12 A ratificata, nessun nuovo conflitto.
- File: src/cli/{mod,args,exit_code,paths,tests}.rs e src/main.rs; Cargo.toml/lock,
  README.md, docs/DEPENDENCIES.md, docs/verification/T-P8-01.md, questo registro.
- Scheletro reale con help/parsing, request LocalOnly, root esplicite per coppie
  complete, errori tipizzati e mapping 0/1/2/130. Validate non esegue ancora
  alcuna analisi: rifiuto tecnico esplicito fino a T-P8-03, nessun output fittizio.
- Verifiche: 297 test, fmt/check/Clippy/release/Windows GNU check/diff passati;
  processo release help 0, nessuna opzione e --bom strip 1.
- Dipendenze: clap 4.6.7 features minime, licenza/MSRV/tree registrati, nessun
  import nel core. Nessuna baseline/fonti o API libreria modificate.
- Rinvii: esecuzione 8-03, embedded 15-03, reference P11, Full P13 e persistenza;
  T-P2-03/M1 aperte.
- Git work/p1-foundations, codice da pubblicare per CI. Next T-P8-02 dopo gate.


HANDOFF — 2026-10-21 — T-P8-01 DONE / avvio T-P8-02
- Codice 98d952b, CI 39628560176 completed/success nativa Linux/Windows.
- 297 test e tutti i gate locali passati; file e spiegazioni nel dossier 8-01.
- Contratti nuovi solo adapter, DEC-12 approvata, nessuna modifica architetturale.
- Nessun blocco 8-01; comando validate effettivo resta 8-03, embedded 15-03.
- Git work/p1-foundations, sorgente pubblicato, chiusura documentale qui.
- Next T-P8-02 avviata automaticamente; conformità testuale sempre nel modo
  selezionato, nessuna promozione da locale a Full. Rinvii P11/P13/PERSIST aperti.


HANDOFF — 2026-10-21 — T-P8-02 gate locali
- DONE_UNVERIFIED fino a CI; prerequisito 8-01 CI completed/success verificato.
- File src/report/human.rs, report/mod.rs, lib.rs, tests/human_readable_sink.rs,
  docs/verification/T-P8-02.md, questo registro. Unico nuovo export HumanReadableSink.
- Summary-only: Mode esplicito, nessun dato record stampato/trattenuto, conteggi
  E_* ordinati, scope/input/counter/elapsed, interruzioni indicate senza conformità.
- Sette test nuovi, 304 globali, fmt/check/Clippy/release/Windows GNU check/diff
  passati; errore mut superfluo corretto senza cambiare i contratti.
- Nessuna dipendenza nuova, fonti/baseline invariate.
- Git work/p1-foundations, codice da pubblicare per CI. Next T-P8-03 dopo gate,
  payload reali/LocalOnly invariati; P11/P13/PERSIST/T-P2-03/M1 restano aperti.
