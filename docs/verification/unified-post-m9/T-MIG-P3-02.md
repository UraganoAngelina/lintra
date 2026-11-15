# T-MIG-P3-02 — Pin e supply chain frontend

15 novembre2026. **DONE**; P3-01 PASS Linux/Windows su c8ad22e/CI43731922567.
Versioni dirette esatte e lockv3 con URL registry/SSRI SHA512/licenze verificati,
identici allo spike qualificato. Nessun nuovo pacchetto/versione Cargo o npm.
Install locale offline senza lifecycle, con nuovi vincoli engine, exit0.
[Receipt](T-MIG-P3-02-checks.json). Le licenze runtime originali sono copiate
in public/THIRD_PARTY_NOTICES, senza dipendenze globali o del core.

| Dipendenza | Scopo concreto |
|---|---|
| React / React DOM | Rendering della shell locale |
| @tauri-apps/api | Trasporto tipizzato del client P3-05 |
| @tauri-apps/cli | Build nativa riproducibile dell’adapter |
| TypeScript | Strict typecheck, nessun compilatore remoto |
| @types/react / @types/react-dom | Contratti dei componenti realmente compilati |
| Vite | Bundle locale statico, nessun dev server in release |

Node26.13.0 esatto in .nvmrc/engines. Profili npm espliciti già osservati:
CachyOS12.2.2 e Windows CI11.20.0; engines ammette solo questi due, non un range
aperto. .npmrc locale impone engine-strict, ignore-scripts e save-exact.
Nessuna installazione/upgrade globale per uniformare artificialmente i profili.
P3-06 ricontrolla install/build in entrambi gli OS sul nuovo stato sorgente;
nessuna nuova CI o scansione advisory dichiarata qui.49/128 DONE; segue P3-04.
