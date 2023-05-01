# FHE with LWE

## Abstract

Régóta téma a Kriptográfiában a FHE ötlete, előszőr Gentry mutatta meg hogy meg is valósítható. Mi egy rácsproblémákra épülő FHE-et megvalósító Pythoon könyvtárban implementáltunk egy algoritmust ami Adás/Vételt bonyolít le anélkül hogy felfedné az adók/vevők adatait.

Interest in FHE has been growing due to recent privacy conserns and the threat of Quantum computing. FHE allows for operations on encrypted data while being secure, even Quantum resistante. However implementations have suffered with slow performance and required too much space. Using ZAMA's TFHE-rs library we show that its possible to achive good performance with FHE librarys and also outline while FHE is important and how THFE achives full homomorphism. We show how we implemented a Dark Market algorithm in THFE-rs and 
what runtimes we could achive.

// Esetleg még egy két szót az algoritmusról, Conclusion

## Teljesen homomorfikus titkosítás

A FHE vagyis Fully Homomorphic Encryption ötlete az hogy létezik-e olyan $\varphi$ titkosítás hogy az homomorfikus bizonyos műveltekre (megőrzi azokat), vagyis
$$ \exists \varphi : M \to C  $$
$$ \forall m,n \in M: \varphi(m \circ n) = \varphi(m) \circ \varphi(n) $$
ahol $\circ$ valamilyen művelet ami értelmezve van $C$ felett.