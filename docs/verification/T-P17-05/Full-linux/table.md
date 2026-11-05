| Record | Carico | Seq s | 1worker s | 2worker s | 4worker s | Speedup4 |
|---:|---|---:|---:|---:|---:|---:|
|10,000|bilanciato, pochi errori locali|0.1014|0.1184|0.1050|0.1002|1.012x|
|100,000|bilanciato, pochi errori locali|1.1495|1.3693|1.1833|1.1141|1.032x|
|100,000|bilanciato, pochi errori locali, proiezioni ampie|1.5517|1.6893|1.5325|1.5115|1.027x|
|100,000|bilanciato, molti errori locali|2.5070|3.9532|3.8992|3.8272|0.655x|
|100,000|sbilanciato, pochi errori locali|1.6336|1.7336|1.8198|1.7164|0.952x|
|500,000|bilanciato, pochi errori locali|6.2408|7.6742|7.0418|6.5584|0.952x|
|500,000|bilanciato, molti errori locali|12.9864|21.1607|20.2771|19.7499|0.658x|
|500,000|sbilanciato, pochi errori locali|8.6725|10.0639|9.7808|9.2767|0.935x|
