# T-P13-08 — Preflight golden normativo e memoria

2026-10-25, prima delle modifiche. Prerequisiti 13-07 codice5dbb92c/CI40183843102
Linux/MSVC success e 0-08 scope minimale G0 approvato con rinvii verificati.
Letti prompt13-08, DET§152/§73–79, Piano§4/§9/§14/DoD M6, report0-08/0-09,
SCHEMA_AUTHORING, DEC-SIOC-MINIMAL-01, profilo operativo .1 e ledger.
Nessuna skill Superpowers/subagente/processo locale in background.

## Golden G0 — decisione documentale pendente

Vedi proposta docs/proposals/sioc-referential-g0-contract.md, sottoposta all'autore.
Join obbligatori non hanno regole eseguibili nel minimale; revoking_act ha policy
negativa N/A mentre R-4.30-REFERENCES-4.22 richiede esistenza. La regola attiva è
self-join, non golden cross-trace reale di DET§152. Non sostituire il requisito
con un esempio sintetico né una regola clinica rinviata. Non modificare profilo
approvato, FROZEN, fonti o baseline senza la decisione richiesta. Nessuna M6.

## Porzione indipendente autorizzata — memoria

Nuovo tests/memory.rs: allocatore di test System con conteggio dei byte richiesti
vivi/peak, senza crate nuovi. Unsafe limitato al forwarding del contratto
GlobalAlloc e documentato; nessun unsafe nel core. Un solo test nello specifico
eseguibile evita interferenze di test paralleli dello stesso allocatore.
Misura richieste all'allocatore, non RSS o un benchmark di performance.

Eseguire il servizio Full reale su file sintetici generati prima della misura,
schemi compilati e byte validi. Stessi campi necessari (PK/join/dato confronto),
solo filler non usato variabile. Campionare memoria al checkpoint CrossStarted,
dopo rilascio del reader, e peak dell'esecuzione. Confrontare numero record 4x
e larghezza fisica circa128x: crescita per record attesa sulle sole proiezioni,
non Nxlarghezza. Verificare direttamente che filler non sia nel projection plan.

Adapter sink conta errori/summary effettivi senza raccogliere record/errori;
nessuna progress Vec proporzionale ai record. Misura include registry/compiler,
indici, audit e contatori reali; tolleranze dichiarate per overhead ambientale.
Drop del run fra i casi; non attribuire al contesto i buffer input fissi.
Validare outcome/contatori/cross/completeness e checkpoint effettivamente visto.
stdout solo metriche aggregate di dati sintetici, nessun record paziente.

Gate locali pertinenti e globale, release/Windows GNU e CI Linux/MSVC. Una
consegna memoria positiva resta parziale: 13-08 non è DONE senza golden G0 reale.

## Decisione e sviluppo successivi — 2026-10-25

L'autore ha ratificato CNF-SIOC-REFERENTIAL-G0-01 con precisazioni vincolanti:
nuova .2, quattro regole esplicite e policy revoking_act, nessun automatismo
engine, .1/FROZEN preservati, componente operativo tracciato autorizzato.
Il conflitto sopra è storico/risolto, non pendente. Stop specifico dopo13-08.
Implementazione/golden/memoria locali passati: 611 test/72 suite, gate completi,
33 hash artefatti e baseline90 intatta. CI nativa da verificare. Dettagli nel
report T-P13-08-goldens-memory.md e nella ratifica registrata; nessuna M6 anticipata.
