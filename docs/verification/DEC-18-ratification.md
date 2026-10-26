# DEC-18 — Ratifica definitiva,2026-10-26

Fonte: approvazione esplicita dell'autore in questa sessione, con precisazioni
vincolanti. La proposta ../proposals/DEC-18-schema-reference-catalog.md è
approvata salvo le precisazioni seguenti. DEC-18 risolta; T-P14-05 autorizzata.
Nessuna modifica retroattiva di ARCH/DET/Master Plan/baseline/grammatica.

- SchemaCatalog separato da SchemaRepository::load_raw, filesystem/in-memory,
  listing senza compiler: disponibile non significa eseguibile o completo.
- Layout root/flow/version/flow.json, root esplicita, vuota ammessa, assente/
  non leggibile errore; niente discovery implicita/latest/casefold/conversioni
  authoring. Ordine flow/version UTF-8 e trace dichiarati non vuoti/univoci.
  Ordinare candidati prima dei controlli; errore pertinente mai omesso.
  Rifiutare duplicati JSON, mismatch identità e symlink nei livelli/candidati.
  Nessuna promessa contro mutazioni ostili concorrenti non realmente garantita.
- schema list JSON soltanto dopo verifiche complete, nessuna certificazione.
- reference-data list: soltanto dipendenze dello schema esplicitamente scelto,
  API minima tipizzata con lowering condiviso, niente compile globale obbligatoria.
  Duplicati/invalidi/metadata unsupported rifiutati; nessun manifest/provider fittizio.
- ContinueWithAvailableEvidence esplicita per inventory; preservare distinzione
  fra leggibilità, formato, digest, identità/versione e attendibilità. Verified
  richiede tutte le garanzie; Degraded consultabile non è normativamente verificato;
  Unavailable non è set vuoto. Nessun recupero permissivo di file malformati.
- Root invalide/non leggibili, traversal/boundary/symlink escape e dichiarazioni
  strutturalmente invalide fatali. Dipendenze mancanti/degradate/malformate
  classificabili non impediscono l'inventario delle altre. Exit0 non implica
  Verified, Full eseguibile o futuro successo Strict. Policy dei run invariate.
- JSON: selettore schema, declaration ID, dataset ID/versione, path relativo,
  policy, availability, digest atteso/effettivo opzionale, cause tipizzate;
  ordine deterministico ID/versione/parità stabile, nessuna chiave dataset.
- schema import: input XLSX/blueprint/profilo obbligatori, alias opzionali,
  importer e normalizzatore reali, lint, raw draft canonical; Error blocca
  output, Warning JSON stderr completo con severity/codice/coordinate.
  Output nuovo atomico senza overwrite/collisioni con input, input preservati.
  Note/provenienza/hash/cached formulas invariati, nessuna norma inferita.
- schema lint espone warning anche su compile riuscita, senza duplicazioni,
  cambio severity o bypass Registry. Distinti successo/warning/fallimento.
- Tooling exit0 successo/1 errore tecnico-authoring;2 solo violazioni dei record.
  Listing non scrive risultati prima degli errori noti; I/O stdout propagata,
  nessuna promessa di atomicità stdout. Import conserva atomicità del file.
- Test obbligatori catalogo (vuoto/mancante/permessi/ordine/parità/mismatch/
  duplicati/JSON malformato/symlink/file estranei), inventory (tutti gli stati,
  assenza/hash/formato/identità/digest/cause/exit/boundary/no chiavi), import
  (XLSX reale positivo/negativo/profilo/alias/warning/error/no overwrite/
  collisioni/atomicità/provenienza), regressioni load/provider/core senza clap.
- Gate locali e CI Linux/MSVC effettivi prima di DONE/M7, evidenze/stato/handoff
  obbligatori. Nessun P15/embedded/flusso nuovo/persistenza/UI/refactor estraneo.
- STOP al termine di T-P14-05, attendere nuova autorizzazione.

I contratti storici CNF-REFERENCE-FORMAT-01 e CNF-COMPILER-01 restano validi.
La ratifica autorizza l'implementazione, non costituisce evidenza di test o CI.
