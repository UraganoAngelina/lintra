# T-PERSIST-04 — protocollo di protezione v1 (pre-implementazione)

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
