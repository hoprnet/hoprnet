// SPDX-License-Identifier: UNLICENSED
pragma solidity >=0.6.0 <0.9.0;

import './utils/ERC1820Registry.sol';
import './utils/Accounts.sol';
import './utils/Channels.sol';
import './utils/Tickets.sol';
import '../src/HoprChannels.sol';
import 'forge-std/Test.sol';

contract HoprChannelsTest is
  Test,
  ERC1820RegistryFixtureTest,
  AccountsFixtureTest,
  ChannelsUtilsTest,
  TicketsUtilsTest
{
  HoprChannels public hoprChannels;
  bytes32 channelIdAB = getChannelId(accountA.accountAddr, accountB.accountAddr);
  bytes32 channelIdBA = getChannelId(accountB.accountAddr, accountA.accountAddr);

  // uint256 public globalSnapshot;

  function setUp() public virtual override {
    super.setUp();
    // make vm.addr(1) HoprToken contract
    hoprChannels = new HoprChannels(vm.addr(1), 15);
  }

  function testAnnounceAddressFromPublicKey() public {
    bytes memory multiAddress = hex'1234';
    vm.prank(accountA.accountAddr);
    vm.expectEmit(true, false, false, true, address(hoprChannels));
    emit Announcement(accountA.accountAddr, accountA.publicKey, multiAddress);
    hoprChannels.announce(accountA.publicKey, multiAddress);
  }

  function testRevert_AnnouceWrongPublicKey() public {
    bytes memory multiAddress = hex'1234';
    vm.prank(accountB.accountAddr);
    vm.expectRevert("publicKey's address does not match senders");
    hoprChannels.announce(accountA.publicKey, multiAddress);
  }

  function testRevert_AnnouceRandomPublicKey(bytes calldata randomPublicKey) public {
    vm.assume(keccak256(randomPublicKey) != keccak256(accountB.publicKey));
    bytes memory multiAddress = hex'1234';
    vm.prank(accountB.accountAddr);
    vm.expectRevert("publicKey's address does not match senders");
    hoprChannels.announce(randomPublicKey, multiAddress);
  }

  // // it should fail to fund without accountA announcement
  function testRevert_FundChannelMultiWithoutAccountAAnnoucement(uint256 amount1) public {
    amount1 = bound(amount1, 1, 1e36);
    // accountA is not annouced and only accountB is announced
    _helperAnnounceAB(false, true);

    vm.prank(address(1));
    vm.expectRevert('source has not announced');
    hoprChannels.fundChannelMulti(accountA.accountAddr, accountB.accountAddr, amount1, 0);
  }

  // it should fail to fund without accountB announcement
  function testRevert_FundChannelMultiWithoutAccountBAnnoucement(uint256 amount1) public {
    amount1 = bound(amount1, 1, 1e36);
    // accountB is not annouced and only accountA is announced
    _helperAnnounceAB(true, false);

    vm.prank(address(1));
    vm.expectRevert('destination has not announced');
    hoprChannels.fundChannelMulti(accountA.accountAddr, accountB.accountAddr, amount1, 0);
  }

  function testFundChannelMulti(uint256 amount1, uint256 amount2) public {
    amount1 = bound(amount1, 1, 1e36);
    amount2 = bound(amount2, 1, 1e36);

    // channels
    HoprChannels.Channel memory channelAB = HoprChannels.Channel(
      amount1,
      bytes32(0),
      0,
      0,
      HoprChannels.ChannelStatus.WAITING_FOR_COMMITMENT,
      1,
      0
    );
    HoprChannels.Channel memory channelBA = HoprChannels.Channel(
      amount2,
      bytes32(0),
      0,
      0,
      HoprChannels.ChannelStatus.WAITING_FOR_COMMITMENT,
      1,
      0
    );

    // fund channel for two parties triggers
    vm.expectEmit(true, true, false, true, address(hoprChannels));
    emit ChannelUpdated(accountA.accountAddr, accountB.accountAddr, channelAB);
    vm.expectEmit(true, true, true, true, address(hoprChannels));
    emit ChannelFunded(address(1), accountA.accountAddr, accountB.accountAddr, amount1);
    vm.expectEmit(true, true, false, true, address(hoprChannels));
    emit ChannelUpdated(accountB.accountAddr, accountA.accountAddr, channelBA);
    vm.expectEmit(true, true, true, true, address(hoprChannels));
    emit ChannelFunded(address(1), accountB.accountAddr, accountA.accountAddr, amount2);

    // announc account A and B. Fund multi for A->B and B->A
    _helperFundMultiAB(amount1, amount2);
    // check vallidate from channels()
    assertEqChannels(getChannelFromTuple(hoprChannels, channelIdAB), channelAB);
    assertEqChannels(getChannelFromTuple(hoprChannels, channelIdBA), channelBA);
  }

  function testFailFundChannelMulti_SameSourceAndDestination(uint256 amount1, uint256 amount2) public {
    amount1 = bound(amount1, 1, 1e36);
    amount2 = bound(amount2, 1, 1e36);
    // accountA is announced
    _helperAnnounceAB(true, false);
    vm.prank(address(1));
    hoprChannels.fundChannelMulti(accountA.accountAddr, accountA.accountAddr, amount1, amount2);
  }

  function testFailFundChannelMulti_FromSourceZero(uint256 amount1, uint256 amount2) public {
    amount1 = bound(amount1, 1, 1e36);
    amount2 = bound(amount2, 1, 1e36);
    // accountA is announced
    _helperAnnounceAB(true, false);
    vm.prank(address(1));
    hoprChannels.fundChannelMulti(address(0), accountA.accountAddr, amount1, amount2);
  }

  function testFailFundChannelMulti_ToDestinationZero(uint256 amount1, uint256 amount2) public {
    amount1 = bound(amount1, 1, 1e36);
    amount2 = bound(amount2, 1, 1e36);
    // accountA is announced
    _helperAnnounceAB(true, false);
    vm.prank(address(1));
    hoprChannels.fundChannelMulti(accountA.accountAddr, address(0), amount1, amount2);
  }

  function testFailFundChannelMulti_AmountZero() public {
    // both accountA and accountB are announced
    _helperAnnounceAB(true, true);
    vm.prank(address(1));
    hoprChannels.fundChannelMulti(accountA.accountAddr, address(0), 0, 0);
  }

  function testBumpChannel(
    uint256 amount1,
    uint256 amount2,
    bytes32 secret
  ) public {
    amount1 = bound(amount1, 1, 1e36);
    amount2 = bound(amount2, 1, 1e36);
    vm.assume(secret != bytes32(0));

    _helperFundMultiAB(amount1, amount2);

    // channels
    HoprChannels.Channel memory channelAB = HoprChannels.Channel(
      amount1,
      bytes32(0),
      0,
      0,
      HoprChannels.ChannelStatus.WAITING_FOR_COMMITMENT,
      1,
      0
    );
    // ticket epoch is 1
    HoprChannels.Channel memory channelBA = HoprChannels.Channel(
      amount2,
      secret,
      1,
      0,
      HoprChannels.ChannelStatus.OPEN,
      1,
      0
    );
    vm.expectEmit(true, true, false, true, address(hoprChannels));
    emit ChannelUpdated(accountB.accountAddr, accountA.accountAddr, channelBA);
    vm.expectEmit(true, true, false, true, address(hoprChannels));
    emit ChannelBumped(accountB.accountAddr, accountA.accountAddr, secret, 1, amount2);

    // accountA bump channel B->A with a non-zero secret
    vm.prank(accountA.accountAddr);
    hoprChannels.bumpChannel(accountB.accountAddr, secret);

    // check vallidate from channels()
    assertEqChannels(getChannelFromTuple(hoprChannels, channelIdAB), channelAB);
    assertEqChannels(getChannelFromTuple(hoprChannels, channelIdBA), channelBA);
  }

  function testBumpChannelWithPreCommitment(
    uint256 amount1,
    uint256 amount2,
    bytes32 secret
  ) public {
    amount1 = bound(amount1, 1, 1e36);
    amount2 = bound(amount2, 1, 1e36);
    vm.assume(secret != bytes32(0));
    // Firstly, accountA bumps channel B->A with a non-zero secret
    vm.prank(accountA.accountAddr);
    hoprChannels.bumpChannel(accountB.accountAddr, secret);

    // channels
    HoprChannels.Channel memory channelAB = HoprChannels.Channel(
      amount1,
      bytes32(0),
      0,
      0,
      HoprChannels.ChannelStatus.WAITING_FOR_COMMITMENT,
      1,
      0
    );
    // ticket epoch is 1
    HoprChannels.Channel memory channelBA = HoprChannels.Channel(
      amount2,
      secret,
      0,
      0,
      HoprChannels.ChannelStatus.OPEN,
      1,
      0
    );
    // order of logs matters
    vm.expectEmit(true, true, false, true, address(hoprChannels));
    emit ChannelUpdated(accountA.accountAddr, accountB.accountAddr, channelAB);
    vm.expectEmit(true, false, false, false, address(hoprChannels));
    emit ChannelOpened(accountB.accountAddr, accountA.accountAddr);
    vm.expectEmit(true, true, false, true, address(hoprChannels));
    emit ChannelUpdated(accountB.accountAddr, accountA.accountAddr, channelBA);

    // then fund channel
    _helperFundMultiAB(amount1, amount2);

    // check vallidate from channels()
    assertEqChannels(getChannelFromTuple(hoprChannels, channelIdAB), channelAB);
    assertEqChannels(getChannelFromTuple(hoprChannels, channelIdBA), channelBA);
  }

  function testBumpChannelWithBothCommitments(
    uint256 amount1,
    uint256 amount2,
    bytes32 secret1,
    bytes32 secret2
  ) public {
    amount1 = bound(amount1, 1, 1e36);
    amount2 = bound(amount2, 1, 1e36);
    vm.assume(secret1 != bytes32(0));
    vm.assume(secret2 != bytes32(0));

    // Both accountA and accountB bump respective channels with secrets
    vm.prank(accountA.accountAddr);
    hoprChannels.bumpChannel(accountB.accountAddr, secret1);
    vm.prank(accountB.accountAddr);
    hoprChannels.bumpChannel(accountA.accountAddr, secret2);

    // channels
    HoprChannels.Channel memory channelAB = HoprChannels.Channel(
      amount1,
      secret2,
      0,
      0,
      HoprChannels.ChannelStatus.OPEN,
      1,
      0
    );
    // ticket epoch is 1
    HoprChannels.Channel memory channelBA = HoprChannels.Channel(
      amount2,
      secret1,
      0,
      0,
      HoprChannels.ChannelStatus.OPEN,
      1,
      0
    );
    // order of logs matters
    vm.expectEmit(true, true, false, true, address(hoprChannels));
    emit ChannelUpdated(accountA.accountAddr, accountB.accountAddr, channelAB);
    vm.expectEmit(true, false, false, false, address(hoprChannels));
    emit ChannelOpened(accountB.accountAddr, accountA.accountAddr);
    vm.expectEmit(true, true, false, true, address(hoprChannels));
    emit ChannelUpdated(accountB.accountAddr, accountA.accountAddr, channelBA);

    // then fund channel
    _helperFundMultiAB(amount1, amount2);

    // check vallidate from channels()
    assertEqChannels(getChannelFromTuple(hoprChannels, channelIdAB), channelAB);
    assertEqChannels(getChannelFromTuple(hoprChannels, channelIdBA), channelBA);
  }

  /**
   *@dev it should fund A->B using send
   */
  function testFundChannelABWithSend(uint256 amount) public {
    amount = bound(amount, 1, 1e36);
    // both accountA and accountB are announced
    _helperAnnounceAB(true, true);
    // expect to emit a channel
    HoprChannels.Channel memory channelBA = HoprChannels.Channel(
      amount,
      bytes32(0),
      0,
      0,
      HoprChannels.ChannelStatus.WAITING_FOR_COMMITMENT,
      1,
      0
    );
    vm.expectEmit(true, true, false, true, address(hoprChannels));
    emit ChannelUpdated(accountB.accountAddr, accountA.accountAddr, channelBA);
    vm.expectEmit(true, true, true, true, address(hoprChannels));
    emit ChannelFunded(accountB.accountAddr, accountB.accountAddr, accountA.accountAddr, amount);

    // mock token contract to call `tokensReceived()` on hoprChannels contract by account B
    vm.prank(vm.addr(1));
    hoprChannels.tokensReceived(
      address(0),
      accountB.accountAddr,
      address(hoprChannels),
      amount,
      abi.encode(accountB.accountAddr, accountA.accountAddr, amount, HoprChannels.ChannelStatus.CLOSED),
      hex'00'
    );
  }

  /**
   * @dev With single funded HoprChannels: AB: amount1, where amount1 is above 10,
     it should reedem ticket for account B -> directly to wallet
   */
  function testRedeemTicketWithSingleFundedHoprChannel(uint256 amount1) public {
    // channel is funded for at least 10 HoprTokens (Ticket's amount)
    amount1 = bound(amount1, TICKET_AB_WIN.amount, 1e36);
    // Firstly, accountB bumps channel A->B with SECRET_2
    vm.prank(accountB.accountAddr);
    hoprChannels.bumpChannel(accountA.accountAddr, SECRET_2);
    // then fund channel
    _helperFundMultiAB(amount1, 0);
    // mock token transfer from hoprChannels
    vm.prank(address(hoprChannels));
    vm.mockCall(
      vm.addr(1),
      abi.encodeWithSignature('transfer(address,uint256)', accountB.accountAddr, TICKET_AB_WIN.amount), // provide specific
      abi.encode(true)
    );

    // channels
    HoprChannels.Channel memory channelAB = HoprChannels.Channel(
      amount1,
      SECRET_2,
      0,
      0,
      HoprChannels.ChannelStatus.OPEN,
      1,
      0
    );
    // ticket epoch is 1
    HoprChannels.Channel memory channelBA = HoprChannels.Channel(
      0,
      bytes32(0),
      0,
      0,
      HoprChannels.ChannelStatus.CLOSED,
      0,
      0
    );
    // check vallidate from channels()
    assertEqChannels(getChannelFromTuple(hoprChannels, channelIdAB), channelAB);
    assertEqChannels(getChannelFromTuple(hoprChannels, channelIdBA), channelBA);

    vm.expectEmit(true, true, false, true, address(hoprChannels));
    emit TicketRedeemed(
      TICKET_AB_WIN.source,
      accountB.accountAddr,
      TICKET_AB_WIN.nextCommitment,
      TICKET_AB_WIN.ticketEpoch,
      TICKET_AB_WIN.ticketIndex,
      TICKET_AB_WIN.proofOfRelaySecret,
      TICKET_AB_WIN.amount,
      TICKET_AB_WIN.winProb,
      TICKET_AB_WIN.signature
    );

    // accountB redeem ticket
    vm.prank(accountB.accountAddr);
    hoprChannels.redeemTicket(
      TICKET_AB_WIN.source,
      TICKET_AB_WIN.nextCommitment,
      TICKET_AB_WIN.ticketEpoch,
      TICKET_AB_WIN.ticketIndex,
      TICKET_AB_WIN.proofOfRelaySecret,
      TICKET_AB_WIN.amount,
      TICKET_AB_WIN.winProb,
      TICKET_AB_WIN.signature
    );
    vm.clearMockedCalls();
  }

  /**
   * @dev With funded HoprChannels: AB: amount1, BA: amount 2,
   * where both amount1 and amount2 are above 10,
   * it should redeem ticket for account A
   * it should redeem ticket for account B
   */
  function testRedeemTicketWithDoubleFundedHoprChannel(uint256 amount1, uint256 amount2) public {
    // channel is funded for at least 10 HoprTokens (Ticket's amount)
    amount1 = bound(amount1, TICKET_AB_WIN.amount, 1e36);
    amount2 = bound(amount2, TICKET_BA_WIN.amount, 1e36);
    // Open channels A<->B with some tokens that are above possible winning tickets
    _helperOpenBidirectionalChannels(amount1, amount2);

    // channels before ticket redemption
    HoprChannels.Channel memory channelAB = HoprChannels.Channel(
      amount1,
      SECRET_2,
      0,
      0,
      HoprChannels.ChannelStatus.OPEN,
      1,
      0
    );
    // ticket epoch is 1
    HoprChannels.Channel memory channelBA = HoprChannels.Channel(
      amount2,
      SECRET_2,
      0,
      0,
      HoprChannels.ChannelStatus.OPEN,
      1,
      0
    );
    // check vallidate from channels()
    assertEqChannels(getChannelFromTuple(hoprChannels, channelIdAB), channelAB);
    assertEqChannels(getChannelFromTuple(hoprChannels, channelIdBA), channelBA);

    vm.expectEmit(true, true, false, true, address(hoprChannels));
    emit TicketRedeemed(
      TICKET_AB_WIN.source,
      accountB.accountAddr,
      TICKET_AB_WIN.nextCommitment,
      TICKET_AB_WIN.ticketEpoch,
      TICKET_AB_WIN.ticketIndex,
      TICKET_AB_WIN.proofOfRelaySecret,
      TICKET_AB_WIN.amount,
      TICKET_AB_WIN.winProb,
      TICKET_AB_WIN.signature
    );

    // create a snapshot
    uint256 snapshotBeforeAccountBRedeemTicket = vm.snapshot();

    // accountB redeem ticket
    vm.prank(accountB.accountAddr);
    hoprChannels.redeemTicket(
      TICKET_AB_WIN.source,
      TICKET_AB_WIN.nextCommitment,
      TICKET_AB_WIN.ticketEpoch,
      TICKET_AB_WIN.ticketIndex,
      TICKET_AB_WIN.proofOfRelaySecret,
      TICKET_AB_WIN.amount,
      TICKET_AB_WIN.winProb,
      TICKET_AB_WIN.signature
    );

    // channels after ticket redemption
    HoprChannels.Channel memory channelABAfterBRedeems = HoprChannels.Channel(
      amount1 - TICKET_AB_WIN.amount,
      SECRET_1,
      0,
      1,
      HoprChannels.ChannelStatus.OPEN,
      1,
      0
    );
    // ticket epoch is 1
    HoprChannels.Channel memory channelBAAfterRedeems = HoprChannels.Channel(
      amount2 + TICKET_AB_WIN.amount,
      SECRET_2,
      0,
      0,
      HoprChannels.ChannelStatus.OPEN,
      1,
      0
    );
    // check vallidate from channels()
    assertEqChannels(getChannelFromTuple(hoprChannels, channelIdAB), channelABAfterBRedeems);
    assertEqChannels(getChannelFromTuple(hoprChannels, channelIdBA), channelBAAfterRedeems);

    // reset to snapshot
    vm.revertTo(snapshotBeforeAccountBRedeemTicket);
    // accountA redeem ticket
    vm.prank(accountA.accountAddr);
    hoprChannels.redeemTicket(
      TICKET_BA_WIN.source,
      TICKET_BA_WIN.nextCommitment,
      TICKET_BA_WIN.ticketEpoch,
      TICKET_BA_WIN.ticketIndex,
      TICKET_BA_WIN.proofOfRelaySecret,
      TICKET_BA_WIN.amount,
      TICKET_BA_WIN.winProb,
      TICKET_BA_WIN.signature
    );

    // channels after ticket redemption
    HoprChannels.Channel memory channelBAAfterARedeems = HoprChannels.Channel(
      amount2 - TICKET_BA_WIN.amount,
      SECRET_1,
      0,
      1,
      HoprChannels.ChannelStatus.OPEN,
      1,
      0
    );
    // ticket epoch is 1
    HoprChannels.Channel memory channelABAfterARedeems = HoprChannels.Channel(
      amount1 + TICKET_BA_WIN.amount,
      SECRET_2,
      0,
      0,
      HoprChannels.ChannelStatus.OPEN,
      1,
      0
    );
    // check vallidate from channels()
    assertEqChannels(getChannelFromTuple(hoprChannels, channelIdAB), channelABAfterARedeems);
    assertEqChannels(getChannelFromTuple(hoprChannels, channelIdBA), channelBAAfterARedeems);
  }

  /**
   * @dev With funded HoprChannels: AB: amount1, BA: amount 2, where both amount1 and amount2 are above 10 (amount of TICKET_AB_WIN and TICKET_BA_WIN),
   * it should fail to redeem ticket when ticket has been already redeemed
   */
  function testFail_RedeemARedeemedTicket(uint256 amount1, uint256 amount2) public {
    // channel is funded for at least 10 HoprTokens (Ticket's amount)
    amount1 = bound(amount1, TICKET_AB_WIN.amount, 1e36);
    amount2 = bound(amount2, TICKET_BA_WIN.amount, 1e36);
    // Open channels A<->B with some tokens that are above possible winning tickets
    _helperOpenBidirectionalChannels(amount1, amount2);

    vm.expectEmit(true, true, false, true, address(hoprChannels));
    emit TicketRedeemed(
      TICKET_AB_WIN.source,
      accountB.accountAddr,
      TICKET_AB_WIN.nextCommitment,
      TICKET_AB_WIN.ticketEpoch,
      TICKET_AB_WIN.ticketIndex,
      TICKET_AB_WIN.proofOfRelaySecret,
      TICKET_AB_WIN.amount,
      TICKET_AB_WIN.winProb,
      TICKET_AB_WIN.signature
    );

    // accountB redeem ticket
    vm.prank(accountB.accountAddr);
    hoprChannels.redeemTicket(
      TICKET_AB_WIN.source,
      TICKET_AB_WIN.nextCommitment,
      TICKET_AB_WIN.ticketEpoch,
      TICKET_AB_WIN.ticketIndex,
      TICKET_AB_WIN.proofOfRelaySecret,
      TICKET_AB_WIN.amount,
      TICKET_AB_WIN.winProb,
      TICKET_AB_WIN.signature
    );

    // fail to redeem the redeemed ticket
    vm.expectRevert(bytes('redemptions must be in order'));
    hoprChannels.redeemTicket(
      TICKET_AB_WIN.source,
      SECRET_0,
      TICKET_AB_WIN.ticketEpoch,
      TICKET_AB_WIN.ticketIndex,
      TICKET_AB_WIN.proofOfRelaySecret,
      TICKET_AB_WIN.amount,
      TICKET_AB_WIN.winProb,
      TICKET_AB_WIN.signature
    );

    // fail to redeem the redeemed ticket
    vm.expectRevert(bytes('ticket epoch must match'));
    hoprChannels.redeemTicket(
      TICKET_AB_WIN.source,
      SECRET_0,
      TICKET_AB_WIN.ticketEpoch + 1,
      TICKET_AB_WIN.ticketIndex,
      TICKET_AB_WIN.proofOfRelaySecret,
      TICKET_AB_WIN.amount,
      TICKET_AB_WIN.winProb,
      TICKET_AB_WIN.signature
    );
  }

  /**
   *@dev Helper function to announce account A and B
   */
  function _helperAnnounceAB(bool annouceA, bool announceB) internal {
    // both accountA and accountB are announced
    bytes memory multiAddress = hex'1234';
    if (annouceA) {
      vm.prank(accountA.accountAddr);
      hoprChannels.announce(accountA.publicKey, multiAddress);
    }
    if (announceB) {
      vm.prank(accountB.accountAddr);
      hoprChannels.announce(accountB.publicKey, multiAddress);
    }
  }

  /**
   *@dev Helper function to fund channel A->B and B->A with `fundChannelMulti`
   */
  function _helperFundMultiAB(uint256 amount1, uint256 amount2) internal {
    // both accountA and accountB are announced
    _helperAnnounceAB(true, true);
    vm.prank(address(1));
    vm.mockCall(
      vm.addr(1),
      abi.encodeWithSignature(
        'transferFrom(address,address,uint256)',
        address(1),
        address(hoprChannels),
        amount1 + amount2
      ),
      abi.encode(true)
    );
    // fund channel A->B and B->A
    hoprChannels.fundChannelMulti(accountA.accountAddr, accountB.accountAddr, amount1, amount2);
  }

  /**
   * @dev Helper function to fund channel A->B (amount1) and B->A (amount2) to OPEN,
   * where both amount1 and amount2 are above the amount of possible winning ticket
   * (i.e. TICKET_AB_WIN and TICKET_BA_WIN)
   */
  function _helperOpenBidirectionalChannels(uint256 amount1, uint256 amount2) internal {
    // accountB bumps channel A->B with SECRET_2
    // accountA bumps channel B->A with SECRET_2
    vm.prank(accountB.accountAddr);
    hoprChannels.bumpChannel(accountA.accountAddr, SECRET_2);
    vm.prank(accountA.accountAddr);
    hoprChannels.bumpChannel(accountB.accountAddr, SECRET_2);
    // then fund channel
    _helperFundMultiAB(amount1, amount2);
  }
}
