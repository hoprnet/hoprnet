class Balance(object):
    """
    HOPR tokens from this balance is used to fund payment channels between this node and other nodes on the network.
    Either wxHOPR or mHOPR, depending on the network. These HOPR tokens fund payment channels/pay nodes to relay data.
    NATIVE balance (ETH) is used to pay for the gas fees for the blockchain network.
    This will show the native tokens used to pay gas fees, currently xDAI. For example, opening and closing payment
    channels would require on-chain transactions paid for in xDAI.
    """

    def __init__(self, native: int, hopr: int) -> None:
        self.native: int = native  # ETH
        self.hopr: int = hopr  # HOPR
