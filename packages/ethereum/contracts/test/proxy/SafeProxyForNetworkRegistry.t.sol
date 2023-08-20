// SPDX-License-Identifier: GPL-3.0-or-later
pragma solidity >=0.6.0 <0.9.0;

import "../../src/proxy/SafeProxyForNetworkRegistry.sol";
import "forge-std/Test.sol";

contract HoprSafeProxyForNetworkRegistryTest is Test {
    HoprSafeProxyForNetworkRegistry public hoprSafeProxyForNetworkRegistry;
    address public owner;
    address public token;
    address public safeAddress;
    address public nodeSafeRegistry;
    uint256 public constant DEFAULT_STAKE_THRESHOLD = 500 ether;
    uint128 public constant DEFAULT_SNAPSHOT_BLOCK_NUMBER = 123;

    /**
     * Manually import the errors and events
     */

    function setUp() public virtual {
        owner = vm.addr(101); // make address(101) new owner
        token = vm.addr(102); // make address(102) new token
        safeAddress = vm.addr(255); // make address(255) the default safe address
        nodeSafeRegistry = vm.addr(103); // make vm.addr(103) nodeSafeRegistry

        // set _minStake with the production value
        hoprSafeProxyForNetworkRegistry = new HoprSafeProxyForNetworkRegistry(
        owner,
        owner,
        DEFAULT_STAKE_THRESHOLD,
        DEFAULT_SNAPSHOT_BLOCK_NUMBER,
        token,
        nodeSafeRegistry
    );
    }

    /**
     * @dev test interface id is supported
     */
    function test_supportsInterface() public {
        bytes4 interfaceId = type(IHoprNetworkRegistryRequirement).interfaceId;
        assertTrue(hoprSafeProxyForNetworkRegistry.supportsInterface(interfaceId));
    }

    /**
     * @dev test the maximum amount of nodes that
     */
    function testFuzz_MaxAllowedRegistrations(address stakingAccount, uint256 tokenBalance) public {
        _helpeMockSafeRegistyAndTokenBalance(stakingAccount, DEFAULT_SNAPSHOT_BLOCK_NUMBER, tokenBalance);
        assertEq(
            hoprSafeProxyForNetworkRegistry.maxAllowedRegistrations(stakingAccount),
            tokenBalance / DEFAULT_STAKE_THRESHOLD
        );
        vm.clearMockedCalls();
    }

    /**
     * @dev manager can set the threshold
     */
    function testFuzz_SetThreshold(uint256 newThreshold) public {
        vm.assume(newThreshold != DEFAULT_STAKE_THRESHOLD);
        vm.prank(owner);
        hoprSafeProxyForNetworkRegistry.updateStakeThreshold(newThreshold);
        assertEq(hoprSafeProxyForNetworkRegistry.stakeThreshold(), newThreshold);
    }
    /**
     * @dev fail to calculate allowance when threshold is 0.
     * @notice This essentially disables "selfRegister" a node
     */

    function testRevert_SetThresholdToZero(address stakingAccount, uint256 tokenBalance) public {
        _helpeMockSafeRegistyAndTokenBalance(stakingAccount, DEFAULT_SNAPSHOT_BLOCK_NUMBER, tokenBalance);

        vm.prank(owner);
        hoprSafeProxyForNetworkRegistry.updateStakeThreshold(0);
        vm.expectRevert(stdError.divisionError);
        hoprSafeProxyForNetworkRegistry.maxAllowedRegistrations(stakingAccount);
        vm.clearMockedCalls();
    }

    /**
     * @dev manager cannot set the threshold of the same value
     */
    function testRevert_UpdateThreshold() public {
        vm.prank(owner);
        vm.expectRevert(HoprSafeProxyForNetworkRegistry.SameValue.selector);
        hoprSafeProxyForNetworkRegistry.updateStakeThreshold(DEFAULT_STAKE_THRESHOLD);
    }

    /**
     * @dev manager can set the snapshot block number
     */
    function testFuzz_UpdateSnapshotBlockNumber(uint128 newSnapshotBlockNumber) public {
        vm.assume(newSnapshotBlockNumber != DEFAULT_SNAPSHOT_BLOCK_NUMBER);
        vm.prank(owner);
        hoprSafeProxyForNetworkRegistry.updateSnapshotBlockNumber(newSnapshotBlockNumber);
        assertEq(hoprSafeProxyForNetworkRegistry.snapshotBlockNumber(), newSnapshotBlockNumber);
    }

    /**
     * @dev test return of operate for
     */
    function testFuzz_CanOperateFor(address otherSafeAddress) public {
        vm.assume(otherSafeAddress != safeAddress);
        address nodeAddress = vm.addr(254);
        // other nodes point to a different address than safeAddress
        vm.mockCall(nodeSafeRegistry, abi.encodeWithSignature("nodeToSafe(address)"), abi.encode(vm.addr(1)));
        // nodeSafeRegistry is able to reply to call nodeToSafe
        vm.mockCall(
            nodeSafeRegistry, abi.encodeWithSignature("nodeToSafe(address)", nodeAddress), abi.encode(safeAddress)
        );
        assertTrue(hoprSafeProxyForNetworkRegistry.canOperateFor(safeAddress, nodeAddress));
        assertFalse(hoprSafeProxyForNetworkRegistry.canOperateFor(otherSafeAddress, nodeAddress));
        vm.clearMockedCalls();
    }

    /**
     * @dev manager cannot set the threshold of the same value
     */
    function testRevert_UpdateSnapshotBlockNumber() public {
        vm.prank(owner);
        vm.expectRevert(HoprSafeProxyForNetworkRegistry.SameValue.selector);
        hoprSafeProxyForNetworkRegistry.updateSnapshotBlockNumber(DEFAULT_SNAPSHOT_BLOCK_NUMBER);
    }

    function _helpeMockSafeRegistyAndTokenBalance(address safeAddr, uint128 blockNum, uint256 tokenBalance) private {
        // balanceOf safeAddress to be the given balance
        vm.mockCall(
            token, abi.encodeWithSignature("balanceOfAt(address,uint128)", safeAddr, blockNum), abi.encode(tokenBalance)
        );
    }
}
