# LINTRA — SIGNATURE UI OVERHAUL
## Refactoring integrale dell'interfaccia Slint con UI UX Pro Max → Impeccable → No AI Slop

**TIPO DI INTERVENTO:** Redesign stilistico e UX completo della UI esistente.

**AUTORIZZAZIONE:** Esplicita.

**OBIETTIVO:** Portare l'interfaccia di Lintra da una realizzazione visivamente basilare a un prodotto desktop professionale di alto livello, moderno, distintivo, accattivante e tecnicamente solido.

**PRIORITÀ:** Le istruzioni di questo messaggio sostituiscono le precedenti indicazioni estetiche eventualmente incompatibili. Restano invariati i vincoli normativi, architetturali, di sicurezza e funzionali del progetto.

---

# 1. Visione del prodotto

Lintra è un motore di validazione di flussi dati regionali, sviluppato in Rust, con interfaccia grafica nativa Slint.

Non è un generico gestionale sanitario e non deve essere progettato come una tradizionale applicazione per medici, infermieri o pazienti.

Il pubblico principale è costituito da:

- Analisti esperti di flussi informativi regionali.
- Professionisti che conoscono tracciati, campi, schemi, versioni e controlli normativi.
- Operatori che analizzano quotidianamente file e risultati di validazione.
- Utenti tecnici che devono individuare, interpretare e risolvere rapidamente errori nei dati.
- Futuri utilizzatori meno esperti, che devono poter apprendere il funzionamento senza incontrare un'interfaccia ostica.

La progettazione deve quindi adottare un modello **expert-first, novice-accessible**.

Per l'utente esperto, la UI deve favorire velocità, precisione, densità informativa controllata e accesso immediato alle informazioni rilevanti.

Per l'utente meno esperto, deve garantire gerarchie comprensibili, terminologia coerente, feedback chiari e percorsi operativi prevedibili.

Non semplificare artificialmente il dominio eliminando informazioni che servono agli analisti.

Non rendere però ogni schermata una superficie sovraccarica di dati.

---

# 2. Ambizione estetica: Lintra è un flagship product

Lintra è il progetto di punta del suo sviluppatore.

L'interfaccia deve rispecchiare l'ambizione del prodotto.

**Non è accettabile un risultato semplicemente pulito, ordinato o funzionale ma esteticamente anonimo.**

Il redesign deve produrre una UI che trasmetta immediatamente:

- Qualità ingegneristica.
- Modernità.
- Precisione.
- Professionalità.
- Identità visiva.
- Cura dei dettagli.
- Solidità del prodotto.
- Coerenza compositiva.
- Competenza nel trattamento di dati complessi.

Voglio un'applicazione desktop che sembri progettata intenzionalmente da un team di product design esperto, non una collezione di widget standard affiancati.

La direzione deve essere contemporanea, visivamente interessante e riconoscibile.

Non interpretare professionalità come monotonia cromatica, assenza di carattere o minimalismo estremo.

Puoi utilizzare:

- Una palette distintiva e sofisticata.
- Tipografia con una gerarchia forte.
- Contrasti visuali consapevoli.
- Superfici e profondità ben controllate.
- Composizioni moderne.
- Iconografia coerente.
- Indicatori di stato curati.
- Microinterazioni appropriate.
- Ritmo e proporzioni visive riconoscibili.

Evita tuttavia:

- Gradienti decorativi indiscriminati.
- Glassmorphism utilizzato senza motivazione.
- Eccesso di card.
- Elementi troppo arrotondati senza logica.
- Ombre casuali.
- Dashboard SaaS generiche.
- Estetica ludica.
- Animazioni che rallentano il lavoro.
- Effetti che riducono leggibilità e precisione.
- Interfacce che sembrano landing page anziché strumenti desktop.

Il criterio guida è:

**Visualmente ambiziosa, operativamente eccellente.**

L'interfaccia deve essere memorabile grazie al proprio sistema visivo, non per l'accumulo di effetti.

---

# 3. Scalabilità del prodotto e neutralità rispetto ai flussi

Il redesign non deve essere costruito intorno al solo caso d'uso SIOC.

Lintra è un motore generico, versionato e schema-driven, destinato a supportare diversi flussi regionali.

L'interfaccia deve essere progettata per adattarsi a:

- SIOC.
- CAD.
- Ulteriori flussi futuri.
- Versioni differenti dello stesso tracciato.
- Schemi con strutture diverse.
- Differenti insiemi di reference data.
- Diversi livelli di disponibilità delle capability.
- Nuove funzionalità future, quando realmente implementate.

Non introdurre nomi di flussi, numeri di tracciati o campi normativi hardcoded nei componenti generici della UI.

Il design system deve permettere di aggiungere nuove schermate senza dover reinventare colori, spaziature, controlli o modelli di navigazione.

Se una scelta stilistica funziona soltanto con i dati SIOC attualmente visibili, non è sufficientemente generale.

**La qualità del redesign deve valere per il prodotto Lintra, non per una singola schermata dimostrativa.**

---

# 4. Regola fondamentale: le skill devono lavorare in sequenza

Le seguenti tre skill devono essere applicate nell'ordine esatto:

**FASE A — UI UX Pro Max**

→ definizione della direzione estetica, del design system e delle scelte iniziali di UX.

**FASE B — Impeccable**

→ trasformazione e rifinitura dell'interfaccia Slint utilizzando come input vincolante il lavoro prodotto dalla Fase A.

**FASE C — No AI Slop**

→ revisione editoriale dei testi della UI risultante, utilizzando come riferimento i componenti, le gerarchie e i flussi stabilizzati nelle prime due fasi.

**FASE D — Quality Review integrata**

→ verifica finale da parte dell'agente principale, senza avviare altre skill in parallelo.

Ogni fase deve produrre artefatti espliciti che costituiscano l'input della fase successiva.

Non trattare le tre skill come consulenti indipendenti.

Non generare tre proposte estetiche separate.

Non applicare i risultati di una skill ignorando le decisioni della precedente.

Se una fase evidenzia un conflitto con una decisione estetica precedente, correggi il design system in modo tracciabile e giustificato.

Non creare tre fonti di verità stilistiche incompatibili.

---

# 5. Preflight: ricostruzione dello stato reale

Prima di utilizzare le skill e modificare il codice:

1. Leggi `AGENTS.md` e le istruzioni operative applicabili.
2. Esamina lo stato Git.
3. Consulta `IMPLEMENTATION_STATUS.md` e l'handoff più recente.
4. Individua il crate `lintra-ui`.
5. Individua tutti i file Slint effettivamente utilizzati.
6. Ricostruisci l'integrazione con i view model e i callback Rust.
7. Individua i documenti di design preliminari.
8. Individua gli eventuali design token e componenti condivisi.
9. Ricostruisci le schermate effettivamente implementate.
10. Verifica gli strumenti disponibili per eseguire e acquisire screenshot della UI.

Non assumere che tutte le schermate previste dal piano siano già implementate.

Non inventare componenti, stati, modelli o capability inesistenti.

## Baseline visuale

Prima del refactoring, acquisisci screenshot delle schermate principali realmente disponibili, includendo gli stati rappresentativi che riesci a riprodurre in modo sicuro.

Utilizza fixture sintetiche.

Non utilizzare dati sanitari personali.

Registra i difetti osservati, distinguendo:

- Problemi estetici.
- Problemi di gerarchia.
- Problemi di layout.
- Problemi di navigazione.
- Problemi di densità informativa.
- Problemi di accessibilità.
- Problemi di presentazione degli stati.

La baseline serve a dimostrare concretamente la differenza fra interfaccia iniziale e risultato finale.

Non dichiarare problemi visivi osservati se hai potuto soltanto leggere il codice.

---

# 6. FASE A — UI UX PRO MAX
## Direzione creativa e design system

### Localizzazione obbligatoria

La skill `ui-ux-pro-max` è installata all'interno del repository Lintra nel percorso:

```text
.codex/skills/ui-ux-pro-max/
```

Il file principale da caricare è:

```text
.codex/skills/ui-ux-pro-max/SKILL.md
```

Questa è la posizione autorevole dichiarata per il progetto.

**Non cercare una copia globale alternativa e non sostituire questa skill con un'altra.**

Partendo dalla root del repository, verifica l'esistenza del file e leggilo integralmente.

Individua quindi gli script e i riferimenti effettivamente disponibili al suo interno.

Se il percorso non esiste o è inutilizzabile, riporta il problema reale. Non fingere che una versione diversa della skill sia equivalente.

### Esecuzione reale

Se lo script è presente e la documentazione ne conferma la sintassi, esegui una ricerca iniziale di design system equivalente a:

```bash
python3 .codex/skills/ui-ux-pro-max/scripts/search.py \
  "professional desktop data validation workbench expert users modern distinctive sophisticated information dense" \
  --design-system \
  -p "Lintra" \
  -f markdown
```

La sintassi effettiva deve essere verificata contro il `SKILL.md` installato.

Esegui successivamente soltanto le ricerche mirate necessarie per definire con maggiore precisione:

- Direzione estetica.
- Sistema cromatico.
- Tipografia.
- Pattern per applicazioni operative.
- Tabelle dense.
- Form e selezione file.
- Navigazione.
- Feedback.
- Stati di validazione.
- Accessibilità.
- Gestione di interfacce con informazioni tecniche complesse.

Non impostare forzatamente uno stack web quando la skill non supporta Slint.

I risultati di UI UX Pro Max devono essere tradotti in principi e specifiche implementabili con la versione Slint realmente utilizzata nel progetto.

Non introdurre HTML, CSS, Tailwind o React per applicare raccomandazioni della skill.

### Selezione della direzione visiva

Utilizza la skill per esplorare alternative estetiche plausibili.

Confronta brevemente le opzioni secondo:

- Originalità e riconoscibilità.
- Coerenza con un prodotto professionale per analisti.
- Modernità.
- Leggibilità.
- Capacità di gestire tabelle e risultati complessi.
- Scalabilità futura.
- Fattibilità in Slint.
- Accessibilità.

Seleziona **una sola direzione estetica finale**, motivandola rispetto al brief.

Non scegliere automaticamente la soluzione più conservativa.

Non scegliere automaticamente la soluzione più decorativa.

Ricerca il miglior equilibrio fra personalità visiva e operatività.

### Design system obbligatorio

La Fase A deve produrre un design system completo, comprendente almeno:

**Identità visiva**
- Principi del linguaggio visivo.
- Personalità del prodotto.
- Sistema cromatico distintivo.
- Colori semantici.
- Regole per superfici, bordi, raggi e profondità.

**Tipografia**
- Famiglie tipografiche compatibili con la distribuzione.
- Scala dei caratteri.
- Peso e gerarchia.
- Trattamento dei dati numerici.
- Trattamento di codici, percorsi, identificativi e valori tecnici.
- Leggibilità delle tabelle.

**Composizione**
- Griglia.
- Spaziature.
- Proporzioni.
- Navigazione.
- Struttura delle schermate.
- Densità dei dati.
- Contenitori.
- Dialog.

**Componenti**
- Pulsanti.
- Campi.
- Selettori.
- Tabelle.
- Indicatori.
- Notifiche.
- Stati vuoti.
- Messaggi di errore.
- Componenti di riepilogo.
- Controlli di avanzamento.
- Elementi di navigazione.

**Interazione**
- Hover, focus e selezione.
- Stati disabilitati.
- Feedback delle azioni.
- Priorità visiva.
- Microinterazioni.
- Navigazione da tastiera.
- Comportamento al ridimensionamento.

Non aggiungere effetti visuali che introducano dipendenze tecniche sproporzionate rispetto al loro valore.

### Output obbligatorio della Fase A

Produci una specifica persistente, ad esempio:

```text
docs/ui/LINTRA_DESIGN_DIRECTION.md
```

e aggiorna il design system canonico già presente nel repository, se esistente.

Se la skill genera un proprio `design-system/MASTER.md`, verifica prima gli eventuali file esistenti e integra il risultato evitando duplicazioni o sovrascritture inconsapevoli.

L'output deve documentare:

- La direzione scelta.
- Le alternative scartate.
- I token visuali definiti.
- Le regole applicabili ai componenti.
- I pattern UX selezionati.
- Gli antipattern da evitare.
- Le decisioni da trasmettere a Impeccable.

**Non iniziare la Fase B prima di avere completato e registrato questi risultati.**

---

# 7. FASE B — IMPECCABLE
## Redesign, implementazione e rifinitura visuale

### Input obbligatorio

Impeccable deve ricevere come punto di partenza:

1. Il design system prodotto dalla Fase A.
2. La direzione estetica selezionata.
3. La baseline visuale.
4. L'inventario delle schermate e dei componenti.
5. I problemi UX iniziali.
6. I vincoli Rust/Slint.
7. Le priorità operative degli esperti di flussi.

**Non consentire a Impeccable di ignorare o rimpiazzare arbitrariamente la direzione visiva definita da UI UX Pro Max.**

Impeccable deve svilupparla, migliorarla e renderla coerente nel prodotto reale.

### Caricamento della skill

Individua la skill `impeccable` realmente installata nell'ambiente Codex.

Leggi il relativo `SKILL.md` e i riferimenti pertinenti.

Verifica le procedure effettivamente supportate dalla versione disponibile.

Utilizza le funzionalità più pertinenti a:

- Redesign.
- Shape e composizione.
- Design critique.
- Visual hierarchy.
- Typography.
- Color.
- Layout.
- Spacing.
- Design system extraction.
- UI audit.
- Polish.
- Harden.
- Revisione degli stati interattivi.

Se sono presenti procedure specifiche per rendere più distintiva una UI eccessivamente generica, valuta la loro applicazione.

Non invocare comandi inesistenti.

Non utilizzare subagenti.

Se una procedura richiede agenti paralleli, applica solo le tecniche compatibili attraverso il singolo agente principale e registra la limitazione.

### Redesign sostanziale

Questa non è una normale task di manutenzione stilistica.

Se la UI attuale presenta una struttura visivamente debole, puoi sostituire la composizione della schermata, purché le funzionalità esistenti restino corrette e accessibili.

Non limitarti a:

- Cambiare il colore primario.
- Modificare qualche border-radius.
- Aumentare il padding.
- Sostituire le icone.
- Aggiungere una sidebar decorativa.

Intervieni sul sistema visuale complessivo.

Le schermate devono condividere una grammatica di design riconoscibile.

L'applicazione deve sembrare un unico prodotto, non un insieme di pagine sviluppate in momenti diversi.

### Esperienza per utenti esperti

Particolare attenzione deve essere rivolta a:

**Selezione e preparazione**
- Accesso chiaro ai flussi disponibili.
- Selezione delle versioni.
- Identificazione immediata dei tracciati.
- Visibilità delle dipendenze.
- Riepilogo della configurazione prima dell'esecuzione.

**Esecuzione**
- Stato del processo.
- Progresso realmente disponibile.
- Distinzione fra attività in corso e anomalie.
- Feedback non invasivo.
- Possibilità di interpretare un'interruzione.

**Consultazione dei risultati**
- Gerarchia fra esito complessivo e dettaglio.
- Tabelle leggibili anche con molte righe.
- Codici errore facilmente riconoscibili.
- Riferimenti ai campi e ai tracciati.
- Navigazione nei dettagli.
- Distinzione fra severità.
- Filtri e ordinamenti esistenti.
- Utilizzo efficiente dello spazio.

**Funzionalità storiche**
- Chiarezza fra sessioni, snapshot e invii.
- Distinzione fra anteprima e approvazione.
- Immutabilità degli stati approvati.
- Rappresentazione corretta delle differenze storiche.

Intervieni soltanto sulle funzionalità effettivamente implementate.

Le capability future possono influenzare l'estensibilità dei componenti, ma non devono essere simulate come disponibili.

### Semantica degli stati

Mantieni chiaramente distinguibili:

- Violazione normativa.
- Warning.
- Errore tecnico.
- Controllo saltato.
- Verifica inconcludente.
- Evidenza provvisoria.
- Validazione completa.
- Validazione parziale.
- Risultato valido.
- Risultato non valido.
- Funzionalità non disponibile.

Non associare indiscriminatamente il verde a un'esecuzione conclusa quando esistono controlli inconcludenti.

Non presentare una validazione parziale come certificazione di conformità.

Utilizza colore, icone, testi e gerarchia in modo complementare.

### Refactoring Slint

Implementa il nuovo design nel codice nativo esistente.

Dove opportuno:

- Centralizza i design token.
- Realizza componenti riutilizzabili.
- Uniforma controlli equivalenti.
- Migliora i layout.
- Rafforza la tipografia.
- Ottimizza la densità delle tabelle.
- Migliora focus, selezione e feedback.
- Rendi coerenti dialog e notifiche.
- Introduci microinterazioni sobrie quando sostenibili.

Non introdurre astrazioni eccessive.

Non riscrivere la logica Rust di dominio.

Non cambiare i contratti del ValidationEngine, dello SchemaCompiler o della persistenza.

Il codice deve rimanere mantenibile e coerente con la versione Slint del repository.

### Output obbligatorio della Fase B

La fase deve concludersi con:

- Refactoring concreto delle schermate disponibili.
- Componenti condivisi coerenti.
- Design token applicati.
- Screenshot aggiornati.
- Elenco dei principali miglioramenti.
- Eventuali limiti visuali ancora presenti.
- Verifica preliminare delle interazioni.

**La Fase C deve partire dai testi presenti nella UI risultante dalla Fase B, non da quelli originari.**

---

# 8. FASE C — NO AI SLOP
## Microcopy professionale e precisione linguistica

### Input obbligatorio

La skill `no-ai-slop` deve ricevere:

- La direzione estetica finale.
- La gerarchia delle schermate.
- I componenti effettivamente implementati.
- L'inventario delle stringhe UI.
- I flussi operativi effettivi.
- La terminologia del dominio.
- I vincoli normativi.

Individua e leggi il `SKILL.md` della versione installata.

Non utilizzare una versione generica o ipotetica della skill se quella effettiva è disponibile.

### Obiettivo editoriale

Lintra deve comunicare con un linguaggio:

- Professionale.
- Preciso.
- Diretto.
- Concreto.
- Coerente.
- Tecnico quando necessario.
- Accessibile senza essere banalizzato.

Non rendere l'interfaccia artificiosamente amichevole.

Non scrivere come un'applicazione consumer.

Non utilizzare microcopy promozionale.

Non sostituire termini tecnici corretti con sinonimi più semplici ma meno precisi.

### Stringhe da revisionare

Intervieni sulle stringhe visuali pertinenti:

- Titoli e sottotitoli.
- Etichette.
- Pulsanti.
- Testi di supporto.
- Placeholder.
- Stati vuoti.
- Messaggi informativi.
- Conferme.
- Dialog.
- Warning della UI.
- Feedback di caricamento.
- Messaggi relativi alle azioni dell'utente.

Evita testi ridondanti e formulazioni generiche.

Preferisci un testo che spieghi chiaramente lo stato o l'azione successiva.

### Protezione della semantica normativa

Non modificare arbitrariamente:

- Codici di errore normativi.
- Testi diagnostici contrattuali.
- Identificativi di regola.
- Identificativi di campo.
- Trace ID.
- Identificatori di schema.
- Valori di dominio.
- Terminologia definita dalla normativa.
- Distinzioni fra `Unknown`, `NotApplicable`, violazione e risultato verificato.
- Versioni e informazioni di provenienza.

La revisione editoriale riguarda il livello comunicativo della UI.

Se una stringa normativa è difficile da interpretare, puoi migliorarne la presentazione e aggiungere un contesto esplicativo compatibile con i contratti, senza alterare il significato originario.

### Output obbligatorio della Fase C

Produci:

- Le stringhe UI revisionate nel codice.
- Un piccolo glossario delle denominazioni ricorrenti.
- Esempi verificabili prima/dopo.
- Elenco delle stringhe normative lasciate intenzionalmente invariate.
- Indicazione delle eventuali incongruenze editoriali rimaste.

Non dichiarare la skill applicata senza modifiche o senza motivare perché le stringhe esistenti non richiedevano interventi.

---

# 9. FASE D — REVISIONE FINALE INTEGRATA

Dopo le tre skill, svolgi una verifica complessiva come agente principale.

Non avviare nuovi agenti.

Non ripartire da zero con una nuova direzione estetica.

Controlla che le tre fasi abbiano contribuito a un unico prodotto coerente.

## Criteri visuali

La UI deve dimostrare:

- Una personalità visiva riconoscibile.
- Una direzione moderna.
- Una palette coerente.
- Una gerarchia tipografica forte.
- Un sistema di spaziature consistente.
- Componenti professionali.
- Navigazione chiara.
- Tabelle adatte a un utilizzo intensivo.
- Buona leggibilità dei dettagli tecnici.
- Un buon equilibrio fra contenuto e spazio vuoto.
- Microinterazioni utili.
- Assenza di decorazioni gratuite.
- Coerenza fra schermate.
- Possibilità di estensione a nuovi flussi.

**Se il risultato finale è soltanto più pulito ma ancora genericamente anonimo, il requisito estetico principale non è soddisfatto.**

In quel caso individua le carenze specifiche e correggile all'interno di un passaggio finale di rifinitura, senza avviare cicli indefiniti.

## Criteri UX

Verifica che un utente esperto riesca a:

- Comprendere rapidamente dove si trova.
- Individuare il flusso e lo schema selezionati.
- Capire quali dati sta per validare.
- Distinguere configurazione ed esecuzione.
- Leggere l'esito complessivo.
- Individuare le anomalie rilevanti.
- Accedere al dettaglio disponibile.
- Distinguere errori normativi e tecnici.
- Capire se il risultato è completo.
- Riconoscere le azioni disponibili.

Verifica inoltre che un utente meno esperto disponga di etichette e feedback sufficientemente chiari.

Non nascondere informazioni importanti soltanto per ridurre il rumore visivo.

## Criteri di estensibilità

La UI deve poter accogliere nuovi flussi senza dipendere da:

- Posizioni fisse dei tracciati SIOC.
- Numero prefissato di file.
- Etichette normative hardcoded.
- Dati dimostrativi.
- Componenti duplicati per ciascun flusso.

Le differenze fra flussi devono essere rappresentate a partire dai modelli realmente disponibili.

---

# 10. Verifica visuale reale e confronto prima/dopo

La revisione non può essere dichiarata riuscita soltanto perché la build compila.

Devi osservare, per quanto tecnicamente possibile, l'applicazione effettivamente renderizzata.

Utilizza gli strumenti nativi compatibili con Slint disponibili nel progetto.

Verifica le principali schermate alle seguenti risoluzioni o dimensioni di finestra rappresentative:

- 1366×768.
- 1920×1080.
- 1920×1200.

Adatta la verifica alle dimensioni minime realmente supportate dall'applicazione.

Per ogni schermata controlla:

- Assenza di sovrapposizioni.
- Assenza di testi tagliati.
- Corretta gestione del ridimensionamento.
- Gerarchia leggibile.
- Corretta densità delle tabelle.
- Controlli accessibili.
- Focus visibile.
- Contrasti adeguati.
- Dialog correttamente posizionati.
- Iconografia coerente.
- Stati vuoti curati.
- Errori e warning chiaramente distinguibili.

Confronta gli screenshot iniziali con quelli finali in condizioni equivalenti.

Se non puoi acquisire screenshot reali, dichiara il limite, indica quali verifiche alternative hai effettivamente effettuato e non inventare evidenze.

Non spacciare mockup statici per screenshot dell'applicazione funzionante.

---

# 11. Vincoli funzionali e di sicurezza

Il redesign riguarda la UI.

Non è autorizzato a modificare la logica normativa.

Preserva:

- Schema-driven architecture.
- Motore di validazione generico.
- Semantica degli errori.
- Gestione delle reference data.
- Completezza dei run.
- Risultati inconcludenti.
- Versionamento.
- Contratti di persistenza.
- Protezione dei dati.
- Audit.
- Comportamento delle operazioni realmente disponibili.

Non introdurre dati sanitari reali nei test o negli screenshot.

Non utilizzare fixture contenenti informazioni personali.

Non simulare capability future come se fossero già operative.

Quando una funzione non è supportata, deve risultare indisponibile in modo chiaro, oppure essere assente dalla UI secondo il comportamento approvato.

---

# 12. Quality Gate e test

Prima di completare il redesign:

- Verifica la compilazione di `lintra-ui`.
- Esegui formattazione e Clippy secondo il protocollo del repository.
- Esegui i test pertinenti.
- Verifica le integrazioni Rust/Slint modificate.
- Verifica le callback interessate.
- Verifica la navigazione.
- Verifica la gestione degli stati.
- Verifica i layout.
- Controlla eventuali warning o regressioni.
- Esegui i gate CI Linux/Windows applicabili.

Non dichiarare verifiche superate senza evidenze reali.

Il refactoring non deve degradare le prestazioni in modo significativo o introdurre nuove dipendenze native non motivate.

Se Codex sta eseguendo build e test nel medesimo workspace, evita compilazioni concorrenti che competano inutilmente per il lock della directory Cargo.

---

# 13. Gestione della documentazione precedente

I documenti di design preliminare devono essere utilizzati come fonti per:

- Requisiti funzionali.
- Navigazione prevista.
- Comportamenti approvati.
- Vincoli di prodotto.
- Terminologia.
- Flussi operativi.

Non devono imporre di conservare:

- Palette preliminari.
- Spaziature provvisorie.
- Impaginazioni basilari.
- Gerarchie visuali deboli.
- Componenti esteticamente immaturi.
- Scelte grafiche non più coerenti con il nuovo design system.

Il mandato è un **redesign autorizzato**.

Puoi modificare i documenti UI pertinenti alla direzione estetica, mantenendo traccia delle decisioni sostituite e senza alterare i requisiti funzionali.

Evita la proliferazione di documenti di design incompatibili.

Il design system finale deve diventare il riferimento visuale canonico per le successive task UI.

---

# 14. Governance e consumo risorse

Restano valide le direttive globali di Lintra.

**Vietato categoricamente:**
- Utilizzare subagenti.
- Utilizzare Superpowers per attività parallele.
- Delegare revisioni a più agenti.
- Avviare sessioni Codex concorrenti.
- Creare worktree per sviluppo parallelo autonomo.

Le tre skill devono lavorare in successione all'interno della stessa sessione gestita dal singolo agente.

Commit, push e CI sono autorizzati quando necessari.

Non richiedere conferma per normali decisioni stilistiche comprese in questa autorizzazione.

Fermati soltanto in presenza di criticità effettive, ad esempio:

- Skill obbligatoria realmente indisponibile.
- Conflitto con un contratto normativo.
- Necessità di cambiare un comportamento applicativo approvato.
- Incompatibilità tecnica rilevante.
- Necessità di introdurre una dipendenza non autorizzabile nel perimetro corrente.
- Impossibilità di preservare una funzionalità esistente.

Non trasformare preferenze estetiche ordinarie in un gate umano.

Non sacrificare la qualità visuale per terminare anticipatamente il lavoro.

Al tempo stesso, non consumare risorse in iterazioni illimitate senza nuovi problemi concreti da risolvere.

---

# 15. Report finale obbligatorio

Al termine produci un documento:

**LINTRA SIGNATURE UI OVERHAUL REPORT**

Il report deve contenere i seguenti elementi.

### A. Utilizzo verificato delle skill

Per `ui-ux-pro-max`:

- Conferma del percorso `.codex/skills/ui-ux-pro-max/SKILL.md`.
- Script realmente eseguiti.
- Risultati ottenuti.
- Design system definito.
- Direzione estetica scelta.

Per `impeccable`:

- Percorso della skill utilizzata.
- Procedure realmente consultate o eseguite.
- Problemi UX/visuali identificati.
- Refactoring realizzato.
- Miglioramenti derivati dal design system della Fase A.

Per `no-ai-slop`:

- Percorso della skill utilizzata.
- Stringhe analizzate.
- Testi modificati.
- Esempi prima/dopo.
- Elementi normativi preservati.

### B. Risultato estetico

Descrivi:

- Identità visiva finale.
- Palette.
- Tipografia.
- Componenti principali.
- Navigazione.
- Trattamento delle tabelle.
- Trattamento degli stati.
- Scelte distintive rispetto alla vecchia UI.

### C. Before/after

Produci screenshot comparabili delle schermate principali.

Spiega le differenze sostanziali, non soltanto quelle cromatiche.

### D. Qualità tecnica

Riporta:

- File modificati.
- Componenti creati o revisionati.
- Test effettivamente eseguiti.
- Build.
- Clippy.
- CI.
- Verifiche visuali.
- Eventuali regressioni corrette.
- Limiti rimasti.

### E. Valutazione critica finale

Esprimi una valutazione motivata del risultato rispetto a quattro dimensioni:

1. Identità estetica.
2. Professionalità e qualità esecutiva.
3. Efficienza per utenti esperti di flussi.
4. Estensibilità a nuovi flussi e nuove funzionalità.

Per ogni dimensione indica evidenze concrete e carenze residue.

Non utilizzare valutazioni autocelebrative o giudizi non supportati da evidenze visuali.

### F. Handoff

Aggiorna lo stato e l'handoff nel rispetto del workflow del repository.

Registra il nuovo design system come riferimento per le future task UI.

---

# 16. Definition of Done

Questa revisione è completata soltanto quando:

- Tutte e tre le skill sono state realmente utilizzate nell'ordine previsto.
- L'output di ciascuna fase è stato utilizzato dalla successiva.
- Il nuovo design system è documentato.
- La UI Slint è stata concretamente modificata.
- Il risultato presenta un'identità moderna, professionale e distintiva.
- Le principali schermate implementate condividono lo stesso linguaggio visivo.
- La UI continua a essere efficace per esperti di flussi.
- Il redesign non è vincolato a SIOC.
- La microcopy è stata revisionata.
- I contratti normativi e applicativi sono preservati.
- Le verifiche tecniche richieste sono state effettivamente superate.
- Il confronto visuale prima/dopo è disponibile, quando l'ambiente consente di produrlo.
- Non rimangono regressioni note incompatibili con i criteri di accettazione.

Un semplice restyling cromatico non soddisfa questi criteri.

Una UI genericamente minimale non soddisfa questi criteri.

Una UI esteticamente impressionante ma funzionalmente peggiorata non soddisfa questi criteri.

---

# ISTRUZIONE ESECUTIVA FINALE

**Procedi con il redesign integrale della UI di Lintra.**

Applica nell'ordine:

**UI UX Pro Max → Impeccable → No AI Slop → Revisione integrata.**

Parti dal codice e dalle schermate realmente esistenti, utilizza le skill installate, definisci una direzione visiva forte e implementala concretamente in Slint.

Tratta Lintra come un prodotto desktop di punta destinato a professionisti che lavorano quotidianamente con flussi dati complessi.

Non conservare scelte estetiche preliminari soltanto perché già documentate.

Non fermarti alla formulazione di un piano.

**L'obiettivo è consegnare una UI con una qualità visiva e un'identità di prodotto nettamente superiori all'implementazione attuale, senza compromettere precisione, efficienza e correttezza.**

Inizia dalla ricostruzione dello stato e dalla baseline visuale. Completa poi le tre fasi in sequenza, implementa il redesign e verifica il risultato.