# CAD11 — recupero checkout Windows, 6 novembre 2026

CI42197434407 su aae2e12: Linux positivo; Windows ha superato tutti i108 test
library (4ignored), inclusi journal scomparso, ACL e contesa CAS originale.
Il job Windows si è poi fermato nei test CAD11: sei errori su sette, perché il
checkout aveva convertito LF in CRLF nei file sorgente e membership JSON.
La CI complessiva è fallita; CAD03 eCAD11 restano aperte, senza gate ereditato.

## Causa e correzione

La SHA256 osservata su Windows per la sorgente ICD9 è
d324f83a26e914563380ea2bc6b498c0fb1db9b5d47da1a7e2a0aa8806fb420b:
coincide esattamente con la trasformazione CRLF del file originale LF.
La membership ICD9 produce 4683262c4e519bd1556c8214615ca85060661092168449b8e2aeb34a105aa200
con la stessa trasformazione. Il provider ha correttamente rifiutato l'integrità
alterata, senza promuovere questi dati a lookup definitivi.

.gitattributes dichiara -text solo per source/** e data/** del componente
2024-06.clinical-lookups.1: sono artefatti con provenienza/hash, i cui byte fanno
parte del contratto. Non si modificano JSON, hash attesi, parser o controlli del
provider. Nessuna normalizzazione runtime o deroga Windows viene introdotta.

T-CAD-11-checkout-recovery.json registra una riproduzione Git reale in repository
temporanei con core.autocrlf=true: prima della regola tutti e5i file cambiano
hash, dopo la regola tutti e5restano identici byte per byte. Questa è una prova
del checkout configurato come Windows, non un gate nativo sostitutivo.
Nuova CI Linux/MSVC sul commit pubblicato richiesta prima di DONE.

Log del fallimento e risultato remoto preservati nella cartella omonima;
SHA256 dei file di evidenza in evidence-hashes.json. Nessuna fonte CAD originale
riscritta, nessun dato paziente. CAD09 locale resta separata dal gate fallito.
