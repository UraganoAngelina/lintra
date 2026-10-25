# T-P13-08 — Golden referenziale reale SIOC e memoria

2026-10-25. Report tecnico, spiegazione del codice e handoff della ratifica
CNF-SIOC-REFERENTIAL-G0-01. Nessuna skill Superpowers, subagente o processo
locale in background. STOP al termine di questa task secondo mandato specifico.

## 1. Prerequisiti e CI T-P13-07

T-P13-07: codice 5dbb92c6ec5afad50e8f2ae1dfa96a8a36a1ae44 pubblicato;
CI 40183843102, Linux 121140342859 e MSVC 121140343028 completed/success
verificati direttamente. 602 test/70 suite e gate locali passati. La ratifica
non è usata come evidenza CI; il suo riferimento a CI allora pendente è storico.
T-P0-08/G0 approvati con rinvii, documenti/source e prompt13-08/DET§152 verificati.

Il preflight ha rilevato una discrepanza del profilo operativo approvato:
join obbligatori senza assertion e revoking_act negativo N/A contro E012.
Riproduzione sintetica reale e proposta restano nella cronologia; l'autore ha
ratificato una nuova revisione, non nuovi controlli impliciti del motore.
[Decisione](CNF-SIOC-REFERENTIAL-G0-01-ratification.md), matrice e AGENTS aggiornati
per appendice, senza riscrivere decisioni precedenti. Nessun residuo interpretativo
su queste relazioni dopo la ratifica; gli ID REL-* non sono nuovi codici regionali.

## 2. Nuova revisione e differenze esatte

`schemas/sioc/2026-04-20.structural-minimal.2` è la nuova revisione derivata.
Profilo .1 e FROZEN invariati. Il source metadata distingue versione padre,
hash e decisione; versioning non viene dedotto da latest o dalla data del sistema.

| Trace | Nuova regola esplicita | Fonte/puntatore approvato FROZEN | Join / chiavi / cardinalità |
|---|---|---|---|
| 2 | REL-trace2_to_trace1 | /traces/1/joins/0, required=true | trace2_to_trace1: (2.1,2.49,2.50)→1.(1.1,1.17,1.18), exactly_one |
| 3 | REL-trace3_to_trace2 | /traces/2/joins/0, required=true | trace3_to_trace2: (3.1,3.35,3.36)→2.(2.1,2.49,2.50), exactly_one |
| 3 | REL-trace3_to_trace1 | /traces/2/joins/1, required=true | trace3_to_trace1: (3.1,3.35,3.36)→1.(1.1,1.17,1.18), exactly_one |
| 4.2 | REL-trace42_to_trace41 | /traces/4/joins/0, required=true | trace42_to_trace41: (4.20,4.21)→4.1.(4.1,4.2), one_or_many da at_least_one |

Tutte le dichiarazioni sorgente risultano prive di guard aggiuntive; fonti
formalizzate in docs/normative/sioc/rules-and-joins.md e decisione strutturale
DEC-SIOC-MINIMAL-01. La ratifica§3/4 autorizza const true per queste relazioni
incondizionate. Il generator verifica required=true, assenza di guard, target,
chiavi ordinate, cardinalità e codice prima di serializzare; non inferisce una
nuova norma dalla sola cardinalità. Una diversa fonte viene rifiutata, non adattata.

Per le quattro regole: assertion record_exists sul join, codice dichiarato
E_MISSING_PARENT_RECORD, messaggio «Record correlato richiesto assente», policy
record/trace Violation ereditate. Il codice è simbolico della fonte approvata,
non viene attribuito un codice numerico regionale inesistente. Ledger per ogni
regola con i 14 elementi richiesti: fonte/puntatore/snapshot/hash, trace/target,
join/chiavi/cardinalità, applicabilità, policy/codice/legacy/messaggio e test.

revoking_act mantiene self-join4.2, chiave4.30→4.22, zero_or_many, guard della
regola originale, scope current_submission e nessun filtro UDO aggiunto.
Le sole policy cambiate sono quelle esattamente autorizzate: record assente
verificato Violation E_RELATED_RECORD_MISSING/RELATED_RECORD_ABSENT con testo
approvato; trace indisponibile Unknown/RELATED_TRACE_UNAVAILABLE con testo
approvato. E012 resta metadato legacy di R-4.30-REFERENCES-4.22.

Join opzionale trace1_to_trace2_episode e le due policy N/A sono identici a .1.
Le regole originali, field layout/type/presence/constraint, condizioni, identità,
PK, capability, policy persistente e rinvii restano identici. Il test ripristina
solo le differenze autorizzate e confronta l'intero albero con .1; verifica anche
il prefisso originale delle regole e hash/proiezioni dei cinque trace nuovi.

## 3. Hash e provenienza

Hash esatti e dimensioni di tutti gli artefatti: [manifest](T-P13-08-artifact-hashes.json).
Il manifesto include profilo .1, relativo ledger e formalizzazione come sorgenti,
nuova revisione completa, componente, ledger e fixture/expected. Non contiene
un hash autoreferenziale; i manifesti figli dichiarano il proprio ambito.

- FROZEN flow sorgente: 3567bafd3cb9b762becdd46a356454b89df094c7b88c5b1538c97a1bbaf26c45.
- Profilo .1: 2592c46889a2c06fc9dd5da5b9220f2f1c2a3ac42985f56bd24e476058128c31.
- Nuovo flow .2: b1bffeb800570d8f4ac045110bf849e5380ed4062f209b4cb1f34dee3df1c33b.

Il componente dichiara il vero hash del padre .2; non confonde tale hash con
quello del nonno .1. Le prove Rust verificano anche questo binding. Gli hash
source_join_snapshot sono SHA della serializzazione JSON pretty del sottoalbero,
non presunte slice byte originali del documento; l'hash FROZEN intero resta quello
esatto letto localmente. Il kit normativo serve solo alla riproduzione documentale,
non alla build/runtime/CI dei test. .gitattributes preserva byte della fonte .1
usata per hash e nuovi artefatti .2/golden su Windows; i contenuti .1 non cambiano.

## 4. Componente operativo autorizzato e limiti

Versione separata `2026-04-20.references-golden.1`, sotto tests/golden/cross-trace.
Seleziona tutte e quattro le relazioni cross-trace reali e la revoca, più la
regola locale già approvata 4.30 richiede4.29. Cinque trace, coordinate/larghezze
originali, PK/join/identity invariati. Venti field selezionati conservano physical,
missing e presence originali; le regole sono identiche a .2.

Esclusioni esplicite nel scope-ledger del componente:
- 115 field non selezionati, con hash della definizione sorgente; relativi
  controlli/condizioni/derivati e rinvii non vengono dichiarati eseguiti.
- Sei constraint logical_length/composite_segments di tre codici azienda non
  supportati: restano nella .2 completa, sono elencati per field/indice/contenuto.
- Sei span opachi coprono coordinate escluse senza fingere campi/check normativi;
  il tipo contenitore AN verifica soltanto il trasporto dei byte sintetici.
- Persistenza storica/audit persistente e requirement persistence.audit rimangono
  nel completo; questo diverso componente non li seleziona e non li implementa.
- Dataset ufficiali, lookup metadata/temporali, domini clinici e altri rinvii G0
  non vengono ricevuti, inventati o dichiarati chiusi.

Il profilo .2 completo conserva sei capability incluso persistence.audit.v1 e
la policy persistente originaria: il compiler continua a rifiutarlo. Il test
verifica il rifiuto effettivo, non rimuove flag per renderlo compilabile.
Il componente è dichiaratamente diverso, con scope/hash/versione/esclusioni;
Full/Complete del relativo summary riguarda solo tale componente e non l'intero
SIOC. Nessuna certificazione regionale o accettazione clinica completa.

## 5. Golden produttivo e oracolo indipendente

Il test sioc_referential usa repository filesystem/registry/compiler veri,
ValidationService Full, InputAuditSession, FlowValidator, store e passata cross,
JSONL e summary reali. Input invertiti rispetto all'ordine schema provano
l'ordinamento. Clock iniettato fisso (port reale previsto), input SHA e schema
fingerprint canonico prodotti dal servizio. Si normalizza soltanto elapsed,
non schema hash, contatori, versione o metadati audit.

L'oracolo Python non legge output Rust. I record sono costruiti da zero su
coordinate dichiarate, con valori sintetici. Gli expected derivano dalle tre
specifiche normative dichiarate, non da un dump del software. L'hash canonico
schema è calcolato indipendentemente ordinando le proprietà JSON; array/byte e
interi esatti sono preservati. Hash input sui byte originali, compresi padding/LF.
Identity diagnostica fisica20 byte distinta dalla chiave logical, come contratto
IdentityExtractor preesistente: nessun trim inventato dal test.

| Golden | Record | Errori | Esito cross/completamento |
|---|---:|---:|---|
| valid | 7, tutti validi | 0 | Quattro parent richiesti presenti; due parent amministrativi ammessi; atto revocante trovato; Complete |
| missing | 3, tutti invalidi | 5 | Quattro obblighi parent distinti + atto revocante assente; invalid record contato una sola volta anche con più errori; Complete nello scope |
| cardinality | 7, 1 invalido | 1 | Duplicato PK anagrafica; due join scalari Unknown/Ambiguous, niente first(); one_or_many amministrativo soddisfatto; Partial |

Golden missing: trace2 una violazione, trace3 due verso target distinti,
trace4.2 due per revoca e struttura. Sono cinque cause/regole distinte,
non ripetizioni dello stesso controllo. Ordine trace/riga/regola stabile.
Gli errori di assenza hanno chiave/source reale; un parent inesistente non può
produrre valori target inventati. Audit con RuleId, schema/ledger con JoinId
identificano obbligo e destinazione. Il supporto generico al contesto di target
presenti nel report è già verificato dalle regressioni cross13-05.

## 6. Regressioni richieste dalla ratifica

Oltre ai tre confronti byte-exact:
- Full target indispensabile omesso fallisce in preflight anche quando tutti
  gli altri path sono inesistenti: prova nessuna lettura input prima del rifiuto.
- Riga 4.2 non proiettabile rende l'universo revoca Partial: una ricerca negativa
  diventa Unknown/IncompleteRelatedContext; solo E_RECORD_LENGTH, non falso E012;
  due valutazioni cross note saltate, conteggi e audit coerenti.
- Variante puramente sintetica senza regole, con min=1 invariato: zero controlli
  impliciti/violazioni generati dalla dichiarazione del join.
- Sonda sintetica della policy opzionale copiata intatta: N/A, testo approvato,
  JoinId, zero errori; non viene aggiunta una regola inversa al profilo SIOC.
- Policy trace Unknown corretta testata nel public executor standalone; nessuna
  conversione in False/Violation/N/A. Questo non bypassa il servizio Full, il
  cui rifiuto di trace indispensabile mancante è provato separatamente.
- Integrità .1/.2, capability complete ancora rifiutate, definizioni selezionate,
  ogni field/constraint escluso nel ledger, hash/trace proiettati verificati.
Nessun source file Rust di produzione modificato da questa task.

## 7. Memoria — codice e ragioni di correttezza

Un solo test in tests/memory.rs, nessun crate nuovo. Meter implementa GlobalAlloc
inoltrando alloc/alloc_zeroed/dealloc/realloc a System con puntatore/layout esatti.
Unsafe confinato al test e documentato per il contratto dell'allocatore; nessun
unsafe introdotto nel core. Le atomiche LIVE/PEAK registrano byte richiesti vivi,
non RSS, overhead dell'allocatore o transitori interni a realloc. acquired aggiorna
il peak solo nella finestra MEASURING; alloc fallita non viene contata, realloc
fallita conserva il vecchio conteggio. Le callback non allocano/interpretano byte.

schema(width) costruisce un flow sintetico reale con PK8, KEY8, DATA2 e filler
non usato; il piano compilato deve proiettare esattamente i tre field necessari.
write_records genera file prima della misura con buffer riusabile, chiavi uniche
e valori validi: nessuna Vec di righe contamina la misura. sample esegue il
servizio Full reale; CountSink conta gli errori/summary effettivi senza trattenerli.
Il callback campiona i byte vivi al CrossStarted, dopo rilascio dei reader.
L'outcome deve essere CompletedValid/Complete con tutti i controlli attesi;
un contesto vuoto fittizio o nessuna passata non può superare il test.

Confronti: 1024→4096 righe per trace (4x), poi larghezza32→4096 (128x) a N fisso.
Il contesto deve crescere col numero di proiezioni; a N fisso può differire meno
di256KiB per overhead, non Nxwidth. Anche il peak deve rimanere entro tale
tolleranza; il contesto wide deve essere meno della metà dei byte grezzi delle
righe. Si misura la proprietà richiesta, non un benchmark di throughput.

Misura locale esplicita con --nocapture:

| Righe per trace | Larghezza | Byte contesto a pass1 | Picco richiesto |
|---:|---:|---:|---:|
| 1024 | 32 | 1.768.776 | 1.869.275 |
| 4096 | 32 | 7.040.904 | 7.393.307 |
| 4096 | 4096 | 7.040.904 | 7.397.371 |

Contesto identico a N fisso, picco +4064 byte, non copia dei 33.554.432 byte
fisici complessivi. Il contesto include indici/Arc/mappe/PK/identità necessari;
non si promette memoria costante rispetto a N. I gate nativi verificano le
stesse proprietà su Linux/MSVC, non questi numeri specifici.

## 8. Generator e riproduzione

profiles.py: encoded/digest definiscono encoding/hash; build verifica .1/FROZEN
prima di creare le sole differenze autorizzate; subset_of enumera selection ed
esclusioni; opaque preserva coordinate senza simulare controlli; main verifica
per default e scrive nuovi artefatti solo con --write. I source non sono destinazioni.

Goldens.py: row produce bytes dal layout; rows definisce i tre scenari;
identity/context/error costruiscono l'oracolo diagnostico da contratti e valori
sintetici; controls assegna gli esiti attesi per obblighi/cardinalità; main produce
summary/hashes/manifest indipendenti. Default verifica; --write riproduce la
specifica, non copia output Rust. Rust assert_bytes segnala il primo diff limitato;
execute usa il servizio e conservazione audit reali, non validator simulati.

## 9. Gate, M6 e arresto

Esiti locali/nativi e commit saranno registrati soltanto dopo esecuzione.
13-08/M6 richiedono insieme golden reale, memoria e gate Linux/Windows, non la
sola approvazione o la porzione memoria. M6 eventualmente raggiunta è il DoD
Full-flow nello scope referenziale autorizzato, non certificazione Full SIOC;
PERSIST/G0 clinico/dataset ufficiali/UI restano distinti e aperti.

Al completamento aggiornare stato/handoff con hash, fonti, risultati e limiti.
Fermarsi: nessuna task successiva senza nuova autorizzazione specifica.

### Integrazione con l'harness storico

Il primo gate globale ha correttamente rifiutato i nuovi expected multi-input
come orfani: il discovery storico richiede input.txt e un producer a singolo
lettore. tests/golden.rs ora registra esplicitamente quattro producer sintetici
e la famiglia cross-trace delegata al runner produttivo dedicato. Verifica
ricorsivamente famiglie/versioni: profili sconosciuti o symlink restano errori.
Il discovery originale rimane rigoroso dentro ogni profilo single-input;
il runner normativo verifica l'esatto insieme delle directory dei tre casi.
LINTRA_UPDATE_GOLDEN non può copiare output runtime negli expected SIOC.
Nessun golden precedente aggiornato o normalizzato per risolvere il fallimento.

### Gate locale effettivo

2026-10-25: generator profili default9 artefatti e generator golden default22
artefatti entrambi passati; cargo fmt --all -- --check, cargo check --all-targets,
cargo clippy --all-targets -- -D warnings, cargo test (611/72 suite, 0 fail,
1 ignored), cargo build --release e cargo check --all-targets --target
x86_64-pc-windows-gnu passati con --offline --locked/CARGO_HOME isolata.
git diff --check passato. Manifest33 hash senza mismatch, baseline90 identica
a6e4b411. I gate sono stati ripetuti dopo l'integrazione del nuovo harness;
nessun expected precedente modificato per risolvere il primo rifiuto.
Gate nativo Linux/MSVC avviato sul commit pubblicato; esito non accessibile,
come dettagliato sotto. Nessuna attestazione di successo.


### Pubblicazione, CI remota e handoff finale della sessione

Codice52762c8525b878a72c7a59f9589c207a17e1efc6 pubblicato su
work/p1-foundations il2026-10-25. Il push include79b31f3 (chiusura13-07).
Sono stati contabilizzati tre tentativi, con intervalli≥45s:

1. POST dispatch: timeout del resolver DNS prima di contattare l’API GitHub.
2. POST dispatch: risposta di successo senza body, circa12:18 UTC.
3. GET check-runs dello SHA52762c8, dopo oltre8 minuti: timeout di connessione
   dopo10s. Nessun risultato job ricevuto.

Non è stato effettuato un quarto tentativo, né un nuovo gate per aggirare il
limite. Il timeout non dimostra un test fallito o una CI positiva. Run/job non
identificati; Linux/MSVC **non verificati**. La CI13-07 resta invece positivamente
verificata come riportato nella sezione1. Nessun risultato storico modificato.

**T-P13-08 resta IN_PROGRESS; M6 non dichiarata.** Il golden reale DET§152 e
la memoria sono superati localmente; manca soltanto il gate nativo effettivo.
Alla ripresa autorizzata verificare il run già avviato sul commit esatto, senza
duplicare il codice. Nessuna task ulteriore viene avviata: stop della ratifica.

[Stato di avanzamento completo](PROJECT_PROGRESS_2026-10-25.md):84/140 DONE,60%;
54 non iniziate, T-P2-03 parziale e13-08 locale verificata/native pendente.
Il manifest33 hash e i90 file protetti sono stati ricontrollati dopo il push:
identici alle evidenze locali. La modifica esterna alla newline della proposta
reference rimane esclusa da staging/commit.


### Chiusura verificata alla ripresa autorizzata — 2026-10-25

Il nuovo mandato autorizza lettura del gate già avviato e prosecuzione automatica
oltre lo stop precedente. Prima chiamata GET della ripresa sullo SHA esatto:
[CI40192491629](https://github.com/UraganoAngelina/lintra/actions/runs/40192491629),
Linux job121166275121 e MSVC job121166274875 entrambi completed/success sul codice
52762c8525b878a72c7a59f9589c207a17e1efc6. Nessun nuovo dispatch. I timeout della
sessione precedente restano evidenze storiche, non sono reinterpretati.

**T-P13-08 DONE; M6 raggiunta nello scope referenziale ratificato di DET§152.**
Tutti i criteri obbligatori golden reale, memoria e gate sono verificati;
non costituisce certificazione Full SIOC né chiusura dei rinvii già elencati.
Avanzamento85/140=60,7%. Next T-P14-01 preflight, secondo nuovo mandato.
