# T-P11-05 — Preflight dei contratti reference

2026-10-23. Attività preparatoria: avvio implementativo dopo verifica nativa
T-P11-04. Nessun cambiamento a grammar v1/v2 o fonti architetturali.

La ratifica aggiornata CNF-REFERENCE-FORMAT-01 supera Result<(),FieldViolation>
per le membership non definitive; il bool del LookupSet resta una misura locale,
mai sufficiente da solo per inferire un'assenza normativa da sorgenti parziali.

Estensione minima prevista: esito tipizzato con VerifiedPresent,
VerifiedAbsent, ProvisionalPresent e Inconclusive; composizione di snapshot
preparati senza copie degli insiemi o I/O, conservando evidenza delle sorgenti.
Il compiler risolve dataset_indexes sulle dichiarazioni ordinate, rifiuta ref
non dichiarate con MissingReferenceDatasetDeclaration, controlla binding/hash,
identità/versione e metadati non supportati. I fallback espliciti dell'AST
rimangono una fonte dichiarativa separata dal dataset: non inventare una
sorgente Verified se la presenza è dimostrata dal solo fallback dello schema.

L'API legacy RecordValidator::validate_local non deve restituire valid=true
come conclusione definitiva per un nuovo controllo non conclusivo. Conservare
le firme esistenti e aggiungere un percorso esplicito con artefatti reference;
l'API priva di contesto reference continua a rifiutare obblighi non eseguibili.
I nuovi artefatti distinguono violazioni e valutazioni non definitive. Non
modificare preventivamente summary/RunMetadata/RunStatus in questa task.

Regole locali: Provisional/Inconclusive non diventano False né successi
verificati. Continuare gli altri controlli; unsupported DSL/schema malformato
rimangono errori tecnici. Conservare la provenienza anche quando un'altra
assertion determinate dimostra una violazione reale. Warning e contatori
aggregati persistenti restano integrazione T-P11-06 secondo ledger.

Verifiche richieste: intera matrice Union della ratifica; fallback distinto;
compiler ref/dipendenze/metadati/hash invalidi; record e regole continuano su
sorgenti parziali senza falsi ValidationError; errori indipendenti conservati;
API legacy non pubblica conformità fittizia; tutti i gate locali e nativi.
