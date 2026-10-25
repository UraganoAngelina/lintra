# Lintra — avanzamento al 2026-10-25

**85 task verificate su140:60,7%.** T-P13-08 ha superato anche la CI nativa:
run40192491629, Linux e MSVC completed/success sul commit52762c8.
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

- 53 task non iniziate: tooling schema restante (3), packaging/release (7), UI
  Slint (9), hardening/benchmark (6), documentazione operativa (4), persistenza
  core (12), integrazione CAD (12).
- T-P2-03 resta parziale con residuo approvato relativo ai payload applicativi.
- T-P13-08 e M6 nello scope referenziale ratificato sono verificate: non
  costituiscono certificazione dell’intero SIOC.
- Restano i rinvii G0: dataset ufficiali e capability normative non supportate,
  lookup metadata/tempo, regole cliniche e storico secondo le decisioni aperte.
  Il profilo SIOC .2 completo conserva persistence.audit e continua a essere
  rifiutato dal compiler: il sottoinsieme golden non aggira tale rifiuto.

La persistenza del summary/audit su file è reale; il repository storico persistente
core rimane da implementare. Non sono ancora consegnati prodotto completo,
certificazione SIOC/CAD, UI o distribuzione finale.

## Workflow precedente, superato dal mandato di ripresa sotto

Commit e push sul branch work/p1-foundations autorizzati, CI automatica entro tre
chiamate e intervalli di almeno 45 secondi. La ratifica più recente impone uno
stop dopo T-P13-08: nessuna task successiva verrà avviata senza nuova autorizzazione.

## Storico — esito operativo del gate remoto nella sessione precedente

Codice52762c8525b878a72c7a59f9589c207a17e1efc6 pubblicato. Tre tentativi
contabilizzati: timeout DNS prima dell’API, dispatch accettato, lettura finale
dello SHA esatto in timeout di connessione dopo oltre8 minuti. Nessuna quarta
chiamata. Job Linux/MSVC non verificati, nessun esito presunto. T-P13-08 rimane
aperta soltanto sul gate nativo; implementazione/golden/memoria locali conclusi.
La CI T-P13-07 è invece verificata positiva: run40183843102, entrambi i target.

## Ripresa autorizzata e verifica positiva

Il nuovo mandato supera lo stop dopo13-08. Una GET dello SHA52762c8 ha verificato
CI40192491629: job Linux121166275121 e MSVC121166274875 completed/success.
I timeout descritti sopra restano storici; esito ottenuto alla ripresa senza
nuovo dispatch. T-P13-08 DONE, M6 REACHED nello scope ratificato.
Next T-P14-01 preflight, avanzamento automatico con gate/prerequisiti verificati.

## Stato della prosecuzione14-04 e ricezione sorgente XLSX

T-P14-04 implementata nel commit7024f5f: API diff revisioni con hash e puntatori
per lato, campi aggiunti/rimossi/modificati e ordine,9 nuovi test. Gate locale
620/73 suite positivo; CI nativa non avviata perché push e unico retry sono
stati respinti prima dell’esecuzione per timeout della revisione automatica
dei permessi. Non conteggiata come DONE: progresso resta85/14060,7%.

L'autore ha fornito il workbook SIOC:135 campi, coordinate tutte coincidenti
col profilo approvato,8 note differenti solo per whitespace. Esame e hash nel
[preflight14-01](T-P14-01-preflight.md); nessuna implementazione aggiuntiva
per altri flussi. File utente preservati. Serve sbloccare il push nell’ambiente
per completare il gate14-04 e continuare14-01 senza duplicare codice.
