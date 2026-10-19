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
