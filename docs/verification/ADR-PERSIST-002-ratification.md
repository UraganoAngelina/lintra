# ADR-PERSIST-002 — ratifica dell’autore,2026-10-28

Fonte: risposta esplicita dell’autore nella sessione di sviluppo del28ottobre.
Proposta A approvata con prescrizioni architetturali/operative vincolanti.
Questo registro integra la proposta; non riscrive le decisioni precedenti.

## Decisione e responsabilità

SQLite incorporato e statico è il primo backend LOCALE. Eccezione N-20 limitata
al motore C e dipendenze tecniche necessarie, tutte da qualificare con DEC-15.
PersistentContextRepository/PersistenceTransaction restano port generici senza
SQL, path SQLite o nomi SIOC/CAD. SQLitePersistenceAdapter gestisce connessione,
codec/schema/indici/transazioni/versioni/errori; i servizi scelgono le query e
le proiezioni, proteggono prima della scrittura e applicano le regole normative.
Il motore non interroga SQL. Un commit tecnico non approva un invio CAD.

## Quattro prescrizioni vincolanti

1. Impostare E LEGGERE effettivamente `journal_mode=DELETE` e
   `synchronous=EXTRA`, verificandoli su Linux/MSVC e filesystem supportati.
   Stage privato, CAS sotto transazione, snapshot/indici/audit coerenti, abort/
   drop senza pubblicazione. Distinguere errore prima del commit da esito
   ambiguo dopo possibile pubblicazione/perdita risposta; recovery verificabile,
   niente falso rollback. Idempotenza/lifecycle secondo i contratti esistenti,
   snapshot storici non riscritti. Nessun WAL/OFF o indebolimento per comodità;
   incompatibilità reali richiedono proposta equivalente prima di cambiare.
2. SQLite non cifra automaticamente. Solo identità pseudonimizzate/proiezioni
   minime/fingerprint/sessioni/namespace/provenienza/audit/hash autorizzati.
   Vietati record raw, CF/ID paziente grezzi, payload clinici completi, segreti
   nel DB e PHI in log/file/diagnostica. Usare HMAC04 e provider esterno;
   assenza/incompatibilità chiave è errore, mai nuovo storico silenzioso.
   Controlli root esplicita, Linux permessi, Windows ACL, namespace, traversal,
   symlink, concorrenza e redazione. Cifratura a riposo/retention/provisioning/
   backup sicuro restano decisioni da risolvere prima dei dati reali.
3. Versione storage distinta da software/schema normativo/reference. Stati
   tipizzati compatibile/non supportato/non inizializzato/corrotto. Mai
   sovrascrivere o reinizializzare uno storico sconosciuto, perdere snapshot o
   fare migrazioni distruttive implicite. Minimo formato05 autorizzato,
   migrazioni evolutive future rinviate. Dati letti validati nel dominio.
4. Indici e query puntuali parametrizzate, nessuna scansione globale per ogni
   PK, copia del DB in RAM o HashMap con tutti gli snapshot. Cache documentata,
   staging compatibile con i port. load_snapshot può materializzare O(n):
   riconoscere il limite e indicare futura API streaming senza rompere i port.
   Misure sintetiche crescenti fino a centinaia di migliaia su lookup, picco
   memoria, commit, riapertura/restart, indici e contesa; niente soglie DEC-14
   inventate o fake03 usato come prova del backend reale.

## Ambito e distribuzione

Un archivio distinto per installazione; nessun Git-sync, SMB/NFS automatico,
server, merge o storico organizzativo implicitamente condiviso. L'assenza di
un archivio locale non prova assenza di eventi nell'intera organizzazione.
Utenti senza installazione separata SQLite/Git/Rust/Cargo. Qualificare binding,
versione/feature/amalgamation/licenze/checksum/provenance/C compiler/linking
Linux e MSVC e autosufficienza artefatti. Staticità SQLite non elimina i normali
DLL/ABI OS: preservare i gate, segnalare conflitti specifici senza aggirarli.

## Accettazione05

Roundtrip/process restart/snapshot immutabili e identità omesse conservate;
commit/abort/drop/errori/CAS obsoleto/contesa tra processi/interruzione/recovery/
esito ambiguo; codec senza raw/secret, namespace/root/permessi/traversal/symlink/
errori redatti; nuovo/compatibile/unsupported/corrotto senza reset distruttivo;
indici/scalabilità/memoria reali; build e linking Linux/MSVC, CI e gate effettivi.
Esclusivamente fixture sintetiche. Nessuna dichiarazione PASS senza evidenza.

## Confini delle task successive e governance

Non anticipare backup/restore completo, lifecycle approvazioni, servizio
storico, policy normative, UI, migrazioni ulteriori o backend condiviso.
Registrare dipendenze qualificate e decisioni ancora aperte. Nessuna norma
CAD/SIOC o baseline modificata. Ratifica autorizza l'implementazione05;
nessun nuovo stop umano ordinario a fine05. Single-agent, nessun Superpowers,
subagente/background; commit/push/CI autorizzati, gate e handoff obbligatori,
continuare alla successiva READY senza criticità. Fermare solo conflitti reali.
