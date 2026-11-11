# Lintra desktop Tauri: brief operativo

T-MIG-P0-07,11 novembre2026. **Mode: Operate.** Target futuro:
apps/lintra-desktop; nessuna nuova UI implementata da questo brief.
Utenti: autore e colleghi autorizzati che preparano invii, individuano anomalie
e consultano il dettaglio necessario a correggerle. Il lavoro parte dalle
selezioni reali e termina con risultati, export e provenienza verificabili.

Autorità: mandato unificato10 novembre, ADR-MIG-001/002/003/004 e
[scope obbligatorio](../unified-post-m9/MIGRATION_SCOPE_PARITY.md).
Il mondo visivo è Copper Ledger già approvato. La migrazione estende Analisi/
Risultati al Full, preservando i comportamenti e adattando la composizione
alla WebView. Le decisioni estetiche ordinarie sono delegate dal mandato
Signature; non richiedono un nuovo giro di scelte o una ratifica globale.

## Direzione e fonti visive

Grafite/avorio/rame, IBM Plex Sans per comandi e lettura, IBM Plex Mono per
codici/valori. Font e asset locali; marchio geometrico Lintra esistente,
nessuna immagine clinica o nuovo logo. Colore per azione/selezione, rosso per
violazioni, ambra per incompletezza; testo e icona accompagnano sempre lo stato.

Fonte eseguibile verificata: ui/ui/tokens.slint, qualificata incb70bee. I token
iniziali del nuovo frontend derivano da questi ruoli in un registro CSS/theme
unico, senza valori sparsi nei componenti. I due stack restano distinti durante
la coesistenza; non si rigenerano o alterano i token/golden Slint per adattarli
a React. I valori seguenti sono intenzioni di partenza verificate sul legacy,
non una promessa di uguaglianza pixel WebView/Slint.

| Ruolo | Chiaro | Scuro |
| --- | --- | --- |
| Fondo | #F5F4F0 | #101112 |
| Superficie | #FFFFFF | #1B1D1F |
| Testo | #20272B | #F5F4F0 |
| Testo secondario | #56616A | #B6BABC |
| Azione/focus | #A13E1A | #FF9C66 |
| Selezione | #F9EDE4 | #392A21 |
| Violazione | #AE2925 | #FF9288 |
| Parzialità | #855408 | #EFC177 |

Titoli28/18, corpo14 e dati tecnici12 nella scala esistente; controllo/riga36,
bordi1, focus2, raggi4/6 e spazi4/8/12/16/24/32. La WebView adatta geometrie e
reflow dopo verifica alle dimensioni di consegna. Contrasto del testo almeno
4,5:1 secondo il criterio già usato nel progetto, anche nei due temi; focus e
stato non dipendono solo dal colore. Nessuna ombra/blur o movimento decorativo.

DESIGN.md frontmatter riporta ink-dark#F4F3EE, mentre Tok e direzione canonica
eseguono#F5F4F0: è una discrepanza preesistente documentata, non corretta
silenziosamente. Qui prevalgono token e render qualificati. Le capture Slint
sintetiche fresche analysis-dark1366×768@2 e results-light640×480@1 sono state
osservate; servono a ereditare identità/densità, non a qualificare il nuovo frontend.

## Percorso e composizione

La barra mantiene Analisi, Risultati, tema e Informazioni. Analisi è la vista
iniziale; Risultati indica la disponibilità del report reale. Il titolo e lo
stato orientano la lettura, senza dashboard KPI o attività inventate.

Il form espone flusso/versione e modalità, con dipendenze comprensibili. In
LocalOnly un input per tracciato; in Full un elenco derivato dallo schema con
file assegnati esplicitamente, presenza/assenza e parametri richiesti. Il file
può essere scelto prima di completare il form, ma solo Rust stabilisce se la
richiesta è pronta. Namespace/periodo non si ricavano dal nome del file.
Profilo, perimetro e reference effettivi restano visibili vicino all'avvio.

Configurazione e stato esecuzione affiancati quando lo spazio lo consente;
sotto il breakpoint di contenuto passano in colonna. Il riferimento legacy è
1100px, da verificare sul form Full reale. Una pagina scorrevole mantiene
raggiungibili tutti i file/parametri; nessun pannello con contenuto tagliato o
azione fuori portata. Una sola azione primaria per stato; cancel disponibile
solo quando Rust consente l'interruzione della lavorazione attiva.

Risultati porta prima esito, ambito e completezza, poi diagnostici e dettaglio.
Export è vicino al riepilogo, legato al report visto. Le cinque ricerche hanno
etichette permanenti e combinazione AND Rust; valori e contatori sono quelli
del backend. La tabella virtualizza soltanto la pagina/finestra richiesta.
Inspector a fianco sul desktop largo, sotto la tabella nelle finestre strette;
lettura completa del messaggio e domini a pagine, senza copia dell'indice in JS.
La viewport della tabella gestisce il proprio scorrimento orizzontale; la pagina
non si allarga per una colonna o un valore lungo.

Le nuove sezioni storiche arrivano con U-CAD/U-NORM-04 quando esistono i servizi
e i gate applicativi: non si riempie una navigazione con approvazioni finte.
Il componente SIOC limitato espone i controlli effettivi; il nome Full non rende
completo il workbook regionale. Una situazione Unknown/Partial resta tale.

## Stati e parole dell'interfaccia

| Situazione reale | Presentazione e azione |
| --- | --- |
| Nessuna analisi | “Prepara una nuova analisi”; spiegare quale selezione manca, nessun successo a zero errori |
| Pronta | “Avvia analisi”; riepilogo file/schema/ambito effettivi |
| In corso | Passata/tracciato e contatori Rust; avanzamento indeterminato quando il totale non è noto |
| Cancellazione richiesta | “Interruzione richiesta”; attendere il terminale Rust, nessun completamento simulato |
| Cancelled | “Analisi annullata”; dire quali risultati/report sono effettivamente disponibili |
| Violazioni | “Violazioni rilevate”; record errati e diagnostici distinti, dettaglio della causa |
| Controlli incompleti | “Controlli incompleti”; motivi Unknown/mancanze e ambito, anche se non ci sono violazioni |
| Completo senza violazioni | “Nessuna violazione nei controlli eseguiti”; mantenere ambito/profilo, nessuna certificazione implicita |
| Errore tecnico | Spiegare l'operazione fallita e l'azione utile, senza stack trace o dati in console |
| Export parziale | Elenco dei file davvero pubblicati e falliti; non un messaggio generale di successo |

La UI mappa stati, non ricostruisce regole. NotApplicable, Unknown e Partial
non vengono accorpati. Il report resta legato al suo run/schema anche quando
si cambia il form di una nuova analisi. Un evento tardivo non rimpiazza quel
contesto. ID interni e dettagli IPC non sono testo dell'operatore.

Budget soft: “I file superano la soglia di avviso configurata. L'analisi non
ridurrà l'input per rientrare nella soglia.” Azioni Annulla/Procedi comunque,
con binding verificato da Rust. Nessun comando per ignorare un limite hard.

Shared: “La destinazione è condivisa o remota. Le persone che hanno accesso
alla cartella potranno leggere i report.” Azioni Cambia destinazione/Salva qui.
Se la natura è ignota, dichiararlo e permettere conferma esplicita. Per un
path locale scrivibile nessuna nuova conferma privacy; overwrite resta distinto.
I testi dipendono dalla classificazione Rust e non da una denylist JS.

## Acceptance UX e visiva

Per Windows11 e CachyOS personale raccogliere render della nuova UI, separati
per OS, tema chiaro/scuro e scala1×/2×. Viewport di lavoro:640×480,1024×768,
1366×768,1920×1080 e1920×1200. Minimo e desktop canonico coprono tutti gli stati;
le altre dimensioni verificano reflow e contenuti lunghi. Zoom/DPI200% verifica
che la work area effettiva permetta scroll e focus; non forzare dimensioni
native che nascondono comandi fuori monitor. Non si promette supporto mobile
ufficiale perché uno strumento generico suggerisce375px.

Ogni controllo ha label, focus/hover/active/disabled/loading/error leggibili.
Tab segue l'ordine di lavoro; tasti nativi e navigazione tabella portano al
dettaglio senza trap o perdita di focus alla virtualizzazione. Dialoghi di
consenso/overwrite ripristinano il focus; menu e popover non vengono tagliati
dalle viewport scrollabili. Annunci di errore/esito sono accessibili; il
progresso è aggregato, non un annuncio per ogni record. Reduced-motion rispettato.

Provare percorsi lunghi, messaggi multilinea, campo/actual/expected mancanti,
domìni estesi, file opzionale assente, query senza risultati, report alterato,
partial/truncated/cancel, export negato e più violazioni sulla stessa riga.
Schermate, geometrie, keyboard/a11y e semantica sono prove complementari;
non si allargano tolleranze o aggiornano golden normativi per coprire un difetto.

Il protocollo di misura è nel
[budget prestazionale](../unified-post-m9/MIGRATION_PERFORMANCE_BUDGET.md).
Le scelte visive ordinarie sono autorizzate; G-MIG-0 e i gate UI restano da
verificare. Il prossimo passo è completareP0-08, poi gli spike reali P1.
