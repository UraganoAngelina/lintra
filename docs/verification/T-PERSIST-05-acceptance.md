# T-PERSIST-05 — tracciabilità accettazione

Stato: DONE sul commit3712ed6, CI40844540287 tutti4job success; release
SQLite40844543949 entrambi job success; probeWindows40844049209 success.
Misure ed esiti grezzi nel dossier native-ci e nei JSON benchmark Linux/Windows.

| Requisito | Prova automatizzata reale |
|---|---|
| DELETE/EXTRA impostati e riletti, cache/mmap/versione | `roundtrip_restart_immutable_snapshots_omission_namespace_and_settings` |
| Snapshot/indici/provenienza coerenti, immutabilità, omissioni conservate, namespace | stesso test con due revisioni e due namespace, più `process_restart_and_abrupt_exit_inside_or_after_commit_recover_coherently` |
| Stage invisibile, abort/drop, errore prima di COMMIT, CAS obsoleto | `abort_drop_stale_generation_and_failure_before_commit_do_not_publish` |
| Contesa processi, una sola generazione vincente | `two_process_contenders_publish_exactly_one_generation` |
| COMMIT rifiutato o riuscito senza risposta: nessun falso rollback | `ambiguous_commit_is_resolved_by_exact_immutable_candidate` |
| Connessione con transazione privata non leggibile come pubblicata | `an_unfinished_transaction_view_cannot_be_mistaken_for_publication` |
| Restart di processo, crash durante COMMIT con journal e dopo pubblicazione | `process_restart_and_abrupt_exit_inside_or_after_commit_recover_coherently` e helper `process_worker` |
| DTO pubblici non conferiscono prova HMAC; query con chiave sbagliata non dà falsa assenza | `protection_cannot_be_bypassed_by_public_dto_or_same_label_wrong_key` |
| Materiale chiave mancante/incompatibile non inizializza nuovo storico | stesso test, più apertura esplicita/versioni |
| Nessun record grezzo/fingerprint invio grezzo/segreto nel file | scansione byte del DB nello stesso test, fixture interamente sintetiche |
| Source/record recuperati verificati e validati | `tampered_record_or_source_is_rejected_instead_of_returning_unverified_domain_values` |
| Nuovo/compatibile/non inizializzato/non riconosciuto/versione non supportata/corrotto | `unknown_unsupported_corrupt_and_uninitialized_are_distinct_without_reinitialization` |
| No reinizializzazione distruttiva | confronto byte del file sconosciuto/versione futura e create_new rifiutato |
| Root esplicita, traversal, sidecar estranei, permessi, symlink | `unsafe_paths_permissions_and_foreign_sidecars_fail_closed`; rami Unix e Windows reali |
| ACL Windows ereditata privata, rifiuto World access | `inherited_private_acl_is_required_and_world_access_is_rejected` nel modulo Windows |
| Indici effettivi, query puntuali, busy esplicito | `query_plans_are_indexed_and_external_writer_contention_is_explicit` |
| 10k/100k/300k, lookup/commit/reopen/RSS, indici e contesa | harness reale `benches/persistence.rs`, qualificatore `scripts/qualify_sqlite.py` |
| SQLite statico, niente dipendenza runtime dalla SQLite di sistema | qualificatore release sull'eseguibile che usa realmente l'adapter, con allowlist ELF/PE di xtask invariata |
| Regressioni core/UI/tooling e packaging | CI ordinaria Linux/MSVC, confronto JSONL e pixel, build reproducible/artefatti e smoke SQLite |

Codice: [test adapter](../../src/persistence/sqlite/tests.rs),
[test ACL Windows](../../src/persistence/sqlite/security/windows.rs),
[harness](../../benches/persistence.rs),
[spiegazione implementativa e limiti](T-PERSIST-05-sqlite.md).

L'helper ignorato non è un requisito saltato: i parent lo avviano con un archivio
sintetico e verificano gli exit code. `during` esegue cache_flush su pagine
private, poi esce86 dal commit hook SQLite senza Drop Rust. `after` esce87 dopo
il COMMIT. Il parent verifica il journal prima della recovery e la coerenza di
snapshot, invio e indice dopo riapertura. Questo non simula alimentazione tolta
al dispositivo né certifica firmware/hardware arbitrario.

L'errore prima di COMMIT è iniettato solo nella build test; il test della guard
non-autocommit costruisce realmente una transazione SQL rimasta aperta. Non si
pretende di aver indotto un guasto fisico EIO o provato ogni punto delle syscall.

La memoria include anche C e fixture. Il candidato/load_snapshot restano O(n)
per contratto; la riapertura/lookup in processo fresco non possiede il candidato
precedente. Nessuna soglia DEC14 dedotta dai risultati. Solo dati sintetici e
metadati tecnici; cifratura/retention/chiavi/backup prima dei dati reali restano
fuori dalla ratifica di implementazione05.
