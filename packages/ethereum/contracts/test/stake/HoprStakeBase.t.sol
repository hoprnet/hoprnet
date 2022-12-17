// SPDX-License-Identifier: UNLICENSED
pragma solidity >=0.6.0 <0.9.0;

import '../../src/stake/HoprStakeBase.sol';
import '../utils/ERC1820Registry.sol';
import 'forge-std/Test.sol';

contract HoprStakeBaseTest is Test, ERC1820RegistryFixtureTest {
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
   * @dev Can redeem allowed NFT
   * it succeeds to redeem nfts nr
   */
  function test_RedeemAllowedNft() public {
    _helperMintBoosts();
    _helperAccountsStakeTokensAndNFTs();
  }

  /**
   * @dev Boost NFTs Nr [0, 1, 2, 3, 4]
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
    vm.mockCall(nftAddress, abi.encodeWithSignature('boostOf(uint256)', 0), abi.encode([158, 1]));
    vm.mockCall(nftAddress, abi.encodeWithSignature('boostOf(uint256)', 1), abi.encode([158, 1]));
    vm.mockCall(nftAddress, abi.encodeWithSignature('boostOf(uint256)', 2), abi.encode([158, 2]));
    vm.mockCall(nftAddress, abi.encodeWithSignature('boostOf(uint256)', 3), abi.encode([100, 2]));
    vm.mockCall(nftAddress, abi.encodeWithSignature('boostOf(uint256)', 4), abi.encode([158, 1]));

    // accounts[0] has boost nr [0, 2, 3]
    // accounts[1] has boost nr [1]
    // accounts[2] has boost nr [4]
  }

  function _helperAccountsStakeTokensAndNFTs() internal {
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

    // stake NFTs
  }
}
