# SIOC — bozza strutturale minimale 1

**DRAFT, T-P0-08. Non è un validatore eseguibile o una certificazione normativa.**
Decisione dell’autore: [DEC-SIOC-MINIMAL-01](../../../docs/normative/DECISIONS.md).

`flow_schema.json` è autorevole; `trace-*.json` sono proiezioni identiche dei
cinque trace, da leggere con le definizioni `trace` del meta-schema formato 1.
`scope-ledger.json` conserva ciascun constraint, regola e gruppo sorgente con
la sua disposizione. Non è input eseguibile del compiler. Il loader futuro
non deve caricare tutti i JSON della directory indiscriminatamente.
Versione normativa 2026-04-20; metadato originale dei trace 2025-12-16.
Nessuna modifica ai JSON FROZEN dello starter kit.

| Tracciato | Campi | Lunghezza byte, escluso terminatore |
|---|---:|---:|
| 1 | 18 | 206 |
| 2 | 50 | 271 |
| 3 | 36 | 135 |
| 4.1 | 20 | 394 |
| 4.2 | 11 | 67 |

Sono attivi posizione/lunghezza, tipi, date di calendario, padding, missing
per campo, presenza base/condizionale, cinque PK e relazioni referenziali.
AN conserva testo secondo encoding dichiarato; non equivale ad ASCII-only.
Gli zeri dei codici numerici restano significativi (`preserve_width`).
Gli ID 1.1/2.1/3.1 restano entro i 20 byte, obbligatori, con padding a spazi
e diversi da soli zeri; anno/progressivo e lunghezza logica 8 sono rinviati
in questa revisione minimale. Nessun obbligo di 8 cifre viene applicato agli
ID dei ricoveri iniziati nel 2025.

Semantica attiva limitata a `050` sui tre prefissi dell’azienda inviante e
su 4.7. Restano i vincoli di rappresentazione dichiarati (lunghezze logiche,
cifre, bitset 3.19 e sue posizioni riservate, campi non vuoti).
Le condizioni di obbligatorietà mantengono i confronti sui codici necessari
alla loro valutazione; l’appartenenza dei codici a domini clinici è rinviata.
I gruppi mantengono l’obbligatorietà dei singoli slot tramite i campi;
controlli aggiuntivi su sentinelle/duplicazione diagnosi sono rinviati.
I dodici lookup, i domini clinici, età, ordinamenti temporali e altre regole
semantiche risultano esplicitamente non eseguibili. Nessun binding fittizio.

I cinque join ricevuti conservano cardinalità e ordine delle chiavi.
Anagrafica senza ricovero: NotApplicable e messaggio esplicito approvato.
Il sesto join `revoking_act` esprime l’esistenza di 4.30 nell’insieme dei 4.22,
senza aggiungere uno scope UDO non presente nella fonte. È zero_or_many:
la regola guardata da 4.30 presente impone `record_exists`, senza imporre
un parent quando 4.30 è mancante. Nessuna selezione arbitraria di un record.

PK/audit persistenti sono richiesti per tutti i trace, senza snapshot o
regole cliniche storiche. `organisation_namespace` è un identificatore tecnico
positivo del tenant/ULSS configurato dal chiamante: non un nuovo codice
normativo, nessun default, stessa ULSS deve riusare lo stesso identificatore.
Credenziali e protezioni restano nella configurazione esterna. Compatibilità
limitata alla stessa versione di schema. Le capability richieste devono
essere implementate integralmente prima dell’esecuzione; oggi non lo sono.

Encoding esplicito a runtime. L’ispezione aggregata degli export autorizzati
non identifica una codifica univoca né prova l’allineamento ai layout:
[limiti dei campioni](../../../docs/verification/T-P0-08.md).
Fixture e output da produrre: [piano](../../../docs/normative/fixtures-plan.md).
Il lint automatico Rust resta obbligatorio in T-P3-04.
