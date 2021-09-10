# Cryptography Explainer

## Concepts

### Groups:
A binary operation `*`  on a set `G` is a mapping from GxG to `G`, which associates to elements `x` and y of G a third element `x*y` of `G`.

**Definition:** A group `(G,)` consists of a set `G` together with a binary operation  for which the following properties are satisfied:

1. Associativity:
<img src="https://render.githubusercontent.com/render/math?math=(x\star y)\star z=x\star (y\star z),\forall x,y,z\in G ">

2. Neutral element:
<img src="https://render.githubusercontent.com/render/math?math=\exists!e\in G, e\star x=x=x\star e, \forall x\in G ">

3. Inverse element:
<img src="https://render.githubusercontent.com/render/math?math=\forall x\in G,\exists !x'\in G, x\star x'=e=x'\star x "> 

where `e` is the neutral element of `G`. A group `G` is **abelian** or **(commutative)** if :

<img src="https://render.githubusercontent.com/render/math?math=x\star y=y\star x, \forall x,y\in G ">


**Finite groups**
Let `n=#G`= number of elements in G. Then
<img src="https://render.githubusercontent.com/render/math?math=g^n=e, \forall g\in G ">
 


### Cyclic groups

**Definition:** A group `G` is said to be cyclic, with generator `g`, if every element of `G` is of the form `g^x` for some integer `x`.

A finite group `G`  of `n` elements is cyclic, if there exist an element (or elements) 
<img src="https://render.githubusercontent.com/render/math?math=g\in G"> with <img src="https://render.githubusercontent.com/render/math?math=\{g,g^2,g^3,\dots g^n=e\}=G"> and `g` is a generator of `G`.

### Cyclic Groups & Cryptographic Applications

The security of many cryptographic techniques depends on the intractability of the discrete logarithm problem which has no efficient solution.

**Discrete logarithm problem:** If `G` is a cyclic group and `g` is a generator of `G`, then the discrete logarithm of <img src="https://render.githubusercontent.com/render/math?math=a\in G">  with basis `g`, denoted <img src="https://render.githubusercontent.com/render/math?math=log_ga">, is the unique number <img src="https://render.githubusercontent.com/render/math?math=i\in \{0,\dots |G|-1\}"> such that <img src="https://render.githubusercontent.com/render/math?math=a=g^i"> and `|G|` is the order of the group `G` (number of elements in the group).
Fixing `G` and `g`, the discrete logarithm (DLOG) problem is, given a random <img src="https://render.githubusercontent.com/render/math?math=a\in G"> , find <img src="https://render.githubusercontent.com/render/math?math=log_ga">. We say that the problem is hard for if for every <img src="https://render.githubusercontent.com/render/math?math=poly A, \epsilon Pr_{a\leftarrow R}G[ \, A(G,g,a)=log_ga] \, < \epsilon">. 


## Diffie-Hellman key exchange
The Diffie-Hellman protocol is a method for two computer users to generate a shared private key with which they can then exchange information across an insecure channel. Let the users be named Alice and Bob. First, they agree on two prime numbers `g` and `p`, where `p` is large (typically at least 512 bits) and `g` is a primitive root modulo `p`. (In practice, it is a good idea to choose `p` such that `(p-1)/2` is also prime.) The numbers `g` and `p` need not be kept secret from other users. 
1. Now Alice chooses a large random number `a` as her private key 
2. And Bob similarly chooses a large number `b`.
3. Alice then computes <img src="https://render.githubusercontent.com/render/math?math=A=g^a(mod p)">, which she sends to Bob
4. Bob computes <img src="https://render.githubusercontent.com/render/math?math=B=g^b(mod p)">, which he sends to Alice.
5. Now both Alice and Bob compute their shared key <img src="https://render.githubusercontent.com/render/math?math=K=g^{ab}(mod p)"> , which Alice computes as: <img src="https://render.githubusercontent.com/render/math?math=K=B^a(mod p)=(g^b)^a(mod p)">.
and Bob computes as: <img src="https://render.githubusercontent.com/render/math?math=K=A^b(mod p)=(g^a)^b(mod p)">.
Alice and Bob can now use their shared key `K` to exchange information without worrying about other users obtaining this information.
The security of Diffie-Hellman is based upon the hardness of solving the DLOG Problem.


## Cyclic groups and Elliptic curves

### Elliptic-Curve cryptography and ECDSA signatures
The purpose of this section is to introduce elliptic curves as they are used in cryptography. Put simply, an elliptic curve is an abstract type of group.
To understand elliptic curve groups in cryptography, the reader should be familiar with the basics of finite fields `F_q`
This is because, more generally, elliptic curves are groups which are defined on top of (over) fields
Even though elliptic curve groups permit only one binary operation (the so-called group law), the operation itself is computed within the underlying field, which by definition permits two operations (and their inverses).
Finite fields satisfy a few properties, see Wikipedia, and have a finite amount of elements, e.g. <img src="https://render.githubusercontent.com/render/math?math=(0, 1, \dots, p-1)">  where `p` is the field order. We denote that field as `F_p`. A point `P` is therefore given as <img src="https://render.githubusercontent.com/render/math?math=P = (x,y)\in F_p\times F_p"> .


**Definition:** An Elliptic Curve is a curve given by an equation of the form: 
<img src="https://render.githubusercontent.com/render/math?math=y^2=x^3%2BAx%2BB"> (the short Weierstrass equation)


The elliptic curve points over <img src="https://render.githubusercontent.com/render/math?math=F_pF_p"> form a field, so we can use addition, meaning `Q=P+R` where <img src="https://render.githubusercontent.com/render/math?math=P, Q, R\in F_p\times F_p"> and `M=c*P` where 
 <img src="https://render.githubusercontent.com/render/math?math=M,P\in F_p\times F_p"> and <img src="https://render.githubusercontent.com/render/math?math=c\in F_p">. Note that <img src="https://render.githubusercontent.com/render/math?math=cP=P%2B(c-1)P=P%2BP%2B(c-2)P"> which means that multiplication is done by repeated addition of elliptic curve points. This can be optimized by using fast exponentiation. Exponentiation can be seen here as a synonym of multiplication.

The wording exponentiation and discrete logarithms comes from the time when asymmetric cryptography used finite fields like `F_p` where `p` is a very big prime number, e.g. **4096 bits**. Note that in groups like <img src="https://render.githubusercontent.com/render/math?math=F_p^* "> inverting an exponentiation which is also known as computing a discrete logarithm is computationally infeasible whereas for exponentiation we know efficient algorithms.

For elliptic curves, we have multiplication which is efficient to compute whereas division is believed to be infeasible to compute.

Note that there exists as well multiplication of two elliptic curve points, namely <img src="https://render.githubusercontent.com/render/math?math=S=T\times U "> where <img src="https://render.githubusercontent.com/render/math?math=T,U\in E_0"> and <img src="https://render.githubusercontent.com/render/math?math=S\in E_1">  where `E_0`, `E_1` are two potentially different elliptic curves. This operation is called **pairing** and we still don’t know an efficient way to compute it.