# DEC-RELEASE-SETUP-01 — ratifica A

2026-10-26. L’autore sceglie esplicitamente «A: entrambi privati, baseline proposta»
in risposta alla domanda relativa a DEC-RELEASE-SETUP-01. È ratificato il contratto
A della [proposta](../proposals/DEC-RELEASE-SETUP-01.md), inclusi:

- DEC-08: lintra-binaries e lintra-data privati, creazione se assenti e bootstrap
  README autorizzati; sorgente e sua visibilità invariati, niente inviti o
  pubblicazione di dati regionali non approvata.
- DEC-09: PAT fine-grained dedicato a lintra-binaries, Contents write, metadata
  read, scadenza90giorni; secret sorgente LINTRA_RELEASE_TOKEN configurato
  direttamente dall’autore in GitHub. Vietato copiare la credenziale locale
  dell’agente. L’autore ne conferma la configurazione; API Actions verifica il nome presente.
  Permessi effettivi verificabili soltanto con la pubblicazione CI reale.
- DEC-11: GNU su runner ubuntu-24.04 con glibc della baseline documentata in audit;
  Linux solo interno/personale; Windows MSVC/CRT statica e baseline già definita
  invariati. Allowlist e gate nativi/riproducibilità non indeboliti.
- Release di prova esplicita solo CI, destinazioni private approvate, nessuna
  sovrascrittura di tag/versioni/release o firma Authenticode. Niente DONE15-05
  senza prova realmente pubblicata e verificata.

Autorizzata ripresa T-P15-05: workflow, packaging e controlli verificabili prima
  della configurazione finale del secret. Il contratto è risolto; la disponibilità
  della credenziale rimane un prerequisito tecnico della prova di pubblicazione.
Nessuna modifica a ARCH/DET, baseline normative o contratti runtime.
