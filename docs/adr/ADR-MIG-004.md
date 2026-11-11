# ADR-MIG-004 — Uso interno, test sintetici e autorizzazioni dati

Owner: autore Lintra; D01/D02/D03/D05/D14 e DEC-MIG-09 approvate in chat
il10 novembre2026, recepite in ADR-POST-M9-001 `2cd4bdf`.
Trascrizione11 novembre2026, T-MIG-P0-06.
Stato: **USER_APPROVED / REPOSITORY_RATIFIED**, nessuna autorizzazione aziendale
ulteriore attestata. Fonte: [ratifica](../unified-post-m9/RATIFICATION_2026-11-10.md)
e [uso interno](../INTERNAL_USE.md).

Lintra è destinata a software realmente usabile da autore e colleghi autorizzati,
con LocalOnly, Full e storico previsti; non una demo come obiettivo finale.
La consegna richiede G-REL e tutti i gate tecnici applicabili. Non si afferma
certificazione clinica né che una decisione del progetto autorizzi l'uso di
qualunque dato del datore di lavoro.

Test, CI, benchmark, esempi, log dimostrativi e screenshot usano **sempre dati
sintetici**. Non importare record reali dalle cartelle dell'autore per popolare
fixture o UI. I materiali normativi forniti e i cataloghi qualificati sono fonti
progettuali; non rendono lecita una fixture sanitaria reale. L'autorizzazione
al trattamento operativo non rende quei dati adatti a Git/CI/chat/screenshot.

Per specifici dati personali lavorativi valgono le autorizzazioni e i controlli
aziendali effettivamente applicabili e documentati. Non presumere che siano
presenti. G-REAL è limitato a questi casi: il relativo controllo non introduce
un'infrastruttura enterprise e non rinvia GUI, prove sintetiche o package
tecnico funzionante. Non richiede un'altra ratifica D01–D15 già acquisita.

Filesystem locale per account, ACL/ownership OS, HMAC e chiave comune32B
restano le protezioni ratificate. Nessuna nuova password custom, cifratura,
retention, purge, server o cloud è condizione implicita di migrazione. Protezioni
OS aggiuntive della cartella sono facoltative, accessi ordinari/temporanei
privati sono obbligatori. Custodia, continuità chiave, backup e recovery hanno
procedure e prove proprie, non nascono da un'etichetta di sicurezza.

Il renderer segue [ADR-MIG-003](ADR-MIG-003.md); i report esportati mantengono
i dati previsti dal formato canonico, senza redazioni silenziose per ottenere
un test verde. La scelta di destinazioni shared richiede warning/consenso
D04, senza denylist generale o nuova conferma privacy per un path locale.

Le diciture storiche «real data bloccati finché ADR/DEC aperte» non chiedono
nuovamente le decisioni già ratificate. Gli ADR ora recepiti chiudono la parte
di policy; restano reali i gate software, G-REL/signoff e le autorizzazioni
aziendali dei dati specifici. La distribuzione ai colleghi non è autorizzata
dal solo cutover, da questo ADR o da una CI sintetica positiva.
