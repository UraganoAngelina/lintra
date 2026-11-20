# T-MIG-P5-01 — Shell React responsive e tema

Data 20 novembre 2026. Stato: **IN REVIEW**, non DONE prima della review Codex
e dell'integrazione. Prerequisiti: T-MIG-P4-10 DONE e G-MIG-4 PASS
([checkpoint](G-MIG-4.md), CI44706977518/desktop44706949307 su 1ab34d8).
Branch `work/T-MIG-P5-01`, basato sul commit del checkpoint G-MIG-4
(`73e28ee`, non ancora integrato) a sua volta su `integration` 1ab34d8.

Fonti: prompt atomico, MIGRATION_PLAN v1.1 riga P5-01, Master Plan §6,
ADR-MIG-001, `MIGRATION_TAURI_BRIEF.md` (barra, stati, acceptance),
`LINTRA_OFFICIAL_BRAND.md` (palette, contrasti), dossier P3-03 (tema esplicito
rinviato alla UI funzionale).

## Cosa cambia

- `src/shell.ts`: modello puro. Sezioni della barra ratificata (Analisi
  iniziale, Risultati, Informazioni), preferenza tema Sistema/Chiaro/Scuro,
  guard per valori non fidati, `applyTheme` su `data-theme` di `<html>`.
  Scelta solo di sessione: nessuno storage JS (Master Plan §6 delta 2).
- `src/AppShell.tsx`: shell semantica. Skip link, `header` con `nav
  aria-label="Sezioni"` di `button` e `aria-current="page"`, gruppo radio
  `fieldset/legend` per il tema, stato del servizio `role=status`/`alert`,
  `main` con un solo `h1`; al cambio sezione il focus va al titolo (mai al
  primo render). Risultati mostra la disponibilità del report reale
  (`reportAvailable`, oggi sempre false finché P5-05/06 non collegano il job).
  Le sezioni espongono solo stati reali: nessun KPI, conformità o attività
  inventati.
- `src/main.tsx`: stato di sezione/tema e bootstrap IPC invariato
  (`app_info`, `client_ready` dopo font e due frame).
- `src/tokens.css`: registro unico con coppie `light-dark()`; il tema segue il
  sistema salvo scelta esplicita (`color-scheme` su `data-theme`). Nuovi token
  selezione (`--selected-bg/ink` da palette del book) e scala del brief
  (dati 12, controllo 36, raggio 4, focus 2, spazi 8/16, barra 224).
- `src/style.css`: barra in alto fino a 959 px, colonna laterale da 960 px;
  focus visibile su ogni controllo; sezione corrente con peso e marcatore oltre
  al colore; nessuna ombra, blur o animazione; nessun colore fuori dai token.

## Cosa NON cambia

IPC, DTO, Rust/src-tauri, CSP, core/CLI/Slint, golden e JSONL. Nessuna
dipendenza nuova: Tailwind e shadcn/ui (DEC-MIG-08) non servono a shell e
tema e il brief/brand chiedono una necessità concreta documentata per ogni
libreria UI; si valutano nella prima task che ne ha bisogno. Logo: esiste solo
la tavola del book, non artwork separati; l'header resta etichetta testuale
come in P3-03 (residuo esplicito). Form, run, risultati e About/licenze
restano a P5-02..07 e P6.

## Skill applicate

Ponytail (cambiamento minimo, nessuna dipendenza). ui-ux-pro-max: ricerche
mirate `--domain ux` e `--stack react` su sezione corrente, focus visibile,
skip link, scroll orizzontale, HTML semantico; adottate. Il design system
generato non è usato: palette e font sono già ratificati.

## Test

| Verifica | Esito |
| --- | --- |
| `npm run lint` | PASS rc 0 |
| `npm test` (41 preesistenti + 9 `tests/shell.test.mjs`) | PASS 50/50 |
| `npm run build` (typecheck strict + Vite) | PASS rc 0 |
| `cargo fmt --manifest-path src-tauri/Cargo.toml --check` | PASS rc 0 |
| src-tauri test/Clippy release Linux | vedi handoff |
| `npm ci` locale | NOT_RUN valido: engine pin node 26.13.0, locale 26.13.1 (ambientale) |
| Build release Windows MSVC (VM) | PASS exit 0, binario `657b7303…3c61` |
| `tests/windows_capture.ps1` (VM, console) | PASS 11/11 al run 2; run 1 FAIL per dimensioni, conservato |

`tests/shell.test.mjs` compila il componente reale con `transformWithOxc` di
Vite e lo rende con `react-dom/server`: landmark e ordine, una sola sezione
corrente per ogni vista, radio con preferenza, stati servizio/report,
escaping del messaggio d'errore, assenza di `style=`/`<img>` (CSP) e di click
su elementi generici; valori non fidati rifiutati; `applyTheme`; contrasto
WCAG ≥4,5:1 di tutti i ruoli testo e ≥3:1 del focus in entrambi i temi;
nessun colore fuori da `tokens.css`.

## Prove native Linux (CachyOS/Hyprland, personale)

`tests/native_capture.py` avvia il binario release in XDG isolato, agisce solo
sulla finestra del proprio PID (`window=address:`; tasti con `send_shortcut`,
nessun device di input globale), la rende opaca, la colloca nella work area,
cattura con grim, termina e attende il processo. Binario SHA-256
`3f8cdb86…3e34`; [receipt](T-MIG-P5-01-native/receipt.json) con hash delle
dieci immagini, verificati. Osservate con vision:

- 640×480, 1024×768, 1366×768 e work area 1780×1040: Analisi, nessun overflow
  orizzontale; sotto 960 px barra in alto su due righe.
- Tastiera: Tab mostra lo skip link; Tab×3+Invio apre Risultati, Tab×4+Invio
  apre Informazioni (versione `0.1.1-rc.3` dal backend), focus sul titolo.
- Tema: Tab×5+Giù sceglie Chiaro (1366 e 640), +Giù Scuro; la scelta esplicita
  prevale sul tema di sistema scuro della sessione.

1920×1080 non entra nella work area di questo monitor accanto al pannello; la
work area intera la sostituisce. `GTK_THEME` non cambia `prefers-color-scheme`
in WebKitGTK 2.54: il chiaro è provato via scelta esplicita, non via sistema.

## Prove native Windows 11 (VM, review Codex round 1 — P501-02)

Binario costruito nella VM `Windows11-Lintra-Test` dai sorgenti esatti di
`e3c316c` (`git archive`, SHA-256 `bed791fc…f81f`) con il `dist` prodotto dal
worktree pulito (stessi asset `index-C7DZeOx8.css`/`index-TIfr-Ihh.js`):
`cargo build --locked --release --target x86_64-pc-windows-msvc --features
tauri/custom-protocol`, account standard `lintra-agent`, exit 0,
`lintra-desktop.exe` SHA-256 `657b7303…3c61`. Il frontend non è ricompilato
con Node del guest; lock invariato da 1ab34d8.

`tests/windows_capture.ps1` (gemello di `native_capture.py`) gira in una task
pianificata Interactive/Limited nella vera sessione console 1 di
`lintra-agent`, non elevata; rifiuta token elevati, sessione 0 e sessioni
diverse dalla console. Per ogni scenario avvia il proprio processo, verifica
finestra visibile/titolo/sessione, ridimensiona l'area client, invia i tasti
con SendKeys solo mentre la finestra possiede il foreground, cattura l'area
client, chiude normalmente ed esige exit 0; il tema di sistema usa
`AppsUseLightTheme` dell'utente, ripristinato a fine run.

Windows 10.0.26300, DPI 96, area client massima 1264×713 sul monitor della VM.

- **Run 1 FAIL** (conservata in `T-MIG-P5-01-windows/run1-size-fail/`):
  4/11 PASS; 1024×768 e 1366×768 rifiutati esplicitamente perché non entrano
  nella work area. Nessun difetto della shell osservato.
- **Run 2 PASS 11/11** con scenari adattati: 640×480 (barra in alto) e work
  area 1264×713 (colonna laterale), tema di sistema chiaro e scuro, Tab sullo
  skip link, Tab×3+Invio → Risultati (entrambe le larghezze), Tab×4+Invio →
  Informazioni (versione `0.1.1-rc.3`), frecce sul gruppo tema → Scuro e
  Chiaro sopra il sistema scuro. Task result 0, exit 0 per ogni processo,
  hash immagini verificati, guard di pixel diversi dalla baseline; immagini
  osservate con vision.

[Manifest](T-MIG-P5-01-windows/manifest.json) con hash di sorgenti, dist,
binario, harness, log build e capture.

Console reale: autologon temporaneo via Sysinternals Autologon (firma
Microsoft valida, SHA `5d96bbc4…381b`) con password casuale generata nella VM,
mai stampata né in chiaro in Winlogon, come in P1-02/P1-04. Snapshot
`LINTRA-P5-01-PRETEST-20261120` creato a VM spenta prima della prova; dopo la
raccolta: UUID/disco verificati, revert, avvio e controlli PASS (nessun
autologon/password/task/cartelle della prova; stato P1-04 conservato; SSH
admin elevato e `lintra-agent` standard), stato finale spento.

## Consegna (review Codex round 1 — P501-01)

Revisionare il worktree dedicato pulito `/home/alberto/Desktop/LINTRA/lintra-p5-01`
sul branch `work/T-MIG-P5-01`. Il repository principale resta su
`work/G-MIG-4-checkpoint` con le modifiche dell'autore a `CLAUDE.md` e le 965
aggiunte staged `graphify-out/` preservate intatte (status invariato prima e
dopo il cambio di branch); non fanno parte della consegna.

## Rischi residui

- Windows 1024×768, 1366×768, 1920×1080 e scala 2× non provati: i monitor
  disponibili (VM 1264×713, PC 1780×1040 utili) non li contengono. Le larghezze
  provate coprono entrambi i layout, sotto e sopra 960 px.
- Prova VM, non CI: la CI desktop Windows sul delta resta NOT_RUN (nessun push).
- Il titolo focalizzato mostra un anello largo quanto il contenuto: estetico,
  non bloccante.
- Tema non persistente tra avvii: richiede impostazioni lato Rust, fuori scope.

Prossima tecnicamente disponibile: T-MIG-P5-02.
