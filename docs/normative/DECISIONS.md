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

## DEC-05-SIOC-OPTIONAL-PARENT — RISOLTA, 2026-10-19

Ambito: join opzionale `trace1_to_trace2_episode`, anagrafica SIOC del
tracciato 1 presente senza ricovero correlato del tracciato 2.

L’autore ha risposto:

> in questo caso è non applicabile e deve essere esplicitato che abbiamo l'anagrafica ma non i dati del ricovero correlato

- Policy del caso indicato: `NotApplicable`, nessun errore di parent mancante.
- Il controllo dell’anno nell’ID non viene dichiarato superato né fallito:
  è non applicabile perché mancano i dati del ricovero correlato.
- L’esito deve riportare esplicitamente «Anagrafica presente; dati del
  ricovero correlato assenti», con riferimento alla relazione interessata.
- La policy va dichiarata nello schema/profilo e il testo diagnostico resta
  configurabile; nessuna conoscenza dei campi SIOC nel core generico.
- La decisione riguarda l’assenza del target del join opzionale, non un
  target multiplo/ambiguo o una data invalida in un target presente. Questi
  casi conservano i controlli e gli esiti Unknown previsti dal contratto.

La precedente proposta Unknown per il target assente è superata dalla
risposta umana. Il kit FROZEN e gli snapshot ricevuti restano inalterati;
applicazione nella configurazione versionata della task pertinente.

Evidenze: [dossier SIOC](sioc/rules-and-joins.md),
[verifica T-P0-05](../verification/T-P0-05.md).

## CNF-LINTRA-DSL-01 — DEROGA CIRCOSCRITTA APPROVATA, 2026-10-19

L’autore ha risposto **«Approvo la deroga circoscritta (consigliata)»**
alla proposta A di T-P0-07. È autorizzata la definizione del contratto per
`substring`, `age_in_completed_years` e `calendar_year_delta`, necessari
ai profili ricevuti, in deroga al rinvio di Piano §13.5 / DET §135.

Implementazione nelle future task pertinenti, con prerequisiti, gate e test;
nessun codice Rust dei derivati, storage, temporali CAD o UI in T-P0-07.
L’autorizzazione non estende la DSL a funzioni arbitrarie e non approva
automaticamente il formato o l’intero contratto di authoring, che richiede
revisione umana finale. ARCH, DET e Master Plan sorgenti restano inalterati.

[Contratto da revisionare](../SCHEMA_AUTHORING.md).

## T-P0-07 — CONTRATTO COMPLETO APPROVATO, 2026-10-19

L’autore ha risposto **«Approvo il contratto T-P0-07»** alla revisione
del commit `72f9f5e`: SCHEMA_AUTHORING.md e artefatti collegati, inclusi
formato 1, policy persistenti, insiemi/consenso e piani compiled tipizzati.

L’accettazione chiude T-P0-07 e ratifica DEC-AUTHORING-FORMAT-01 e le
forme contrattuali di DEC-05-CAD-SET-CONTRACT / EVENT-CONTEXT. Non equivale
a compiler implementato, G0 chiuso, dataset SIOC ricevuti o profili
regionali eseguibili. Restano le boundary normative esplicite del contratto.
ARCH/DET/Master Plan originali non sono modificati; gli adeguamenti di
contratto delle task future sono quelli approvati nell’authoring.

## DEC-SIOC-MINIMAL-01 — PROFILO INIZIALE APPROVATO, 2026-10-19

L’autore richiede un validatore iniziale strutturalmente completo con controlli
semantici minimi e autorizza a procedere senza i dodici dataset mancanti.
È una riduzione esplicita del profilo iniziale, non una modifica della baseline
FROZEN o una dichiarazione di conformità normativa completa.

Conferma successiva ricevuta: `050` sul prefisso azienda inviante 1.17, 2.49,
3.35 e sulla regione sede UDO 4.7. Non imporre `050` a residenza 1.13 o
provenienza 2.29, che possono essere esterne al Veneto.

Nel profilo `2026-04-20.structural-minimal.1` restano layout byte-oriented,
tipi, padding, missing, presenza anche condizionale, PK e relazioni strutturali.
Si rinviano lookup, domini clinici, congruenze cliniche/temporali e codifica
anno/progressivo dell’ID; l’ID resta valorizzato e diverso da soli zeri.
Il rinvio della codifica ID in questo profilo minimale comprende anche il
2026; nel futuro profilo normativo completo resta la precedente distinzione
approvata per ricoveri iniziati nel 2025. Ogni omissione è registrata nel ledger.
Il controllo minuscole non viene attivato senza un repertorio esplicito;
la segnalazione dell’autore di dati maiuscoli non definisce una code page.

Per il futuro controllo dell’età compiuta SIOC l’autore sceglie **1 marzo**
(`mar01`) per i nati il 29 febbraio negli anni non bisestili. Non si abilita
il controllo d’età nel profilo minimale. I dataset sintetici dell’authoring
non sostituiscono quelli ufficiali. Encoding sempre richiesto al chiamante.
Persistenza minima PK/audit mantenuta come contratto, nessun backend implementato.

Artefatti e limiti: [profilo minimale](../../schemas/sioc/2026-04-20.structural-minimal.1/README.md),
[verifica T-P0-08](../verification/T-P0-08.md). Il gate G0 richiede ancora la
propria approvazione; questa decisione non vale come approvazione G0.

## T-P0-09 — G0 APPROVATO CON RINVII, 2026-10-19

L’autore ha risposto **«Approvo G0 con i rinvii indicati»** alla checklist
concreta del commit `407f89e`. T-P0-09 è DONE, G0 è chiuso con i rinvii
esplicitamente elencati e M0 raggiunta. L’ambito è il contratto generico
corrente e il profilo SIOC iniziale minimale, non la conformità normativa
completa SIOC o la promozione CAD a FROZEN. Restano tutti i prerequisiti
tecnici, la CI Linux/Windows reale e i gate runtime/persistenti.

[Checklist approvata](../../IMPLEMENTATION_STATUS.md#g0--checklist-t-p0-09-approvata).

## CNF-DOMAIN-01 — PROPOSTA A APPROVATA, 2026-10-19

L’autore approva integralmente la proposta A del
[dossier T-P2-03](../verification/T-P2-03-preflight.md), superando il precedente
blocco: SchemaKey neutra nel dominio, ScopeSummary Trace/Flow con soli ID e
ordine preservato, parte autonoma di T-P2-03 consegnata prima del residuo.
ValidationRunError resta nel confine applicativo con payload tecnici reali,
senza placeholder; le task con prerequisiti atomici soddisfatti possono
proseguire a partire da T-P3-01 dopo la verifica della parte autonoma.
T-P2-03 e M1 non sono complete finché il residuo non è verificato.
Nessuna modifica retroattiva di ARCH/DET/Piano o schemi FROZEN; nessuna
semantica normativa aggiunta e nessuna implementazione storage anticipata.


## ADR-PERSIST-002 — SQLITE LOCALE RATIFICATA, 2026-10-28

L’autore approva SQLite incorporato/statico dietro i port generici03, con
eccezione N-20 limitata al motore C e dipendenze qualificate DEC-15/N-26.
Il repository è il contratto del core; SQLitePersistenceAdapter gestisce schema,
indici, transazioni e storage locale. Preparazione/minimizzazione e HMAC04
precedono la scrittura. Nessun accesso SQL dal motore né norma CAD/SIOC nuova.
DELETE/EXTRA impostati e riletti, CAS/staging privato/snapshot immutabili, esito
commit ambiguo esplicito; formato storage indipendente e nessun reset implicito.
Query puntuali/cache limitata, load_snapshot O(n) riconosciuto, benchmark reali
fino300k e gate Linux/MSVC necessari. Nessuna fonte organizzativa condivisa,
sincronizzazione Git/share o database server. Restano aperti cifratura a riposo,
retention, provisioning/rotazione chiavi, backup sicuro e DEC-14 quantitativa;
non autorizzati dati sanitari reali. Binding rusqlite0.41.0 bundled con SQLite
3.54.1/libsqlite3-sys0.39.1, dettaglio e checksum nel dossier dipendenze.

[Prescrizioni vincolanti e tracciabilità ADR001/03/04](../verification/ADR-PERSIST-002-ratification.md).
Questa registrazione non riscrive le decisioni storiche né le baseline normative.
