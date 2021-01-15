```eval_rst
.. DANGER::
   These docs are currently being updated to reflect major changes to our software since our last release. In the meantime, you can visit our `github <https://www.github.com/hoprnet/hoprnet>`_ for basic instructions on how to install and run a node.
```

# HOPR

The HOPR ecosystem is a two-platform network with dynamic components powering its communication and incentivization mechanisms.

In one side, we have the **HOPR Core,** the privacy-networking module able to communicate and transfer messages securely. In the other side, we have a **Payment Gateway**, which is a Distributed Ledger Technology \(DLT\) or Blockchain infrastructure able to open payment channels on behalf of nodes running in the HOPR Network.

In its first implementation, HOPR relies on the **Ethereum Blockchain** as its first payment gateway using **Ethereum Smart Contracts.** Using Ethereum Smart Contracts**,** we can open **State Channels** on behalf of the relayers while forwarding messages. Senders of the messages then attach **\$HOPR** tokens in their messages, which upon successful delivery, are deducted and paid to the relayers involved.

To implement this process, a HOPR node implements a **Connector Interface** that communicates to the Ethereum network using its popular web library, **Web3.js.** These interfaces allow HOPR nodes to monitor, approve, sign and verify when a message is transfered, and thus close a State Channel and get their \$HOPR earned. Each node verify each other, avoiding foul play and rewarding only **Honest Relayers**.

![](./images/paper.bloc.8-2.png)

Although the first interaction of the HOPR network is on the Ethereum network, HOPR is by design **Chain Agnostic,** which means that HOPR nodes can eventually implement different payment channels in different Blockchains. At the time of writing, HOPR is also able to implement a [Polkadot-enabled payment gateway.](https://github.com/hoprnet/hopr-polkadot)

## Contents

```eval_rst
.. toctree::
   :maxdepth: 1
   :caption: Installing HOPRd

   src/install-hoprd/using-script.md
   src/install-hoprd/using-avado.md
   src/install-hoprd/using-npm.md
   src/install-hoprd/using-docker.md
```
