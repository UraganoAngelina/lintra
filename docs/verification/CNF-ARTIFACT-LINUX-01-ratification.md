# CNF-ARTIFACT-LINUX-01 — Ratifica A

2026-10-26. L'autore approva esplicitamente la proposta A: uso Linux soltanto
interno/personale, distribuzione ad altri attraverso Windows.

Per il CLI x86_64-unknown-linux-gnu sono ammesse esclusivamente le dipendenze
dinamiche dirette libc.so.6, libm.so.6, ld-linux-x86-64.so.2 e libgcc_s.so.1.
Nessun wildcard o discovery dell'allowlist dall'host. Altri SONAME falliscono.
Windows conserva DLL di sistema soltanto, CRT statico e divieti esistenti.
Musl, se verificato, deve essere statico. Path host e openssl-sys restano vietati.

La motivazione non rimuove i Quality Gate Linux e non seleziona implicitamente
una baseline di distribuzione, un nuovo target o modifiche ad ARCH/DET. DEC-11
resta registrata per la strategia release da affrontare nella relativa task.
La ratifica sblocca T-P15-02 e autorizza il verifier e i relativi test/gate.
La proposta originale e il preflight bloccato restano documenti storici.
