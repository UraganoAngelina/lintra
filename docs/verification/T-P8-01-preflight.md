# T-P8-01 — preflight DEC-12, politica input esposta dalla CLI

2026-10-21. **A APPROVATA** dall’autore: «A: confermo il comportamento esistente (consigliata)».
T-P7-06 è verificata localmente e in CI nativa Linux/Windows 39621523482; nessun codice CLI
nuovo scritto. Il mandato ripristina l’avanzamento autonomo solo senza ambiguità.

## Decisione ancora aperta nelle fonti

Master Plan §13.1 assegna DEC-12 all’autore e la elenca come condizionante di
T-P8-01: righe vuote/finali, default BomPolicy, esistenza di `--bom strip`.
La scheda P8 §9 la riporta fra i blocchi. DET §30 raccomanda Reject come default
iniziale, ma dice `--bom strip` solo «se prevista». Le task reader precedenti
hanno usato il default provvisorio autorizzato dal piano, senza chiudere DEC-12.

Stato reale: RecordReader conserva una riga vuota come record di zero byte e
non inventa una riga aggiuntiva dopo il terminatore finale. TraceValidator usa
Reject esplicito; ValidationRequest/Options non espongono BomPolicy. Input layer
ha già StripKnownBom, ma il servizio non permette di selezionarlo. Proporre
`--bom strip` ora senza wiring applicativo reale sarebbe un parametro ignorato.

## A — confermare il comportamento esistente per la prima CLI (consigliata)

- Riga vuota: record di zero byte, E_RECORD_LENGTH secondo lo schema selezionato.
- Un solo terminatore dell’ultima riga non crea un record vuoto aggiuntivo;
  due terminatori consecutivi rappresentano invece una riga vuota reale.
- BOM: Reject; è errore tecnico prima del framing, senza summary definitivo.
- Non esporre `--bom strip` nella prima CLI. Non accettare opzioni ignorate.
  L’eventuale esposizione futura richiederà un vero parametro applicativo e test.

La scelta conferma il profilo tecnico già verificato e non cambia il core,
le baseline normative o le regole di flusso. UTF-8 rimane una scelta di encoding
esplicita, non una deduzione dalla presenza del BOM.

## B — esporre subito stripping esplicito

Prima dell’implementazione CLI occorre ratificare un contratto minimo che
trasporti BomPolicy dalla richiesta fino al TraceValidator, preservando Reject
come default e hashing sui byte originali prima dello strip. Servono test reali
per BOM, audit, cancellazione e portabilità; non basta aggiungere un flag clap.
Non viene modificata autonomamente la firma della request.

## Handoff

DEC-12 RISOLTA per la prima CLI, scelta A ratificata. T-P8-01 autorizzata. Nessuna
modifica di ARCH/DET/Master Plan, nessun crate/flag/fallback introdotto.
T-P7-06 conclusa DONE LocalOnly, prerequisiti soddisfatti. Il seguito conserva
la ricostruzione storica della decisione; alternativa B non approvata.
