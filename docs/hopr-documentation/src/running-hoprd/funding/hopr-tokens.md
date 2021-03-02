# HOPR on Ethereum mainnet

The HOPR mainnet token, "HOPR Token", is deployed on Ethereum mainnet on [0xF5581dFeFD8Fb0e4aeC526bE659CFaB1f8c781dA](https://etherscan.io/address/0xF5581dFeFD8Fb0e4aeC526bE659CFaB1f8c781dA), denoted as "HOPR".

Since the Ethereum blockchain is still expensive to use due to high gas fees, the HOPR token can be also used on xDAI, a side-chain of Ethereum, by converting them to xHOPR. The "xHOPR Token" is deployed in address [0x12481c3Ed97b32D94E71C2039DBC44432ADD39a0](https://blockscout.com/poa/xdai/address/0x12481c3Ed97b32D94E71C2039DBC44432ADD39a0/transactions) on xDAI.

# xHOPR on xDAI

When sending "mainnet" HOPR through a bridge between Ethereum and xDAI, the mainnet "HOPR token", initially a ERC777 token and fully compatible with ERC20 becomes a ERC677 token on xDAI.

```eval_rst
.. ATTENTION::
   The HOPR client is still under development and not all issues are fixed. We recommend to not add more than 10 wxHOPR and 1 xDAI to it.
```

In order to use it with HOPR and the payment channel logic on xDAI, it need to be wrapped into a ERC777 token on xDAI, namely "wxHOPR" - "wrapped xHOPR". This *can* be done using the wrapper logic on [wrapper.hoprnet.org](https://wrapper.hoprnet.org)

[![](../../../images/wxhopr.png)](https://wrapper.hoprnet.org)

After wrapping the tokens from *xHOPR* to *wxHOPR* they can be used within the HOPR client and will appear in the client when typing `balance`.


