# LINTRA — roadmap ingegneristica post-M9

10 novembre 2026. Proposta decisionale basata sul
[report](LINTRA_RETROSPECTIVE_AUDIT_REPORT.md) e sul
[registro delle lacune](LINTRA_POST_M9_GAP_REGISTER.md). Non è una nuova lista
di task READY, non modifica le 140 task DONE e non autorizza automaticamente
implementazioni, migrazioni o distribuzioni.

L'ordine suggerito segue rischio e risultato d'uso. Le decisioni umane già prese
(C1, L1, chiave comune, diagnostiche phase-major, produzione sequenziale) sono
prerequisiti acquisiti; le decisioni aperte rimangono esplicite.

## Percorso 1 — Correttezza e hardening

| Ordine | Intervento / evidenza | Beneficio e perimetro | Uscita verificabile |
|---|---|---|---|
| 1 | Due correzioni SQLite già implementate in af3bccc, AUD-M9-001/002 | Elimina falsa corruzione su letture interleaved e falsa assenza con latest dangling; delta localizzato, nessun formato/API nuovo | Red riprodotti; green adapter e suite; qualifica nativa sullo SHA esatto nel dossier, senza usare la CI storica per il delta |
| 2 | Conservare regressioni, tracciabilità e frase README corretta | Evita reintroduzione di due guasti e istruzioni obsolete | Commit, log/digest e nuove regressioni collegati ai finding; nessun golden riscritto |
| 3 | Deliberare e poi implementare eventuale budget input/reference, GAP-08 | Limita consumo sotto input sproporzionati e latenza di cancellation | Contratto approvato, sonde bounded; EOF/CRLF/hash/diagnostiche/report invariati nei casi ammessi |
| 4 | Valutare consumo spreadsheet dei CSV, GAP-07 | Riduce interpretazioni indesiderate nel consumatore previsto | Policy esplicita, verifica nativa nel consumatore scelto e test di preservazione della fonte canonica |
| 5 | Chiarire requisito di tree reference controllata, GAP-09 | Evita deployment fuori dal modello qualificato | Controllo operativo dimostrato; hardening per handle solo se il nuovo modello lo richiede e viene autorizzato |

Le righe 3–5 sono rischi/condizioni operative, non bug normativi dimostrati.
Non va introdotto escaping tacito, un limite numerico arbitrario o una nuova
garanzia di sicurezza con un refactoring. Il progresso delle prime due righe è
effettivo e già nel codice; i risultati finali sono nel ledger dei gate.

Responsabilità: manutentore per correzioni/gate; autore per i nuovi contratti
di consumo/risorse. Impatto del delta già attuato basso e circoscritto; ulteriori
stime dipendono dalle decisioni, non sono ricavate dal numero di righe.

## Percorso 2 — Readiness operativa

Questo percorso ha priorità P1 **prima dei dati sanitari reali**, indipendentemente
dalla scelta di aggiungere schermate o nuovi profili.

1. **Definire il contesto di esercizio e le responsabilità — GAP-01.**
   Utenti, macchine/filesystem, dati e destinatari ammessi, autorizzazione d'uso
   e criterio di accettazione. Output: decisione operativa versionata; nessuna
   certificazione legale o clinica derivata da M9.
2. **Ratificare la protezione a riposo e dei report — GAP-02/06.**
   Inventario DB/journal/temporanei/report/copie, accessi e redazione, confine
   di fiducia. Scegliere i controlli solo dopo la decisione; non confondere HMAC
   con cifratura o namespace con autenticazione multiutente. Uscita: prove
   native Linux/Windows di creazione, sostituzione, errore, cleanup e recupero.
3. **Definire custodia e risposta alla compromissione delle chiavi — GAP-04.**
   Preservare file comune e Key ID negli upgrade; recupero da copia protetta,
   scenari di perdita e compromissione. Qualsiasi rotazione/migrazione richiede
   una specifica approvata. Uscita: drill sintetico senza reset o perdita storica.
4. **Approvare retention e trattamento delle copie — GAP-03.**
   Durata, eccezioni, responsabilità e compatibilità con immutabilità degli
   snapshot. Uscita: policy verificabile; nessuna cancellazione automatica
   introdotta senza decisione e nessuna promessa di erase fisico da unlink.
5. **Qualificare il recupero operativo completo — GAP-05.**
   Destinazioni/frequenza/accessi, inventario atteso, freschezza e custodia
   separata della chiave. Partire dalle prove di copia chiusa già esistenti;
   non chiamarle backup live. Uscita: restore isolato con namespace, revisioni,
   manifest/fatti e input Present/Absent confrontati, danno e chiave errata
   rifiutati, tempi misurati rispetto agli obiettivi approvati.

Dipendenze: contesto → scelte di protezione/retention/chiave → procedura di
backup e collaudo → autorizzazione operativa finale. Le analisi documentali
possono essere indipendenti, ma qui non è approvata un'esecuzione concorrente.
Il rischio condiviso della chiave comune resta quello già ratificato: non si
riapre automaticamente il provisioning per sostituirlo con un provider nuovo.

## Percorso 3 — Completamento funzionale

| Priorità proposta | Risultato di prodotto | Prerequisiti e decisioni | Criterio di completamento |
|---|---|---|---|
| P2 | Full multi-trace raggiungibile da Slint, GAP-10 | Scope GUI autorizzato, semantica di Partial/Unknown, progress/cancellation; Signature preservata | Stesse richieste/diagnostiche/summary del servizio reale, niente norme nel frontend; regressioni funzionali e visuali native |
| P2 | CAD83 snapshot raggiungibile dalle interfacce scelte, GAP-11 | Readiness pertinente, parametri/archivio/chiave, origini C1, omissioni/review; C1/L1 già ratificate | T1/T2/T3, due namespace, annullamenti L1, Unknown senza approval, report failure/CAS/replay verificati dal percorso utente |
| P2 | Profilo SIOC completo determinato e qualificabile, GAP-12 | Riconciliare ogni rinvio; dati ufficiali realmente forniti, versioni/hash, regole e primitive | Nuovo profilo versionato con ledger di ogni esclusione/integrazione; golden normativi, dataset indisponibili distinti da set vuoti, capability non eseguibili ancora rifiutate |
| P2 se richiesto | Enforcement SIOC cross-run, GAP-13 | Decisione normativa DEC-SIOC-HIST-01 | Casi leciti/illeciti/Unknown ratificati e testati; fino ad allora rimane audit-only |
| P2 quando si distribuisce | Artefatto del nuovo SHA qualificato, GAP-15 | Scope release, provenienza, packaging/runbook, upgrade/key continuity, eventuale gate normativo CAD | Artefatti Linux/MSVC riproducibili, checksum e audit, autorizzazione di pubblicazione; release/freeze/readiness distinti |

Le prime due integrazioni possono riutilizzare servizi esistenti. Non devono
duplicare il compilatore, i piani storici o le regole in UI/CLI. La schermata
di conferma deve approvare il candidato effettivamente mostrato e non attribuire
a PROCEED la copertura mancante. Non attivare snapshot passando dal percorso
audit e togliendo capability per far compilare lo schema.

Il completamento SIOC non coincide con “fornire qualche dataset”: il ledger
conserva domini, regole, gruppi e integrazioni non eseguibili. Prima occorre
separare ciò che ha già una decisione e una primitiva disponibile da ciò che
richiede nuova fonte o contratto. Gli elenchi CAD ricevuti non diventano
cataloghi nazionali completi, né ICPC/ISTAT sono inventati per chiudere il registro.

La migrazione React/Tauri in docs/migration resta un workstream distinto. Un
nuovo adapter dovrà rispettare gli stessi servizi, storici e chiavi, ma questo
non autorizza a modificarlo o ad avviarlo nell'audit.

## Percorso 4 — Miglioramenti opzionali

1. **Profiling operativo prima delle ottimizzazioni — GAP-14.**
   Concordare aggregati anonimi di volumi, distribuzione per trace, errori,
   reference e crescita storico. Misurare end-to-end e attribuire costi a
   compilazione, parsing, validation, projection, history/SQLite e report.
   Confrontare carichi bilanciati/sbilanciati e UI separatamente. Risultato:
   una proposta A/B con beneficio concreto e costo, oppure decisione motivata
   di non cambiare nulla. Nessuna riapertura automatica DEC-17.
2. **Valutare memoria owned e rilettura dello storico soltanto dopo il profilo.**
   load_snapshot materializza O(n); staging, token e validità per campo hanno
   costi reali documentati. Cache/streaming additivi o API diverse possono essere
   proposti se le misure dimostrano un problema operativo, preservando CAS,
   immutabilità, autenticità e isolamento. Nessun refactoring per sola eleganza.
3. **Qualifica hardware ulteriore solo per garanzie richieste — GAP-16.**
   Il crash di processo è già testato; ulteriori prove fisiche richiedono
   hardware/modello/criteri definiti. Non dichiarare una proprietà di power-loss
   sulla base delle prove di processo esistenti.
4. **Allineamento della fonte originale dell'audit — GAP-17.**
   Se il documento separato viene reperito, confrontare eventuali requisiti
   aggiuntivi con il mandato eseguito. Nessun nuovo lavoro automatico nasce
   dall'assenza del file o da TODO storici già superati.

## Handoff decisionale

L'autore può scegliere come prossimo risultato la readiness operativa oppure
una specifica integrazione funzionale in ambiente sintetico. La prima è
necessaria prima dei dati reali; la seconda può essere preparata preservando
quel limite. Il profiling resta proporzionato a una domanda operativa e a
volumi concordati, non un nuovo ciclo generale di sviluppo.

L'audit consegna correzioni, evidenze e decisioni necessarie. Non crea una nuova
sequenza approvata di task, un tag o un freeze; nessuna attività ordinaria READY
residua nel piano originale viene inventata al termine.
