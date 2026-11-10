# U-NORM-01 — PK SIOC e perimetro vigente

Data: 2026-11-10, Europe/Rome. Prerequisito U-GOV-03 `2cd4bdf`.
Stato: DONE per inventario. G-SIOC0/1 NOT_VERIFIED: norma operativa nuova e
enforcement devono ancora essere completati in U-NORM-02/05/03/04.

## PK esatte verificate

| Trace | PK ordinata | Campi / byte record |
|---|---|---|
| 1 | 1.1, 1.17, 1.18 | 18 / 206 |
| 2 | 2.1, 2.49, 2.50 | 50 / 271 |
| 3 | 3.1, 3.2, 3.35, 3.36 | 36 / 135 |
| 4.1 | 4.1, 4.7, 4.8, 4.19.1 | 20 / 394 |
| 4.2 | 4.20, 4.22 | 11 / 67 |

Tupla, ordine, uniqueness_scope=trace e E_DUPLICATE_RECORD_KEY identici in
FROZEN, structural-minimal.1/.2 e references-golden.1. Confronto automatico
effettivo, non dedotto da una tabella storica. Dettagli di rappresentazione,
allineamento e nomi per ogni campo chiave in U-NORM-01-inventory.json: nessun
trim globale, cambio codec o chiave alternativa viene introdotto.

Namespace legacy: organisation_namespace intero positivo 1..4294967295,
esplicito, tecnico e stabile per la stessa ULSS; non è autenticazione né un
nuovo codice clinico. Compatibilità legacy same_schema_version. Tutti e cinque
i record_key partecipano a run_uniqueness/cross_run_audit; snapshot=null e
history_identity_invariants vuoto. Questi fatti descrivono **il profilo legacy**,
non riportano D11 ad audit-only: il nuovo vincolo è DEC-SIOC-HIST-02.

## Capacità attive ed esclusioni

La baseline minimale conserva layout, rappresentazione, presenza e PK; .2
aggiunge le quattro relazioni referenziali ratificate G0 e conserva i rinvii
di .1. Ledger ereditato: constraints83 deferred,29 active,8 physical_only,
3 partial_050_only; rules70 deferred/2 active; tre gruppi deferred.
L'inventario JSON identifica **tutti gli 83/70/3 elementi deferred** con
trace/field/index/ID e digest della dichiarazione, senza attivarli.
I rinvii non diventano regole implementate perché le primitive del runtime
sono nel frattempo aumentate. D12 vieta nuovi sourcing e completamenti inventati.

Il profilo .2 contiene 135 campi, zero binding reference runtime per scelta di
scope e sei capability richieste. Il componente referenziale conserva 26 campi
fisici (20 selezionati + sei span opachi),115 campi sorgente esclusi e sei
controlli selezionati: è un componente qualificato, non tutto SIOC.
La fixture audit-synthetic aggiunge la policy persistence al componente, senza
trasformare il profilo sorgente completo in compilabile o conforme.

## Fonte normativa disponibile e limiti

Workbook ricevuto SHA256 `6be8ae60...` completo nel JSON; foglio
«Tracciato_SIOC definitivo», celle H8/H29/H82 per1.1/2.1/3.1. Dice che il numero
deve essere univoco per struttura e non ripetersi nel corso dell'anno, con
codifica per ingressi2026 e conservazione dell'ID per episodi2025 conclusi2026.
Le celle e i marker di chiave sono inventariati direttamente dall'XLSX.

Questa nota è prova indipendente di quella clausola su ID episodio; **non**
dimostra da sola il divieto di ogni nuova trasmissione della stessa PK su tutti
i cinque trace, né eccezioni di rettifica/reinvio. D11 resta mandato vincolante
di enforcement versionato: U-NORM-05 distinguerà fonte e semantica approvata,
isolando soltanto eventuali vere ambiguità. Nessuna eccezione o reset annuo
del registro viene inventato qui. Gli snapshot conservati non sono nuovi invii.

## Prova e delta

Su10dc66b: test esistente source_pk_tuples_are_exact_and_compiler_does_not_enable_history_implicitly,
all-features,1 pass/zero fail. Comando/digest/log in U-NORM-01-pk-oracle-linux.
Conferma PK dichiarate, producer storico esplicito e rifiuto persistente del
profilo sorgente completo, senza rimuovere capability per ottenere un verde.
Nessuna modifica a schemi, ledger, norme, dataset, codice o golden legacy.
Solo inventario, dossier/log e stato: nessuna CI nuova necessaria/attribuita.
Next READY U-NORM-02, poi U-NORM-05; AUTO_CONTINUE.
