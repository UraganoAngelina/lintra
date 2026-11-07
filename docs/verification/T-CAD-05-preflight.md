# T-CAD-05 — preflight, 7 novembre 2026

READY: prompt05 richiede03/PERSIST10, entrambeDONE con evidenze reali.
03qualificata su6af5f85/CI42203885693; PERSIST10su c884692/CI41822083629,
settejob positivi ciascuna. CAD04giàDONE usata come ulteriore base qualificata
per verificare i diff sui progressivi. I riferimenti04/PERSIST09 scritti nei
precedenti handoff come prerequisiti05 erano imprecisi: prevalgono prompt
03/PERSIST10 verificati qui, nessuna dipendenza pendente è stata aggirata.

CAD07sospesa sulla solaCI42390593313, SHA6477bb2 esatto e prima lettura
in_progress. Un timeout della revisione automatica della letturaGitHub è stato
risolto con l'unico retry autorizzato; nessun gate dedotto o pipeline duplicata.
Single-agent, no background/subagenti; docs/migration preesistente esclusa.

Fonti lette: prompt05, ARCH45/DET166–174/specpersistenza, piano14/20,
CAD_HISTORICAL_PROFILE3, fixture15/19, componente namespace/SESSIONS e
progressives qualificati, PERSIST10provider/APIclient/ratifica portabile.

Il core già consegna TracePresence Present/Absent, warning schema, consenso
PROCEED tipizzato, ABORT per consumo/cancel della preparazione, fonteprotetta
e inventario/consenso persistiti. Il confronto seleziona l'ultima baseline
PRESENT: l'omissione di un file non cancella progressivi. Present(0) rimane
PRESENT e può invece annullare i precedenti record del file completo.

Non serve un nuovo executor o un'interfacciafinta. Scope05: qualificare la
composizione CAD dei port esistenti con test di accettazione aggiunti ai fixture
progressivi04, usando A/B1/C/D senza B2, warning prima della review, mancanza/
copertura errata di consenso, ABORT/PROCEED, restartSQLite, HMAC della scelta,
assenza contro filevuoto e providerfile portabile reale su materiale pubblico
sintetico. Nessuna chiave operativa, norma, SQL/HMAC/wire o API nuova.

Gli outcome sono fixture di componente esplicite, come03/04: non attestano
validazione clinica Full. I componenti qualificati non vengono spogliati di
policy per fingere un profilo CAD completo. Integrazione produttore/Slint e
storia cumulativa/temporalità integrata restano ai gate pertinenti/CAD12;
PERSIST10ha espressamente escluso la schermata storicaSlint concreta.

File previsti: tests/cad_progressives.rs (riuso dei helper a coordinate originali),
documentazione componente/dossier. Codiceproduttivo invariato salvo bugordinario
riprodotto dai test. Gate locali pertinenti e CIpropria primaDONE05.
