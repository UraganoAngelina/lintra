# U-FIX-04 — Permessi della reference tree

Data: 2026-11-10, Europe/Rome. Prerequisito U-GOV-03 `2cd4bdf`.
Stato: WAITING_GATE. Controlli locali finali PASS sul commit
`76fa08a11409f5d6cd67b97f025b6686d2c6902c`; CI nativa ancora necessaria.
G-HARD NOT_VERIFIED.

## Delta

tree_security.rs verifica senza mutare file/ACL: Unix owner corrente/root e
nessuna scrittura gruppo/altri; Windows owner corrente/System/Administrators,
nessun diritto di mutazione per altri soggetti in una DACL interpretabile.
Read/execute pubblici ammessi. Nessuna policy aziendale di gruppo inventata.
GetNamedSecurityInfo, token SID e GetAce usano le API Windows già nel lock;
buffer e descriptor possiedono le rispettive allocazioni fino alla fine della
verifica, con rilascio RAII di LocalAlloc/handle. Nessuna dipendenza nuova.

FileSystemReferenceDataProvider controlla root e componenti effettivamente usati
prima di fs::read; ritorna UntrustedTree con causa distinta, in entrambe le
policy. Riusa i metadata del path checker per non trasformare l'I/O ordinario
già classificato in nuove eccezioni. Il preflight budget riusa il controllo:
«Procedi comunque» non aggira permessi non attendibili. SHA/identità/dataset
missing, Verified/Degraded/Unavailable e snapshot immutabili restano invariati.

Il modello resta una tree locale controllata, non un filesystem ostile durante
la risoluzione. Root esplicita/canonicalizzazione e confini legacy non sono
spacciati per open atomico per handle. Nessuna modifica allo storage SQLite,
HMAC, chiavi, codec, CSV o golden. Documentazione: docs/REFERENCE_TREE.md.

## Prove preliminari effettive

Linux: reference_filesystem 24 pass, reference_service 16 pass, resource_budget
10 pass. Nuovi casi negativi root/directory/file con scrittura di gruppo,
entrambi Strict/Continue: errore, nessun chmod automatico; ripristinando soltanto
la fixture il dataset si carica. Caso positivo file0444 condiviso in sola
lettura: Verified con i byte reali. Symlink e Unknown preesistenti ancora verdi.

Windows: aggiunto test nativo su file sintetico e ACL reali, broad write
rifiutato e broad read consentito, nessuna riparazione automatica. **Non ancora
eseguito nativamente.** Check Windows-GNU prodotto tentato: fallito per assenza
di x86_64-w64-mingw32-gcc richiesto da SQLite bundled, non prova di errore Rust.
Type-check isolato del medesimo file e dei test, con windows-sys/tempfile pin
esistenti, completato offline in /tmp/lintra-reference-acl-check: PASS in2,55s.
Non è build del prodotto, né esecuzione Windows/MSVC; nessun gate nativo PASS.

Le prove finali e gli hash del commit verranno acquisiti separatamente. Non
attribuire CI42829144043 (codice budget precedente) a questo nuovo delta.
Next: commit e controlli mirati; poi attesa qualifica nativa, task indipendenti.

## Qualifica locale sul commit

U-FIX-04-reference-linux.json/log.gz: **89 pass, zero fail, zero ignored**
in otto suite (reference filesystem/preparation/service, embedded, Full, golden,
SIOC referential, budget). U-FIX-04-clippy e U-FIX-04-fmt: PASS, workspace
all-targets/all-features con -Dwarnings. In tutti i casi i 447 input inventariati
sono invariati prima/dopo, HEAD sempre76fa08a.
U-FIX-04-windows-module-typecheck: PASS sullo stesso modulo; manifest/harness/lock
e limite del tentativo prodotto conservati in windows-check-context.json.
Non sostituisce la prova nativa Windows né installa MinGW o nuove dipendenze.

Task in WAITING_GATE; avviare CI del nuovo delta dopo la run U-FIX-03 già in
corso, senza attribuirne l'esito a questo codice. Proseguire con U-OPS-01
indipendente, poi U-NORM-01/02/05. Nessun gate tecnico non provato è PASS.
