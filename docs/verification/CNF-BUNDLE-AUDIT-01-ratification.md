# CNF-BUNDLE-AUDIT-01 — ratifica A

2026-10-26. L’autore approva esplicitamente «la proposta A».
Fonte: messaggio della sessione successivo alla proposta CNF-BUNDLE-AUDIT-01.

È approvato il contratto dei sei punti della [proposta](../proposals/CNF-BUNDLE-AUDIT-01.md):
manifest embedded obbligatorio e verificato; manifest esterno opzionale, esplicito
con --bundle-manifest oppure soltanto MANIFEST.json del parent comune delle root
schemas/ e reference-data/. Senza manifest esterno audit external/version null/
manifest_status absent; con manifest valido versione dichiarata e status present.
Nessuna ricerca cwd/home, selezione latest o fallback per singolo file.

Verificare i byte dello schema selezionato prima della compilazione e la coerenza
path/digest delle dichiarazioni reference con il manifest. Non controllare
preventivamente tutti i byte reference: Strict/Continue e Verified/Degraded/
Unavailable restano quelli ratificati. Present non certifica i dataset.

Audit al confine adapter/report con contesto tipizzato e writer additivo;
ValidationService, ValidationRequest, RunMetadata, ValidationSummary e writer
preesistenti restano compatibili. API generiche senza provenienza non la inventano.
Stesso contenuto esterno/embedded: hash schema e JSONL errori identici, origine
distinta. I bundle di T-P15-04 avranno sempre manifest.

Autorizzata ripresa immediata T-P15-03 dal wiring sospeso. Nessuna modifica a
ARCH/DET, baseline normative o regole engine. DONE solo dopo gate effettivi.
