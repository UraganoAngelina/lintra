# T-CAD-04 — preflight, 6 novembre 2026

Stato: READY → PREFLIGHT. CAD03 ePERSIST08 DONE; CAD09 ha tutti gli11gate
locali positivi e attende soltanto le riaperture crossOS della CI42231182241.
La task04 è indipendente da09. Sviluppo sequenziale, un solo agente.

## Fonti verificate

Prompt04; ARCH45; DET166–174; ADR-PERSIST-001/PERSISTENCE_CORE_SPEC; piano14/20;
authoring persistence_policy; CAD_NOTES_AUDIT, note2.36/3.5/4.11/5.7 efixture13;
componente namespace02 conPK normative complete, invariantiRaw/Compiled e codice
reale di protection/repository/approval/history_audit/historical_validation.
La ratificaT-P0-07 ha già formalizzato history_identity_invariants. Non si apre
una nuova norma clinica: ULSS e trace isolano i progressivi; il legame all'evento
resta immutabile anche dopo annullamento, mentre gli altri attributi modificano
il record. Lo stesso evento può essere riattivato. Nessuna selezione first().

## Stato reale del codice

HistoricalAuditService calcola già inseriti, aggiornati, scomparsi e riattivati,
con fonti approvate e query storiche dopo riaperturaSQLite. Il confronto corrente
è esplicitamente sullaPK record_key. Non va reinterpretato silenziosamente come
un confronto sul solo progressivo: lePK completeC/D comprendono anche altri campi.

RawInvariant eHistoricalIdentityInvariant già descrivono progressivo/evento,
codice di riassegnazione e flag di riattivazione. compile_persistence_profile
rifiuta ancora tutti questi piani. Il produttoreHistoricalValidationService
supporta solo audit senza snapshot/invarianti; non va dichiarato FullCAD o
abilitato eliminando policy/capability per aggirare il compiler.

PersistentRecordIdentity comprende già key_kind e il protocollo HMAC lo separa
nel contesto. StoredIdentity accetta solo digest e fingerprint protetti. SQLite
indicizza l'identità completa e mantiene l'ultima occorrenza anche se la chiave
non compare nello snapshot corrente. Questi contratti generici sono una base
per indici progressivo/evento senza salvare identificativi in chiaro. Prima di
scrivere codice va verificata l'intera catena: produzione delle proiezioni,
conteggi fisici distinti dagli indici, validazione dei token, diff e approvazione.
Nessun cambiamento del formato storage/HMAC è finora necessario o implementato.

## Direzione di implementazione da completare

Compilare i riferimenti delle invarianti in piani tipizzati e circoscritti.
Riutilizzare l'estrazione canonica delle chiavi e i token protetti; il vincolo
non può basarsi sull'hash dellaPK completa, che cambia se cambia l'evento.
Tenere distinti l'indice del progressivo, il suo evento e il fingerprint degli
attributi. I diff sul progressivo devono essere espliciti e lasciare invariato
il significato dell'audit record_key preesistente.

Il percorso di approvazione deve rifiutare proiezioni mancanti/incoerenti e la
riassegnazione prima di qualsiasi commit, con CAS/cancellazione/report invariati.
Gli errori normativi devono restare distinti da dati di storage non verificabili;
non inventare numeri di riga a partire da digest. Un produttore di proiezioni
coerenti deve conservare l'origine diagnostica senza persistere dati sensibili.
I piani temporali non implementati restano rifiutati. Nessuna compatibilità tra
versioni dedotta da etichette o soli hashPK, nessuna migrazione implicita.

File da valutare: schema/persistence_profile e metadati di proiezione; nuovo modulo
applicativo generico per invarianti/diff sui progressivi; session e audit nei soli
punti necessari; componenteCAD versionato e tests/cad_progressives.rs. Nessun nuovo
crate, driverCAD, chiave operativa, input reale, CLI/UI o schemaFROZEN modificato.
Il disegno dei token e delle guardie di approvazione è ancora in verifica; nessuna
nuova API o correttezza end-to-end viene dichiarata da questo preflight.

## Accettazione

Fixture sinteticheT1/T2/T3, reale SQLite e riaperture: nuovo/modificato/annullato/
riattivato, stesso progressivo su evento diverso rifiutato anche dopo annullamento;
stesso progressivo separato fraB1/B2/C/D eULSS; altri attributi modificabili,
PKC/D complete; annullamenti solo su file presente, mai su file assente.
Provare esiti deterministici, provenienza/integrità, token incompleti, generazione
concorrente, cancellazione e reportfailure. Scope di componente distinto dalla
validazione clinicaCAD completa, che resta al gate12. Locali eCI reali primaDONE.
