# PROMPT ATOMICO — T-P6-03: Alignment

Sei l'agente coding Lintra e stai eseguendo ESCLUSIVAMENTE `T-P6-03`. Il master prompt e il protocollo del Master Implementation Plan §14 hanno priorità operativa.

## Obiettivo vigente — revisione esplicitamente autorizzata dall’autore

Obiettivo: sola estrazione zero-copy del logical dal lato dichiarato tramite
logical_value, senza controlli fisici del padding e senza E_ALIGNMENT.
RightPad/StripTrailingFill elimina il solo fill finale contiguo;
LeftPad/StripLeadingFill elimina il solo fill iniziale contiguo;
preserve_width/exact conservano l'intero raw; con stripping, all-fill produce
slice vuoto. Combinazioni lato/policy incoerenti sono errori tecnici tipizzati,
non FieldViolation. Nessun parsing numerico, trim globale o dominio.
Rif.: DET §44 revisionato con autorizzazione dell'autore; §15 per il modello.
Prereq.: T-P6-01. File: src/validators/alignment.rs, export e test sintetici.
Test: lati dichiarati/opposti, fill interno, preserve/exact, all-fill, slice
vuoto, prestito zero-copy, combinazioni incoerenti. Accettazione: STD-VERIFY.
Il controllo fisico del padding è fuori ambito del formato 1, senza residuo.
CNF-ALIGNMENT-01 archiviata come estensione futura non implementata.
Dopo verifica procedere a T-P6-04 secondo il mandato autonomo vigente.

## Procedura vincolante
1. Leggi `IMPLEMENTATION_STATUS.md`, Master Plan §4 e §14, la scheda di fase §9 e le sezioni ARCH/DET sopra indicate.
2. Verifica la dipendenza `Prereq.` descritta sopra contro evidenze reali, non solamente `DONE` dichiarato. Registra `BLOCKED` se mancante.
3. Ispeziona git, stato dei file e contratti correnti. Dichiara in apertura massimo 5 righe di piano: file, cambiamenti, test.
4. Implementa solo questa modifica, rispettando la separazione Rust core vs schema e senza introdurre condizioni `if flow == CAD/SIOC` nel core.
5. Scrivi unit/integration test che dimostrino almeno un caso positivo e uno negativo/boundary pertinente. Se la task è puramente documentale, verifica completezza e tracciabilità invece di inventare un test artificiale.
6. Esegui gli strumenti disponibili: `cargo fmt --all -- --check`; `cargo clippy --all-targets --locked -- -D warnings`; `cargo test --locked`; oppure, se non applicabili/non disponibili, documenta il motivo. Non spacciarne l'esito per CI verde.
7. Aggiorna `IMPLEMENTATION_STATUS.md` con evidenza, decisioni, conflitti, modifiche ai documenti, e HANDOFF §15.3. FERMATI: nessun task successivo autonomo.

## Esclusioni valide per qualunque fase
- Non usare file reali di pazienti come fixture/versionati.
- Non alterare la baseline SIOC FROZEN né promuovere CAD a FROZEN senza approvazione/gate.
- Non anticipare task di storage/implementazione CAD o la UI Slint se non ricadono nello scope. **La persistenza è capacità core approvata** (ADR-PERSIST-001), da introdurre progressivamente nelle task T-PERSIST-01..12; i tipi/contratti minimi possono essere predisposti quando autorizzati dalla task.

## Formato finale obbligatorio
TASK; STATUS; FILE MODIFICATI; EVIDENZE TEST; BLOCCANTI; DECISIONI; GIT STATUS; HANDOFF; NEXT TASK.

## Tracciabilità del piano storico

La voce originale del Master Plan resta immutata; il mandato successivo
dell’autore sostituisce per questa task l’obiettivo seguente:

> Obiettivo: `AlignmentRule::{RightPad, LeftPad, Exact, Builtin}` e `validate_alignment`. Rif.: DET §15, §44. Prereq.: T-P6-01. File: `src/validators/alignment.rs`. Casi limite: non convertire in numero, non validare dominio, non dedurre regole dal solo nome del tipo; `Builtin` come boundary. Test: padding corretto/errato, campo tutto fill. Accettazione: STD-VERIFY.
