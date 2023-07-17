// SPDX-License-Identifier: UNLICENSED
pragma solidity >=0.6.0 <0.9.0;

import 'forge-std/Test.sol';
import './utils/ERC1820Registry.sol';
// import './utils/Channels.sol';
// import './utils/Tickets.sol';
import '../src/Channels.sol';
import '../src/node-stake/NodeSafeRegistry.sol';

contract MyHoprChannels is HoprChannels {
  constructor(address _token, Timestamp _noticePeriodChannelClosure, HoprNodeSafeRegistry safeRegistry) HoprChannels(_token, _noticePeriodChannelClosure, safeRegistry) {}

  function myValidateBalance(HoprChannels.Balance balance) validateBalance(balance) public {}

  function myValidateChannelParties(address source, address destination) validateChannelParties(source, destination) public {}
}

contract HoprChannelsTest is Test, ERC1820RegistryFixtureTest {
  // Copied from HoprChannels contract
  event ChannelOpened(address indexed source, address indexed destination, HoprChannels.Balance amount);
  event ChannelBalanceIncreased(bytes32 indexed channelId, HoprChannels.Balance newBalance);
  event ChannelBalanceDecreased(bytes32 indexed channelId, HoprChannels.Balance newBalance);
  event CommitmentSet(bytes32 indexed channelId, HoprChannels.ChannelEpoch epoch);
  event OutgoingChannelClosureInitiated(bytes32 channelId, HoprChannels.Timestamp closureInitiationTime);
  event ChannelClosed(bytes32 channelId);
  event TicketRedeemed(bytes32 channelId, HoprChannels.TicketIndex newTicketIndex);

  bytes32 constant PROOF_OF_RELAY_SECRET_0 = keccak256(abi.encodePacked('PROOF_OF_RELAY_SECRET_0'));
  // HoprChannels.WinProb constant WIN_PROB_100 = HoprChannels.WinProb.wrap(type(uint56).max);
  // HoprChannels.WinProb constant WIN_PROB_0 = HoprChannels.WinProb.wrap(type(uint56).min);

  bytes32 constant SECRET_0 = keccak256(abi.encodePacked('secret'));
  bytes32 constant SECRET_1 = keccak256(abi.encodePacked(SECRET_0));
  bytes32 constant SECRET_2 = keccak256(abi.encodePacked(SECRET_1));

  function getChannelFromTuple(bytes32 channelId) public view returns (HoprChannels.Channel memory) {
    (
      HoprChannels.Balance balance,
      HoprChannels.TicketIndex ticketIndex,
      HoprChannels.Timestamp closureTime,
      HoprChannels.ChannelEpoch epoch,
      HoprChannels.ChannelStatus status
    ) = myHoprChannels.channels(channelId);
    return HoprChannels.Channel(balance, ticketIndex, closureTime, epoch,status );
  }

  address hoprToken;

  MyHoprChannels public myHoprChannels;
  HoprNodeSafeRegistry public safeRegistry;

  // HoprChannels.RedeemableTicket TICKET_AB_WIN =
  //   HoprChannels.RedeemableTicket({
  //     data: HoprChannels.TicketData({
  //       channelId: hoprChannels._getChannelId(accountA.accountAddr, accountB.accountAddr),
  //       amount: HoprChannels.Balance.wrap(10),
  //       index: HoprChannels.TicketIndex.wrap(1),
  //       epoch: HoprChannels.ChannelEpoch.wrap(0),
  //       winProb: WIN_PROB_100,
  //       reserved: HoprChannels.TicketReserved.wrap(0)
  //     }),
  //     opening: SECRET_0,
  //     porSecret: PROOF_OF_RELAY_SECRET_0,
  //     signature: HoprChannels.CompactSignature({
  //       r: 0xe28514db6bf62eab85e382e77a551639f616a51e527480dc922004a197b44600,
  //       vs: 0x6e396a671f35c69bbe966fbf26ccdd31da7722ea566fd34ba2724f6a10d7231b
  //     })
  //   });

  // HoprChannels.Ticket immutable TICKET_BA_WIN =
  //   HoprChannels.Ticket({
  //     channelId: HoprChannels._getChannelId(accountB.accountAddr, accountA.accountAddr),
  //     nextCommitment: SECRET_1,
  //     ticketEpoch: 0,
  //     ticketIndex: 1,
  //     proofOfRelaySecret: PROOF_OF_RELAY_SECRET_0,
  //     amount: 10,
  //     winProb: WIN_PROB_100,
  //     signature: hex'40e302cb0b8b18dbdd08ca1bfc93f1f2c40d5b93e8366dbd97f323aabb26e05f91b2e8828a3a15ffb39d10c55e3fbc2fca72c4d3a3083f09fe07797a6a9ecc54'
  //   });

  // HoprChannels.Ticket immutable TICKET_AB_LOSS =
  //   HoprChannels.Ticket({
  //     channelId: HoprChannels._getChannelId(accountA.accountAddr, accountB.accountAddr),
  //     nextCommitment: SECRET_1,
  //     ticketEpoch: 0,
  //     ticketIndex: 1,
  //     proofOfRelaySecret: PROOF_OF_RELAY_SECRET_0,
  //     amount: 10,
  //     winProb: WIN_PROB_0,
  //     signature: hex'c81d8a3fe9d2dfbbf916bad5c3ff2acfb557c4972eb172f6441b85058e8cbd26b67afed7d5e72eae4e5bbf0b4ed9d949c0b06b0755b81b80742e4898f36fcc33'
  //   });

  // HoprChannels.Ticket immutable TICKET_AB_WIN_RECYCLED =
  //   HoprChannels.Ticket({
  //     channelId: HoprChannels._getChannelId(accountA.accountAddr, accountB.accountAddr),
  //     nextCommitment: SECRET_1,
  //     ticketEpoch: 0,
  //     ticketIndex: 1,
  //     proofOfRelaySecret: PROOF_OF_RELAY_SECRET_0,
  //     amount: 10,
  //     winProb: WIN_PROB_100,
  //     signature: hex'43bbf7f4a28786e47be61b6e3c40f4ff95f214e0ac3b43b10d9d962a076e7e0f0a35e4a487ba460af46f9b061e3474c1af399a50033a3f6a48f84a279acdc981'
  //   });

  bytes32 channelIdAB;
  bytes32 channelIdBA;

  uint256 MIN_USED_BALANCE;
  uint256 MAX_USED_BALANCE;

  uint256 constant ENOUGH_TIME_FOR_CLOSURE = 100;

  // uint256 public globalSnapshot;

  function setUp() public virtual override {
    super.setUp();

    hoprToken = vm.addr(1);

    safeRegistry = new HoprNodeSafeRegistry();
    myHoprChannels = new MyHoprChannels(vm.addr(1), HoprChannels.Timestamp.wrap(15), safeRegistry);

    MIN_USED_BALANCE = HoprChannels.Balance.unwrap(myHoprChannels.MIN_USED_BALANCE()) + 1;
    MAX_USED_BALANCE = HoprChannels.Balance.unwrap(myHoprChannels.MAX_USED_BALANCE());   
  }

  function testValidateBalance(uint96 amount) public {
    amount = uint96(bound(amount, MIN_USED_BALANCE ,MAX_USED_BALANCE));

    myHoprChannels.myValidateBalance(HoprChannels.Balance.wrap(amount));
  }

  function testRevert_validateBalance() public {
    vm.expectRevert(abi.encodeWithSelector(InvalidBalance.selector));
    myHoprChannels.myValidateBalance(HoprChannels.Balance.wrap(0));

    vm.expectRevert(abi.encodeWithSelector(BalanceExceedsGlobalPerChannelAllowance.selector));
    myHoprChannels.myValidateBalance(HoprChannels.Balance.wrap(uint96(MAX_USED_BALANCE) + 1));
  }

  function testValidateChannelParties(address source, address destination) public {
    vm.assume(source != destination);

    myHoprChannels.myValidateChannelParties(source, destination);
  }

  function testRevert_validateChannelParties(address addr) public {
    vm.expectRevert(abi.encodeWithSelector(SourceEqualsDestination.selector));
    myHoprChannels.myValidateChannelParties(addr, addr);

    vm.expectRevert(abi.encodeWithSelector(ZeroAddress.selector, 'source must not be empty'));
    myHoprChannels.myValidateChannelParties(address(0), addr);

    vm.expectRevert(abi.encodeWithSelector(ZeroAddress.selector, 'destination must not be empty'));
    myHoprChannels.myValidateChannelParties(addr, address(0));
  }

  // function testFundChannelMulti(uint96 amount1, uint96 amount2, address account1, address account2) public {
  //   amount1 = uint96(bound(amount1, MIN_USED_BALANCE ,MAX_USED_BALANCE));
  //   amount2 = uint96(bound(amount2, MIN_USED_BALANCE, MAX_USED_BALANCE));
  //   vm.assume(account1 != account2);

  //   HoprChannels.Channel memory channelAB = HoprChannels.Channel(
  //     bytes32(0),
  //     HoprChannels.Balance.wrap(uint96(amount1)),
  //     HoprChannels.TicketIndex.wrap(0),
  //     HoprChannels.Timestamp.wrap(0),

  //     HoprChannels.ChannelEpoch.wrap(1),
  //     HoprChannels.ChannelStatus.OPEN
  //   );
  //   HoprChannels.Channel memory channelBA = HoprChannels.Channel(
  //     bytes32(0),
  //     HoprChannels.Balance.wrap(uint96(amount2)),
  //     HoprChannels.TicketIndex.wrap(0),
  //     HoprChannels.Timestamp.wrap(0),
  //     HoprChannels.ChannelEpoch.wrap(1),
  //     HoprChannels.ChannelStatus.OPEN
  //   );

  //   vm.prank(address(1));
  //   vm.mockCall(
  //     hoprToken,
  //     abi.encodeWithSignature(
  //       'transferFrom(address,address,uint256)',
  //       address(1),
  //       address(hoprChannels),
  //       amount1 + amount2
  //     ),
  //     abi.encode(true)
  //   );
  //   // fund channel for two parties triggers
  //   vm.expectEmit(true, true, false, true, address(hoprChannels));
  //   emit ChannelOpened(account1, account2, HoprChannels.Balance.wrap(uint96(amount1)));
  //   vm.expectEmit(true, true, false, true, address(hoprChannels));
  //   emit ChannelOpened(account2, account1, HoprChannels.Balance.wrap(uint96(amount2)));

  //   // fund channel A->B and B->A
  //   hoprChannels.fundChannelMulti(
  //     account1,
  //     HoprChannels.Balance.wrap(amount1),
  //     account2,
  //     HoprChannels.Balance.wrap(amount2)
  //   );
  //   vm.clearMockedCalls();

  //   assertEq(
  //     keccak256(
  //       abi.encode(getChannelFromTuple(channelIdAB))
  //     ),
  //     keccak256(abi.encode(channelAB))
  //   );
  //   assertEq(
  //     keccak256(
  //       abi.encode(getChannelFromTuple(channelIdBA))
  //     ),
  //     keccak256(abi.encode(channelBA))
  //   );
  // }

  // function testRevert_FundChannelMulti_SameSourceAndDestination(uint96 amount1, uint96 amount2, address account1) public {
  //   amount1 = uint96(bound(amount1, MIN_USED_BALANCE ,MAX_USED_BALANCE));
  //   amount2 = uint96(bound(amount2, MIN_USED_BALANCE, MAX_USED_BALANCE));

  //   vm.mockCall(
  //     hoprToken,
  //     abi.encodeWithSignature(
  //       'transferFrom(address,address,uint256)',
  //       address(1),
  //       address(hoprChannels),
  //       amount1 + amount2
  //     ),
  //     abi.encode(true)
  //   );

  //   vm.prank(address(1));
  //   vm.expectRevert(SourceEqualsDestination.selector);
  //   hoprChannels.fundChannelMulti(account1, HoprChannels.Balance.wrap(amount1), account1, HoprChannels.Balance.wrap(amount2));

  //   vm.clearMockedCalls();
  // }

  // function testRevert_FundChannelMulti_ZeroSource(uint96 amount1, uint96 amount2, address account1) public {
  //   amount1 = uint96(bound(amount1, MIN_USED_BALANCE, MAX_USED_BALANCE));
  //   amount2 = uint96(bound(amount2, MIN_USED_BALANCE, MAX_USED_BALANCE));
  //   vm.assume(account1 != address(0));

  //   vm.mockCall(
  //     hoprToken,
  //     abi.encodeWithSignature(
  //       'transferFrom(address,address,uint256)',
  //       address(1),
  //       address(hoprChannels),
  //       amount1 + amount2
  //     ),
  //     abi.encode(true)
  //   );

  //   vm.prank(address(1));
  //   vm.expectRevert(abi.encodeWithSelector(ZeroAddress.selector, "source must not be empty"));
  //   hoprChannels.fundChannelMulti(address(0), HoprChannels.Balance.wrap(amount1), account1, HoprChannels.Balance.wrap(amount2));

  //   vm.clearMockedCalls();
  // }

  // function testRevert_FundChannelMulti_ZeroDestination(uint96 amount1, uint96 amount2) public {
  //   amount1 = uint96(bound(amount1, MIN_USED_BALANCE, MAX_USED_BALANCE));
  //   amount2 = uint96(bound(amount2, MIN_USED_BALANCE, MAX_USED_BALANCE));

  //   vm.mockCall(
  //     hoprToken,
  //     abi.encodeWithSignature(
  //       'transferFrom(address,address,uint256)',
  //       address(1),
  //       address(hoprChannels),
  //       amount1 + amount2
  //     ),
  //     abi.encode(true)
  //   );

  //   vm.prank(address(1));
  //   vm.expectRevert(abi.encodeWithSelector(ZeroAddress.selector, "destination must not be empty"));
  //   hoprChannels.fundChannelMulti(accountA.accountAddr, HoprChannels.Balance.wrap(amount1), address(0), HoprChannels.Balance.wrap(amount2));

  //   vm.clearMockedCalls();
  // }

  // function testRevert_FundChannelMulti_AmountTooLow() public {

  //   vm.mockCall(
  //     hoprToken,
  //     abi.encodeWithSignature(
  //       'transferFrom(address,address,uint256)',
  //       address(1),
  //       address(hoprChannels),
  //       0
  //     ),
  //     abi.encode(true)
  //   );

  //   vm.prank(address(1));
  //   vm.expectRevert(InvalidBalance.selector);
  //   hoprChannels.fundChannelMulti(accountA.accountAddr, HoprChannels.Balance.wrap(0), accountB.accountAddr, HoprChannels.Balance.wrap(0));

  //   vm.clearMockedCalls();
  // }


  // function testRevert_FundChannelMulti_AmountATooHigh(uint96 amount2) public {
  //   amount2 = uint96(bound(amount2, MIN_USED_BALANCE, MAX_USED_BALANCE));

  //   uint256 highAmount = MAX_USED_BALANCE + 1;

  //   vm.mockCall(
  //     hoprToken,
  //     abi.encodeWithSignature(
  //       'transferFrom(address,address,uint256)',
  //       address(1),
  //       address(hoprChannels),
  //       amount2 + highAmount
  //     ),
  //     abi.encode(true)
  //   );

  //   vm.prank(address(1));
  //   vm.expectRevert(BalanceExceedsGlobalPerChannelAllowance.selector);
  //   hoprChannels.fundChannelMulti(accountA.accountAddr, HoprChannels.Balance.wrap(uint96(highAmount)), accountB.accountAddr, HoprChannels.Balance.wrap(amount2));

  //   vm.clearMockedCalls();
  // }

  // function testRevert_FundChannelMulti_AmountBTooHigh(uint96 amount1) public {
  //   amount1 = uint96(bound(amount1, MIN_USED_BALANCE, MAX_USED_BALANCE));

  //   uint256 highAmount = MAX_USED_BALANCE + 1;

  //   vm.mockCall(
  //     hoprToken,
  //     abi.encodeWithSignature(
  //       'transferFrom(address,address,uint256)',
  //       address(1),
  //       address(hoprChannels),
  //       amount1 + highAmount
  //     ),
  //     abi.encode(true)
  //   );

  //   vm.prank(address(1));
  //   vm.expectRevert(BalanceExceedsGlobalPerChannelAllowance.selector);
  //   hoprChannels.fundChannelMulti(accountA.accountAddr, HoprChannels.Balance.wrap(amount1), accountB.accountAddr, HoprChannels.Balance.wrap(uint96(highAmount)));

  //   vm.clearMockedCalls();
  // }

  // function testRevert_FundChannelMultiTokenTransfer(uint96 amount1, uint96 amount2) public {
  //   amount1 = uint96(bound(amount1, MIN_USED_BALANCE, MAX_USED_BALANCE));
  //   amount2 = uint96(bound(amount2, MIN_USED_BALANCE, MAX_USED_BALANCE));

  //   vm.prank(address(1));
  //   vm.mockCall(
  //     hoprToken,
  //     abi.encodeWithSignature(
  //       'transferFrom(address,address,uint256)',
  //       address(1),
  //       address(hoprChannels),
  //       amount1 + amount2
  //     ),
  //     abi.encode(false)
  //   );

  //   vm.expectRevert(abi.encodeWithSelector(TokenTransferFailed.selector));
  //   hoprChannels.fundChannelMulti(accountA.accountAddr, HoprChannels.Balance.wrap(amount1), accountB.accountAddr, HoprChannels.Balance.wrap(amount2));

  //   vm.clearMockedCalls();
  // }

  // function test_SetCommitment(uint96 amount1, uint96 amount2, uint256 commitmentA, uint256 commitmentB) public {
  //   amount1 = uint96(bound(amount1, MIN_USED_BALANCE, MAX_USED_BALANCE));
  //   amount2 = uint96(bound(amount2, MIN_USED_BALANCE, MAX_USED_BALANCE));

  //   commitmentA = bound(commitmentA, uint256(1), type(uint256).max);
  //   commitmentB = bound(commitmentB, uint256(1), type(uint256).max);

  //   HoprChannels.Channel memory channelAB = HoprChannels.Channel(
  //     bytes32(commitmentA),
  //     HoprChannels.Balance.wrap(uint96(amount1)),
  //     HoprChannels.TicketIndex.wrap(0),
  //     HoprChannels.Timestamp.wrap(0),

  //     HoprChannels.ChannelEpoch.wrap(1),
  //     HoprChannels.ChannelStatus.OPEN
  //   );
  //   HoprChannels.Channel memory channelBA = HoprChannels.Channel(
  //     bytes32(commitmentB),
  //     HoprChannels.Balance.wrap(uint96(amount2)),
  //     HoprChannels.TicketIndex.wrap(0),
  //     HoprChannels.Timestamp.wrap(0),
  //     HoprChannels.ChannelEpoch.wrap(1),
  //     HoprChannels.ChannelStatus.OPEN
  //   );

  //   vm.prank(address(1));
  //   vm.mockCall(
  //     hoprToken,
  //     abi.encodeWithSignature(
  //       'transferFrom(address,address,uint256)',
  //       address(1),
  //       address(hoprChannels),
  //       amount1 + amount2
  //     ),
  //     abi.encode(true)
  //   );

  //   vm.expectEmit(true, false, false, false, address(hoprChannels));
  //   emit ChannelOpened(accountA.accountAddr, accountB.accountAddr, HoprChannels.Balance.wrap(amount1));

  //   vm.expectEmit(true, false, false, false, address(hoprChannels));
  //   emit ChannelOpened(accountB.accountAddr, accountA.accountAddr, HoprChannels.Balance.wrap(amount2));

  //   hoprChannels.fundChannelMulti(accountA.accountAddr, HoprChannels.Balance.wrap(amount1), accountB.accountAddr, HoprChannels.Balance.wrap(amount2));
  //   vm.clearMockedCalls();
   
  //   vm.expectEmit(true, false, false, false, address(hoprChannels));
  //   emit CommitmentSet(channelIdAB, HoprChannels.ChannelEpoch.wrap(1));

  //   vm.prank(accountB.accountAddr);
  //   hoprChannels.setCommitment(bytes32(commitmentA), accountA.accountAddr);

  //   vm.expectEmit(true, false, false, true, address(hoprChannels));
  //   emit CommitmentSet(channelIdBA, HoprChannels.ChannelEpoch.wrap(1));

  //   vm.prank(accountA.accountAddr);
  //   hoprChannels.setCommitment(bytes32(commitmentB), accountB.accountAddr);

  //   assertEq(
  //     keccak256(
  //       abi.encode(getChannelFromTuple(channelIdAB))
  //     ),
  //     keccak256(abi.encode(channelAB))
  //   );
  //   assertEq(
  //     keccak256(
  //       abi.encode(getChannelFromTuple(channelIdBA))
  //     ),
  //     keccak256(abi.encode(channelBA))
  //   );
  // }

  // function test_SetCommitmentEpochBump(uint96 amount1, uint96 amount2) public {
  //   amount1 = uint96(bound(amount1, MIN_USED_BALANCE, MAX_USED_BALANCE));
  //   amount2 = uint96(bound(amount2, MIN_USED_BALANCE, MAX_USED_BALANCE));

  //   bytes32 commitment1 = 0xa74616c1018b7fccab7c6d508560a93768df100a80661bf2693d79d5b474d75e;
  //   bytes32 commitment2 = 0x677cbfec2c3998a7ed96e60b2f408abe400313dc5f365a465ab6d40888b18375;

  //   HoprChannels.Channel memory channelAB = HoprChannels.Channel(
  //     bytes32(commitment1),
  //     HoprChannels.Balance.wrap(uint96(amount1)),
  //     HoprChannels.TicketIndex.wrap(0),
  //     HoprChannels.Timestamp.wrap(0),

  //     HoprChannels.ChannelEpoch.wrap(1),
  //     HoprChannels.ChannelStatus.OPEN
  //   );

  //   vm.prank(address(1));
  //   vm.mockCall(
  //     hoprToken,
  //     abi.encodeWithSignature(
  //       'transferFrom(address,address,uint256)',
  //       address(1),
  //       address(hoprChannels),
  //       amount1 + amount2
  //     ),
  //     abi.encode(true)
  //   );

  //   vm.expectEmit(true, false, false, false, address(hoprChannels));
  //   emit ChannelOpened(accountA.accountAddr, accountB.accountAddr, HoprChannels.Balance.wrap(amount1));

  //   vm.expectEmit(true, false, false, false, address(hoprChannels));
  //   emit ChannelOpened(accountB.accountAddr, accountA.accountAddr, HoprChannels.Balance.wrap(amount2));
    
  //   hoprChannels.fundChannelMulti(accountA.accountAddr, HoprChannels.Balance.wrap(amount1), accountB.accountAddr, HoprChannels.Balance.wrap(amount2));
  //   vm.clearMockedCalls();

  //   vm.expectEmit(true, false, false, true, address(hoprChannels));
  //   emit CommitmentSet(channelIdAB, HoprChannels.ChannelEpoch.wrap(1));

  //   vm.prank(accountB.accountAddr);
  //   hoprChannels.setCommitment(bytes32(commitment1), accountA.accountAddr);

  //   assertEq(
  //     keccak256(
  //       abi.encode(getChannelFromTuple(channelIdAB))
  //     ),
  //     keccak256(abi.encode(channelAB))
  //   );

  //   vm.prank(accountB.accountAddr);
  //   vm.expectRevert(abi.encodeWithSelector(InvalidCommitment.selector));
  //   hoprChannels.setCommitment(bytes32(commitment1), accountA.accountAddr);

  //   HoprChannels.Channel memory channelABBumped = HoprChannels.Channel(
  //     bytes32(commitment2),
  //     HoprChannels.Balance.wrap(uint96(amount1)),
  //     HoprChannels.TicketIndex.wrap(0),
  //     HoprChannels.Timestamp.wrap(0),

  //     HoprChannels.ChannelEpoch.wrap(2),
  //     HoprChannels.ChannelStatus.OPEN
  //   );


  //   vm.expectEmit(true, false, false, true, address(hoprChannels));
  //   emit CommitmentSet(channelIdAB, HoprChannels.ChannelEpoch.wrap(2));

  //   vm.prank(accountB.accountAddr);
  //   hoprChannels.setCommitment(bytes32(commitment2), accountA.accountAddr);

  //   assertEq(
  //     keccak256(
  //       abi.encode(getChannelFromTuple(channelIdAB))
  //     ),
  //     keccak256(abi.encode(channelABBumped))
  //   );
  // }

  // function testRevert_SetCommitment() public {
  //   vm.prank(accountA.accountAddr);
  //   vm.expectRevert(InvalidCommitment.selector);
  //   hoprChannels.setCommitment(bytes32(0), accountB.accountAddr);
  // }

  // function testRevert_SetCommitmentChannelNotOpen(uint256 commitment) public {
  //   commitment = bound(commitment, uint256(1), type(uint256).max);

  //   vm.prank(accountA.accountAddr);
  //   vm.expectRevert(abi.encodeWithSelector(WrongChannelState.selector, "Cannot set commitments for channels that are not in state OPEN."));
  //   hoprChannels.setCommitment(bytes32(commitment), accountB.accountAddr);
  // }

  // function testOpenChannel

  // /**
  //  *@dev it should fund A->B using send
  //  */
  // function testFundChannelABWithSend(uint256 amount) public {
  //   amount = bound(amount, 1, 1e36);
  //   // both accountA and accountB are announced
  //   _helperAnnounceAB(true, true);
  //   // expect to emit a channel
  //   HoprChannels.Channel memory channelBA = HoprChannels.Channel(
  //     amount,
  //     bytes32(0),
  //     0,
  //     0,
  //     HoprChannels.ChannelStatus.WAITING_FOR_COMMITMENT,
  //     1,
  //     0
  //   );
  //   vm.expectEmit(true, true, false, true, address(hoprChannels));
  //   emit ChannelUpdated(accountB.accountAddr, accountA.accountAddr, channelBA);
  //   vm.expectEmit(true, true, true, true, address(hoprChannels));
  //   emit ChannelFunded(accountB.accountAddr, accountB.accountAddr, accountA.accountAddr, amount);

  //   // mock token contract to call `tokensReceived()` on hoprChannels contract by account B
  //   vm.prank(vm.addr(1));
  //   hoprChannels.tokensReceived(
  //     address(0),
  //     accountB.accountAddr,
  //     address(hoprChannels),
  //     amount,
  //     abi.encode(accountB.accountAddr, accountA.accountAddr, amount, HoprChannels.ChannelStatus.CLOSED),
  //     hex'00'
  //   );
  // }

  // /**
  //  * @dev With single funded HoprChannels: AB: amount1, where amount1 is above 10,
  //    it should reedem ticket for account B -> directly to wallet
  //  */
  // function testRedeemTicketWithSingleFundedHoprChannel(uint256 amount1) public {
  //   // channel is funded for at least 10 HoprTokens (Ticket's amount)
  //   amount1 = bound(amount1, TICKET_AB_WIN.amount, 1e36);
  //   // Firstly, accountB bumps channel A->B with SECRET_2
  //   vm.prank(accountB.accountAddr);
  //   hoprChannels.bumpChannel(accountA.accountAddr, SECRET_2);
  //   // then fund channel
  //   _helperFundMultiAB(amount1, 0);
  //   // mock token transfer from hoprChannels
  //   vm.prank(address(hoprChannels));
  //   vm.mockCall(
  //     vm.addr(1),
  //     abi.encodeWithSignature('transfer(address,uint256)', accountB.accountAddr, TICKET_AB_WIN.amount), // provide specific
  //     abi.encode(true)
  //   );

  //   // channels
  //   HoprChannels.Channel memory channelAB = HoprChannels.Channel(
  //     amount1,
  //     SECRET_2,
  //     0,
  //     0,
  //     HoprChannels.ChannelStatus.OPEN,
  //     1,
  //     0
  //   );
  //   // ticket epoch is 1
  //   HoprChannels.Channel memory channelBA = HoprChannels.Channel(
  //     0,
  //     bytes32(0),
  //     0,
  //     0,
  //     HoprChannels.ChannelStatus.CLOSED,
  //     0,
  //     0
  //   );
  //   // check vallidate from channels()
  //   assertEqChannels(getChannelFromTuple(hoprChannels, channelIdAB), channelAB);
  //   assertEqChannels(getChannelFromTuple(hoprChannels, channelIdBA), channelBA);

  //   vm.expectEmit(true, true, false, true, address(hoprChannels));
  //   emit TicketRedeemed(
  //     TICKET_AB_WIN.source,
  //     accountB.accountAddr,
  //     TICKET_AB_WIN.nextCommitment,
  //     TICKET_AB_WIN.ticketEpoch,
  //     TICKET_AB_WIN.ticketIndex,
  //     TICKET_AB_WIN.proofOfRelaySecret,
  //     TICKET_AB_WIN.amount,
  //     TICKET_AB_WIN.winProb,
  //     TICKET_AB_WIN.signature
  //   );

  //   // accountB redeem ticket
  //   vm.prank(accountB.accountAddr);
  //   hoprChannels.redeemTicket(
  //     TICKET_AB_WIN.source,
  //     TICKET_AB_WIN.nextCommitment,
  //     TICKET_AB_WIN.ticketEpoch,
  //     TICKET_AB_WIN.ticketIndex,
  //     TICKET_AB_WIN.proofOfRelaySecret,
  //     TICKET_AB_WIN.amount,
  //     TICKET_AB_WIN.winProb,
  //     TICKET_AB_WIN.signature
  //   );
  //   vm.clearMockedCalls();
  // }

  // /**
  //  * @dev With funded HoprChannels: AB: amount1, BA: amount 2,
  //  * where both amount1 and amount2 are above 10,
  //  * it should redeem ticket for account A
  //  * it should redeem ticket for account B
  //  */
  // function testRedeemTicketWithDoubleFundedHoprChannel(uint256 amount1, uint256 amount2) public {
  //   // channel is funded for at least 10 HoprTokens (Ticket's amount)
  //   amount1 = bound(amount1, TICKET_AB_WIN.amount, 1e36);
  //   amount2 = bound(amount2, TICKET_BA_WIN.amount, 1e36);
  //   // Open channels A<->B with some tokens that are above possible winning tickets
  //   _helperOpenBidirectionalChannels(amount1, amount2);

  //   // channels before ticket redemption
  //   HoprChannels.Channel memory channelAB = HoprChannels.Channel(
  //     amount1,
  //     SECRET_2,
  //     0,
  //     0,
  //     HoprChannels.ChannelStatus.OPEN,
  //     1,
  //     0
  //   );
  //   // ticket epoch is 1
  //   HoprChannels.Channel memory channelBA = HoprChannels.Channel(
  //     amount2,
  //     SECRET_2,
  //     0,
  //     0,
  //     HoprChannels.ChannelStatus.OPEN,
  //     1,
  //     0
  //   );
  //   // check vallidate from channels()
  //   assertEqChannels(getChannelFromTuple(hoprChannels, channelIdAB), channelAB);
  //   assertEqChannels(getChannelFromTuple(hoprChannels, channelIdBA), channelBA);

  //   vm.expectEmit(true, true, false, true, address(hoprChannels));
  //   emit TicketRedeemed(
  //     TICKET_AB_WIN.source,
  //     accountB.accountAddr,
  //     TICKET_AB_WIN.nextCommitment,
  //     TICKET_AB_WIN.ticketEpoch,
  //     TICKET_AB_WIN.ticketIndex,
  //     TICKET_AB_WIN.proofOfRelaySecret,
  //     TICKET_AB_WIN.amount,
  //     TICKET_AB_WIN.winProb,
  //     TICKET_AB_WIN.signature
  //   );

  //   // create a snapshot
  //   uint256 snapshotBeforeAccountBRedeemTicket = vm.snapshot();

  //   // accountB redeem ticket
  //   vm.prank(accountB.accountAddr);
  //   hoprChannels.redeemTicket(
  //     TICKET_AB_WIN.source,
  //     TICKET_AB_WIN.nextCommitment,
  //     TICKET_AB_WIN.ticketEpoch,
  //     TICKET_AB_WIN.ticketIndex,
  //     TICKET_AB_WIN.proofOfRelaySecret,
  //     TICKET_AB_WIN.amount,
  //     TICKET_AB_WIN.winProb,
  //     TICKET_AB_WIN.signature
  //   );

  //   // channels after ticket redemption
  //   HoprChannels.Channel memory channelABAfterBRedeems = HoprChannels.Channel(
  //     amount1 - TICKET_AB_WIN.amount,
  //     SECRET_1,
  //     0,
  //     1,
  //     HoprChannels.ChannelStatus.OPEN,
  //     1,
  //     0
  //   );
  //   // ticket epoch is 1
  //   HoprChannels.Channel memory channelBAAfterRedeems = HoprChannels.Channel(
  //     amount2 + TICKET_AB_WIN.amount,
  //     SECRET_2,
  //     0,
  //     0,
  //     HoprChannels.ChannelStatus.OPEN,
  //     1,
  //     0
  //   );
  //   // check vallidate from channels()
  //   assertEqChannels(getChannelFromTuple(hoprChannels, channelIdAB), channelABAfterBRedeems);
  //   assertEqChannels(getChannelFromTuple(hoprChannels, channelIdBA), channelBAAfterRedeems);

  //   // reset to snapshot
  //   vm.revertTo(snapshotBeforeAccountBRedeemTicket);
  //   // accountA redeem ticket
  //   vm.prank(accountA.accountAddr);
  //   hoprChannels.redeemTicket(
  //     TICKET_BA_WIN.source,
  //     TICKET_BA_WIN.nextCommitment,
  //     TICKET_BA_WIN.ticketEpoch,
  //     TICKET_BA_WIN.ticketIndex,
  //     TICKET_BA_WIN.proofOfRelaySecret,
  //     TICKET_BA_WIN.amount,
  //     TICKET_BA_WIN.winProb,
  //     TICKET_BA_WIN.signature
  //   );

  //   // channels after ticket redemption
  //   HoprChannels.Channel memory channelBAAfterARedeems = HoprChannels.Channel(
  //     amount2 - TICKET_BA_WIN.amount,
  //     SECRET_1,
  //     0,
  //     1,
  //     HoprChannels.ChannelStatus.OPEN,
  //     1,
  //     0
  //   );
  //   // ticket epoch is 1
  //   HoprChannels.Channel memory channelABAfterARedeems = HoprChannels.Channel(
  //     amount1 + TICKET_BA_WIN.amount,
  //     SECRET_2,
  //     0,
  //     0,
  //     HoprChannels.ChannelStatus.OPEN,
  //     1,
  //     0
  //   );
  //   // check vallidate from channels()
  //   assertEqChannels(getChannelFromTuple(hoprChannels, channelIdAB), channelABAfterARedeems);
  //   assertEqChannels(getChannelFromTuple(hoprChannels, channelIdBA), channelBAAfterARedeems);
  // }

  // /**
  //  * @dev With funded HoprChannels: AB: amount1, BA: amount 2, where both amount1 and amount2 are above 10 (amount of TICKET_AB_WIN and TICKET_BA_WIN),
  //  * it should fail to redeem ticket when ticket has been already redeemed
  //  */
  // function testFail_RedeemARedeemedTicket(uint256 amount1, uint256 amount2) public {
  //   // channel is funded for at least 10 HoprTokens (Ticket's amount)
  //   amount1 = bound(amount1, TICKET_AB_WIN.amount, 1e36);
  //   amount2 = bound(amount2, TICKET_BA_WIN.amount, 1e36);
  //   // Open channels A<->B with some tokens that are above possible winning tickets
  //   _helperOpenBidirectionalChannels(amount1, amount2);

  //   vm.expectEmit(true, true, false, true, address(hoprChannels));
  //   emit TicketRedeemed(
  //     TICKET_AB_WIN.source,
  //     accountB.accountAddr,
  //     TICKET_AB_WIN.nextCommitment,
  //     TICKET_AB_WIN.ticketEpoch,
  //     TICKET_AB_WIN.ticketIndex,
  //     TICKET_AB_WIN.proofOfRelaySecret,
  //     TICKET_AB_WIN.amount,
  //     TICKET_AB_WIN.winProb,
  //     TICKET_AB_WIN.signature
  //   );

  //   // accountB redeem ticket
  //   vm.prank(accountB.accountAddr);
  //   hoprChannels.redeemTicket(
  //     TICKET_AB_WIN.source,
  //     TICKET_AB_WIN.nextCommitment,
  //     TICKET_AB_WIN.ticketEpoch,
  //     TICKET_AB_WIN.ticketIndex,
  //     TICKET_AB_WIN.proofOfRelaySecret,
  //     TICKET_AB_WIN.amount,
  //     TICKET_AB_WIN.winProb,
  //     TICKET_AB_WIN.signature
  //   );

  //   // fail to redeem the redeemed ticket
  //   vm.expectRevert(bytes('redemptions must be in order'));
  //   hoprChannels.redeemTicket(
  //     TICKET_AB_WIN.source,
  //     SECRET_0,
  //     TICKET_AB_WIN.ticketEpoch,
  //     TICKET_AB_WIN.ticketIndex,
  //     TICKET_AB_WIN.proofOfRelaySecret,
  //     TICKET_AB_WIN.amount,
  //     TICKET_AB_WIN.winProb,
  //     TICKET_AB_WIN.signature
  //   );

  //   // fail to redeem the redeemed ticket
  //   vm.expectRevert(bytes('ticket epoch must match'));
  //   hoprChannels.redeemTicket(
  //     TICKET_AB_WIN.source,
  //     SECRET_0,
  //     TICKET_AB_WIN.ticketEpoch + 1,
  //     TICKET_AB_WIN.ticketIndex,
  //     TICKET_AB_WIN.proofOfRelaySecret,
  //     TICKET_AB_WIN.amount,
  //     TICKET_AB_WIN.winProb,
  //     TICKET_AB_WIN.signature
  //   );
  // }

  // /**
  //  * @dev With funded open channels:
  //  * it should fail to redeem ticket when signer is not the issuer
  //  */
  // function testFail_RedeemATicketFromAnotherSigner(uint256 amount1, uint256 amount2) public {
  //   // channel is funded for at least 10 HoprTokens (Ticket's amount)
  //   amount1 = bound(amount1, TICKET_AB_WIN.amount, 1e36);
  //   amount2 = bound(amount2, TICKET_BA_WIN.amount, 1e36);
  //   // Open channels A<->B with some tokens that are above possible winning tickets
  //   _helperOpenBidirectionalChannels(amount1, amount2);

  //   // accountB redeem ticket
  //   vm.prank(accountB.accountAddr);
  //   // fail to redeem the redeemed ticket due to wrong signature
  //   vm.expectRevert(bytes('signer must match the counterparty'));
  //   hoprChannels.redeemTicket(
  //     TICKET_AB_LOSS.source,
  //     TICKET_AB_LOSS.nextCommitment,
  //     TICKET_AB_LOSS.ticketEpoch,
  //     TICKET_AB_LOSS.ticketIndex,
  //     TICKET_AB_LOSS.proofOfRelaySecret,
  //     TICKET_AB_LOSS.amount,
  //     TICKET_AB_LOSS.winProb,
  //     TICKET_AB_LOSS.signature
  //   );
  // }

  // /**
  //  * @dev With funded open channels, A can initialize channel closure
  //  */
  // function test_AInitializeChannelClosure(uint256 amount1, uint256 amount2) public {
  //   // channel is funded for at least 10 HoprTokens (Ticket's amount)
  //   amount1 = bound(amount1, TICKET_AB_WIN.amount, 1e36);
  //   amount2 = bound(amount2, TICKET_BA_WIN.amount, 1e36);
  //   // Open channels A<->B with some tokens that are above possible winning tickets
  //   _helperOpenBidirectionalChannels(amount1, amount2);

  //   // Update channel AB state
  //   HoprChannels.Channel memory channelAB = getChannelFromTuple(hoprChannels, channelIdAB);
  //   channelAB.status = HoprChannels.ChannelStatus.PENDING_TO_CLOSE;
  //   channelAB.closureTime = uint32(block.timestamp) + hoprChannels.secsClosure();

  //   vm.expectEmit(true, true, false, true, address(hoprChannels));
  //   emit ChannelUpdated(accountA.accountAddr, accountB.accountAddr, channelAB);
  //   vm.expectEmit(true, true, false, true, address(hoprChannels));
  //   emit ChannelClosureInitiated(accountA.accountAddr, accountB.accountAddr, uint32(block.timestamp));
  //   // account A initiate channel closure
  //   vm.prank(accountA.accountAddr);
  //   hoprChannels.initiateChannelClosure(accountB.accountAddr);

  //   // channels after ticket redemption
  //   HoprChannels.Channel memory channelBA = HoprChannels.Channel(
  //     amount2,
  //     SECRET_2,
  //     0,
  //     0,
  //     HoprChannels.ChannelStatus.OPEN,
  //     1,
  //     0
  //   );
  //   // check vallidate from channels()
  //   assertEqChannels(getChannelFromTuple(hoprChannels, channelIdAB), channelAB);
  //   assertEqChannels(getChannelFromTuple(hoprChannels, channelIdBA), channelBA);
  // }

  // /**
  //  * @dev With funded open channels, B can initialize channel closure
  //  */
  // function test_BInitializeChannelClosure(uint256 amount1, uint256 amount2) public {
  //   // channel is funded for at least 10 HoprTokens (Ticket's amount)
  //   amount1 = bound(amount1, TICKET_AB_WIN.amount, 1e36);
  //   amount2 = bound(amount2, TICKET_BA_WIN.amount, 1e36);
  //   // Open channels A<->B with some tokens that are above possible winning tickets
  //   _helperOpenBidirectionalChannels(amount1, amount2);

  //   // Update channel BA state
  //   HoprChannels.Channel memory channelBA = getChannelFromTuple(hoprChannels, channelIdBA);
  //   channelBA.status = HoprChannels.ChannelStatus.PENDING_TO_CLOSE;
  //   channelBA.closureTime = uint32(block.timestamp) + hoprChannels.secsClosure();

  //   vm.expectEmit(true, true, false, true, address(hoprChannels));
  //   emit ChannelUpdated(accountB.accountAddr, accountA.accountAddr, channelBA);
  //   // account B initiate channel closure
  //   vm.prank(accountB.accountAddr);
  //   hoprChannels.initiateChannelClosure(accountA.accountAddr);

  //   // channels after ticket redemption
  //   HoprChannels.Channel memory channelAB = HoprChannels.Channel(
  //     amount1,
  //     SECRET_2,
  //     0,
  //     0,
  //     HoprChannels.ChannelStatus.OPEN,
  //     1,
  //     0
  //   );
  //   // check vallidate from channels()
  //   assertEqChannels(getChannelFromTuple(hoprChannels, channelIdAB), channelAB);
  //   assertEqChannels(getChannelFromTuple(hoprChannels, channelIdBA), channelBA);
  // }

  // /**
  //  * @dev With funded open channels:
  //  * it should fail to redeem ticket if it's a loss
  //  */
  // function testFail_RedeemLossTicket(uint256 amount1, uint256 amount2) public {
  //   // channel is funded for at least 10 HoprTokens (Ticket's amount)
  //   amount1 = bound(amount1, TICKET_AB_WIN.amount, 1e36);
  //   amount2 = bound(amount2, TICKET_BA_WIN.amount, 1e36);
  //   // Open channels A<->B with some tokens that are above possible winning tickets
  //   _helperOpenBidirectionalChannels(amount1, amount2);

  //   // accountB redeem ticket
  //   vm.prank(accountB.accountAddr);
  //   // fail to redeem the redeemed ticket due to wrong signature
  //   vm.expectRevert(bytes('ticket must be a win'));
  //   hoprChannels.redeemTicket(
  //     TICKET_AB_WIN.source,
  //     TICKET_AB_WIN.nextCommitment,
  //     TICKET_AB_WIN.ticketEpoch,
  //     TICKET_AB_WIN.ticketIndex,
  //     TICKET_AB_WIN.proofOfRelaySecret,
  //     TICKET_AB_WIN.amount,
  //     TICKET_AB_WIN.winProb,
  //     hex'2bea87a4a771731c3dcf8c17443765344a0bfee9354f87636d068055cde15f6e2ddb4ce35e11b57dcc44f28206af89894feea677834c6e5be17d180cbeb514ba' // use AccountB to sign `TICKET_AB_WIN`
  //   );
  // }

  // /**
  //  * @dev With funded open channels:
  //  * it should fail to initialize channel closure A->A
  //  */
  // function testRevert_InitiateChannelClosureCircularChannel(uint256 amount1, uint256 amount2) public {
  //   // channel is funded for at least 10 HoprTokens (Ticket's amount)
  //   amount1 = bound(amount1, TICKET_AB_WIN.amount, 1e36);
  //   amount2 = bound(amount2, TICKET_BA_WIN.amount, 1e36);
  //   // Open channels A<->B with some tokens that are above possible winning tickets
  //   _helperOpenBidirectionalChannels(amount1, amount2);

  //   // accountA redeem ticket
  //   vm.prank(accountA.accountAddr);
  //   // fail to initiate channel closure for channel pointing to the source
  //   vm.expectRevert(bytes('source and destination must not be the same'));
  //   hoprChannels.initiateChannelClosure(accountA.accountAddr);
  // }

  // /**
  //  * @dev With funded open channels:
  //  * it should fail to initialize channel closure A->0
  //  */
  // function testRevert_InitiateChannelClosureFromAddressZero(uint256 amount1, uint256 amount2) public {
  //   // channel is funded for at least 10 HoprTokens (Ticket's amount)
  //   amount1 = bound(amount1, TICKET_AB_WIN.amount, 1e36);
  //   amount2 = bound(amount2, TICKET_BA_WIN.amount, 1e36);
  //   // Open channels A<->B with some tokens that are above possible winning tickets
  //   _helperOpenBidirectionalChannels(amount1, amount2);

  //   // accountA redeem ticket
  //   vm.prank(accountA.accountAddr);
  //   // fail to initiate channel closure pointing to address zero
  //   vm.expectRevert(bytes('destination must not be empty'));
  //   hoprChannels.initiateChannelClosure(address(0));
  // }

  // /**
  //  * @dev With funded open channels:
  //  * it should fail to finalize channel closure when is not pending
  //  */
  // function testRevert_FinalizedNotInitiatedChannelClosure(uint256 amount1, uint256 amount2) public {
  //   // channel is funded for at least 10 HoprTokens (Ticket's amount)
  //   amount1 = bound(amount1, TICKET_AB_WIN.amount, 1e36);
  //   amount2 = bound(amount2, TICKET_BA_WIN.amount, 1e36);
  //   // Open channels A<->B with some tokens that are above possible winning tickets
  //   _helperOpenBidirectionalChannels(amount1, amount2);

  //   // accountA redeem ticket
  //   vm.prank(accountA.accountAddr);
  //   // fail to force finalize channel closure
  //   vm.expectRevert(bytes('channel must be pending to close'));
  //   hoprChannels.finalizeChannelClosure(accountB.accountAddr);
  // }

  // /**
  //  * @dev With funded non-open channels:
  //  * it should fail to initialize channel closure when channel is not open
  //  */
  // function testRevert_InitializedClosureForNonOpenChannel(uint256 amount1, uint256 amount2) public {
  //   // channel is funded for at least 10 HoprTokens (Ticket's amount)
  //   amount1 = bound(amount1, TICKET_AB_WIN.amount, 1e36);
  //   amount2 = bound(amount2, TICKET_BA_WIN.amount, 1e36);
  //   // fund channels
  //   _helperFundMultiAB(amount1, amount2);
  //   // accountA redeem ticket
  //   vm.prank(accountA.accountAddr);
  //   // initiate channel closure first
  //   hoprChannels.initiateChannelClosure(accountB.accountAddr);

  //   // fail to force initiate again channel closure
  //   vm.expectRevert(bytes('channel must be open or waiting for commitment'));
  //   hoprChannels.initiateChannelClosure(accountB.accountAddr);
  // }

  // /**
  //  * @dev With funded non-open channels:
  //  * it should finalize channel closure
  //  */
  // function test_FinalizeInitializedClosure(uint256 amount1, uint256 amount2) public {
  //   // channel is funded for at least 10 HoprTokens (Ticket's amount)
  //   amount1 = bound(amount1, TICKET_AB_WIN.amount, 1e36);
  //   amount2 = bound(amount2, TICKET_BA_WIN.amount, 1e36);
  //   // fund channels
  //   _helperFundMultiAB(amount1, amount2);
  //   // accountA redeem ticket
  //   vm.startPrank(accountA.accountAddr);
  //   // initiate channel closure first
  //   hoprChannels.initiateChannelClosure(accountB.accountAddr);

  //   // increase enough time for channel closure;
  //   vm.warp(block.timestamp + ENOUGH_TIME_FOR_CLOSURE);

  //   // Update channel AB state
  //   HoprChannels.Channel memory channelAB = getChannelFromTuple(hoprChannels, channelIdAB);
  //   // succeed in finalizing channel closure
  //   vm.expectEmit(true, true, false, true, address(hoprChannels));
  //   emit ChannelClosureFinalized(accountA.accountAddr, accountB.accountAddr, channelAB.closureTime, amount1);
  //   vm.expectEmit(true, true, false, true, address(hoprChannels));
  //   emit ChannelUpdated(
  //     accountA.accountAddr,
  //     accountB.accountAddr,
  //     HoprChannels.Channel(0, bytes32(0), 0, 0, HoprChannels.ChannelStatus.CLOSED, 1, 0)
  //   );
  //   vm.mockCall(
  //     vm.addr(1),
  //     abi.encodeWithSignature('transfer(address,uint256)', accountA.accountAddr, amount1), // provide specific
  //     abi.encode(true)
  //   );
  //   hoprChannels.finalizeChannelClosure(accountB.accountAddr);
  //   vm.stopPrank();
  // }

  // /**
  //  * @dev With funded non-open channels:
  //  * it should fail to initialize channel closure, in various situtations
  //  */
  // function testRevert_FinalizeChannelClosure(uint256 amount1, uint256 amount2) public {
  //   // channel is funded for at least 10 HoprTokens (Ticket's amount)
  //   amount1 = bound(amount1, TICKET_AB_WIN.amount, 1e36);
  //   amount2 = bound(amount2, TICKET_BA_WIN.amount, 1e36);
  //   // fund channels
  //   _helperFundMultiAB(amount1, amount2);
  //   // accountA redeem ticket
  //   vm.startPrank(accountA.accountAddr);
  //   // initiate channel closure first
  //   hoprChannels.initiateChannelClosure(accountB.accountAddr);

  //   // fail when source and destination are the same
  //   vm.expectRevert(bytes('source and destination must not be the same'));
  //   hoprChannels.finalizeChannelClosure(accountA.accountAddr);

  //   // fail when the destination is empty
  //   vm.expectRevert(bytes('destination must not be empty'));
  //   hoprChannels.finalizeChannelClosure(address(0));

  //   // fail when finallization hasn't reached yet
  //   vm.expectRevert(bytes('closureTime must be before now'));
  //   hoprChannels.finalizeChannelClosure(accountB.accountAddr);
  //   vm.stopPrank();
  // }

  // /**
  //  * @dev With a closed channel:
  //  * it should fail to redeem ticket when channel in closed
  //  */
  // function testRevert_WhenRedeemTicketsInAClosedChannel(uint256 amount1, uint256 amount2) public {
  //   // channel is funded for at least 10 HoprTokens (Ticket's amount)
  //   amount1 = bound(amount1, TICKET_AB_WIN.amount, 1e36);
  //   amount2 = bound(amount2, TICKET_BA_WIN.amount, 1e36);
  //   // with a closed channel A->B
  //   _helperWithAClosedChannel(amount1, amount2);

  //   vm.prank(accountB.accountAddr);
  //   // fail when source and destination are the same
  //   vm.expectRevert(bytes('spending channel must be open or pending to close'));
  //   hoprChannels.redeemTicket(
  //     TICKET_AB_WIN.source,
  //     TICKET_AB_WIN.nextCommitment,
  //     TICKET_AB_WIN.ticketEpoch,
  //     TICKET_AB_WIN.ticketIndex,
  //     TICKET_AB_WIN.proofOfRelaySecret,
  //     TICKET_AB_WIN.amount,
  //     TICKET_AB_WIN.winProb,
  //     TICKET_AB_WIN.signature
  //   );
  // }

  // /**
  //  * @dev With a closed channel:
  //  * it should allow a fund to reopen channel
  //  */
  // function test_AllowFundToReopenAClosedChannel(
  //   uint256 amount1,
  //   uint256 amount2,
  //   uint256 reopenAmount1,
  //   uint256 reopenAmount2
  // ) public {
  //   // channel is funded for at least 10 HoprTokens (Ticket's amount)
  //   amount1 = bound(amount1, TICKET_AB_WIN.amount, 1e36);
  //   amount2 = bound(amount2, TICKET_BA_WIN.amount, 1e36);
  //   // reopen amount should be larger than 0
  //   reopenAmount1 = bound(reopenAmount1, 1, 1e36);
  //   reopenAmount2 = bound(reopenAmount2, 1, 1e36);

  //   // with a closed channel A->B
  //   _helperWithAClosedChannel(amount1, amount2);

  //   // fund channel again
  //   vm.prank(address(1));
  //   vm.mockCall(
  //     vm.addr(1),
  //     abi.encodeWithSignature(
  //       'transferFrom(address,address,uint256)',
  //       address(1),
  //       address(hoprChannels),
  //       reopenAmount1 + reopenAmount2
  //     ),
  //     abi.encode(true)
  //   );
  //   // succeed in reopening a channel
  //   hoprChannels.fundChannelMulti(accountA.accountAddr, accountB.accountAddr, reopenAmount1, reopenAmount2);

  //   // succeed in reopening a channel
  //   HoprChannels.Channel memory channelAB = HoprChannels.Channel(
  //     reopenAmount1,
  //     SECRET_2,
  //     0,
  //     0,
  //     HoprChannels.ChannelStatus.OPEN,
  //     2,
  //     0
  //   );

  //   HoprChannels.Channel memory channelBA = HoprChannels.Channel(
  //     amount2 + reopenAmount2,
  //     hex'00', // never bumped
  //     0,
  //     0,
  //     HoprChannels.ChannelStatus.WAITING_FOR_COMMITMENT,
  //     1,
  //     0
  //   );
  //   // check vallidate from channels()
  //   assertEqChannels(getChannelFromTuple(hoprChannels, channelIdAB), channelAB);
  //   assertEqChannels(getChannelFromTuple(hoprChannels, channelIdBA), channelBA);

  //   vm.clearMockedCalls();
  // }

  // /**
  //  * @dev With a reopened channel:
  //  * it should pass the sanity check
  //  */
  // function test_SanityCheck(uint256 amount1, uint256 amount2, uint256 reopenAmount1, uint256 reopenAmount2) public {
  //   // channel is funded for at least 10 HoprTokens (Ticket's amount)
  //   amount1 = bound(amount1, TICKET_AB_WIN.amount, 1e36);
  //   amount2 = bound(amount2, TICKET_BA_WIN.amount, 1e36);
  //   // reopen amount should be larger than 0
  //   reopenAmount1 = bound(reopenAmount1, 1, 1e36);
  //   reopenAmount2 = bound(reopenAmount2, 1, 1e36);
  //   _helperWithAReopenedChannel(amount1, amount2, reopenAmount1, reopenAmount2);

  //   // succeed in reopening a channel
  //   HoprChannels.Channel memory channelAB = HoprChannels.Channel(
  //     reopenAmount1,
  //     SECRET_2,
  //     0,
  //     0,
  //     HoprChannels.ChannelStatus.OPEN,
  //     2,
  //     0
  //   );

  //   HoprChannels.Channel memory channelBA = HoprChannels.Channel(
  //     amount2 + reopenAmount2,
  //     SECRET_2,
  //     0,
  //     0,
  //     HoprChannels.ChannelStatus.OPEN,
  //     1,
  //     0
  //   );
  //   // check vallidate from channels()
  //   assertEqChannels(getChannelFromTuple(hoprChannels, channelIdAB), channelAB);
  //   assertEqChannels(getChannelFromTuple(hoprChannels, channelIdBA), channelBA);
  // }

  // /**
  //  * @dev With a reopened channel:
  //  * it should fail to redeem ticket when channel in in different channelEpoch
  //  */
  // function testRevert_WhenDifferentChannelEpochFailToRedeemTicket(
  //   uint256 amount1,
  //   uint256 amount2,
  //   uint256 reopenAmount1,
  //   uint256 reopenAmount2
  // ) public {
  //   // channel is funded for at least 10 HoprTokens (Ticket's amount)
  //   amount1 = bound(amount1, TICKET_AB_WIN.amount, 1e36);
  //   amount2 = bound(amount2, TICKET_BA_WIN.amount, 1e36);
  //   // reopen amount should be larger than 0
  //   reopenAmount1 = bound(reopenAmount1, 1, 1e36);
  //   reopenAmount2 = bound(reopenAmount2, 1, 1e36);
  //   _helperWithAReopenedChannel(amount1, amount2, reopenAmount1, reopenAmount2);

  //   vm.prank(accountB.accountAddr);
  //   // fail when source and destination are the same
  //   vm.expectRevert(bytes('signer must match the counterparty'));
  //   hoprChannels.redeemTicket(
  //     TICKET_AB_WIN.source,
  //     TICKET_AB_WIN.nextCommitment,
  //     TICKET_AB_WIN.ticketEpoch,
  //     TICKET_AB_WIN.ticketIndex,
  //     TICKET_AB_WIN.proofOfRelaySecret,
  //     TICKET_AB_WIN.amount,
  //     TICKET_AB_WIN.winProb,
  //     TICKET_AB_WIN.signature
  //   );
  // }

  // /**
  //  * @dev With a reopened channel:
  //  * it should should reedem ticket for account B
  //  */
  // function test_RedeemTicketForAccountB(
  //   uint256 amount1,
  //   uint256 amount2,
  //   uint256 reopenAmount1,
  //   uint256 reopenAmount2
  // ) public {
  //   // channel is funded for at least 10 HoprTokens (Ticket's amount)
  //   amount1 = bound(amount1, TICKET_AB_WIN.amount, 1e36);
  //   amount2 = bound(amount2, TICKET_BA_WIN.amount, 1e36);
  //   // reopen amount should be larger than TICKET_AB_WIN_RECYCLED.amount and 0 respectively
  //   reopenAmount1 = bound(reopenAmount1, TICKET_AB_WIN_RECYCLED.amount, 1e36);
  //   reopenAmount2 = bound(reopenAmount2, 1, 1e36);
  //   _helperWithAReopenedChannel(amount1, amount2, reopenAmount1, reopenAmount2);

  //   vm.prank(accountB.accountAddr);
  //   // fail when source and destination are the same
  //   hoprChannels.redeemTicket(
  //     TICKET_AB_WIN_RECYCLED.source,
  //     TICKET_AB_WIN_RECYCLED.nextCommitment,
  //     TICKET_AB_WIN_RECYCLED.ticketEpoch,
  //     TICKET_AB_WIN_RECYCLED.ticketIndex,
  //     TICKET_AB_WIN_RECYCLED.proofOfRelaySecret,
  //     TICKET_AB_WIN_RECYCLED.amount,
  //     TICKET_AB_WIN_RECYCLED.winProb,
  //     TICKET_AB_WIN_RECYCLED.signature
  //   );

  //   // succeed in redeeming the ticket
  //   HoprChannels.Channel memory channelAB = HoprChannels.Channel(
  //     reopenAmount1 - TICKET_AB_WIN_RECYCLED.amount,
  //     SECRET_1,
  //     0,
  //     1,
  //     HoprChannels.ChannelStatus.OPEN,
  //     2,
  //     0
  //   );

  //   HoprChannels.Channel memory channelBA = HoprChannels.Channel(
  //     amount2 + reopenAmount2 + TICKET_AB_WIN_RECYCLED.amount,
  //     SECRET_2,
  //     0,
  //     0,
  //     HoprChannels.ChannelStatus.OPEN,
  //     1,
  //     0
  //   );
  //   // check vallidate from channels()
  //   assertEqChannels(getChannelFromTuple(hoprChannels, channelIdAB), channelAB);
  //   assertEqChannels(getChannelFromTuple(hoprChannels, channelIdBA), channelBA);
  // }

  // /**
  //  * @dev With a reopened channel:
  //  * it should allow closure
  //  */
  // function test_AllowClosingReopenedChannel(
  //   uint256 amount1,
  //   uint256 amount2,
  //   uint256 reopenAmount1,
  //   uint256 reopenAmount2
  // ) public {
  //   // channel is funded for at least 10 HoprTokens (Ticket's amount)
  //   amount1 = bound(amount1, TICKET_AB_WIN.amount, 1e36);
  //   amount2 = bound(amount2, TICKET_BA_WIN.amount, 1e36);
  //   // reopen amount should be larger than TICKET_AB_WIN_RECYCLED.amount and 0 respectively
  //   reopenAmount1 = bound(reopenAmount1, TICKET_AB_WIN_RECYCLED.amount, 1e36);
  //   reopenAmount2 = bound(reopenAmount2, 1, 1e36);
  //   _helperWithAReopenedChannel(amount1, amount2, reopenAmount1, reopenAmount2);

  //   // accountA initiate channel closure
  //   vm.startPrank(accountA.accountAddr);
  //   // initiate channel closure first
  //   hoprChannels.initiateChannelClosure(accountB.accountAddr);
  //   // increase enough time for channel closure;
  //   vm.warp(block.timestamp + ENOUGH_TIME_FOR_CLOSURE);
  //   // finalize channel closure
  //   vm.mockCall(
  //     vm.addr(1),
  //     abi.encodeWithSignature('transfer(address,uint256)', accountA.accountAddr, reopenAmount1), // provide specific
  //     abi.encode(true)
  //   );
  //   hoprChannels.finalizeChannelClosure(accountB.accountAddr);
  //   vm.stopPrank();

  //   // succeed in redeeming the ticket
  //   HoprChannels.Channel memory channelAB = HoprChannels.Channel(
  //     0,
  //     SECRET_2,
  //     0,
  //     0,
  //     HoprChannels.ChannelStatus.CLOSED,
  //     2,
  //     0
  //   );

  //   HoprChannels.Channel memory channelBA = HoprChannels.Channel(
  //     amount2 + reopenAmount2,
  //     SECRET_2,
  //     0,
  //     0,
  //     HoprChannels.ChannelStatus.OPEN,
  //     1,
  //     0
  //   );
  //   // check vallidate from channels()
  //   assertEqChannels(getChannelFromTuple(hoprChannels, channelIdAB), channelAB);
  //   assertEqChannels(getChannelFromTuple(hoprChannels, channelIdBA), channelBA);
  // }

  // /**
  //  * @dev test internals with mock:
  //  * it should get channel id, returning the same channel ID
  //  */
  // function test_SameChannelId() public {
  //   assertEq(channelIdAB, channelsMockTest.getChannelIdInternal(accountA.accountAddr, accountB.accountAddr));
  // }

  // /**
  //  *@dev Helper function to announce account A and B
  //  */
  // function _helperAnnounceAB(bool annouceA, bool announceB) internal {
  //   // both accountA and accountB are announced
  //   bytes memory multiAddressA = hex'1234';
  //   bytes memory multiAddressB = hex'4321';
  //   if (annouceA) {
  //     vm.prank(accountA.accountAddr);
  //     hoprChannels.announce(accountA.publicKey, multiAddressA);
  //   }
  //   if (announceB) {
  //     vm.prank(accountB.accountAddr);
  //     hoprChannels.announce(accountB.publicKey, multiAddressB);
  //   }
  // }

  // /**
  //  *@dev Helper function to fund channel A->B and B->A with `fundChannelMulti`
  //  */
  // function _helperFundMultiAB(uint256 amount1, uint256 amount2) internal {
  //   // both accountA and accountB are announced
  //   _helperAnnounceAB(true, true);
  //   vm.prank(address(1));
  //   vm.mockCall(
  //     vm.addr(1),
  //     abi.encodeWithSignature(
  //       'transferFrom(address,address,uint256)',
  //       address(1),
  //       address(hoprChannels),
  //       amount1 + amount2
  //     ),
  //     abi.encode(true)
  //   );
  //   // fund channel A->B and B->A
  //   hoprChannels.fundChannelMulti(accountA.accountAddr, accountB.accountAddr, amount1, amount2);
  //   vm.clearMockedCalls();
  // }

  // /**
  //  * @notice overwrites _helperFundMultiAB with checks on emitted events
  //  * @dev Helper function to fund channel A->B and B->A with `fundChannelMulti`
  //  * and check event emissios.
  //  */
  // function _helperFundMultiAB(
  //   uint256 amount1,
  //   uint256 amount2,
  //   address account1ddr,
  //   address account2ddr,
  //   HoprChannels.Channel memory channel12,
  //   HoprChannels.Channel memory channel21
  // ) internal {
  //   // both accountA and accountB are announced
  //   _helperAnnounceAB(true, true);
  //   vm.prank(address(1));
  //   vm.mockCall(
  //     vm.addr(1),
  //     abi.encodeWithSignature(
  //       'transferFrom(address,address,uint256)',
  //       address(1),
  //       address(hoprChannels),
  //       amount1 + amount2
  //     ),
  //     abi.encode(true)
  //   );

  //   // order of logs matters
  //   vm.expectEmit(true, true, false, true, address(hoprChannels));
  //   emit ChannelUpdated(account1ddr, account2ddr, channel12);
  //   vm.expectEmit(true, false, false, false, address(hoprChannels));
  //   emit ChannelOpened(account2ddr, account1ddr);
  //   vm.expectEmit(true, true, false, true, address(hoprChannels));
  //   emit ChannelUpdated(account2ddr, account1ddr, channel21);

  //   // fund channel A->B and B->A
  //   hoprChannels.fundChannelMulti(account1ddr, account2ddr, amount1, amount2);
  //   vm.clearMockedCalls();
  // }

  // /**
  //  * @dev Helper function to fund channel A->B (amount1) and B->A (amount2) to OPEN,
  //  * where both amount1 and amount2 are above the amount of possible winning ticket
  //  * (i.e. TICKET_AB_WIN and TICKET_BA_WIN)
  //  */
  // function _helperOpenBidirectionalChannels(uint256 amount1, uint256 amount2) internal {
  //   // accountB bumps channel A->B with SECRET_2
  //   // accountA bumps channel B->A with SECRET_2
  //   vm.prank(accountB.accountAddr);
  //   hoprChannels.bumpChannel(accountA.accountAddr, SECRET_2);
  //   vm.prank(accountA.accountAddr);
  //   hoprChannels.bumpChannel(accountB.accountAddr, SECRET_2);
  //   // then fund channel
  //   _helperFundMultiAB(amount1, amount2);
  // }

  // /**
  //  * @dev With a closed channel:
  //  */
  // function _helperWithAClosedChannel(uint256 amount1, uint256 amount2) public {
  //   // make channel A->B open
  //   vm.prank(accountB.accountAddr);
  //   hoprChannels.bumpChannel(accountA.accountAddr, SECRET_2);
  //   // fund channels
  //   _helperFundMultiAB(amount1, amount2);

  //   // accountA initiate channel closure
  //   vm.startPrank(accountA.accountAddr);
  //   // initiate channel closure first
  //   hoprChannels.initiateChannelClosure(accountB.accountAddr);
  //   // increase enough time for channel closure;
  //   vm.warp(block.timestamp + ENOUGH_TIME_FOR_CLOSURE);
  //   // finalize channel closure
  //   vm.mockCall(
  //     vm.addr(1),
  //     abi.encodeWithSignature('transfer(address,uint256)', accountA.accountAddr, amount1), // provide specific
  //     abi.encode(true)
  //   );
  //   hoprChannels.finalizeChannelClosure(accountB.accountAddr);
  //   vm.stopPrank();
  // }

  // /**
  //  * @dev With a reopened channel:
  //  */
  // function _helperWithAReopenedChannel(
  //   uint256 amount1,
  //   uint256 amount2,
  //   uint256 reopenAmount1,
  //   uint256 reopenAmount2
  // ) public {
  //   // make channel A->B and B=>A open
  //   _helperOpenBidirectionalChannels(amount1, amount2);

  //   // accountA initiate channel closure
  //   vm.startPrank(accountA.accountAddr);
  //   // initiate channel closure first
  //   hoprChannels.initiateChannelClosure(accountB.accountAddr);
  //   // increase enough time for channel closure;
  //   vm.warp(block.timestamp + ENOUGH_TIME_FOR_CLOSURE);
  //   // finalize channel closure
  //   vm.mockCall(
  //     vm.addr(1),
  //     abi.encodeWithSignature('transfer(address,uint256)', accountA.accountAddr, amount1), // provide specific
  //     abi.encode(true)
  //   );
  //   hoprChannels.finalizeChannelClosure(accountB.accountAddr);
  //   vm.stopPrank();

  //   // fund channel again
  //   vm.prank(address(1));
  //   vm.mockCall(
  //     vm.addr(1),
  //     abi.encodeWithSignature(
  //       'transferFrom(address,address,uint256)',
  //       address(1),
  //       address(hoprChannels),
  //       reopenAmount1 + reopenAmount2
  //     ),
  //     abi.encode(true)
  //   );
  //   // succeed in reopening a channel
  //   hoprChannels.fundChannelMulti(accountA.accountAddr, accountB.accountAddr, reopenAmount1, reopenAmount2);
  //   vm.clearMockedCalls();
  // }
}
