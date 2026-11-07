# CAD — componente conclusioni 2024-06.conclusions.1

Componente parziale A/B1/C per T-CAD-07, non schemaCAD clinico completo,
FROZEN o snapshot operativo. Fixture sintetiche con filler esplicito fuori
dai13campi selezionati; lunghezzeA218/B1198/C45 e PK complete originali.
Source_fields conserva i campi sorgente; source include digestPDF/tracciati/
profilo runtime e decisioni. Date e sede mantengono coordinate esatte.

La data2.6facoltativa definisce se esiste una conclusione;2.7è richiesta
quando2.6presente. CodiciNdi due byte conservati esattamente:03/04/07/82per
i decessi. Il tipoPIC2.3governa i divieti10/13della fonte; i vincoli sul codice
si applicano solo quando presente, senza rendere indeterminata un'assenza
facoltativa. Gli altri campi clinici non sono dichiarati convalidati.

C→B1usa namespace sending_ulss e consensus della tupla data/motivo fra i
record con data presente. Conclusioni uguali sono qualificabili anche con
progressivi diversi; conclusioni discordanti restanoUnknown con diagnostico.
Il filtrodecesso si applica dopo il consenso, senza cancellare conflitti.

Due piani: sede1vietata strettamente dopo ogni conclusione; sedi2/4/5/6vietate
inoltre dopo decesso. Sede3ammessa e stessa data non bloccante. L'unione dei
divieti coincide con la policyfonte, verificata nel source-check07. Si riusa
E_CAD_ACCESS_AFTER_CONCLUSION anche per l'estensione post-decesso.

Solo inputcorrenti: nessuna lettura/approvazione persistente o conformità
CADglobale simulata. Dettagli, limiti e gate effettivi nel dossier07.
