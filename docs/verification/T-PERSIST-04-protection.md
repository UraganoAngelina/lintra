# T-PERSIST-04 — protezione delle identità e audit v1

Prereq02/e9f477b e03/49a3f4e qualificatiCI40528607856/40581933176.
Scope autorizzato: digest keyed stabile, proiezione minima protetta per PK,
configurazione/provider chiavi e audit. Fonti ADR-PERSIST-001 §5, ARCH45,
DET167/172. Nessun backend reale, servizio storico o nuova regola normativa.

Scelta tecnica entro il mandato04: HMAC-SHA256 tramite RustCrypto hmac0.13.0,
sha2=0.11.1 già adottata con zeroize abilitato; SecretKey owned Zeroizing<Vec<u8>>
zeroize1.9.1. Nessuna crittografia riscritta a mano, chiave predefinita/hardcoded,
lettura implicita di env/file o esportazione dei segreti. Chiave almeno32byte,
fornita esplicitamente da KeyProvider iniettato; qualità/entropia e provisioning
restano responsabilità della sorgente configurata, non dedotti dalla lunghezza.
La cancellazione del buffer owned non è una promessa di eliminazione di tutte
le copie RAM/register/swap o del materiale interno temporaneo del vendor.

## Encoding deterministico v1

Ogni componente variabile: lunghezza byte u64 big endian + byte originali.
Nessun trim, case folding, normalizzazione Unicode o codepage locale.
HMAC riceve in ordine componenti dominio `LINTRA-PERSISTENCE-PROTECTION-v1`,
purpose, key_id; poi namespace con conteggio u64BE, parametri nell'ordine
canonico del tipo02, per ciascuno nome+tagString/Integer+valore (StringUTF8,
Integer i64BE); flow; tagSchemaVersion/CompatibilityGroup e valore; trace;
key_kind. I tag sono byteASCII espliciti: String=S, Integer=I, SchemaVersion=V,
CompatibilityGroup=G; nessun nome inferito dai flussi.

Identity: aggiungere il JoinKey PK canonico del core come componente.
Record: aggiungere PK canonica, digest32byte esplicito della definizione della
proiezione e JoinKey della proiezione selezionata. Il descrittore entra anche
nel protection_scheme del record: definizioni diverse sono contesti diversi,
non record «aggiornati» dedotti in modo implicito. Il producer futuro deve
legare il descrittore al compiled profile; non si inventa una compatibilità.

Scheme identity `lintra.hmac-sha256.key.v1`; scheme record
`lintra.hmac-sha256.record.v1:<projection-definition-sha256>`.
Output solo PersistentRecordIdentity + record Fingerprint in StoredIdentity;
mai JoinKey/raw/CF/righe o valori recuperabili. Date/stati temporali recuperabili
e cifratura di payload ulteriori restano task successive con policy esplicita.
SchemaScope è metadata esatto: compatibilità deve essere qualificata dal compiler,
non dal digest o da una label scelta qui. Nessuna modalità storica abilitata.

## Preflight e audit chiave

Mancanza/ID vuoto/provider indisponibile/chiave corta => errore tecnico redatto,
senza fallback, output o chiave generata. SecretKey non Serialize né Clone,
Debug redatto. Provider restituisce Arc<SecretKey> senza copie manuali del buffer.

ProtectionAudit pubblico contiene policy_id e conferma HMAC di dominio/purpose
`key-confirmation`/key_id, senza namespace o dati paziente. Schema conferma
`lintra.hmac-sha256.confirm.v1`. Preflight con audit storico verifica policy,
key_id e tag con verify_slice constant-time: stessa label con materiale cambiato
è KeyMismatch, non un nuovo universo di PK silenzioso. Audit assente è ammesso
solo per inizializzare un contesto nuovo; il futuro service/preflight07 deve
richiedere l'audit della base prima del confronto, non può saltarlo per uno store
esistente. Rotazione/migrazione automatica e provider OS non vengono simulati.

## Fonti e gate previsti

Fonti primarie: [RustCrypto hmac0.13.0](https://docs.rs/hmac/0.13.0/hmac/),
[manifest tag ufficiale](https://github.com/RustCrypto/MACs/blob/hmac-v0.13.0/hmac/Cargo.toml),
[zeroize1.9.1](https://docs.rs/zeroize/1.9.1/zeroize/),
[RFC4231](https://www.rfc-editor.org/rfc/rfc4231.html).
MSRV verificate1.85/1.87 <= pin1.99.1, licenze MIT/Apache2.0.
Dipendenze risolte prima in workspace temporaneo con copia del lock: nuovi soli
hmac0.13.0,zeroize1.9.1,ctutils0.4.6,cmov0.5.4; pin precedenti preservati.
Test: vettore RFC indipendente, golden di frame da Pythonhmac separato,
determinismo namespace/case/tipi/key/purpose/definition, mancanza chiave,
audit/tamper/mismatch, secret debug/noSerde/noPHI, estrazione logical reale.
Report e gate finali saranno aggiornati con risultati effettivi, non presunti.

## Implementazione e primi test

src/persistence/protection.rs implementaKey/Provider/Scope/Audit/Protector/Error,
export additivi root. Nessuna modifica delle DTO02 o dei port03. Errori typed
statici; SecretKey owned nonClone/nonSerialize eDebug redatto, Arc nel provider.
Preflight verifica con MAC constant-time l'audit della base. Scope exact metadata
non è un proof compilato: binding al profile/recorddefinition affidato al futuro
preflight applicativo, senza attivazione della modalità storica o storage reale.

7integration+unitRFC passati dopo allineamento del helper alla firma effettiva
SchemaCompiler.compile(raw) con encoding.resolved esplicita. Golden scritto prima
con Pythonhmac/struct/hashlib da frame documentato, nessun expected ricavato daRust.
Testano ogni scope, namespace/type/padding, PK/payload/purpose/definition, chiavi
mancanti/corte/diverse e audit corrotto/mismatch; Secret debug/no raw inoutput.
Aggiunta al caso reale UTF8 la proiezioneÉ con stessaPK e fingerprint diverso.
Guard lock: tutti i pin/checksum precedenti intatti, sole4aggiunte già qualificate.
Gate completi in esecuzione, nonDONE04 prima della nuovaCI nativa.

## Ripresa dal commit utente dd7135e — 2026-10-28

Il commit `inter session staging commit` conserva implementazione e primi test,
non la chiusura della task. Working tree iniziale pulito; branch
`work/p1-foundations`, remoto sullo stesso SHA. Ultima CI40589848698 positiva
riguarda UI04, non le aggiunte di protezione. Prerequisiti02/03 confermati dai
rispettivi dossier e gate nativi, nessun lavoro DONE rieseguito.

La pipeline locale salvata documenta fmt, Clippy core/workspace/all-features e
750 test workspace positivi (93 suite, due ignored). Il log all-features
precedente termina durante la compilazione: non vale come gate positivo.
La ripresa completa quel comando e il core senza feature prima della nuova CI.

### Spiegazione delle funzioni e delle condizioni

`PersistenceKey::new` trasferisce il buffer in `Zeroizing` prima di verificarne
la lunghezza. Anche un candidato rifiutato viene quindi gestito dal contenitore
che azzera il buffer al rilascio. L'assenza di Clone/Serialize e il Debug redatto
limitano le copie e gli output accidentali; il provider condivide un Arc senza
esporre un accessore ai byte. Nessuna fonte segreta viene scelta implicitamente.

`label` rifiuta metadati vuoti o con caratteri di controllo, conservando il
valore originale valido. `IdentityProtectionScope::new` controlla identificativi
e variante schema prima di possedere lo scope; il namespace è già validato dal
tipo02. Non modifica padding, maiuscole o tipi dei parametri.

`component` antepone una lunghezza u64 big endian verificata: concatenazioni di
componenti differenti non diventano lo stesso messaggio. `mac` inizializza la
primitiva qualificata e separa dominio, purpose e ID chiave. `scoped_mac`
aggiunge numero e componenti canonici del namespace, tag di tipo, flow,
versione/gruppo, trace e key_kind. Ogni differenza nello scope entra dunque nel
messaggio autenticato, anche quando i byte della PK sono gli stessi.

`preflight` rifiuta ID assente/vuoto e propaga errori typed del provider senza
fallback. Se esiste audit storico chiama `verify_audit` prima di restituire
l'handle. `audit` produce esclusivamente policy/ID e conferma keyed separata.
`verify_audit` rifiuta policy/scheme/lunghezza inattesi, poi ID differente,
infine confronta il MAC con la verifica constant-time della libreria. Un cambio
di materiale sotto lo stesso ID non può passare come continuità dello storico.

`protect_key` autentica scope e PK canonica, costruendo solo identità protetta.
`protect_projection` riusa quell'identità e autentica separatamente PK,
definizione della proiezione e valore canonico selezionato. Anche lo scheme del
fingerprint include la definizione, così il confronto di contesto non nasconde
un cambio della selezione. Non conserva dati raw né sceglie campi normativi.

`ProtectionError::Display` usa messaggi statici; `source` espone soltanto errori
metadata già redatti. Gli altri Debug pubblici omettono chiavi/scope/audit.
I test usano esclusivamente materiale RFC e fixture sintetiche dichiarate.

Oracle riproducibile: `python3 docs/verification/T-PERSIST-04-golden-check.py`,
PASS nella ripresa. Il programma ricostruisce header, scope e JoinKey logical
con struct/hmac/hashlib standard, senza importare il codice Rust o rigenerare il
golden. Un disaccordo fallisce invece di aggiornare automaticamente l'atteso.

La prima ripresa all-features è stata interrotta esplicitamente (exit130):
linker concorrenti in attesa I/O da oltre dieci minuti. Nuovo tentativo con
`-j 2`, stessi test/feature/lock; nessuna modifica della configurazione globale.
Rimossi solo otto temporanei linker di ieri, non i file della build corrente.
CI sul salvataggio originale dd7135e avviata:40656244960, ancora non qualificata.

## Gate della ripresa e correzione del tooling

All-features con due job terminato exit0:750passed,0failed,2ignored,93suite.
I gate precedenti default/Clippy/fmt restano validi per il codice protezione
immutato. L'oracle indipendente e l'audit governance sono positivi.

Prima letturaCI40656244960: Linux supera fmt/Clippy/test default/all-features,
core senza feature, grafo/font/pixel e benchmark. Fallisce poi Clippy tooling:
`cannot update the lock file .../xtask/Cargo.lock because --locked was passed`.
Nessun test fallito e nessuna eccezione ai gate: il lock separato non includeva
le nuove dipendenze del core. Allineato con risoluzione offline, aggiungendo solo
i quattro package già qualificati. Audit tomllib: ogni pin/checksum precedente
preservato; nessuna versione nuova o rimozione. Metadata locked offline native
positiva. La prima metadata non filtrata chiedeva block2 per Apple: non è un
nuovo requisito desktop Linux; ripetuta con filter-platform reale e passata.

`cargo clippy --manifest-path xtask/Cargo.toml --all-targets --locked --offline
-j 2 -- -D warnings`:PASS. Test tooling in verifica, poi commit/push della
correzione e CI nuova, senza cancellare la prima mentre Windows termina.
Nessun DONE o milestone prima del gate nativo completo sulla correzione.

Gate tooling completato: Clippy/fmt e test standalone locked/offline PASS; 14 test positivi, 1 ignored (probe release separati in CI). Residuo04: CI corretta, non DONE.

## Chiusura — 2026-10-28

Code `dd7135e` e correzione `c475f0d`, [CI40658672940](https://github.com/UraganoAngelina/lintra/actions/runs/40658672940)
completed/success su tutti quattro i job Linux, MSVC, confronto JSONL e pixel.
Qualificati anche tooling locked, release riproducibile e README smoke nativi.
Dossier strutturato in T-PERSIST-04-native-ci.json; task DONE, nessun residuo04.
Non abilita storico/storage reale, provider OS, rotazione/migrazione o normative.
La prossima05 deve confrontare filesystem/SQLite e ottenere la scelta backend
richiesta dal suo prompt; UI05 indipendente resta in verifica dei propri gate.
