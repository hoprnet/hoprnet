// SPDX-License-Identifier: GPL-3.0
pragma solidity >=0.6.0 <0.9.0;

import 'forge-std/Test.sol';
import './Channels.sol';
import './Accounts.sol';
import '../../src/Channels.sol';

abstract contract TicketsUtilsTest is Test, AccountsFixtureTest, ChannelsUtilsTest {
  bytes32 public PROOF_OF_RELAY_SECRET_0 = keccak256(abi.encodePacked('PROOF_OF_RELAY_SECRET_0'));
  bytes32 public CHANNEL_AB_ID = getChannelId(accountA.accountAddr, accountB.accountAddr);
  bytes32 public CHANNEL_BA_ID = getChannelId(accountB.accountAddr, accountA.accountAddr);
  HoprChannels.WinProb public WIN_PROB_100 = HoprChannels.WinProb.wrap(type(uint56).max);
  HoprChannels.WinProb public WIN_PROB_0 = HoprChannels.WinProb.wrap(type(uint56).min);
  HoprChannels.TicketReserved public RESERVED = HoprChannels.TicketReserved.wrap(uint16(0));

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
  HoprChannels.TicketData public TICKET_AB_WIN =
    HoprChannels.TicketData({
      channelId: CHANNEL_AB_ID,
      amount: HoprChannels.Balance.wrap(uint96(10)),
      index: HoprChannels.TicketIndex.wrap(uint64(1)),
      epoch: HoprChannels.ChannelEpoch.wrap(uint24(0)),
      winProb: WIN_PROB_100,
      reserved: RESERVED
    });

  HoprChannels.CompactSignature public TICKET_AB_WIN_SIG = HoprChannels.CompactSignature({
    r: hex'e28514db6bf62eab85e382e77a551639f616a51e527480dc922004a197b44600',
    vs: hex'6e396a671f35c69bbe966fbf26ccdd31da7722ea566fd34ba2724f6a10d7231b'
  });

  HoprChannels.RedeemableTicket public REDEEMABLE_TICKET_AB_WIN = HoprChannels.RedeemableTicket({
    signature: TICKET_AB_WIN_SIG,
    data: TICKET_AB_WIN,
    opening: SECRET_1,
    porSecret:PROOF_OF_RELAY_SECRET_0
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
    HoprChannels.TicketData public TICKET_BA_WIN =
    HoprChannels.TicketData({
      channelId: CHANNEL_BA_ID,
      amount: HoprChannels.Balance.wrap(uint96(10)),
      index: HoprChannels.TicketIndex.wrap(uint64(1)),
      epoch: HoprChannels.ChannelEpoch.wrap(uint24(0)),
      winProb: WIN_PROB_100,
      reserved: RESERVED
    });

  HoprChannels.CompactSignature public TICKET_BA_WIN_SIG = HoprChannels.CompactSignature({
    r: hex'40e302cb0b8b18dbdd08ca1bfc93f1f2c40d5b93e8366dbd97f323aabb26e05f',
    vs: hex'91b2e8828a3a15ffb39d10c55e3fbc2fca72c4d3a3083f09fe07797a6a9ecc54'
  });

  HoprChannels.RedeemableTicket public REDEEMABLE_TICKET_BA_WIN = HoprChannels.RedeemableTicket({
    signature: TICKET_BA_WIN_SIG,
    data: TICKET_BA_WIN,
    opening: SECRET_1,
    porSecret:PROOF_OF_RELAY_SECRET_0
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
  HoprChannels.TicketData public TICKET_AB_LOSS =
    HoprChannels.TicketData({
      channelId: CHANNEL_AB_ID,
      amount: HoprChannels.Balance.wrap(uint96(10)),
      index: HoprChannels.TicketIndex.wrap(uint64(1)),
      epoch: HoprChannels.ChannelEpoch.wrap(uint24(0)),
      winProb: WIN_PROB_0,
      reserved: RESERVED
    });

  HoprChannels.CompactSignature public TICKET_BA_LOSS_SIG = HoprChannels.CompactSignature({
    r:  hex'c81d8a3fe9d2dfbbf916bad5c3ff2acfb557c4972eb172f6441b85058e8cbd26',
    vs: hex'b67afed7d5e72eae4e5bbf0b4ed9d949c0b06b0755b81b80742e4898f36fcc33'
  });

  HoprChannels.RedeemableTicket public REDEEMABLE_TICKET_AB_LOSS = HoprChannels.RedeemableTicket({
    signature: TICKET_BA_LOSS_SIG,
    data: TICKET_AB_LOSS,
    opening: SECRET_1,
    porSecret:PROOF_OF_RELAY_SECRET_0
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
  HoprChannels.TicketData public TICKET_AB_WIN_RECYCLED =
    HoprChannels.TicketData({
      channelId: CHANNEL_AB_ID,
      amount: HoprChannels.Balance.wrap(uint96(10)),
      index: HoprChannels.TicketIndex.wrap(uint64(1)),
      epoch: HoprChannels.ChannelEpoch.wrap(uint24(0)),
      winProb: WIN_PROB_100,
      reserved: RESERVED
    });

  HoprChannels.CompactSignature public TICKET_BA_WIN_RECYCLED_SIG = HoprChannels.CompactSignature({
    r: hex'43bbf7f4a28786e47be61b6e3c40f4ff95f214e0ac3b43b10d9d962a076e7e0f',
    vs: hex'0a35e4a487ba460af46f9b061e3474c1af399a50033a3f6a48f84a279acdc981'
  });

  HoprChannels.RedeemableTicket public REDEEMABLE_TICKET_AB_RECYCLED = HoprChannels.RedeemableTicket({
    signature: TICKET_BA_WIN_RECYCLED_SIG,
    data: TICKET_AB_WIN_RECYCLED,
    opening: SECRET_1,
    porSecret:PROOF_OF_RELAY_SECRET_0
  });
}
