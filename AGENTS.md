# Istruzioni operative Lintra

Mandato dell'autore aggiornato il 2026-10-19:

- Non usare le skill Superpowers o i relativi workflow.
- Procedere direttamente, una task atomica alla volta, verificandone il risultato.
- Avanzare senza conferme ordinarie quando prerequisiti ed esito sono chiari.
- Fermare l'avanzamento interessato in presenza di ambiguità o decisioni mancanti.
- Per questa modalità lavorare senza subagenti; non avviare processi in background.
- Rispettare ARCH, DET, Master Plan e prompt atomici dello starter kit;
  il nuovo mandato sostituisce il precedente flusso di orchestrazione.
- Registrare stato, evidenze reali e handoff in `IMPLEMENTATION_STATUS.md`.
- I file di pianificazione e orchestrazione precedenti sono materiale storico.
- Mandato successivo dell'autore: dopo una task verificata, iniziare subito
  la successiva se prerequisiti ed esito sono chiari. Questo mandato prevale
  sullo stop tra task contenuto nei prompt atomici; restano i blocchi reali.
- Alla fine di ogni task comunicare un breve recap dei file creati/modificati.

Mandato specifico successivo — CNF-AUDIT-01 approvata:
- Implementare T-P5-05 con le precisazioni dell’autore su EOF, sorgente
  autorevole degli audit, overflow, cancellazione e test.
- Fermarsi al termine di T-P5-05 e attendere autorizzazione prima della
  task successiva; questo mandato supera la progressione autonoma generale.

Mandato successivo dell'autore — 2026-10-20:
- Autorizzata la prossima task, T-P5-06, dopo la consegna di T-P5-05.
- Regola vincolante: «OGNI QUAL VOLTA SCRIVI DEL CODICE DEVI ESPRIMERE
  DETTAGLIATAMENTE COSA FA E COME MAI FUNZIONA, CON DEL CODICE INTENDO
  STRUCT FUNZIONI IMPLEMENTAZIONI E CONDIZIONI».
- Accompagnare quindi il codice nuovo o modificato con spiegazioni del
  comportamento e delle ragioni di correttezza: documentazione/commenti
  pertinenti, spiegazione tecnica consultabile e recap finale dei file.

Mandato più recente — ripresa del workflow autonomo, 2026-10-20:
- Dopo T-P5-07, procedere da T-P6-02 e iniziare autonomamente le task successive
  quando la precedente è verificata e prerequisiti/contratti non presentano blocchi.
- Questo mandato supera gli stop ordinari dei prompt e lo stop storico dopo
  T-P5-05. Restano vincolanti i gate e le decisioni realmente mancanti.
- Restano esclusi Superpowers, subagenti e processi locali in background;
  mantenere spiegazioni dettagliate del codice e recap dei file per ogni task.
