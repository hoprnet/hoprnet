// SPDX-License-Identifier: GPL-3.0
pragma solidity >=0.6.0 <0.9.0;

import 'forge-std/Test.sol';

/**
 * Contract deployed bytecode and addresses
 * https://github.com/safe-global/safe-deployments
 */
abstract contract SafeSingletonFixtureTest is Test {
  address constant SAFE_SINGLETON_ADDRESS = 0x914d7Fec6aaC8cd542e72Bca78B30650d45643d7;
  bytes constant MAINNET_SAFE_SINGLETON_DEPLOYED_CODE =
    bytes(
      hex'7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe03601600081602082378035828234f58015156039578182fd5b8082525050506014600cf3'
    );

  function setUp() public virtual {
    // deploy singleton contract
    vm.etch(SAFE_SINGLETON_ADDRESS, MAINNET_SAFE_SINGLETON_DEPLOYED_CODE);
  }

  function hasSingletonContract() internal view returns (bool) {
    uint256 singletonCodeSize = 0;
    assembly {
      singletonCodeSize := extcodesize(SAFE_SINGLETON_ADDRESS)
    }
    return singletonCodeSize > 0;
  }

  function mustHaveSingletonContract() internal view {
    require(hasSingletonContract(), 'No Safe Singleton deployed');
  }
}
