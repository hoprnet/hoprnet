# Tickets

When your node is used to relay messages, it is rewarded in the form of payment tickets. Tickets are generated as part of HOPR's proof-of-relay protocol, when a node receives two matching key halves. Tickets can be redeemed for HOPR.

To minimize the numbers of calls to the blockchain, future versions of HOPR will employ probabilistic payments, where not all tickets are redeemable. In the current version, however, the winning probabiliy is 100%, meaning every succesful relay will generate a ticket.

To redeem tickets, type `redeemTickets`. Your node will then attempt to redeem all the tickets it has earned. The earned HOPR will be added to you balance, which can be checked by typing `balance`.

Redeeming tickets involves calling the HOPR smart contract, so there is an associated gas fee. If the redemption fails, it could be that your don't have a sufficient balance of native tokens.
