# Scope e parità obbligatoria della nuova UI

T-MIG-P0-04,11 novembre2026. Decision owner: autore Lintra; scope D01/D02 e
DEC-MIG-07 ricevuti il10 novembre, recepiti in ADR-POST-M9-001 e
[ADR-MIG-001](../adr/ADR-MIG-001.md). **Scope approvato, implementazione UI
ancora NOT_VERIFIED**. Baseline codicecb70bee, CI42990775714 success;
Slint resta operativo fino a cutover e rollback qualificati.

## Matrice di scope e stato reale

| Capacità | Stato reale alla baseline | Consegna obbligatoria / percorso |
| --- | --- | --- |
| Catalogo flow/version/trace e bundle | Rust registry/catalog e Slint attuali; nessun catalogo React | MIG-P2/P4/P5, selezione esplicita e provenienza; mai latest implicito |
| Input singolo e preview | Slint prepara Trace/LocalOnly; selettore corrente UTF-8 | Parità del percorso esistente, broker file Rust e preview bounded; niente file intero in JS |
| LocalOnly | ValidationService e CLI qualificati sui profili compilabili | Obbligatorio in React; dichiarare solo controlli locali supportati, mai conformità globale |
| Full multi-tracciato | ValidationService::validate_with_parameters/typed_parameters e CLI validate-flow reali; non presente in Slint | Obbligatorio prima del cutover: P2-11, P4-11/12, P5-09/10, P6-10, P7-09, P9-08 |
| Parametri Full | Rust verifica nomi/tipi/range/namespace e schema compilato | Form esplicita da metadata Rust; TS non deduce ULSS, periodo o regole dal filename |
| Due passate e progresso | Engine reale, ordine/cancel/max_errors qualificati | Monitor di passata/tracciato e terminale affidabile; niente percentuali inventate |
| Risultati e cinque filtri AND | JSONL lazy, query/offset/cache bounded Rust | Paging Rust e virtualizzazione TanStack; nessun array completo/indice clinico nel renderer |
| Dettaglio e guida campi/domìni | Rust model/field guide esistenti | Parità con detail bounded e domìni paginati; tipo, posizioni, larghezza, fill/allineamento reali |
| Export JSONL/CSV/XLSX e summary | Writer/backend canonici e receipt già qualificati | MIG export e U-OPS-04: locale libero, shared con warning/consenso, overwrite e errori parziali espliciti |
| Soft budget | Preflight Rust/CLI con conferma legata ai file/richiesta | P5-05: Annulla/Procedi comunque; hard non aggirabili, nessun taglio record/hash |
| Namespace/Unknown/Partial | Contratti Rust qualificati | Stato e causa visibili, distinti da errore tecnico/violazione; nessuna coercizione in Valid |
| SIOC D11 | Nuovo profilo cross-run.1 e producer Rust qualificati; tutte4 PK UDO conservate | U-NORM-04/bridge dopo G-MIG-4: UI effettiva, cumulato e contesto amministrativo approvato distinto dai nuovi invii |
| CAD83/C1/L1 e storico | Snapshot service Rust e SQLite qualificati; nessuna schermata corrente | U-UI-02 → U-CAD-01..09: namespace, T1/T2/T3, consultazione, review/approve CAS reali |
| Setup dati per account | Controlli backend privati già qualificati; adapter Tauri assente | MIG-P2-08/P7-03 e U-OPS-03; root/spool/cache effettivi, nessun DB in Program Files |
| Backup/restore e recovery | Procedure/capacità parziali dichiarate, nessun wizard implicito | U-OPS-06/08 e U-REL-03: BACKUP-B e G-DUR separati, convergenza in G-REL |
| Installer/update/uninstall | Backend native CI, nessun installer Tauri qualificato | Windows11 NSIS offline/perMachine, WebView2 offline, non-admin/due account; gate P1/P8/P9 |

La matrice è il recepimento dello scope approvato, non una dichiarazione che il
frontend abbia già queste funzioni. Le prove Slint/CLI/backend non diventano
prove React/WebView. Ogni task mantiene le proprie acceptance native.

## Profili compilabili e instradamento

Il frontend riceve da Rust capability e motivi di disponibilità. Una modalità
inclusa nello scope non rende ogni profilo automaticamente eseguibile in quella
modalità. Lo schema/versione deve compilare con il producer corretto; nessuna
capability viene eliminata per mostrare un pulsante abilitato.

- La validazione ordinaria usa ValidationService e i parametri verificati.
  Full richiede input per tracciato espliciti e perimetro/schema reale; più
  invocazioni LocalOnly non simulano Full o join cross-trace.
- I profili snapshot CAD richiedono HistoricalValidationService::validate_snapshot
  e contesto effettivo. Il percorso storico e l'approval non vengono esposti
  dalle89 MIG prima delle task U-CAD previste. Nessun validatore alternativo JS.
- I profili normativi cross-run richiedono il producer storico Rust esplicito.
  LocalOnly/Full ordinari non devono ignorare la policy. La UI storica userà
  outcome, review e stato reali; una preview non approvata non è baseline.
- SIOC cross-run.1 aggiunge D11 al componente referenziale qualificato di26 campi
  fisici/sei controlli, non all'intero workbook135 campi. I deferred/dataset fissi
  rimangono dichiarati. CAD83 non diventa FROZEN per un cambio di frontend.
- Se il contesto richiesto manca o non è qualificato: errore tecnico/Unknown/
  Partial secondo Rust. Un amministrativo già approvato non si trasforma in
  nuova trasmissione4.1/4.2 per superare un join. Niente falsa approval.

## Criteri vincolanti di parità

Una prova LocalOnly e una Full confrontano invocazione diretta Rust e adapter
su identici input/opzioni/parametri/schema: esito, completezza, contatori,
ordine/errori JSONL e audit effettivi. Esito normativo e completezza sono due
assi: CompletedValid locale o Partial non è conformità globale. Cancelled,
truncated, sorgente/reference non verificata e fallo di report non diventano
un completamento riuscito. Un errore tecnico non è una violazione normativa.

Run/report ID e possesso backend impediscono eventi tardivi da una vecchia
analisi, secondi start mentre un job è attivo e uso di report non pubblicati.
Questi contratti saranno definiti/verificati inP2/P4; non sono API già esistenti
nel nuovo frontend. Una sola validazione attiva per istanza, produzione
sequenziale; nessuna DEC-14 o parallelizzazione automatica.

Filtri/query/paging, report ed eventuale storico restano Rust. Il renderer
presenta solo le pagine/dettagli richiesti, mantiene ordine canonico e non
riscrive diagnostici/export. Sorting interattivo e installazione arbitraria di
nuovi bundle non sono aggiunte implicite alla migrazione. Informazioni espone
versioni, provenienza e attribuzioni verificate degli asset effettivamente usati.

Redesign responsive con prove per OS, keyboard/focus/a11y e assenza di overflow
incontrollato. Nessuna regressione viene nascosta aggiornando golden legacy o
allargando tolleranze. I budget e il brief visivo sono inP0-07;300k/1M sono
stress, distinti dai volumi realistici decisionali.

Il cutover MM9 richiede LocalOnly+Full qualificati e rollback. La distribuzione
ai colleghi richiede inoltre G-CADUI3/G-SIOC1/G-OPS2/G-DUR e G-REL con signoff
previsto: né questa matrice né G-MIG-0 autorizzano pubblicazione/tag/release.
