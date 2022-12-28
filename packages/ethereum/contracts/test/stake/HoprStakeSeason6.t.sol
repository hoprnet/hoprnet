// SPDX-License-Identifier: UNLICENSED
pragma solidity >=0.6.0 <0.9.0;

import '../../src/stake/HoprStakeSeason6.sol';
import '../utils/ERC1820Registry.sol';
import 'forge-std/Test.sol';

contract HoprStakeSeason6Test is Test, ERC1820RegistryFixtureTest {
  HoprStakeSeason6 public hoprStakeSeason6;
  address public constant OWNER = 0xD9a00176Cf49dFB9cA3Ef61805a2850F45Cb1D05;
  address public constant NFT_ADDRESS = 0x43d13D7B83607F14335cF2cB75E87dA369D056c7;
  address public constant LOCK_TOKEN_ADDRESS = 0xD057604A14982FE8D88c5fC25Aac3267eA142a08;
  address public constant REWARD_TOKEN_ADDRESS = 0xD4fdec44DB9D44B8f2b6d529620f9C0C7066A2c1;
  address[] public stakeForAccounts;
  uint256[] public stakeForStakes;

  /**
   * Manually import the errors and events
   */
  event Staked(address indexed account, uint256 indexed actualAmount);

  function setUp() public virtual override {
    super.setUp();

    hoprStakeSeason6 = new HoprStakeSeason6(OWNER, NFT_ADDRESS, LOCK_TOKEN_ADDRESS, REWARD_TOKEN_ADDRESS);
  }

  /**
   * @dev compare token and owner addresses
   */
  function test_RightParameters() public {
    assertEq(OWNER, hoprStakeSeason6.owner());
    assertEq(NFT_ADDRESS, address(hoprStakeSeason6.NFT_CONTRACT()));
    assertEq(LOCK_TOKEN_ADDRESS, hoprStakeSeason6.LOCK_TOKEN());
    assertEq(REWARD_TOKEN_ADDRESS, hoprStakeSeason6.REWARD_TOKEN());
  }

  /**
   * @dev Only owner can stake for other accounts
   */
  function testRevert_WhenNotOwnerCannotStakeForOthers(address caller) public {
    _helperSetAccountsAndStakes(3, 3);
    vm.assume(caller != OWNER);
    vm.prank(caller);
    vm.expectRevert('Ownable: caller is not the owner');
    hoprStakeSeason6.batchStakeFor(stakeForAccounts, stakeForStakes);
  }

  /**
   * @dev Only before program ends can stake for others
   */
  function testRevert_WhenNotOwnerCannotStakeForOthers() public {
    _helperSetAccountsAndStakes(3, 3);
    vm.warp(hoprStakeSeason6.PROGRAM_END() + 1);

    vm.prank(OWNER);
    vm.expectRevert('HoprStake: Program ended, cannot stake anymore.');
    hoprStakeSeason6.batchStakeFor(stakeForAccounts, stakeForStakes);
  }

  /**
   * @dev Only when array lengths match
   */
  function testRevert_WhenArrayLengthsNotMatch(uint256 accLen, uint256 stakeLen) public {
    accLen = bound(accLen, 0, 100);
    stakeLen = bound(stakeLen, 0, 100);
    vm.assume(accLen != stakeLen);
    _helperSetAccountsAndStakes(accLen, stakeLen);

    vm.prank(OWNER);
    vm.expectRevert('HoprStake: accounts and stakes array lengths do not match');
    hoprStakeSeason6.batchStakeFor(stakeForAccounts, stakeForStakes);
  }

  /**
   * #dev owner cannot stake for other accounts when not enough tokens are approved
   */
  function testRevert_WhenNotEnoughApprovedTokensBatchStakeFor(uint256 accLength) public {
    accLength = bound(accLength, 2, 100);
    uint256 amount = _helperSetAccountsAndStakes(accLength, accLength);

    vm.prank(OWNER);
    vm.mockCall(
      LOCK_TOKEN_ADDRESS,
      abi.encodeWithSignature('transferFrom(address,address,uint256)', OWNER, address(hoprStakeSeason6), amount),
      abi.encode(false)
    );
    vm.expectRevert();
    hoprStakeSeason6.batchStakeFor(stakeForAccounts, stakeForStakes);
  }

  /**
   * #dev owner can stake for other accounts
   */
  function test_batchStakeFor(uint256 accLength) public {
    accLength = bound(accLength, 0, 100);
    uint256 amount = _helperSetAccountsAndStakes(accLength, accLength);

    vm.prank(OWNER);
    vm.mockCall(
      LOCK_TOKEN_ADDRESS,
      abi.encodeWithSignature('transferFrom(address,address,uint256)', OWNER, address(hoprStakeSeason6), amount),
      abi.encode(true)
    );
    for (uint256 index = 0; index < accLength; index++) {
      vm.expectEmit(true, true, false, false, address(hoprStakeSeason6));
      emit Staked(stakeForAccounts[index], stakeForStakes[index]);
    }
    hoprStakeSeason6.batchStakeFor(stakeForAccounts, stakeForStakes);
  }

  /**
   * @dev helper function to create an array of accounts and token amounts
   */
  function _helperSetAccountsAndStakes(uint256 lengthAccounts, uint256 lengthStake) internal returns (uint256 sum) {
    stakeForAccounts = new address[](lengthAccounts);
    for (uint256 i = 0; i < lengthAccounts; i++) {
      stakeForAccounts[i] = vm.addr(i + 1);
    }
    stakeForStakes = new uint256[](lengthStake);
    for (uint256 j = 0; j < lengthStake; j++) {
      stakeForStakes[j] = j;
      sum += j;
    }
  }
}
