# Dettaglio diagnostico e domini catturati — P2-06

Design IPC v1,14 novembre2026. Envelope/DTO schemaVersion1, limiti ratificati
P2-02. [Reference eseguibile](detail_reference.py) integrata con query/paging
P2-05 e codec comune. Non è il bridge Rust/Tauri: G-MIG-2 resta aperto.

## Selezione e contenuti

Il renderer seleziona un diagnostic_ref della query autorizzata. Rust risolve
il token nelle due pagine correnti e trattiene un solo diagnostico privato;
il nuovo selection_ref è legato a owner nativo, RunRef, query e report
immutabile. La selezione sopravvive all’espulsione della pagina, ma non a un
cambio query, owner o selezione. Nessun path o indice fisico dal renderer.
Errori di autorizzazione e contesto sono tecnici, senza cambiare il verdetto.

SelectedDiagnostic DTO versionato: selection_ref/query_ref/diagnostic_ref,
RunRef, line u64 testuale, field_position opzionale (start/end testuali,
unità byte1-based inclusivi), actual_visibility e componenti richieste.
Code/message/expected/nome campo/guida restano esattamente quelli catturati;
preview UTF-8 max256byte con complete esplicito, non sostituti del testo.
Per la sola riga selezionata sono consultabili i singoli identificatori
sioc_record_id/fiscal_code/episode_id. Nessuna mappa tecnica indiscriminata,
actual.raw_hex, riga sanitaria fisica completa o array di tutti i diagnostici.

La copertura del campo viene controllata in Rust sullo schema compilato
catturato: trace, FieldId e range devono coincidere, ed essere un sottoinsieme
proprio della lunghezza del record. La reference riceve questa mappa nativa;
non accetta una dichiarazione client. Un actual che copre il record intero è
`local_report_only_record_scope`, senza valore IPC. La diagnostica generica
E_RECORD_LENGTH con solo conteggio ASCII, senza raw_hex, conserva il conteggio
di byte: è metadato, non la riga. Nessun export/JSONL canonico viene riscritto.
Questa è minimizzazione del dettaglio autorizzato, non una garanzia PHI-free:
anche un singolo campo o messaggio va trattato come dato riservato nel DOM.

## Testi e contesto

Ogni componente ha handle opaco nativo. read_text restituisce schemaVersion1,
offset/next_offset byte u64 testuali, text e has_more. Da1 a16384byte UTF-8 per
lettura; offset su continuazione UTF-8, limite incapace di contenere il prossimo
carattere e range oltre EOF sono rifiutati esplicitamente. EOF esatto restituisce
text vuoto/has_more false. Nessun trim, HTML, link attivo, normalizzazione di
Unicode, spazi, zeri o testo canonico. Le letture complete ricostruiscono il
testo su richiesta; il client mantiene una sola finestra di testo, senza
prefetch/accumulo nascosto o persistenza DOM. Non esiste un get_record_text.

Context è richiesto separatamente, massimo64elementi a pagina. Ogni elemento
versionato contiene preview trace/field, visibility e valore display con
component_ref solo se il campo catturato è un sottoinsieme proprio. Mai raw_hex.
Una sola pagina di componenti contesto resta trattenuta; pagina nuova invalida
i token della precedente. Elenco e testo restano esatti nel report Rust.
Le richieste condividono il budget dati P2-04; chunk e pagine vengono codificati
e verificati come envelope interi prima della pubblicazione. Nessun taglio
silenzioso per superare il cap1MiB e nessun dataset completo in memoria JS.

## Domini, provenienza e disponibilità

Rust risolve il campo usando il trace del diagnostico nel CompiledFlowSchema
del report, anche per Full multi-tracciato. Non usa il form corrente o un
catalogo ricaricato dopo il run. La vecchia UI Slint limita questa vista a
ScopeSummary::Trace: tale limite non viene trasferito al contratto Full.
Guide, encoding, confronto raw/logical, dichiarazioni e reference sono catturati
assieme al report. API esistenti: CompiledFlowSchema::trace,
CompiledRecordSchema::{field,record_length}, CompiledFieldSpec::range,
field_format_guide e constraint_comparison_guide. Nessuna regola in React.

Una fonte alla volta, legata alla selezione e alla dichiarazione nativa.
Kind esplicito allowed_values/range/fallback/reference/regex/non_enumerable;
le dichiarazioni condizionali conservano RuleId e condizione nei componenti
di guida nativi e flag conditional. Non diventano vincoli incondizionati.
Fallback autorizzati restano distinti dal contenuto verificato del dataset.
Fonte e pagina nuova invalidano i vecchi token di valori/cursori.

DeclaredDomain usa declared_domain_page, max64: ordine dichiarativo per enum,
IntegerLiteral esatto anche oltre u128 per range e continuazioni native.
Regex e altri vincoli non enumerabili producono not_enumerable/entries null;
non si inventano esempi o elenchi finiti di valori ammessi.

Reference usa PreparedReferenceEvidence::{evidence,domain_page} catturata e
domain_literal con encoding catturato. Risposta versionata: kind/conditional,
expected_sha256/actual_sha256, status, entries e next_cursor opaco. I digest
derivano dall’evidenza nativa, mai da input IPC. Stati distinti:

| Stato | entries | Significato |
|---|---|---|
| available | pagina<=64 | Lettura riuscita da fonte ammessa |
| available_empty | [] | Lettura riuscita, fonte effettivamente vuota/fine |
| degraded/unavailable/not_captured | null | Evidenza insufficiente o assente |
| unsupported | null | Provider non espone enumerazione |
| cancelled | null | Lettura annullata, nessun verdetto inventato |
| not_enumerable/record_scope | null | Nessun elenco di dominio applicabile |

Solo Verified e digest effettivo uguale a quello atteso consentono la lettura
reference; mismatch è errore tecnico. Un’osservazione Degraded+present non
costituisce conformità, None non è false. LookupSet::page_after preserva i byte
esatti, zeri e padding; domain_literal rende visibili gli spazi secondo le API
esistenti, senza alterare lookup o decisioni normative. Ordine lessicografico
per ByteLookupSet. La pagina richiede O(N log64) sul set nativo: worker Rust,
scratch bounded, nessuna promessa di tempo costante o lettura sull’event loop.

Il cursore nativo contiene DomainPosition o byte esatti dell’ultimo valore;
IPC trasporta solo l’handle. Una pagina e64componenti al massimo, con preview
e lettura testuale bounded per valori lunghi. Provider che restituisce65valori,
pagina vuota con continuazione, hash incoerenti o fallimento tecnico non viene
presentato come dominio vuoto. Non cambia il risultato della validazione.

## Evidenze e confini

verify_detail.py collega codec, paging su JSONL sintetico privato e dettaglio,
con reader nativi iniettati per i domini. Verifica owner/stale, selezione,
guide Full per trace, copertura record, context64, Unicode e ricostruzione,
immutabilità della fonte, digest e tutti gli stati negativi. Non riproduce
il motore, LookupSet o la semantica delle regole in Python.
P3/P4/P5/P7 devono qualificare adapter Rust, worker/cancel reale, consumatore
DOM bounded, accessibilità e parity. Nessuna nuova suite/CI/VM in questo task.
