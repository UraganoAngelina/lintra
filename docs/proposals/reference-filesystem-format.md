# CNF-REFERENCE-FORMAT-01 — Formato del provider filesystem

2026-10-23 — **PROPOSTA NON RATIFICATA, non implementata.**

## Problema verificato

T-P11-03 richiede lettura nel formato di REFERENCE_DATA_SPEC. La specifica
SIOC del kit definisce identità/versioni, assenza come errore tecnico,
immutabilità e bisogni di attributi/validità; presenta però manifest e singole
righe con «such as» e «may use», senza un contenitore runtime completo.
SCHEMA_AUTHORING.md, «Reference bindings e capability», stabilisce espressamente:
«Il formato dei provider e le versioni reali restano in REFERENCE_DATA_SPEC
quando i dati saranno disponibili; il dataset d’esempio non è quel contratto».
T-P0-07 ratifica questa distinzione, non un formato JSON dimostrativo.
Onboarding T-P0-06 dichiara il formato canonico runtime non definito e le dodici
tabelle SIOC mancanti. Nessuna ratifica successiva reperita stabilisce il formato.

Non è un dubbio sulle norme degli importi/età né riapre una CNF già risolta.
Scegliere il wire format del provider trasformando l'esempio in contratto
contraddirebbe la riserva esplicita del documento approvato. P10 e il provider
in memoria non dipendono da questa decisione; proseguono secondo i propri gate.

## Soluzione circoscritta proposta

Approvare un **profilo tecnico locale di membership versione 1**, destinato a
dataset esplicitamente convertiti in tale formato. Non rappresenta il formato
regionale originale né autorizza versioni/contenuti normativi inventati.
Gli importatori di sorgenti regionali, quando disponibili, rimangono separati.

Esempio soltanto sintetico del file UTF-8 JSON, senza BOM:

```json
{
  "dataset_format_version": 1,
  "dataset_id": "synthetic-codes",
  "version": "test-1",
  "keys_hex": ["30303120", "4162", "00ff"]
}
```

1. Oggetto chiuso, tutti i campi obbligatori, nomi/identità senza spazi esterni;
   rifiuto di proprietà ignote, duplicate e versioni formato sconosciute.
2. keys_hex è una lista di stringhe hex lowercase di lunghezza pari; ogni coppia
   rappresenta esattamente un byte. È una codifica di trasporto, mai una
   normalizzazione di chiavi: zeri, spazi, NUL e byte non UTF-8 preservati.
   Duplicati della stessa chiave rifiutati come dataset malformato; lista vuota
   ammessa e distinta dal file/versione assente. Chiave zero-byte esplicita
   ammessa, con missing determinato soltanto dalla policy del campo.
3. Il costruttore riceve root PathBuf e binding espliciti delle dipendenze
   (coppia DatasetKey, path relativo, SHA-256 atteso), riusando il modello
   ReferenceDependency già esistente. Nessuna scansione o versione latest,
   nessuna deduzione di filename dal nome dataset, nessun secondo manifest.
4. Percorso relativo confinato alla root; rifiuto traversal/assoluti/escape
   tramite symlink. SHA-256 sui byte originali prima del parsing; identità e
   versione dentro il file devono coincidere con la dipendenza richiesta.
   Assenza, hash mismatch, malformazione o identità errata sono errori tecnici
   di preparazione con causa reale, mai code-invalid del record.
5. Dataset preparato immutabile condiviso in Arc; il loop di membership non
   legge file. Nessun provider di default fittizio nel servizio; wiring in 11-06.
6. Profilo iniziale **membership soltanto**: metadata_attributes non vuoti,
   attributi extra, intervalli temporali o richieste di lookup effettivo per data
   sono rifiutati esplicitamente prima dei record. Non si appiattiscono tabelle
   diagnosi/attributi a set di codici né si finge che siano supportate.

Questo chiude solo il formato membership di T-P11-03. Le funzionalità dichiarate
dall'authoring per metadata_bindings restano obblighi espliciti e non diventano
DONE: richiederanno un profilo/API tipizzato prima di accettare tali schemi.
L'eventuale consegna incrementale di 11-05 dovrà nominare quel residuo e
rifiutarlo come previsto da CNF-COMPILER-01; nessuna omissione silenziosa.

## Verifiche richieste dopo approvazione

Fixture sintetiche con hash indipendente: successo membership byte esatto,
versioni diverse, dataset vuoto, identity/hash/version mismatch, JSON/hex/BOM
malformati, chiavi/proprietà duplicate, path traversal e symlink escape,
attributi/tempo non supportati rifiutati. Gate locali e CI Linux/Windows.
Baseline SIOC/CAD, documenti normativi, grammar flow v1/v2 e sentinel invariati.

## Decisione richiesta

Approvare questo profilo tecnico membership esplicito, con i limiti sopra,
oppure mantenere T-P11-03 in attesa di una specifica runtime alternativa.
Non è richiesta una nuova decisione sui codici/regole regionali.
