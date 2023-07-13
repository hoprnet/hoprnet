// SPDX-License-Identifier: UNLICENSED
pragma solidity >=0.6.0 <0.9.0;

import '../../src/proxy/SafeProxyForNetworkRegistry.sol';
import 'forge-std/Test.sol';

contract HoprSafeProxyForNetworkRegistryTest is Test {
  HoprSafeProxyForNetworkRegistry public hoprSafeProxyForNetworkRegistry;
  address public owner;
  address public token;
  address public safeAddress;
  address public nodeSafeRegistry;
  uint256 public stakeThreshold;
  address[] public accounts = new address[](7);

  /**
   * Manually import the errors and events
   */

  function setUp() public virtual {
    owner = vm.addr(101); // make address(101) new owner
    token = vm.addr(102); // make address(102) new token
    safeAddress = vm.addr(255); // make address(255) the default safe address
    nodeSafeRegistry = vm.addr(103); // make vm.addr(103) nodeSafeRegistry

    stakeThreshold = 500 ether;
    // set _minStake with the production value
    hoprSafeProxyForNetworkRegistry = new HoprSafeProxyForNetworkRegistry(
        owner,
        stakeThreshold,
        token,
        nodeSafeRegistry
    );

    // assign vm.addr(1) to vm.addr(6) to accounts
    accounts[0] = vm.addr(1);
    accounts[1] = vm.addr(2);
    accounts[2] = vm.addr(3);
    accounts[3] = vm.addr(4);
    accounts[4] = vm.addr(5);
    accounts[5] = vm.addr(6);
    accounts[6] = vm.addr(7);
  }
    /**
     * @dev test the maximum amount of nodes that 
     */
  function testFuzz_MaxAllowedRegistrations() public {

  }

  function _helpeMockSafeRegistyAndTokenBalance(
    address nodeAddr,
    uint256 tokenBalance
  ) private {
    // nodeSafeRegistry is able to reply to call nodeToSafe
    vm.mockCall(
      nodeSafeRegistry,
      abi.encodeWithSignature(
        'nodeToSafe(address)',
        nodeAddr
      ),
      abi.encode(true)
    );
    // balanceOf safeAddress to be the given balance
    vm.mockCall(
      token,
      abi.encodeWithSignature(
        'balanceOf(address)',
        safeAddress
      ),
      abi.encode(tokenBalance)
    );
  }
}