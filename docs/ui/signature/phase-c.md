# Fase C — No AI Slop: revisione editoriale

Skill installata letta integralmente:
`/home/alberto/.codex/plugins/cache/openai-curated-remote/no-ai-slop/1.0.6/skills/no-ai-slop/SKILL.md`.
Letto anche eval.md e applicata la verifica direttamente, senza altri agenti.
Input: direzione A, componenti e capture B, inventario strings-phase-b.txt,
controller di presentazione/export/worker e requisiti diagnostici. Nessuna nuova
proposta estetica. L'inventario contiene anche import/Path/font per completezza
della scansione: questi non sono microcopy e non sono revisionati.

## Modifiche verificabili

| Prima (Fase B) | Dopo | Motivo |
|---|---|---|
| Field ID | ID campo | Terminologia italiana coerente con il dettaglio |
| n righe visibili | n diagnostiche trovate | Il numero conta l'intero modello filtrato, non la viewport; singolare per1 |
| Consulta l’esito del run o modifica i filtri applicati. | Consulta il riepilogo dell’analisi o modifica i filtri. | Riferimento riconoscibile per chi non conosce run |
| Dall’inizio | Primi valori | Specifica che la navigazione riguarda il dominio |
| Superiore | Cartella padre | Destinazione esplicita nel selettore |
| Apri selezionato | Apri selezione | Azione valida sia per file sia per cartella |
| Avvia l’analisi quando la configurazione è pronta. | Completa la configurazione, poi avvia l’analisi. | Istruzione operativa |
| Il riepilogo verrà salvato accanto al report, aggiungendo .summary.json al nome. | Il riepilogo usa il percorso del report con suffisso .summary.json. | Percorso e suffisso conservati, meno parole |
| Verranno salvati il report e un riepilogo JSON con versioni e provenienza. | Il salvataggio include report e riepilogo JSON con versioni e provenienza. | Forma diretta, stesso contenuto |
| Attendere l’esito dell’analisi. | Attendi l’esito dell’analisi. | Voce coerente con le altre istruzioni |

## Glossario UI

Analisi: operazione richiesta dall'utente; run resta il termine tecnico nei
contratti. Flusso: famiglia dello schema. Versione schema: versione esplicitamente
scelta. Tracciato: struttura del file sottoposto ai controlli. Diagnostica:
segnalazione del report; non sinonimo di errore tecnico. Valore letto/atteso:
contenuti originali del report. Dominio ammesso: valori dichiarati dallo schema,
consultabili a pagine. Riepilogo: conteggi, completezza, ambito e provenienza.
Esporta report: salvataggio esplicito; preferenze di formato non sono file salvati.

## Semantica preservata

Invariati codici E_*, messaggi/actual/expected/context del report, campi e Trace ID,
chiavi/hash/versioni, nomi dei dataset, valori raw/logical e loro spazi/zeri,
Unknown/NotApplicable e completezza, evidenze Verified/Degraded/Unavailable.
Nessuna modifica ai moduli core, schemi, golden normativi o serializer.
«Controlli locali» resta sempre esplicito; nessuna promessa di conformità globale.
«Annullamento richiesto» rimane distinto da un annullamento confermato.

Verifica eval: significato e termini preservati; nessun claim, esempio normativo
o tono promozionale aggiunto; istruzioni concrete e azioni riconoscibili. Le
stringhe già precise (Avvia analisi, Annulla analisi, Esporta report, Salva report,
versioni e provenienza) sono lasciate invariate. Residuo intenzionale: alcuni
diagnostici del core sono lunghi; l'ispettore li presenta integralmente e non
li riscrive per ragioni editoriali.

Testi completi finali nel codice Slint e nei due controller UI; inventario finale
strings-phase-c.txt. La successiva Fase D verifica questa stessa composizione.
