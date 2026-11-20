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

## Rischi residui

- **Windows: screenshot e interazione nativa NOT_RUN.** I test Node girano
  nella CI desktop Windows al prossimo run autorizzato; mancano capture
  WebView2. Da coprire al checkpoint G-MIG-5 o su VM Windows 11.
- Scala 2×/zoom 200% non provata (monitor 1×).
- Il titolo focalizzato mostra un anello largo quanto il contenuto: estetico,
  non bloccante.
- Tema non persistente tra avvii: richiede impostazioni lato Rust, fuori scope.

Prossima tecnicamente disponibile: T-MIG-P5-02.
