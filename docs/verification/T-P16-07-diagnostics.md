# T-P16-07 — diagnostici e collaudo visivo (in corso)

## Implementazione

`field_guide.rs` legge esclusivamente handle compilati: il core descrive tipo,
classe lessicale, larghezza e posizioni fisiche, lato del contenuto/riempimento e
regola di estrazione logical. Nessuna nuova decisione normativa, nessun controllo
fisico e nessun divieto implicito delle cifre in AN. `ErrorFactory` aggiunge la
guida al messaggio comune a UI/CLI/report, con anteprima massima16 valori delle
dichiarazioni; la regola effettivamente fallita e il suo atteso restano presenti.
`range.rs` specifica entrambi gli estremi inclusivi invece di un riferimento vago.

`DeclaredDomain` enumera letterali esatti e intervalli IntegerLiteral senza limite
u128; il successore lavora sulle cifre e normalizza -0 attraverso il parser
esistente. `declared_domain_page` limita a64 elementi e restituisce un cursore
controllato, evitando materializzazione completa o scansioni delle pagine già
visitate. Sono valori della dichiarazione, non record completi validati né una
promessa che ogni valore soddisfi l'intersezione di tutti gli altri vincoli.

`PreparedAnalysis` conserva l'Arc readonly dello schema risolto dallo stesso
registry del servizio. Prima di pubblicare il report il worker verifica hash e
schema_key dell'outcome; una modifica intercorsa produce errore tecnico esplicito
e abort del report, preservando eventuali errori di cleanup. `AnalysisReport`
conserva lo schema con lo spool. Il controller consulta questo schema e lo scope
del risultato visualizzato, senza usare la selezione corrente della maschera.

La UI mostra spazi come ␠ e conserva l'hex canonico per byte non stampabili.
Il dettaglio contiene un browser di pagine con ripartenza esplicita; le pagine
non vengono accumulate in una collezione permanente. Regex/reference non sono
sostituite con codici inventati: il browser statico dichiara il proprio limite.
L'enumerazione dei dataset esterni completi resta da verificare prima della
chiusura del requisito dell'autore; non dichiarare tale requisito già completato.

## Collaudo in corso

Intervalli firmati, attraversamento dello zero, oltre-u128, pagine senza buchi,
continuazioni incompatibili e dominio enorme sono coperti dai nuovi test core.
I test visuali usano produzione Slint/software e le reference ratificate
invariate. Le zone comuni e i cambiamenti funzionali del layout sono dichiarati
in ui/tests/reference/README.md: non è un'approvazione implicita di nuove immagini.

Il gate locale UI06 è iniziato su245bf34; tutti i test di esecuzione default e
all-features sono passati, ma il rustdoc finale ha letto le nuove sorgenti UI07
contro il vecchio rlib e ha fallito con DomainPosition mancante. Non è una prova
di un difetto del commit245bf34 e non viene registrato come gate completo verde.
La qualificazione finale UI06 rimane affidata anche al runner CI con checkout
immutabile40897488097. Da questo momento nessuna modifica sorgenti durante i
comandi Cargo che le consumano; ripetere i gate finali sulla revisione coerente.

Il confronto ha individuato due differenze pregresse, non compensate aumentando
le tolleranze: nav-bg chiaro #16232E invece dell'ink approvato #14202B e tracking
del marchio assente invece di0.5px. Ripristinati entrambi i valori originali.
La fixture tecnica `error_factory_messages.txt` è aggiornata esplicitamente per
la nuova guida richiesta dall'autore; non è una baseline normativa.

Correzione isolata06:9ba2470, gate nativo40900252019 avviato.40897488097 non è
verde (Windows path protection), nessuna chiusura prematura06/07.

### Reference data — estensione in qualificazione

`LookupSet::page_after` è un metodo opzionale additivo: provider esterni non
implementanti restituiscono indisponibilità esplicita. Il backend standard
seleziona64 chiavi esatte in ordine lessicografico con65 riferimenti temporanei
(O(N log64) per pagina richiesta, non per record); nessun secondo indice o copia
permanente del dataset. Interrompe la scansione quando riceve cancellazione.
L'operazione viene eseguita da un thread UI joinable con slot singolo; cambio
selezione/ripartenza/shutdown cancella e ricongiunge prima di riusare il cursore.

`PreparedReferenceEvidence` clonabile condivide gli Arc dei lookup: il wrapper
CapturingReferences conserva precisamente l'evidenza restituita al servizio,
non rilegge file e non carica un dataset alternativo in fase di dettaglio.
Prima di associare la cattura al report si verificano key, declared key, hash
atteso/effettivo e disponibilità rispetto all'audit dell'outcome. Una cattura
assente/incoerente non viene presentata come dominio vuoto o valido.
Il browser enumera soltanto disponibilità Verified e mostra identità/versione/
hash effettivo. Degraded/Unavailable/unsupported restano espliciti, senza inventare
codici. I fallback letterali appartengono separatamente alla dichiarazione.

Test aggiunti:300 chiavi esatte attraversano pagine senza perdita e rispettano
cancellazione; worker reale cattura il dataset prima di una modifica successiva
del file, mantenendo l'elenco verificato originale; evidenza degradata non è
presentata come dominio valido. Gate ancora da leggere/completare.

Le dichiarazioni condizionali delle regole locali sono consultabili separatamente
con il loro ID; non sostituiscono incondizionatamente il dominio del campo.
Il template di messaggio scritto nello schema resta identico; l'atteso core viene
arricchito con guida e dominio della specifica assertion fallita. La UI mostra
anche il tracciato nel dettaglio, oltre alle posizioni fisiche già disponibili.

Clippy workspace all-targets/all-features positivo sulla revisione comprendente
browser e cattura dei dataset. Test core e nativi finali ancora in esecuzione.
Nessun DONE sulla sola compilazione o su un gate parziale.

Le regressioni pregresse assumevano attesi identici quando un campo cambiava
posizione fisica. Ora confrontano la semantica mantenendo verificata la guida
specifica di ciascun layout; non ignorano il contenuto diagnostico. Il nuovo test
padding deve ricreare esplicitamente il binding encoding runtime dopo il roundtrip
JSON (quel binding non fa parte dell'authoring serializzato): corretto il fixture,
nessun fallback dell'engine.

Aggiornamento deliberato dei soli golden sintetici per message/expected sotto
LINTRA_UPDATE_GOLDEN=1, da revisionare con confronto strutturale ignorando
esclusivamente questi due campi. Nessuna modifica a input, schema, esiti o norme.

Audit golden registrato in T-P16-07-golden-diff.json:12 diagnostiche in3 file
sintetici, sole chiavi message/expected modificate, ordine e tutti gli altri
valori identici. Nessun summary/input/schema aggiornato. Suite core finale:
solo test DSL di confronto layout da correggere nel ramo senza campo fisico
(assertion di record); tutti gli altri target e doctest positivi. Non è un cambio
dell'esito del motore: il nuovo helper del test non deve fare unwrap di field=None.
Correzione limitata al test; da verificare con l'intera suite DSL.

La guida ora esplicita anche la codifica effettiva e il target del confronto:
raw include tutto lo slot e il fill; logical segue l'estrazione dichiarata;
il range confronta il valore intero. Per strip, uno slot composto solo da fill
produce logical vuoto, senza aggiungere un validator fisico. Test AN con sole
cifre preserva la liceità del dato secondo il tipo. Gli errori di consultazione
reference distinguono binding mancante, Degraded e Unavailable; il provider
non enumerabile non viene confuso con un dominio verificato vuoto.

Il nuovo test negativo schema_binding sostituisce esplicitamente il servizio con
uno schema valido diverso ma stessa key: il worker rifiuta l'outcome con
SchemaChanged prima di esporre un report con guida obsoleta. Il test numerico
con lettera conserva A01 come letto e spiega cifre ASCII, segni e separatori.
Il test visivo puramente presentazionale congela l'orologio a ZERO; i test con
controller asincroni usano invece un Instant monotono reale per guidare i Timer.

Memoria: la cattura condivide i lookup già preparati dal servizio e ne prolunga
la vita fino al rilascio del report visualizzato; il costo è proporzionale ai
reference dataset di quel run, non nullo. Non duplica tutte le chiavi né crea
una cache permanente di tutti i run. La selezione della pagina conserva al più65
riferimenti temporanei e64 valori di output; nessuna copia dell'archivio storico.
I letterali reference usano lo stesso rendering core dei domini statici: le
virgolette distinguono il codice vuoto da un insieme senza codici, gli spazi
restano visibili. L'insieme verificato vuoto è dichiarato esplicitamente.
