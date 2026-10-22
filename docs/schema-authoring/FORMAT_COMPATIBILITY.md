# Inventario compatibilità formato / eseguibile — CNF-FORMAT-01

2026-10-19. **APPROVATO dall’autore: «Approvo la mappa con baseline di sviluppo 0.1.0».**
CNF-FORMAT-01 risolto. La tabella è la mappatura ufficiale corrente del progetto;
non è una dichiarazione di release pubblicata o supporto completo di local.v1.
L’autore ha richiesto l’elenco completo dei casi e successivamente approvato
la mappa qui riportata. La precedente proposta di messaggio provvisorio nel
preflight è superata da questa mappatura ratificata.

## Stato verificato

- Formati definiti: 1 (contratto invariato) e 2 (estensione Decimal approvata
  dall’autore il 2026-10-22; consegna T-P9-03).
- Motore di sviluppo corrente: Cargo package 0.1.0. Il compiler del formato 1
  nel sottoinsieme approvato è verificato su b658e05, CI 39264362959.
- `git tag --list`: nessun tag locale. `gh release list --repo
  UraganoAngelina/lintra --limit 100 --json tagName,name,isDraft,isPrerelease,publishedAt`:
  risposta `[]`, exit 0. Nessuna release GitHub da assumere come baseline ufficiale.
- Il formato JSON di flow e manifest non contiene minimum_engine_version.
  Nessuna mappatura di release future è dichiarata dai documenti.

## Partizione completa degli input di schema_format_version

Le classi seguenti coprono l’insieme degli input JSON: non si elencano
separatamente i 4.294.967.293 interi da 3 a u32::MAX, che non hanno ancora
una specifica Lintra distinta e condividono lo stesso stato.

| Caso | Esempi | Stato attuale verificato | Mappatura approvata |
|---|---|---|---|
| Proprietà assente | nessuna chiave | Errore caricamento raw, nessun default | Documento non conforme; nessuna versione minima applicabile |
| null | null | Errore caricamento raw | Documento non conforme; nessuna versione minima applicabile |
| Tipo non numerico | stringhe "1"/"1.0.0", booleani, array, oggetti | Il DTO richiede u32, nessuna conversione di questi tipi | Documento non conforme; nessuna versione minima applicabile |
| Numero non rappresentabile come u32 dal parser raw | negativi, frazioni, valori maggiori di 4.294.967.295 | Errore caricamento raw | Documento non conforme; nessuna versione minima applicabile |
| Intero 0 | 0 | Deserializzabile, rifiutato dal lint/compiler | Formato Lintra non definito; nessun eseguibile compatibile dichiarato |
| Intero 1 | 1 | Formato corrente; compiler vincolato al sottoinsieme approvato | **Minimo 0.1.0 come baseline di sviluppo approvata**, senza dichiarare una release pubblicata |
| Intero 2 | 2 | Decimal esplicito, solo profilo importi ratificato; formato 1 preservato | Baseline sviluppo 0.1.0 della consegna T-P9-03, commit/CI verificati nel dossier della task; nessuna release pubblicata dichiarata |
| Interi da 3 a 4.294.967.295 | 3, 4, …, u32::MAX | Deserializzabili se il resto del documento ha shape raw riconosciuta, rifiutati dal lint/compiler | Numeri di formato non ancora definiti; nessun eseguibile compatibile dichiarato. Assegnazione puntuale obbligatoria quando un nuovo formato viene definito |

JSON malformato, chiavi duplicate e documenti raw con struttura errata sono
errori di caricamento indipendenti dalla mappa. Non consentono fallback.
La compatibilità del formato numerico non prova supporto delle capability,
validità dello schema, disponibilità dataset o conformità di record sanitari.

## Assegnazione concreta approvata

```text
schema_format_version: 1
minimum_engine_version: 0.1.0
baseline_kind: development
verified_compiler_commit: b658e05
release_published: false
scope: formato 1 + sottoinsieme compiler autorizzato da CNF-COMPILER-01
```

Questa assegnazione è stata approvata dall’autore; non aggiunge metadata ai
JSON normativi. La stringa 0.1.0 identifica il package corrente ed è stata
ratificata come minimo ufficiale della baseline di sviluppo. L’attuale commit disambigua
le build di sviluppo precedenti con lo stesso numero, che non disponevano
necessariamente del compiler. Una futura release dovrà essere immutabile.
La mappa è una dichiarazione di supporto del contratto dati, non pubblicizza
local.v1 completa e non promette che gli attuali schemi SIOC/CAD compilino.

Per 0 e 3..=u32::MAX non esistono oggi specifiche di formato da associare a una
release: la decisione ufficializza **nessun eseguibile compatibile dichiarato**,
non un numero di release fittizio. Un nuovo formato richiederà un’entry propria
con versione minima effettiva prima di essere distribuito.

## Decisione approvata

L’autore ha approvato congiuntamente:

1. Baseline di sviluppo 0.1.0 per formato 1, con riferimento al compiler
   verificato e senza attribuirle lo stato di release pubblicata.
2. Stato non definito / nessun eseguibile compatibile dichiarato per 0 e
   3..=u32::MAX; messaggio esplicito che non suggerisce un aggiornamento inesistente.
3. Input mancanti/non validi come errori di caricamento senza mapping di
   versione minima. Ogni formato futuro sarà mappato quando viene definito.

Ogni futura modifica della baseline o assegnazione di formato richiede una
nuova dichiarazione approvata. Nessuna versione futura viene dedotta da
numeri di formato o da semver.

## Estensione formato 2 — ratifica 2026-10-22

L'autore ha approvato la proposta Decimal, incluso il nuovo formato e gli
aggiornamenti della mappa. Format 2 minimo: 0.1.0, baseline di sviluppo della
consegna T-P9-03; non significa che i vecchi commit 0.1.0 supportino formato 2.
Il commit effettivo e la CI nativa sono registrati in
[docs/verification/T-P9-03.md](../verification/T-P9-03.md) dopo verifica.
Non si associa al formato 2 il vecchio compiler b658e05: quello resta evidenza
esclusivamente del formato 1. Nessuna release/tag o compatibilità futura inventata.
Al momento dell'introduzione le verifiche sono in corso, senza DONE anticipato.

I vecchi test di formato non definito usano ora 3 invece di 2: le stesse
asserzioni di errore rimangono, con la distinzione approvata che 2 è definito.
Una descrizione EngineCapabilities creata dal chiamante continua a non
riconfigurare il compiler; soltanto l'implementazione verificata abilita 1..=2.
