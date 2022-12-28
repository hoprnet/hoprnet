// SPDX-License-Identifier: GPL-3.0
pragma solidity >=0.6.0 <0.9.0;

import 'forge-std/Test.sol';
import './Channels.sol';
import './Accounts.sol';

contract TicketsUtilsTest is Test, AccountsFixtureTest, ChannelsUtilsTest {
  struct Ticket {
    address source;
    bytes32 nextCommitment;
    uint256 ticketEpoch;
    uint256 ticketIndex;
    bytes32 proofOfRelaySecret;
    uint256 amount;
    uint256 winProb;
    bytes signature;
  }

  bytes32 public PROOF_OF_RELAY_SECRET_0 = keccak256(abi.encodePacked('PROOF_OF_RELAY_SECRET_0'));
  uint256 public WIN_PROB_100 = type(uint256).max;
  uint256 public WIN_PROB_0 = type(uint256).min;

  /**
     *@dev a winning ticket issued by account A:

        {
            recipient: accountB.accountAddr,
            proofOfRelaySecret: PROOF_OF_RELAY_SECRET_0,
            ticketEpoch: '0',
            ticketIndex: '1',
            amount: '10',
            winProb: WIN_PROB_100.toString(),
            channelEpoch: '1',
            counterParty: accountA.accountAddr,
            nextCommitment: SECRET_1
        )
     */
  Ticket public TICKET_AB_WIN =
    Ticket({
      source: accountA.accountAddr,
      nextCommitment: SECRET_1,
      ticketEpoch: 0,
      ticketIndex: 1,
      proofOfRelaySecret: PROOF_OF_RELAY_SECRET_0,
      amount: 10,
      winProb: WIN_PROB_100,
      signature: hex'e28514db6bf62eab85e382e77a551639f616a51e527480dc922004a197b446006e396a671f35c69bbe966fbf26ccdd31da7722ea566fd34ba2724f6a10d7231b'
    });

  /**
     *@dev a winning ticket issued by account B:

        {
            recipient: accountA.accountAddr,
            proofOfRelaySecret: PROOF_OF_RELAY_SECRET_0,
            ticketEpoch: '0',
            ticketIndex: '1',
            amount: '10',
            winProb: WIN_PROB_100.toString(),
            channelEpoch: '1',
            counterParty: accountB.accountAddr,
            nextCommitment: SECRET_1
        )
    */
  Ticket public TICKET_BA_WIN =
    Ticket({
      source: accountB.accountAddr,
      nextCommitment: SECRET_1,
      ticketEpoch: 0,
      ticketIndex: 1,
      proofOfRelaySecret: PROOF_OF_RELAY_SECRET_0,
      amount: 10,
      winProb: WIN_PROB_100,
      signature: hex'40e302cb0b8b18dbdd08ca1bfc93f1f2c40d5b93e8366dbd97f323aabb26e05f91b2e8828a3a15ffb39d10c55e3fbc2fca72c4d3a3083f09fe07797a6a9ecc54'
    });

  /**
     *@dev a loss ticket issued by account A:

        {
            recipient: accountB.accountAddr,
            proofOfRelaySecret: PROOF_OF_RELAY_SECRET_0,
            ticketEpoch: '0',
            ticketIndex: '1',
            amount: '10',
            winProb: WIN_PROB_0.toString(),
            channelEpoch: '1',
            counterParty: accountA.accountAddr,
            nextCommitment: SECRET_1
        )
    */
  Ticket public TICKET_AB_LOSS =
    Ticket({
      source: accountA.accountAddr,
      nextCommitment: SECRET_1,
      ticketEpoch: 0,
      ticketIndex: 1,
      proofOfRelaySecret: PROOF_OF_RELAY_SECRET_0,
      amount: 10,
      winProb: WIN_PROB_0,
      signature: hex'c81d8a3fe9d2dfbbf916bad5c3ff2acfb557c4972eb172f6441b85058e8cbd26b67afed7d5e72eae4e5bbf0b4ed9d949c0b06b0755b81b80742e4898f36fcc33'
    });

  /**
     *@dev a recycled wubbubg ticket issued by account A:

        {
            recipient: accountB.accountAddr,
            proofOfRelaySecret: PROOF_OF_RELAY_SECRET_0,
            ticketEpoch: '0',
            ticketIndex: '1',
            amount: '10',
            winProb: WIN_PROB_100.toString(),
            channelEpoch: '2',
            counterParty: accountA.accountAddr,
            nextCommitment: SECRET_1
        )
    */
  Ticket public TICKET_AB_WIN_RECYCLED =
    Ticket({
      source: accountA.accountAddr,
      nextCommitment: SECRET_1,
      ticketEpoch: 0,
      ticketIndex: 1,
      proofOfRelaySecret: PROOF_OF_RELAY_SECRET_0,
      amount: 10,
      winProb: WIN_PROB_100,
      signature: hex'43bbf7f4a28786e47be61b6e3c40f4ff95f214e0ac3b43b10d9d962a076e7e0f0a35e4a487ba460af46f9b061e3474c1af399a50033a3f6a48f84a279acdc981'
    });
}
