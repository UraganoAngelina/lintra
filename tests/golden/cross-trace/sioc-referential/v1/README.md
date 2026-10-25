# Golden referenziale SIOC — componente operativo esplicito

**Scope: sole relazioni strutturali selezionate, non Full dell'intero SIOC.**
Versione `2026-04-20.references-golden.1`, derivata da structural-minimal.2
secondo CNF-SIOC-REFERENTIAL-G0-01. Il servizio esegue Full di questo componente
esplicitamente delimitato; Complete nel relativo summary riguarda questo scope.
Non è certificazione regionale, accettazione clinica o chiusura dei rinvii G0.

Sono presenti tutti i cinque trace con larghezze/coordinate originali, PK,
join, identity e sei regole della revisione .2 (cinque cross-record, una locale).
Sono selezionati 20 campi; le definizioni originali di physical/missing/presence
sono preservate. Sei constraint logical_length/composite_segments non supportati
sono esplicitamente esclusi da questo componente e rimangono nella .2 completa.
I 115 campi non selezionati e le loro verifiche sono elencati nel scope-ledger.
Sei span opachi coprono le posizioni escluse: sono contenitori byte dichiarati,
non campi normativi simulati e non eseguono i controlli mancanti.

Il ledger specifica esclusioni e fonte. persistence.audit e la policy persistente
restano nella revisione completa, che il compiler attuale rifiuta. Questo diverso
componente non seleziona storico/persistenza/lookup/clinica: non li implementa,
non elimina requirement dal completo e non usa dataset ufficiali inventati.
organisation_namespace viene fornito esplicitamente come intero tecnico 1.

Tutti i record sono costruiti da zero per i test. E001/E002/V001, UDO, MRA e STS
sono valori sintetici e non attestazioni di appartenenza a tabelle regionali.
Byte non selezionati sono spazi; ciò non implica validità del tracciato completo.
Nessun export paziente viene letto/copiato o usato come fixture.

| Caso | Significato normativo e risultati attesi |
|---|---|
| valid | Quattro obblighi parent soddisfatti, atto revocante trovato; due parent amministrativi ammessi da one_or_many; 7 record validi, 0 errori |
| missing | Universi disponibili/vuoti o senza riscontro; 4 obblighi parent violati + atto revocante E012 negativo; 3 record invalidi una volta ciascuno, 5 errori distinti |
| cardinality | Due anagrafiche con la stessa PK: una violazione duplicato; due join scalari Ambiguous/Unknown, nessun first(); parent amministrativi multipli ammessi; Partial |

Le due violazioni su trace 3 riguardano due obblighi distinti verso trace2/trace1,
non duplicazioni della medesima regola. Ordine: trace schema, riga, regola.
I record assenti non producono valori target fittizi: l'errore conserva la
sorgente/chiave reale; RuleId/JoinId e destinazione sono ricostruibili da audit
per regola e ledger/schema hash. La diagnostica identity conserva i 20 byte
fisici, mentre la join usa logical con stripping dichiarato; non sono la stessa chiave.

Il test attraversa loader/registry/compiler/service/FlowValidator/store/cross
pass/JSONL/summary reali. Clock iniettato fisso, hash schema canonico e input SHA
calcolati dal percorso produttivo; viene normalizzato solo elapsed wall time.
Il confronto è byte per byte, compresi padding, Unicode e newline.

L'oracolo Python non legge output Rust: costruisce record e risultati dai tre
scenari normativi dichiarati. oracle-manifest.json contiene motivazioni, SHA
canonico schema e SHA di tutti gli input/expected. Non usare un update che copi
l'output corrente del software come aspettativa.

```sh
python3 docs/verification/generate_sioc_referential_goldens.py
cargo test --locked --test sioc_referential
```

Default Python verifica soltanto. --write è la riproduzione esplicita dell'oracolo.
Le regressioni supplementari coprono target indispensabile assente in preflight,
universo revoca parziale Unknown, N/A opzionale, policy trace Unknown e assenza
di controlli impliciti. Le assertion sintetiche usate per sondare N/A non vengono
aggiunte a questa componente o alla revisione normativa derivata.
