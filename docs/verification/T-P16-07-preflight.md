# T-P16-07 — collaudo visivo e requisito diagnostici

Prerequisito16-05 realmente DONE, CI40668541764/release40668592173.06 implementata
su ff600d3+245bf34, gate40897488097 pendente: non è un prerequisito dichiarato07,
né viene considerata DONE. Preparazione07 sequenziale durante l'attesa remota,
nessun subagente o task parallela.08 attende gate07 e15-05.

Perimetro aggiuntivo esplicitamente richiesto dall'autore29/10: messaggi chiari,
dominio completo consultabile anche per range e spiegazione formato/allineamento/
riempimento. Non implica nuove regole fisiche, divieto di cifre in AN o modifica
CAD/SIOC/FROZEN. Contratto in docs/ui/DIAGNOSTIC_REQUIREMENTS.md.

File previsti: formatter/descrittore generico core e paging domini, integrazione
ErrorFactory, snapshot readonly dello schema effettivamente risolto nel reportUI,
dettaglio risultati e browser valori; test core/UI, visual.rs e zone/riferimenti
approvati. Nessuna nuova dipendenza prevista, nessun SQL nell'engine/UI.

Lo schema è già risolto da prepare con lo stesso registry del servizio: conservare
l'Arc di sola lettura, associandolo al hash effettivo del run, senza ricompilare o
usare la selezione corrente per descrivere errori di un run precedente. Nessun
nuovo campo obbligatorio in ValidationError/JSONL o modifica dei Parts pubblici.

L'elenco grande va letto a pagine, non copiato dentro ogni messaggio/errore o
materializzato integralmente nel modello UI. L'intervallo integer va percorso
senza cast limitante u64 e senza soglie DEC14 inventate. Etichettare dichiarazione,
rappresentazione fisica e regola effettivamente violata separatamente; nessun
esempio è un certificato di conformità a tutte le regole cliniche.

Per le reference esterne non inventare codici o descrizioni: usare i contratti
qualificati e la provenienza del run, con indisponibilità dichiarata quando il
dominio non è recuperabile/verificabile. Test positivi/negativi prima dei gate.

Visual: riferimenti approvati docs/ui/review/results-1x.png/2x.png invariati fino
ad aggiornamento esplicito. Geometria/bordi con confronto stretto, testo con zona
dichiarata, software renderer e IBM Plex sui due runner. Nessun allargamento di
tolleranza per nascondere deviazioni. Produrre risultati e capture reviewabili
prima di qualsiasi decisione umana realmente necessaria sui riferimenti.
