# U-NORM-04 — Golden cross-run e integrazione da completare

Data11 novembre2026. Stato: **WAITING_INTEGRATION**, non DONE.
Prerequisito U-NORM-03 DONE backend, commit di handoff a89c3f5.
Codice qualificato cb70bee, CI42990775714 success7 job; inventario520 input
invariati dopo i soli commit documentali. G-SIOC1 aggregato NOT_VERIFIED.

## Prove già realizzate e qualificate

Non duplicare i test consegnati in03: `tests/cross_run_validation.rs` contiene
21 test pubblici eseguiti63 volte per OS complessivamente (21 per ciascuna
configurazione standard, all-features, core senza bundle). Il dossier03
contiene la matrice dei16 scenari ratificati, file e condizioni; la prova
macchina U-NORM-03-native-42990775714.json verifica ogni nome in ogni step.

Il golden indipendente `tests/golden/sioc-cross-run/v1/expected.errors.jsonl`
è confrontato byte per byte dal writer canonico effettivo. La famiglia ha un
producer reale dedicato, registrato nel discovery senza permesso di update
single-reader. Nessun golden legacy modificato. Completata la regressione
locale workspace/all-features:1096pass, zero fail,7ignored espliciti; la CI
include regressione nativa, riapertura SQLite opposta e confronto JSONL.

| Caso richiesto | Esito backend effettivo |
| --- | --- |
| Gennaio/Febbraio/Marzo | Approvazioni reali; ricoperture identiche ammesse e nuovi conteggiati una volta |
| PK pregressa diversa | Violazione in seconda passata; caso anche su profilo SIOC vero con join skipped fedeli |
| Duplicato intra-file | Regola preservata;4.1 scarto/emissione in seconda passata con budget/cancel |
| Omissione | Partial/ValidationOnly, nessuna cancellazione dello storico |
| Baseline carente | Unknown o errore tecnico secondo causa; gennaio non vuoto senza approvazione iniziale non è NEW |
| Namespace/tracciati | PK identiche isolate per ambito e tracciato |
| SQLite/replay | Reopen reale, idempotenza e revisione invariata; più ricoperture senza nuove acquisizioni |
| Report/CAS fallito | Nessuna modifica della baseline perdente |
| Legacy | Snapshot leggibile invariato; confronto non qualificato Unknown, niente conversione silenziosa |
| Contenuto canonico | HMAC esistente su riga fixed-width qualificata, esclusi solo LF/CRLF |
| Determinismo | Oracle indipendente uguale su Linux/MSVC in tre configurazioni |
| PK UDO | Ciascuno dei4 campi discrimina; stesso composto fra invii diversi rifiutato |
| CLI e confini | CLI subprocess usa il vero servizio normativo; il public draft non promuove audit a norma |

L'assenza non viene reinterpretata come annullamento SIOC. Non esiste una
procedura di rettifica/annullamento nuova implicitamente approvata: la policy
non abilita snapshot/temporal/invarianti CAD per ottenere quel comportamento.
Le regole CAD e le decisioni C1/L1 sono rimaste nelle regressioni native.

## Residuo materiale e prossimo percorso

Il frontend React/Tauri non esiste ancora; quindi non sono qualificati UI,
consultazione, preview/review/approve, né una presentazione globale conforme.
Il componente SIOC conserva join referenziali current-run: un amministrativo
4.1/4.2 già approvato non può essere ritrasmesso come nuovo invio per superare
il join. Va collegato nel servizio Rust un contesto storico di consultazione
qualificato e distinto dalle nuove trasmissioni, prima di dimostrare il caso
operativo completo con i soli tracciati mensili1–3. Fino ad allora contesto
mancante significa Partial/ValidationOnly. Nessuna eccezione alla PK ratificata
viene introdotta e nessun flag audit-only abilita l'approval.

Questo residuo passa all'integrazione applicativa prevista dopo G-MIG-4,
come master §8/§10. Non viene chiusa04 o G-SIOC1 sulla sola copertura di unit
ed engine: per G-REL serve il flusso realmente usabile. Non si inventano nuovi
ID o un arco che blocchi la migrazione sintetica. D11 e4.19.1 restano approvati;
nessuna nuova ratifica ordinaria richiesta. Next READY T-MIG-P0-02.
