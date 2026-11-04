# PERSIST-KEY-PROVISIONING-01 — ratifica aggiornata dell'autore

**4 novembre 2026, APPROVATA CON PRECISAZIONI VINCOLANTI.** Fonte: risposta
esplicita dell'autore alla proposta durante la ripresa di sviluppo. Questo
documento registra la decisione; la proposta originaria resta consultabile.
Integra ADR-PERSIST-001, ADR-PERSIST-002 e T-PERSIST-04/05/10. In caso di
divergenza prevale la ratifica dell'autore riportata qui.

## Decisione e distribuzione

**Alternativa A ratificata: chiave HMAC unica, file binario esterno
all'eseguibile, incluso nel bundle privato portabile Slint.**

- Prima distribuzione privata a un gruppo ristretto di colleghi autorizzati:
  Slint portabile, senza installer o provisioning amministrativo. Nessun Git,
  Rust/Cargo, SQLite separato, server, cloud o credential manager richiesto.
- Esattamente32byte binari casuali, generati una sola volta mediante CSPRNG
  per la distribuzione, stesso materiale e stesso Key ID non segreto per tutti
  i destinatari. Nessuna generazione per PC/account/installazione/avvio e
  nessuna derivazione individuale che modifichi le identità persistenti.
- Il file contiene solo i32byte: niente password testuale, JSON, Base64,
  certificato o metadati. Key ID in configurazione non segreta o contratto
  esistente. Materiale mai incorporato in Rust o nelle risorse compilate.
- Il layout illustrativo dell'autore (`lintra-ui.exe`, `hmac.key`, `config`,
  `schemas`, `resources`) va confrontato col packaging reale prima di diventare
  contratto. La preparazione effettiva della release resta alle task packaging.
- Artefatto software generico privo di segreti; bundle privato completo con
  chiave comune, identico per i destinatari previsti. Assemblaggio offline
  controllato dopo la build generica, oppure processo privato equivalente
  esplicitamente qualificato. Mai chiave operativa in Git, artifact pubblico,
  cache CI accessibile, log dell'agente, report o prompt. CI/test solo sintetici.

## Caricamento e confini di sicurezza

- Autorizzata risoluzione deterministica limitata dalla **posizione effettiva
  dell'eseguibile** o da configurazione esplicita e controllata del bundle;
  esempio `<directory-eseguibile>/hmac.key`. Nessuna ricerca in cwd, home,
  temporanei, percorsi di sistema non configurati o scansione ricorsiva.
- CLI mantiene Key ID e percorso file espliciti. Il provider riceve sempre una
  sorgente precisamente identificata; l'adattamento Slint non è discovery
  generale. Nessun wizard/copia automatica in area privata obbligatorio ora.
- File nel bundle non significa file sicuro: mantenere ownership, permessi/ACL,
  tipo regolare, rifiuto link/reparse/hardlink secondo contratto e controlli
  contro sostituzioni **sul file effettivamente aperto**. Mai disabilitarli per
  far funzionare l'estrazione portabile. Documentare collocazioni e accessi
  richiesti; incompatibilità reale richiede proposta circoscritta all'autore.
- Provider legge al massimo33byte e accetta solo32, usa memoria azzerabile
  prevista dal contratto, chiude il file e non espone segreti in Debug/errori/
  log/report. Nessuna chiave sostitutiva o generazione al caricamento.
- Chiave distinta da archivio/input/report/temporanei: nessun output può
  sovrascriverla. Conservare tutte le altre garanzie della proposta originaria.

## Contratti preservati e continuità

- `PersistenceKeyProvider` carica e protegge il materiale secondo T-PERSIST-04;
  `PersistentContextRepository` resta port generico senza path/layout bundle.
  SQLite non acquisisce nuove regole HMAC. Algoritmo, canonizzazione, formati,
  namespace, regole normative e semantica storica invariati.
- Ogni PC conserva un archivio SQLite **locale indipendente**. Nessun database
  di rete, sincronizzazione, merge, Git degli invii o servizio remoto. Chiave
  comune consente digest coerenti a parità di input canonici/scope; non prova
  uguaglianza degli storici né assenza di eventi su altre postazioni.
- Chiave/Key ID stabili durante aggiornamenti, nuove build e spostamenti del
  bundle. Nessuna sovrascrittura automatica, reinizializzazione o conversione
  tacita. Key-confirmation obbligatoria anche con namespace vuoto; Key ID
  incompatibile o stesso ID con materiale diverso sono errore tecnico.
- L'autore accetta esposizione comune e possibilità di correlazione con chiave
  condivisa. HMAC non anonimizza automaticamente e non cifra SQLite; questa
  scelta non equivale a chiavi individuali in credential manager. Restano
  minimizzazione, protezione degli archivi, isolamento e diagnostica redatta.
- Copia di riferimento protetta del file originario/Key ID conservata dal
  proprietario fuori Git/CI/log/report/prompt, mai sostituita automaticamente.
  Documentare recupero manuale controllato e verifica di compatibilità.
  Compromissione interessa potenzialmente tutti gli archivi; nessuna semplice
  sostituzione della chiave o rotazione/migrazione implicita in questa task.

## Scope autorizzato e verifiche

Autorizzati adapter file, wiring CLI T-PERSIST-10, convenzione deterministica
Slint, contratto documentale portabile, test di caricamento/identità/continuità
e specifica di assemblaggio privato. Confermati preview senza commit, abort,
approvazione separata, report fallito prima del commit, CAS, cancellazione,
esiti ambigui e assenza di pubblicazioni parziali.

Verificare32byte validi, assente/vuoto/corto/lungo/non regolare/illeggibile,
permessi/ACL non sicuri, symlink/reparse/hardlink, sostituzione, redazione;
percorso relativo all'eseguibile senza cwd discovery, bundle spostato o senza
chiave, ID errato, collisioni con tutti gli input/output/part; due installazioni
sintetiche con chiave comune, HMAC coerente, namespace isolati e DB indipendenti;
restart/key-confirmation, update compatibile e nessuna inizializzazione implicita.
Qualificare le capacità pertinenti su Linux e Windows; presenza di `hmac.key`
da sola non prova protezione del bundle. Nessuna chiave operativa nei test.

Non autorizzati installer anticipato, credential manager/DPAPI/secret manager,
infrastrutture remote, sincronizzazione, rotazione o migrazione automatica,
nuove norme CAD/SIOC, modifica privacy/garanzie senza ratifica o uso sanitario
reale. Cifratura a riposo, retention, backup completo e autorizzazione dati reali
restano aperti. Il restore SQLite rimane alle task dedicate.

React+TypeScript+Tauri2 e installer/wizard sono evoluzione **successiva al core**:
nessuna implementazione anticipata. In quella fase potranno essere valutati
area privata, ACL automatiche, upgrade/disinstallazione, recupero, chiavi
individuali e deposito credenziali. La transizione dovrà preservare gli storici
creati con chiave comune o avere migrazione esplicita; vincolo evolutivo, non
funzionalità richiesta ora. Nessuna modifica autonoma dell'architettura futura.

Workflow confermato: single-agent, niente Superpowers/subagenti/background,
commit/push/CI autorizzati, gate obbligatori, prosecuzione automatica delle READY,
arresto solo su criticità reali. **Procedere con T-PERSIST-10** applicando questa
ratifica; il precedente blocco di scelta del provider è superato.
