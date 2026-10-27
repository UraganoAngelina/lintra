# T-P16-02 — bootstrap desktop, font e temi

## Prerequisiti e decisioni

T-P16-01 DONE, CI40513168784/e5b00b2 Linux/MSVC/JSONL success. T-P16-00 ha
riferimento chiaro approvato dall’autore e CI40485215039; font IBM Plex forniti,
licenza OFL e Slint1.18.1 già qualificati. Applicati ARCH39D.1/3, DET Addendum B,
DEC-07/15, mandato temi e skill no-ai-slop/Impeccable/Hallmark. Un solo agente,
nessun daemon/delega, nessuna modifica ARCH/DET/FROZEN o regole normative.

## Moduli, condizioni e ragioni di correttezza

- ui/Cargo.toml: dipendenze esclusivamente desktop, Slint/slint-build pinned,
  default GPU disabilitati; software+winit-X11 (Win32 sul target Windows).
  png per capture e tempfile per fixture font, entrambi package già qualificati.
- build_support/fonts.rs: elenco delle cinque facce originali; verify legge ogni
  file e richiede intestazione TrueType. File mancante/corrotto è errore, non un
  fallback. build.rs usa questa funzione prima di Slint e dichiara rerun per font;
  EmbedFiles incorpora risorse/font, style fluent-light deterministico. Il
  compiler Slint verifica poi il contenuto. OFL originale accompagnatorio.
- src/lib.rs: moduli generati e funzione che seleziona esplicitamente winit e
  software. src/main.rs costruisce la finestra, usa versione reale Cargo e avvia
  il loop nativo; non chiama validazione, non inventa report o persistenza.
- ui/tokens.slint: valori chiari originali byte per byte nei rami light; dark-mode
  sceglie solo token scuri neri/rosso-arancioni. Font, dimensioni, geometria,
  pesi, spazi, bordi e colori nello stesso file. Snapshot palette/hash font in
  source-assets.json; nessuna versione/checksum preesistente rimosso dal lock.
- ui/main.slint: finestra base ridimensionabile, layout verticale esplicito
  header/body/footer. Nessuna schermata analisi/risultati o export anticipati.
  ActionButton ha una sola azione per puntatore, accessibilità e Space/Return;
  FocusScope a larghezza zero evita l’intercettazione degli eventi del TouchArea,
  seguendo il pattern del framework. Focus visibile e forward-focus, radio con
  checked/label per i temi, Informazioni raggiungibile dal livello principale.
  La scelta modifica Tok.dark-mode, non una preferenza del sistema o un file.
  Palette.color-scheme è impostata esplicitamente per il widget ufficiale
  AboutSlint: attribuzione reale e versione Slint, non un testo sostitutivo.
- tests/bootstrap.rs: piattaforma software reale, nessun display/GPU simulato.
  Eventi puntatore selezionano temi, Space/Tab provano tastiera; capture complete
  NewBuffer a1×/2× con sfondo esatto, contrasto sRGB calcolato per testi normali,
  secondari e selezionati. Informazioni renderizzata in entrambi i temi con
  superficie verificata; resize640×480 conserva il controllo tema raggiungibile.
- tests/font_contract.rs: cinque facce originali positive, copia con faccia
  mancante e stub vuoto negative. check_missing_font.py va oltre il test helper:
  copia temporanea di sorgenti/manifest/lock, rimuove una faccia dalla sola copia,
  esegue Cargo reale offline e pretende il messaggio della build obbligatoria.
  Riusa gli artifact del target nativo senza modificare manifest/lock/font reali.
- compare_pixels.py: quattro inventari RGB con dimensioni esatte, confronto
  byte per byte e posizione del primo mismatch; assenza/troncamento non sono
  successi. Un test Python prova positivo, vuoto, un canale alterato e truncation.
- CI: workspace intero sui due runner, fontconfig come prerequisito Linux come
  nello spike, negativa build reale, upload RGB/PNG/contrasti e job separato
  compare-ui-pixels.35min per compilare anche il workspace grafico; controlli
  mantenuti, compare JSONL e release CLI riproducibile/verify-artifact invariati.

## Evidenze locali

Fmt e Clippy workspace all-targets default/all-features positivi.703/711test,
87suite,0fail/2ignored, incluso bootstrap e contratto font, poi boundary UI
aggiuntive ripetute con Tab/Space/resize/attribuzione e Clippy finale positivi.
20test Python pass, py_compile/actionlint1.7.12 pass (shellcheck/pyflakes esterni
non disponibili). Cargo build senza faccia respinto realmente, font originali
intatti; guard metadata core/CLI senza Slint pass. Filehash/colortoken e vecchi
lock versions/checksums preservati. Due PNG1× visti, geometria/controlli coerenti.
Contrasti minimi: light body15.25, muted6.46, selected4.84; dark17.97/10.00/6.46.

Impeccable craft-floor e Operate applicati, contesto non ricaricato una seconda
volta nella sessione. Hallmark sloplint non riconosce file Slint: risultato
no supported web files registrato, non un audit passato. Nessuna falsa versione
HTML, animazione o metrica. Vedere docs/ui/SKILL_APPLICATION.md.

## Limiti, stato e handoff

IN_PROGRESS/WAITING_GATE: CI Linux/MSVC e identità pixel1×/2× dei due temi
ancora da completare. I PNG bootstrap in docs/ui/bootstrap sono diagnostici;
i riferimenti Risultati approvati sono invariati, nessun golden autoapprovato.
Nessuna prova manuale del compositore desktop/accessibilità di sistema dichiarata;
test headless software reali, non una certificazione finale di usabilità.
Versione schema/bundle nella UI quando selezionati, worker, progress, cancellazione,
risultati reali/export e UI packaging/allowlist rimangono nelle task16-03..08.
La licenza è dichiarata nel modulo UI, il core non dipende dal framework.

Next16-03 dopo gate16-02 positivo. Durante attesa, PERSIST-02 indipendente è
ora pronta: P2/M1 qualificate tramite CI40515191339 e PERSIST-01 DONE tramite
CI40510757138. Nessun consenso ordinario richiesto; AUTO_CONTINUE.

## Diagnosi primo gate nativo

CI40519581845/9832d85: Linux success, MSVC failure nello step guard metadata.
Test Rust/UI default/all-features su Windows già success; compare RGB/JSONL
skipped, nessuna identità cross-OS presunta. Log reale: CP1252 non decodifica
un byte0x81 dei metadati Cargo UTF-8. Il guard ora acquisisce stdout come byte
e decodifica UTF-8 strict, senza fallback permissivo. Rustc -vV usa UTF-8
esplicito. Regressione Unicode riproduce il fallimento CP1252 e prova UTF-8
corretto/byte invalido respinto.21Python e metadata reale positivi.
Nessun pixel/font/contrasto o gate rimosso; run sostitutiva da avviare.
