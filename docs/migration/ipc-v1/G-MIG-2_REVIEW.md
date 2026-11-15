# G-MIG-2 — Pacchetto di design da ratificare

15 novembre2026. **READY_FOR_REVIEW, NON APPROVED**. La ratifica del14 novembre
è limitata aP2-02 e non viene estesa tacitamente al gate. P2-01..09 DONE nel
perimetro di design; P2-11 pubblicata e verificata, in attesa della ratifica
che è parte della sua acceptance. P2-10 si chiude solo dopo l’approvazione.

## Contratti sottoposti al gate

| Ambito | Fonte | Evidenza corrente |
|---|---|---|
| Ownership Rust/renderer | [Processo](../TAURI_PROCESS_OWNERSHIP.md) | ADR ratificati, API/callers e confine dipendenze |
| Wire v1 | [Envelope](CONTRACT.md) | Ratifica autore acquisita,84vettori PASS |
| Run e cancellazione | [Lifecycle](LIFECYCLE.md) |53casi PASS; terminale dopo sink/join |
| Progressi bounded | [Progressi](PROGRESS.md) |49casi PASS;10k update, latest+lease, control separato |
| Query/filtri/offset | [Paging](PAGING.md) |70casi PASS;128righe,2pagine, ordine e scope |
| Dettaglio e reference | [Dettaglio](DETAIL.md) |58casi PASS; testo16KiB, contesto/domìni64, stati espliciti |
| Export | [Export](EXPORT.md) |60casi PASS; path/conferme nativi, receipt per-file |
| Privacy/permission | [Sicurezza](SECURITY.md) | Threat model ADR-MIG-003 approvato; ACL generate auditate |
| Oracoli parity | [Harness](PARITY.md) |14scenari/64file congelati; comparatore e negativi PASS |
| Full multi-tracciato | [Full](FULL.md) | Ownership/DTO e18casi; wire/anchor/lifecycle verificati |

Le receipt con SHA e comandi/esiti sono nei dossier T-MIG-P2-* sotto
docs/verification/unified-post-m9. Reference eseguibili solo standard library,
collegate tra loro; **nessun bridge Rust/Tauri di prodotto è implementato**.
Il gate congela il design per consentire scaffold P3 e implementazione nei
task previsti. Non è una qualifica nativa/di rilascio o parity del nuovo bridge.

## Decisione concreta

Approvare il pacchetto LocalOnly+Full con: JSON UTF-8 raw Tauri e schemaVersion1;
1MiB intero messaggio in entrambe le direzioni e profondità32; errori tecnici/
protocollo separati dagli esiti normativi; un RunRef per bundle e terminale
affidabile; query/indice/dati/schema/reference e writer posseduti da Rust;
pagine e testi bounded, nessun dataset/record completo o bulk diagnostici IPC;
selezione/conferme export native legate allo scope; capability minime e assenza
di approval/PERSIST/C1 nelle89MIG. I dettagli ordinari restano autorizzati.

La ratifica complessiva soddisfa la parte approvativa P2-11; si può poi chiudere
P2-10 verificando tutti i prerequisiti, registrare G-MIG-2 APPROVED e avviareP3.
Fino ad allora non si scrive codice bridge o scaffold applicativo P3.
Fonte del vincolo: prompt P2-10, «G-MIG-2 APPROVED prima del codice bridge;
firme e primitive non finali sino ad allora», eP2-11, «pubblicati e approvati
nel gate G-MIG-2». Non è una nuova approvazione imposta dalle skill.

## Garanzie da qualificare nell’implementazione

Conservati tutti i gate del piano: parsing/limiti/autorizzazione Rust e DTO TS
reali, AppManifest/capability effettive, consumer/worker bounded e race/cancel,
Full bundle/parametri e protezione di tutti gli input negli export, parity nativa,
sentinelle PHI/XSS/log/temp/cache, Windows/Linux/offline/installer eG-REL.
Nessuna modifica a engine/norme/SQLite/HMAC/CLI/golden o requisiti di accettazione.

Limiti dichiarati: cap applicativo dopo allocazione framework possibile;
TOCTOU ostile non neutralizzato dal preflight ordinario; nessuna transazione
multi-file/DB, power-loss universale o secure erase; parti delle prove restano
modelli di design. Il servizio conserva receipt di pubblicazione reale anche
quando l’export fallisce. Nessuna ripetizione di CI/VM per ratificare documenti.
