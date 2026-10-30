# T-PERSIST-08 — preflight

2026-10-30, avvio dopo T-P16-08/M8 (CI41036453232, tutti7job success).
Prerequisiti reali: PERSIST07/CI40877202683 e P13/CI40192491629 positivi.
Fonte: prompt08, ADR001, ARCH45, DET166–174, piano20 e contratti ratificati.
Git: work/p1-foundations, base fe5ada8 pubblicata. Single-agent; nessuna UI,
nuova regola SIOC/CAD, nuova crate o modifica a storage/durabilità.

## Perimetro e API previste

Servizio applicativo di audit read-only in application/history_audit; dominio
HistoricalDiff/HistoricalChange già esistente, re-export lib, test sintetici.
Port additivo per una singola identità con provenienza e revisione dell'ultima
occorrenza; default fallisce esplicitamente per adapter terzi non aggiornati.
SQLite usa gli indici latest e le PK esistenti con join puntuali: nessun indice,
formato o migrazione nuovi. Fake aggiornato per la sola prova dei contratti.

L'audit fissa la generazione del namespace, scorre metadati paginati e seleziona
la più recente baseline presente per ciascun trace dichiarato cross_run_audit.
Flussi distinti non si confrontano; versione/hash/scope devono corrispondere al
profilo compilato o ai membri della compatibilità esplicitamente verificata.
Un trace assente non prova la scomparsa delle sue identità; un file presente
vuoto è invece confrontabile. Nessuna assenza locale prova lo storico completo
dell'organizzazione. Metadati espliciti per baseline e copertura locale.

Prima delle letture storiche e della conclusione verificare generazione fissata:
un avanzamento produce conflitto tecnico, mai diff apparentemente coerente.
Ogni identità/source viene verificata con il protector approvato prima dell'uso;
lookup con provenienza evita di attribuire un token a un invio diverso o futuro.
Diff emessi uno alla volta in ordinamento esplicito (nessun ordine HashMap),
con summary solo al successo. Errori sink/cancel/storage restano tecnici.
Nessuna chiamata begin/stage/commit nel servizio di audit.

## Memoria e prove

load_snapshot restituisce oggetti posseduti: costo proporzionale alle baseline
selezionate, mai tutte le identità di tutto lo storico. Indici ordinati leggeri
su candidato/baseline; nessun Vec globale di diff e lookup puntuali dello storico.
Il limite dell'API owned rimane documentato, senza presentarla come streaming DB.
Test: più trimestri/trace/flussi, inserito/modificato/scomparso/riattivato/immutato,
tracciati assenti e presenti vuoti, namespace/versioni/hash/chiavi incompatibili,
concorrenza durante la lettura, ordine deterministico, cancellazione/sink fallito,
nessuna mutazione, SQLite reale/restart e query plan. Solo fixture sintetiche.
Qualifica: fmt, Clippy, workspace default/allfeatures, core-no-bundle, boundary,
tooling quando pertinente, CI Linux/MSVC e confronti. Non DONE prima dei gate.
