# Decisioni normative ricevute

## DEC-05-CAD-ICPC-000 — RISOLTA, 2026-10-19

Ambito: CAD B1, campo `2.12`, patologia concomitante.

L'autore ha confermato: **«Confermo: 000 valido, spazi mancanti»**.

- Il campo resta obbligatorio.
- `000` è un valore valido quando non è presente una patologia concomitante.
- Tre spazi ASCII sono un dato fisicamente mancante.
- Non viene introdotto un lookup ICPC, già escluso dalle decisioni ricevute.

Policy di assenza da adottare nella configurazione versionata pertinente:

```json
{"kind": "spaces", "raw": "   "}
```

La presenza rimane `required`; tipo fisico e padding non sono cambiati.
Il kit ricevuto e le tabelle che ne attestano i valori restano inalterati.
La policy legacy `exact: 000` è documentata come valore ricevuto superato
da questa decisione, da correggere nella task di schema pertinente.

Fonti ed evidenze: [T-P0-03](../verification/T-P0-03.md),
[matrice note CAD](cad/notes-matrix.md#dec-05-cad-icpc-000--risolta-dallautore).

## DEC-05-SIOC-ID-2026 — RISOLTA, 2026-10-19

Ambito: SIOC, campi `1.1`, `2.1`, `3.1`, per i ricoveri iniziati nel
2025 e terminati nel 2026.

L'autore ha precisato:

> per i ricoveri iniziati nel 2025 e terminati nel 2026 non c'è bisogno di controllare tale conformazione (2 cifre per l'anno e 6 per il progressivo) controlleremo solo che sia un campo alfanumerico (valorizzato con qualcosa di diverso da tutti zeri e che rispetti le regole di padding di quel campo)

Per questo ambito:

- Il valore logico è alfanumerico, valorizzato e diverso da una sequenza
  composta interamente da zeri; il confronto esclude il padding fisico.
- Restano larghezza fisica di 20 byte, allineamento a sinistra e padding
  con spazi ASCII, già comuni ai tre campi nelle fonti ricevute.
- Non si applicano lunghezza logica esatta otto, sole cifre, prefisso anno
  o controllo delle sei cifre del progressivo.
- Un campo di soli spazi resta mancante: l'obbligo ricevuto è conservato.

La deroga riguarda esclusivamente il caso indicato; non estende il regime
legacy ad altri anni o episodi. Non introduce decisioni sull'unicità tra invii.
La selezione del ramo normativo e i controlli saranno dichiarati nello schema
versionato pertinente, senza logica specifica SIOC nel core. Il kit FROZEN e
le tabelle che ne attestano il contenuto restano inalterati.

Fonti ed evidenze: [matrice note SIOC](sioc/notes-matrix.md#dec-05-sioc-id-2026--risolta-dallautore),
[T-P0-04](../verification/T-P0-04.md).

## DEC-05-CAD-VALUTAZIONE-INIZIALE — RISOLTA, 2026-10-19

Ambito: CAD B1, campo `2.8`, almeno una valutazione iniziale per evento.

Alla domanda «nell’intero storico disponibile (incluso l’invio corrente)
oppure in ogni singolo invio?» l’autore ha risposto: **«Nello storico
dell’evento»**.

- Cercare almeno una B1 con `2.8 = 1` nello storico dell’evento, includendo
  l’invio corrente, nel namespace ULSS pertinente.
- Un invio con sole rivalutazioni non viola questo obbligo se l’iniziale
  risulta nello storico del medesimo evento.
- La decisione riguarda l’esistenza, non impone esattamente una iniziale.
- Contesto o tracciato indispensabile indisponibile non dimostrano una
  violazione: applicare la distinzione `Unknown` / `NOT_EVALUABLE` già
  prevista dal contratto, senza modificare lo storico durante la verifica.

La regola verrà espressa nel profilo versionato, usando la persistenza core
e la preview autorizzata; non viene implementata né aggiunta ai JSON ricevuti
in T-P0-05. Nessuna nuova eccezione normativa è dedotta dalla risposta.

Evidenze: [regole e join CAD](cad/rules-and-joins.md),
[matrice note CAD](cad/notes-matrix.md#dec-05-cad-valutazione-iniziale--risolta-dallautore).
