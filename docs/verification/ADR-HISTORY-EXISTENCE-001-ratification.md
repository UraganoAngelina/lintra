# ADR-HISTORY-EXISTENCE-001 — ratifica A condizionata, 8 novembre 2026

Fonte: risposta esplicita dell'autore nella sessione dell'8 novembre 2026,
«Ratifico ADR-HISTORY-EXISTENCE-001, alternativa A, con tutte le condizioni riportate».
Il messaggio integrale dell'autore prevale su questa registrazione operativa.

## Limite immediato vincolante

Autorizzata esclusivamente la specifica attuativa verificabile, analisi dei
contratti/piani/modello/copertura/provenienza/lifecycle/codec proposto/compatibilità,
privacy/transazioni e matrice dei test. Nessuna modifica immediata a codecSQLite,
schema storage attivo, scrittura di fatti, crypto/keyprovider, runtime storico,
compiler fail-closed, migrazione, CADcompleto/FROZEN, release/M9.
Implementazione persistente richiede successiva autorizzazione e propria qualifica.
Non interpretare questa ratifica come autorizzazione implicita al sottosistema.

## Semantica ratificata

DEC-05 rimane invariata: B1.2.8=1 almeno una volta nel perimetro storico approvato
qualificato dell'evento+invio corrente, non un'iniziale per trimestre, non esattamente
una. Namespace/flow/trace/correlazione dal profilo; nessun ID CAD nel core.
Piano generico da history_including_current/set_exists/predicato locale chiuso;
sottoinsieme minimo verificabile, predicati non supportati rifiutati prima dello storage.

Separare fatto positivo, copertura e provenienza autenticabili. Non basta una mappa
chiave→bool. True con prova autonoma affidabile anche se altre zone incomplete;
False solo senza prove e con universo sufficiente interamente qualificato;
Unknown per insufficienza, legacy non indicizzato, dati/tracce mancanti o incompatibilità
semantica. Corruzione/I/O/chiave/codec obbligatorio/transazione sono errori tecnici,
non False e non Unknown generico. Una regola obbligatoriaUnknown non attestaComplete.

La specifica deve definire inizio/continuità/perimetro normativo, sessioni mancanti,
parzialità/Present/Absent/cumulatività e ancoraggio dell'evento. Primo trimestreDB
non prova l'inizio storico; leggere tutte le sessioniDB non prova copertura normativa.
Senza base affidabile preservareUnknown. Staging corrente partecipa senza commit;
solo storico approvato compatibile, mai preview/altre sessioni non approvate.

Specificare insert/update/annullamento/reinvio/riattivazione/replay/snapshotimmutabili.
Non mantenere un fatto invalidato né cancellarlo solo perché il corrente ha rivalutazioni.
Se i contratti non decidono l'effetto di un'iniziale successivamente annullata sul
predicato storico, registrare conflitto e richiedere scelta esplicita prima del codice.
Non introdurre nuove norme di storicizzazione o annullamento.

## Prova, protezione e transazioni

Origine dalla stessa riga PASS1, validità locale dei soli operandi necessari,
Missing/Unavailable distinti, proiezioni minime e audit reali. Un errore locale
non pertinente non annulla automaticamente evidenze indipendenti; operandi invalidi
non provano True/False. Identità del predicato comprende AST/operandipresenza/scope/
join/versioni semanticamente rilevanti con canonicalizzazione/fingerprint esistenti.
Nome regola non basta; compatibilità fra versioni solo esplicita e dimostrata.

Provenienza minima: namespace/flow/trace/identitàprotetta/predicato/schema/sessione/
snapshot/approvazione/compatibilità/formato/keyIDversion quando rilevante.
Dati protetti, minacce, verifica integrità e fallimento vanno descritti tecnicamente.
SQLite non autentica metadati. HMACidentità non autentica automaticamente ogni
altro dato; non riusare il segreto per un protocollo nuovo. Nuova costruzione,
derivazione o usochiave richiede decisione separata prima di dichiarare garanzie.
Preservare filesystem/ACL/chiavi/backup; nessuna PHI o riga clinica in chiaro.

Fatti/copertura/provenienza pubblicati nella stessa unità validate→preflightstorico→
preview→eventualeconsenso→approve_commit→commitatomico→snapshotattivo. Nessun effetto
con errore report/sink, annullamento, crash/rollback/CASfallito, preview non approvata,
scrittura incompleta o commit incerto. Specificare recovery/idempotenza/reopen.

## Specifica richiesta e handoff

Definire formato/versione reale da registro, predicato/chiavi/copertura/provenienza/
indici/collegamento sessione-snapshot/compatibilità/letture-update/legacy/rollback.
Nessun codec attivo, migrazione tacita, reinterpretazione o Complete per default.
Nessun fatto rigenerato da fingerprint ciechi; ricostruzione da fonti originali
richiederebbe approvazione separata. CAD12nonDONE finché storico non eseguibile e
qualificato; componenti con filler non sonoCAD83completo. CNFordine separato.

Matrice minima: positivi storici/correnti, assenza completa/incompleta, più iniziali,
namespace, anni, correzioni/annullamenti, Present/Absent/PROCEED/ABORT/Partial,
compatibilità schema/predicato/formato, reopen, atomicità/CAS/report/cancel/crash,
privacy e Linux/Windows. Tutti pianificati distinti da test realmente eseguiti.
Handoff finale distingue decisioni chiuse, domande materiali, stato codice, modello,
query/verdetti, lifecycle/versionamento/integrità/transazioni, test futuri, modifiche
future, task bloccate e indipendentiREADY. Single-agent/no delega, gate e push
pertinenti autorizzati; nessuna ulteriore conferma ordinaria sulle scelte ratificate.
