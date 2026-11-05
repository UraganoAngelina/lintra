# CNF-CAD-NAMESPACE-BINDING-01 — ratifica A estesa, 5 novembre 2026

Fonte: risposta integrale dell’autore del 5 novembre2026. Registro operativo,
non trascrizione verbatim. **ALTERNATIVA A RATIFICATA con estensioni vincolanti.**
RawOperand::Parameter/alternativaB non autorizzata. La norma preesistente, la
lacuna del contratto, l’estensione ora approvata e la sua implementazione
verificata sono stati distinti. Nessuna attribuzione retroattiva a vecchie release.

## Contratto e versionamento

Binding dichiarativo/generico/versionato/compilato, non legato a SQLite. Per CAD:
A.1.1 == request.namespace.sending_ulss; parametro stringa, dominio501..509,
comparison logical_text_exact. PK A resta(1.1,1.23); evento non globale fra ULSS.
501 e502 sono entrambi di dominio valido, ma mismatch se request501/record502.

Struttura semantica approvata: namespace_bindings con trace/field/parameter,
comparison, error_code E_NAMESPACE_VALUE_MISMATCH e message_template configurato.
Collocazione sotto persistence_policy ammissibile, da verificare contro modelli
reali; non rende il controllo dipendente dall’apertura del database. Eseguibile
anche nel normale percorso di validazione quando il parametro è disponibile
ed è richiesto dalla modalità. Non delegarlo al repository persistente.

Formato3 approvato se ancora libero: prima verificarne disponibilità nel repo.
Se già assegnato non sovrascrivere e registrare conflitto/successore con protocollo.
Capability persistence.namespace_binding.v1. Formati1/2 invariati, binding
rifiutato nei vecchi formati, nessuna migrazione o modifica FROZEN, nessuna
capability attribuita alla rc.2 precedente. Binding incluso in canonicalizzazione,
fingerprint e compatibilità. Valore concreto della request escluso dal fingerprint;
contratto dello schema e contesto variabile/audit del run restano distinti.

Compiler risolve trace/campo/parametro/tipi/comparison/codice/template/duplicazioni;
rifiuta riferimenti inesistenti, incompatibilità, duplicati/contraddizioni,
proprietà sconosciute e formato/capability errati. Piano con riferimenti risolti,
nessuna ricerca raw per nome nel loop e nessuna conversione implicita.

## Namespace autorevole e confronto

Un’unica request verificata produce il contesto tipizzato immutabile per binding,
scope join, sessione, lettura storico, preview/approvazione/commit e audit.
Non ammettere valori indipendenti501pervalidazione/502perstorage. Incompatibilità
fallisce prima di mutazioni. Identità del DB non prova appartenenza dei record.

logical_text_exact usa il valore logico secondo encoding/padding/missing policy
compilati. Vietati trim/case folding/locale/numero→testo impliciti, mutazione dei
byte o schema/literal sostituiti dopo fingerprint. Parametro stringa tipizzato
soggetto al dominio; tipo errato/fuori dominio è request invalida di preflight.

Binding in passata1 dopo i controlli locali necessari per confrontare il campo,
prima di proiezione; passata2 mantiene cross-record/cross-trace, niente terza
passata né cambi di classificazione normativa. Compatibile con futuro spike
ibrido DEC17, ma nessun parallelismo di produzione autorizzato.

## Esiti e modalità

Parametro obbligatorio mancante/incompatibile: errore tecnico preflight, nessun
falso mismatch di record e nessun fallback. Campo valido differente: ValidationError
E_NAMESPACE_VALUE_MISMATCH, CompletedInvalid, provenienza/conteggi/summary/ordine/
JSONL emax_errors ordinari. Campo già invalido: preservare violazioni primarie,
nessun mismatch derivato. Campo assente/nonvalutabile: presenza/validità compilate,
nessun confronto con placeholder e nessuna attestazione namespace verificato.

LocalOnly diagnostico isolato senza contesto organizzativo può validare struttura,
lunghezza/presenza/tipi/padding/domini/vincoli locali e reference applicabili.
Binding non valutato con distinzioni diagnostiche/completezza già esistenti;
nessun nuovo esito normativo o mismatch inventato, nessuna piena conformità CAD
per invioULSS. Se parametro presente esplicitamente, verificarlo ed eseguire
binding; presente-invalido non equivale ad assente.

Full con binding obbligatorio richiede parametro valido in preflight e controlla
tutti i record applicabili; mismatch normativo. Historical preview/approve_commit
usano lo stesso namespace verificato; mismatch impedisce approvazione/snapshot.
Non aggirare con LocalOnly senza namespace. Distinti report diagnostico, eventuale
preview non approvabile se supportata, approvazione e commit atomico. Validare non
muta lo storico.

A presente: ogni record applicabile confrontato. A presente-vuoto: nessun mismatch
su record inesistenti, ma nessuna prova positiva per appartenenza dei child.
A assente: niente mismatch inventato, rispettare missing trace/PROCEED-ABORT/join/
cardinalità, non inferire appartenenza dei figli. Nessun nuovo divieto generale
su file mancanti. Se percorsi preesistenti consentono preview/approvazione con
A assente, non introdurre false attestazioni: incompatibilità reali vanno risolte
esplicitamente, mai bypassate.

## Invarianti e test obbligatori

Nessun cambio PK/join/HMAC/keyprovider/storageformat/namespace persistente/
trimestri/immutabilità/CAS/audit. ContextStore effimero distinto dal repository.
Minimizzazione privacy e nessun identificativo superfluo in log/errori tecnici.
Niente ifCAD/SIOC/A/1.1/sending_ulss nei componenti generici; nomi da schema.

Test sintetici: match501/501, mismatch501/502, assenza/tipo numerico501/fuori
dominio999 per LocalOnly/Full/historical/approve; campo999giàinvalido senza cascade;
due ULSS stesso evento echild/join confinati, Aassente/presentevuoto/presentecon
record echildsenzaA; continuità anno/trimestre senza periodo nel namespace.
Formati1/2 invariati, formato nuovo/capability, rifiuto binding incompatibile,
FROZEN invariati, fingerprint deterministico che include binding ed esclude
requestvalue. SIOC e schemi senza binding invariati; ordine/conteggi/budget,
report reale, no mutazione su invalid e nessuna preview approvabile erroneamente.

## Mandato implementativo

T-CAD-02 autorizzata per raw/formato/capability/compiler, request tipizzata,
confronto generico passata1/diagnostica, fonte unica namespace validazione/storage,
loweringCAD e regressioni. Non autorizzati DSL Parameter generale, riscrittura
motore, nuove norme, retroFROZEN, storage/HMAC nuovi o parallelismo di produzione.

Aggiornare decision log/authoring/formato/capability/profilo/request/test/stato,
non riscrivere storia né chiamare FROZEN un profilo senza gate. Verifica preliminare
5nov: codice e compatibility ledger assegnano solo formati1/2,3 è disponibile;
ciò non dichiara già implementato/supported il formato3.

Single-agent, nessun subagente/Superpowers/delega; commit/push/CI e scelte ordinarie
nel contratto autorizzati, gate reali eprosecuzione READY. Stop solo per conflitti
reali irrisolvibili entro la ratifica. Registration01/release18/altre indipendenti
possono proseguire; nonDONEbinding prima dei gate pertinenti.
