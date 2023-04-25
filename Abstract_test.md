# FHE with LWE

## Abstract

Régóta téma a Kriptográfiában a FHE ötlete, előszőr Gentry mutatta meg hogy meg is valósítható. Mi egy rácsproblémákra épülő FHE-et megvalósító Pythoon könyvtárban implementáltunk egy algoritmust ami Adás/Vételt bonyolít le anélkül hogy felfedné az adók/vevők adatait.

## Teljesen homomorfikus titkosítás

A FHE vagyis Fully Homomorphic Encryption ötlete az hogy létezik-e olyan $\varphi$ titkosítás hogy az homomorfikus bizonyos műveltekre (megőrzi azokat), vagyis
$$ \exists \varphi : M \to C  $$
$$ \forall m,n \in M: \varphi(m \circ n) = varphi(m) \circ varphi(n) $$
ahol $\circ$ valamilyen művelet ami értelmezve van $C$ felett.

