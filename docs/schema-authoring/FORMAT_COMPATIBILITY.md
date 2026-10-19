# Inventario compatibilità formato / eseguibile — CNF-FORMAT-01

2026-10-19. **DRAFT: inventario verificato; assegnazioni proposte da approvare.**
L’autore ha richiesto l’elenco completo dei casi e una mappatura ufficiale.
Questo non costituisce ancora approvazione della proposta A del preflight.

## Stato verificato

- Unico formato definito nel contratto approvato: intero 1.
- Motore di sviluppo corrente: Cargo package 0.1.0. Il compiler del formato 1
  nel sottoinsieme approvato è verificato su b658e05, CI 39264362959.
- `git tag --list`: nessun tag locale. `gh release list --repo
  UraganoAngelina/lintra --limit 100 --json tagName,name,isDraft,isPrerelease,publishedAt`:
  risposta `[]`, exit 0. Nessuna release GitHub da assumere come baseline ufficiale.
- Il formato JSON di flow e manifest non contiene minimum_engine_version.
  Nessuna mappatura di release future è dichiarata dai documenti.

## Partizione completa degli input di schema_format_version

Le classi seguenti coprono l’insieme degli input JSON: non si elencano
separatamente i 4.294.967.294 interi da 2 a u32::MAX, che non hanno ancora
una specifica Lintra distinta e condividono lo stesso stato.

| Caso | Esempi | Stato attuale verificato | Mappatura proposta |
|---|---|---|---|
| Proprietà assente | nessuna chiave | Errore caricamento raw, nessun default | Documento non conforme; nessuna versione minima applicabile |
| null | null | Errore caricamento raw | Documento non conforme; nessuna versione minima applicabile |
| Tipo non numerico | stringhe "1"/"1.0.0", booleani, array, oggetti | Il DTO richiede u32, nessuna conversione di questi tipi | Documento non conforme; nessuna versione minima applicabile |
| Numero non rappresentabile come u32 dal parser raw | negativi, frazioni, valori maggiori di 4.294.967.295 | Errore caricamento raw | Documento non conforme; nessuna versione minima applicabile |
| Intero 0 | 0 | Deserializzabile, rifiutato dal lint/compiler | Formato Lintra non definito; nessun eseguibile compatibile dichiarato |
| Intero 1 | 1 | Formato corrente; compiler vincolato al sottoinsieme approvato | **Da assegnare: minimo 0.1.0 come baseline di sviluppo**, senza dichiarare una release pubblicata |
| Interi da 2 a 4.294.967.295 | 2, 3, …, u32::MAX | Deserializzabili se il resto del documento ha shape raw riconosciuta, rifiutati dal lint/compiler | Numeri di formato non ancora definiti; nessun eseguibile compatibile dichiarato. Assegnazione puntuale obbligatoria quando un nuovo formato viene definito |

JSON malformato, chiavi duplicate e documenti raw con struttura errata sono
errori di caricamento indipendenti dalla mappa. Non consentono fallback.
La compatibilità del formato numerico non prova supporto delle capability,
validità dello schema, disponibilità dataset o conformità di record sanitari.

## Unica assegnazione concreta oggi proponibile

```text
schema_format_version: 1
minimum_engine_version: 0.1.0
baseline_kind: development
verified_compiler_commit: b658e05
release_published: false
scope: formato 1 + sottoinsieme compiler autorizzato da CNF-COMPILER-01
```

Questo è un candidato alla decisione, non metadata aggiunto ai JSON normativi.
La stringa 0.1.0 identifica già il package corrente; la sua assegnazione come
minimo ufficiale richiede la decisione dell’autore. L’attuale commit disambigua
le build di sviluppo precedenti con lo stesso numero, che non disponevano
necessariamente del compiler. Una futura release dovrà essere immutabile.
La mappa è una dichiarazione di supporto del contratto dati, non pubblicizza
local.v1 completa e non promette che gli attuali schemi SIOC/CAD compilino.

Per 0 e 2..=u32::MAX non esistono oggi specifiche di formato da associare a una
release: la proposta ufficializza **nessun eseguibile compatibile dichiarato**,
non un numero di release fittizio. Un nuovo formato richiederà un’entry propria
con versione minima effettiva prima di essere distribuito.

## Decisione richiesta

Approvare congiuntamente:

1. Baseline di sviluppo 0.1.0 per formato 1, con riferimento al compiler
   verificato e senza attribuirle lo stato di release pubblicata.
2. Stato non definito / nessun eseguibile compatibile dichiarato per 0 e
   2..=u32::MAX; messaggio esplicito che non suggerisce un aggiornamento inesistente.
3. Input mancanti/non validi come errori di caricamento senza mapping di
   versione minima. Ogni formato futuro sarà mappato quando viene definito.

Alternativa: scegliere una diversa versione iniziale prima della prima release;
in tal caso serve indicare quale e se l’assegnazione è pianificata o già effettiva.
Nessuna versione futura viene dedotta da numeri di formato o da semver.
