# T-CAD-11 — lookup e condizioni cliniche, 6 novembre 2026

IN_PROGRESS / WAITING_GATE. Preflight11 documenta prerequisiti P11/P10/G0 e fonti lette.
CAD03 attende CI42190202683 sul proprio42f92ed; il presente delta è indipendente.

## Componente, sorgenti e trasformazioni esplicite

schemas/cad/components/2024-06.clinical-lookups.1 conserva tre dataset reviewed
byte-per-byte in source e le copie dei9campi legacy selezionati incomponent.json.
Hash PDF/tracce/dataset e decisioni CAD10/16/11/G05 sono espliciti. I63codici
sorgenteICD9 sono stati riscontrati letteralmente nella pagina7 del PDF usando
pdftotext -layout; risultato in T-CAD-11-source-check.json. Non si dichiara una
nuova revisione delle descrizioni cliniche né un dataset nazionale completo.

I file data sono membership v1 ratificata: coppie dataset_id/version, keys_hex
byteesatto eSHA256 atteso nelle due reference_data. I63ICD9 normalizzano solo
il punto del codice sorgente, preservando7prefissiV; i33codici prestazione sono
01–28,71–74,99. Nessun intervallo01–33, wildcard, fallback o lookupISTAT/ICPC.
Il catalogo70posizioni è conservato come fonte delT70 già implementato inCAD10,
non inventa una terza dependency di membership su valori del record.

Le8definizioniB1 (2.3,2.27–2.33) eD5.5 conservano coordinate, presenza e note
regolatorie esatte. Il lowering fisico è dichiarato nel README: exact sui
numerici di1/2byte; right_pad32/strip_trailing_fill sui codiciICD di5byte.
I domini dei controllori sono raw; le membership usano il logical dichiarato.
Non si rimuovono punti dall'input, non si cambia case o si convertono codici
alphanumerici in numeri. Gli zeri prestazione restano significativi. Nessun
nuovo costrutto AST, validator normativo, HMAC, schemaSQL o dipendenzaCargo.

Il compiler risolve le condizioni nominate già approvate: is_ucpdom (2.3=2)
e belongs_to_network (2.27=1). Presence2.28 usa entrambe, required in quel solo
caso e forbidden altrimenti;2.29 richiesto inUCPDOM;2.30–33 FAC inUCPDOM e vietati
fuori. Presence precede membership, quindi un missing richiesto resta errore
primario, non un secondo errorelookup. Il provider conserva i comportamenti
Verified/Degraded/Unavailable già ratificati e non legge file nel loop record.

## Funzioni dei test e correttezza

In tests/cad_lookups.rs, component/source leggono artefatti versionati; model
assembla un flow con identitàCAD-LOOKUP-COMPONENT-TEST e filler espliciti che
coprono i gap delle coordinateB1/D reali. La PK sintetica serve al compiler,
non pretende di sostituire la PK normativa. I campi fuori scope non sono validati.
Non si produce un finto schemaCAD completo oFROZEN. I dati sono solo sintetici.

Fixture possiede una directoryRAII nel target e vi copia soltanto i2dataset
membership. service costruisce repository/registry/compiler e providerfilesystem
reali, con policy esplicita. run scrive un file di test, usa ValidationService
LocalOnly streaming e verifica la scrivibilità del summary reale. Gli outcome
non sono fixture costruite a mano. set lavora con offsetone-based convertito
una volta in slice; la lunghezza del literal è verificata prima di copiare.
b1/d creano record della larghezza reale198/39; jsonl usa il sink canonico.

I7test verificano:

- Inventario63/33/70, integrità SHA dei3source e2membership, domini esatti,
  normalizzazione punto sorgente e prefissiV; campi lowered mantengono ID,
  start/length/presence e metadata ricevuti. Solo2dependency, nessunaISTAT/ICPC.
- Tutti63ICD9 nei5slot2.29–33:315lookup definitivi nell'audit reale, Complete,
  nessun errore. Controprova fuori dataset in ogni slot, diagnostico preciso.
- Tutte33prestazioni ammesse aD5.5 e codici00/29/34/75/space1/A1 rifiutati.
- SintomiFAC possono mancare inUCPDOM, possono esservi presenti, sono vietati
  fuori; sintomo1 mancante inUCPDOM produce ConditionalRequired primario.
- Matrice rete:2.28 required solo2.3=2/2.27=1, forbidden negli altri casi validi;
  assenza/presenza confrontate con l'attesa derivata dalle due condizioni.
- InputICD dotted/lowercase/zero-coercion/nonenumerato rifiutati senza correzioni;
  codici normalizzati di fonte validi; errori JSONL ripetuti byteidentici/LF.
- SHA errato con file conforme: Strict errore tecnico, Continue match provvisorio
  Partial. File assente: Strict errore tecnico; Continue resta Partial senza
  spacciare membership indisponibile per violazione normativa.

## Tentativi preliminari

preliminary-first.log: due erroriRust nelle fixture (Path/PathBuf e LowerHex
non implementato dal digestsha2 della versione effettiva). Corrette le chiamate
con PathBuf e il formatter Sha256Digest del dominio, senza cambiare librerie.
preliminary-second.log:6testpass/1fail, perché99 era stato erroneamente indicato
come negativo nel test. La fonte revieweda p5 contiene99; la membership già lo
accettava correttamente. Corretto l'oracle ai codici realmente fuori lista,
conservando il test positivo di tutti33codici. Nessuna modifica al comportamento
prodotto per accomodare l'attesa sbagliata. preliminary-third.log:7/7pass.
Dopo quel run aggiunti controlli SHA/source/coordinate e315conteggi nel test;
i gate finali verificano il sorgente aggiornato, con hash prima/dopo.

## Gate e limiti

Gate locali in T-CAD-11-local-gates.json e log T-CAD-11/gate-N.log. La suite
workspace precedente diCAD03 (922esiti positivi/0falliti/6ignored) resta evidenza
della base produttiva; non viene rietichettata come nuova suite completa11.
Il delta11 riguarda soltanto test/componente/dataset/docs: gate mirati sul delta
e regressioni pertinenti, poi nuovaCI nativa deve coprire l'intero workspace.
Non ripetuta localmente la release già positiva sulla base a codice produttivo
invariato; release eworkspace completi saranno ricontrollati dallaCI11.

Nessuna validazione clinica/storicaCAD globale, distribuzione dati/bundle o nuova
release/tag anticipata. Namespace03, reconciliation04 e altre task conservano
il proprio stato; il gate Windows diCAD11 non è dichiarato superato prima della nuovaCI.

## Gate locali conclusi / handoff

Cinque comandi exit0 sul delta stabile: fmt, Clippyworkspace all-targets allfeatures,
58test reference/presence/T70/lookup,30test allfeatures lookup/namespace/sessione,
boundarydipendenze;0fallimenti,0ignored. Hash prima/dopo identici nel ledger.
Diff produttivo verso42f92ed vuoto per src/Cargo/UI/CI: nuova policy e dataset
sono artefatti di componente, senza distribuzione implicita nel bundle esistente.

CAD11 nonDONE prima della CI nativa propria. Pubblicare il commit11, ma dispatch
solo quando CI42190202683 diCAD03 è terminale: concurrency del workflow sullo
stesso branch annullerebbe il run precedente. Non avviare una pipeline duplicata.
Durante l'attesa è ammesso preflight della task09 indipendente, in sequenza.
CAD04/05 attendono03 effettivamenteDONE.130/140DONE,10aperte invariati.
Nessuna conferma ordinaria richiesta, release o chiave operativa generata.
