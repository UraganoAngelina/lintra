---
name: Lintra Copper Ledger
description: Banco di lavoro nativo per la validazione di flussi regionali
colors:
  primary-light: "#A13E1A"
  primary-dark: "#FF9C66"
  background-light: "#F5F4F0"
  background-dark: "#101112"
  surface-light: "#FFFFFF"
  surface-dark: "#1B1D1F"
  ink-light: "#20272B"
  ink-dark: "#F4F3EE"
typography:
  title:
    fontFamily: IBM Plex Sans
    fontSize: 28px
    fontWeight: 600
  body:
    fontFamily: IBM Plex Sans
    fontSize: 14px
    fontWeight: 400
  technical:
    fontFamily: IBM Plex Mono
    fontSize: 12px
rounded:
  control: 4px
  panel: 6px
spacing:
  small: 8px
  group: 16px
  page: 24px
---
# Lintra UI — design system canonico

## Overview

Copper Ledger: grafite, avorio e rame; gerarchia tipografica diretta, navigazione
stabile e spazio dedicato alla lettura dei diagnostici. Modalità Operate.
Frontmatter estratto dai token eseguibili, senza introdurre valori runtime.

## Colors

Palette completa e regole semantiche in LINTRA_DESIGN_DIRECTION.md; valori
eseguibili in Tok. Azione e focus rame, violazioni rosse, parzialità ambra.

## Typography

IBM Plex Sans14px corpo,18px sezioni,28px titoli; IBM Plex Mono12px per
identificativi e valori. Font incorporati, numeri delle righe allineati a destra.

## Layout

Barra64px, margini24px, gap16/24px. Configurazione60/40; risultati con
ispettore404px e tabella che usa lo spazio residuo. Sotto1100px pannelli
impilati; minimo640×480 con scorrimento.

## Elevation & Depth

Bordi e superfici distinguono i piani. Nessuna ombra o animazione decorativa.

## Shapes

Raggi4px controlli,6px pannelli. Bordi1px, focus2px senza variazioni geometriche.

## Components

ActionButton primaria/secondaria/navigation, FieldEdit, ChoiceBox, ReportCheck;
ListView virtuale con selezione e tastiera, dettaglio e dominio a pagine.

## Do's and Don'ts

Usare i modelli reali e i componenti condivisi; distinguere esito e completezza.
Non aggiungere norme al renderer o trasformare la fine del run in conformità.



Dal2026-10-29 il riferimento visuale corrente è
[LINTRA_DESIGN_DIRECTION.md](LINTRA_DESIGN_DIRECTION.md), direzione Signature
«Copper Ledger», autorizzata dal prompt di redesign dell'autore. Token eseguibili
unici in `ui/ui/tokens.slint`; nessuno stack web o nuovo font.

Il contratto estetico precedente è conservato integralmente in
[signature/DESIGN-before.md](signature/DESIGN-before.md); mockup e PNG ratificati
in review/ restano evidenze storiche. Il nuovo mandato sostituisce palette,
geometrie e gerarchie precedenti, senza modificare contratti funzionali,
normativi, di persistenza, privacy o distribuzione.

Schermate effettive: analisi/configurazione, esecuzione, risultati/dominio,
selettore file, export e Informazioni. Nessuna UI storica disponibile.
LocalOnly rimane esplicito. Completo/parziale, errore tecnico e violazione
rimangono distinti attraverso testi e colore; fine run non implica conformità.
Modelli/callback Rust restano le fonti dei dati e delle azioni.

Evidenze e fasi sequenziali in `signature/` e nel report Signature.
