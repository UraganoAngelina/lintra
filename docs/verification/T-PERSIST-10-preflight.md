# T-PERSIST-10 — preflight CLI e contratti client storici

31 ottobre 2026, base codice `4313f22`, handoff remoto `c49631b`.
Task09 sospesa fedelmente sul gate CI41194045486; nessuna dipendenza aggirata.

Prerequisiti formali: PERSIST07 DONE su `1e92b1f`, CI40877202683 positiva
([evidenza](T-PERSIST-07-native-ci.json)); CLI8-03 DONE su `10932f9`,
CI39743161012 positiva, [report](T-P8-03.md) e registro corrente.
Il collegamento al nuovo servizio audit09 richiede anche il relativo gate
prima di qualificarne l'uso come prerequisito eseguibile.

Letture: prompt10, ARCH45/DET166–174/ADR001, ratifica ADR002, CLI args/command/
exit_code, request e servizi session/history_audit/historical_validation,
contratti del protector e wiring corrente UI. La UI Slint concreta è esclusa.

## API e file candidati

`src/cli/args.rs` per intento storico e argomenti espliciti; command/wiring
dedicato per preview, audit, pubblicazione report e approvazione; contratti
client in application/lib; adapter del provider solo dopo decisione. Test
CLI sintetici proceed/abort, output audit, exit0/1/2/130, report failure,
cancellazione, approvazione e guasti del provider. Nessun SQL o regola nel client.

## BLOCCO circoscritto — provisioning della chiave

Non esiste un provider sicuro configurabile dall'eseguibile. La ratifica ADR002
§2 lascia provisioning/cifratura/retention/backup aperti prima dei dati reali;
il trait generico non approva implicitamente un adapter di gestione dei segreti.
Nessun parametro deve essere ignorato e nessuna chiave sintetica può diventare
un default runtime. Decisione concreta proposta in
[PERSIST-KEY-PROVISIONING-01](../proposals/PERSIST-KEY-PROVISIONING-01.md).

Impatto: si sospende l'implementazione dipendente del provider/CLI10, senza
modificare codice o dichiarare test passati. Le API esistenti non vengono
disabilitate e la CI09 prosegue. Non è un nuovo checkpoint ordinario di sessione.
T-PERSIST-11 è indipendente da10 e09: richiede05/07/08, già qualificate, e può
proseguire in sequenza durante l'attesa. Nessun consenso è dedotto dal timeout.


## Aggiornamento4novembre2026 — blocco superato da ratifica esplicita

L’autore approva A aggiornata: [ratifica completa](PERSIST-KEY-PROVISIONING-01-ratification.md).
Chiave comune32byte/Key ID stabile nel bundle Slint privato, CLI path/ID espliciti,
risoluzione Slint deterministica dall’eseguibile/config controllata. Garanzie del
file effettivamente aperto, ACL/ownership/link/race, lettura33byte e azzeramento
invariati. Nessun installer, deposito credenziali o segreto operativo prodotto.
La proposta31ottobre resta storica e subordinata alle nuove prescrizioni.

PERSIST09 è ora DONE: CI41194045486 realmente success su4313f22, entrambi i job
nativi e tre confronti pass.07 e8-03 restano qualificate come sopra. Il blocco di
scelta è superato; task10 IN_PROGRESS/preflight, prossima prioritaria dopo il
commit/push e dispatch della task11 localmente qualificata. Verificare layout
reale packaging e contratti file sicuro prima di implementare il provider.

Scope aggiuntivo autorizzato: helper di risoluzione del bundle e documentazione
portabile/assemblaggio offline/recupero manuale. Non introdurre schermate storiche
Slint, installer o migrazioneTauri, né preparare il bundle operativo ora. Sicurezza
incompatibile col modello portabile richiede conflitto circoscritto, mai bypass.
