# T-P15-02 — Verifica dell'artefatto

## Perimetro e decisioni

2026-10-26. Ripresa da cce79bd dopo ratifica A dell'autore:
CNF-ARTIFACT-LINUX-01 risolta, Linux solo interno/personale e distribuzione ad
altri su Windows. Allowlist GNU esatta dei quattro SONAME; nessun wildcard.
Prerequisito15-01 realmente verificato dalla CI40307076976 su7c919fb, entrambi
i job nativi success. Nessuna modifica di ARCH/DET, baseline normativa, API
applicative, regole engine, profilo panic o Cargo.lock del prodotto.

Il verifier e il parser sono esclusivamente nel tooling xtask. Il comando
build-release --verify-artifact verifica l'artefatto prodotto e la prova negativa;
verify-artifact --target TARGET --artifact PATH verifica un file esistente.
Target obbligatorio e nessuna deduzione di identità dal nome del file.
Musl è verificabile strutturalmente, ma non viene scelto come nuova baseline
di build/distribuzione. La motivazione dell'autore non elimina i gate Linux.

## Strutture, funzioni e condizioni

- Options nel main aggiunge verify_artifact, false se non richiesto; parsing
  rifiuta duplicati. Il workflow lo richiede esplicitamente insieme al confronto
  delle due release. Il risultato della build non viene certificato implicitamente.
- main legge args_os e converte con errore tecnico gli argomenti non Unicode,
  evitando il panic di env::args. Instrada soltanto il comando tooling esplicito;
  la ricetta build precedente mantiene flags/isolamento/confronto integrale.
- supported limita i target ai tre formati dichiarati. cli richiede selettori
  unici, target e file espliciti; argomenti sconosciuti o mancanti falliscono.
- inspect usa Goblin sui byte del file, senza eseguirlo. Rifiuta formato o
  architettura errati, immagini PE DLL/entry zero ed ELF senza entry o di tipo
  non eseguibile. Windows richiede PE64 x86_64. ELF richiede64 bit little-endian,
  x86_64, ET_EXEC oppure ET_DYN per PIE.
- ELF GNU controlla DT_NEEDED con la lista ratificata e accetta soltanto il
  loader ld-linux-x86-64.so.2 nei percorsi di sistema /lib64, /lib oppure
  /lib/x86_64-linux-gnu. ELF musl rifiuta qualunque PT_INTERP e qualunque libreria
  dinamica: il solo suffisso del target non prova staticità.
- Windows usa basenames ASCII case-insensitive, una lista esplicita di DLL OS
  e i namespace api-ms-win-/ext-ms-win- con suffisso.dll. Rifiuta path/traversal,
  DLL estranee, runtime GCC/pthread/VCRUNTIME/MSVCP e CRT dinamica, compreso
  api-ms-win-crt-. Il [contratto API set Microsoft](https://learn.microsoft.com/en-us/windows/win32/apiindex/windows-apisets)
  descrive questi namespace e il ruolo del loader; non si deduce la disponibilità
  su tutte le versioni Windows dalla sola forma del nome.
- Goblin espone gli import PE ordinari ma non un inventario completo dei delay
  imports. Perciò immagini con directory delay import vengono rifiutate
  esplicitamente: nessuna certificazione di import non esaminati.
- check_libraries ordina/deduplica i nomi prima dei controlli: output e primo
  errore deterministici. GNU confronta byte/case esatti; Windows segue la
  semantica dei nomi OS. Nessuna ricerca di DLL o librerie installate sull'host.
- host_paths raccoglie root repository, home, Cargo/Rustup home, forme lessicali
  e canonical e firme host precise. Errori di risoluzione/Unicode falliscono.
  normalized uniforma slash e case ASCII soltanto nella copia di ricerca;
  check_paths controlla byte UTF-8 e UTF-16 LE/BE, con pattern e offset in errore.
  L'artefatto non viene modificato o normalizzato per farlo passare.
- Un primo test reale ha rilevato la firma /private/ troppo ampia: intercettava
  /cargo/.../serde/src/private/de.rs, percorso virtuale correttamente remappato.
  La firma è ora /private/tmp/, insieme alle radici reali e /home/, /users/,
  /root/, /tmp/, /__w/ e directory CI Windows. Il test usa la lista effettiva
  per accettare quel percorso virtuale senza perdere il controllo degli host.
- verify propaga errori file/parser/path e avvia cargo tree --locked
  --all-features --target TARGET --prefix none --format {p}, eventualmente
  --offline. Solo exit0 e output UTF-8 interpretabile possono attestare assenza
  di openssl-sys. check_tree confronta l'identità esatta del package, non una
  sottostringa. Il tree completo evita di scambiare l'errore di cargo tree -i
  su un package assente con un guasto tecnico del lock o della rete.
- negative_probe copia l'eseguibile verificato in un nuovo file esclusivo,
  aggiunge un path host dichiaratamente errato e invoca il vero sottocomando
  tramite current_exe. Richiede exit1 e la causa host path: un crash o un altro
  errore non valgono come successo del test. Originale preservato, nessuna
  esecuzione dell'artefatto alterato, niente background/delega.

## Test e limiti

9 test tooling: selettori build e verifica; path con spazi; liste GNU/Windows
positive/negative; UTF-8 e UTF-16/case/slash; binari troncati/formato errato;
header ELF/PE sintetici con architettura, staticità, interprete e delay imports;
parse dell'eseguibile nativo reale dei test e mutazione delle librerie importate;
identità openssl-sys distinta dalle sottostringhe. Le fixture header sono
esplicitamente sintetiche; non rappresentano un programma clinico o normativo.

Il checker riguarda import dichiarati e tracce dei path noti, non una prova
universale di tutti i caricamenti dinamici futuri o dell'integrità delle librerie
presenti sul PC destinatario. Non esegue dlopen/LoadLibrary per certificare ciò
che trova sull'host. Non promette supporto delay imports, UI o baseline future.
M5 richiede ancora bundle/release/test cross-OS delle task successive.

## Evidenze locali

Tutti exit0: fmt prodotto e tooling; Clippy prodotto/tooling all-targets -D
warnings; test prodotto (671 pass/77 suite/0fail/2ignored già previsti), tooling
(9 pass); build release Linux locked/offline2m34s; verifier positivo con i quattro
SONAME e openssl_sys=absent; probe del comando reale negativo exit1 per host path.
Toolchain1.99.1, CARGO_HOME=/tmp/lintra-cargo-home, debug dev/test0 e incremental0.
Regressione prodotto riusa target/verification-p14-05; tooling/log in
target/verification-p15-02; la release usa una directory nuova propria.

Il prodotto non è cambiato dopo i gate. Dopo l'aggiunta del controllo interpreter,
Clippy/test tooling9/9, verifica positiva e negativa sull'artefatto reale e check
Windows GNU del tooling ripetuti, tutti exit0; negativo effettivo exit1/host path.
Un test inizialmente mutava anche PT_INTERP: ora il caso DT_NEEDED preserva il
loader e ne verifica separatamente il rifiuto, senza indebolire il checker.
Tree tooling Linux/MSVC e licenze transitive verificati nel registro DEC-15.
Audit governance PASS e git diff --check PASS. CI nativa da avviare, nessun DONE.

## Handoff provvisorio

Task IN_PROGRESS; implementazione disponibile, ratifica applicata. Restano
evidenze finali e CI Linux/MSVC sul commit pubblicato. Nessun blocker contrattuale
residuo. Prossima candidata15-03; da verificare prima di iniziarla, senza aggirare
prerequisiti o assumere dati ufficiali. Prosecuzione AUTO_CONTINUE dopo i gate.
