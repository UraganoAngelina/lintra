# Chiarezza dei diagnostici — requisito dell'autore, 2026-10-29

Stato corrente: implementato in T-P16-07 e verificato con prove mirate core/UI;
regressione completa e gate nativi in qualificazione, requisito non ancora chiuso.
Evidenze: docs/verification/T-P16-07-diagnostics.md e local-gates.json.
Integra la qualità della schermata Risultati; non modifica le norme degli schemi,
le decisioni di CNF-ALIGNMENT-01 né lo stato delle consegne16-00…05 già qualificate.

L'autore richiede errori estremamente chiari, senza descrizioni vaghe: se il
valore non appartiene al dominio, deve conoscere il dominio effettivo; se il
dominio è un range, vuole tutti i valori ammessi. Per i tipi numerico e
alfanumerico devono essere spiegati formato, allineamento e riempimento dello
slot fisico quando il contenuto non occupa tutta la larghezza.

## Criteri di accettazione

- Il dettaglio identifica tracciato, riga, campo/nome e posizioni iniziale/finale
  in byte, valore effettivo e regola violata. Spazi/fill e caratteri non stampabili
  devono essere distinguibili; non limitarsi a «tipo non valido» o «fuori dominio».
- Dominio enumerato: valori ammessi completi, con zeri, case, spazi e larghezze
  preservati secondo target raw/logical. Non suggerire valori normalizzati che
  il validatore rifiuterebbe. Le eventuali descrizioni provengono dallo schema
  o reference qualificato, non sono inventate dal renderer.
- Range discreto: estremi e inclusività espliciti e tutti i valori consultabili.
  Un insieme grande richiede consultazione progressiva/virtualizzata senza
  taglio silenzioso: non duplicare milioni di valori in ogni errore o bloccare
  thread UI/validazione. Distinguere i valori numerici dalla loro rappresentazione
  fisica. Per date/importi precisare anche formato, calendario/precisione e passo
  ammesso; non promettere una lista finita per intervalli non discreti.
- Tipo: indicare caratteri/formato effettivamente ammessi dal piano compilato.
  Numeric non ammette automaticamente segni, virgole, Unicode numerici o spazi.
  Alphanumeric NON significa sole lettere: una sequenza di cifre può essere valida.
  Non introdurre un errore per la sua sola presenza in un campo AN.
- Rappresentazione: larghezza, lato del contenuto e lato/byte del riempimento,
  oltre a preserve/exact/strip quando serve a spiegare il confronto. Esempi
  esplicitamente sintetici e coerenti con quella specifica dichiarazione.
  Non assumere universalmente numeri a destra con zeri o testo a sinistra con
  spazi: mostrare la configurazione reale di quello schema.
- Separare causa accertata e suggerimento: una violazione del dominio non prova
  che il campo sia spostato o allineato male. Non riattivare il validator fisico
  escluso da CNF-ALIGNMENT-01, né introdurre E_ALIGNMENT implicito.
- Messaggi e valori attesi devono provenire dal core comune. UI, JSONL, CSV/XLSX
  e CLI devono essere coerenti; niente nuova implementazione di regole nella UI.
  Il dettaglio completo non deve essere nascosto dal riassunto breve della riga.

## Gap osservati e file interessati

`src/validators/domain.rs` produce oggi «uno dei valori del dominio dichiarato»;
`src/engine/error_factory.rs` costruisce «valore fuori dominio»/«tipo di dato non
valido» senza descrizione completa. Alcune classi lessicali hanno atteso generico.
`ui/src/results/diagnostic.rs` legge già actual/expected/message dal JSONL:
la UI non deve ricostruire norme dal nome del campo. Interventi previsti su
arricchimento core, disponibilità del dominio nel dettaglio e presentazione UI,
con compatibilità del wire e golden tecnici esplicitamente verificata.

Test da aggiungere: dominio enumerato completo con raw/logical e fill; range
inclusivo con estremi e valori interni; dominio grande senza perdita di valori
né memoria proporzionale alla cardinalità nel renderer; numerico con lettera,
segno/spazio non ammesso, AN con sole cifre lecito; entrambi i lati di padding,
exact/preserve, stringhe UTF-8 e byte non stampabili. Coerenza CLI/UI/export,
leggibilità a1×/2× e disponibilità del dettaglio senza troncamento.

## Stato UI verificato alla richiesta

16-00…05 DONE (6/9 task), evidenze nel ledger.16-06 export/Informazioni e16-07
collaudo visivo finale non iniziate ma con prerequisiti soddisfatti;16-08 packaging
attende16-07. Le compare pixel già verdi sono regressioni intermedie, non la
chiusura del gate visivo finale né della milestoneM8. Questo nuovo requisito
rimane aperto fino a implementazione e prove reali; non è soddisfatto da questo
solo documento. La task attiva di sviluppo resta PERSIST07 dopo06DONE.
