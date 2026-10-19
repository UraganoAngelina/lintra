# T-P3-05 — Preflight e CNF-COMPILER-01

2026-10-19. **Proposta A APPROVATA dall’autore: CNF-COMPILER-01 risolto.**
Risposta: «Approvo A: compiler incrementale con rifiuto esplicito dei costrutti esclusi».
Il preflight sotto conserva problema e proposta sottoposta alla decisione;
la consegna effettiva è registrata nel report T-P3-05.
Nessun codice di produzione modificato, nessuna task successiva avviata.

## Prerequisiti verificati

- T-P3-02: compiled immutabile, commit f04bb60,
  [CI 39250930467](https://github.com/UraganoAngelina/lintra/actions/runs/39250930467)
  Linux/Windows verde; costruttori privati riservati al compiler discendente.
- T-P3-04: lint strutturale, commit a1ddabb,
  [CI 39257768950](https://github.com/UraganoAngelina/lintra/actions/runs/39257768950)
  Linux/Windows verde; lint P0 effettuato. Nessun problema di dipendenze.
- Stato Git: work/p1-foundations, HEAD 372c38a. La modifica preesistente a
  docs/superpowers/plans/2026-10-16-foundations.md è estranea e intatta.

Letture: prompt T-P3-05; DET §11/§13/§157; ARCH/DET relativi a compiled,
encoding e persistenza già richiamati dai modelli; Piano §4/§9/P3/§14 e
T-P3-05/06, P6/P9; contratto SCHEMA_AUTHORING approvato, formato 1 e lowering;
modello raw/compiled/linter e relativi report. Nessuna skill o subagente.

## BLOCCO — CNF-COMPILER-01

**Descrizione.** Il prompt e Piano T-P3-05 chiedono un compiler che produca
CompiledFlowSchema per il sottoinsieme minimo e rifiuti gli step non ancora
supportati, senza implementarli in anticipo. Il contratto approvato rende
local.v1 una capability più ampia di quel sottoinsieme.

**Evidenze.** SCHEMA_AUTHORING, tabella Capability, definisce local.v1 come
«Tipi fisici, padding/missing, scalar constraint e chiavi locali». Subito dopo:
«Capability sconosciuta, mancante o non implementata blocca la compilazione;
un motore non pubblicizza una capability implementata solo parzialmente».
T-P3-05 comprende tipi/presenza base/range; regex e compatibilità generale
constraint/tipo appartengono a T-P9-04/05, reference data a P11 e così via.
Il formato 1 non dispone di una capability separata per il solo sottoinsieme
fisico. required_capabilities può essere vuoto nella grammatica, ma il
contratto impone di inferire dall’AST le capability necessarie: omettere
local.v1 per aggirare il gate non è una soluzione.

**Impatto.** Con una lettura rigorosa del gate non si può consegnare ora un
compiler positivo per il sottoinsieme minimo, né dichiarare local.v1 completa.
Si può solo fallire esplicitamente; ciò non soddisfa la consegna positiva
prevista da T-P3-05. Implementare in anticipo tutti i costrutti locali
supererebbe lo scope autorizzato. T-P3-05 e le sue dipendenti restano ferme.
Nessuna violazione normativa viene convertita in errore tecnico o viceversa.

## Proposta A — approvata: compilazione incrementale verificata sull’AST

Deroga circoscritta al solo gate di compilazione di local.v1 durante lo
sviluppo incrementale, senza dichiarare completa la capability del motore:

1. T-P3-05 può produrre CompiledFlowSchema soltanto se **ogni costrutto presente**
   appartiene al sottoinsieme effettivamente implementato e verificato.
   local.v1 deve comunque essere dichiarata; nessuna capability inferita
   viene omessa per eludere controlli.
2. Sottoinsieme iniziale: Numeric/Alphanumeric del formato 1, Required/Optional
   senza condizioni, posizioni/lunghezze, padding e missing espliciti,
   classi lessicali; risoluzione delle PK locali obbligatorie nel formato e
   proiezione dei relativi campi. Nessun controllo runtime di unicità anticipato.
   Alphabetic resta escluso per il rinvio già approvato; Date resta un
   costrutto non supportato da questo primo nucleo, con errore esplicito.
3. Constraint non vuoti, DSL/regole, join, identità, derivati, gruppi,
   reference dataset, parametri e policy persistenti/temporali non supportati
   causano issue Error aggregate: non sono eliminati o trasformati in no-op.
   Le capability diverse da local.v1 restano bloccanti nel nucleo iniziale,
   anche se dichiarate con AST vuoto. Metadata non eseguibili sono preservati.
4. Nessun EngineCapabilities, CLI o runtime può pubblicizzare local.v1
   completa sulla base di questa consegna. La compatibilità del motore resta
   distinta dalla capacità del compiler di risolvere questo preciso schema.
5. Test negativi per i costrutti esclusi, capability mancanti/non supportate,
   binding encoding mancante e issue aggregate; positivi con schema sintetico
   del solo sottoinsieme. SIOC/CAD attuali devono fallire esplicitamente
   sulle funzionalità ancora escluse, senza ridurre i profili o le baseline.
6. La decisione va registrata come integrazione del contratto approvato;
   ARCH/DET/Piano e baseline originali non vengono riscritti. Il report della
   task deve elencare esattamente il supporto e mantenere tutti i gate/CI.

Questa proposta autorizza un gate di compilazione per costrutti durante lo
sviluppo; non modifica regole normative, verità a tre valori o semantica degli
esiti e non autorizza automaticamente le task successive.

## Binding encoding: attuazione già prevista dal contratto

SCHEMA_AUTHORING richiede che il servizio leghi l’encoding al RawEncodingSpec
in memoria prima di compile(raw), mantenendo la signature. Attualmente il
DTO generato conserva solo mode=runtime_required e position_unit=byte.
L’attuazione proposta aggiunge al DTO un binding tipizzato **solo Rust**, non
serializzato né deserializzabile dal JSON; adegua la generazione riproducibile
per questa estensione senza cambiare la grammatica. Il compiler non prende
encoding dal locale o da un default. Inizialmente UTF-8 esplicito per le fixture
sintetiche; gli altri encoding restano non supportati con issue tecnica.
Questo non seleziona l’encoding regionale e non chiude DEC-13. Il binding
risolto va preservato separatamente dall’hash del documento originale, come
già richiesto dal contratto. Nessun field raw dipenderà da strutture compiled.

## Alternativa B — mantenere il blocco

Mantenere invariato il gate capability completo e rivedere separatamente la
granularità delle capability o il piano di consegna. Nessun kernel parziale
viene dichiarato T-P3-05 DONE; nessuna nuova capability viene inventata qui.

## Verifiche eseguite / non eseguite

Ispezione diretta dei contratti e delle API; script preflight exit 0 verifica
che il formato ammetta local.v1, preveda solo encoding runtime_required e che
il compiler non esista. Nessun cargo/test/CI eseguito per questo dossier
puramente documentale; le evidenze dei prerequisiti restano quelle sopra.
Decisione richiesta all’autore: approvare A oppure mantenere B.
