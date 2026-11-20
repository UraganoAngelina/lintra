# T-MIG-P5-01 — Shell React responsive e tema

Data 20 novembre 2026. Stato: **DONE**. Review Codex round 3 APPROVED sul
delta funzionale fino a `e726310`; integrazione autorizzata dall'autore per
fast-forward di `integration` e `work/p1-foundations`, senza push. CI remota
sul delta NOT_RUN, al checkpoint del blocco P5; G-MIG-5 aperto. Prerequisiti: T-MIG-P4-10 DONE e G-MIG-4 PASS
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
- `src-tauri/src/window_fit.rs` e `main.rs` (review Codex round 2, P501-03,
  opzione A autorizzata dall'autore): su Windows, all'avvio, la finestra
  principale viene ridotta e se serve spostata dentro la work area del
  proprio monitor; se nemmeno il minimo configurato (640×480) ci sta, il
  minimo viene abbassato a quanto il monitor può mostrare. Senza questa
  correzione, a 200% su 1920×1080 la finestra si apriva a 1950×1110 fisici,
  oltre lo schermo e sotto la barra delle applicazioni (brief: niente
  dimensioni native che nascondono comandi fuori monitor). Funzione pura
  `fit_to_work_area` compilata e testata su ogni OS; applicazione solo su
  Windows: su Wayland il compositor possiede il posizionamento e applica le
  richieste di dimensione in modo asincrono (una prima versione attiva anche
  su Linux entrava in race con il posizionamento e falliva la cattura 2×).
  Best effort: un monitor non misurabile lascia la dimensione configurata.

## Cosa NON cambia

IPC, DTO, CSP, `tauri.conf.json`, core/CLI/Slint, golden e JSONL. Nessuna
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
| src-tauri fmt `--check` | PASS rc 0 |
| src-tauri test release Linux | PASS 68 (62 + 6 `window_fit`), 1 ignored |
| src-tauri Clippy release all-targets `-D warnings` Linux | PASS, nessun warning |
| Build release Linux + `tests/native_smoke.py` | PASS, `NATIVE_DESKTOP_SMOKE_PASS`, binario `0e4c0d1c…04dc` |
| Windows VM: build release MSVC | PASS exit 0, binario `32ec4b77…801e` |
| Windows VM: Clippy all-targets `-D warnings` MSVC | PASS exit 0, nessun warning |
| Windows VM: test release MSVC | PASS 67 (61 + 6 `window_fit`) |
| `npm ci` locale | NOT_RUN valido: engine pin node 26.13.0, locale 26.13.1 (ambientale) |

`tests/shell.test.mjs` compila il componente reale con `transformWithOxc` di
Vite e lo rende con `react-dom/server`: landmark e ordine, una sola sezione
corrente per ogni vista, radio con preferenza, stati servizio/report,
escaping del messaggio d'errore, assenza di `style=`/`<img>` (CSP) e di click
su elementi generici; valori non fidati rifiutati; `applyTheme`; contrasto
WCAG ≥4,5:1 di tutti i ruoli testo e ≥3:1 del focus in entrambi i temi;
nessun colore fuori da `tokens.css`. I test `window_fit` coprono: finestra
più grande della work area (caso osservato a 200%), finestra già contenuta
(nessuna modifica), finestra contenuta ma fuori area (spostata), work area
spostata da un pannello, dimensioni mai aumentate e aree inutilizzabili
ignorate, coordinate estreme senza overflow.

## Prove native (tutte sui binari finali)

Gli harness agiscono solo sulla finestra del processo posseduto, catturano
l'area della finestra/client, chiudono il processo e verificano gli hash; un
guard esige pixel diversi dalla baseline per ogni sequenza di tasti. Tutte le
immagini sono state osservate.

**Linux 1× — CachyOS/Hyprland, `tests/native_capture.py`**
([receipt](T-MIG-P5-01-native/receipt.json)): PASS 10/10 su `0e4c0d1c`.
640×480, 1024×768, 1366×768 e work area 1850×1060; Tab sullo skip link,
Tab×3+Invio → Risultati, Tab×4+Invio → Informazioni, frecce sul gruppo
tema → Chiaro/Scuro. Hyprland restituisce 1367/1851 px dove sono richiesti
1366/1850 (arrotondamento del compositor; la correzione non è attiva su
Linux). 1920×1080 non entra nella work area accanto al pannello.

**Windows 11 1× — VM, `tests/windows_capture.ps1`**
([manifest](T-MIG-P5-01-windows/manifest.json)): PASS 11/11 su `32ec4b77`,
sessione console 1, `lintra-agent` non elevato, DPI 96, display 1280×800
(work area 1280×752). 640×480 e work area 1264×713, tema di sistema chiaro e
scuro, stessi percorsi da tastiera. In ogni scenario la finestra si apre
**dentro** la work area (es. `[26,0 → 1142,752]`): prima della correzione la
dimensione configurata sarebbe uscita sotto la barra delle applicazioni.

## Scala 200% (review Codex round 2 — P501-03)

[Manifest](T-MIG-P5-01-scale2/manifest.json). Il brief (righe 147–152)
richiede scala 2× e zoom/DPI 200% con scroll e focus nella work area
effettiva, senza finestre fuori monitor.

- **Linux 2×**: output headless Hyprland 1920×1080 a scala 2 creato per la
  prova e rimosso (monitor reale invariato); PASS 7/7 su `0e4c0d1c`. 640×480
  logici (1280×960 fisici) e work area 890×520 (1780×1040 fisici): barra in
  alto, skip link, navigazione e focus sul titolo, tema Chiaro/Scuro.
- **Windows 2×**: display 1920×1080 a 192 DPI per `lintra-agent`; PASS 7/7
  su `32ec4b77`. Work area 1920×984 fisici (960×492 logici): la finestra si
  apre esattamente a **`[0,0 → 1920,984]`** (prima della correzione
  `[0,0 → 1950,1110]`, oltre lo schermo). Area client 947×456 CSS: barra in
  alto, tema di sistema chiaro/scuro, skip link, Risultati/Informazioni da
  tastiera con focus sul titolo, Scuro e Chiaro sopra il sistema scuro.
- **Scroll e focus**: con il contenuto della shell nessuna pagina supera la
  viewport né in orizzontale né in verticale, anche nell'area minima a 2×
  (947×456 CSS); Tab raggiunge ogni controllo e nessun comando resta fuori
  dalla finestra o dal monitor. A 200% su 1080p un viewport 640×480 più la
  barra del titolo non può entrare nei 492 px logici: il minimo viene
  abbassato dalla correzione invece di sporgere dal monitor.

Procedura VM: snapshot `LINTRA-P5-01-PRETEST-20261120` creato a VM spenta;
autologon temporaneo con Sysinternals Autologon (firma Microsoft valida,
password casuale generata nella VM, mai stampata, nessuna password in chiaro
in Winlogon); task Interactive/Limited. Per il 2× la risoluzione preferita
del firmware OVMF è stata portata a 1920×1080 dal menu di setup (l'adattatore
video di base espone solo la modalità del firmware), dopo un backup del file
NVRAM eseguito dall'autore; `LogPixels=192` per l'utente. XML del dominio e
snapshot ripristinati e verificati (XML originale, nessun autologon/task/
cartella della prova, stato P1-04 conservato, SSH standard), VM spenta. Il
file NVRAM è fuori dallo snapshot: il ripristino dal backup è a cura
dell'autore. Il binario MSVC non è bit-riproducibile tra build (stessa
sorgente, hash diversi): ogni prova Windows è legata all'hash del binario
effettivamente eseguito.

Storico: le prove sui binari precedenti alla correzione (`e3c316c`/`e1874f6`,
Windows run 1 FAIL per dimensioni e run 2 PASS) restano nella storia Git;
una prima esecuzione 2× Windows (FAIL per tempi di rendering dell'harness,
corretto con un'attesa di frame stabili) e una 2× Linux con la correzione
attiva anche su Linux (FAIL per race, motivo della limitazione a Windows)
sono documentate qui e non conservate come immagini.

## Consegna (review Codex round 1 — P501-01)

Revisionare il worktree dedicato pulito `/home/alberto/Desktop/LINTRA/lintra-p5-01`
sul branch `work/T-MIG-P5-01`. Su richiesta esplicita dell'autore, le sue
modifiche a `CLAUDE.md` e la cartella `graphify-out/` (965 file prima staged
nel repository principale) sono committate sul branch in un commit separato,
non parte del delta funzionale della task.

## Rischi residui

- Windows 1024×768/1366×768/1920×1080 a 1× e scala 2× su monitor più grandi
  non provati. A 1× entrambi i layout (barra superiore sotto 960 px CSS,
  colonna laterale da 960 px) sono provati su entrambi gli OS; a 2× tutte le
  work area provate restano sotto 960 px CSS (Linux 640/890, Windows 947),
  quindi è provata soltanto la barra superiore. La colonna laterale a 2×
  richiede un monitor con work area di almeno 1920 px fisici di larghezza
  utile oltre la cornice e non è provata (review Codex round 3, P501-04).
- Prove VM e personali, distinte dalla CI: la CI desktop sul delta resta
  NOT_RUN (nessun push).
- La correzione della finestra vale all'avvio; uno spostamento successivo su
  un altro monitor è gestito dal sistema operativo.
- Il titolo focalizzato mostra un anello largo quanto il contenuto: estetico.
- Tema non persistente tra avvii: richiede impostazioni lato Rust, fuori scope.

Prossima tecnicamente disponibile: T-MIG-P5-02.
