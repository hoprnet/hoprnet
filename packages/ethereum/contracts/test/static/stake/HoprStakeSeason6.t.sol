// SPDX-License-Identifier: GPL-3.0-or-later
pragma solidity >=0.6.0 <0.9.0;

import "../../../src/static/stake/HoprStakeSeason6.sol";
import "../../utils/ERC1820Registry.sol";
import "../../utils/PermittableToken.sol";
import "forge-std/Test.sol";

contract HoprStakeSeason6Test is Test, ERC1820RegistryFixtureTest, PermittableTokenFixtureTest {
    // to alter the storage
    using stdStorage for StdStorage;

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
        if (block.chainid == 31_337) {
            etchPermittableTokenAt(LOCK_TOKEN_ADDRESS);
        }
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
        vm.expectRevert("Ownable: caller is not the owner");
        hoprStakeSeason6.batchStakeFor(stakeForAccounts, stakeForStakes);
    }

    /**
     * @dev Only before program ends can stake for others
     */
    function testRevert_WhenNotOwnerCannotStakeForOthers() public {
        _helperSetAccountsAndStakes(3, 3);
        vm.warp(hoprStakeSeason6.PROGRAM_END() + 1);

        vm.prank(OWNER);
        vm.expectRevert("HoprStake: Program ended, cannot stake anymore.");
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
        vm.expectRevert("HoprStake: accounts and stakes array lengths do not match");
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
            abi.encodeWithSignature("transferFrom(address,address,uint256)", OWNER, address(hoprStakeSeason6), amount),
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

        vm.startPrank(OWNER);
        // mock the caller (OWNER) has LOCK_TOKEN to stake for others
        vm.store(
            address(LOCK_TOKEN_ADDRESS),
            bytes32(stdstore.target(address(LOCK_TOKEN_ADDRESS)).sig("balanceOf(address)").with_key(OWNER).find()),
            bytes32(abi.encode(amount))
        );
        (bool successReadTokenBalance, bytes memory returndataTokenBalance) =
            LOCK_TOKEN_ADDRESS.staticcall(abi.encodeWithSignature("balanceOf(address)", OWNER));
        assertTrue(successReadTokenBalance);
        assertEq(abi.decode(returndataTokenBalance, (uint256)), amount);
        // mock the caller (OWNER) has allowance for the staking contract to transfer LOCK_TOKEN
        vm.store(
            address(LOCK_TOKEN_ADDRESS),
            bytes32(
                stdstore.target(address(LOCK_TOKEN_ADDRESS)).sig("allowance(address,address)").with_key(OWNER).with_key(
                    address(hoprStakeSeason6)
                ).find()
            ),
            bytes32(abi.encode(amount))
        );
        (bool successReadAllowance, bytes memory returndatallowance) = LOCK_TOKEN_ADDRESS.staticcall(
            abi.encodeWithSignature("allowance(address,address)", OWNER, address(hoprStakeSeason6))
        );
        assertTrue(successReadAllowance);
        assertEq(abi.decode(returndatallowance, (uint256)), amount);

        for (uint256 index = 0; index < accLength; index++) {
            vm.expectEmit(true, true, false, false, address(hoprStakeSeason6));
            emit Staked(stakeForAccounts[index], stakeForStakes[index]);
        }
        hoprStakeSeason6.batchStakeFor(stakeForAccounts, stakeForStakes);

        // check each account has its respective stake
        for (uint256 j = 0; j < accLength; j++) {
            (uint256 actualLockedAfterBatchStake,,,) = hoprStakeSeason6.accounts(stakeForAccounts[j]);
            assertEq(actualLockedAfterBatchStake, stakeForStakes[j]);
        }
        // caller doesn't have stake
        (uint256 actualLockedOfCaller,,,) = hoprStakeSeason6.accounts(OWNER);
        assertEq(actualLockedOfCaller, 0);
        // check the total stake in the contract equals to the provided value
        assertEq(hoprStakeSeason6.totalLocked(), amount);

        // check the LOCK_TOKEN balance of the contract equals to the provided value
        (bool successReadBalanceOfStake, bytes memory returndatBalanceOfStake) =
            LOCK_TOKEN_ADDRESS.staticcall(abi.encodeWithSignature("balanceOf(address)", address(hoprStakeSeason6)));
        assertTrue(successReadBalanceOfStake);
        assertEq(abi.decode(returndatBalanceOfStake, (uint256)), amount);
        // check the remaining LOCK_TOKEN stake is zero
        (bool successReadBalanceOfOwner, bytes memory returndatBalanceOfOwner) =
            LOCK_TOKEN_ADDRESS.staticcall(abi.encodeWithSignature("balanceOf(address)", OWNER));
        assertTrue(successReadBalanceOfOwner);
        assertEq(abi.decode(returndatBalanceOfOwner, (uint256)), 0);
        vm.stopPrank();
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
