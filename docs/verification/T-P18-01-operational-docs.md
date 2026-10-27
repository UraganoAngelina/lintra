# T-P18-01 — README operativo e accettazione nativa

## Prerequisiti

M5 MVP generico realmente raggiunta da15-07/30d82fba/CI40475046094:
Linux/MSVC/confronto JSONL success. Prerelease privata0.1.1-rc.1 e asset di15-05/06
verificati nella recovery. Non equivale a certificazione clinica SIOC/CAD,
persistenza storica o UI completa. Fonti DET97, ARCH39C.3–4 e CLI reale attuale.

## Documentazione

README aggiunge quickstart separati Linux eWindowsPowerShell5.1, eseguiti dalla
cartella estratta senza bundle/ambiente esterno, con dati dichiaratamente
SYNTHETIC/v1. Il file è di6byte oltre terminatore, creato con printf o WriteAllBytes,
così nessun editor/pipeline decide padding o encoding. Versione, help e comando
validate hanno parametri reali (nessun CSV CLI inventato). Output JSONL/summary/
XLSX e risultato atteso sono specificati; il successo è limitato allo scope locale.

Exitcode0/1/2/130 corrispondono a from_status/from_parse_error esistenti: il parser
non usa2 per la sintassi. Precedenza directory esatta, nessuna latest implicita,
BOM Reject, LF/CRLF, byte offset, output UTF8/noBOM/LF e PowerShell5.1 con `>`
sono espliciti. XLSX per lettura Excel; pubblicazione atomica per file, non una
transazione fra output. Informazioni Windows/Authenticode conformi alla decisione
ARCH39C.4, nessun nuovo perimetro di distribuzione o obbligo di sicurezza inventato.
Aggiornate soltanto note README obsolete su release e bootstrap; specifiche protette
inalterate. I testi evitano di promettere capacità normative non implementate.

## Probe ed esecuzione da zero

scripts/readme_smoke.py estrae i blocchi marcati DAL README, non una copia degli
argv tenuta nel test. Esige un solo blocco per shell, usa un vero binario release,
crea un'installazione temporanea nuova con spazi e copia soltanto l'eseguibile.
Rimuove dalla sola env del subprocess le due variabili dati (case-insensitive
anche su Windows), senza cambiare ambiente o filesystem dell'operatore.

Linux esegue bash; Windows powershell.exe, versione5.1 verificata esplicitamente,
non pwsh7. La sintassi PS è ASCII, i byte di input sono scritti con .NET senza
BOM. Il runner valida exit e versione/commit; dopo il blocco verifica input
immutato/CRLF, JSONL vuoto, summary VALID/local_only/embedded/1record/0errori,
ZIP XLSX con Riepilogo edErrori. Non finge di aver avviato l'applicazione Excel.

Casi indipendenti eseguono CLI reale: un valore sinteticamente errato restituisce2
con esattamente1errore, JSONL UTF8 senzaBOM conLF; BOM, versione inesistente e
coppia directory incompleta restituiscono1 senza report completati. Opzione
inesistente restituisce1, distinta dall'invalidità normativa. Ogni input e binario
restano byte-identici, tempdir rimossa e sola evidenza sintetica/hash conservata.
Ctrl+C/130 è documentato e già qualificato dai test CLI con token; questo probe
non dichiara un nuovo test end-to-end del tasto Ctrl+C/SmartScreen/Excel desktop.

La CI esegue il probe DOPO la release riproducibile e il verifier, passando il
log xtask con l'unico artifact effettivo; nessun vecchio target indovinato.
Carica JSON di accettazione separato per i due target, da leggere prima di DONE.

## Verifiche locali e residuo

Linux literal README e tutti6casi passati sul binario release qualificato
59ba771 (hash/versione esatta nel JSON locale), con argv del README attuale.
21unittest degli strumenti, py_compile e actionlint1.7.12 pass. La vecchia copia
actionlint in /tmp non esisteva più: recuperata stessa versione da release
ufficiale con SHA256 dell'archivio confrontato al checksum ufficiale, senza nuove
dipendenze Cargo. Nessun Rust modificato in questa task: non ripetuti i gate
735/735/717 già passati su49a3f4e, eseguiti nuovamente comunque in CI.

Residuo: commit/push e nuova CI nativa per PERSIST03+18-01. Non DONE fino a
PowerShell5.1 reale e gate completo. Nessun dato paziente o release creata.
UI03 ora DONE/CI40577920052,16-04 pronta. AUTO_CONTINUE single-agent.
