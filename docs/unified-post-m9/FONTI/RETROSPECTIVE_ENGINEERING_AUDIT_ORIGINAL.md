# LINTRA — RETROSPECTIVE ENGINEERING AUDIT

## Mandato

Esegui una revisione tecnica retrospettiva del codice Lintra già sviluppato, utilizzando il livello di ragionamento High.

Gran parte del progetto è stata implementata utilizzando Codex Medium. Questo non costituisce di per sé un difetto e non autorizza refactoring arbitrari.

L'obiettivo è verificare, attraverso evidenze concrete, che l'implementazione rispetti realmente i contratti architetturali, normativi e transazionali ratificati.

**Non riscrivere il progetto. Non modificare codice corretto soltanto per migliorarne apparentemente lo stile.**

### Ricostruzione

Esamina lo stato reale del repository, le task completate, i relativi handoff, le decisioni ratificate e i Quality Gate disponibili.

Non assumere che una task dichiarata DONE sia necessariamente priva di difetti, ma non considerarla sospetta per il solo fatto che è stata sviluppata in Medium.

Concentrati sul codice effettivamente implementato.

### Priorità dell'audit

**Persistenza e SQLite**

Verifica atomicità, commit, rollback, errori ambigui, CAS, concorrenza cross-process, crash recovery, versionamento storage, isolamento namespace e protezione delle identità.

Riesamina in particolare il test di contesa modificato nel commit `4e44703`, senza assumere che rappresenti un bug o una correzione valida prima di aver esaminato il diff.

**SchemaCompiler e DSL**

Verifica compilazione delle regole, operatori, tipi, gestione delle condizioni, cardinalità dei join, determinismo, errori di schema e assenza di inferenze normative implicite.

**ValidationEngine e profili CAD/SIOC**

Verifica semantica delle violazioni, controlli cross-record, condizioni temporali implementate, identità storiche e distinzione fra regole effettivamente disponibili e capability future.

**Reference data**

Verifica SHA-256, identità dei dataset, boundary filesystem, disponibilità parziale, stati Verified/Degraded/Unavailable e distinzione fra evidenze definitive e provvisorie.

**Contratti trasversali**

Verifica propagazione degli errori, audit, gestione della memoria, assenza di dati sanitari nei log e coerenza delle API.

### Metodo

Esegui l'audit sequenzialmente, senza subagenti o Superpowers parallelo.

Per ciascuna area:

1. Individua gli invarianti e i contratti applicabili.
2. Esamina le implementazioni reali.
3. Esamina i test esistenti.
4. Cerca casi limite non coperti.
5. Identifica eventuali divergenze verificabili.
6. Classifica i risultati per severità.
7. Distingui bug dimostrati, rischi plausibili e semplici opportunità di miglioramento.

Non introdurre nuovi requisiti normativi.

Non reinterpretare le ADR approvate.

Non modificare i test per mascherare difetti del codice.

Non modificare il codice per soddisfare test che impongono garanzie inesistenti.

### Correzioni

Se trovi un bug reale e la soluzione è compatibile con i contratti già approvati, puoi correggerlo con test di regressione.

Se la soluzione richiede una nuova decisione normativa o architetturale, fermati sulla parte interessata e presenta una proposta motivata.

Evita refactoring ampi non necessari.

### Verifica

Esegui i test e i Quality Gate pertinenti.

Utilizza la CI Linux/Windows quando necessaria.

Non dichiarare verifiche superate senza evidenze.

### Report finale

Produci `LINTRA_RETROSPECTIVE_AUDIT_REPORT.md` con:

- Aree effettivamente esaminate.
- Contratti verificati.
- Evidenze nel codice.
- Test esistenti e nuovi.
- Bug confermati.
- Rischi non ancora dimostrati.
- Correzioni effettuate.
- Gate eseguiti.
- Criticità residue.
- Raccomandazioni ordinate per priorità.

**Non confondere assenza di bug individuati con dimostrazione di assenza di bug.**

### Governance

Rispetta le istruzioni globali single-agent.

Commit, push e CI sono autorizzati quando necessari.

Questa revisione ha priorità temporanea sulla prosecuzione ordinaria delle task. Dopo il completamento dell'audit e la risoluzione delle eventuali criticità bloccanti, riprendi automaticamente il normale workflow di Lintra.

## Principio conclusivo

Il progetto deve essere giudicato sulla correttezza verificabile della sua implementazione, non sul livello di ragionamento utilizzato durante la generazione del codice.

Esegui una revisione critica, indipendente e basata su evidenze. Evita sia conferme superficiali sia riscritture ingiustificate.
