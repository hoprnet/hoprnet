# Cryptography Explainer

## Concepts

### Groups:
A binary operation `*`  on a set `G` is a mapping from GxG to `G`, which associates to elements `x` and y of G a third element `x*y` of `G`.

**Definition:** A group `(G,)` consists of a set `G` together with a binary operation  for which the following properties are satisfied:

1. Associativity:
![formula](https://render.githubusercontent.com/render/math?math=(x\times y)\times z=x\times (y\times z), 	\forall x,y,z\in G)
2. Neutral element:
![formula](https://render.githubusercontent.com/render/math?math=\exists!e\in G, e\times x=x=x\times e, \forall x\in G)
3. Inverse element:
![formula](https://render.githubusercontent.com/render/math?math=\forall x\in G,\exists !x'\in G, x\times x'=e=x'\times x) where `e` is the neutral element of `G`.
A group `G` is **abelian** or **(commutative)** if :
![formula](https://render.githubusercontent.com/render/math?math=xy=yx, x,yG.)


**Finite groups**
Let n=#G= number of elements in G. Then
gn=e for all gG. 


### Cyclic groups

**Definition:** A group `G` is said to be cyclic, with generator g, if every element of `G` is of the form `gx for some integer x.


A finite group G  of n elements is cyclic, if there exist an element (or elements) gG with g,g2,g3,...............gn=e=G and g is a generator of G.

### Cyclic Groups & Cryptographic Applications

The security of many cryptographic techniques depends on the intractability of the discrete logarithm problem which has no efficient solution.

Discrete logarithm problem: If G is a cyclic group and g is a generator of G, then the discrete logarithm of aG with basis g, denoted logga, is the unique number i0,........G-1 such that a=gi and G is the order of the group G (number of elements in the group).
Fixing G and g, the discrete logarithm (DLOG) problem is, given a random aG, find logga. We say that the problem is hard for if for every poly A, , PraRGA(G,g,a)=logga<. 


Diffie-Hellman key exchange
The Diffie-Hellman protocol is a method for two computer users to generate a shared private key with which they can then exchange information across an insecure channel. Let the users be named Alice and Bob. First, they agree on two prime numbers g and p, where p is large (typically at least 512 bits) and g is a primitive root modulo p. (In practice, it is a good idea to choose p such that (p-1)/2 is also prime.) The numbers g and p need not be kept secret from other users. 
Now Alice chooses a large random number a as her private key 
And Bob similarly chooses a large number b.
Alice then computes A=ga(mod p), which she sends to Bob
Bob computes B=gb(mod p), which he sends to Alice.
Now both Alice and Bob compute their shared key K=gab(mod p), which Alice computes as:  K=Ba(mod p)=(gb)a(mod p).
and Bob computes as: K=Ab(mod p)=(ga)b(mod p).
Alice and Bob can now use their shared key K to exchange information without worrying about other users obtaining this information.
The security of Diffie-Hellman is based upon the hardness of solving the DLOG Problem.


### Cyclic groups and Elliptic curves
Elliptic-Curve cryptography and ECDSA signatures
The purpose of this section is to introduce elliptic curves as they are used in cryptography. Put simply, an elliptic curve is an abstract type of group.
To understand elliptic curve groups in cryptography, the reader should be familiar with the basics of finite fields Fq
This is because, more generally, elliptic curves are groups which are defined on top of (over) fields
Even though elliptic curve groups permit only one binary operation (the so-called group law), the operation itself is computed within the underlying field, which by definition permits two operations (and their inverses).
Finite fields satisfy a few properties, see Wikipedia, and have a finite amount of elements, e.g. (0, 1, ..., p-1) where p is the field order. We denote that field as Fp. A point P is therefore given as  P = (x,y) FpFp.


Definition: An Elliptic Curve is a curve given by an equation of the form: y2=x3+Ax+B (the short Weierstrass equation)


The elliptic curve points over FpFp form a field, so we can use addition, meaning Q=P+R where P, Q, RFpFp and M=c*Pwhere M,PFpFp and cFp. Note that cP=P+(c-1)P=P+P+(c-2)Pwhich means that multiplication is done by repeated addition of 
elliptic curve points. This can be optimized by using fast exponentiation. Exponentiation can be seen here as a synonym of multiplication.

The wording exponentiation and discrete logarithms comes from the time when asymmetric cryptography used finite fields like Fp where p is a very big prime number, e.g. 4096 bits. Note that in groups like Fp* which is Fp\{0}, inverting an exponentiation which is also known as computing a discrete logarithm is computationally infeasible whereas for exponentiation we know efficient algorithms.

For elliptic curves, we have multiplication which is efficient to compute whereas division is believed to be infeasible to compute.

Note that there exists as well multiplication of two elliptic curve points, namely S=TUwhere T,UE0 and SE1 where E0, E1 are two potentially different elliptic curves. This operation is called pairing and we still don’t know an efficient way to compute it.


Within HOPR, we are using the secp256k1 curve y2=x3+7 (mod p) where 
p = 2256 - 232 - 29 - 28 - 27 - 26 - 24 - 1 is the field order.
We can plot the function xx3+7 over the real numbers (-1,0,2,) which gives us a graphical overview.


