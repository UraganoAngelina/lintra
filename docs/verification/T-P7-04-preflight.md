# T-P7-04 — Preflight / CNF-RUN-SCOPE-01

2026-10-21. Decisione richiesta, nessun codice T-P7-04 scritto.

## Evidenza e impatto

T-P7-03 verificata: 5d2f371, CI 39571479598 Linux/Windows success.
T-P3-07 già verificata; T-P14-03 fornisce ora il fingerprint reale in locale,
con gate CI separato. Input audit e reader sono disponibili e non sono il blocco.

L'autore, CNF-RECORD-01 §3, impone: «La validazione locale non deve mai essere
presentata come una validazione completa» e «Eventuali report o risultati
applicativi devono rispettare lo scope effettivamente eseguito». Il servizio
T-P7-04 deve chiamare sink.on_summary e restituire ValidationRunOutcome.
Tuttavia RunMetadata/ValidationSummary (DET §57/101) e il serializer corrente
src/report/summary.rs non rappresentano la modalità. scope_summary contiene
solo Trace/Flow e ID, secondo CNF-DOMAIN-01: un Trace può concettualmente essere
usato anche con Full (DET §23–24), quindi non codifica LocalOnly in modo univoco.
Un JSONL sink non può supplire a metadata assenti dal summary del servizio.

Non cambieremo il significato di CompletedValid, non useremo validator_version
come etichetta della modalità, non dichiareremo Full implementato e non
inseriremo testo dentro gli ID. Un commento API o un messaggio CLI esterno non
rende distinguibili i report strutturati esportati separatamente dalla request.
Serve ratifica per estendere i contratti domain/summary e aggiornare i golden
tecnici; non è un nuovo obbligo normativo dei flussi.

## Proposta A — modalità esplicita nei metadata tecnici

1. Collocare il valore neutro ValidationMode { LocalOnly, Full } nel domain,
   con re-export application/root: la request mantiene identica forma/API.
   Nessun tipo applicativo importato dal dominio o dall'engine.
2. Aggiungere validation_mode: ValidationMode a RunMetadata e ValidationSummary.
   La fonte autorevole è il metadata costruito dalla request verificata;
   il builder trasferisce quel valore al summary senza default indipendenti.
3. Scrivere nel summary.json il campo obbligatorio validation_mode:
   "local_only" / "full". Nessuna modifica a RunStatus/Cancelled/truncated,
   digest/input audit o scope_summary e nessun parametro aggiunto agli schemi.
4. T-P7-04 può produrre solo local_only: preflight continua a rifiutare Full.
   L'enum può descrivere il futuro Full, ma non abilita un executor.
5. Aggiornare tutti i metadata dei test con modalità esplicita, le fixture
   summary/golden tecniche e la documentazione del report. I dati normativi,
   condizioni attese, conteggi e byte degli input restano invariati. Test:
   report locale inequivocabile, trasferimento metadata->summary, round-trip
   e determinismo dei byte; Full richiesto resta errore prima dell'elaborazione.
6. Autorizzare poi T-P7-04 nel perimetro LocalOnly già ratificato, con cause
   tecniche engine composte da ValidationRunError applicativo incrementale.
   P11/P13 e T-P2-03/M1 rimangono aperte. Fonti ARCH/DET/Piano invariate;
   registrare la decisione nei report e nel log, non riscrivere le baseline.

La proposta cambia un contratto pubblico di metadata e JSON, e la proprietà
dell'enum neutro, quindi viene sottoposta all'autore secondo Piano §14.2/14.4.

## Alternativa B

Mantenere il blocco e definire con l'autore una rappresentazione alternativa
che renda distinguibile la modalità negli output strutturati, preservando
CNF-DOMAIN-01 e i confini dei moduli. Nessuna modifica implicita dei contratti.

## Handoff

T-P14-03 e la sua CI proseguono indipendentemente. Nessun codice, campo JSON
o golden modificato per T-P7-04. T-P7-04 e dipendenti attendono ratifica;
non aggirare il blocco pubblicando un report di validità senza modalità.
