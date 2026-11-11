# U-OPS-03 — Preflight accessi e continuità

Data11 novembre2026. Stato: **WAITING_INTEGRATION**, non DONE.
Prerequisito U-OPS-02 DONE. Preflight su42620cf, working tree pulita prima
delle sole evidenze di qualifica. Nessun nuovo crypto/password custom.

Il backend già possiede i controlli necessari: file_security::root crea una
root privata esplicita e la apre senza riparare permessi; Unix0700/0600,
owner corrente, hardlink/symlink e filesystem qualificati; Windows DACL protetta
user/System, root owner corrente e rifiuto di reparse. RootGuard mantiene
l'handle; file/key loader verifica il file aperto e redige errori senza dati.
SQLite open non inizializza dopo errore; create è esplicito. Nessun chmod/ACL
repair di dati esistenti. Gli snapshot e la chiave originale restano invariati.

Fonti lette: src/persistence/file_security.rs e windows.rs, key_file.rs,
sqlite/mod.rs e test di permessi/riapertura. Le suite complete cb70bee sono
qualificate nella CI42990775714; ciò prova i controlli correnti, non un path
Tauri che non è ancora chiamato. Locale1096pass/7ignored, uid della sessione
Linux1000. Non sono stati creati account o letti file/chiavi operativi.

Resta da collegare e collaudare il provisioning reale di
%LOCALAPPDATA%/Lintra e delle sue directory private al Job Manager Tauri,
con selezione OS del percorso, spool/XLSX/cache e continuità di DB/report.
Per Linux solo personale valgono XDG_DATA_HOME e fallback documentato.
Il prodotto attuale espone archive/key path espliciti e Slint portabile:
non li si riscrive automaticamente e non si aggiunge una API inutilizzata
solo per dichiarare completata la task. Il frontend Tauri assente è il
prerequisito materiale d'integrazione; il relativo perimetro è previsto in MIG.

Nessun gate G-OPS1 o test Windows11 non-admin/due account è dichiarato PASS.
La task riprenderà quando esiste il vero adapter/worker; le task MIG READY
proseguono ora senza dipendere da questo residuo. Nessun nuovo arco o ID
inventato, nessuna ratifica D01–D15 riproposta. Stato e dossier conservano
il limite, senza bloccare GUI sintetica o qualifica backend indipendente.
