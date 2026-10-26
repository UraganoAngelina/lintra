# Lintra — checkpoint T-P14-02,2026-10-26

**88 task verificate su140:62,9%.** T-P14-02 è DONE: gate locali e CI nativa
Linux/Windows superati sul commit e68881cd89b5cf100c58d7f682db191c4210e86b.
Workflow fermo al checkpoint richiesto dall’autore.

La percentuale conta gli ID atomici del piano con peso uguale:116 task originali
P0–P18,12 PERSIST core e12 CAD. Non misura ore, costi, percentuale di tempo rimasto
o copertura normativa. Una task DONE con scope ratificato conta per quel solo
scope, senza dichiarare completate le capacità rinviate.

| Area | Task verificate / totale | Stato effettivo |
|---|---:|---|
| Core, runtime, CLI e report P0–P13 |84/85| Operativi nello scope ratificato; T-P2-03 conserva il residuo approvato |
| Tooling schema P14 |4/5| Importer/normalizzazione/lint/canonical/diff verificati; CLI import/list futura |
| Packaging e release P15 |0/7| Non iniziato |
| UI Slint P16 |0/9| Non iniziata |
| Hardening/benchmark P17 |0/6| Non iniziato |
| Documentazione operativa P18 |0/4| Dossier di sviluppo esistenti, task di prodotto ancora future |
| Persistenza core |0/12| Requisito definitivo, backend/repository storico da implementare |
| Integrazione CAD |0/12| Formalizzazioni ricevute, runtime completo non consegnato |

Dopo la chiusura14-02:88 DONE,1 task parziale(T-P2-03),51 non iniziate.
Il totale rimane140; sono conteggiate soltanto le consegne verificate.

## Cosa è già operativo

- Pipeline byte-oriented streaming con schemi versionati, compiler/registry e
  handle compilati immutabili, rifiuto delle capability non supportate.
- Validazione locale, datatype/date/range/regex, Decimal monetario unsigned con
  due cifre e zero ammesso; codici/range di compartecipazione preservati.
- Membership e Union con attendibilità Verified/Degraded/Unavailable, risultati
  provvisori/inconcludenti e Complete/Partial distinti dagli errori normativi.
- Servizio/CLI Flow Full a due passate, Context Store selettivo, join/cardinalità,
  policy Missing/Unknown/NotApplicable, summary/audit e ordine riproducibile.
- Report JSONL/CSV/XLSX, cancellazione, digest input e pubblicazione atomica.
- Golden referenziale SIOC con regole reali approvate su dati sintetici, memoria
  proporzionata alle proiezioni e nuova revisione operativa structural-minimal.2.
- Importer XLSX reale con blueprint revisionato,135 binding SIOC, note originali,
  formule/cached values tracciati, hash degli stessi byte letti e lint successivo.
- Diff deterministico delle revisioni con hash canonical, add/remove/modify e
  ordine significativo, source metadata e pointer per entrambe le versioni.

## Consegna T-P14-02

Normalizzazione di alias espliciti con report, conservando token originali e
blueprint; nessuna classificazione automatica per nome del campo. Lint con
warning per note importate non revisionate, classificazione manuale legata
all'hash del testo, riferimenti a regole reali e diagnosi di domini incompatibili.

Le note non vengono trasformate in DSL o considerate prova di implementazione.
I warning di authoring restano distinti dagli errori normativi dei record. I
controlli strutturali e il compiler mantengono il rifiuto dei veri errori.
[Report tecnico e spiegazione codice](T-P14-02-authoring-lint.md).

642 test in75 suite locali passati, nessun fallimento;2 test esplicitamente
ignored nella suite ordinaria. La qualificazione sul workbook SIOC originale è
eseguita separatamente; la fonte esterna non è un prerequisito nascosto della CI.
Fmt/check/clippy/release/GNU e qualificazione SIOC post-lint passati.
CI [40290169690](https://github.com/UraganoAngelina/lintra/actions/runs/40290169690):
Linux121463096084 e MSVC121463095876 entrambi completed/success, letti direttamente
sull’esatto commit e68881c. Tre chiamate totali, intervalli superiori a60 secondi.

## Limiti e consegne future

Il componente SIOC verificato non equivale alla certificazione dell'intero SIOC.
Il completo.2 conserva persistence.audit e altre capability non supportate ed è
correttamente rifiutato. Baseline.1/FROZEN, fonti protette e dati normativi non
sono stati modificati per far passare test o compiler.

Restano12 task di persistenza storica,12 CAD,UI,release,hardening e documentazione
operativa. La persistenza dei file summary/audit è reale e distinta dal repository
storico persistente ancora futuro. Dataset ufficiali mancanti e rinvii clinici/
metadata-tempo/G0 non sono chiusi da queste task generiche. Nessun PDF/nuovo
flusso fornito dall'autore è stato implementato.

M6 raggiunta nel perimetro ratificato DET§152, M7 ancora non raggiunta: manca
14-05 e DEC-18 relativa ai contratti di elenco. L'esposizione CLI dei warning di
compilazione riuscita appartiene al wiring futuro14-05, non anticipata qui.

## Evidenze native e stop

14-01 e regressioni14-04: CI40238421197 su7b8bd8b, Linux121304841503 e
MSVC121304841698 entrambi completed/success letti direttamente. Il polling14-04
precedente incompleto resta storico, non viene trasformato in successo presunto.

Il mandato del checkpoint prevale sul workflow automatico: terminata14-02,
fermo con questo resoconto. Nessuna14-05/P15 o altra task senza nuova ripresa.
Source utente in schemas/xlsx e modifica preesistente alla proposta reference
preservati ed esclusi dai commit. Nessun Superpowers/subagente utilizzato.
