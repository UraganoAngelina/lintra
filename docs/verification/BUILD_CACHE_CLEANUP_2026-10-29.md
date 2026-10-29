# Pulizia delle cache di compilazione — 2026-10-29

Richiesta dell'autore: recuperare gli artefatti eliminabili, poi riprendere lo
sviluppo. Misure effettive in BUILD_CACHE_CLEANUP_2026-10-29.json: recuperati
31.611.547.648 byte sul filesystem del repository; dopo verifica disponibili
59.775.143.936 byte (unità decimali, diverse dai GiB mostrati da df).

Rust conserva librerie compilate, eseguibili di ogni suite e variante e database
incrementali. Feature, target, profili e cambiamenti nel codice generano varianti;
le vecchie build isolate delle verifiche non vengono eliminate da Cargo. Il volume
logico dei file non coincide con lo spazio recuperabile, anche per hardlink e
allocazione del filesystem: il risultato riportato usa la variazione reale dello
spazio libero, non la somma delle dimensioni logiche.

Eliminati database incrementali (circa29GB logici), build isolate/prototipi e target
non correnti, release riproducibili,145 eseguibili test rigenerabili. Conservati
UI/CLI correnti e relativi hardlink, cache librerie riutilizzabili, sorgenti, lock,
fixture, documenti, risultati e log delle verifiche. Nessuna build attiva durante
la pulizia. Il target del generatore legacy in /tmp è stato eliminato separatamente:
essendo tmpfs non concorre al recupero sul disco del repository.

`.cargo/config.toml` imposta `[build] incremental = false`: Cargo riutilizza ancora
le dipendenze già compilate ma non conserva i database incrementali Rust per ogni
variante. Ottimizzazioni, simboli debug e collegamento release restano invariati.
Una directory incremental vuota può esistere: non indica una cache attiva.
L'override esplicito `CARGO_INCREMENTAL=1` resta possibile. Le ricompilazioni dopo
una modifica e la rigenerazione dei test eliminati possono richiedere più tempo.

Verifica reale: `cargo check -p lintra --locked --offline -j2`, exit0,8,91s.
Log conservato in target/verification-cache-cleanup-check.log. Configurazione
supportata da Cargo: https://doc.rust-lang.org/cargo/reference/config.html#buildincremental.
La pulizia non costituisce un nuovo gate funzionale del prodotto e non sostituisce
le evidenze CI delle task. Prosecuzione da T-PERSIST-06 e prossima READY.
