# CAD B1 T(70), componente 2024-06.t70.1

Componente dichiarativo T-CAD-10, campi2.3 e2.22, derivato dal candidato
CAD2024-06 e dalla ratifica G03. Conserva posizioni reali B1:2.3 a15,
2.22 a67 con larghezza70, in record198. Hash/provenienza sono in component.json.
Questo file è un componente da comporre esplicitamente, non un flow.json
caricabile o un profilo CAD certificato/FROZEN. Nessun bundle lo include ora.

Il valore T non riceve una semantica numerica: exact e regex sul raw preservano
ogni byte.1–59 accettano1/2;61–70 accettano solo2;60 accetta1/2 per2.3=2,
uno spazio ASCII per2.3=1. Nessun trim, zero-fill, riempimento implicito o fallback.
I nodi regex/condition sono già supportati dal formato approvato: il compiler
continua a rifiutare positional_bitset non implementato. Il lowering equivalente
è dichiarato qui e non eseguito implicitamente durante il caricamento legacy.

La fixture Rust usa contenitori sintetici per i byte estranei a questi due
campi, mantenendo le coordinate reali. Non certifica gli altri34campi B1,
namespace ULSS, PK, storico, dataset o gli altri tracciati. Il profilo completo
resta alle task CAD dedicate e al loro gate normativo.

Le regole condizionali verificano prima la presenza dei due campi: se manca
un valore, la violazione required resta primaria e non si inventa un confronto.
source_fields conserva integralmente le dichiarazioni ricevute per audit,
compreso il positional_bitset legacy; quei metadata non vengono eseguiti.
