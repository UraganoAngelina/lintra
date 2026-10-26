# CNF-ARTIFACT-LINUX-01 — Runtime Linux ammesso dal verifier

2026-10-26. **PROPOSTA, NON RATIFICATA.** Preflight T-P15-02.

## Evidenza e fonti del conflitto

T-P15-01 DONE: CI40307076976 sul commit7c919fb, Linux/MSVC entrambi success.
Il gate verifica ottimizzazione, remapping e riproducibilità, non certifica ancora
autosufficienza: quel controllo appartiene a15-02.

ARCH§39C.1, DET Addendum A.3 e prompt T-P15-02 richiedono che il CLI Linux
dipenda soltanto dalla «libc di base», oppure sia statico se musl. Non specificano
una allowlist di SONAME e non ratificano l'inclusione del runtime GCC.
T-P1-02 e docs/DEVELOPMENT.md mantengono DEC-11 aperta per glibc baseline/musl.
Ricerca mirata in ratifiche AGENTS, matrice, registro dipendenze, proposte e
dossier toolchain: nessuna decisione applicabile trovata su libgcc_s Linux.

Ispezione readelf --dynamic dell'artefatto Linux GNU locale prodotto dalle due
build pulite della15-01 (stesso prodotto, nessun dato paziente) restituisce:

```text
NEEDED libgcc_s.so.1
NEEDED libm.so.6
NEEDED libc.so.6
NEEDED ld-linux-x86-64.so.2
```

Il binario remoto non è stato scaricato né viene dichiarato ispezionato.
L'evidenza locale basta a mostrare il caso concreto che il verifier deve poter
classificare correttamente; non si presume l'inventario della futura release.

libgcc_s è il runtime GCC, non il SONAME libc: lo conferma la
[documentazione GCC](https://gcc.gnu.org/onlinedocs/gccint/Libgcc.html).
La sua relazione con l'unwinding di glibc è documentata dai
[manutentori glibc](https://sourceware.org/pipermail/libc-alpha/2025-June/168169.html).
Questo motiva la proposta, ma non sostituisce una decisione sul contratto Lintra.

## Impatto concreto

Un verifier letterale che ammetta soltanto componenti glibc rifiuta questo
artefatto. Includere libgcc_s senza decisione allargherebbe implicitamente il
gate architetturale. Scegliere musl/staticità completa o cambiare panic/unwinding
non è una semplice correzione del controllo: toccherebbe strategia Linux ancora
aperta o comportamento del prodotto. Non vengono adottati flag instabili o
modifiche del panic per aggirare il requisito.

## Soluzione minima proposta A — raccomandata

Ratificare per il solo CLI x86_64-unknown-linux-gnu la seguente allowlist
esplicita di dipendenze dinamiche dirette:

```text
libc.so.6
libm.so.6
ld-linux-x86-64.so.2
libgcc_s.so.1
```

libgcc_s.so.1 è l'unica eccezione qui proposta al runtime glibc/loader, per il
runtime di basso livello/unwinding della toolchain Rust. Nessun wildcard per
libgcc, nessuna scoperta automatica delle librerie ammesse dall'host, nessuna
promozione di una libreria presente sulla macchina a dipendenza autorizzata.
Ogni altro SONAME fallisce; eventuali ulteriori componenti della famiglia glibc
richiedono una revisione esplicita della lista, non sono presunti necessari.

Restano invariati: Windows solo DLL di sistema/CRT statico e divieti GCC/pthread/
VCRUNTIME/MSVCP; musl soltanto statico; controlli path host e openssl-sys;
verifica negativa del verifier e CI sui due OS. Nessuna pubblicazione/release,
baseline glibc minima o revisione di ARCH/DET è approvata implicitamente.
DEC-11 resta aperta per la distribuzione in15-05. Nessuna dipendenza runtime
nuova: l'eccezione descrive la dipendenza già osservata e ne delimita il controllo.

## Alternativa B

Mantenere il vincolo letterale senza libgcc_s dinamica e formalizzare una ricetta
che la elimini, qualificandone supporto su runner, licenze, unwind e baseline.
Nessuna fattibilità è dichiarata già dimostrata. La scelta di musl o di una nuova
strategia Linux deve essere esplicita e coordinata con DEC-11, senza aggirare
dipendenze o criteri di accettazione.

## Autorizzazione richiesta

Approvare A, oppure indicare il vincolo alternativo da formalizzare. Fino alla
ratifica T-P15-02 è BLOCKED al preflight contrattuale: nessuna implementazione
del verifier o nuova dipendenza aggiunta; nessun DONE/M5 anticipato.
La governance single-agent/autonoma rimane attiva; questo è un gate umano reale,
non lo stop ordinario di fine task rimosso dalla migrazione.
