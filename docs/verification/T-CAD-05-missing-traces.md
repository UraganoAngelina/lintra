# T-CAD-05 — file mancanti, warning e consenso

WAITING_GATE, 7 novembre2026. Sei casi nuovi e52regressioni =58test positivi,
0fail/0ignored. Clippyworkspacealltargetsallfeatures positivo. Sette gate locali
finali conclusi positivamente; nessunaCI05 oDONE attribuiti.
Prerequisiti reali03/PERSIST10 verificati nel preflight;04qualificata usata
per la prova aggiuntiva sui progressivi, senza dipendenza pendente.

## Perimetro

Il comportamento richiesto esiste già nei port core. Si qualifica la sua
composizione CAD invece di duplicare codice o introdurre schermate simulate.
Il solo Rust modificato è un modulo test in cad_progressives.rs; nessun
cambiamento applicativo/engine/storage, crate o contratto. La policy snapshot
compilata e le proiezioni originali sono esattamente quelle04già qualificate.
Non si disabilitano invarianti o capability per rendere eseguibile lo schema
clinico incompleto. Gli outcome sono fixture pubbliche di componente dichiarate,
non attestazioni di una validazione clinica Full. La verifica produttore/CLI/
Slint storica completa rimane distinta ai rispettivi gate/CAD12.

## Helper e correttezza

portable_archive riusa archive del fixture04 per il database di prova. Crea
un contenitore privato separato usando lo stesso helper pubblico qualificato
dal test CLI10, così le ACL Windows ereditate e il modo Unix sono reali. Scrive
soltanto32byte pubblici fissi0xaa, già usati dai fixture precedenti, nel percorso
portabile calcolato dal vero portable_persistence_key_path. Il file è distinto
dall'archivio effettivo; non si genera o distribuisce una chiave operativa.

provider carica quel percorso con FilePersistenceKeyProvider::load e KeyID
esplicito. Nessun fallback/ambiente/ricerca o provider fittizio sostituisce
il controllo del file. Il proteggitore e SQLite vengono poi costruiti sul
provider reale, anche dopo chiusura/riapertura dei rispettivi scope.

prepared riusa parts/outcome/projected del fixture04 su righe sintetiche alle
coordinate originali, con ULSS501/anno2026 dichiarati. Il profilo non viene
alterato: restituisce preparazione privata, outcome di componente e proiezioni
opache dello stesso input selezionato. absent controlla esplicitamente presenza
e proiezione; un dato nel buffer del fixture non viene fatto passare per file
fornito se la richiesta lo omette. decision costruisce il vero consenso con
tracce esplicite e data UTC fissa; nessun orologio host decide l'esito.

## Casi nuovi

Il primo caso seleziona A/B1/C/D senzaB2, legge warning/lista prima di una
review e prova il rifiuto di accesso aValidated senzaPROCEED. Generazione e
indiceinvii restano vuoti. Il secondo approvaT1, poi cancella sia un draft
sia una preview con omissione: nessuna seconda source né callbackreport può
essere pubblicata, anche dopo riapertura del database.

Il terzo usa T1completo/T2senzaB2/T3conB2uguale e providerfile ricaricato.
InT2i tre progressivi dei file presenti restano invariati, nessunB2 compare
nei diff e disappeared=0. La fonte riaperta conserva Absent senza hash perB2,
Present(1) con hash perA/B1/C/D e il consenso esatto con timestamp. Il vero
proteggitore verifica la fonte restituita dal codec SQLite. InT3B2non genera
riattivazione: non era stato annullato per un file non ricevuto.

Il quarto distingue B2Present(0): nessunwarning/assente, hash del contenuto
vuoto presente, consenso perB2rifiutato perché non è assente. DaT1aT2compare
esattamente un annullamentoB2; inT3il ritorno dello stesso evento è una vera
riattivazione. Tale differenza deriva dalla policy, non da un contatore locale.

Il quinto richiede copertura di B1/B2entrambi mancanti e rifiuta consenso su
tracce presenti/sconosciute, lista duplicata/vuota o incompleta. L'ordine del
consenso viene canonizzato, non dedotto da uno stato implicito. Il sesto
verifica wirePresent/Absent e nega partial, campi extra suAbsent, ABORT come
PROCEED durevole o campi extra nel consenso. Nessun nuovo stato viene inventato.

## Provenienza pubblica e protetta

Il JSON pubblico di una fonte approvata conserva decision/traces/recorded_at,
ma deserializzarlo rimuove deliberatamente la prova privata. verify_submission
e protect_submission rifiutano la DTO export anche se non alterata, così
come una copia con timestamp cambiato. La prova del persistito viene invece
controllata sul valore realmente ripristinato daSQLite, con il providerfile
ricaricato. Non si confonde la serializzazione pubblica con un'attestazione
né si attribuisce una provaHMAC a una semplice deserializzazione.

I preliminari registrano due correzioni dei test: Utc::now non disponibile
nel build senza clock host, sostituito con timestamp fisso; aspettativa errata
di attestazione dopo JSONpubblico, sostituita con il rifiutoUnattested previsto.
Il codice produttivo e le protezioni non sono stati modificati per il test.

## Gate previsti e limiti

Come per03, modifica solo di accettazione/documentazione: fmt, Clippyworkspace,
workspace completo, regressione pertinente allfeatures, regressione core senza
bundle, release e boundary. Nessuna ripetizione dei test UIallfeatures locale
per lo stesso codiceprodotto immutato; laCI05eseguirà nuovamente l'intero
workspace default/allfeatures/core su Linux/MSVC e i confronti traOS.
Gli esiti07non qualificano i nuovi casi05. Nessuna fonte reale/paziente,
chiave operativa, FROZEN, release/tag o nuovo formato; docs/migration esclusa.


## Qualificazione locale finale

Sette comandi tutti exit0;469impronte prima/dopo identiche e ricontrollate
alla conclusione. Workspace987pass/0fail/6ignored, regressione pertinente
allfeatures58pass/0fail e core senza bundle58pass/0fail. Fmt, Clippyworkspace,
release e boundary positivi. I conteggi sono ricostruiti dai log e verificati
nel ledger05; inventario11log include i preliminari falliti. Il diff verifica
che src/UI/Cargo/build/workflow non abbiano alcuna modifica rispetto6477bb2.

La CI05propria deve ancora qualificare nuovi casi suLinux/MSVC, workspace
completo default/allfeatures/core e confronti traOS. NonDONE05 prima degli
esiti effettivi, nessuna nuova release o chiave operativa.
