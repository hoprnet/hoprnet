The idea is to let each party pay for forwarding a packet to the next hop. Since the next hop will only forward the packet if it contains a payment, the protocol ensures that the nodes only get paid for forwarding the packet in the direction of the designated receiver

# Per Hop Payment Scheme
Every packet header includes not only the address of the subsequent node R_{i+1} but also a transaction that pays a fee to that node for relaying the message to node R_{i+2}.

The transaction will be encrypted in a way such that the party R_{i+1} is only able to decrypt it with the help of party R_{i+2}. Once party R_{i+2} receives the correct message, it will acknowledge the package and thereby allow party R_{i+1} to decrypt the previously received transaction.

After that, party R_{i+1}, will become the sender of the next message and party R_{i+2} will be the next relay node. Party R_{i+2} will only send the acknowledgement of the previous message back to R_{i+1} if the current message includes a transaction that pays to R_{i+2}.

# Cash flow
Assume that the network makes use of _r_ relay nodes, i. e. the traffic goes over _r_ intermediate nodes until it finally receives its destination. Assume further that the transaction fee for every hop is _c_. Let R_0 -> R_1 -> R_2 -> ... -> R_r be route of the packet.

When using a per-hop payment scheme, party R_{1} will receive _r_ times the fee. As R_1 is unable to decrypt the transaction by himself, he need to forward the packet to R_2 in combination with a transaction that pays _r-1_ times the fee to R_2. By chaining this principle, the packet will eventually reach its designated receiver since every intermediate has a financial interest to forward the message in order to get payed.

# Problems
Not all of the nodes may act honestly, some of them may act maliciously to increase their outcome by deviating from the protocol.

In contrast to previous protocols, it is usually problematic when nodes decide to quit the protocol because in this case, it usually happens that some nodes don't get payed for the service.

For that reason, the payment scheme has to provide security against nodes that
* decide to drop messages
* send wrong messages
* forget about long-term secret such as their secret keys

In case of the previously described protocol, it is problematic when nodes does not
* embed a valid transaction transaction that pays the fee to the relay
* acknowledge a successfully send packet

Both of the previously described problems rely on the problem that two parties have when they engage in a protocol with each other and the final result depends on the last message that is sent to one of the parties. One approach is to prevent such a final message by splitting the final result into multiple messages such that the last message becomes less important. This seems to be a bit unhandy since this approach requires a lot of communication between the parties and the message overhead of the protocol should be as low as possible to keep the availability requirements of each node as low as possible.

Since the payments also depend on the services of the recipient, which is in this case the forwarding of the packet, it seems like the protocol is an instance of the the so-called fair exchange problem which is shown to be unsolvable without a trusted third party that acts as a judge in order to punish a party that deviates from the protocol.

# Blockchains as a trusted third party
An approach to solve this problem without a trusted third party is to use a blockchain to resolve conflicts. In the context of Ethereum this could be done by appropriate smart contracts that enforce the correct behaviour of the clients.

The idea is to use so-called _optimisitc_ fair exchange. The parties act as if each party follows the protocol but each party is able to detect inappropriate behaviour of the previous and the subsequent node.

Once a party detects such a behaviour it calls the blockchain to punish the nodes for their bad behaviour and/or force them to provide missing messages. The protocol must also punish misuse of that mechanism in order to prevent potentially malicious parties that try to spam other parties or try to decrease their outcome by enforcing them to spend unnecessary transaction fees.

The smart contracts must thereforce be realized in a way such that they punish misuse. An idea is to model this behaviour like a trial at court and let the losing party pay for the process costs.

# Smart contracts
It seems to be good to have the following smart contracts: relays calls the InvalidTransactionContract when they are able to derive the correct opening keys but detect that the signature of the embedded transaction is invalid / or something else is wrong. The relays will call the NoAckContract in case that parties forget to send the acknowledgement message back. It may also happen that parties send a wrong acknowledgement message.

## InvalidTransactionContract
The party got the correct acknowledgement from the subsequent party and notices that the derived keys yield to an invalid signature.

Let ... -> P_A -> P_B -> P_C -> ...

The key is given as k_{BC} = H(k_B \oplus k_C) where k_B is the key that P_B is able to derive, k_C is derivable by party P_C, H is a collision-resistant hash function. The parties are also able to derive a blinding factor k_R^B respectively k_R^C where k_R^B and k_R^C are computionally indistinguishable from k_B respectively k_C.

Party P_A will get k_{BC} from the original sender through the header of the packet. He will also get the values k_B \oplus k_R^B and k_R^B \oplus k_C and checks that H((k_B \oplus k_R^B) \oplus (k_R^B \oplus k_C)) = H(k_B \oplus k_C) = k_{BC}. The blinding key k_R^B is mandatory since A might collude with C and using the plain key would allow A figure out that the message went over B to C.

Party will then sign the following:
H(k_B \oplus k_R^B) || H(k_B \oplus k_C) || H(k_{BC}) || k_{BC} \oplus Sig_A(Tx_A)

By this, party A will commit to the derivation of k_{BC} and give party B the opportunity to prove to a smart contract the he has
* correctly derived k_{BC}
* that the signature is invalid although he has derived the correct keys

The smart contract will then derive the key again, calculate the corresponding hash values and derive the required key. The contract will then check whether the key matches and whether the transaction is correct respectively the signature is valid. The smart contract will be executed trustfully since it is run on the blockchain and other miners won't accept a packet with a wrong execution of that smart contract.

In case that the signature of the transaction is invalid, the smart contract will detect that party A has embedded a wrong transaction. The smart contract will then transfer the requested money to B and withdraws the transaction fees from the account of A.

Otherwise it will let party B pay the transaction fees in order to punish misuse of the contract.

## WrongAcknowledgement
In order to get the correct acknowledgement which should include the key k_C, party P_B will send P_C a challenge to provide a value k'_C such that H(k'_C) = H(k_C). Since H is a collision-resistant hash function (and this implies that it is also hard to find pre-images) it will be infeasible for party P_C to answer that challenge correctly with a different value than k_C.

So, party B will send Sig_B(H(k_C)) || H(k_C) to party P_C and party will respond to that challenge with Sig_C(k_C || Sig_B(H(k_C))) || k_C and discloses thereby k_C that allows party P_B to decrypt the transaction of party P_A.

It may also happen that party P_C answers for some reason the challenge with a wrong acknowledgement message that contains an invalid key k_C. Party P_B will only accept messages that are signed by P_C, so the message will contain a signature of P_C. In case that P_C has signed an incorrect key k'_C, party P_C will call the smart contract with Sig_C(k'_C || Sig_B(H(k_C))) and checks whether H(k'_C) = H(k_C). In case that this not the case, the smart contract will cancel the payment from P_B to P_C and let P_C pay the transaction fees.

Otherwise party P_B has to pay the transaction fees to prevent from misuse of that functionality.

## NoAcknowledgement
It may also happen that there is no acknowledgement message from P_C. In that case, party P_B will send the challenge Sig_B(H(k_C)) || H(k_C) also to the smart contract and give party P_C some blocks time to provide the requested key. Otherwise the smart contract will cancel the payment from P_B to P_C and let both parties pay the transaction fees.

To save transaction fees, party P_B can also wait until the payment channel between himself and party P_C becomes invalid and claim the requested money by closing the payment channel without the last update transaction that transfered the relay fees to party P_C