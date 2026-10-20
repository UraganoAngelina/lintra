# CNF-ALIGNMENT-01 — Proposta di contratto fisico completo

2026-10-20. **DRAFT — da approvare prima dell'implementazione.**

L'autore ha scelto «B: definiamo prima il contratto fisico» nel dossier
T-P6-03 del commit 73b94cb. La proposta A di consegna parziale non è approvata.
Questo documento propone un contratto verificabile per completare T-P6-03;
non modifica ancora le fonti approvate, gli schemi regionali o il codice.

## 1. Significato e limite del controllo

Un errore di allineamento significa violazione di un vincolo fisico **dichiarato**,
non ricostruzione dell'intenzione di chi ha generato il file. Il motore non può
distinguere il valore 1000 dal valore 1 erroneamente riempito a destra se riceve
soltanto i medesimi quattro byte `1000`.

RightPad e LeftPad continuano a indicare il lato del riempimento e il modo
esplicito di ricavare il logical. Si propone un unico criterio fisico aggiuntivo:
la possibilità che il fill compaia sul bordo opposto, quando esiste almeno un
byte diverso dal fill. È una restrizione configurabile sul dato osservabile,
non un'euristica legata a Numeric, Alphanumeric, SIOC o CAD.

Non vengono dedotti lunghezza originaria del contenuto, numero obbligatorio di
byte di padding, repertorio lessicale, dominio o posizione dei fill interni.
Questi eventuali requisiti devono avere regole esplicite distinte.

## 2. Proprietà proposta nello schema

Per `alignment.kind = right_pad` oppure `left_pad` si propone il campo:

`opposite_edge_fill`: **`allow`** oppure **`forbid`**.

- `allow`: il fill può appartenere al contenuto anche sul bordo opposto.
  Nessun errore di alignment viene dedotto da quel solo byte.
- `forbid`: se esiste contenuto diverso dal fill, il bordo opposto non può
  contenere il byte fill; in caso contrario si emette `E_ALIGNMENT`.
- Proprietà assente: equivale esplicitamente ad `allow`, per non introdurre
  retroattivamente divieti negli schemi formato 1 già approvati.
- Valori diversi, null o proprietà ripetute sono errori di schema, non
  violazioni del record. Restano attive le regole generali sulle chiavi duplicate.

Il default qui proposto richiede approvazione dell'autore: non è dedotto
dall'assenza della proprietà nel contratto precedente. Conserva quel contratto
senza aggiungere una restrizione sul bordo opposto. Non certifica requisiti
regionali di padding non ancora espressi nello schema.

Esempio sintetico di testo con spazi a destra e spazio iniziale vietato:

```json
{
  "kind": "right_pad",
  "fill_byte": 32,
  "content_policy": "strip_trailing_fill",
  "opposite_edge_fill": "forbid"
}
```

`kind` sceglie il lato; `fill_byte` identifica un singolo byte, non una classe
di whitespace; `content_policy` stabilisce il logical; `opposite_edge_fill`
stabilisce il vincolo fisico indipendente. Gli spazi interni restano ammessi.

Esempio sintetico di codice con zeri significativi:

```json
{
  "kind": "left_pad",
  "fill_byte": 48,
  "content_policy": "preserve_width",
  "opposite_edge_fill": "allow"
}
```

Il valore logical conserva l'intera larghezza. Gli zeri iniziali/finali non
diventano errori o assenze implicite. Per un numero con stripping degli zeri
iniziali si sceglie invece `strip_leading_fill`; il criterio sul bordo resta
una scelta separata, non viene dedotto dal tipo numerico.

Per `kind = exact` il campo `opposite_edge_fill` è **vietato**: non esiste un
lato di padding né un fill da controllare. Non si estende l'allineamento exact
delle date e non si aggiungono Builtin o relativi identificativi.

## 3. Predicato fisico deterministico

Precondizioni: schema compilato e coerente; slice del campo già disponibile
con larghezza esatta. Il record corto/lungo è gestito prima dalla pipeline.
Il confronto è sui byte originali, senza decodifica o conversione numerica.

Per RightPad/LeftPad:

1. Se tutti i byte sono fill, l'allineamento passa: non esiste contenuto
   osservabile su cui applicare il divieto del bordo opposto.
2. Con `allow`, nessun controllo aggiuntivo sul bordo opposto è richiesto.
3. Con `forbid` e RightPad, il primo byte deve essere diverso dal fill.
4. Con `forbid` e LeftPad, l'ultimo byte deve essere diverso dal fill.
5. Violazione di 3 o 4: un solo `FieldViolation` con codice `Alignment`.

Exact non impone un predicato di padding e passa questo stage. Ciò non
attesta larghezza, presenza o validità del contenuto: sono controlli distinti.
La slice vuota non rappresenta un campo compilato di larghezza valida; una
chiamata diretta deve comunque essere sicura e non generare panic. Il caso
all-fill comprende vacuamente la slice vuota, senza autorizzare record corti.

I fill interni possono essere contenuto, indipendentemente da `allow/forbid`.
Nessun carattere viene rimosso durante questo controllo. La policy vale anche
con `preserve_width`: conservare il logical non disattiva il vincolo fisico.

Il modello compiled proposto aggiunge una policy tipizzata Allow/Forbid ai
soli RightPad/LeftPad. Il compiler applica il default dichiarato quando il
campo raw manca e conserva la policy nel compiled immutabile. Non si deduce
la policy dal tipo, da fill=0/32, dall'ID del campo o dal flusso.

## 4. All-fill, presenza e logical

Il risultato fisico all-fill **non significa** che il campo sia presente,
opzionale o semanticamente valido. Rimane invariato l'ordine DET §40:

presenza sul raw → alignment fisico → datatype → vincoli semantici.

- MissingValuePolicy decide l'assenza sul raw; Required/Optional si applicano
  prima dell'allineamento. Un mancante non deve generare cascata di errori.
- NeverMissing può classificare un raw all-fill come presente. Il controllo
  fisico lo accetta, poi datatype e vincoli dichiarati decidono il resto.
- StripTrailingFill elimina solo la sequenza finale contigua di fill;
  StripLeadingFill elimina solo quella iniziale; all-fill produce slice vuota.
- PreserveWidth ed Exact conservano il raw. Non si esegue trim globale,
  conversione numerica, sostituzione del logical vuoto con zero o modifica dei
  byte usati dall'audit SHA-256.

Esempio: `0000` con NeverMissing e strip_leading_fill supera presenza e
alignment, ma produce logical vuoto. Un datatype Numeric che richiede almeno
una cifra non deve inventare 0; con preserve_width le quattro cifre restano
disponibili. La scelta corretta deve essere dichiarata dal profilo del campo.

## 5. Oracoli di accettazione sintetici da ratificare

`␠` rappresenta ASCII 20. Per tutti i casi la larghezza è corretta.
La colonna fisica riguarda soltanto alignment, non la validità del record.

| Lato / fill | Controllo | Raw | Esito fisico | Logical con stripping dal lato dichiarato |
|---|---|---|---|---|
| Right / spazio | forbid | `AB␠␠` | OK | `AB` |
| Right / spazio | forbid | `␠AB␠` | E_ALIGNMENT | Non fornito alla pipeline successiva |
| Right / spazio | forbid | `A␠B␠` | OK | `A␠B` |
| Right / spazio | allow | `␠AB␠` | OK | `␠AB` |
| Right / spazio | forbid | `␠␠␠␠` | OK, presenza separata | Vuoto se il controllo presenza permette di proseguire |
| Left / spazio | forbid | `␠␠12` | OK | `12` |
| Left / spazio | forbid | `␠12␠` | E_ALIGNMENT | Non fornito alla pipeline successiva |
| Left / spazio | forbid | `␠A␠B` | OK | `A␠B` |
| Left / zero | allow | `0010` | OK | `10` |
| Left / zero | allow | `1000` | OK | `1000` |
| Left / zero | forbid | `0010` | E_ALIGNMENT, divieto esplicito del bordo finale | Non fornito alla pipeline successiva |
| Left / zero | forbid | `0000` | OK, presenza separata | Vuoto se si prosegue |
| Right / zero | forbid | `0100` | E_ALIGNMENT, divieto esplicito del bordo iniziale | Non fornito alla pipeline successiva |
| Right / zero | allow | `0100` | OK | `01` |

Con PreserveWidth gli esiti fisici restano gli stessi, mentre il logical dei
casi OK è l'intero raw. Un campo in cui è lecito terminare con zero non deve
essere configurato come LeftPad zero + forbid: sarebbe una restrizione
normativa ulteriore, non una regola universale di formattazione numerica.

Test aggiuntivi obbligatori dopo l'approvazione: policy omessa -> Allow;
valore sconosciuto/null -> errore schema; policy su Exact -> errore schema;
fill non-ASCII con UTF-8 -> errore schema secondo il controllo già esistente;
byte NUL/non-UTF-8 trattati per identità byte nello stage alignment; codice
E_ALIGNMENT e metadati generati da ErrorFactory, senza duplicare i template;
assenza di allocazioni nei casi OK e input non modificato.

## 6. API, errori e confini

Rimane la firma semantica DET §44: validate_alignment(rule, value) restituisce
successo oppure FieldViolation. La policy è nel rule compiled; non occorre
passare un contesto normativo né modificare l'esito del run.

Il fallimento fisico usa ValidationCode::Alignment, expected descrive il bordo
richiesto, reason/context restano payload leggeri. ErrorFactory aggiunge
campo, posizioni, raw/diagnostica e messaggio stabile. Nessun nuovo codice
normativo, log SIOC/CAD o payload applicativo dentro il validator.

Un piano incoerente (per esempio stripping dal lato opposto) è un errore
tecnico di schema/compiler, non E_ALIGNMENT del paziente. Il compiler deve
continuare a costruire solo combinazioni coerenti. Un eventuale helper pubblico
di estrazione deve mantenere distinto questo errore tecnico; non viene
autorizzata una conversione in FieldViolation per aggirare il suo tipo.

Il compiler non pubblicizza completa local.v1: resta valida CNF-COMPILER-01
per il sottoinsieme incrementale. Builtin resta non esprimibile nel formato
approvato, con rifiuto esplicito al caricamento; nessun executor fittizio.

## 7. Versioning, compatibilità e documenti da adeguare

Si propone un'estensione **additiva e opzionale del formato 1**, limitata a
un parametro dell'operazione padding già in local.v1. Non si introduce una
capability nuova né si inventano formato 2 o versioni future del motore.

- I documenti preesistenti senza proprietà conservano il significato di
  content_policy e acquisiscono soltanto il default Allow qui da approvare.
- I vecchi parser hanno deny_unknown_fields: devono rifiutare i documenti
  con la nuova proprietà, senza ignorare un divieto. Il supporto del numero
  di formato non garantisce supporto di ogni costrutto, come già dichiarato
  dalla mappa e dalla deroga di compilazione incrementale.
- La baseline storica 0.1.0/b658e05 resta tale, non viene reinterpretata come
  supporto del nuovo parametro. Il supporto effettivo verrà collegato al commit
  implementativo e alla sua CI, prima di distribuire schemi che lo richiedono.
- I byte degli schemi regionali esistenti e i relativi hash non cambiano.
  Questo contratto non assegna automaticamente forbid ai campi SIOC/CAD:
  una revisione normativa separata deve dichiarare i loro vincoli necessari.
  Non si certifica che il precedente profilo minimale copra già ogni requisito
  fisico regionale solo perché il motore ora dispone del predicato.

Solo se l'autore approva questo documento, l'implementazione completa di
T-P6-03 potrà adeguare: SCHEMA_AUTHORING nella sezione padding;
flow.schema.json; note di lowering e compatibilità; raw generato (con lo script
già previsto), compiled, compiler/lint, validator, export e test sintetici.
L'approvazione deve comprendere il cambiamento dei payload enum pubblici
RightPad/LeftPad: il parametro tipizzato aggiunto richiede l'aggiornamento dei
costruttori e pattern di test/chiamanti. Nessuna responsabilità cambia modulo.

ARCH, DET e Master Plan originali restano immutati; l'adattamento viene
collegato come decisione contrattuale approvata. Non sono autorizzate modifiche
a baseline normative, reference dataset, storage, UI o task successive prima
del completamento verificato di T-P6-03. Poi torna la progressione autonoma
generale, se non emergono altri blocchi.

## 8. Decisione richiesta e stato delle evidenze

Si richiede approvazione del contratto completo, inclusi: proprietà esplicita,
default Allow per compatibilità, trattamento all-fill, tabella degli oracoli,
estensione additiva del formato 1 e adeguamenti contrattuali elencati sopra.

Fino all'approvazione: T-P6-03 BLOCKED, nessun Rust di alignment, nessun
parser/compiled/compiler o schema modificato; nessuna prova di conformità
fisica dichiarata. Questa sessione produce solo proposta, decision log e
handoff. La scelta B è registrata; non equivale ad approvazione dei dettagli
qui proposti. Se un punto non è condiviso, si rivede questa proposta prima
di implementare il validator completo, senza tornare alla consegna parziale A.
