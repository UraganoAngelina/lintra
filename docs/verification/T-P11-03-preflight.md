# T-P11-03 — Preflight contrattuale

2026-10-23. Stato corrente: **SBLOCCATA dalla ratifica aggiornata**
[CNF-REFERENCE-FORMAT-01](CNF-REFERENCE-FORMAT-01-ratification.md).
La domanda storica sotto ha risposta; non è più pendente.

## Preflight API minimo prima dell'implementazione autorizzata

Estensione additiva proposta e attuata nel perimetro §17–18: nuovi tipi
DatasetAvailability/PreparationPolicy/Evidence e PreparedReferenceEvidence,
più FileSystemReferenceDataProvider::prepare(policy) per tutte le dipendenze.
Lo snapshot conserva mappe di lookup Arc e informazioni ordinate di provenienza;
observe restituisce una misura opzionale accompagnata dall'evidenza tipizzata:
Unavailable non offre un false di membership, Degraded non offre una misura
spacciata per verificata. Nessun metodo restituisce il suo raw LookupSet degradato.

Le firme LookupSet::contains, ReferenceDataProvider::load, prepare_reference_data
e PreparedReferenceData esistenti restano compatibili. Il filesystem implementa
load soltanto in Strict: restituisce un set solo dopo verifica completa. Questo
impedisce che un consumer legacy interpreti un set degradato come affidabile.
Il nuovo percorso Continue richiede policy esplicita e conserva tutte le dipendenze,
anche indisponibili. Errori di sicurezza/configurazione o primitive non supportate
restano fatali e non diventano fallback degradati. Nessuna modifica Summary/
RunStatus/RunMetadata/service; rispettive evoluzioni nel ledger futuro.

File rigorosamente conforme ma hash diverso → Degraded. BOM/JSON ambiguo/
proprietà ignote o duplicate/hex invalido/entry duplicate → Unavailable;
nessun recupero selettivo o parser permissivo. È il confine iniziale esplicito,
non una dichiarazione che ogni forma di recupero futuro sia impossibile.
Hash originale conservato anche per file che poi risultano malformati o estranei.

Sicurezza: controlli lessicali portabili, componenti symlink rifiutati e path
canonico confinato alla root canonica, file regolare. Questi controlli proteggono
configurazioni stabili, NON risoluzione atomica contro modifiche ostili concorrenti
(TOCTOU). Root/albero vanno mantenuti sotto controllo del processo di deployment;
non si dichiara una garanzia che std::fs non fornisce. Nessuna lettura nella membership.

## Registro storico del blocco originario

Stato al rilevamento: **BLOCKED soltanto sul formato filesystem**.
Prerequisiti 11-01 e 0-06 completati; il problema è distinto dalla fornitura
ancora mancante delle tabelle ufficiali e non riapre le regole dei tracciati.

## Fonti verificate e conflitto

- Prompt T-P11-03: leggere «nel formato di REFERENCE_DATA_SPEC.md».
- `SCHEMA_INPUTS/SIOC_2026-04-20_FROZEN/REFERENCE_DATA_SPEC.md` del kit:
  esempi di manifest («such as») e riga («may use»); nessun contenitore/file
  runtime completo con encoding, attributi, intervalli e risoluzione dei path.
- `docs/SCHEMA_AUTHORING.md`, «Reference bindings e capability»:
  riserva esplicita del formato provider e distinzione dal dataset dimostrativo.
- `docs/verification/T-P0-07.md`: contratto ratificato ma dataset inventato
  espressamente escluso dalla ratifica del formato provider.
- `docs/normative/reference-datasets.md` T-P0-06: non definisce il formato
  canonico runtime; per le tabelle SIOC mancano file/formato/encoding/versioni.
- Ledger `IMPLEMENTATION_STATUS.md`, tabella «Formato dataset — CHIUSO»:
  descrive binding id/version/hash/path/normalizzazione/metadata, non definisce
  un wire format che possa superare la riserva esplicita dell'authoring.
  Nessuna evidenza di ratifica successiva del contenitore runtime reperita.

## Impatto e proposta

Non è lecito promuovere l'esempio authoring a formato ufficiale, inventare
attributi per i dataset mancanti o ignorare i vincoli temporali. È possibile
ratificare un profilo tecnico membership esplicito, distinto dalle fonti
regionali, con rifiuto dei requisiti ancora non supportati.
[Proposta concreta](../proposals/reference-filesystem-format.md), non implementata.
L'approvazione del profilo membership non chiuderebbe metadata_bindings o la
validità temporale: obblighi futuri conservati, mai dichiarati eseguiti.

La domanda all'autore è pendente. P10, 11-02 e altre attività indipendenti
restano autorizzate e continuano secondo dipendenze/gate. Nessuna modifica a
REFERENCE_DATA_SPEC, baseline FROZEN o decisioni già ratificate.

## Verifiche

Controllo documentale di prompt, specifica, authoring e ratifiche/ledger.
Nessun test Rust artificiale per questa rilevazione; nessun risultato runtime
o CI dichiarato per il provider filesystem, ancora non implementato.
