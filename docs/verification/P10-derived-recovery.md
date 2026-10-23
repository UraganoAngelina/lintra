# Rettifica dello stato P10 — contratto esteso già ratificato

**Aggiornamento 2026-10-23:** residui 10-01/03/04/05 DONE con gate nativi;
10-06 DONE, CI 39920597392 su c2e0d1d workflow/job Linux e Windows
completed/success. P10 estesa chiusa dopo verifica, 458 test/55 suite.
T-P11-02 completata e verificata durante l'attesa indipendente dei gate P10
(CNF-02); 11-03 richiede la decisione separata sul formato filesystem.
Nessuna nuova decisione sul contratto dei derivati. Il testo seguente conserva
la rilevazione originaria, senza presentarla come stato implementativo attuale.

## Registro storico — rilevazione e piano di recupero

2026-10-22. Nessuna decisione nuova richiesta o riaperta.

La revisione dell'authoring per l'avvio P11 ha rilevato una consegna P10
approvata ma non eseguita. SCHEMA_AUTHORING.md, «Adeguamenti delle task e
accettazione», assegna a P10 i tre derivati e lowering const/derived_eq/
derived_compare. La sezione «Valori derivati approvati» ne definisce i contratti.
DECISIONS.md T-P0-07 registra l'approvazione completa del commit 72f9f5e,
comprensiva degli adeguamenti delle task future. Non è una proposta pendente.

Le task P10 sono state eseguite contro il nucleo dei prompt atomici originali:
Eq/Ne/In/NotIn/presenza/booleani, regole locali non derivate, golden e controlli
statici. Quel codice e i gate locali/nativi sono reali e restano validi.
Tuttavia il compiler rifiuta ancora derived_values/derived_eq/derived_compare,
le capability derived restano escluse e il runtime restituisce UnsupportedPlan
per questi piani. Perciò il nucleo è verificato, la fase estesa NON è conclusa.
Le precedenti diciture di chiusura P10 erano troppo ampie e sono rettificate.

## Residuo autorizzato e collocazione nelle task esistenti

- T-P10-01: lowering dei tre RawDerived in piani tipizzati, riferimenti locali,
  range substring verificati e dipendenze derived_eq/compare risolte.
- T-P10-03: valutazione read-only Substring/AgeInCompletedYears/CalendarYearDelta
  e dei nodi Expr derivati, con Unknown per input mancanti/invalidi; nessun
  default per leap_day_anniversary, niente float o data inventata.
- T-P10-04: integrazione reale del contesto con derivati per presence, senza
  eliminare gli obblighi né cambiare gli esiti di run già conservati.
- T-P10-05: assertion DerivedCompare e diagnostica del contesto effettivo,
  mantenendo il rifiuto esplicito delle funzionalità cross-record future.
- T-P10-06: casi limite, compatibilità tipi/range, dipendenze/ref mancanti,
  operatori e golden dei percorsi derivati, nuovi gate Linux/Windows.

Queste task tornano IN_PROGRESS nel perimetro del contratto esteso; le loro
consegne base precedenti rimangono evidenza, non vengono ricostruite da zero.
T-P10-02 resta DONE: l'algebra a tre valori e tutti i motivi sono già verificati.
Non si crea un nuovo task ID, non si cambia il Master Plan o una baseline:
si applica l'allineamento ratificato dall'autore nel contratto T-P0-07.

## Punto di ripresa

T-P11-01 è già implementata e nei gate nativi: completarne le verifiche, perché
il suo prerequisito è T-P7-05 e non la chiusura P10. Poi completare il residuo
P10 appena identificato; riprendere T-P11-02 dopo la chiusura corretta di P10.
Nessun provider fittizio o filesystem anticipato, nessun reset di codice valido.
M4/M5/PERSIST/P13 e T-P2-03/M1 rimangono aperti. Non ci sono nuovi conflitti
contrattuali da sottoporre: gli executor mancanti sono già autorizzati.
