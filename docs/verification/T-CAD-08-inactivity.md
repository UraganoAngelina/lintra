# T-CAD-08 — tre mesi calendario e sospensioni

IN_PROGRESS / gate locali qualificati, 8 novembre 2026.23prove nuove positive;
11gate finali exit0 con475impronte sorgenti invariate. Prerequisiti verificati nel preflight. Nessun gate
nativo08 attribuito e nessuna conformità CAD completa dichiarata.

## Contratto e confine della consegna

CAD-17 e G04 ratificati: ultima data C con almeno una D correlata su
(evento,data,tipo,figura), mai progressivi; se nessuna, inizio B1.2.4. Tre mesi
calendario con clamp, pause dei giorni strettamente interni unite e proroga a
punto fisso, valutazione a fine trimestre. D/altro input indispensabile assente
Unknown. Non imporre una seconda conclusione a evento già concluso legittimamente.

Non è stata introdotta una DSL per il periodo: il mapping anno/trimestre è già
in snapshot.period. Il compilatore dei profili restituisce metadati verificati,
non un flusso eseguibile o un token di approvazione. La qualifica del piano usa
compile_scoped privato e lo stesso motore a due passate chiamato dal servizio.
Gli ingressi pubblici ordinario e storico-audit restano fail-closed sui profili
snapshot combinati: nessuna policy viene rimossa per fingere eseguibilità.
La qualifica integrata del produttore storico/CAD completo resta T-CAD-12.
Non si modifica storage/HMAC/wire, non si persistono date/righe, non si aggiungono
API applicative/CLI/UI, dipendenze o nuove norme/FROZEN. Dati soltanto sintetici.

## Compilazione: dati e condizioni

InactivityWithPausesPlan già esisteva; riceve i due indici year_parameter e
quarter_parameter dal mapping snapshot dichiarato. CompiledAssertion aggiunge
la variante tipizzata con Box del piano, senza nuovo nodo raw o formato.

schema/inactivity::compile risolve owner e quattro join distinti dello stesso
owner. Richiede zero_or_many per tutti; consenso esplicito per le valutazioni,
nessuna riduzione per pause/accessi/prestazioni. Missing-trace policy dei join
identica al piano e Unknown: nessuna falsa assenza. Il consenso include
start/end/code, include_if const true: un filtro sulle sole righe concluse
nasconderebbe date iniziali discordanti. Date fisiche, campi esistenti, liste di
correlazione non vuote/uniche/di pari arità e tipi corrispondenti, numero mesi
positivo e limitato e codice non vuoto sono verificati prima di creare il piano.
Il core non riconosce nomi di trace, ID clinici o il literal12.

schema/temporal::compile verifica l'accordo capability/declarazioni per tutti
i tre piani e delega il terzo al nuovo modulo. Il compilatore del profilo ora
può qualificare i piani insieme a snapshot: questo ingresso non ha mai prodotto
una validazione o approvazione. I controlli separati sugli ingressi eseguibili
pubblici rimangono attivi e sono provati esplicitamente.

compile_scoped abbassa il piano nel controllo
`temporal.inactivity_with_pauses.<indice>` con when costante true e codice
schema. Riusa l'ordine esistente, i conflitti di ID e le primitive di seconda
passata. Il raccoglitore RuleDependencies include tutti i campi start/end/code,
estremi pausa, tuple e data accesso e i quattro join. Non proietta righe intere.
La capability snapshot richiede un piano snapshot effettivo nella policy
compilata: la sola presenza di una policy audit non la abilita. Le capability
compilate sono abilitate soltanto con policy compilata fornita al percorso interno; nessun ingresso pubblico prende policy forgiabili.

## Periodo e contesto

projected_context::session_horizon legge i valori Integer dagli indici compilati,
richiede anno1..9999 e trimestre1..4 e costruisce marzo31/giugno30/settembre30/
dicembre31. Più piani devono concordare sull'orizzonte. Nessun default, lookup
per nome o data dell'host. ProjectedEvaluationContext conserva il valore risolto,
verificato prima dei lookup. VerifiedRunContext richiede il periodo solo in Full;
LocalOnly può diagnosticare i record senza parametri cross. Anche l'ingresso
diretto della seconda passata verifica il periodo prima di scorrere record o
osservare cancellazione: input vuoti non aggirano il preflight.

## Valutatore e correttezza

engine/inactivity::reference crea riferimenti Local dalle sole posizioni già
compilate. bytes conserva la distinzione fra Present/Missing/Unavailable; date
interpreta il formato esplicito DDMMYYYY. tuple richiede ogni componente e
qualifica come date i campi di tipo Date: una data D invalida non può diventare
una tupla diversa che "dimostra" assenza di prestazioni. Le copie delle tuple
sono temporanee in memoria, mai registrate o persistite.

evaluate verifica disponibilità e coverage completa dei quattro insiemi.
Una fornitura presente vuota è evidenza completa; un input assente/parziale è
Unknown con causa del join. B1 deve fornire una tupla qualificata: QualifiedTarget
riusa il consenso esistente, senza una riga rappresentativa. Start mancante,
conflitti, date invalide o data/motivo mancanti solo da un lato danno Unknown.
Una conclusione datata prima dell'inizio è invalida. Data e motivo presenti
entro l'orizzonte soddisfano l'obbligo senza imporre un'altra chiusura. Una
conclusione futura non chiude retroattivamente la sessione. Non è stata inventata
una regola aggiuntiva che vieti conclusioni amministrative anticipate.

Un HashSet contiene soltanto le tuple D minime; la scansione C seleziona il
massimo delle date non future aventi una tupla D uguale. Senza match usa start.
Un accesso senza prestazione non ripristina l'attività; progressivi indipendenti
non fanno parte della tupla CAD. Le sospensioni vengono lette con estremi
espliciti, fine Missing aperta. Le date non interpretabili preservano Unknown.
La memoria aggiuntiva è O(tuple D + pause) per l'evento, senza rilettura input.

inactivity_calendar::date verifica calendario e dominio. due aggiunge mesi con
chrono::Months (clamp al fine mese), converte ogni pausa in intervallo inclusivo
di giorni interni, lo ritaglia dopo la base e all'orizzonte e unisce gli
intervalli di giorni ordinati. Fine dichiarata esclusa; Missing copre anche
l'orizzonte. Due estremi dichiarati coincidenti non creano un giorno sospeso.
Date invertite danno Unknown; pause vuote o fuori finestra non azzerano la base.

La scansione dei segmenti disgiunti calcola esattamente il punto fisso: se la
scadenza raggiunge il primo giorno sospeso, la lunghezza del segmento sposta
la scadenza oltre quel segmento. I segmenti successivi raggiunti dalla proroga
vengono consumati a loro volta. Nessun doppio conteggio o loop per giorno.
Scadenza <= orizzonte significa obbligo raggiunto, altrimenti nessuna violazione.

flow_validator associa True a soddisfatto, False al codice dello schema e
Unknown alla causa/diagnostica del join tramite indeterminate esistente. La
diagnostica non inventa un campo locale A per i valori B1/C/D: il controllo è
di evento e non allega righe/identificativi arbitrari. Sink, observer, aggregati,
limite errori e cancellazione fra record restano quelli comuni della passata2.

## Componente e prove

2024-06.inactivity.1 conserva25campi selezionati e PK complete dei cinque trace,
coordinate originali218/198/41/45/39, metadati/note e digest sorgenti. La data
2.4 viene abbassata esplicitamente a Date DDMMYYYY con calendario. Il filler
sintetico non afferma validità clinica delle colonne omesse. Non è un nuovo
schema CAD completo o un freeze. Codice errore già formalizzato:
E_CAD_MISSING_ADMINISTRATIVE_CLOSURE.

inactivity_tests::model compone lo schema di componente e filler esplicito;
raw imposta la sola codifica controllata; compile usa il profilo verificato e
compile_scoped privato. I costruttori a/evaluation/pause/access/provision
scrivono soltanto valori pubblici sintetici a coordinate originali. B1 aperto
usa motivo00 secondo MissingValuePolicy, non blank inventato. execute_budgeted
scrive file temporanei, esegue il reale motore/lettore/audit/summary/proiezioni,
con lo scope dei soli input effettivamente presenti e gli stessi port bounded
usati dal servizio. execute è il caso senza budget; expect controlla il verdetto
e vieta errori locali accidentali nelle prove positive/negative normative.
Observed registra gli esiti del controllo effettivo, non risultati simulati.

Prove: febbraio comune/bisestile,29–31 e mesi diversi da90giorni; quattro
trimestri e anni1/9999; pause duplicate/sovrapposte/tangenti/aperte, proroga che
raggiunge nuove pause e mancato reset. Riferimento indipendente giorno per
giorno su100scenari deterministici per l'algoritmo di unione/punto fisso.
Ultimo C con D, nessun accesso o C senza D, tuple differenti e progressivi
indipendenti; input assenti contro vuoti, date invalide in tutti gli insiemi,
B1 uguali/confliggenti, conclusioni legittime/future. Preflight parametri anche
senza input e con cancellazione, LocalOnly diagnostico, rinomina parametri/trace,
compiler fail-closed, cancellazione e limite errori con JSONL ripetibile dal
vero JsonlSink. Le firme/gli helper nuovi sono privati, eccetto la variante e
gli indici del modello compiled già previsto; nessuna nuova API applicativa.

Preliminari: corretti il motivo Missing del fixture e lo scope summary con soli
input forniti. Il red sul D con data invalida ha dimostrato una deduzione
impropria di assenza: corretto qualificando i campi Date delle tuple. Nessun
validator o comportamento Missing comune è stato indebolito. Log preliminari
conservati nel dossier; i risultati finali vanno attribuiti solo ai gate conclusi.

## Gate e handoff

Sequenza finale11comandi exit0: formatting, Clippyworkspaceallfeatures,
workspace1010pass/0fail/6ignored, workspaceallfeatures1010/0/6,
core senza bundle967/0/6, build release, fmt/Clippytooling,
tooling18/0/0 inclusiignored con LINTRA_TEST_BIN del vero binariorelease,
24Python e boundaryworkspace/pixel exit0.475impronte identiche prima/dopo e
ricontrollate prima della pubblicazione; indice eHEAD invariati durante i gate.
Log/conteggi verificati, inventario di25file nel dossier. La prima sequenza
fallita resta separata. Nessun test UI omesso nelle due suiteworkspace.

Da completare: pubblicazione sul branch autorizzato, CI Linux/MSVC e tre
confronti JSONL/pixel/due riapertureSQLite crossOS sul commit esatto.
08resta IN_PROGRESS/WAITING_GATE;137/140DONE invariato. Nessuna nuova task
indipendente READY:12attende08, P18-04attende tutte. Nessun nuovo blocco umano;
proseguire12 automaticamente soltanto dopo la qualifica08.

## Recovery del gate completo — 8 novembre 2026

Il primo workspace gate ha rilevato la regressione SIOC sul rifiuto della
capability snapshot dichiarata senza piano. Riprodotta isolatamente prima
della correzione: il nuovo controllo generico richiede policy.snapshot presente
anche nel lowering delle capability. Il compilatore audit pubblico continua a
rifiutare profili snapshot reali. Nessuna capability ignorata o test indebolito.
La sequenza fallita e il red isolato restano distinti dai gate finali.
