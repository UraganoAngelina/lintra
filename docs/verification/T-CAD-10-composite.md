# T-CAD-10 — T(70) e posizione60 condizionale

## Preflight e fonti — 2026-11-05

Task indipendente daCAD02 e dal nuovo binding ULSS ancora proposto. P6 qualificata
in P6-08 daa9f15/CI39449345726, poi ripetutamente nel core attuale; P10 conclusa,
ultimo ampliamento derivati c2e0d1d/CI39920597392. G0 approvato407f89e con rinvii,
contratto authoring72f9f5e approvato. Prerequisiti del prompt10 soddisfatti senza
usareCAD01 ancora in attesa del proprio gate nativo. Single-agent, nessun daemon.

Letti ARCH45, DET166–174, specifica persistente, prompt10 e piano4/14/20; candidato
trace_b1.json/CAD_RUNTIME_PROFILE, CAD_NOTES_AUDIT e fixture plan. Riscontro
pdftotext del PDF regionale:2.22 lunghezza70, inizio67, righe problemi di salute
con61–70 fissi2; allegato posizione60 riservata UCPDOM. G03 ratificata determina
lo spazio ASCII fuori UCPDOM; nessuna nota trasformata in nuova scelta normativa.

## File e comportamento

schemas/cad/components/2024-06.t70.1/component.json conserva fonte/hash e copie
integrali dei due campi legacy per audit. Espone due definizioni campo e due
regole composte con nodi già approvati; non è un nuovo formato di flow e non è
caricato come profilo CAD completo. Nessuna distribuzione/bundle/FROZEN modificati.
Il README dichiara scope, limiti e composizione esplicita futura.

2.3 conserva posizione15/larghezza1, dominio1/2, required eassenza exact0.
2.22 conserva posizione67/larghezza70, required/assenza spaces. Il lowering
esplicito di T è byte text con alignment exact e regex raw:1–59 =1/2,61–70=2.
Il dominio base ammette a60 soltanto1/2/spazio; le due regole condizionali,
guardate da presenza dei due campi, impongono1/2 in UCPDOM eASCII32 fuori.
Con campo mancante le guardie impediscono una seconda valutazione indefinita;
la presenza required continua a produrre la violazione primaria. Nessun nuovo
validator, byte-range, conversione numerica, trim o fallback nel motore.

Le regex usano ancore assolute \A/\z e operano su raw: nessun carattere extra,
normalizzazione Unicode o stripping implicito. Positional_bitset raw non ancora
implementato resta rifiutato dal compiler; non viene automaticamente convertito
per fingere che il candidato legacy sia eseguibile. La versione del componente
sceglie esplicitamente un’espressione equivalente con primitive già supportate.
Gli errori usano InvalidDomain del campo e CrossField della regola esistenti;
le regole aggiungono messaggi espliciti per posizione60. Nessun nuovo codice errore.

## Test e oracle indipendente

tests/cad_composite.rs costruisce un contenitore sintetico da198byte con gap
sintetici espliciti,2.3 a15 e2.22 a67; non valida gli altri34campi B1 né usa una
PK normativa incompleta come prova CAD. Il flow di test ha un’identità diversa
(CAD-T70-COMPONENT-TEST). Nessun dato sanitario, chiave o archivio reale.

Cinque test mirati passati. Il test esaustivo esegue2×70×256=35.840casi; l’oracle
usa i domini byte discreti della fonte, non le regex del componente. Altri casi
provano contesti validi, limiti fisici e byte vicini esterni, spazio/zero-fill,
blank in coda econtrolli mancanti/nonvalidi. Posizione60 errata produce diagnostico
campo2.22 a67–136, riga7 e messaggioesplicito. JSONL ripetuto byte-identico,
senzaCR. Nessuna summaryComplete o inputaudit inventata nei test del record.

Tentativi preliminari conservati:3test pass/2fail per UndeterminedRule su campo
mancante; prima forma di guardia respinta per proprietà field invece di ref;
corretto il contratto delle guardie e poi5/5pass. Nessun tentativo fallito
reinterpretato come successo; nessun contratto runtime cambiato per nasconderlo.

## Stato del gate

IN_PROGRESS. Gate locali completi in corso, poi commit/push e CI propria sulle
modificheCAD. La CI rc.2 qualifica soltanto8430de9 e non queste nuove policy/test.
Nessun DONE10 e nessuna qualifica del profilo CAD completo anticipati.


Primo gate completo interrotto daClippy: cinque liste di byte nei test richiedono
la forma byte-string. Corrette senza allow/soppressioni; semantica invariata.
Log/ledger tentativo in target/verification-cad-composite-clippy-initial;
il gate finale viene rieseguito sul nuovo hash dei test, non riusa quel fallimento.


## Gate locali finali conclusi

Cinque comandi exit0,408impronte immutate inclusi componenteJSON/README;
899esiti workspace/114riepiloghi,0fail/6ignored.12test mirati allfeatures
(CADcomposite5 e registry7), fmt/Clippyworkspace all-targets allfeatures eboundary
positivi. Ledger T-CAD-10-local-gates.json. Nessun engine nuovo, schemaFROZEN,
dipendenza o chiave operativa. Restano push/CI propria prima diDONE10.


## Chiusura nativa — DONE

CI42004833488 completed/success su46d574f8ec4627bcc9561d298c2dc33007137654;
sette job richiesti positivi, optionalUI skipped. Default/allfeatures/core su
Linux/MSVC, confronti JSONL/pixel e due riapertureSQLite crossOS effettivamente
positivi. Due letture ordinarie distanziate. Fonte T-CAD-01-10-native-ci.json.
Questo gate qualifica i due scope indipendenti registry01 e componente10,
non il profilo CAD completo né il futuro bindingformato3/parallelismo.
