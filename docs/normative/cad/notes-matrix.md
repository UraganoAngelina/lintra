# CAD — Matrice delle note normative, T-P0-04

Verifica diretta aggiornata al 2026-10-19; bozza originaria del 2026-10-16. Classificazione documentale di **83/83 campi**.
Prerequisito T-P0-02: 218 proiezioni verificate e review approvata nei commit
`419600d`/`6a4e9ac`; accettazione a campione dell’autore ricevuta il
2026-10-16: «Campione verificato: approvo». Le etichette storiche
DONE_UNVERIFIED nelle tabelle sono conservate e non annullano tale evidenza.

Questa matrice applica ARCH §5.7/§36 e DET §107.6/§144: il motore non esegue
le note. `regola eseguibile candidata` identifica clausole da formalizzare o
candidati **già ricevuti**; non approva norme, DSL, compiler o runtime.
`metadata` conserva descrizioni/istruzioni di raccolta e semantiche escluse
esplicitamente dalle decisioni; non significa che il dato sia facoltativo.
`ambigua` segnala una clausola con interpretazione/disallineamento aperto,
con registro DEC-05 sotto; le altre clausole restano visibili nella motivazione.
La classe è unica per nota (ambigua prevale sulla parte candidata); nessuna
inferenza di copertura completa viene fatta dalla sola presenza di constraints.
Le coperture parziali e i vincoli ricevuti più forti del testo sono dichiarati.

| Classe primaria | Note |
|---|---:|
| metadata | 12 |
| regola eseguibile candidata | 70 |
| ambigua | 1 |

Tutte le note ricevute sono stringhe non vuote: **none/nota assente: 0**.
Una futura nota nulla/vuota va registrata esplicitamente come metadata/none,
non omessa dalla copertura.

## Fonti e recupero del testo integrale

`K/` = starter kit adiacente `LINTRA_STARTER_KIT_FINAL_20261016/`.
Le tabelle T-P0-02 collegate in ciascun tracciato conservano integralmente
`fields[n].regulatory_metadata.source_note`, con whitespace e refusi originali.
Ogni riga identifica **tracciato + ID esatto + indice n + SHA-256 UTF-8 della
nota** ed estratto iniziale JSON (solo anteprima, ellissi esterna alla stringa).
Il testo completo è quindi recuperabile senza interpretare l’estratto;
`provenance` della tabella identifica trace ricevuto e relativo SHA-256.
Gli ID possono ripetersi tra flussi: usare sempre il flusso e il tracciato.

`C:` nell’inventario candidati indica i `constraints[].kind` ricevuti;
`P:` indica soltanto presenza di required_if/forbidden_if; `R:` elenca regole
ricevute con prefisso del campo. Sono **localizzatori**, non analisi di copertura:
il JSON sorgente sotto `/fields/n`, `/rules`, `/joins`, `/record_key`,
`/documented_non_executable_rules` e `/repeated_groups` resta consultabile.
La motivazione distingue le clausole della nota dai dettagli aggiuntivi ricevuti.

Fonti: [audit puntuale83 note](../../../../LINTRA_STARTER_KIT_FINAL_20261016/CAD_NOTES_AUDIT.md),
[decision register](../../../../LINTRA_STARTER_KIT_FINAL_20261016/CAD_REVISED_CANDIDATE/DECISION_REGISTER.md),
[flow candidato](../../../../LINTRA_STARTER_KIT_FINAL_20261016/CAD_REVISED_CANDIDATE/flow_schema.json),
[profilo runtime e decisioni G03–G06](../../../../LINTRA_STARTER_KIT_FINAL_20261016/CAD_REVISED_CANDIDATE/CAD_RUNTIME_PROFILE.json),
[profilo storico](../../../../LINTRA_STARTER_KIT_FINAL_20261016/CAD_REVISED_CANDIDATE/CAD_HISTORICAL_PROFILE.md)
e [freeze gate report](../../../../LINTRA_STARTER_KIT_FINAL_20261016/CAD_REVISED_CANDIDATE/FREEZE_GATE_REPORT.md).
Fonte PDF ricevuta `K/SCHEMA_INPUTS/CAD_2024-06_SOURCE/2025-05-14_FLUSSO_ADI.pdf`;
la pagina di ogni nota è riportata nella tabella. Revisione derivata
`2026-10-16.reviewed-g02-g06`, normativa2024-06, etichetta file2025-05-14:
nessuna nuova decorrenza viene dedotta. CAD resta **RELEASE_CANDIDATE**.

Le categorie dell’audit sono evidenza ricevuta, non le tre classi di questa
matrice. In particolare, DOCUMENTED non elimina clausole normative esplicite,
IN_SCHEMA non dimostra runtime e PENDING non significa automaticamente ambigua.
Il punto5 storico del decision register su pos60 è superato da **G03**:
ASCII0x20 fuori UCPDOM e1/2 in UCPDOM, pos61–70 sempre2. **G04** e il profilo
corrente precisano calendario/pause/fine mese; nessuna riapertura della vecchia
lacuna di calendario. PARTIAL_PDF_ONLY_LOOKUP dell’audit non significa che
manchino oggi i dataset: ICD9 limitato a PDFp7 e prestazioni p5 già ricevuti,
G05 ne fissa il perimetro. Vedi [registro dataset](../reference-datasets.md).
Ulteriori note esterne alle celle restano rintracciate nell’audit (pp1,4–7):
TEAM(1*), obblighi anche nelle rivalutazioni, uso eccezionale non disponibile,
valutazione bisogni, mappa pos60 e codiciICD9 senza punto/V. Questa matrice
copre i source_note di campo, senza perdere il rinvio a tali istruzioni.
Nessuna lettura di export/dati sanitari né promozione a FROZEN o chiusura G0.

## Tracciato A — 24 note

Testo integrale: [A-fields.json](A-fields.json); fonte `K/CAD_REVISED_CANDIDATE/trace_a.json`.

| ID / indice nota | Classe | SHA-256 nota / estratto | Clausole, motivazione e limiti | Candidati ricevuti (inventario) |
|---|---|---|---|---|
| `1.1` / `fields[0]`; PDF p.1 | regola eseguibile candidata | `8cffa78be4be2c80b18dd55eb0665fb6cf6b523ba5a0c714d4bd7b4b4b56d7d9`<br>"Codice dell'azienda ULSS erogatrice es. 501 ,….,509" | Dominio ULSS erogatrice501–509: allowed_values ricevuto e CAD-E/G06. Esempio in nota non abilita codici arbitrari; contesto sending_ulss deriva dal profilo ricevuto. | C: allowed_values; P: nessuna condizione |
| `1.2` / `fields[1]`; PDF p.1 | metadata | `2baf26dc7047a43858fd3f7c80c5a4fed1708b8af1e30ea030979ae6f6e7413d`<br>"Il campo va valorizzato con il Codice Fiscale per tutte le schede rife" … | Istruzione scelta CF/TEAM/ENI/STP conservata come metadata per CAD-03: nessuna coerenza semantica con1.3. Audit REVIEWED_NO_CROSS_FIELD_BY_DECISION; vincoli fisici/presenza restano separati. | C: nessuno; P: nessuna condizione |
| `1.3` / `fields[2]`; PDF p.1 | regola eseguibile candidata | `804bfa40d795699cfd1ffc83199291f30969beaaacc6b817e77394750ebe741b`<br>"1. Codice fiscale\n2. Codice ENI\n3. Codice STP\n4. Codice ID utente dell" … | Dominio tipo identificativo1–6/9: allowed_values ricevuto, senza relazione semantica con1.2 (CAD-03). | C: allowed_values; P: nessuna condizione |
| `1.4` / `fields[3]`; PDF p.1 | regola eseguibile candidata | `e3935103c380caba6f03bd45150d0f6648c3924e8e0f1b054890d59dd4ba8a01`<br>"Indica il codice di identificazione assegnato a livello nazionale alla" … | Descrizione istituzione e regolamento come metadata; obbligo per tipo4 in required_if ricevuto. Nota(1*) esterna alla cella conservata nell’audit, nessuna regola aggiuntiva inferita. | C: nessuno; P: required_if |
| `1.5` / `fields[4]`; PDF p.1 | regola eseguibile candidata | `1abf33e81f28f1aff772f21bac09aea9802cd78893a264fc3874ce8d130cec2e`<br>"Indicare la data di scadenza della tessera TEAM per i cittadini comuni" … | Clausole formato GGMMAAAA e obbligo tipo4: date_if_present/required_if ricevuti; descrizione TEAM e refuso tesssera conservati nel testo integrale. | C: date_if_present; P: required_if |
| `1.6` / `fields[5]`; PDF p.1 | metadata | `5cf28de56e29a2e5fed480659a473f8806b8ad0b4c503899a29366346db466da`<br>"Master Patient Index (Id regionale della posizione anagrafica).\nChiave" … | Descrizione MPI, provenienza Anagrafe Unica e facoltatività fino a diversa indicazione. Audit DOCUMENTED_METADATA_OR_PHYSICAL_ONLY; presence opzionale ricevuta, nessun controllo univocità MPI. | C: digits_only_if_present; P: nessuna condizione |
| `1.7` / `fields[6]`; PDF p.1 | metadata | `8e4ea7f22baeae2eb5b8501ecef5719781da9d22f31295804190b854334c8cce`<br>"Il campo va valorizzato, in combinazione con il codice fiscale, per i " … | Clausola combinazione CF/residenti Veneto conservata senza esecuzione per decisione già ricevuta: audit REVIEWED_DOCUMENTED_ONLY_NO_SEMANTIC_CHECK. digits_only_if_present non dimostra verifica residenza. | C: digits_only_if_present; P: nessuna condizione |
| `1.8` / `fields[7]`; PDF p.1 | regola eseguibile candidata | `16fca8d5f381b336f18da79f00f9df1cfce351f99199ddee0dfaa3cf7f49469b`<br>"Il campo va valorizzato, in combinazione con il codice fiscale, obblig" … | Clausola nominativo obbligatorio quando CF indisponibile: required_if1.3!=1 ricevuto, esplicitamente CAD-04. Testo CF e proxy tipo identificativo restano distinti; nessuna equivalenza nuova. | C: nessuno; P: required_if |
| `1.9` / `fields[8]`; PDF p.1 | regola eseguibile candidata | `16fca8d5f381b336f18da79f00f9df1cfce351f99199ddee0dfaa3cf7f49469b`<br>"Il campo va valorizzato, in combinazione con il codice fiscale, obblig" … | Clausola nominativo obbligatorio quando CF indisponibile: required_if1.3!=1 ricevuto, esplicitamente CAD-04. Testo CF e proxy tipo identificativo restano distinti; nessuna equivalenza nuova. | C: nessuno; P: required_if |
| `1.10` / `fields[9]`; PDF p.1 | regola eseguibile candidata | `21cc7820d084d135a491176c80e6c0b0dc334e07c60eedb8c5203591b2d562d7`<br>"Utilizzare i codici seguenti: 1 - maschio, 2 - femmina." | Dominio sesso1/2 in allowed_values ricevuto. | C: allowed_values, digits_only_if_present; P: nessuna condizione |
| `1.11` / `fields[10]`; PDF p.1 | regola eseguibile candidata | `f7f2607159fe76d57f6f8e29e0b35fac3ff8b9e7a66311bb5a58c96a2d37991c`<br>"Utilizzare il formato GGMMAAAA." | Formato GGMMAAAA in constraint date ricevuto. | C: date; P: nessuna condizione |
| `1.12` / `fields[11]`; PDF p.1 | regola eseguibile candidata | `f486e87370e8f8729dd4cfa2e90dfa68007e9af75336eec0b5ab480bc9107088`<br>"Codici ISTAT (primi 3 caratteri per la provincia e i rimanenti 3 carat" … | Clausole codifica ISTAT3+3, zero iniziale e estero999+Stato: candidate sintattiche non formalizzate nel campo; lookup esterno espressamente escluso CAD-E/CAD-10/16 e audit REVIEWED_NO_EXTERNAL_LOOKUP. Copertura parziale, non semplice verifica lunghezza. | C: nessuno; P: nessuna condizione |
| `1.13` / `fields[12]`; PDF p.1 | regola eseguibile candidata | `f69007d1a4f831c5b15f4b33a7c6a3013d32191b772c1a850c51a183b671a7e2`<br>"Identifica la cittadinanza dell’assistito alla fine del periodo di rif" … | Clausole cittadinanza al termine periodo, italiano100/apolide999. Nessun constraint locale di codice speciale; lookup escluso da decisione. Istruzioni di codifica candidate non eseguite; audit REVIEWED_NO_EXTERNAL_LOOKUP non equivale ad attuarle. | C: nessuno; P: nessuna condizione |
| `1.14` / `fields[13]`; PDF p.1 | regola eseguibile candidata | `f486e87370e8f8729dd4cfa2e90dfa68007e9af75336eec0b5ab480bc9107088`<br>"Codici ISTAT (primi 3 caratteri per la provincia e i rimanenti 3 carat" … | Clausole codifica ISTAT3+3, zero iniziale e estero999+Stato: candidate sintattiche non formalizzate nel campo; lookup esterno espressamente escluso CAD-E/CAD-10/16 e audit REVIEWED_NO_EXTERNAL_LOOKUP. Copertura parziale, non semplice verifica lunghezza. | C: nessuno; P: nessuna condizione |
| `1.15` / `fields[14]`; PDF p.1 | regola eseguibile candidata | `2da7aa68109e0f4a72d9c7090613b5cab0cc7dee6554eef94bfe12adccf40af0`<br>"Utilizzare il formato RRRUUU; RRR = codici ISTAT delle regioni UUU = c" … | Clausola formato RRRUUU con codici Regione/azienda: candidato composito non formalizzato (constraints vuoti); lookup esclusi da CAD-E. Non confondere assenza lookup con copertura della composizione. | C: nessuno; P: nessuna condizione |
| `1.16` / `fields[15]`; PDF p.1 | regola eseguibile candidata | `3d6bd0da982f4ebdf0d79eaea026b15f7e9abb8005e20c7da9dcb2296504cc98`<br>"Codici ISTAT (primi 3 caratteri per la provincia e i rimanenti 3 carat" … | Clausola ISTAT3+3 e zero iniziale: candidato di codifica non formalizzato; lookup escluso per decisione, audit REVIEWED_NO_EXTERNAL_LOOKUP; non si inventa repertorio. | C: nessuno; P: nessuna condizione |
| `1.17` / `fields[16]`; PDF p.1 | metadata | `995931a77ae203651551a563affb68e49802e18de04d81870a4ae4acf486d198`<br>"Codice del distretto relativo al comune di domicilio" | Descrive distretto relativo al comune domicilio; audit DOCUMENTED_METADATA_OR_PHYSICAL_ONLY. digits_only_if_present non verifica relazione geografica, nessun join nuovo. | C: digits_only_if_present; P: nessuna condizione |
| `1.18` / `fields[17]`; PDF p.1 | metadata | `8492010785fe3c085a628c33215dd45d47eec31a4ace7012fe9f2d35b6a8f659`<br>"Codice fiscale del MMG o PLS." | Descrive CF MMG/PLS; audit DOCUMENTED_METADATA_OR_PHYSICAL_ONLY. Nessun checksum/lookup di anagrafe medici autorizzato. | C: nessuno; P: nessuna condizione |
| `1.19` / `fields[18]`; PDF p.1 | regola eseguibile candidata | `276757d63633edce31a955dd515cd102984492995e2fbc56b98dd1b8355dc34d`<br>"Indica lo stato civile dell’assistito al momento della rilevazione: 1 " … | Dominio stato civile1–5/9: allowed_values ricevuto; momento rilevazione è significato del dato. | C: allowed_values, digits_only_if_present; P: nessuna condizione |
| `1.20` / `fields[19]`; PDF p.1 | metadata | `65e53832da1b52e5a716e427a38ac79b44e2ffbed2c22ce9b0038a46ef59935c`<br>"Indica il numero dei componenti del nucleo familiare convivente, esclu" … | Descrive componenti conviventi da contare e persone escluse: istruzione di raccolta non verificabile dal solo totale. Audit DOCUMENTED_METADATA_OR_PHYSICAL_ONLY; digits_only non certifica conteggio delle persone. | C: digits_only_if_present; P: nessuna condizione |
| `1.21` / `fields[20]`; PDF p.1 | regola eseguibile candidata | `e61b174e9754a2b3b79171e07ff7a231d04f03e72a4f88cf70e0907e5400e557`<br>"Asssitenza a pagamento fornita da una o più persone non appartenenti a" … | Dominio assistente non familiare1–7: allowed_values ricevuto; ore/giorno sono descrizioni delle categorie, non durate osservabili da calcolare. | C: allowed_values, digits_only_if_present; P: nessuna condizione |
| `1.22` / `fields[21]`; PDF p.1 | regola eseguibile candidata | `9985342e790b69123e39d105716ee9afd2b93d5b45835c8a50e6e9fb0b4a2295`<br>"1 – nessuno, 2 – licenza elementare, 3 – licenza media inferiore, 4 – " … | Dominio titolo di studio1–8: allowed_values ricevuto. | C: allowed_values, digits_only_if_present; P: nessuna condizione |
| `1.23` / `fields[22]`; PDF p.1 | regola eseguibile candidata | `323cbc3e1b0847661cd5eec67ba1fb48d522a9e69c110b352acb920be49a0555`<br>"Numero che identifica univocamente l’ evento assistenziale di CD (pazi" … | Clausole univocità evento, join file e stabilità codice nel tempo: record_key A(1.1,1.23), join figli e profilo storico ricevuti. G06 definisce ambito ULSS e input missing non valutabile; stabilità storica non resa eseguibile dalla sola PK, audit PENDING_FLOW_CARDINALITY_JOIN. | C: digits_only_if_present; P: nessuna condizione |
| `1.24` / `fields[23]`; PDF p.1 | regola eseguibile candidata | `4df86c6cff6be96c40a47a0cd17f52c00d708aca4d8c505989ca0f05b48f924a`<br>"Indica chi ha la responsabilità nei confronti dell’assistito se minore" … | Clausole minore<=17 secondo differenza anni e dominio1–5/9. allowed_values ricevuto copre solo dominio; CAD-05 e calendar_year_delta/field_presence_logic nel profilo richiedono contesto B1, non età compiuta; audit PENDING_DSL_EXTENSION_AGE non nuova ambiguità. | C: allowed_values, digits_only_if_present; P: nessuna condizione |

## Tracciato B1 — 36 note

Testo integrale: [B1-fields.json](B1-fields.json); fonte `K/CAD_REVISED_CANDIDATE/trace_b1.json`.

| ID / indice nota | Classe | SHA-256 nota / estratto | Clausole, motivazione e limiti | Candidati ricevuti (inventario) |
|---|---|---|---|---|
| `2.1` / `fields[0]`; PDF p.2 | regola eseguibile candidata | `c0eb801f586e58884bbbf79095fdc55e79489e59719c12b5e7b27fb00e488fbc`<br>"Ricollega il servizio erogato all’evento assistenziale di CD (vedi TRA" … | Clausola ricollegamento evento ad A: join *_to_a_event ricevuto con contesto same_ulss_submission; G06 esplicita sending_ulss ed A assente→non valutabile. Audit DOCUMENTED_METADATA_OR_PHYSICAL_ONLY non prova esecuzione del join; qui classificata candidata per la relazione esplicita. | C: digits_only_if_present; P: nessuna condizione |
| `2.2` / `fields[1]`; PDF p.2 | regola eseguibile candidata | `438533c61b7af1cb72487e9503c13327c5db224f99475a578702e44d91d28a7f`<br>"Indica il soggetto richiedente la evento assistenziale di CD INIZIALE." … | Dominio richiedente1–14 in allowed_values ricevuto; riferimento evento iniziale descrittivo, non nuovo obbligo di cardinalità dedotto. | C: allowed_values, digits_only_if_present; P: nessuna condizione |
| `2.3` / `fields[2]`; PDF p.2 | regola eseguibile candidata | `dcead3d4941fc6eeb2b40f675c66e9fa3ccff75492c87ecee16858ed61022b2f`<br>"Indica la tipologia di presa in carico. I valori ammessi sono i seguen" … | Dominio presa in carico1/2 in allowed_values ricevuto. | C: allowed_values, digits_only_if_present; P: nessuna condizione |
| `2.4` / `fields[3]`; PDF p.2 | regola eseguibile candidata | `281d55a82536252e1a22a60f46dfc0d3061c59a9760de8fd834926a77900ac6d`<br>"Indica la data dell’attuale evento assistenziale di CD (non varia fino" … | Clausola data invariata fino chiusura: date ricevuto copre formato, non immutabilità. Audit PENDING_TEMPORAL_EVENT_DATE_IMMUTABLE; storico CAD-12/15 documenta contesto, non prova esecuzione né autorizza nuova regola. | C: date; P: nessuna condizione |
| `2.5` / `fields[4]`; PDF p.2 | regola eseguibile candidata | `f248c5bcdfa78bfbdfb50084680e364c52ca281a7c1a8339d46281d920512a9d`<br>"Indica se esiste una pianificazione condivisa delle cure in cartella/f" … | Dominio1/2/9 e compilazione solo UCPDOM: allowed_values/required_if/forbidden_if ricevuti distinti. | C: allowed_values, digits_only_if_present; P: required_if/forbidden_if |
| `2.6` / `fields[5]`; PDF p.2 | metadata | `64032a028c9062d67f3f40d30dc8d73072267bc15c34436786f09e2142df8b6c`<br>"Indica la data di conclusione della evento assistenziale di CD dell’as" … | Descrizione conclusione evento. date_if_present e facoltatività ricevuti da CAD-06; controlli temporali CAD-14/17 nel profilo, non contenuti nella sola nota né implementati dal formato data. | C: date_if_present; P: nessuna condizione |
| `2.7` / `fields[6]`; PDF p.2 | regola eseguibile candidata | `38f0f02cf2157a95c30413e5ff85485d76075dd5c53642431a241c9e02a8e4d7`<br>"Il campo ha dominio condizionato dal tipo presa in carico.\nValori amme" … | Clausole dominio, obbligo con2.6, esclusioni10(UCPDOM)/13(CD), codice12 dopo3mesi. allowed_values/presence/CAD-R-2.7-* ricevuti coprono parte locale; CAD-17/G04/profilo definiscono3mesi calendario e pause ma runtime mancante. Vecchie lacune fine mese nel decision register sono superate dal profilo/G04, non riaperte. | C: allowed_values, digits_only_if_present; P: required_if; R: CAD-R-2.7-UC-10, CAD-R-2.7-CD-13 |
| `2.8` / `fields[7]`; PDF p.2 | ambigua | `4471e1046b23b44700e7e67eeab26833ca05ba9135313ed4652a085bfbc45829`<br>"valori ammessi: 1 - valutazione iniziale (per ogni nuovo evento assist" … | Dominio1/2 ricevuto; inciso valutazione iniziale per ogni nuovo evento richiede cardinalità. Audit PENDING_FLOW_CARDINALITY_AT_LEAST_ONE_INITIAL_EVAL lascia conferma eccezioni aperta: non basta allowed_values. | C: allowed_values, digits_only_if_present; P: nessuna condizione |
| `2.9` / `fields[8]`; PDF p.2 | regola eseguibile candidata | `ff3becdb6a49790ee8e000ce2e57a27776883709998a7f6eb343408cb61f730c`<br>"Indica la data nella quale è stata effettuata la valutazione iniziale " … | Clausola formato GGMMAAAA in date ricevuto; data ultimo aggiornamento e descrizione valutazione come metadata senza confronto arbitrario tra record. | C: date; P: nessuna condizione |
| `2.10` / `fields[9]`; PDF p.2 | regola eseguibile candidata | `adf7610de35770e3e2775791ab6b8bdc1439030eccdccb81c69a93e5d8756e2e`<br>"indicare la data dell'ultima UVMD (anche se fatta nei periodi preceden" … | Formato canonico DDMMYYYY già ricevuto nel constraint date_if_present, con lunghezza D(8); audit SOURCE_TYPO_GGMMAAA_8_BYTELENGTH documenta il refuso GGMMAAA. Testo originale preservato; nessun nuovo formato o correzione della fonte. | C: date_if_present; P: nessuna condizione |
| `2.11` / `fields[10]`; PDF p.2 | metadata | `29e462e4b559db4543a1ae33bfcdcbca98ba412af152bdcaab85ab33cf1a3f7a`<br>"Codice ICPC della diagnosi \"principale\" (quella che condiziona maggior" … | Descrizione diagnosi principale ICPC, senza algoritmo clinico. Lookup escluso CAD-10/16 e audit REVIEWED_NO_ICPC_LOOKUP; nessuna regola semantica aggiunta. | C: nessuno; P: nessuna condizione |
| `2.12` / `fields[11]`; PDF p.2 | regola eseguibile candidata | `bcf99c14db656c219705f7f10559b8826f0357bd5cddf943f01cffd01113d4ec`<br>"Codice ICPC della patologia concomitante in grado di condizionare la e" … | DEC-05-CAD-ICPC-000 chiusa dall’autore il 2026-10-19: campo obbligatorio, 000 valido per patologia assente, tre spazi mancanti. Il candidato ricevuto resta documentato con missing exact000; correzione della configurazione rinviata alla task pertinente. Nessun lookup ICPC aggiunto. | C: nessuno; P: nessuna condizione |
| `2.13` / `fields[12]`; PDF p.2 | regola eseguibile candidata | `843f84c047fa72dfbc08b290a1872bc053919a24ee252f66fe8a9bdbb33fbb1c`<br>"da Svama completa o scheda di valutazione multidimensionale semplifica" … | Dominio cognitivo1/2/3: allowed_values ricevuto; derivazione SVaMA/scheda clinica metadata. | C: allowed_values, digits_only_if_present; P: nessuna condizione |
| `2.14` / `fields[13]`; PDF p.2 | regola eseguibile candidata | `7d3d8f3cd3b9175be41b6ee9bbabfbd7fd2614e07dd52b67d9dd0d98d10179ed`<br>"da NPI o scheda di valutazione multidimensionale semplificata: 1 - ass" … | Dominio disturbi1/2/3: allowed_values ricevuto; derivazione NPI/scheda clinica metadata. | C: allowed_values, digits_only_if_present; P: nessuna condizione |
| `2.15` / `fields[14]`; PDF p.2 | regola eseguibile candidata | `c658e390c41dca59d778412498dad1c2c703e6e082f5f9e7bc99ee398a8c10cc`<br>"per flusso riabilitazione territoriale: 1 - assenti / lievi, 2 - moder" … | Dominio disturbi1/2/3 da riabilitazione territoriale: allowed_values ricevuto; provenienza valutazione metadata. | C: allowed_values, digits_only_if_present; P: nessuna condizione |
| `2.16` / `fields[15]`; PDF p.2 | regola eseguibile candidata | `c658e390c41dca59d778412498dad1c2c703e6e082f5f9e7bc99ee398a8c10cc`<br>"per flusso riabilitazione territoriale: 1 - assenti / lievi, 2 - moder" … | Dominio disturbi1/2/3 da riabilitazione territoriale: allowed_values ricevuto; provenienza valutazione metadata. | C: allowed_values, digits_only_if_present; P: nessuna condizione |
| `2.17` / `fields[16]`; PDF p.2 | regola eseguibile candidata | `253b3ef4339f52a628ee6629b11d04e28c2cb658032fcb5c85a96cb532e6b937`<br>"Indica il livello di autonomia dell’assistito nell’area di mobilità co" … | Clausole dominio1/2/3 e obbligo da3anni (differenza anni presa in carico/nascita). Campo copre solo dominio; CAD-05/07 e profilo calendar_year_delta/field_presence_logic ricevuti coprono intenzione non esecuzione. Audit PENDING_DSL_EXTENSION_AGE. | C: allowed_values, digits_only_if_present; P: nessuna condizione |
| `2.18` / `fields[17]`; PDF p.2 | regola eseguibile candidata | `e946056f6ef93268a8bd9ffa6f2278afa02544b564e471f67b12abe2af6060b9`<br>"Indica il livello di autonomia dell’assistito nell’area di mobilità co" … | Clausole dominio1/2/3 e obbligo da6anni (differenza anni). Campo copre solo dominio; CAD-05/07 e profilo età ricevuti, non required_if compilato. Audit PENDING_DSL_EXTENSION_AGE. | C: allowed_values, digits_only_if_present; P: nessuna condizione |
| `2.19` / `fields[18]`; PDF p.2 | regola eseguibile candidata | `b71cb1fa2d309cc3fcf85972c1c5b5a84a3436e68ed0b92acd3cba5e3b9237b3`<br>"da Svama completa o scheda di valutazione multidimensionale semplifica" … | Dominio1/2/3 in allowed_values ricevuto; origine SVaMA e descrizione scala metadata. | C: allowed_values, digits_only_if_present; P: nessuna condizione |
| `2.20` / `fields[19]`; PDF p.2 | regola eseguibile candidata | `a4313a19782000b2f48a6852ae848c9f5127de1ea38bd1a3bd2bed740f8fa278`<br>"da Svama completa o scheda di valutazione multidimensionale semplifica" … | Dominio1/2/3 in allowed_values ricevuto; origine SVaMA e descrizione scala metadata. | C: allowed_values, digits_only_if_present; P: nessuna condizione |
| `2.21` / `fields[20]`; PDF p.2 | regola eseguibile candidata | `786d90bb14bfa9cca02e7daf45065f2542f449d2caa8b6046739114bebaa6173`<br>"Indica la presenza di Fragilità familiare. Valore ammessi: 1 - present" … | Dominio fragilità1/2/9 in allowed_values ricevuto; significato clinico metadata. | C: allowed_values, digits_only_if_present; P: nessuna condizione |
| `2.22` / `fields[21]`; PDF p.2 | regola eseguibile candidata | `2c8e3f77b208e8b9b05978af31da4d32f49e22660df51a3c2856243c1c67b7c3`<br>"problemi di salute allineato con DM 17-12-2008; Codificare in questo m" … | Clausole bit1/2, mappa bisogni allegatoD e pos61–70=2. positional_bitset ricevuto e G03 già approvata: pos60 ASCII0x20 fuori UCPDOM,1/2 in UCPDOM. Copertura richiede conditional_positional_bitset; nessuna ambiguità nuova per il vecchio punto5 del decision register. | C: positional_bitset; P: nessuna condizione |
| `2.23` / `fields[22]`; PDF p.2 | regola eseguibile candidata | `28b46a20e17ffcc21ddfd9d47ea80d0ac9cb82b91ccca460a05f2c8f84da38f1`<br>"Indica se l’assistito è a rischio sanguinamento acuto. Valori ammessi:" … | Dominio rischio sanguinamento1/2/9 in allowed_values ricevuto; nessuna diagnosi del rischio dedotta. | C: allowed_values, digits_only_if_present; P: nessuna condizione |
| `2.24` / `fields[23]`; PDF p.2 | regola eseguibile candidata | `492aec7eb3ad8af3c29f82fd9d048ce138254566b913ce7699b30abf2ef29806`<br>"Codificare in questo modo:\"1\" in caso di servizio presente nel periodo" … | Clausole vettore posizionale1/2 e mappa servizi; positional_bitset ricevuto rappresenta i bit, CAD-09 tratta padding fisico. Audit POSITIONAL_BITS_NEED_RUNTIME: esecuzione e corrispondenza clinica non dimostrate; non solo metadata. | C: positional_bitset; P: nessuna condizione |
| `2.25` / `fields[24]`; PDF p.2 | regola eseguibile candidata | `90c45daece256fbb7135878f17b437e253662d0eac4ba8a80a9f278624c7662e`<br>"Codificare in questo modo:\"1\" in caso di servizio presente nel periodo" … | Clausole vettore posizionale1/2 e mappa servizi; positional_bitset ricevuto rappresenta i bit, CAD-09 tratta padding fisico. Audit POSITIONAL_BITS_NEED_RUNTIME: esecuzione e corrispondenza clinica non dimostrate; non solo metadata. | C: positional_bitset; P: nessuna condizione |
| `2.26` / `fields[25]`; PDF p.2 | regola eseguibile candidata | `d49ecc6586e5175983309a7352e988361bddb9277e6b8f4d79bf5ab6accfca3c`<br>"Codificare in questo modo:\"1\" in caso di servizio presente nel periodo" … | Clausole vettore posizionale1/2 e mappa servizi; positional_bitset ricevuto rappresenta i bit, CAD-09 tratta padding fisico. Audit POSITIONAL_BITS_NEED_RUNTIME: esecuzione e corrispondenza clinica non dimostrate; non solo metadata. | C: positional_bitset; P: nessuna condizione |
| `2.27` / `fields[26]`; PDF p.2 | regola eseguibile candidata | `111754ee41c899b6347864241e88e9f667678377163d8199ab5853a1de8eb089`<br>"Indica se l’erogatore appartiene alla rete.\nValori ammessi:1 - Si, 2 -" … | Dominio1/2/9 e obbligo solo UCPDOM in allowed_values/presence ricevuti. | C: allowed_values, digits_only_if_present; P: required_if/forbidden_if |
| `2.28` / `fields[27]`; PDF p.2 | regola eseguibile candidata | `806081d7ffddfce58998f29bc8be46a8837e181c59c299284628c74cb9693988`<br>"Indica a quale rete appartiene l’erogatore. Valori ammessi:1 - RLCP (r" … | Dominio1/2/3/9 e condizione UCPDOM+appartenenza rete: allowed_values/presence ricevuti e CAD-11; testo descrittivo rete preservato. | C: allowed_values, digits_only_if_present; P: required_if/forbidden_if |
| `2.29` / `fields[28]`; PDF p.3 | regola eseguibile candidata | `fa4a994d5c3fd7e8989c8ac2806bac4410cb86125de9feda5cf926e80cf14abc`<br>"I valori ammessi sono riportati nella tabella di riferimento 5.1 (ICD " … | Clausole lookup ICD9CM e obbligo solo UCPDOM; reference_data_if_present/presence ricevuti. Audit PARTIAL_PDF_ONLY_LOOKUP è storico: dataset ricevuto limitato al PDFp7 secondo G05, non più un file da inventare; non copre codici nazionali arbitrari. | C: reference_data_if_present; P: required_if/forbidden_if |
| `2.30` / `fields[29]`; PDF p.4 | regola eseguibile candidata | `04927ce04c2a1c5e3a285096968352398c3a50c55a455b353230878422f6a704`<br>"I valori ammessi sono riportati nella tabella di riferimento (ICD 9 CM" … | Clausole lookup ICD9CM e facoltatività solo UCPDOM; reference_data_if_present/forbidden_if ricevuti e CAD-11/G05. PARTIAL_PDF_ONLY_LOOKUP storico superato quanto alla disponibilità: resta limite elenco PDFp7, nessuna estensione nazionale. | C: reference_data_if_present; P: forbidden_if |
| `2.31` / `fields[30]`; PDF p.4 | regola eseguibile candidata | `04927ce04c2a1c5e3a285096968352398c3a50c55a455b353230878422f6a704`<br>"I valori ammessi sono riportati nella tabella di riferimento (ICD 9 CM" … | Clausole lookup ICD9CM e facoltatività solo UCPDOM; reference_data_if_present/forbidden_if ricevuti e CAD-11/G05. PARTIAL_PDF_ONLY_LOOKUP storico superato quanto alla disponibilità: resta limite elenco PDFp7, nessuna estensione nazionale. | C: reference_data_if_present; P: forbidden_if |
| `2.32` / `fields[31]`; PDF p.4 | regola eseguibile candidata | `04927ce04c2a1c5e3a285096968352398c3a50c55a455b353230878422f6a704`<br>"I valori ammessi sono riportati nella tabella di riferimento (ICD 9 CM" … | Clausole lookup ICD9CM e facoltatività solo UCPDOM; reference_data_if_present/forbidden_if ricevuti e CAD-11/G05. PARTIAL_PDF_ONLY_LOOKUP storico superato quanto alla disponibilità: resta limite elenco PDFp7, nessuna estensione nazionale. | C: reference_data_if_present; P: forbidden_if |
| `2.33` / `fields[32]`; PDF p.4 | regola eseguibile candidata | `04927ce04c2a1c5e3a285096968352398c3a50c55a455b353230878422f6a704`<br>"I valori ammessi sono riportati nella tabella di riferimento (ICD 9 CM" … | Clausole lookup ICD9CM e facoltatività solo UCPDOM; reference_data_if_present/forbidden_if ricevuti e CAD-11/G05. PARTIAL_PDF_ONLY_LOOKUP storico superato quanto alla disponibilità: resta limite elenco PDFp7, nessuna estensione nazionale. | C: reference_data_if_present; P: forbidden_if |
| `2.34` / `fields[33]`; PDF p.4 | regola eseguibile candidata | `b2763826ed3d5e367f038ea14b569a93dc97492fe6631c8b4a21dbb650fe3cc4`<br>"Valori ammessi: 1 - Si, 2 - No, 9- non disponibile*\nNBB: Il campo deve" … | Dominio1/2/9 e obbligo solo UCPDOM: allowed_values/presence ricevuti; non introdurre controlli sul contenuto clinico dello strumento. | C: allowed_values, digits_only_if_present; P: required_if/forbidden_if |
| `2.35` / `fields[34]`; PDF p.4 | regola eseguibile candidata | `b2763826ed3d5e367f038ea14b569a93dc97492fe6631c8b4a21dbb650fe3cc4`<br>"Valori ammessi: 1 - Si, 2 - No, 9- non disponibile*\nNBB: Il campo deve" … | Dominio1/2/9 e obbligo solo UCPDOM: allowed_values/presence ricevuti; non introdurre controlli sul contenuto clinico dello strumento. | C: allowed_values, digits_only_if_present; P: required_if/forbidden_if |
| `2.36` / `fields[35]`; PDF p.4 | regola eseguibile candidata | `8f4b5c09849e4f5efa588baaa7c952c5c1c3731ba375abf7816c02a96c46ac78`<br>"Identificativo univoco assoluto per il tipo record, valorizzato in fas" … | Clausole progressivo univoco per tipo, riuso identico nei rinvii, annullamento se scompare. record_key/digits_only ricevuti coprono solo parte locale; CAD-12/15-H02 e profilo storico precisano ULSS/evento immutabile, file presente/assente, riattivazioni. Runtime storico mancante, audit PENDING_HISTORICAL_RUNTIME non ambiguità nuova. | C: digits_only; P: nessuna condizione |

## Tracciato B2 — 5 note

Testo integrale: [B2-fields.json](B2-fields.json); fonte `K/CAD_REVISED_CANDIDATE/trace_b2.json`.

| ID / indice nota | Classe | SHA-256 nota / estratto | Clausole, motivazione e limiti | Candidati ricevuti (inventario) |
|---|---|---|---|---|
| `3.1` / `fields[0]`; PDF p.4 | regola eseguibile candidata | `c0eb801f586e58884bbbf79095fdc55e79489e59719c12b5e7b27fb00e488fbc`<br>"Ricollega il servizio erogato all’evento assistenziale di CD (vedi TRA" … | Clausola ricollegamento evento ad A: join *_to_a_event ricevuto con contesto same_ulss_submission; G06 esplicita sending_ulss ed A assente→non valutabile. Audit DOCUMENTED_METADATA_OR_PHYSICAL_ONLY non prova esecuzione del join; qui classificata candidata per la relazione esplicita. | C: digits_only_if_present; P: nessuna condizione |
| `3.2` / `fields[1]`; PDF p.4 | metadata | `be7ede19f9e40011a20f34356ecc011a08a03275f540a1cae4401868fb24c380`<br>"Indica la data in cui inizia la sospensione dell’erogazione delle cure" … | Descrizione inizio/fine sospensione come metadata della data; audit PENDING_TEMPORAL_RUNTIME e CAD-14/profilo ricevuti precisano giorni interni, confini e fine aperta. La sola nota non definisce altri confronti; formato data non prova temporalità. | C: date; P: nessuna condizione |
| `3.3` / `fields[2]`; PDF p.4 | metadata | `e2cac6cad27af245902ac8c314f44363d33e1c3eccfbd7e749755ca1b0e068fc`<br>"Indica la data in cui termina la sospensione dell’erogazione delle cur" … | Descrizione inizio/fine sospensione come metadata della data; audit PENDING_TEMPORAL_RUNTIME e CAD-14/profilo ricevuti precisano giorni interni, confini e fine aperta. La sola nota non definisce altri confronti; formato data non prova temporalità. | C: date_if_present; P: nessuna condizione |
| `3.4` / `fields[3]`; PDF p.4 | regola eseguibile candidata | `3648d908e88d54aa05b1fa76d8882c8e8f58d70df14f9c58d7adb428ac23693d`<br>"I valori ammessi sono: 1 - ricovero in ospedale, 2 - allontanamento te" … | Dominio motivo sospensione1/2/3/4/7/9 in allowed_values ricevuto; CAD-14 applica effetto sospensione per ogni motivo valido, senza nuova deduzione clinica. | C: allowed_values, digits_only_if_present; P: nessuna condizione |
| `3.5` / `fields[4]`; PDF p.4 | regola eseguibile candidata | `8f4b5c09849e4f5efa588baaa7c952c5c1c3731ba375abf7816c02a96c46ac78`<br>"Identificativo univoco assoluto per il tipo record, valorizzato in fas" … | Clausole progressivo univoco per tipo, riuso identico nei rinvii, annullamento se scompare. record_key/digits_only ricevuti coprono solo parte locale; CAD-12/15-H02 e profilo storico precisano ULSS/evento immutabile, file presente/assente, riattivazioni. Runtime storico mancante, audit PENDING_HISTORICAL_RUNTIME non ambiguità nuova. | C: digits_only; P: nessuna condizione |

## Tracciato C — 11 note

Testo integrale: [C-fields.json](C-fields.json); fonte `K/CAD_REVISED_CANDIDATE/trace_c.json`.

| ID / indice nota | Classe | SHA-256 nota / estratto | Clausole, motivazione e limiti | Candidati ricevuti (inventario) |
|---|---|---|---|---|
| `4.1` / `fields[0]`; PDF p.5 | regola eseguibile candidata | `c0eb801f586e58884bbbf79095fdc55e79489e59719c12b5e7b27fb00e488fbc`<br>"Ricollega il servizio erogato all’evento assistenziale di CD (vedi TRA" … | Clausola ricollegamento evento ad A: join *_to_a_event ricevuto con contesto same_ulss_submission; G06 esplicita sending_ulss ed A assente→non valutabile. Audit DOCUMENTED_METADATA_OR_PHYSICAL_ONLY non prova esecuzione del join; qui classificata candidata per la relazione esplicita. | C: digits_only_if_present; P: nessuna condizione |
| `4.2` / `fields[1]`; PDF p.5 | regola eseguibile candidata | `7bd7a0515ab193c7a506ebf8b1abec13751d19a2a4f0ae3dc697a56ad6e13ddc`<br>"Utilizzare il formato GGMMAAAA" | Formato GGMMAAAA in date ricevuto; vincoli temporali aggiuntivi del profilo non attribuiti a questa sola nota. | C: date; P: nessuna condizione |
| `4.3` / `fields[2]`; PDF p.5 | metadata | `4f294e7e5c2a55d27c2d133c686da30a3797409914a1cb02277915e4d8514070`<br>"indica il numero di accessi della stessa figura professionale nella st" … | Descrive totale accessi stessa figura/data: audit DOCUMENTED_METADATA_OR_PHYSICAL_ONLY. Solo digits_only ricevuto, nessun dato elementare degli accessi per verificare il totale e nessuna aggregazione dedotta. | C: digits_only_if_present; P: nessuna condizione |
| `4.4` / `fields[3]`; PDF p.5 | regola eseguibile candidata | `9e9b5eeb7740ff2a24071dba42451372983fe9269aff1f925e72fb9d9b9781e3`<br>"Valori ammessi:1 - programmato Si, 2 - programmato No, 9 - non disponi" … | Dominio programmato1/2/9 in allowed_values ricevuto. | C: allowed_values, digits_only_if_present; P: nessuna condizione |
| `4.5` / `fields[4]`; PDF p.5 | regola eseguibile candidata | `53d318a847fcd051217ab4e98cc1bc74c2672a46a7355d61e0a2b93d2d8bcf8a`<br>"valori ammessi: 1 - MMG, 2 - PLS, 3 - infermiere, 4 - medico specialis" … | Clausole dominio figure professionali e codice15 non valido per PIC iniziate dal2025. allowed_values ricevuto copre dominio, documentato CAD-4.5-15/CAD-5.4-15 e profilo cross_trace_conditional_domains richiedono B1.2.4. Audit PENDING_CROSS_TRACE_RULE_2025: copertura parziale, non nuova decisione anno. | C: allowed_values, digits_only_if_present; P: nessuna condizione |
| `4.6` / `fields[5]`; PDF p.5 | regola eseguibile candidata | `801102646c77e4e3143951449a1aabca0f82e2e4935fe673a6626a1ce8500dc3`<br>"1 - adimed, 2 - programmata, 3 - malati oncologici*, 9 - altro\n*Assist" … | Dominio accessi MMG/PLS1/2/3/9 in allowed_values ricevuto; descrizione oncologici e DGR3889 come metadata, nessun controllo clinico dedotto. | C: allowed_values, digits_only_if_present; P: nessuna condizione |
| `4.7` / `fields[6]`; PDF p.5 | regola eseguibile candidata | `d30d408bc891ec062806648b913c503e045f47e528135370b16446bb02b78a8e`<br>"sede: 1 - a domicilio, 2 – in ospedale (dimissioni protette), 3 – a do" … | Clausole dominio sedi1–6, ricovero preclude1, decesso preclude1/2/4/5/6. allowed_values copre solo dominio; CAD-14/profilo precisano sospensione, conclusione, confini e sede3. Runtime temporale mancante; non introdurre sede7 per dati reali citati dall’audit. | C: allowed_values, digits_only_if_present; P: nessuna condizione |
| `4.8` / `fields[7]`; PDF p.5 | regola eseguibile candidata | `21a684cb2328973219bad64cdbc831815364262902b0f4e8264714851eb1a7c2`<br>"valori ammessi: 1 - gestione in appalto, 2 - gestione pubblica (preval" … | Clausole dominio titolarità1/2/3 e prevalenza gestione pubblica se compresente stessa giornata/figura. allowed_values copre solo dominio: precedenza/compresenza non rappresentata da regole nel trace. Clausola aggiuntiva candidata da formalizzare senza inventare deduplicazione o riscrittura record. | C: allowed_values, digits_only_if_present; P: nessuna condizione |
| `4.9` / `fields[8]`; PDF p.5 | regola eseguibile candidata | `ba66774f8f2c6a5c9a98dcb66dbd1255e6a8ed9377cfef6c28f504fe89a0f56e`<br>"Da utilizzare solo per il servizio 4 = medico specialista; utilizzare " … | Clausole solo medico specialista e codice tabella branca. forbidden_if ricevuto copre restrizione per figura; nessun lookup branca previsto per decisione CAD-E. Audit PARTIAL_FORBIDDEN_IF_IMPLEMENTED_NO_BRANCH_LOOKUP; non equivale a dominio specialistico verificato. | C: digits_only_if_present; P: forbidden_if |
| `4.10` / `fields[9]`; PDF p.5 | regola eseguibile candidata | `00dda4655915f8a9c1f3ac50eb5b702eed43a85c8a180e3ff28ddf6610c491f9`<br>"Da utilizzare solo per OSS da esprimere in minuti. Nel caso di unico a" … | Clausole solo OSS, minuti, somma se accesso unico con più operatori. forbidden_if per60/61 e digits_only ricevuti coprono solo figura/formato; nessun AST della somma, né dati operatori elementari. Copertura parziale oltre audit PARTIAL_FORBIDDEN_IF_IMPLEMENTED, nessuna somma inferita. | C: digits_only_if_present; P: forbidden_if |
| `4.11` / `fields[10]`; PDF p.5 | regola eseguibile candidata | `8f4b5c09849e4f5efa588baaa7c952c5c1c3731ba375abf7816c02a96c46ac78`<br>"Identificativo univoco assoluto per il tipo record, valorizzato in fas" … | Clausole progressivo univoco per tipo, riuso identico nei rinvii, annullamento se scompare. record_key/digits_only ricevuti coprono solo parte locale; CAD-12/15-H02 e profilo storico precisano ULSS/evento immutabile, file presente/assente, riattivazioni. Runtime storico mancante, audit PENDING_HISTORICAL_RUNTIME non ambiguità nuova. | C: digits_only; P: nessuna condizione |

## Tracciato D — 7 note

Testo integrale: [D-fields.json](D-fields.json); fonte `K/CAD_REVISED_CANDIDATE/trace_d.json`.

| ID / indice nota | Classe | SHA-256 nota / estratto | Clausole, motivazione e limiti | Candidati ricevuti (inventario) |
|---|---|---|---|---|
| `5.1` / `fields[0]`; PDF p.5 | regola eseguibile candidata | `c0eb801f586e58884bbbf79095fdc55e79489e59719c12b5e7b27fb00e488fbc`<br>"Ricollega il servizio erogato all’evento assistenziale di CD (vedi TRA" … | Clausola ricollegamento evento ad A: join *_to_a_event ricevuto con contesto same_ulss_submission; G06 esplicita sending_ulss ed A assente→non valutabile. Audit DOCUMENTED_METADATA_OR_PHYSICAL_ONLY non prova esecuzione del join; qui classificata candidata per la relazione esplicita. | C: digits_only_if_present; P: nessuna condizione |
| `5.2` / `fields[1]`; PDF p.5 | regola eseguibile candidata | `7bd7a0515ab193c7a506ebf8b1abec13751d19a2a4f0ae3dc697a56ad6e13ddc`<br>"Utilizzare il formato GGMMAAAA" | Formato GGMMAAAA in date ricevuto; vincoli temporali aggiuntivi del profilo non attribuiti a questa sola nota. | C: date; P: nessuna condizione |
| `5.3` / `fields[2]`; PDF p.5 | regola eseguibile candidata | `9e9b5eeb7740ff2a24071dba42451372983fe9269aff1f925e72fb9d9b9781e3`<br>"Valori ammessi:1 - programmato Si, 2 - programmato No, 9 - non disponi" … | Dominio programmato1/2/9 in allowed_values ricevuto. | C: allowed_values, digits_only_if_present; P: nessuna condizione |
| `5.4` / `fields[3]`; PDF p.5 | regola eseguibile candidata | `53d318a847fcd051217ab4e98cc1bc74c2672a46a7355d61e0a2b93d2d8bcf8a`<br>"valori ammessi: 1 - MMG, 2 - PLS, 3 - infermiere, 4 - medico specialis" … | Clausole dominio figure professionali e codice15 non valido per PIC iniziate dal2025. allowed_values ricevuto copre dominio, documentato CAD-4.5-15/CAD-5.4-15 e profilo cross_trace_conditional_domains richiedono B1.2.4. Audit PENDING_CROSS_TRACE_RULE_2025: copertura parziale, non nuova decisione anno. | C: allowed_values, digits_only_if_present; P: nessuna condizione |
| `5.5` / `fields[4]`; PDF p.5 | regola eseguibile candidata | `52172f7c09351b468b7042be03886395db6bc621523e3a5e4b942d4132c5dd9c`<br>"Indica le informazioni inerenti le prestazioni.I valori ammessi sono:\n" … | Elenco prestazioni completo nella nota lunga: reference_data cad-prestation-types ricevuto, CAD-10/16 e G05 distinguono questo dataset p5 dall’ICD9 p7. Fonte delle etichette conservata, nessun test runtime dedotto dall’elenco. | C: reference_data, digits_only_if_present; P: nessuna condizione |
| `5.6` / `fields[5]`; PDF p.5 | metadata | `8f9c9fc7d7673e02db004f45eea8a6a7979a7babc31ce9acb673c08c7ac8f8bf`<br>"Indica il numero di prestazioni erogate a domicilio nel caso in cui lo" … | Descrive totale prestazioni per operatore/giorno/tipologia quando più accessi: audit DOCUMENTED_METADATA_OR_PHYSICAL_ONLY. digits_only non verifica conteggio; istruzione di raccolta con dati elementari esterni, nessuna aggregazione automatica nuova. | C: digits_only_if_present; P: nessuna condizione |
| `5.7` / `fields[6]`; PDF p.5 | regola eseguibile candidata | `8f4b5c09849e4f5efa588baaa7c952c5c1c3731ba375abf7816c02a96c46ac78`<br>"Identificativo univoco assoluto per il tipo record, valorizzato in fas" … | Clausole progressivo univoco per tipo, riuso identico nei rinvii, annullamento se scompare. record_key/digits_only ricevuti coprono solo parte locale; CAD-12/15-H02 e profilo storico precisano ULSS/evento immutabile, file presente/assente, riattivazioni. Runtime storico mancante, audit PENDING_HISTORICAL_RUNTIME non ambiguità nuova. | C: digits_only; P: nessuna condizione |

## Registro DEC-05 — clausole ambigue

Voci documentali con stato esplicito (aperte, differite o chiuse); nessuna modifica runtime consegue
a questa registrazione. Citazioni esatte estratte dal source_note, senza
compressione whitespace interna. La motivazione distingue il testo ambiguo
dall’AST ricevuto e dalle decisioni già approvate.

### DEC-05-CAD-VALUTAZIONE-INIZIALE

Campi: `cad/B1/2.8` (`fields[7]` nella tabella di tracciato sopra).

Citazione esatta:

> 1 - valutazione iniziale (per ogni nuovo evento assistenziale di CD)

Motivo e fonte decisionale: Audit PENDING_FLOW_CARDINALITY_AT_LEAST_ONE_INITIAL_EVAL e sezione Lacune eseguibili punto5 chiedono conferma delle eventuali eccezioni. allowed_values1/2 non esprime la cardinalità; profilo storico da solo non ne chiude il significato.

Domanda aperta: Confermare ambito temporale e possibili eccezioni dell’almeno una iniziale: nuovo evento nell’invio o intero storico approvato? Non inventare errore per una iniziale assente in un solo trimestre.

### DEC-05-CAD-FORMATO-UVMD — CHIUSA come rilievo tecnico

Campi: `cad/B1/2.10` (`fields[9]` nella tabella di tracciato sopra).

Citazione esatta:

> Utilizzare il formato GGMMAAA

Motivo e fonte decisionale: Nota con7lettere contro physical.length8, source_typeD e date_if_present DDMMYYYY ricevuti. Audit conserva SOURCE_TYPO_GGMMAAA_8_BYTELENGTH ma l’inferenza del refuso non è correzione della fonte.

Esito: il formato ricevuto DDMMYYYY e D(8) è già esplicito; l’audit documenta il refuso. Si classifica la clausola come candidata ricevuta, senza correggere la nota né introdurre un nuovo formato. Nessuna ulteriore decisione è richiesta per questa trascrizione.

### DEC-05-CAD-ICPC-000 — RISOLTA DALL’AUTORE

Campi: `cad/B1/2.12` (`fields[11]` nella tabella di tracciato sopra).

Citazione esatta:

> se non è presente inserire il codice 000.

Motivo e fonte decisionale: Il candidato ricevuto dichiara missing_value.kind exact/raw000 e presence.base required sul campo AN(3), allineamento left/padspace. Perciò la rappresentazione richiesta quando patologia assente è classificata mancante e rifiutata dalla presenza. CAD-10/16 esclude lookup ICPC ma non decide questa distinzione semantica/assenza fisica.

Decisione ricevuta il 2026-10-19: «Confermo: 000 valido, spazi mancanti». Il campo resta obbligatorio; 000 è un valore semantico valido, tre spazi ASCII indicano assenza fisica. Decisione registrata in [DECISIONS.md](../DECISIONS.md); il kit e gli schemi originali restano invariati in questa task.

## Verifica documentale

Copertura, hash, identità delle note col derivato, classi consentite, unicità
e citazioni DEC-05 controllati con `docs/verification/check_notes.py`; evidenze nel
[report T-P0-04](../../verification/T-P0-04.md).
Nessun test Rust/CI, lint normativo completo o approvazione esterna dichiarati.
