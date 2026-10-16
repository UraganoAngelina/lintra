# Inventario degli input normativi — T-P0-01

Rilevazione locale: **2026-10-16**. Ambito: SIOC e CAD/ADI dello starter kit
`LINTRA_STARTER_KIT_FINAL_20261016`, adiacente al repository Lintra.
Fonte dei requisiti: [DET §106 e §154](../DETAILED_DESIGN.md),
[Piano §4, §9/P0, T-P0-01 e §14](../MASTER_IMPLEMENTATION_PLAN.md).

L'inventario è completo rispetto alle **6 categorie per gli schemi e 5 per i
reference data** di DET §154. Lo stato di ricezione non certifica la correttezza
normativa, la completezza delle fonti regionali o l'eseguibilità dello schema.
T-P0-01 non ha prerequisiti di task. **G0 resta aperto**: le formalizzazioni di
DET §106 e l'approvazione di T-P0-09 non sono oggetto di questa consegna.

## Criterio e fonti

Gli stati ammessi sono `ricevuto`, `mancante`, `non applicabile`. Le voci con
copertura diversa sono separate: ricevere un JSON derivato non equivale a
ricevere l'originale ufficiale, né una data di aggiornamento dimostra una data
di efficacia. `non applicabile` vale solo per l'ambito e la decisione indicati.
Non sono state scaricate fonti esterne né dedotte regole dai dati dei pazienti.

I riferimenti `K/...` indicano percorsi sotto
`../../../LINTRA_STARTER_KIT_FINAL_20261016/` rispetto a questo documento.

| Rif. | Evidenza consultata | Natura e limite |
|---|---|---|
| F1 | `K/KIT_MANIFEST.json`; `K/VERIFY_KIT.py` | Manifest di consegna e verifica di integrità, non certificazione normativa |
| F2 | `K/SCHEMA_INPUTS/CAD_2024-06_SOURCE/2025-05-14_FLUSSO_ADI.pdf` | PDF regionale fornito: 7 pagine; intestazione «aggiornamento giugno 2024»; fonte primaria CAD disponibile |
| F3 | `K/SCHEMA_INPUTS/SIOC_2026-04-20_FROZEN/manifest.json`, `flow_schema.json`, cinque `trace_*.json`, `FREEZE_REPORT.md`, `SCHEMA_AUTHORING.md`, `REFERENCE_DATA_SPEC.md` | Configurazione derivata SIOC FROZEN; originale XLSX citato ma assente; compiler/runtime non verificati da questi artefatti |
| F4 | `K/CAD_REVISED_CANDIDATE/manifest.json`, `flow_schema.json`, cinque `trace_*.json`, `CAD_RUNTIME_PROFILE.json`, `CAD_HISTORICAL_PROFILE.md`, `DECISION_REGISTER.md` | Formalizzazione CAD `RELEASE_CANDIDATE`, non FROZEN; decisioni e profili applicativi non sostituiscono il PDF |
| F5 | Tre JSON in `K/CAD_REVISED_CANDIDATE/datasets/` | Estrazioni `SOURCE_EXTRACTED_REVIEWED` da F2, non cataloghi esterni completi né manifest temporali di dataset |
| F6 | `K/CAD_FIXTURE_PLAN.md`; directory `tests/` del candidato CAD | Piano fixture e test Python disponibili; nessun bundle di file sintetici fixed-width valid/invalid con golden approvati rilevato |
| F7 | `K/CONFLICT_RESOLUTION_MATRIX.md`; `K/RECONCILIATION_QA_REPORT.md`; copie in `docs/` | ARCH/DET/PLAN riconciliati; le precedenti asserzioni di conflitto di `K/INPUT_INVENTORY.md` sono superate |

F2 è attribuito alla Regione del Veneto anche da F4. Il suo SHA-256 identifica
il file ricevuto; non prova autonomamente autenticità, URL di pubblicazione o
assenza di revisioni successive. Per F3 la Regione del Veneto e il titolo
dell'XLSX sono metadati del derivato: non è stata verificata la fonte originale.
I report QA preesistenti restano evidenza delle attività che dichiarano, non
test Rust/CI né una nuova verifica della validità dei campioni.

## Schemi dei flussi — DET §154, primo elenco

| ID | Categoria | Ambito / materiale | Stato | Motivo, evidenza e lacuna |
|---|---|---|---|---|
| S1a | 1. Tracciati regionali ufficiali completi | CAD A/B1/B2/C/D e allegati | ricevuto | F2 contiene le cinque sezioni e le tabelle alle pp. 5–7. È presente il PDF completo di 7 pagine del kit; non si assume che esaurisca eventuali ulteriori fonti regionali. |
| S1b | 1. Tracciati regionali ufficiali completi | SIOC XLSX ufficiale | mancante | `SIOC_tracciato_regionale_20260420-definitivo_v05.xlsx` è citato in F3 ma nessun XLS/XLSX è presente nel kit. Servono originale e provenienza. |
| S1c | 1. Tracciati regionali ufficiali completi | Schemi derivati SIOC e CAD | ricevuto | F3: cinque trace 1/2/3/4.1/4.2, 135 campi; F4: A/B1/B2/C/D, 83 campi. Sono derivati versionati, distinti dagli originali. |
| S2a | 2. Eventuali documenti di accompagnamento | Note interne al PDF CAD | ricevuto | F2 contiene istruzioni di codifica e allegati; F4 e `K/CAD_NOTES_AUDIT.md` ne documentano la formalizzazione. Audit e decisioni sono materiali di progetto. |
| S2b | 2. Eventuali documenti di accompagnamento | Accompagnamento ufficiale separato CAD/SIOC | mancante | Nessun documento regionale separato identificato nel kit. Occorre consegna oppure conferma dell'autore che non ne esistano per le versioni adottate; l'assenza non è classificata automaticamente `non applicabile`. |
| S3a | 3. Note su versioni/date di efficacia | Versioni e aggiornamenti dichiarati | ricevuto | F3: normativa `2026-04-20`, canonico `1.0.0`; freeze con baseline storica `2025-12-16`. F2/F4: normativa `2024-06`, filename/data etichetta `2025-05-14`, candidato `1.0.0`, revisione `2026-10-16.reviewed-g02-g06`. Queste date hanno significati diversi. |
| S3b | 3. Note su versioni/date di efficacia | Decorrenze e supersessione ufficiali | mancante | Il kit non fornisce un'attestazione separata di efficacia/sostituzione per tutte le fonti né l'originale SIOC. Non usare filename, data di freeze o revisione dello schema come decorrenza normativa. |
| S4a | 4. Esempi di file validi, se reperibili | SIOC e CAD: esempi verificati utilizzabili | mancante | Non rilevati file di esempio con validità normativa attestata, né bundle sintetici approvati fixed-width/golden per tutti i trace. F6 è un piano, non la consegna di tali file. |
| S4b | 4. Esempi di file validi, se reperibili | Export CAD reali A/B1/C/D | ricevuto | Quattro percorsi extra `EXPORT_CAD/{A,B1,C,D}.txt` esistono nel kit locale, fuori manifest. Contenuti non letti, copiati, hashati o versionati; validità non verificata. Non sono fixture approvate. B2 non è presente tra gli export. |
| S5a | 5. Codifiche esterne richiamate | Dichiarazioni SIOC | ricevuto | F3 elenca dodici dipendenze reference (tabella seguente). Ricevuta la dichiarazione, non i dataset ufficiali. |
| S5b | 5. Codifiche esterne richiamate | Dati ufficiali SIOC | mancante | Nessun file di tali dataset rilevato nel kit; `REFERENCE_DATA_SPEC.md` descrive un contratto e richiede fonte autorevole, non fornisce le tabelle. |
| S5c | 5. Codifiche esterne richiamate | Tabelle CAD contenute nel PDF | ricevuto | F2 e F5: prestazioni p. 5, bisogni p. 6, segni/sintomi ICD-9-CM p. 7. La lista p. 7 non è un catalogo ICD completo. |
| S5d | 5. Codifiche esterne richiamate | Lookup CAD ISTAT/ICPC e catalogo ICD completo | non applicabile | Nel perimetro CAD corrente, `DECISION_REGISTER.md` CAD-10/16 esclude lookup ISTAT/ICPC; inventario ereditato differisce ICD completo e limita il lookup al PDF. Questa esclusione non si estende a SIOC né rimuove i fallback aperti annotati nel JSON ICD. |
| S6a | 6. Identificazione delle chiavi di correlazione tra tracciati | CAD | ricevuto | F2 dichiara chiavi/evento; F4 distingue PK, progressivo storico e correlazioni (incluso D→C su evento/data/tipo/figura, contesto ULSS). Ricevuta la formalizzazione con decisioni approvate; verifica compiler/runtime fuori scope. |
| S6b | 6. Identificazione delle chiavi di correlazione tra tracciati | SIOC | ricevuto | F3 contiene `record_key`, join e regole cross-trace nei JSON. Non confondere PK intra-trace con politica cross-run: `DEC-SIOC-HIST-01` resta aperta. Il confronto con XLSX originale resta subordinato a S1b. |

## Reference data — DET §154, secondo elenco

| ID | Categoria | Ambito / materiale | Stato | Motivo, evidenza e lacuna |
|---|---|---|---|---|
| R1a | 1. File ufficiali | SIOC, dodici dataset esterni | mancante | Disponibili solo nomi e contratto F3; nessun file ufficiale consegnato. Elenco puntuale sotto. |
| R1b | 1. File ufficiali | CAD, tabelle incorporate | ricevuto | Originale F2 presente; F5 sono estrazioni JSON delle pp. 5–7. Non sostituiscono cataloghi esterni o dati ufficiali SIOC. |
| R2a | 2. Versione/data | SIOC, versioni dei singoli dataset | mancante | Versione normativa del flusso e template `YYYY-MM-DD-or-authority-version` non costituiscono versioni di dataset ricevuti. |
| R2b | 2. Versione/data | CAD, versione del documento sorgente | ricevuto | F2/F4 distinguono aggiornamento `2024-06` da etichetta file `2025-05-14`; F5 lega le tre estrazioni al PDF e alla pagina. |
| R2c | 2. Versione/data | CAD, decorrenze/versioni autonome delle tabelle | mancante | Nei tre JSON non sono dichiarate versioni temporali autonome né `effective_from`/`effective_to`; non assumere identità con la data del filename. |
| R3a | 3. Fonte | CAD, origine delle tre estrazioni | ricevuto | Ogni JSON F5 dichiara nome PDF e pagina; il PDF è presente e identificato con hash sotto. |
| R3b | 3. Fonte | SIOC, provenienza ufficiale dei singoli dataset | mancante | Le chiavi e gli usi sono descritti in F3, ma non i file autorevoli ricevuti, URL/documento di origine e release di ciascuno. |
| R4a | 4. Regole di validità | SIOC, requisiti di lookup | ricevuto | F3 specifica chiavi, membership, metadati sesso/età diagnosi, union strutture e sentinel distinti dai dataset. È una specifica derivata ricevuta, non prova dei metadati ufficiali. |
| R4b | 4. Regole di validità | SIOC, metadati ufficiali e validità temporale | mancante | Mancano dataset e regole autorevoli per periodi di validità, codici cessati, attributi e normalizzazione per ciascuna release (DET §108). |
| R4c | 4. Regole di validità | CAD, contenuto delle tabelle e normalizzazione documentata | ricevuto | F2/F5 e CAD-10/16: estrazioni delle tabelle; ICD con rimozione del punto e conservazione prefisso V. Le due voci `open_catalogue_fallbacks` nel JSON ICD non ricevono un catalogo completo. |
| R4d | 4. Regole di validità | CAD, metadati temporali delle tabelle | mancante | Nessuna policy ufficiale per validità dei codici nel tempo consegnata separatamente; non introdurre intervalli presunti. |
| R5a | 5. Eventuale storicizzazione | SIOC e CAD, storia/versioni delle tabelle | mancante | Nessuna serie storica di dataset o decisione autorevole che ne dichiari l'inapplicabilità. I profili di persistenza dei record/invii F4 non sono storicizzazione dei reference data. |

### Dipendenze SIOC senza dataset ufficiale

Per tutte le voci: **`mancante`**, stato da mantenere `PENDING_DATASET` secondo
F3. Il motivo comune è l'assenza del file ufficiale, della release/data e della
provenienza verificabile; chiavi e nomi provengono dal manifest e dalla specifica
ricevuti. I tre dataset CAD non soddisfano implicitamente queste dipendenze.

| Dataset richiesto | Uso dichiarato |
|---|---|
| `foreign-country-codes` | Cittadinanza / residenza estera |
| `health-authority-codes` | ULSS/ASL |
| `hsp11-structure-codes` | Struttura di provenienza |
| `icd9-cm-diagnosis-codes` | Diagnosi e metadati sesso/età |
| `icd9-cm-procedure-codes` | Procedure |
| `icpc-codes` | Patologie SVAMA |
| `istat-municipality-codes` | Comuni italiani |
| `italian-region-codes` | Regioni |
| `regional-entity-manager-codes-dgr-2961-2012` | Enti gestori regionali |
| `regional-udo-codes` | UDO regionali |
| `ria11-structure-codes` | Struttura di provenienza |
| `sts11-structure-codes` | Strutture e provenienza |

I tre file CAD ricevuti sono `cad_prestations_type_5_5_page5.json`,
`cad_needs_70_positions_page6.json`, `cad_icd9_cm_symptoms_page7.json` sotto F5.
Per onboarding completo raccogliere i metadati di DET §108; questa task non
definisce il formato runtime né chiude il versioning di DET §106.

## Elenco dei mancanti consegnato all'autore

Questa sezione è l'handoff documentale dei mancanti di T-P0-01, disponibile nel
repository per l'autore; non implica un invio esterno o un'approvazione.

1. **S1b:** XLSX SIOC ufficiale `SIOC_tracciato_regionale_20260420-definitivo_v05.xlsx`,
   con provenienza; eventuale baseline `2025-12-16` solo se necessaria al confronto
   storico, oggi disponibile come riferimento nel freeze e non come originale.
2. **S2b/S3b:** documenti ufficiali di accompagnamento, decorrenze/supersessione e
   conferma della completezza delle fonti per entrambe le versioni adottate;
   oppure dichiarazione motivata di non applicabilità degli accompagnamenti.
3. **S5b/R1a/R2a/R3b/R4b/R5a:** i dodici dataset SIOC sopra elencati, ciascuno con
   release/data, fonte, validità, normalizzazione e storia oppure decisione
   esplicita sulla non applicabilità della storia. Non ricostruirli dagli export.
4. **R2c/R4d/R5a:** date/versioni e policy temporali delle tabelle CAD, oppure
   chiarimento autorevole che non abbiano validità/storia autonoma dal documento.
5. **S4a:** esempi validi se reperibili e, per le task fixture, bundle sintetico
   completo SIOC/CAD con attesi tracciabili e golden; includere B2 CAD sintetico.
   Il solo export reale B2 e il dettaglio errori reale non sono stati forniti:
   non sono necessari per completare questo inventario e non si richiede di
   importare dati sanitari nel repository.

Le assenze bloccano la verifica normativa delle attività che richiedono tali
fonti, non l'inventario stesso. Le task successive devono verificare i propri
prerequisiti reali; nessuna dipendenza o gate viene qui derogata.

## Verifica del filesystem e integrità

- Elencati **201 file** del kit senza leggere il contenuto degli export:
  196 file censiti, manifest stesso e quattro export extra.
- `python3 VERIFY_KIT.py`: **FAIL, exit 1**, esclusivamente per
  `EXPORT_CAD/A.txt`, `B1.txt`, `C.txt`, `D.txt` extra. Non dichiarato PASS
  per il kit completo. Un controllo indipendente di dimensioni/SHA-256 sui
  **196 file censiti** ha trovato **0 errori**; nessun export è censito.
- Unico PDF rilevato: F2; nessun XLS/XLSX/CSV ufficiale rilevato. Il PDF è stato
  letto con `pdftotext -layout`: intestazione, cinque sezioni e sette pagine
  confermate; nessun contenuto di file pazienti letto.
- Le copie `docs/ARCHITECTURE.md`, `docs/DETAILED_DESIGN.md` e
  `docs/MASTER_IMPLEMENTATION_PLAN.md` coincidono byte per byte con F7.
  La vecchia nota «addendum assenti» è superata. L'affermazione storica di F7
  sull'assenza di export nel kit non descrive il filesystem attuale.
- Controlli di accettazione: 11/11 categorie DET §154 coperte, stati e motivi
  espliciti, dodici dipendenze SIOC e tre dataset CAD riconciliati coi manifest,
  elenco mancanti presente. Nessun contratto pubblico o documento sorgente
  modificato; nessuna chiusura G0.
- `cargo fmt`, `cargo clippy`, `cargo test`: **non eseguiti per T-P0-01**,
  modifica solo documentale; verificata completezza/tracciabilità come previsto
  dal prompt. Nessun test runtime o risultato CI attribuito all'inventario.

### Impronte delle fonti chiave (SHA-256)

| File nel kit | SHA-256 |
|---|---|
| `KIT_MANIFEST.json` | `a56536ec63bba07b533e8da9ea7fe4124faaae6af72ca380a3d655f1aa56ac71` |
| `SOURCE_DOCUMENTS/DETAILED_DESIGN.md` | `f626c3b0f3f321cca70476c41eefeb7c37b49db0bbfc8215d6ff7c81394987dc` |
| `SOURCE_DOCUMENTS/MASTER_IMPLEMENTATION_PLAN.md` | `6a2161709e929a8de66984eba1c12540d998df042d499376cd546f2bd9e88b40` |
| `SCHEMA_INPUTS/CAD_2024-06_SOURCE/2025-05-14_FLUSSO_ADI.pdf` | `8f828829d7912a8205073b0c097386774e996a64ab17f35b59cba7afbafd7b29` |
| `SCHEMA_INPUTS/SIOC_2026-04-20_FROZEN/manifest.json` | `9900ab3c0bf9f27abd4bcd6d8289dd0b6ee98a05833605f668a7fb9eb418a0fd` |
| `CAD_REVISED_CANDIDATE/manifest.json` | `3b220a34cd317b4bf657a9702d56f841f17fc7435986711c3744a3dcdb5d4561` |
