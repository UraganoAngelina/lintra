# T-CAD-06 — preflight, 7 novembre 2026

READY: CAD02/P10/P13/G0 qualificati nel registro. CAD04 sospesa sulla sola
attesa CI42368062138, senza dipendenza06. Workflow single-agent sequenziale.

Fonti lette: prompt06, mappa/indice, ARCH45, DET166–174, authoring437ss,
CAD_HISTORICAL_PROFILE4, CAD-14 ratificata, note3.2/3.3/3.4/4.7, fixture16.
RawTemporal::IntervalExclusion e IntervalExclusionPlan già formalizzati, ma il
compiler rifiuta ancora temporal_rules. Il join set e le proiezioni/coverage
esistono; nessuna selezione arbitraria di un B2 è ammessa.

Implementazione prevista: lowering verificato del solo interval_exclusion in
un'asserzione compilata interna alla seconda passata esistente; dipendenze
di proiezione complete e summary/observer/error budget esistenti. Il compilatore
abilita esclusivamente profili di lettura senza snapshot/audit/invarianti; non
elimina obblighi dal profilo CAD completo. After-date/inattività restano rifiutati.

Ogni B2 collegato contribuisce indipendentemente dal motivo. Interni stretti,
fine mancante aperta; nessun match su contesto completo equivale a nessuna
sospensione, file assente/coverage parziale rimangono Unknown se non esiste
un controesempio certo. Date invalide non diventano date mancanti. Nessun
driver storico o formato persistente nuovo. Integrazione cumulativa CAD12 resta
distinta; la fixture di componente non attesta il profilo clinico CAD completo.

File previsti: schema temporal/compiler/projection/dependencies/plans; engine
seconda passata e diagnostica; componente sospensioni versionato, test di servizio
sintetici e documentazione. Verificare confini/aperti/sovrapposti/motivi/namespace,
input assenti e vuoti, date invalide/coverage, ordine/cancellazione/budget,
rinomina generica e guardie compiler. Nessun nuovo crate, norma o DSL raw.
