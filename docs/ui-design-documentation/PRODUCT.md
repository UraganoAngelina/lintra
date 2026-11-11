# Lintra — contesto di prodotto

Per la nuova UI React/Tauri vale il [marchio ufficiale](LINTRA_OFFICIAL_BRAND.md)
ratificato l’11 novembre 2026; il contesto Slint sottostante resta la fonte
storica del workbench. Scope corrente nel brief MIGRATION_TAURI_BRIEF.md.

Fonte: prompt Signature dell'autore,2026-10-29, e inventario del codice5197a98.

## Piattaforma

Desktop nativo Linux/Windows, Rust e Slint1.18.1/software. Nessuno stack web.

## Utenti e obiettivo

Analisti esperti di flussi regionali: preparare input e schema, eseguire controlli,
individuare e correggere anomalie. Accesso comprensibile anche ai meno esperti,
senza eliminare informazioni tecniche. Prodotto generico, versionato e guidato
dallo schema, non gestionale per pazienti o personale clinico.

## Capacità presenti

Catalogo reale, selezioni esplicite, un file/tracciato in LocalOnly, controllo
preliminare, worker/cancellazione, risultati su JSONL con indice e cache128,
filtri esatti, dettaglio e dominio paginato, export condiviso con la CLI,
Informazioni con attribuzione Slint. Nessuna UI per sessioni/snapshot storici.

## Vincoli confermati

Separare violazione, errore tecnico, Unknown/NotApplicable, parzialità e
indisponibilità. Nessuna norma nel renderer, nessun totale/progresso simulato,
nessuna certificazione implicita. Provenienza riferita al run visto. Font
IBM Plex incorporati, due temi, minimo640×480. Fixture solo sintetiche.
Contratti motore/compiler/persistenza invariati. Nessuna dipendenza aggiunta.

## Successo della revisione

Identità distintiva ed efficienza operativa dimostrate con render reali,
confronto prima/dopo e test. Il nuovo mandato delega esplicitamente le decisioni
estetiche ordinarie all'agente e richiede le skill in successione.


## Ambito della migrazione post-M9

Il documento sopra conserva il contesto Slint del29 ottobre. Il mandato del10
novembre approva React/TS strict/Tauri2, LocalOnly+Full e CAD storico nella
roadmap. [MIGRATION_TAURI_BRIEF.md](MIGRATION_TAURI_BRIEF.md) descrive il nuovo
adapter e i criteri di consegna, senza attribuirgli funzioni già implementate.
