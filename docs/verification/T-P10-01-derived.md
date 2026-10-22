# T-P10-01 — completamento del modello derivati ratificato

2026-10-22, gate locali passati, CI nativa pendente.
Ripresa secondo [rettifica P10](P10-derived-recovery.md), non nuova decisione.
Autorità: T-P0-07 completo 72f9f5e, CNF-LINTRA-DSL-01, authoring §valori derivati
ed adeguamenti P10, lowering compiled_contracts; prompt 10-01/DET §17–18.
P11-01 conclusa su 89b4baf/CI 39788642608 Linux/Windows verde; codice valido
preservato. Questo passo consegna modelli/lowering, non executor anticipati.

## Strutture, funzioni e condizioni

schema/derived usa i RawDerived/DerivedValuePlan già approvati. Non aggiunge
kind o parametri. compile_derived_values visita BTreeMap per nome canonico,
verifica ID nonempty/unpadded e raccoglie piani tipizzati fallibili senza
contesto parziale. L'ordine della mappa stabilisce lo stesso derived_index
usato dalle espressioni, senza dipendere dall'ordine dei field o da hash map.
La grammatica di queste tre operazioni legge solo FieldRef, non un altro
derivato: il grafo fields→derived→expr è aciclico per costruzione. I cicli
nelle condizioni nominate restano rifiutati dal resolver già consegnato.

compile_plan risolve ogni source locale tramite il medesimo resolve_local_ref.
Substring conserva target raw/logical, abbassa start one-based/length in
ByteRange e richiede che end_exclusive stia nella width fisica del campo.
Se logical sarà più corto, l'executor dovrà produrre Unknown per indisponibilità,
mai padding, troncamento o slicing unchecked. Qui non si leggono record.
Zero start/length, overflow e range esterno sono errori di schema espliciti.

Age/CalendarYearDelta richiedono due ref a physical Date; date_ref rifiuta altri
tipi con ConstraintTypeMismatch e ref assente/esterna con il codice già previsto.
Format DDMMYYYY mappato esaustivamente. LeapDayAnniversary Feb28/Mar01 è un
parametro obbligatorio del solo Age, senza default normativo. CalendarYearDelta
resta un piano distinto senza compleanno; nessuna età o data calcolata ora.

binding risolve il nome in indice/tipo (Substring Bytes, gli altri Integer);
nome mancante è MissingDerivedValueReference, nuovo diagnostico di schema,
non un FieldRef falsamente mancante o ValidationCode normativo. literal richiede
Text per risultato Bytes, RawInteger per Integer, con parsing esatto già
esistente e nessuna coercizione da stringa/floating point.

compile_local_expr abbassa DerivedEq con field_ref e indice derivato. Per un
risultato Integer esige target physical Numeric: non interpreta Decimal come
intero monetario o inventa centesimi. Il confronto Bytes non converte nulla.
DerivedCompare conserva l'operatore chiuso e il literal del tipo del piano.
operator_plan è estratto dal compiler rules e condiviso per i sei simboli:
nessuna seconda mappatura o nuovo alias. Booleani/condizioni nominate invariati.

source_range è spostato dal compiler in schema/coordinates e riusato per campi
e substring. Mantiene identici checked_sub/TryFrom/ByteRange e mapping dei
codici; non duplica la conversione one-based. I test precedenti del compiler
importano la stessa funzione e conservano la verifica dei limiti.

Checks valida tutte le definizioni derivate e segnala i problemi del modello.
Il gate generale derived_values e le capability derived rimangono bloccanti
fino agli executor: nessuno schema con controllo non eseguibile raggiunge il
runtime e nessun vettore viene scartato da un compiled valido. Non si modifica
applicazione, engine, sorgente normativa, formato o baseline.
I negativi compiler che usavano un nome derivato non dichiarato ora esercitano
un vero ref esterno non supportato; i nuovi negativi verificano precisamente
MissingDerivedValueReference per il nome derivato assente. Non si esige che
un nodo finalmente riconosciuto rimanga etichettato come sconosciuto.

## Test e handoff

Cinque test unitari nel modulo privato derived_tests, con record sintetico a
due date, Numeric AGE e testo CODE: tutti tre i piani/indici/range/tipi,
Feb28 e Mar01 espliciti, raw/logical; coordinate zero/outside, sorgenti Date
incompatibili, assenti/esterne; policy leap day omessa rifiutata in raw;
DerivedEq e DerivedCompare con literal negativo tipizzato; nomi assenti,
stringa numerica scorretta per tipo, numero su substring, AN confrontato a
intero derivato. Nessun costruttore pubblico compiled aperto, test solo modello.
Primo controllo ha rilevato due dettagli del refactor/test: commento rimasto
senza funzione e getter chiamato length invece di len; corretti, nessuna allow.

424 test passati in 52 suite, zero falliti; /tmp/lintra-t-p10-01-derived-test.log.
fmt/check all-targets/Clippy all-targets -D warnings/test/release/Windows GNU
check all-targets/diff --check exit 0, Cargo --locked. 64 file protetti SHA-256
identici a HEAD (schemas/normative/golden/meta-schema v1, no README).
Review di indici/namespace/tipi/range e gate runtime ancora chiuso positiva;
nessun source, Cargo/lock o baseline cambiato, nessun unwrap produttivo nuovo.
Windows GNU non sostituisce CI nativa MSVC. Next residuo T-P10-03 dopo CI: gli executor
reali e il contesto derivati. P10 resta IN_PROGRESS finché presence/rules/test
estesi non sono consegnati; P11-02 attende quella chiusura, senza blocco nuovo.

La regressione record che copiava tutte le regole dell'esempio authoring in
un altro schema ora incontrava ref/nomi derived assenti prima del ramo futuro.
Sostituita quella sola fixture con RecordExists esplicito ancora non eseguibile:
verifica UnsupportedConstruct senza dipendere da ID estranei o da un nodo
appena riconosciuto. ReferenceData resta un negativo indipendente identico.
