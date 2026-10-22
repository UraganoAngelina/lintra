# Decimal per importi — chiarimento dell'autore e proposta contrattuale

2026-10-22. **APPROVATA dall'autore:** «approvo questa estensione, procedi pure».
La ratifica comprende l'estensione dati/versionamento e gli aggiornamenti
contrattuali qui elencati. Il testo originario seguente conserva la proposta
approvata; stato implementativo e verifiche sono nel dossier T-P9-03.
Nessuna migrazione di baseline normative autorizzata.

## Decisioni ricevute dall'autore

1. I decimali nei tracciati rappresentano importi monetari diretti, comprese
   eventuali quote monetarie di compartecipazione. La regola vale per tutti i
   tracciati, non è una condizione SIOC-specific del motore.
2. La parte frazionaria è sempre valorizzata con esattamente due cifre.
3. Il separatore fisico è il punto ASCII, al terzultimo byte del campo.
4. Nessun separatore delle migliaia.
5. Nessun segno + o - memorizzato; valori negativi vietati.
6. Zero ammesso: semantica di importi non negativi, non strettamente maggiori di zero.
7. Chiarimento ulteriore: campi di compartecipazione con codici/range già
   dichiarati conservano tipo e vincoli. Decimal solo dove è memorizzato
   direttamente un importo, mai conversioni per nome o argomento del campo.

La citazione operativa è stata fornita in chat dall'autore; la sua applicabilità
trasversale, la policy del segno e dello zero sono conferme esplicite dell'autore.
Non si attribuisce a un PDF letto dall'agente una verifica non eseguita.

## Rappresentazione proposta per il profilo importi

Campo presente di larghezza N byte, con almeno una cifra della parte intera:
- N almeno 4; punto in posizione one-based N-2, indice zero-based N-3;
- N-3 cifre ASCII della parte intera;
- un punto ASCII 0x2E;
- due cifre ASCII finali, comprese quelle zero;
- nessun segno, spazio interno, raggruppamento, esponente o cifra Unicode.

Gli zeri iniziali sono cifre e non cambiano il valore. La parte intera non è
sottoposta a un limite di macchina. La larghezza viene dal campo nello schema,
non da un valore globale. Per questo profilo si propone alignment exact e
rappresentazione a larghezza preservata: nessuno stripping può spostare il punto
rispetto al terzultimo byte fisico. Sono regole del nuovo nodo Decimal,
non una revisione della ratifica CNF-ALIGNMENT-01 per Numeric/AN esistenti.
MissingValuePolicy/presenza restano esplicite e precedono il datatype: non si
tratta 0.00 come assenza e non si introduce una policy globale di campo vuoto.

Esempi sintetici per campo da 7 byte:

| Byte del campo | Esito | Motivo |
|---|---|---|
| `0012.34` | valido | due decimali e posizione fisica corretta |
| `0000.00` | valido | zero esplicitamente ammesso |
| `1234.00` | valido | frazione sempre presente |
| `0012,34` | invalido | separatore errato |
| `0012.3 ` | invalido | seconda cifra frazionaria assente |
| `012.340` | invalido | tre decimali, punto non al terzultimo byte |
| `-012.34` | invalido | segno vietato |
| `+012.34` | invalido | segno vietato |
| `1,23.45` | invalido | separatore nella parte intera |

## Integrazione proposta, senza conversioni implicite

- Il motore valida da DecimalSpec compiled verificata: larghezza, scala,
  separatore e policy senza segno provengono dal nodo dichiarato. Il profilo
  monetario dichiara scala 2, punto e assenza di segno; niente if flow == SIOC,
  ricerca della parola importo/compartecipazione o elenco di ID nel motore.
- Esempio indicativo del nuovo physical per un campo sintetico:

```json
{
  "type": "decimal",
  "start": 1,
  "length": 7,
  "scale": 2,
  "decimal_separator": ".",
  "sign_policy": "unsigned",
  "alignment": { "kind": "exact" }
}
```

- L'esempio è una proposta di AST, non uno schema già accettato dal compiler.
  La validazione del profilo monetario deve dichiarare i parametri esplicitamente,
  senza ricavarli dalla locale o dal contenuto del record.
- Nessun floating point, arrotondamento, rimozione di caratteri o normalizzazione
  correttiva dell'input. Se serve un valore aritmetico, mantissa intera esatta
  e scala esplicita; nessuna conversione degli importi in interi troncati.
- Le incompatibilità schema/tipo falliscono nel compiler. Non si applica
  automaticamente NumericRange a centesimi o parte intera di un Decimal:
  quel significato richiederebbe un contratto esplicito. I range attuali dei
  campi Numeric restano identici.
- Violazione della rappresentazione Decimal: diagnostica datatype normativa
  attraverso il percorso esistente; nessun nuovo stato di run o errore tecnico
  simulato. Mancanza/ambiguità dei parametri: errore di schema prima del motore.

## Versionamento da ratificare

Il formato 1 attualmente esclude Decimal con enum chiusi. Si propone di
preservarlo integralmente e introdurre un formato 2 che aggiunga il nodo
Decimal esplicito; il motore continuerà a leggere il formato 1 invariato.
La mappa formato/eseguibile dovrà descrivere l'implementazione realmente
verificata, senza inventare una release o un commit prima della consegna.
La futura approvazione deve coprire meta-schema/authoring/lowering, modello
raw/compiled, DET §137, mappa compatibilità e prompt T-P9-03 pertinenti.
Nessun JSON normativo esistente viene migrato o riscritto con questa decisione.

È una proposta di evoluzione del contratto, non una ratifica già ricevuta del
numero 2: restano valide CNF-FORMAT-01 e la regola di non modificare le fonti
senza autorizzazione. Lo scope implementation dovrà essere riletto contro
questa estensione prima di dichiarare T-P9-03 sbloccata per il codice.

## Protezione dei campi esistenti

Il caso SIOC 2.11 già ispezionato è un codice del soggetto pagante, con dominio
1–4: non è l'importo e resta invariato. La stessa distinzione vale per ogni
campo con categoria/range di compartecipazione. Non si modifica alcuna baseline
normativa, neppure per sostituire Numeric con Decimal in campi apparentemente
monetari: occorrono identificazione e aggiornamento normativo espliciti separati.

## Verifiche previste dopo ratifica

Test di compiler per nodo/parametri/tipo/versioni e non-forgiabilità;
validator e pipeline per punto fisico, due cifre, segni, zero, width e cifre
ASCII, nessuna cascata dopo presenza/datatype; test CLI/golden sintetici;
regressioni dei range/codici esistenti e confronto di integrità delle baseline.
Gate locali e CI Linux/Windows prima di DONE. La proposta documentale non è
un test runtime e non implementa alcuna capacità.
