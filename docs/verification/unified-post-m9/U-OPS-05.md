# U-OPS-05 — Chiave comune, custodia e recupero

Data: 2026-11-10, Europe/Rome. Prerequisito U-OPS-01 `c42d44e`.
Stato: DONE per contratto/runbook e drill sui provider esistenti invariati.
G-OPS1 aggregato NOT_VERIFIED; provisioning GUI/account resta U-OPS-03/MIG/CAD.

## Reuse verificato, nessuna chiave nuova

FilePersistenceKeyProvider mantiene file binario **esattamente32byte**, Key ID
stabile, verifiche OS/link/replacement e redazione/zeroize. Nessun cambio HMAC1,
codec, fallback, nuova cifratura, rotazione o key-generation operativo.
I test usano solo costanti pubbliche sintetiche e directory private temporanee;
non è stata aperta o modificata alcuna chiave/archivio dell'autore.

## Matrice di drill effettivi

| Caso | Prova / risultato |
|---|---|
| File perso, vuoto,31/33byte o non regolare | key_file::absent_empty_short_long_and_non_regular_files_are_never_defaults; errore, nessun file sostitutivo creato |
| Recupero/relocation della stessa chiave e ID | portable_resolution..., audit protezione identico; la vecchia posizione mancante fallisce senza fallback |
| Copia32byte corretta da altro bundle | shared_key_produces_scoped_identities_but_archives_remain_independent: riapre il vero SQLite originale con il provider della copia e carica lo stesso snapshot |
| Copia32byte errata, stesso Key ID | stesso test: KeyMismatch, byte del DB originale invariati |
| Altro Key ID/stesso materiale o provider non disponibile | persistence_protection: preflight esplicito fallisce, nessuna migrazione implicita |
| Restore chiuso con storico, chiave errata o DB danneggiato | persistence_qualification: due revisioni esatte, isolamento dell'originale, errore su danno/chiave errata; nessun archivio vuoto sostitutivo |
| Sentinelle chiave/record in Debug/audit/log | protezione e CLI reali: sentinelle assenti negli output, controllo positivo dello scanner presente |
| Compromissione della chiave corretta | Tabletop sintetico: la stessa chiave in due installazioni rende confrontabili HMAC nello stesso scope ma non sincronizza DB; sospendere lo storico e preservare le copie, nessuna rotazione automatica. Non è rilevazione automatica di furto/exfiltrazione |

Runbook attuativo: docs/KEY_CUSTODY_RECOVERY.md; aggiornata la parte discordante
di PORTABLE_KEY_BUNDLE rispetto a D03/D05/D07 già ratificate. Contiene stop su
perdita/copia errata, recupero originale con accessi corretti e key-confirmation,
continuità negli update e risposta manuale a compromissione senza nuovo crypto.
history check non viene presentato come inventario completo o prova di backup.

## Evidenze nuove e limiti

Su **556c5c6**,447input invariati:
U-OPS-05-key-file-linux JSON/log.gz:5pass;
U-OPS-05-protection-recovery-linux JSON/log.gz: CLI2 + protection9 + qualification2
=13pass. Totale18pass, zero fail/ignored. Comandi, SHA e digest nei JSON.
Il codice del provider/crypto/storage non cambia: la CI nativa già qualificata
di questi percorsi resta evidenza storica del suo SHA, non una prova inventata
di nuovo codice. Il delta della task è solo documentazione e nuove esecuzioni
di test esistenti; non richiede pipeline duplicata o nuova autorizzazione D06.
Non sono prove Windows11 due-account o di una UI Tauri ancora inesistente.

File: runbook, PORTABLE_KEY_BUNDLE, dossier/JSON/log e stato. SHA nella cronologia
Git. Next READY indipendente U-OPS-07; baseline MIG attende CI42834962873.
