# Carichi di riferimento — U-PERF-01

11 novembre 2026. D13 autorizza misure motivate, mantenendo la produzione
sequenziale. Questo documento seleziona volumi e dichiara ciò che manca;
non misura la nuova UI e non stabilisce SLA o una nuova DEC-14.

## Fonte disponibile

Il kit già consegnato contiene `CAD_SAMPLE_ANALYSIS.md`, analisi del16 ottobre
con **sole statistiche aggregate** di un invio. È stato letto questo documento,
non i quattro file originali, e non sono stati acquisiti record personali.
Fonte e digest sono nel receipt U-PERF-01. Non è necessario un nuovo sourcing.

| Tracciato | Record osservati | Byte per record | Payload senza terminatori |
| --- | ---: | ---: | ---: |
| A | 2.748 | 218 | 599.064 |
| B1 | 2.825 | 198 | 559.350 |
| B2 | Non fornito | 41 nello schema | Non determinato |
| C | 45.104 | 45 | 2.029.680 |
| D | 137.249 | 39 | 5.352.711 |
| Totale dei quattro file forniti | **187.926** | Misto | **8.540.805** |

Questa è una forma osservata per un singolo invio, non una media, un massimo,
un percentile o una previsione per ogni collega. Periodo esatto, hardware,
durata, distribuzione per evento e crescita dello storico non sono ricavabili
da questi aggregati. La fonte indica2.748 eventi distinti; non consente di
ricostruirli o di stimare la distribuzione degli accessi per evento.
Le larghezze corrispondono al profilo CAD83 corrente verificato nel receipt.

## Selezione dei carichi

Il manifest [REPRESENTATIVE_WORKLOADS.json](REPRESENTATIVE_WORKLOADS.json)
è una specifica per le prove successive, non una configurazione del prodotto.

1. **CAD observed shape:** generare da zero dati sintetici con questi quattro
   conteggi e larghezze. Con un LF dopo ogni record il payload è8.728.731byte;
   con CRLF è8.916.657byte. Sono dimensioni sintetiche calcolate, non misure dei
   file originali. Non leggere o trasformare gli originali per crearli.
   B2 resta assente, distinto da un file presente e vuoto. La misura Full/storico
   deve rispettare il warning e il consenso per il tracciato mancante e dichiarare
   la completezza effettiva; non imporre un esito globale Valid/Complete.
2. **Correttezza e smoke:** conservare i piccoli casi sintetici esistenti,
   inclusi gli oracle SIOC cumulativi e i100/300 eventi CAD83. Servono per
   verificare il percorso e i conteggi; non rappresentano invii operativi.
3. **Stress della tabella:**300.000 e1.000.000 diagnostici sintetici, come da
   piano MIG, separati dall'invio CAD e dai tempi decisionali. Non convertirli
   in record di input o in un default operativo da500.000 righe.

Non si distribuiscono indipendentemente record casuali fra i tracciati:
il generatore futuro deve costruire PK e relazioni sintetiche coerenti con i
contratti già qualificati. Conteggi e larghezze non dimostrano da soli join,
conformità normativa o equivalenza delle prestazioni.

## Errori, reference e storia

La fonte contiene anomalie aggregate, ma non un totale di diagnostiche
normative. Il rapporto errori/record resta **non determinato**. Per separare i
costi, le prove successive possono usare varianti sintetiche dichiarate:
nessun errore locale intenzionale,1% o100% di record con un singolo difetto
locale noto. Quelle percentuali sono ipotesi di prova, mai tassi osservati.
Arrotondamento e cardinalità effettiva dei diagnostici devono essere verificati
dal relativo oracle; più regole possono diagnosticare lo stesso record.
Non modificare golden, domini o regole per raggiungere un conteggio desiderato.

Reference e schema provengono dai profili già ricevuti e congelati D12.
Lo storico richiede baseline approvate sintetiche, namespace distinti e le
sequenze T1/T2/T3 già previste. Non attribuire tre volte la dimensione fisica
di un invio al numero di nuove identità logiche. Per SIOC1–3 le ricoperture
identiche appartengono allo stesso cumulato, secondo l'addendum D11 vigente.
Nessuno storico non approvato diventa baseline per un benchmark più semplice.

## Limiti e uso delle misure

**SIOC operativo non calibrato:** il materiale esaminato non fornisce conteggi
aggregati per tracciato/invio, massimi o crescita mensile. Non trasferire a SIOC
la scala CAD o i2/3/4 record dell'esempio normativo. Lasciare i parametri
operativi SIOC non determinati; i test sintetici e lo sviluppo proseguono.
Anche quantità B2 e dimensione della storia CAD restano non determinate.

Il protocollo di [misura MIG](MIGRATION_PERFORMANCE_BUDGET.md) richiede SHA,
hash fixture, conteggi effettivi, stato/completeness, reference e baseline,
OS/CPU/RAM/storage/runtime, build, cold/warm e campioni. Separare preflight,
validazione, storico, indice/query, renderer e export; heap richiesto e RSS
sono metriche diverse. Nessuna misura WebView o Windows11 è qui attestata.
U-PERF-02 dopo MIG-P7-08 confronterà il servizio diretto e il bridge reale.
G-PERF resta NOT_VERIFIED; nessuna soglia o parallelizzazione automatica.
