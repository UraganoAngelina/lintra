# T-MIG-P6-01 — DataGrid DOM virtualizzata paginata

Data 24 novembre 2026. Stato: **DONE**. Review Codex round 1 CHANGES
(P601-01), round 2 APPROVED su `2a6d836` senza nuovi rilievi
(`lintra-review-handoffs/T-MIG-P6-01.review-2.md`). Integrazione autorizzata
dall'autore per fast-forward di `integration`, senza push. CI remota e
Windows/WebView2 NOT_RUN, al checkpoint del blocco P6; G-MIG-6 NOT_VERIFIED.
Prerequisito
T-MIG-P5-08 DONE (`integration` `3a91a11`, G-MIG-5 PASS). Branch
`work/T-MIG-P6-01` da `3a91a11`, worktree `/home/alberto/Desktop/LINTRA/lintra-p6-01`.

## Decisioni dell’autore (24 novembre 2026)

1. **Stack DEC-MIG-08 completo** (Tailwind CSS, componenti shadcn/ui, TanStack
   Table/Virtual), approvato in questa task.
2. **Redesign della shell dentro P6-01**: non esiste una task di redesign
   dedicata nel piano (P5-01 «Shell React responsive e tema» è DONE). La UI
   segue i riferimenti `lintra_ui_references` (`lintra_tauri_ui.png`, logo book;
   direttiva «seguire pedissequamente gli esempi», UI SaaS moderna e responsive).
3. **Colonne «faithful look, real data»**: nessuna colonna Esito con Avviso/Valido
   (in Lintra ogni riga è una violazione, nessuna severità) né KPI inventati.
4. **Grafici dei risultati** dalla reference `Lintra_ quattro interfacce Rust a
   confronto.png` (Dioxus: errori per tracciato, tipologia errori; Tauri+Leptos:
   distribuzione errori, tasso di validità). **Sempre visibili**, anche con
   controlli incompleti, con un avviso esplicito (decisione rivista
   dall’autore nella stessa giornata). Dopo la review round 1 (P601-01) l’avviso
   copre anche la completezza ignota: con `report_about` in lettura i grafici
   portano «Completezza in verifica» (`role="status"`), con `report_about`
   fallito «Completezza non disponibile» (`role="alert"`); solo un report letto
   e completo li mostra senza qualifica.
5. **Motivazione nella tabella**: messaggio, valore atteso e valore trovato
   nelle righe. Modifica del contratto di paging ratificato (P2-05/G-MIG-2),
   decisa dall’autore e documentata come emendamento in `PAGING.md`.
6. **Autorizzazione al core**: modifiche al core consentite solo per aderire
   alla UI delle reference; usata per i conteggi per tracciato.

Fonti: prompt atomico; MIGRATION_PLAN v1.1 riga P6-01 («DOM e memoria bounded
su dataset sintetici, preferire libreria scelta in DEC-MIG-08 con test») e
G-MIG-6; `MIGRATION_DECISIONS_V1_1` DEC-MIG-08; `MIGRATION_SCOPE_PARITY.md`;
`MIGRATION_TAURI_BRIEF.md`; `LINTRA_OFFICIAL_BRAND.md`; IPC v1 `PAGING.md`,
`DETAIL.md`, `CONTRACT.md`. Il `TASK_PROMPTS/T-MIG-P6-01.md` originale non è nel kit.

## Cosa cambia

**Dipendenze** (pin esatti, `ignore-scripts`; supply chain DEC-MIG-08):
runtime `@tanstack/react-table` 9.3.1, `@tanstack/react-virtual` 3.14.28,
`@radix-ui/react-slot` 1.5.0, `class-variance-authority` 0.7.1, `clsx` 2.1.1,
`tailwind-merge` 3.7.1, `lucide-react` 1.79.0; build `tailwindcss` e
`@tailwindcss/vite` 4.3.5. Licenze MIT/ISC/Apache-2.0 (transitive anche
BlueOak-1.0.0 e 0BSD solo in build). 13 pacchetti runtime nuovi con testo di
licenza verbatim in `public/THIRD_PARTY_NOTICES` e hash in
`assets-provenance.json`. Nessun CDN, font remoto o asset esterno.

**Marchio**: `public/brand/lintra-logo-light.png`, `lintra-logo-dark.png`,
`lintra-mark.png`, ritagli del logo book ratificato (SHA-256 sorgente
`85f1c48f…`), solo sfondo del pannello rimosso e trim; area di ritaglio,
comando e hash in `assets-provenance.json`. Il wordmark non è ricostruito.
CSP `img-src 'none'` → `'self'` (solo asset locali; la guard Windows filtra già
per origine).

**Shell e stile** (`AppShell.tsx`, `style.css`, `tokens.css`, `ui.tsx`):
sidebar bianca con logo per tema, WORKSPACE con icone, flusso selezionato
(quello accettato da Rust), tema segmentato, stato del servizio con versione;
header con breadcrumb, titolo grande, azione primaria. Card per form,
esecuzione, Informazioni. Palette del logo book; contrasto testi ≥ 4,5:1
in entrambi i temi (minimo 5,09:1). Ordine di Tab invariato (skip link,
sezioni, tema, contenuto). Sotto 960 px la barra è compatta. Ombre morbide
delle card come nella reference (il brief diceva «nessuna ombra»: prevale la
direttiva successiva dell’autore).

**Risultati** (`Results.tsx`, `pages.ts`, `Charts.tsx`):
- KPI: record analizzati, diagnostiche nel report, record con violazioni,
  esito; tutti da Rust.
- Grafici SVG senza libreria: errori per tracciato, tipologia errori (donut),
  distribuzione per codice (barre orizzontali, i primi 6 codici più «Altri
  codici (n)»), record senza violazioni (anello, percentuale esatta su u64 per
  difetto, mai arrotondata a 100%). Valori scritti accanto ai segni e
  alternative testuali; avviso «Risultati incompleti» se il report è parziale,
  troncato o annullato.
- Griglia: TanStack Table (colonne N., Riga, Tracciato, Campo, Controllo,
  Motivo) e TanStack Virtual (righe a altezza fissa 56 px, overscan 6). Pagina
  logica = una pagina IPC di 128 righe; paginazione della reference con al
  massimo 7 pulsanti; cache di al massimo 2 pagine della stessa query; risposte
  tardive di altra pagina, query o run scartate. Offset e contatori sempre
  come decimali u64 in BigInt.

**Rust** (scope esteso autorizzato):
- core `src/report/index.rs`: `JsonlReportIndex` conta le diagnostiche per
  tracciato nella stessa scansione verificata (`errors_by_trace`).
- adapter `results.rs`/`state.rs`: Row DTO con `message`, `expected`,
  `actual` (preview ≤ 128 byte); nuova operazione `results_overview` (record,
  totali, per codice, per tracciato con nome e handle, legata al run
  pubblicato; somme verificate).
- contratto: emendamento datato in `PAGING.md`; `paging_reference.py` e
  `verify_paging.py` aggiornati.

## Cosa NON cambia

CLI, golden JSONL, servizio di validazione, ordine canonico, filtri e query
Rust, detail (P4-09), export, SQLite, IPC v1 per le altre operazioni.
Nessun ordinamento o filtro nel renderer; nessun conteggio ricavato dalle
righe. `raw_hex`, contesto, identità del record restano in Rust.

## Acceptance

| Requisito | Prova |
| --- | --- |
| DOM bounded | `grid.test.mjs`: pagina di 128 righe su 1.000.000 corrispondenze, nel DOM 16 righe (viewport 10 + overscan 6) più due righe spaziatrici; nativo: fine pagina con righe 119–128 |
| Memoria bounded | renderer: ≤ 2 pagine × 128 righe (`remember`), nessun dataset; Rust: registro token ≤ 2 pagine (test esistente) |
| Libreria DEC-MIG-08 con test | TanStack Table/Virtual nel componente reale renderizzato nei test |
| Dataset sintetici | 100.000 diagnostiche E_RECORD_LENGTH nativo; 1.000.000 nei test |

Vedi «Test» per i comandi.

## Test

| Verifica | Esito |
| --- | --- |
| `npm run lint` / `npm test` / `npm run build` | PASS / 138 PASS / PASS (rieseguiti dopo il rework P601-01) |
| desktop `cargo fmt` / `test --release` / `clippy -D warnings` (target Linux, `tauri/custom-protocol`) | PASS / 0 ×3: 85 test (2 nuovi), 0 fail, 1 ignored. Il primo giro clippy ha segnalato `large_enum_variant` (indice cresciuto): `Source::Owned` ora contiene `Box<JsonlReportIndex>` |
| `python3 docs/migration/ipc-v1/verify_paging.py` | PASS 72/72 (2 nuovi) |
| `npm run tauri -- build --no-bundle` + `native_smoke.py` (Hyprland) | PASS / 0, binario finale `8265a71e854f352c3012e067b6664751f947389450e3cc866613056fdfe49834` |
| `python3 tests/native_capture.py <bin> <out>` | PASS 51/51 + chiusura durante il run (exit 0, 0,03 s, nessuno spool) su `0541b687…9127` ([receipt](T-MIG-P6-01-native/receipt.json)); 43 scenari P5 con la nuova UI + 8 Risultati |
| binario finale: `results-grid-1366x768`, `results-valid-1366x768`, chiusura durante il run | PASS (exit 0, 0,04 s); differenza col binario del giro completo solo `Box` e formattazione dei test, evidenze in `lintra-review-handoffs/T-MIG-P6-01-evidence/final-binary*` |
| workspace `ci.yml` (fmt; clippy e test default e `--all-features`; xtask con `LINTRA_TEST_BIN`), core toccato | PASS / 0 ×8: 1116 test, 0 fail, 9 ignored, sia default sia all-features; xtask 18/18 |
| Windows/WebView2 e CI remota | NOT_RUN (CI remota dopo la review, come per P5-08) |
| `npm ci` con i pin esatti | NOT_RUN in locale (Node 26.13.1, pin 26.13.0); coperto dalla CI |

## Rework dopo la review Codex round 1

Rilievo **P601-01 [P2]** accolto: `Results.tsx` ricavava `incomplete = false`
per ogni stato di `report_about` diverso da `ready`, quindi con overview pronta
e metadati in lettura o falliti i grafici apparivano senza qualifica (anello
`100,0%` senza avviso). Ora `Charts` riceve `scope: ChartScope`
(`complete | incomplete | pending | unavailable`) e mostra un avviso per
ognuno tranne `complete`; lo stato `none` vale `pending`. L’avviso Partial
resta identico. Nessuna modifica Rust, IPC o dipendenze.

Test: `grid.test.mjs`, «Risultati shows the charts once the overview is
read…», esteso con overview pronta e `about` in `loading`/`none`/`failed`
(grafici presenti, avviso con ruolo e testo attesi, nessun «Risultati
incompleti») e controllo Complete senza `notice-warn`. Prova di mutazione:
con la logica precedente di `Results.tsx` il test fallisce (137/138), con
la correzione passa (138/138). Catture native non ripetute: la modifica è
solo di presentazione e coperta dal render del componente reale.

## Rischi residui

- Il valore trovato (`actual`) è nel DOM per le righe visibili: è un dato del
  file analizzato. Decisione dell’autore; il contratto lo limita a preview
  di 128 byte, mai in log/storage.
- Le preview delle righe sono tagliate a 128 byte (marcate); il testo completo
  resta al dettaglio (P6-03).
- Pannello di dettaglio, filtri e ricerca della reference sono P6-02/P6-03.
- La reference mostra «Avvisi»: Lintra non ha severità, quindi non compaiono.
- Con zero diagnostiche la card «Distribuzione errori» resta vuota senza una
  frase esplicita (minore, visibile in `results-valid-1366x768`).
- Le 51 catture sono sul binario precedente alla correzione clippy (`Box`):
  sul binario finale sono ripetute smoke, griglia, report valido e chiusura.
