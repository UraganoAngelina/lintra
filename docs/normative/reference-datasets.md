# Onboarding dei reference dataset — T-P0-06

Rilevazione locale: **2026-10-16**. Ambito: dodici dipendenze SIOC e tre
estrazioni CAD presenti nello starter kit `LINTRA_STARTER_KIT_FINAL_20261016`.
Requisiti: [DET §108–109](../DETAILED_DESIGN.md) e
[Piano T-P0-06](../MASTER_IMPLEMENTATION_PLAN.md). Prerequisito:
[inventario T-P0-01](INPUT_INVENTORY.md), verificato e approvato.

Le **15 schede** esplicitano i dieci metadati richiesti o la relativa lacuna.
Il nome descrittivo di una dipendenza non è attestazione del nome ufficiale
di un dataset non consegnato. «Non dichiarato» non significa «non applicabile».
La data del flusso, la revisione dello schema e la data nel filename non
stabiliscono versione, decorrenza o storia delle singole tabelle.
**G0 resta aperto**; questa consegna non definisce il formato canonico runtime,
non carica dati né implementa provider, normalizzatori o lookup temporali.

## Fonti e criterio

I percorsi `K/...` sono sotto
`../../../LINTRA_STARTER_KIT_FINAL_20261016/` rispetto a questo documento.

| Rif. | Fonte consultata | Natura e limite |
|---|---|---|
| S1 | `K/SCHEMA_INPUTS/SIOC_2026-04-20_FROZEN/REFERENCE_DATA_SPEC.md` | Contratto delle dodici dipendenze, chiavi logiche e attributi richiesti; esempi JSON, non dataset ufficiali |
| S2 | Manifest, flow schema e cinque `trace_*.json` nella stessa directory S1 | Schema derivato FROZEN, canonico `1.0.0`, normativa `2026-04-20`; dodici dipendenze `required: true`, `PENDING_DATASET`; campi fisici e vincoli dichiarati |
| C1 | `K/SCHEMA_INPUTS/CAD_2024-06_SOURCE/2025-05-14_FLUSSO_ADI.pdf` | PDF regionale fornito, aggiornamento interno giugno 2024, etichetta file `2025-05-14`, sette pagine |
| C2 | `K/CAD_REVISED_CANDIDATE/manifest.json`, `flow_schema.json`, `trace_b1.json`, `trace_d.json`, `DECISION_REGISTER.md`, `CAD_RUNTIME_PROFILE.json`; `K/MASTER_AGENT_PROMPT.md` | Candidato `RELEASE_CANDIDATE`, canonico `1.0.0`, normativa `2024-06`, revisione `2026-10-16.reviewed-g02-g06`; decisioni CAD-02/08/09/10/16 e G-05 |
| C3 | Tre JSON in `K/CAD_REVISED_CANDIDATE/datasets/`, nominati nelle schede | Estrazioni `SOURCE_EXTRACTED_REVIEWED` da C1 pp. 5–7; nessun manifest temporale autonomo |

L'originale SIOC `SIOC_tracciato_regionale_20260420-definitivo_v05.xlsx` e
**tutti i dodici file dataset SIOC sono mancanti**. S1/S2 dichiarano i requisiti,
non provano i contenuti ufficiali. S2 attribuisce l'XLSX alla Regione del Veneto;
ciò non identifica automaticamente l'autorità o la release di ogni dataset.
Per C1 l'attribuzione alla Regione del Veneto è riportata da C2. Non sono state
cercate fonti esterne, né letti export o contenuti di pazienti.

La lunghezza fisica del campo trasmesso è distinta dalla lunghezza logica
della chiave e dal formato del file sorgente. Padding e slicing del flusso
non autorizzano trasformazioni del dataset. Il profilo di encoding del flusso
SIOC richiesto a runtime da S2 non è l'encoding dei dataset mancanti.
I tre JSON CAD ricevuti sono decodificabili come UTF-8 senza BOM: è una proprietà
dei file del kit verificati, non una dichiarazione regionale sull'encoding di
un futuro file dati. Gli identificatori numerici rimangono codici testuali;
non si eliminano zeri iniziali per deduzione.

## Schede SIOC — dipendenze non consegnate

Per ciascuna scheda lo stato resta **PENDING_DATASET** (S2). Il campo `code`
è solo la proposta generica d'esempio di S1: non è una colonna verificata di
un file ricevuto. Le lacune **S-G1…S-G6**, richiamate singolarmente, sono
elencate nel registro finale.

### `italian-region-codes`

| Metadato | Valore / lacuna |
|---|---|
| ID dataset | `italian-region-codes` |
| Nome ufficiale | Non attestato (S-G2); denominazione descrittiva: Codici delle regioni italiane |
| Versione / data validità | Non consegnata (S-G1); `2026-04-20` e `1.0.0` appartengono al flusso/schema S2 |
| Fonte | Dichiarazione S1/S2; file ufficiale, release e provenienza del singolo dataset mancanti (S-G1/S-G2) |
| Formato sorgente | Non disponibile (S-G3); gli esempi JSON di S1 non attestano il formato ufficiale |
| Campo chiave | codice regione a 3 caratteri (S1); colonna sorgente non disponibile (S-G3) |
| Lunghezza codice | 3 caratteri logici (S1); campi/segmenti S2 di 3 byte |
| Encoding | Non dichiarato per il dataset (S-G3); nessun file ufficiale ricevuto |
| Normalizzazione ammessa | Non dichiarata per il dataset (S-G4); nessuna trasformazione implicita autorizzata da S1/S2. |
| Validità temporale | Decorrenza, termine, supersessione e storicizzazione non dichiarati (S-G5); nessuna data di riferimento per lookup dedotta |
| Attributi / uso / lacune specifiche | Campi regione, inclusi 1.13, 2.29, 4.7 e segmento 2.26. I valori speciali di schema non sono righe del dataset. |

### `health-authority-codes`

| Metadato | Valore / lacuna |
|---|---|
| ID dataset | `health-authority-codes` |
| Nome ufficiale | Non attestato (S-G2); denominazione descrittiva: Codici delle aziende sanitarie ULSS/ASL |
| Versione / data validità | Non consegnata (S-G1); `2026-04-20` e `1.0.0` appartengono al flusso/schema S2 |
| Fonte | Dichiarazione S1/S2; file ufficiale, release e provenienza del singolo dataset mancanti (S-G1/S-G2) |
| Formato sorgente | Non disponibile (S-G3); gli esempi JSON di S1 non attestano il formato ufficiale |
| Campo chiave | codice azienda sanitaria a 3 caratteri (S1); colonna sorgente non disponibile (S-G3) |
| Lunghezza codice | 3 caratteri logici (S1); 3 byte nei campi/segmenti S2 |
| Encoding | Non dichiarato per il dataset (S-G3); nessun file ufficiale ricevuto |
| Normalizzazione ammessa | Non dichiarata per il dataset (S-G4); nessuna trasformazione implicita autorizzata da S1/S2. |
| Validità temporale | Decorrenza, termine, supersessione e storicizzazione non dichiarati (S-G5); nessuna data di riferimento per lookup dedotta |
| Attributi / uso / lacune specifiche | Lookup piatto: S1/S2 escludono una gerarchia regione → ULSS implicita. S2: 2.30, 4.8, segmenti dei codici azienda e COT; non aggiungere relazioni territoriali non richieste. |

### `istat-municipality-codes`

| Metadato | Valore / lacuna |
|---|---|
| ID dataset | `istat-municipality-codes` |
| Nome ufficiale | Non attestato (S-G2); denominazione descrittiva: Codici ISTAT dei comuni italiani |
| Versione / data validità | Non consegnata (S-G1); `2026-04-20` e `1.0.0` appartengono al flusso/schema S2 |
| Fonte | Dichiarazione S1/S2; file ufficiale, release e provenienza del singolo dataset mancanti (S-G1/S-G2) |
| Formato sorgente | Non disponibile (S-G3); gli esempi JSON di S1 non attestano il formato ufficiale |
| Campo chiave | codice comune ISTAT a 6 caratteri (S1); colonna sorgente non disponibile (S-G3) |
| Lunghezza codice | 6 caratteri logici (S1); S2 1.15, 4.9, 4.14: 6 byte |
| Encoding | Non dichiarato per il dataset (S-G3); nessun file ufficiale ricevuto |
| Normalizzazione ammessa | Non dichiarata per il dataset (S-G4); nessuna trasformazione implicita autorizzata da S1/S2. |
| Validità temporale | Decorrenza, termine, supersessione e storicizzazione non dichiarati (S-G5); nessuna data di riferimento per lookup dedotta |
| Attributi / uso / lacune specifiche | 1.15 ha lookup condizionale e rappresentazione estera distinta. La nota sorgente di 1.15 in S2 raccomanda di conservare lo zero iniziale della provincia. Storia dei comuni non fornita, non dedotta. |

### `foreign-country-codes`

| Metadato | Valore / lacuna |
|---|---|
| ID dataset | `foreign-country-codes` |
| Nome ufficiale | Non attestato (S-G2); denominazione descrittiva: Codici degli Stati esteri |
| Versione / data validità | Non consegnata (S-G1); `2026-04-20` e `1.0.0` appartengono al flusso/schema S2 |
| Fonte | Dichiarazione S1/S2; file ufficiale, release e provenienza del singolo dataset mancanti (S-G1/S-G2) |
| Formato sorgente | Non disponibile (S-G3); gli esempi JSON di S1 non attestano il formato ufficiale |
| Campo chiave | codice Stato estero a 3 caratteri (S1); colonna sorgente non disponibile (S-G3) |
| Lunghezza codice | 3 caratteri logici (S1); 1.11: 3 byte, segmento estero di 1.15: 3 byte entro campo di 6 |
| Encoding | Non dichiarato per il dataset (S-G3); nessun file ufficiale ricevuto |
| Normalizzazione ammessa | Non dichiarata per il dataset (S-G4); nessuna trasformazione implicita autorizzata da S1/S2. |
| Validità temporale | Decorrenza, termine, supersessione e storicizzazione non dichiarati (S-G5); nessuna data di riferimento per lookup dedotta |
| Attributi / uso / lacune specifiche | Cittadinanza e residenza estera. S2 1.15 cita i codici del Ministero dell’Interno per l’anagrafe della popolazione: è un richiamo nella nota del derivato, non consegna di file/versione/provenienza (S-G2). |

### `sts11-structure-codes`

| Metadato | Valore / lacuna |
|---|---|
| ID dataset | `sts11-structure-codes` |
| Nome ufficiale | Non attestato (S-G2); denominazione descrittiva: Codici delle strutture STS11 |
| Versione / data validità | Non consegnata (S-G1); `2026-04-20` e `1.0.0` appartengono al flusso/schema S2 |
| Fonte | Dichiarazione S1/S2; file ufficiale, release e provenienza del singolo dataset mancanti (S-G1/S-G2) |
| Formato sorgente | Non disponibile (S-G3); gli esempi JSON di S1 non attestano il formato ufficiale |
| Campo chiave | codice struttura a 6 caratteri (S1); colonna sorgente non disponibile (S-G3) |
| Lunghezza codice | 6 caratteri logici (S1); campi/segmenti S2 di 6 byte |
| Encoding | Non dichiarato per il dataset (S-G3); nessun file ufficiale ricevuto |
| Normalizzazione ammessa | Non dichiarata per il dataset (S-G4); nessuna trasformazione implicita autorizzata da S1/S2. |
| Validità temporale | Decorrenza, termine, supersessione e storicizzazione non dichiarati (S-G5); nessuna data di riferimento per lookup dedotta |
| Attributi / uso / lacune specifiche | Identificativi STS11 (1.18, 2.50, 3.36, 4.19.1 e segmento 2.26). In 2.31 è uno dei tre dataset alternativi, insieme a HSP11 e RIA11; conservare le identità separate. |

### `hsp11-structure-codes`

| Metadato | Valore / lacuna |
|---|---|
| ID dataset | `hsp11-structure-codes` |
| Nome ufficiale | Non attestato (S-G2); denominazione descrittiva: Codici delle strutture HSP11 |
| Versione / data validità | Non consegnata (S-G1); `2026-04-20` e `1.0.0` appartengono al flusso/schema S2 |
| Fonte | Dichiarazione S1/S2; file ufficiale, release e provenienza del singolo dataset mancanti (S-G1/S-G2) |
| Formato sorgente | Non disponibile (S-G3); gli esempi JSON di S1 non attestano il formato ufficiale |
| Campo chiave | codice struttura a 6 caratteri (S1); colonna sorgente non disponibile (S-G3) |
| Lunghezza codice | 6 caratteri logici (S1); campo S2 2.31 di 6 byte |
| Encoding | Non dichiarato per il dataset (S-G3); nessun file ufficiale ricevuto |
| Normalizzazione ammessa | Non dichiarata per il dataset (S-G4); nessuna trasformazione implicita autorizzata da S1/S2. |
| Validità temporale | Decorrenza, termine, supersessione e storicizzazione non dichiarati (S-G5); nessuna data di riferimento per lookup dedotta |
| Attributi / uso / lacune specifiche | Provenienza in 2.31: unione con RIA11 e STS11, oltre al valore speciale 999999 dichiarato nello schema; non concatenare o contaminare i dataset ufficiali. |

### `ria11-structure-codes`

| Metadato | Valore / lacuna |
|---|---|
| ID dataset | `ria11-structure-codes` |
| Nome ufficiale | Non attestato (S-G2); denominazione descrittiva: Codici delle strutture RIA11 |
| Versione / data validità | Non consegnata (S-G1); `2026-04-20` e `1.0.0` appartengono al flusso/schema S2 |
| Fonte | Dichiarazione S1/S2; file ufficiale, release e provenienza del singolo dataset mancanti (S-G1/S-G2) |
| Formato sorgente | Non disponibile (S-G3); gli esempi JSON di S1 non attestano il formato ufficiale |
| Campo chiave | codice struttura a 6 caratteri (S1); colonna sorgente non disponibile (S-G3) |
| Lunghezza codice | 6 caratteri logici (S1); campo S2 2.31 di 6 byte |
| Encoding | Non dichiarato per il dataset (S-G3); nessun file ufficiale ricevuto |
| Normalizzazione ammessa | Non dichiarata per il dataset (S-G4); nessuna trasformazione implicita autorizzata da S1/S2. |
| Validità temporale | Decorrenza, termine, supersessione e storicizzazione non dichiarati (S-G5); nessuna data di riferimento per lookup dedotta |
| Attributi / uso / lacune specifiche | Provenienza in 2.31: unione con HSP11 e STS11, oltre al valore speciale 999999 dichiarato nello schema; non concatenare o contaminare i dataset ufficiali. |

### `regional-udo-codes`

| Metadato | Valore / lacuna |
|---|---|
| ID dataset | `regional-udo-codes` |
| Nome ufficiale | Non attestato (S-G2); denominazione descrittiva: Codici regionali delle UDO |
| Versione / data validità | Non consegnata (S-G1); `2026-04-20` e `1.0.0` appartengono al flusso/schema S2 |
| Fonte | Dichiarazione S1/S2; file ufficiale, release e provenienza del singolo dataset mancanti (S-G1/S-G2) |
| Formato sorgente | Non disponibile (S-G3); gli esempi JSON di S1 non attestano il formato ufficiale |
| Campo chiave | codice UDO a 6 caratteri (S1); colonna sorgente non disponibile (S-G3) |
| Lunghezza codice | 6 caratteri logici (S1); S2 2.8, 4.1, 4.20: 6 byte, N allineato a destra con zero |
| Encoding | Non dichiarato per il dataset (S-G3); nessun file ufficiale ricevuto |
| Normalizzazione ammessa | Non dichiarata per il dataset (S-G4); nessuna trasformazione implicita autorizzata da S1/S2. |
| Validità temporale | Decorrenza, termine, supersessione e storicizzazione non dichiarati (S-G5); nessuna data di riferimento per lookup dedotta |
| Attributi / uso / lacune specifiche | Il padding fisico dei campi S2 è allineamento a destra con zero; non identifica un formato numerico del dataset né una nuova normalizzazione. Nessuna lista ufficiale fornita. |

### `regional-entity-manager-codes-dgr-2961-2012`

| Metadato | Valore / lacuna |
|---|---|
| ID dataset | `regional-entity-manager-codes-dgr-2961-2012` |
| Nome ufficiale | Non attestato (S-G2); denominazione descrittiva: Codici regionali degli Enti gestori richiamati dalla DGR 2961/2012 |
| Versione / data validità | Non consegnata (S-G1); `2026-04-20` e `1.0.0` appartengono al flusso/schema S2 |
| Fonte | Dichiarazione S1/S2; file ufficiale, release e provenienza del singolo dataset mancanti (S-G1/S-G2) |
| Formato sorgente | Non disponibile (S-G3); gli esempi JSON di S1 non attestano il formato ufficiale |
| Campo chiave | codice Ente gestore a 6 caratteri (S1); colonna sorgente non disponibile (S-G3) |
| Lunghezza codice | 6 caratteri logici (S1); S2 4.11: 6 byte, N allineato a destra con zero |
| Encoding | Non dichiarato per il dataset (S-G3); nessun file ufficiale ricevuto |
| Normalizzazione ammessa | Non dichiarata per il dataset (S-G4); nessuna trasformazione implicita autorizzata da S1/S2. |
| Validità temporale | Decorrenza, termine, supersessione e storicizzazione non dichiarati (S-G5); nessuna data di riferimento per lookup dedotta |
| Attributi / uso / lacune specifiche | Il richiamo DGR nel dataset ID non costituisce una versione ricevuta o una data di efficacia. Il campo fisico non definisce il formato del dataset. |

### `icd9-cm-diagnosis-codes`

| Metadato | Valore / lacuna |
|---|---|
| ID dataset | `icd9-cm-diagnosis-codes` |
| Nome ufficiale | Non attestato (S-G2); denominazione descrittiva: Codici diagnosi ICD9-CM per SIOC |
| Versione / data validità | Non consegnata (S-G1); `2026-04-20` e `1.0.0` appartengono al flusso/schema S2 |
| Fonte | Dichiarazione S1/S2; file ufficiale, release e provenienza del singolo dataset mancanti (S-G1/S-G2) |
| Formato sorgente | Non disponibile (S-G3); gli esempi JSON di S1 non attestano il formato ufficiale |
| Campo chiave | codice diagnosi ICD9-CM (S1); `code` nell’esempio di proiezione, colonna sorgente non disponibile (S-G3) |
| Lunghezza codice | Lunghezza logica del catalogo non dichiarata (S-G3); S2 2.32–2.37: 7 byte AN con padding spazio |
| Encoding | Non dichiarato per il dataset (S-G3); nessun file ufficiale ricevuto |
| Normalizzazione ammessa | Non dichiarata per il dataset (S-G4); nessuna trasformazione implicita autorizzata da S1/S2. La rimozione del punto e la gestione del prefisso V approvate per CAD non sono trasferite a SIOC; l’esempio diagnosi `250.00` in S1 non è una regola di normalizzazione. |
| Validità temporale | Decorrenza, termine, supersessione e storicizzazione non dichiarati (S-G5); nessuna data di riferimento per lookup dedotta |
| Attributi / uso / lacune specifiche | Non ridurre a membership: S1 richiede almeno `code`, `valid`, `sex_constraint`, `min_age`, `max_age`; appartenenza, compatibilità sesso/età quando dichiarate. S2 richiede controlli sesso/età; unicità dei sei valori non mancanti è vincolo di schema. Metadati autorevoli non consegnati (S-G6). |

### `icd9-cm-procedure-codes`

| Metadato | Valore / lacuna |
|---|---|
| ID dataset | `icd9-cm-procedure-codes` |
| Nome ufficiale | Non attestato (S-G2); denominazione descrittiva: Codici procedure ICD9-CM per SIOC |
| Versione / data validità | Non consegnata (S-G1); `2026-04-20` e `1.0.0` appartengono al flusso/schema S2 |
| Fonte | Dichiarazione S1/S2; file ufficiale, release e provenienza del singolo dataset mancanti (S-G1/S-G2) |
| Formato sorgente | Non disponibile (S-G3); gli esempi JSON di S1 non attestano il formato ufficiale |
| Campo chiave | codice procedura ICD9-CM (S1); colonna sorgente non disponibile (S-G3) |
| Lunghezza codice | Lunghezza logica del catalogo non dichiarata (S-G3); S2 2.38–2.48: 8 byte AN con padding spazio |
| Encoding | Non dichiarato per il dataset (S-G3); nessun file ufficiale ricevuto |
| Normalizzazione ammessa | Non dichiarata per il dataset (S-G4); nessuna trasformazione implicita autorizzata da S1/S2. La rimozione del punto e la gestione del prefisso V approvate per CAD non sono trasferite a SIOC; l’esempio diagnosi `250.00` in S1 non è una regola di normalizzazione. |
| Validità temporale | Decorrenza, termine, supersessione e storicizzazione non dichiarati (S-G5); nessuna data di riferimento per lookup dedotta |
| Attributi / uso / lacune specifiche | S1 richiede membership in canonico v1; non trasferire automaticamente i vincoli sesso/età delle diagnosi alle procedure. |

### `icpc-codes`

| Metadato | Valore / lacuna |
|---|---|
| ID dataset | `icpc-codes` |
| Nome ufficiale | Non attestato (S-G2); denominazione descrittiva: Codici ICPC per patologie SVAMA |
| Versione / data validità | Non consegnata (S-G1); `2026-04-20` e `1.0.0` appartengono al flusso/schema S2 |
| Fonte | Dichiarazione S1/S2; file ufficiale, release e provenienza del singolo dataset mancanti (S-G1/S-G2) |
| Formato sorgente | Non disponibile (S-G3); gli esempi JSON di S1 non attestano il formato ufficiale |
| Campo chiave | codice ICPC (S1); colonna sorgente non disponibile (S-G3) |
| Lunghezza codice | S1 non dichiara la lunghezza logica del catalogo (S-G3); S2 3.5–3.7: 3 byte AN |
| Encoding | Non dichiarato per il dataset (S-G3); nessun file ufficiale ricevuto |
| Normalizzazione ammessa | Non dichiarata per il dataset (S-G4); nessuna trasformazione implicita autorizzata da S1/S2. |
| Validità temporale | Decorrenza, termine, supersessione e storicizzazione non dichiarati (S-G5); nessuna data di riferimento per lookup dedotta |
| Attributi / uso / lacune specifiche | Patologia prevalente 3.5 e concomitanti 3.6–3.7. Il lookup SIOC non è rimosso dalla decisione CAD di non effettuare lookup ICPC. |

## Schede CAD — estrazioni regionali ricevute

Per queste tre schede lo stato del JSON ricevuto è
**SOURCE_EXTRACTED_REVIEWED**, distinto dallo stato **RELEASE_CANDIDATE** dello
schema C2. Il contesto documentale comune è C1/C2 (`2024-06`, filename
`2025-05-14`); non è una release autonoma delle tabelle.

### `cad-icd9-cm-clinical-signs`

| Metadato | Valore / lacuna |
|---|---|
| ID dataset | `cad-icd9-cm-clinical-signs` |
| Nome ufficiale | Titolo della tabella C1 p. 7: «Tabella 1 - Segni e sintomi clinici»; nome ufficiale di dataset autonomo non dichiarato (C-G1) |
| Versione / data validità | C1 aggiornamento giugno 2024, etichetta file `2025-05-14`; C3 non dichiara versione autonoma o data di validità (C-G1) |
| Fonte | C1 p. 7; C3 `cad_icd9_cm_symptoms_page7.json`, con `source`, `source_page: 7`, `status`; decisione CAD-10/16 e G-05 in C2 |
| Formato sorgente | Tabella incorporata nel PDF; estrazione ricevuta JSON con 63 `entries`, non catalogo ICD-9-CM completo |
| Campo chiave | `transmitted_code_no_dot` per il lookup trasmesso; conservato `source_code` con punto per risalire alla fonte |
| Lunghezza codice | Nei 63 record ricevuti: `source_code` di 5–6 caratteri, `transmitted_code_no_dot` di 4–5; C2 2.29–2.33: 5 byte AN con padding spazio |
| Encoding | C3 letto come UTF-8 senza BOM; encoding di un file regionale autonomo non dichiarato (C-G3); la fonte originale è PDF |
| Normalizzazione ammessa | C3/CAD-10/16: rimuovere esclusivamente il punto letterale, conservare il prefisso V; esempio sorgente `V66.7` → `V667`. La dicitura PDF «soli valori numerici» contrasta con le voci V della stessa tabella; la decisione approvata conserva V. Nessun’altra trasformazione dedotta |
| Validità temporale | Nessun `effective_from`/`effective_to`, intervallo per voce, supersessione o storia nel JSON (C-G2); non usare date del flusso/file come efficacia |
| Attributi / uso / lacune specifiche | Conservare `description`, `source_code`, `transmitted_code_no_dot`, `source_page`, `contains_alpha_prefix`, oltre ai metadati di dataset. Il JSON conserva due `open_catalogue_fallbacks`: «Altri segni o sintomi: codificare con ICD9 CM» e «Altri motivi non clinici: codificare con ICD9 CM». C2 limita il lookup alle voci enumerate a p. 7: tali frasi non autorizzano codici arbitrari o un catalogo nazionale non fornito. Limite aperto C-G4 |

### `cad-prestation-types`

| Metadato | Valore / lacuna |
|---|---|
| ID dataset | `cad-prestation-types` |
| Nome ufficiale | Elenco «Tipo prestazione», campo 5.5 di C1 p. 5; nome ufficiale di dataset autonomo non dichiarato (C-G1) |
| Versione / data validità | C1 aggiornamento giugno 2024, etichetta file `2025-05-14`; C3 non dichiara versione autonoma o data di validità (C-G1) |
| Fonte | C1 p. 5; C3 `cad_prestations_type_5_5_page5.json`, `source_page: 5`, `field_id: 5.5`; decisioni CAD-02/09/10/16 e C2 trace D |
| Formato sorgente | Elenco nel PDF; estrazione JSON con 33 `entries`: codici 01–28, 71–74 e 99 |
| Campo chiave | `code` testuale a due caratteri; `source_code` conserva il codice del PDF |
| Lunghezza codice | `source_code` di 1–2 caratteri; `code` di 2; C3 `physical_width: 2`, C2 5.5: 2 byte N |
| Encoding | C3 letto come UTF-8 senza BOM; encoding di un file regionale autonomo non dichiarato (C-G3); la fonte originale è PDF |
| Normalizzazione ammessa | C3 rappresenta i codici a una cifra con zero iniziale (`1` → `01`); C2 5.5 applica allineamento a destra e padding zero (CAD-02/09). Conservare `source_code`; nessun’altra trasformazione dichiarata |
| Validità temporale | Nessun `effective_from`/`effective_to`, intervallo per voce, supersessione o storia nel JSON (C-G2); non usare date del flusso/file come efficacia |
| Attributi / uso / lacune specifiche | Conservare `label`, `source_code`, `source_page`, `field_id`, `physical_width` e metadati di dataset. `99` è una voce effettiva della tabella («non disponibile *»); `00` è assenza del campo 5.5 in C2, non una voce da aggiungere al dataset |

### `cad-needs-70-positions`

| Metadato | Valore / lacuna |
|---|---|
| ID dataset | `cad-needs-70-positions` |
| Nome ufficiale | Titolo C1 p. 6: «CLASSIFICAZIONE PROBLEMI DI SALUTE / BISOGNI E PRESTAZIONI ASSISTENZIALI», richiamata dalla nota di 2.22 come allegato D - Bisogni_NEW; nome ufficiale di dataset autonomo non dichiarato (C-G1) |
| Versione / data validità | C1 aggiornamento giugno 2024, etichetta file `2025-05-14`; C3 non dichiara versione autonoma o data di validità (C-G1) |
| Fonte | C1 p. 6; C3 `cad_needs_70_positions_page6.json`, `source_page: 6`, `field_id: 2.22`; decisioni CAD-08/09 e formalizzazione di 2.22 in C2 |
| Formato sorgente | Tabella posizionale nel PDF; JSON con 70 `entries`, una per ogni posizione 1–70. Non è un insieme di 70 codici da cercare per membership |
| Campo chiave | `position`, intero one-based 1–70 che identifica il carattere del campo; `label` non è chiave e può ripetersi |
| Lunghezza codice | Non applicabile come codice di catalogo: ogni sotto-posizione trasporta un carattere, campo 2.22 intero di 70 byte; l’ID di posizione non è il valore trasmesso |
| Encoding | C3 letto come UTF-8 senza BOM; encoding di un file regionale autonomo non dichiarato (C-G3). Il candidato C2 dichiara lo spazio ASCII per la posizione 60 fuori UCPDOM |
| Normalizzazione ammessa | Nessuna conversione di codici dichiarata; preservare indice, ordine, dominio e condizione di ogni posizione. CAD-09 riguarda il padding fisico, non autorizza riordino, deduplicazione o riduzione a set |
| Validità temporale | Nessun `effective_from`/`effective_to`, intervallo per posizione, supersessione o storia nel JSON (C-G2); non usare date del flusso/file come efficacia |
| Attributi / uso / lacune specifiche | Conservare `position`, `label`, `default_domain`, `conditional`, `source_page` e metadati di dataset. Posizioni 1–59: valori 1/2; 61–70: fisso 2. C3 posizione 60 ha dominio 1/2 e `conditional` con rappresentazione fuori UCPDOM ancora `pending`; C2 2.22 e master prompt formalizzano già UCPDOM → 1/2, fuori UCPDOM → esattamente uno spazio ASCII. La nota pendente del JSON è una divergenza di aggiornamento documentale, non una nuova decisione da inventare o motivo per cambiare C3 in questa task (C-G5) |

## Valori dello schema e limiti dei lookup

S1/S2 separano valori ufficiali e sentinel specifici del flusso: `999` nei
contesti di residenza estera, `999999` per struttura semanticamente mancante
o per senza fissa dimora, `998` per cittadinanza sconosciuta. Anche `100` è
ammesso esplicitamente da S2 per 1.11. L'eventuale presenza autonoma di un
codice in un dataset non ricevuto non è qui stabilita: nessuno di questi valori
va inserito in una tabella ufficiale solo perché ammesso dallo schema.
La residenza estera di 1.15 usa prefisso `999` e codice Stato nei tre caratteri
successivi; `999999` per senza fissa dimora è un'eccezione autonoma.
L'unione HSP11/RIA11/STS11 per 2.31 resta distinta dal sentinel `999999`.

CAD non acquisisce lookup ISTAT/ICPC aggiuntivi (CAD-10/16); il catalogo ICD
completo resta fuori dal perimetro CAD corrente. Questo non sostituisce i
dataset SIOC omonimi o correlati. Per C3 prestazioni `99` è una voce sorgente,
mentre `00` è assenza di schema; per C3 bisogni i valori 1/2 e lo spazio
condizionale sono valori di posizione, non righe di un catalogo codici.

S1 distingue dataset richiesto non disponibile da codice paziente non valido:
l'assenza è errore di reference data/engine. È un contratto documentato,
non un comportamento runtime attestato da questa consegna.

## Registro delle lacune e impatto

| ID | Lacuna e ambito | Tracciabilità / limite |
|---|---|---|
| S-G1 | Tutti i 12 dataset SIOC: file ufficiale e versione/data validità mancanti | Inventario S5b/R1a/R2a; mantenere `PENDING_DATASET`; nessuna popolazione da esempi |
| S-G2 | Tutti i 12: nome ufficiale, provenienza autorevole e release non attestati | Inventario R3b; nomi e usi S1/S2 non sono ricevute di dati ufficiali |
| S-G3 | Tutti i 12: formato sorgente, colonna chiave effettiva ed encoding non verificabili; per ICD9/ICPC manca anche lunghezza logica del catalogo | Chiavi attese S1 e larghezze dei campi S2 sono distinte dai metadati sorgente mancanti |
| S-G4 | Tutti i 12: normalizzazione del dataset non dichiarata | Non trasferire regole CAD né inferire trim, cambio maiuscole o rimozione segni |
| S-G5 | Tutti i 12: decorrenze, scadenze, supersessione e storia non fornite | Inventario R4b/R5a; gli esempi `effective_from`, `effective_to`, `valid_from`, `valid_to` di S1 non sono valori ricevuti; non assunta validità illimitata |
| S-G6 | Diagnosi SIOC: valori autorevoli degli attributi `valid`, sesso ed età mancanti | S1/S2 richiedono i metadati; non appiattire a un set di stringhe |
| C-G1 | Tre tabelle CAD: versione/data validità autonoma e nome ufficiale di dataset separato non dichiarati | Inventario R2c; conservare titolo tabella, aggiornamento del documento e nome del file con significati distinti |
| C-G2 | Tre tabelle CAD: efficacia, intervalli per voce/posizione e policy di storicizzazione non dichiarati | Inventario R2c/R4d/R5a; non assumere identità con versione o date del PDF |
| C-G3 | Tre tabelle CAD: encoding di un file regionale autonomo non dichiarato | Ricevuti PDF e JSON derivati UTF-8, non un export ufficiale autonomo con specifica di encoding |
| C-G4 | ICD CAD: due formule aperte al catalogo ICD non enumerate nel JSON | Conservate in `open_catalogue_fallbacks`; lookup corrente limitato alle 63 voci, senza estensione al catalogo completo (C2/G-05) |
| C-G5 | Bisogni CAD: testo pendente in `conditional` posizione 60 non aggiornato alla formalizzazione di schema | Esporre entrambi gli stati; nessuna modifica a fonte/schema o nuova regola in questa task |

L'elenco completa l'onboarding documentale ai sensi dell'accettazione
«scheda completa o lacune elencate». Le assenze impediscono la verifica e l'uso
normativo delle proprietà che richiedono tali fonti; non bloccano questa
rilevazione. DET §109 permette di valutare un lookup con data solo quando
necessario: qui non è stata inventata una policy temporale o una nuova API.
Le richieste all'autore restano quelle dell'inventario: file SIOC e relativi
metadati, chiarimento sulle versioni/storia autonome CAD. Nessuna nuova
obbligazione normativa è introdotta e nessun gate è chiuso.

## Verifica documentale

Confrontati i 12 ID di S2 con S1 e con le schede, e i 3 ID del manifest CAD con
i tre JSON C3 e le relative schede: **15/15**, senza duplicati o ID aggiuntivi.
Ogni scheda ha tutti i dieci metadati di DET §108, con lacune esplicite.
Verificati conteggi C3 (**63 segni/sintomi, 33 prestazioni, 70 posizioni**),
chiavi, lunghezze, normalizzazioni dichiarate, attributi e assenza di campi di
versione/validità temporale. Le osservazioni sui file non equivalgono ad
approvazione normativa o runtime.

Evidenze e controllo riproducibile:
[report T-P0-06](../../.superpowers/sdd/2026-10-16-foundations/task-4-report.md).
`cargo fmt`, `cargo clippy`, `cargo test` non eseguiti per questa task soltanto
documentale; verificata completezza/tracciabilità come previsto dal prompt.
Nessun test runtime artificiale o risultato CI rivendicato.
