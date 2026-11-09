# ADR-HISTORY-EXISTENCE-001 — L1: invalidazione del testimone annullato

Decisione dell’autore ricevuta il **9 novembre 2026**.
Stato: **RATIFICATO — annullamento invalida la prova storica**.

## Fonte e significato

Testo dell’autore:

> L'annullamento significa che non posso avere un record con rivalutazione per quel paziente ma dovrò avere un altro record con valutazione iniziale in quanto la precedente presa in carico è stata annullata

Questa precisazione esclude l’alternativa B proposta nel dossier L1: il record
annullato resta conservato nello snapshot immutabile per audit, ma non è un
witness valido per soddisfare il requisito di esistenza. Serve un altro record
valido di valutazione iniziale. Una rivalutazione da sola non sostituisce tale
prova. La conservazione fisica non mantiene la validità normativa del witness.

Il predicato resta correlato al namespace e all’identità dell’evento già previsti
da DEC-05 e dal piano compilato. Il riferimento al paziente nella spiegazione
non introduce una correlazione globale fra tutti i suoi eventi e non modifica
PK, progressivi o regole di annullamento degli altri tracciati.

## Applicazione al caso L1

- T1 approvata: evento E, P10 tipo 1. La prova è utilizzabile finché valida.
- T2 con B1 presente e completo, P10 omesso e quindi ANNULLATO secondo CAD04/05:
  P10 non contribuisce più al predicato. Se T2 contiene soltanto P20 tipo 2,
  manca una prova positiva valida; occorre un’altra iniziale valida.
- Una correzione esplicita di P10 da tipo 1 a tipo 2 non consente di riutilizzare
  il precedente tipo 1 come witness. Questo effetto era comune alle due
  alternative sottoposte all’autore; non si sceglie una vecchia versione per
  aggirare la rettifica.
- B1 interamente Absent non annulla implicitamente i record: si conserva il
  contratto Present/Absent già ratificato. PROCEED non prova la copertura.
- Un’altra iniziale valida può soddisfare il requisito; non si impone una
  nuova iniziale ogni trimestre quando il testimone precedente resta valido.

## Conseguenze tecniche e limiti

La query deve considerare lo stato qualificato risultante dal lifecycle, non
fare un OR indiscriminato di tutti i positivi mai persistiti. Un witness
annullato o rettificato non viene recuperato da uno snapshot precedente per
produrre True. Snapshot e audit storici rimangono immutabili.

La mancanza di un witness valido non autorizza False senza copertura completa
qualificata: in assenza di tale copertura resta Unknown, mai conformità completa.
C1 è separata ed è già approvata. Corruzione, chiavi e I/O rimangono errori tecnici.

Decisione normativa registrata, **non qualifica implementativa**: il runtime
storico, la composizione CAD83, i test applicativi e i Quality Gate sono ancora
necessari per T-CAD-12 DONE. Nessuna chiusura implicita di P18-04 o M9.
