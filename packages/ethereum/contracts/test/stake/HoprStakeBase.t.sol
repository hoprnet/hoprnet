// SPDX-License-Identifier: UNLICENSED
pragma solidity >=0.6.0 <0.9.0;

import '../../src/stake/HoprStakeBase.sol';
import '../utils/ERC1820Registry.sol';
import 'forge-std/Test.sol';

contract HoprStakeBaseTest is Test, ERC1820RegistryFixtureTest {
  // to alter the storage
  using stdStorage for StdStorage;

  HoprStakeBase public hoprStakeBase;
  address public newOwner;
  address public nftAddress;
  address public lockToken;
  address public rewardToken;
  uint256 public programStart;
  uint256 public programEnd;
  uint256 public baseFactorNumerator;
  uint256 public boostCap;
  address[] public accounts = new address[](3);

  /**
   * Manually import the errors and events
   */
  event Sync(address indexed account, uint256 indexed increment);
  event Staked(address indexed account, uint256 indexed actualAmount);
  event Released(address indexed account, uint256 indexed actualAmount);
  event RewardFueled(uint256 indexed amount);
  event Redeemed(address indexed account, uint256 indexed boostTokenId, bool indexed factorRegistered);
  event Claimed(address indexed account, uint256 indexed rewardAmount);
  event NftBlocked(uint256 indexed typeIndex);
  event NftAllowed(uint256 indexed typeIndex);

  function setUp() public virtual override {
    super.setUp();

    newOwner = vm.addr(100); // make address(100) new owner
    nftAddress = vm.addr(101); // mock _nftAddress with vm.addr(2)
    lockToken = vm.addr(102); // mock _lockToken with vm.addr(3)
    rewardToken = vm.addr(103); // mock _rewardToken with vm.addr(4)

    programStart = uint256(block.timestamp); // mock _programStart with block.timestamp
    programEnd = programStart + 3000; // mock _programEnd with block.timestamp + 3000
    baseFactorNumerator = 100; // mock _basicFactorNumerator with 100
    boostCap = 1 ether; // mock _boostCap with 1 ether

    hoprStakeBase = new HoprStakeBase(
      newOwner,
      programStart,
      programEnd,
      baseFactorNumerator,
      boostCap,
      nftAddress,
      lockToken,
      rewardToken
    );

    // assign vm.addr(1) to vm.addr(6) to accounts
    accounts[0] = vm.addr(1);
    accounts[1] = vm.addr(2);
    accounts[2] = vm.addr(3);
  }

  /**
   * @dev owner can block NFTs
   */
  function test_OwnerBlockNft(uint256 typeIndex) public {
    assertFalse(hoprStakeBase.isBlockedNft(typeIndex));

    // for tokens that are already blocked
    vm.prank(newOwner);
    vm.expectEmit(true, false, false, false, address(hoprStakeBase));
    emit NftBlocked(typeIndex);
    hoprStakeBase.ownerBlockNftType(typeIndex);
    vm.clearMockedCalls();
  }

  /**
   * @dev owner cannot block already-blocked NFTs
   */
  function testRevert_OwnerBlockBlockedNft(uint256 typeIndex) public {
    // mock the id beking on the blocked list
    vm.store(
      address(hoprStakeBase),
      bytes32(stdstore.target(address(hoprStakeBase)).sig('isBlockedNft(uint256)').with_key(typeIndex).find()),
      bytes32(abi.encode(true))
    );
    assertTrue(hoprStakeBase.isBlockedNft(typeIndex));

    // for tokens that are already blocked
    vm.prank(newOwner);
    vm.expectRevert('HoprStake: NFT type is already blocked');
    hoprStakeBase.ownerBlockNftType(typeIndex);
    vm.clearMockedCalls();
  }

  /**
   * @dev owner can unblock NFTs
   */
  function test_OwnerUnblockNft(uint256 typeIndex) public {
    // mock the id beking on the blocked list
    vm.store(
      address(hoprStakeBase),
      bytes32(stdstore.target(address(hoprStakeBase)).sig('isBlockedNft(uint256)').with_key(typeIndex).find()),
      bytes32(abi.encode(true))
    );
    assertTrue(hoprStakeBase.isBlockedNft(typeIndex));

    vm.prank(newOwner);
    // for tokens that are already blocked
    vm.expectEmit(true, false, false, false, address(hoprStakeBase));
    emit NftAllowed(typeIndex);
    hoprStakeBase.ownerUnblockNftType(typeIndex);
    vm.clearMockedCalls();
  }

  /**
   * @dev owner cannot unblock allowed NFTs
   */
  function testRevert_OwnerUnblockAllowedNft(uint256 typeIndex) public {
    // for tokens that are already blocked
    assertFalse(hoprStakeBase.isBlockedNft(typeIndex));
    // owner cannot unblock it
    vm.prank(newOwner);
    vm.expectRevert('HoprStake: NFT type is not blocked');
    hoprStakeBase.ownerUnblockNftType(typeIndex);
    vm.clearMockedCalls();
  }

  /**
   * @dev It fails to stake ERC677 tokens other than the lock token
   */
  function testRevert_StakeTokesOtherThanLockToken(address tokenAddr, address account) public {
    vm.assume(tokenAddr != lockToken);

    // mock that account has some token
    vm.prank(tokenAddr);
    vm.mockCall(
      account,
      abi.encodeWithSignature('transfer(address,uint256)', address(hoprStakeBase), 1 ether),
      abi.encode(true)
    );
    // fail to stake erc677 tokens
    vm.expectRevert('HoprStake: Only accept LOCK_TOKEN in staking');
    hoprStakeBase.onTokenTransfer(account, 1 ether, hex'00');
    vm.clearMockedCalls();
  }

  /**
   * @dev It fails to Redeem NFTs other than the boost tokens
   */
  function testRevert_CannotRedeemOtherNFTs(
    address nftAddr,
    uint256 tokenId,
    address account
  ) public {
    vm.assume(nftAddr != nftAddress);
    // account stakes other NFT of an id
    vm.mockCall(
      account,
      abi.encodeWithSignature('safeTransferFrom(address,address,uint256)', account, address(hoprStakeBase), tokenId),
      abi.encode(true)
    );
    vm.prank(nftAddr);
    vm.expectRevert('HoprStake: Cannot SafeTransferFrom tokens other than HoprBoost.');
    hoprStakeBase.onERC721Received(account, account, tokenId, hex'00');
  }

  /**
   * @dev Can redeem allowed NFT
   * it succeeds to redeem nfts nr
   */
  function testFuzz_RedeemAllowedNft(uint256 tokenId) public {
    tokenId = bound(tokenId, 1, 3);
    _helperAccountsStakeTokensAndNFTs();

    // redeem
    vm.mockCall(
      accounts[0],
      abi.encodeWithSignature(
        'safeTransferFrom(address,address,uint256)',
        accounts[0],
        address(hoprStakeBase),
        tokenId
      ),
      abi.encode(true)
    );

    vm.prank(nftAddress);
    vm.expectEmit(true, true, true, false, address(hoprStakeBase));
    emit Redeemed(accounts[0], tokenId, tokenId == 1 ? false : true); // token of id 1 has the sanme type and rank as token 0
    hoprStakeBase.onERC721Received(accounts[0], accounts[0], tokenId, hex'00');
    vm.clearMockedCalls();
  }

  /**
   * @dev For whitelisting:
   * It can get redeemed token (token id 0 and 4) with multiple functions
   * by the accounts[0] and accounts[2] respectively
   */
  function test_ReadInfoOnRedeemedTokens() public {
    _helperAccountsStakeTokensAndNFTs();

    // for accounts[0]
    for (uint256 index = 0; index < 2; index++) {
      uint256 accountIndex = index * 2;
      // isNftTypeAndRankRedeemed1
      assertTrue(hoprStakeBase.isNftTypeAndRankRedeemed1('demo', 'demo', accounts[accountIndex]));
      // isNftTypeAndRankRedeemed2
      assertTrue(hoprStakeBase.isNftTypeAndRankRedeemed2(1, 'demo', accounts[accountIndex]));
      // isNftTypeAndRankRedeemed3
      assertTrue(hoprStakeBase.isNftTypeAndRankRedeemed3(1, 158, accounts[accountIndex]));
      // isNftTypeAndRankRedeemed4
      assertTrue(hoprStakeBase.isNftTypeAndRankRedeemed4('demo', 158, accounts[accountIndex]));
    }
    vm.clearMockedCalls();
  }

  /**
   * @dev For whitelisting:
   * It can return false when requesting for redeemed token (token id 0 and 4) with wrong type/ranks
   * with multiple functions
   * Both tokens are staked by the accounts[0]
   */
  function test_ReadWrongInfoOnRedeemedTokens() public {
    _helperAccountsStakeTokensAndNFTs();

    // it should be false, when getting redeemed token with isNftTypeAndRankRedeemed1, different rank
    assertFalse(hoprStakeBase.isNftTypeAndRankRedeemed1('demo', 'diamond', accounts[0]));
    // it should be false, when getting redeemed token with isNftTypeAndRankRedeemed1, different rank
    assertFalse(hoprStakeBase.isNftTypeAndRankRedeemed1('Rando type', 'demo', accounts[0]));
    // it should be false, when getting redeemed token with isNftTypeAndRankRedeemed2, different rank
    assertFalse(hoprStakeBase.isNftTypeAndRankRedeemed2(1, 'diamond', accounts[0]));
    // it should be false, when getting redeemed token with isNftTypeAndRankRedeemed2, different type
    assertFalse(hoprStakeBase.isNftTypeAndRankRedeemed2(2, 'demo', accounts[0]));
    // it should be false, when getting redeemed token with isNftTypeAndRankRedeemed3, different factor
    assertFalse(hoprStakeBase.isNftTypeAndRankRedeemed3(1, 888, accounts[0]));
    // it should be false, when getting redeemed token with isNftTypeAndRankRedeemed3, different type
    assertFalse(hoprStakeBase.isNftTypeAndRankRedeemed3(2, 158, accounts[0]));
    // it should be false, when getting redeemed token with isNftTypeAndRankRedeemed4, different factor
    assertFalse(hoprStakeBase.isNftTypeAndRankRedeemed4('demo', 888, accounts[0]));
    // it should be false, when getting redeemed token with isNftTypeAndRankRedeemed4, different type
    assertFalse(hoprStakeBase.isNftTypeAndRankRedeemed4('Rando type', 158, accounts[0]));
    vm.clearMockedCalls();
  }

  /**
   * @dev For whitelisting:
   * It can return false when requesting for owned but not yet redeemed token
   * with multiple functions (accounts[1])
   */
  function test_ReadInfoOnOwnedButNotRedeemedTokens() public {
    _helperAccountsStakeTokensAndNFTs();

    // it should be false, when getting redeemed token with isNftTypeAndRankRedeemed1
    assertFalse(hoprStakeBase.isNftTypeAndRankRedeemed1('demo', 'demo', accounts[1]));
    // it should be false, when getting redeemed token with isNftTypeAndRankRedeemed2
    assertFalse(hoprStakeBase.isNftTypeAndRankRedeemed2(1, 'demo', accounts[1]));
    // it should be false, when getting redeemed token with isNftTypeAndRankRedeemed3
    assertFalse(hoprStakeBase.isNftTypeAndRankRedeemed3(1, 158, accounts[1]));
    // isNftTypeAndRankRedeemed4
    assertFalse(hoprStakeBase.isNftTypeAndRankRedeemed4('demo', 158, accounts[1]));
    vm.clearMockedCalls();
  }

  /**
   * @dev After the program ends:
   * It succeeds in advancing block to PROGRAM_END + 1
   * It can no longer lock more ERC677 tokens, nor canit redeem boost NFTs
   */
  function test_WhenPromgramEndsCannotStakeTokens() public {
    _helperAccountsStakeTokensAndNFTs();
    // advance block timestamp to the end of this staking season
    vm.warp(hoprStakeBase.PROGRAM_END() + 1);
    assertEq(block.timestamp, hoprStakeBase.PROGRAM_END() + 1);

    // it cannot lock more ERC677 tokens with `transferAndCall()`
    vm.prank(lockToken);
    vm.mockCall(
      accounts[0],
      abi.encodeWithSignature('transfer(address,uint256)', address(hoprStakeBase), 1 ether),
      abi.encode(true)
    );
    vm.expectRevert('HoprStake: Program ended, cannot stake anymore.');
    hoprStakeBase.onTokenTransfer(accounts[0], 1 ether, hex'00');

    // it cannot redeem NFTs
    vm.prank(nftAddress);
    vm.mockCall(
      accounts[1],
      abi.encodeWithSignature('safeTransferFrom(address,address,uint256)', accounts[0], address(hoprStakeBase), 1),
      abi.encode(true)
    );
    vm.expectRevert('HoprStake: Program ended, cannot redeem boosts.');
    hoprStakeBase.onERC721Received(accounts[1], accounts[1], 1, hex'00');
    vm.clearMockedCalls();
  }

  /**
   * @dev After the program ends:
   * It succeeds in advancing block to PROGRAM_END + 1
   * It can unlock tokens and receive original locked tokens and staked NFTs
   */
  function test_WhenPromgramEndsCanUnlockTokens() public {
    _helperAccountsStakeTokensAndNFTs();
    // advance block timestamp to the end of this staking season
    vm.warp(hoprStakeBase.PROGRAM_END() + 1);
    // check the amount of rewards
    hoprStakeBase.sync(accounts[0]);
    (uint256 actualLocked, uint256 lastSync, uint256 cumulatedRewards, uint256 claimedRewards) = hoprStakeBase.accounts(
      accounts[0]
    );
    emit log_named_uint('actualLocked', actualLocked);
    emit log_named_uint('lastSync', lastSync);
    emit log_named_uint('cumulatedRewards', cumulatedRewards);
    emit log_named_uint('claimedRewards', claimedRewards);

    uint256 availableRewardSlot = stdstore.target(address(hoprStakeBase)).sig('availableReward()').find();
    vm.store(address(hoprStakeBase), bytes32(availableRewardSlot), bytes32(abi.encode(1 ether)));

    // accounts[0] unlocks tokens
    vm.prank(accounts[0]);
    // mock the transfer of reward tokens
    vm.mockCall(
      rewardToken,
      abi.encodeWithSignature('safeTransfer(address,uint256)', accounts[0], cumulatedRewards - claimedRewards),
      abi.encode(true)
    );
    // mock the transfer of lockTokens
    vm.mockCall(
      lockToken,
      abi.encodeWithSignature('safeTransfer(address,uint256)', accounts[0], actualLocked),
      abi.encode(true)
    );
    // mock the redeemed NFT transfers (of token id 0)
    vm.mockCall(
      address(hoprStakeBase),
      abi.encodeWithSignature('safeTransferFrom(address,address,uint256)', address(hoprStakeBase), accounts[0], 0),
      abi.encode(true)
    );
    // perform actual unlock
    vm.expectEmit(true, true, false, false, address(hoprStakeBase));
    emit Released(accounts[0], actualLocked); // token of id 1 has the sanme type and rank as token 0
    hoprStakeBase.unlock();
    vm.clearMockedCalls();
  }

  /**
   * @dev Boost NFTs Nr [0, 1, 2, 3, 4]
   * @notice It's easier to mocke the interface calls than to manipulate the storage as `_boostType` is saved as a tuple
   */
  function _helperMintBoosts() internal {
    // typeOf
    vm.mockCall(nftAddress, abi.encodeWithSignature('typeOf(uint256)', 0), abi.encode('demo'));
    vm.mockCall(nftAddress, abi.encodeWithSignature('typeOf(uint256)', 1), abi.encode('demo'));
    vm.mockCall(nftAddress, abi.encodeWithSignature('typeOf(uint256)', 2), abi.encode('HODLr'));
    vm.mockCall(nftAddress, abi.encodeWithSignature('typeOf(uint256)', 3), abi.encode('HODLr'));
    vm.mockCall(nftAddress, abi.encodeWithSignature('typeOf(uint256)', 4), abi.encode('demo'));

    // tokenURI
    vm.mockCall(
      nftAddress,
      abi.encodeWithSignature('tokenURI(uint256)', 0),
      abi.encode('https://stake.hoprnet.org/demo/demo')
    );
    vm.mockCall(
      nftAddress,
      abi.encodeWithSignature('tokenURI(uint256)', 1),
      abi.encode('https://stake.hoprnet.org/demo/demo')
    );
    vm.mockCall(
      nftAddress,
      abi.encodeWithSignature('tokenURI(uint256)', 2),
      abi.encode('https://stake.hoprnet.org/HODLr/silver')
    );
    vm.mockCall(
      nftAddress,
      abi.encodeWithSignature('tokenURI(uint256)', 3),
      abi.encode('https://stake.hoprnet.org/HODLr/bronze extra')
    );
    vm.mockCall(
      nftAddress,
      abi.encodeWithSignature('tokenURI(uint256)', 4),
      abi.encode('https://stake.hoprnet.org/demo/demo')
    );

    // boostOf
    vm.mockCall(nftAddress, abi.encodeWithSignature('boostOf(uint256)', 0), abi.encode([158, 0]));
    vm.mockCall(nftAddress, abi.encodeWithSignature('boostOf(uint256)', 1), abi.encode([158, 0]));
    vm.mockCall(nftAddress, abi.encodeWithSignature('boostOf(uint256)', 2), abi.encode([158, 0]));
    vm.mockCall(nftAddress, abi.encodeWithSignature('boostOf(uint256)', 3), abi.encode([100, 0]));
    vm.mockCall(nftAddress, abi.encodeWithSignature('boostOf(uint256)', 4), abi.encode([158, 0]));

    // typeIndexOf
    vm.mockCall(nftAddress, abi.encodeWithSignature('typeIndexOf(uint256)', 0), abi.encode(1));
    vm.mockCall(nftAddress, abi.encodeWithSignature('typeIndexOf(uint256)', 1), abi.encode(1));
    vm.mockCall(nftAddress, abi.encodeWithSignature('typeIndexOf(uint256)', 2), abi.encode(2));
    vm.mockCall(nftAddress, abi.encodeWithSignature('typeIndexOf(uint256)', 3), abi.encode(2));
    vm.mockCall(nftAddress, abi.encodeWithSignature('typeIndexOf(uint256)', 4), abi.encode(1));

    // accounts[1] has boost nr [1]
    vm.mockCall(nftAddress, abi.encodeWithSignature('ownerOf(uint256)', 1), abi.encode(accounts[1]));
    // accounts[0] has boost nr [2, 3]
    vm.mockCall(nftAddress, abi.encodeWithSignature('ownerOf(uint256)', 2), abi.encode(accounts[0]));
    vm.mockCall(nftAddress, abi.encodeWithSignature('ownerOf(uint256)', 3), abi.encode(accounts[0]));
  }

  /**
   * @dev account[0] staes 1000 lock tokens to the stake contract
   * account[0] redeems boost nft of id 0 and 4
   */
  function _helperAccountsStakeTokensAndNFTs() internal {
    _helperMintBoosts();

    vm.mockCall(
      accounts[0],
      abi.encodeWithSignature('transfer(address,uint256)', address(hoprStakeBase), 1000 ether),
      abi.encode(true)
    );
    // different from hardhat-style testing; foundry needs to mock the trace caller
    vm.prank(lockToken);
    hoprStakeBase.onTokenTransfer(accounts[0], 1000 ether, hex'00');
    // intermediate check
    (uint256 actualLockedTokenAmount, , , ) = hoprStakeBase.accounts(accounts[0]);
    assertEq(actualLockedTokenAmount, 1000 ether);

    // accounts[0] stakes NFT of id 0
    vm.mockCall(
      accounts[0],
      abi.encodeWithSignature('safeTransferFrom(address,address,uint256)', accounts[0], address(hoprStakeBase), 0),
      abi.encode(true)
    );
    vm.prank(nftAddress);
    hoprStakeBase.onERC721Received(accounts[0], accounts[0], 0, hex'00');

    // accounts[2] stakes NFT of id 4
    vm.mockCall(
      accounts[2],
      abi.encodeWithSignature('safeTransferFrom(address,address,uint256)', accounts[2], address(hoprStakeBase), 4),
      abi.encode(true)
    );
    vm.prank(nftAddress);
    hoprStakeBase.onERC721Received(accounts[2], accounts[2], 4, hex'00');
  }
}
