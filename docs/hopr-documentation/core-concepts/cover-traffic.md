# Cover Traffic

The HOPR mainnet will be an incentivized decentralized mixnet. For a decentralized mixnet to be truly private and secure, we need what’s known as cover traffic. This is arbitrary data which is transmitted through the network to provide cover for the real data which users are sending between each other. This is particularly important at less busy times, when users are more exposed. With data privacy, there’s always safety in a crowd. Cover traffic ensures that there’s always a crowd to get lost in.

With this layer of extra data constantly moving through the network, outside observers are unable to extract any metadata about who is using the network and how much data is being sent through it.

You can test cover traffic in your HOPR node by typing `covertraffic start`. Your node will then start to stream messages via open payment channels and back to itself. You can check the status of this by typing `covertraffic stats`. Type `covertraffic stop` to stop cover traffic.
