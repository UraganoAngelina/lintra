# CAD 2024-06.calendar-age.1 — T-CAD-09

Componente esplicito A/B1 per le condizioni d'età confermate dalle note originali:
1.24 richiesto con anno(inizioB1.2.4)−anno(nascitaA.1.11)<=17;2.17 da3anni;
2.18 da6anni. Non usa anni compiuti, compleanno o data dell'host. Le due date
DDMMYYYY devono essere parseabili e gregoriane valide prima del confronto.

Il JSON conserva campi legacy/provenance/hash e coordinate originali. Le presence
locali restano quelle ricevute: optional per i3target, domini indipendenti sempre
verificati se valorizzati. FieldPresent in regole di passata2 impone le soglie;
essuna forbidden aggiunta per adulti o bambini sotto soglia. I12byte numerici
restano exact, zero-fill mancante solo dove dichiarato, non trim/coercizione.
I costrutti legacy digits_only/if_present sono abbassati esplicitamente a regex
rawASCII di larghezza esatta; le date usano calendar_valid:true canonico.
Il compiler non accetta per questo i costrutti legacy nei vecchi schemi.

B1 leggeA.1.11 con join scalare sull'evento;A leggeB1.2.4 con consensus della
data di inizio su tutte le valutazioni dell'evento, senza scegliere la prima.
Date discordanti anche nel medesimo anno restanoUnknown con diagnostica dichiarata.
Namespace e bindingA.1.1 condividono sending_ulss, come ratificato inCAD02.
A/B1 hanno le PK normative complete, e il contesto non riceve parent invalidi
per namespace. Tutte le correlazioni di questo componente sono current_submission.

Il compiler completa soltanto calendar_year_delta conRefdate esterne già previste
dall'authoring. Substring eage_in_completed_years restano locali. I riferimenti
esterni devono essere scalari o compresi nella tupla consenso; niente set usati
come scalar, date in campi numerici, piani in presence locale o predicati set che
richiederebbero un altro join. Le API pubbliche rimangono invariate.

LocalOnly controlla i campi locali e conta i controlli crosssaltati/Partial;
Full esegue il derivato sui campi minimi proiettati nella seconda passata.
Missing/invalid/ambiguous non diventanoFalse o una data fittizia. Il motivo join
decisivo segue il derivato per applicare on_missing_record/on_missing_trace;
falseANDunknown/trueORunknown conservano il dominio booleano approvato.

Questo non è un profiloCAD completo oFROZEN, né un'esecuzione storica o temporale.
Valutazioni/fonti storiche mancanti non vengono inventate. Le altre regole cliniche,
reconciliation e vista storica approvata restano alle task pertinenti. Un Full con
fonti crosssorgente totalmente assenti conserva i vincoli di preflight correnti;
i test di targetassente usano gruppi di regole esplicitamente scopiati, non
rimuovono obblighi dichiarando falsa conformitàCAD globale.

Test: tests/cad_age.rs usa il compiler eValidationService reali con record
sintetici alle coordinateA/B1 e filler fuori scope. Copre soglie2/3/5/6/17/18,
proiezioni minime, consenso, date invalide/missing, Aassente contro vuoto,
namespace/duplicati, LocalOnlyPartial, boolean dominance, guardie compilatore,
rinomina ID eJSONL identico. Nessun outcomeFull di fixture creato a mano.
