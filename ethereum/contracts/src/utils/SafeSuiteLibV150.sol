// SPDX-License-Identifier: GPL-3.0-or-later
pragma solidity >=0.8.0 <0.9.0;

/**
 * library for Safe 1.5.0 addresses
 * deployed contract addresses on Gnosis Chain from
 * https://github.com/safe-global/safe-deployments/tree/7a868b24393860b1bc5db8811f4cbbc10c58032d/src/assets/v1.5.0
 */
/// forge-lint: disable-next-item(screaming-snake-case-const)
library SafeSuiteLibV150 {
    address internal constant SAFE_ExtensibleFallbackHandler_ADDRESS = 0x85a8ca358D388530ad0fB95D0cb89Dd44Fc242c3;
    address internal constant SAFE_CompatibilityFallbackHandler_ADDRESS = 0x3EfCBb83A4A7AfcB4F68D501E2c2203a38be77f4;
    address internal constant SAFE_SafeL2_ADDRESS = 0xEdd160fEBBD92E350D4D398fb636302fccd67C7e;
    string internal constant SAFE_VERSION = "1.5.0";
}
