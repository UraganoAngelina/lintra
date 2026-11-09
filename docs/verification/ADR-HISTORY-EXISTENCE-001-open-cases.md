# ADR-HISTORY-EXISTENCE-001 — due casistiche materiali, 8 novembre2026

La ratifica implementativa estesa è ricevuta. Nessuna seconda approvazione ordinaria
per codice/codec/test. Le seguenti sono esclusivamente questioni normative di validità
e origine della copertura, richiamate espressamente dai punti7 e11 della ratifica.
Il lavoro indipendente su piano, fatti, integrità e transazioni prosegue.

## L1 — iniziale poi assente da un file B1 completo

**Decisione ricevuta il 9 novembre 2026:** annullamento invalida il witness;
alternativa B esclusa. [Ratifica e applicazione](ADR-HISTORY-EXISTENCE-001-L1-ratification.md).
Il testo seguente conserva la proposta precedente per tracciabilità.

T1approvato: ULSS501/eventoE, progressivoP10, tipo1(iniziale).
T2: B1Present completo contiene soloP20,tipo2(rivalutazione); P10non compare.
CAD04/05 considera P10ANNULLATO nella vista corrente, pur conservando T1immutabile.
DEC-05richiede lo storico dell'evento e non una nuova iniziale pertrimestre.
Le due norme non dicono esplicitamente se il fatto storico diP10sia ancora un
witnessvalido: conservazione fisica dello snapshot non basta a deciderlo.

Caso distinto: T2riusa lo stessoP10/corretto tipo2, stessa identitàevento.
La correzione di attributi è ammessa, ma non è stabilito se la precedente
classificazione tipo1debba ancora contribuire al predicato storico.

A: annullamento o correzione del tipo invalidano entrambi il witness precedente;
nessuna prova positiva daP10. RestituireFalse solo se universo qualificato completo,
altrimentiUnknown. Una nuova iniziale corrente o altra iniziale ancora valida può
soddisfare la regola. La vista record04non cambia.

B: l'omissioneP10dafile completo annulla il record corrente ma non la prova che
l'iniziale approvata sia esistita; la correzione esplicita dello stessoP10tipo1→2
invalida invece quella prova. Anche qui nessuna modifica al significato di record
ANNULLATO o all'immutabilitàT1; si precisa soltanto il predicato sullo storico.

Alla presentazione nessuna alternativa era scelta implicitamente dal codice.
La successiva precisazione dell’autore decide ora l’invalidazione del witness annullato. Non è una
ripetizione della domanda «storico oppure ogni trimestre», già risolta.

## C1 — origine verificabile per la prova negativa

**Alternativa A approvata.** [Ratifica C1](ADR-HISTORY-EXISTENCE-001-C1-ratification.md).
Le alternative sottostanti sono conservate come storia della proposta.

Un archivio vuoto o conprima sessioneT1non prova l'origine reale dell'evento.
Nessun campo/flag nuovo può dichiarareComplete per default.

Proposta A: accettare una attestazione di origine storica esplicita e qualificata,
separata dalla presenza del file trimestrale. Deve nominare namespace/eventoprotetto,
periodo d'origine e perimetro target/predicato, fonte documentale/input verificabile
con hash, soggetto/responsabilità dell'attestazione e decisione di approvazione.
Il relativo oggetto deve essere collegato allo snapshot e sigillato con i meccanismi
di proiezione protetta già approvati, nella stessa transazione; non primaDB=origine.
La verifica automatica qualifica binding/integrità/provenienza, non inventa né
certifica la verità clinica esterna. Un'attestazione senza fonte qualificata o non
approvata resta insufficiente. Nessuna ricostruzione/migrazione automatica di archivi.
Con anchorqualificata, tutte le sessioni necessarie coperte e nessun positivo è
possibile dimostrareFalse. Senza tali condizioni restaUnknown.

Alternativa B: non ammettere perora una fonte di origine affidabile nel prodotto.
Sono eseguibili positivi storici/correnti qualificati; i negativi privi di anchor
restanoUnknown. Il casoFalse su universo completo resta un oracle di modello,
non viene presentato come integrazione applicativa già qualificata. Non si può
promettereCAD12DONE per un gate di copertura o test non realmente soddisfatto.

La scelta della fonte accettabile è un fatto di dominio/trust, non una scelta di
SQL o di algoritmo. Nessun nuovo schema crypto o distribuzionechiavi è proposto.
