# Lintra — avanzamento al 2026-10-25

**84 task verificate su 140: 60%.** T-P13-08 ha superato il gate locale,
ma la CI nativa è ancora da verificare. Non è inclusa nel conteggio DONE.
Il conteggio esatto per tutti gli ID è nello [snapshot JSON](PROJECT_PROGRESS_2026-10-25.json).
Ogni task pesa uno: la percentuale non misura ore, costi o tempo residuo.
Le consegne con scope ratificato contano per quel solo scope.

## Cosa è operativo

- Pipeline byte-oriented di validazione locale, DSL e Decimal monetario unsigned
  con due cifre frazionarie, zero ammesso; codici/range non riclassificati.
- Schema versionato, compiler e registry: handle compilati immutabili, costrutti
  non supportati rifiutati, nessuna norma specifica nel motore.
- Reference membership/Union con evidenze verificate, provvisorie e inconcludenti;
  summary/audit espliciti e warning aggregati, senza falsi errori normativi.
- Report JSONL, CSV e XLSX, summary e pubblicazione atomica; input audit SHA-256,
  cancellazione e completezza distinti dagli esiti normativi.
- Servizio e CLI Flow/Full a due passate, proiezioni selettive, Context Store,
  join e cardinalità, parametri tipizzati, controlli cross-record/cross-trace.
- Nuova revisione operativa SIOC structural-minimal.2: quattro regole referenziali
  esplicite e policy atto revocante corrette secondo ratifica. Il golden utilizza
  un componente normativo dichiarato con esclusioni: non certifica tutto il SIOC.

## Verifica di T-P13-08

[Report tecnico e handoff](T-P13-08-goldens-memory.md),
[hash/provenienza](T-P13-08-artifact-hashes.json).

611 test passati in 72 suite, nessun fallimento, un probe preesistente ignored.
Fmt, check, Clippy con warning negati, release e cross-check Windows GNU passati.
Tre scenari golden attraversano compiler, servizio, join, diagnostica e audit
produttivi: rispettivamente 0, 5 e 1 errori; il caso cardinalità è Partial con
esiti Unknown, senza selezionare arbitrariamente un parent.

Il test memoria è attivo nella suite: 4096 record per trace, larghezza da 32
a 4096 byte, contesto invariato a 7.040.904 byte; picco richiesto all'allocatore
+4064 byte. Sono byte heap richiesti nel test locale, non RSS o promessa generale
sulle prestazioni. I 90 file protetti preesistenti sono identici; 33 hash nuovi
verificati. Nessuna modifica al runtime Rust di produzione in questa task.

## Cosa resta da consegnare

- 54 task non iniziate: tooling schema restante (4), packaging/release (7), UI
  Slint (9), hardening/benchmark (6), documentazione operativa (4), persistenza
  core (12), integrazione CAD (12).
- T-P2-03 resta parziale con residuo approvato relativo ai payload applicativi.
- T-P13-08 attende l'esito CI Linux/MSVC; M6 non è dichiarata prima del gate.
- Restano i rinvii G0: dataset ufficiali e capability normative non supportate,
  lookup metadata/tempo, regole cliniche e storico secondo le decisioni aperte.
  Il profilo SIOC .2 completo conserva persistence.audit e continua a essere
  rifiutato dal compiler: il sottoinsieme golden non aggira tale rifiuto.

La persistenza del summary/audit su file è reale; il repository storico persistente
core rimane da implementare. Non sono ancora consegnati prodotto completo,
certificazione SIOC/CAD, UI o distribuzione finale.

## Workflow

Commit e push sul branch work/p1-foundations autorizzati, CI automatica entro tre
chiamate e intervalli di almeno 45 secondi. La ratifica più recente impone uno
stop dopo T-P13-08: nessuna task successiva verrà avviata senza nuova autorizzazione.

## Esito operativo del gate remoto

Codice52762c8525b878a72c7a59f9589c207a17e1efc6 pubblicato. Tre tentativi
contabilizzati: timeout DNS prima dell’API, dispatch accettato, lettura finale
dello SHA esatto in timeout di connessione dopo oltre8 minuti. Nessuna quarta
chiamata. Job Linux/MSVC non verificati, nessun esito presunto. T-P13-08 rimane
aperta soltanto sul gate nativo; implementazione/golden/memoria locali conclusi.
La CI T-P13-07 è invece verificata positiva: run40183843102, entrambi i target.
