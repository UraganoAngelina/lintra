# CAD 2024-06.clinical-lookups.1 — T-CAD-11

Componente versionato per i lookup clinici e le condizioni di presenza approvate
CAD10/16, CAD11 eG05. Non è uno schemaCAD completo oFROZEN. Non attiva persistenza,
storico, età, temporali, lookupISTAT/ICPC o dati diversi da quelli dichiarati.

component.json contiene campi B1 2.3/2.27–2.33 e D5.5 alle coordinate originali,
presenza/note sorgente preservate, condizioni nominate e dipendenze reference
esplicite. source conserva byte-per-byte i tre dataset reviewed forniti dal kit;
provenance/hash delle tracce/PDF/dataset e campi legacy selezionati sono nel JSON.

I due file data sono conversioni esplicite al formato membership v1 ratificato:
63ICD9 della pagina7,33prestazioni della pagina5, SHA256 attesi nelle dipendenze.
Gli ICD9 rimuovono il solo punto dal codice sorgente e mantengono i prefissiV;
le prestazioni sono01–28,71–74,99, senza inferire un intervallo01–33. Gli zeri
iniziali sono parte del codice trasmesso. Nessun fallback o codice aggiuntivo.

Il catalogo di70posizioni della pagina6 resta sorgente informativa del bitsetT70,
la cui policy eseguibile è già nel componente2024-06.t70.1. Non si appiattisce
quel catalogo a un lookup inventato sui valori del record.

Il lowering usa primitive già approvate: exact per numerici di1/2byte e per
codici prestazione di2byte; right_pad ASCII32/strip_trailing_fill per i campiICD
alphanumeric di5byte. La membership consulta il logical così dichiarato.
Non rimuove punti dall'input, non converte stringhe in numeri, non cambia case.
Gli allowed_values dei controllori sono raw; le liste membership contengono
soltanto le chiavi trasmesse esatte. I tipi numerici e i domini enumerati rendono
superfluo il costrutto legacy digits_only_if_present, che non viene accettato
implicitamente dal compiler o tradotto negli altri schemi.

B1:2.27 richiesto solo inUCPDOM;2.28 richiesto se2.3=2 e2.27=1 e vietato altrimenti;
2.29 richiesto inUCPDOM;2.30–2.33 facoltativi inUCPDOM e vietati fuori. Si usano
le condizioni dichiarate dal candidato, non controlli clinici nuovi nell'engine.
Campi mancanti seguono la presence prima della membership. Il comportamento
Unknown/degraded e la separazione guasti tecnici/violazioni restano quelli diP11.

Qualificazione in tests/cad_lookups.rs: providerfilesystem e ValidationService
reali, record sintetici B1/D con filler espliciti fuori scope,63codici in5slot,
33prestazioni incluse71–74/99, condizioni required/FAC/forbidden, controprove
membership byteesatto e audit315lookup, sorgenti unavailable/degraded eJSONL
ripetibile. L'identità del flow di test dichiara il perimetro di componente;
non si inventano summary o risultatiFull cliniciCAD.
