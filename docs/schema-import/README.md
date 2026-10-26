# Profili di import XLSX

Tooling offline T-P14-01. Un profilo mappa celle del workbook a un blueprint
RawFlowSchema revisionato. Il workbook è la fonte effettiva di ID, nomi,
coordinate e note originali; il blueprint conserva semantiche che le colonne
non definiscono da sole: alignment/missing, presenza condizionale, PK/identity,
join, regole, reference e policy persistenti. Nessuna nota diventa DSL da sola.

`sioc-2026-04-20.xlsx-profile.json` copre135 campi in5sezioni del solo workbook
SIOC fornito dall'autore; blueprint `2026-04-20.structural-minimal.2` invariato.
I token A/D/Data/AN/N e il vuoto4.5 sono binding espliciti, non alias globali
inferiti per nome. I mapping operativi si basano sulle decisioni già approvate
nel blueprint: per esempio4.5 Numeric e4.6/4.10 Alphanumeric. Nessuna nuova
capability Alphabetic, regola clinica o conversione di compartecipazioni.

Uso API: leggere separatamente blueprint/profilo con serde_json, costruire
`XlsxSchemaImporter::new(blueprint, profile)?`, poi `SchemaImporter::import(path)`.
Il risultato è raw e richiede lint/revisione/compiler. Il profilo non cambia la
baseline, non rende eseguibile l'intero SIOC e non rimuove persistence.audit.
Cambiare un binding semantico richiede aggiornare il profilo revisionato; header,
sezione, ID o token inattesi causano un errore tecnico, non un ripiego.

Coordinate/righe/colonne del profilo sono1-based; puntatori a source sheet e
formule rimangono nell'audit. UseCachedValues legge il valore cached delle sole
coordinate: non ricalcola formule e lo dichiara come non verificato. Policy
Reject disponibile. Note/ID richiedono vere stringhe; formule testuali, cache
mancanti/non intere, ID aggiunti fuori mapping e VBA sono rifiutati.

Ogni file è letto una sola volta in byte per binding fra hash e contenuto parser.
source.xlsx_import contiene hash sorgente, dimensione, hash canonico blueprint,
profilo, annotazioni testuali selezionate e status requires_review. Non contiene
path assoluti o timestamp dell'host. Il nome/posizione del file non determina
versioni o regole. Non si dichiara protezione contro modifiche concorrenti del
filesystem: il digest attesta esattamente i byte letti e interpretati.

Riproduzione profilo: `python3 docs/verification/generate_sioc_xlsx_profile.py`.
Default confronta soltanto; --write scrive solo il profilo, previa verifica hash
originale6be8ae60a2c236e5a25b731365b42f8e7bca849066d68747abfc78b92ce273e3.
Il workbook originale rimane locale, mai destinazione dello script o fixture CI.
I documenti degli altri flussi forniti dall'autore restano fuori scope.

## Normalizzazione e lint avanzato14-02

normalize_import_profile(profile, aliases) normalizza solo i tag revisionati
mediante una tabella esplicita, restituendo il report dei cambiamenti; non tocca
token originali o blueprint. La tabella SIOC allegata deriva dagli alias già
ratificati, inclusa A→alphanumeric; non è una convenzione globale per nuovi flussi.
Alias sconosciuti/ambigui o tentativi di ridefinire tag canonical sono errori.

lint_raw_schema espone warning delle note importate non ancora classificate e
contraddizioni dichiarate. review_imported_note registra la classificazione
esplicita Metadata(reason) o Mapped(rule_ids esistenti), con hash del testo
originale; non compie la revisione e non genera regole. Una modifica del testo
invalida il vecchio acknowledgement. I warning non sono violazioni normative.
Dettagli e limiti nel dossierT-P14-02-authoring-lint.md.
