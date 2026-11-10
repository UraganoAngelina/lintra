# U-FIX-01 — Conservazione delle regressioni SQLite

Data: 2026-11-10, Europe/Rome. Prerequisito U-GOV-01 `8e6bc55`.
Stato: **DONE**. Locale e CI nativa nuovi verificati.

## Contratto e prova

Sorgenti di prodotto invariati rispetto ad af3bccc. Tutti i 498 input
dell'inventario qualified-inputs dell'audit corrispondono byte per byte.
Nessun refactoring, nessuna modifica SQLite, HMAC, codec, snapshot o golden.

Regressioni conservate in src/persistence/sqlite/tests.rs:

- AUD-M9-001: audit_generation_read_is_coherent_across_concurrent_publication.
  L'authorizer pubblica da una seconda connessione nella finestra critica;
  current_generation deve leggere una sola vista coerente, senza race a tempo.
- AUD-M9-002: audit_identity_lookup_rejects_dangling_latest. Una cancellazione
  esplicita dei records lascia latest dangling: atteso CorruptStorage, mai None.

La prova RED originale resta in post-m9-audit/storage-red.json e relativo log:
due fallimenti attesi prima del fix; non viene presentata come nuova esecuzione.
La prova GREEN nuova su **4c169cdde7ef126475327ed0ea5bcadf0693395a** è
[U-FIX-01-sqlite-linux.json](U-FIX-01-sqlite-linux.json) e log .log.gz:
`cargo test -p lintra --lib --locked --offline persistence::sqlite::tests:: -- --test-threads=1`
exit0, 38 passed, zero failed, quattro ignored separati. Include restart, crash,
CAS/contesa, recovery, integrità e entrambe le regressioni audit effettivamente ok.

## CI e handoff

[Nuova CI42823787199](https://github.com/UraganoAngelina/lintra/actions/runs/42823787199)
avviata sul branch pubblicato a 4c169cd; esito nativo ancora da leggere.
CI42789338814 è soltanto baseline storica e non sostituisce questo gate.
Il delta della task è dossier/JSON/log/stato, non codice di prodotto.
G-HARD aggregato ancora NOT_VERIFIED. SHA dossier nella cronologia Git.
In attesa remota proseguire sequenzialmente con U-FIX-02 indipendente.

## Chiusura nativa

CI42823787199 completed/success sullo SHA esatto 4c169cd: sette job richiesti
positivi, packaging UI opzionale skipped. Metadata in U-FIX-01-native-final.json;
log completo gzip e digest in U-FIX-01-native-log-check.json. Entrambe le
regressioni risultano effettivamente passate almeno tre volte per OS
(standard/all-features/core senza bundle), non solo dedotte dal job verde.
I due import reciproci SQLite sono positivi. Nessun nuovo delta SQLite.
La CI qualifica questa baseline; il successivo codice budget 4864c34 richiede
una propria CI. G-HARD aggregato resta NOT_VERIFIED. Next READY U-FIX-05.
