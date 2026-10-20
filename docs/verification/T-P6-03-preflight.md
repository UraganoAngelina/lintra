# T-P6-03 — Preflight / CNF-ALIGNMENT-01

2026-10-20. **RISOLTA — mandato finale dell’autore: sola estrazione logical, nessun residuo fisico formato 1.**

## Decisione finale — prevale sulla cronologia sottostante

L'autore ha scelto A con precisazioni vincolanti: sola estrazione del logical,
nessun validate_alignment/FieldViolation/E_ALIGNMENT, controllo fisico fuori
ambito per formato 1 e **nessun residuo**. Autorizzate revisioni del prompt e
DET §44; poi T-P6-04. CNF-ALIGNMENT-01 archiviata come possibile estensione
futura non implementata. Non introdurre opposite_edge_fill o il suo default.
La proposta parziale A e la proposta fisica B sottostanti sono storiche;
il contratto vigente e le evidenze sono in [T-P6-03.md](T-P6-03.md).

## Decisione successiva dell'autore

Ricevuta la scelta **«B: definiamo prima il contratto fisico»**.
La proposta A sotto resta storica e non approvata; non è autorizzata la
consegna parziale né la prosecuzione con il residuo fisico aperto.
Preparata la [proposta completa di contratto fisico](../proposals/alignment-physical-contract.md),
ancora DRAFT da approvare prima del codice. Le fonti approvate non sono
state modificate e T-P6-03 rimane BLOCKED durante la definizione.

Avvio autonomo dopo T-P6-02 verificata, secondo l'ultimo mandato dell'autore.
Prerequisito T-P6-01 riconfermato in questa sessione: CI 39389225120,
46ab648f8ad96458cf4a7dbfef83a93f68bd7be1, Linux/Windows completed/success.
Stato Git: work/p1-foundations; modifica storica al piano Superpowers estranea.

## BLOCCO

**Descrizione.** La task richiede un validator con casi di padding corretto e
errato. Il formato approvato definisce da quale lato rimuovere il fill per
ottenere il logical, ma non un predicato generale che distingua il fill usato
come dato da quello usato come padding. Sono due responsabilità diverse.
Con soli AlignmentRule e FieldSlice non esiste sempre evidenza per dichiarare
un errore fisico: dedurlo dal primo/ultimo byte può rifiutare valori legittimi.

**Fonti.**

- Prompt T-P6-03: validate_alignment, test padding corretto/errato, niente
  conversione numerica, dominio o regole dedotte dal solo tipo.
- DET §15: modello RightPad/LeftPad/Exact/Builtin e semantica definitiva
  dipendente dai tracciati; nessuna inferenza di alignment dal tipo.
- DET §44: validate_alignment(rule, value) -> Result<(), FieldViolation>.
- SCHEMA_AUTHORING, «Raw, logical, padding e assenza»: strip dal lato
  dichiarato oppure preserve_width, mai trim globale o rimozione indiscriminata
  di zeri; logical vuoto non viene sostituito con zero.
- flow.schema.json, `$defs.alignment`: kind, fill_byte e content_policy;
  nessuna lunghezza del contenuto, regola sui bordi o distinzione fill/dato.
- lowering.json, structure_only: controlli fisici/padding, non un executor
  semantico che non esegue alcun controllo.
- MIP §14.2/§14.4: conflitto/informazione insufficiente -> arrestare la parte
  interessata e chiedere una decisione, senza modificare le fonti autonome.

## Esempi sintetici che mostrano il limite

`␠` indica il byte ASCII 20; non sono dati regionali o sanitari.

| Regola / raw | Logical già definito dal contratto | Cosa non si può dedurre |
|---|---|---|
| RightPad spazio, strip_trailing_fill / `AB␠␠` | `AB` | Nessun conflitto per l'estrazione. |
| RightPad spazio, strip_trailing_fill / `A␠B␠` | `A␠B` | Lo spazio interno può essere contenuto; vietare tutti i fill interni rifiuterebbe nomi/testi con spazi. |
| RightPad spazio, strip_trailing_fill / `␠AB␠` | `␠AB` | Per vietare lo spazio iniziale occorre una regola dichiarata: encoded_text non lo vieta. |
| LeftPad zero, strip_leading_fill / `0010` | `10` | Uno zero finale può essere contenuto: non è automaticamente padding sul lato errato. |
| LeftPad zero, strip_leading_fill / `1000` | `1000` | Gli stessi byte sono valore 1000 senza padding oppure valore 1 riempito sul lato errato. Il validator non conosce il valore originariamente inteso. |
| LeftPad zero, preserve_width / `0010` | `0010` | La policy impone di conservare anche gli zeri iniziali. |
| LeftPad zero, strip_leading_fill / `0000` | Slice vuoto | Assenza fisica è già decisa da MissingValuePolicy sul raw; non si può inventare zero o RequiredField dopo stripping. |
| Exact / qualsiasi raw della larghezza prevista | Raw invariato | Larghezza e disponibilità dello slice sono della pipeline record, non un dominio semantico inventato. |

Il controesempio `1000` non si risolve con un algoritmo migliore: due intenzioni
diverse generano gli stessi byte. Un oracolo di test che lo dichiarasse sempre
padding errato introdurrebbe una regola non presente nel contratto.

`Builtin` è un confine separato già esplicito: il formato 1 non lo ammette e
raw/compiled non lo espongono. Non serve inventare un ID o un executor per
sbloccare questo punto. Non è questo il motivo del blocco.

## Impatto

Non implementare un validate_alignment che rifiuti zeri/spazi con euristiche,
né uno che restituisca sempre Ok fingendo di avere verificato padding errato.
T-P6-03 resta BLOCKED; nessun nuovo test normativo può essere derivato
esclusivamente dal comportamento che sceglieremmo di implementare.

T-P6-04 ha prerequisiti atomici T-P6-01 e T-P0-07, non T-P6-03. La prosecuzione
con un residuo esplicito richiede però chiarimento rispetto all'ultimo mandato
di avanzare solo in assenza di punti aperti nelle task precedenti.
T-P2-03/M1 conservano il proprio residuo distinto, già autorizzato.

## Proposta A — consegna incrementale circoscritta, non approvata (scelta B)

1. Implementare nella parte autonoma di T-P6-03 **soltanto l'estrazione del
   logical**: helper zero-copy da AlignmentRule e FieldSlice a slice preso in
   prestito, con rimozione del solo fill dal lato esplicitamente dichiarato,
   preserve_width/exact invariati, all-fill -> slice vuoto, nessun parsing.
2. Se una AlignmentRule costruita direttamente accoppia lato e stripping
   incompatibili, restituire un errore tecnico tipizzato di configurazione,
   non una violazione normativa. Il compiler attuale costruisce solo coppie
   compatibili. Non si modifica schema, formato o baseline per il nuovo helper.
3. Non dichiarare implementata la verifica fisica del padding e non fornire
   un validate_alignment privo di predicato reale. T-P6-03 resta IN_PROGRESS
   con residuo esplicito finché il contratto dei controlli fisici è approvato
   e verificato. Il helper viene testato su byte letterali indipendenti.
4. Autorizzare la prosecuzione delle sole task con prerequisiti atomici
   soddisfatti, iniziando T-P6-04, senza dichiarare complete T-P6-03 o milestone
   che dipendono dal suo residuo. Ogni consegna mantiene verifiche e CI.

Questa proposta cambia la consegna della task e l'ordine rispetto al residuo;
serve una decisione esplicita, non un adattamento silenzioso di signature.
Non cambia la semantica normativa e non autorizza revisioni autonome di ARCH,
DET, Piano o schemi dei flussi.

## Alternativa B

Rivedere prima il contratto fisico: stabilire quali vincoli espliciti di bordo,
contenuto o lunghezza rendono osservabile l'errore e come sono dichiarati nello
schema, con esempi validi/invalidi. Solo dopo implementare il validator completo.
Nessuna nuova proprietà, versione del formato o norma viene scelta dall'agente.

## Evidenze della sola attività documentale

Consultati fonti e codice raw/compiled/compiler/lowering; nessun Rust scritto
per T-P6-03, nessuna nuova esecuzione Cargo rivendicata. Test e CI di T-P6-02
sono registrati nel report relativo, non usati come prova di T-P6-03.
File di questa attività: questo dossier e IMPLEMENTATION_STATUS.md.
Scelta B ricevuta: ora attesa ratifica/revisione del contratto fisico proposto,
non una nuova scelta A/B. Nessuna implementazione parziale avviata.
