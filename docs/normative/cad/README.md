# CAD/ADI — Tabelle campi per revisione, T-P0-02

**DONE — campione verificato e approvato dall'autore il 2026-10-16.**
L'esito corrente è registrato in `IMPLEMENTATION_STATUS.md`; `document_status`
nei JSON e il report iniziale descrivono lo snapshot preparatorio antecedente
all'approvazione. Nessun valore della fonte è stato riscritto.
Le tabelle trascrivono il derivato **RELEASE_CANDIDATE**
`K/CAD_REVISED_CANDIDATE/`, DSL `1.0.0`, revisione
`2026-10-16.reviewed-g02-g06`. La fonte primaria ricevuta è
`K/SCHEMA_INPUTS/CAD_2024-06_SOURCE/2025-05-14_FLUSSO_ADI.pdf`, 7 pagine.
`K/` indica lo starter kit adiacente al repository. Aggiornamento interno
`2024-06` e data etichetta `2025-05-14` restano distinti da decorrenza normativa.
Vedi [inventario](../INPUT_INVENTORY.md) e [report riproducibile](../../verification/T-P0-02.md).

| Tabella | Campi | Intervallo one-based inclusivo | Pagine PDF |
|---|---:|---|---|
| [A](A-fields.json) | 24 | 1–218 | 1 |
| [B1](B1-fields.json) | 36 | 1–198 | 2–4 |
| [B2](B2-fields.json) | 5 | 1–41 | 4 |
| [C](C-fields.json) | 11 | 1–45 | 5 |
| [D](D-fields.json) | 7 | 1–39 | 5 |

Ogni riga conserva esattamente `id`, `name`, `physical` completo e
`regulatory_metadata` completo dal trace ricevuto. Rimangono distinte le
etichette `source_obligation`, `source_nsis_obligation`, `resolved_obligation`,
le note originali estratte e le annotazioni di progetto già presenti.
I nomi sono quelli del derivato ricevuto, talvolta abbreviati rispetto al PDF
(esempio `1.6: MPI`); non vengono presentati come trascrizione letterale delle
intestazioni PDF. `\n` rappresenta i newline della stringa ricevuta; nessuna
riscrittura o traduzione in regole, nessuna nuova normalizzazione dei tipi.

`source_manifest_metadata` conserva l'intero manifest ricevuto;
`source_trace_metadata` conserva versioni, stato, identificazione/lunghezza,
`value_model` e `freeze_notes`. `provenance` riporta percorsi e SHA-256 di trace,
manifest, flow e PDF. Hash e dimensioni coincidono col manifest del kit;
i cinque split coincidono integralmente con i trace annidati nel flow.

Verifica documentale: **83/83** campi identici al JSON, nessun ID duplicato,
gap o overlap; ultimo byte uguale a `record_length` in tutti i trace.
`pdftotext -layout` conferma **83/83** righe per ID, pagina, lunghezza,
posizione, tipo sorgente e le due colonne di obbligatorietà; le **83/83** note
intere sono reperibili nella pagina dichiarata dopo estrazione della colonna
note e sola compressione del whitespace per il confronto. Il JSON consegnato
mantiene invece il whitespace originale. Questo riscontro testuale non
certifica celle, layout, colori, autenticità del PDF o interpretazione normativa;
`source_note_extraction` continua a richiedere revisione dei passaggi sensibili
al layout. Non sono stati eseguiti confronti visuali né verifiche del runtime.

Esempi consegnati per il campionamento umano, approvato dall'autore:

- A `1.4`/`1.5`, p. 1: `NO (1*)` nella prima colonna e `NBB` in NSIS;
  A `1.16`/`1.17`: `SI` contro `NO`; conservare anche la nota a piè pagina.
- B1 `2.6`/`2.7`, p. 2: `NBB` contro `SI`; `2.10` conserva il refuso
  sorgente `GGMMAAA` insieme alla lunghezza 8, senza correzione silenziosa.
- B1 `2.17`/`2.18`: colonna `SI`, note con soglie d'età e `NBB`; `2.22`
  conserva il tipo sorgente `T`, distinto dal tipo fisico `alphanumeric`.
- B1 `2.29`, p. 3: riga isolata su pagina separata; `2.30–2.35`, p. 4:
  controllare che note e colonne non si confondano con l'intestazione.
- B1 `2.36`, B2 `3.5`, C `4.11`, D `5.7`: prima colonna `SI`, NSIS `NO`,
  `resolved_obligation: NBB` del candidato; non fonderle. C `4.7` ha lo
  stesso disallineamento. D `5.5`, p. 5: intera nota lunga delle prestazioni.

Nessun export/dato sanitario consultato; nessuna promozione CAD a FROZEN,
chiusura G0 o approvazione umana implicita.
