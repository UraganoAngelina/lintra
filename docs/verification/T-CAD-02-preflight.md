# T-CAD-02 — preflight avviato dopo DEC-17, 5 novembre 2026

Stato: IN_PROGRESS, preflight documentale completato; implementazione e gate
T-CAD-02 ancora da eseguire. Nessun DONE o supporto formato 3 dichiarato.
Branch: work/p1-foundations, base della ripresa 42392b3; nessun helper dello
spike è stato trasferito, inclusi quelli cfg(test).

## Prerequisiti ed evidenze

T-CAD-01 DONE: CI42004833488 su46d574f, sette job positivi nel ledger
T-CAD-01-10-native-ci.json, gate locali T-CAD-01-local-gates.json. Il registro
consente la selezione dichiarativa del profilo, non l'esecuzione CAD completa.
T-PERSIST-02 DONE: CI40528607856 completed/success su
e9f477b40b5e3e078ffe763445620562ec6bdca6, T-PERSIST-02-native-ci.json;
gate locali e tipi namespace/sessione/presenza già verificati. G0 approvato
con rinvii espliciti: non è una promozione FROZEN o un via libera a norme future.
La ratifica CNF-CAD-NAMESPACE-BINDING-01 A estesa autorizza l'estensione necessaria;
non richiede una nuova scelta ordinaria. Nessun blocco di autorizzazione rilevato.

Fonti lette: prompt T-CAD-02, AGENTS/master prompt, dependency map, indice/stato,
ARCH §45, DET §166–174, PERSISTENCE_CORE_SPEC, rules-and-joins CAD e ratifica
namespace. Le fonti legacy del candidato restano distinte dall'AST eseguibile.

## Riscontro sul codice e perimetro successivo

compatibility.rs assegna soltanto i formati 1 e 2: 3 disponibile, ancora non
supportato. RawPersistence nega proprietà sconosciute e non contiene binding;
non aggiungere proprietà ai vecchi formati tacitamente. La nuova dichiarazione
richiede raw, meta-schema/authoring, capability, compiler, canonicalizzazione e
compatibilità sincronizzati. Il valore di invio non appartiene al fingerprint.

persistence_profile.rs compila namespace/chiavi/snapshot dichiarativi;
compile_historical_audit supporta soltanto audit e rifiuta snapshot/temporali.
Non scambiare un profilo dichiarativo per esecuzione Full/Historical CAD;
il completamento della riconciliazione trimestrale rimane nelle task pertinenti.
PK normative A/B1/B2/C/D invariate, periodo fuori dal namespace ULSS continuo.

ValidationRequest non porta un namespace tipizzato; validate_with_parameters usa
una mappa testuale e il percorso LocalOnly rifiuta attualmente parametri. Il
binding stringa non può essere simulato tramite literal o coercizione numerica.
L'estensione autorizzata deve introdurre contesto verificato unico e immutabile,
con riuso nello scope join, storico, preview/approvazione/commit e audit.
full::bind_parameters e session::bind sono i confini esistenti da riconciliare;
non mantenere valori indipendenti per validazione e storage.

RecordValidator/TraceValidator eseguono il locale prima di ProjectionCollector;
il confronto compilato deve inserirsi dopo la validità del campo e prima della
proiezione, senza terza passata né riclassificazione di regole cross. Un campo
invalido/assente non genera un mismatch derivato. Full richiede il parametro in
preflight; LocalOnly senza parametro conserva il significato diagnostico senza
attestare appartenenza. Parametro presente invalido è distinto da assente.

A assente, presente-vuoto e presente con record sono casi separati: nessuna
appartenenza dei child dedotta dai metadati del DB. Le policy missing trace e il
consenso già esistenti rimangono validi. Verificare le approvazioni reali senza
bypassare Unknown/Partial o introdurre un divieto generale di file assenti.

## Verifiche richieste all'implementazione

Match501/501, mismatch501/502, dominio/tipo/mancanza parametro nelle modalità
LocalOnly/Full/storico/approvazione; campo invalido senza cascade; ULSS isolate
con stesso evento e continuità fra anni/trimestri; A assente/vuoto/child senza A.
Formati1/2 e FROZEN invariati, rifiuto binding senza formato/capability corretti,
fingerprint sensibile al contratto ma non al valore della request, max_errors,
ordine corrente, summary/JSONL e nessuna mutazione/approvazione dopo invalidità.
Eseguire gate locali e CI Linux/MSVC prima di DONE. Nessun nuovo benchmark,
parallelismo, modifica HMAC/storage o dataset clinico reale.

DEC-17 è chiusa indipendentemente: questo preflight non anticipa un esito di
T-CAD-02. CNF-DIAGNOSTIC-ORDER-01 resta aperto separatamente; preservare l'ordine
runtime corrente. Prossimo passo implementativo: grammatica/compilazione del
binding, poi contesto tipizzato e integrazione locale con regressioni coerenti.
