// SPDX-License-Identifier: GPL-3.0-or-later
pragma solidity >=0.8.0 <0.9.0;

/**
 * library for Safe 1.4.0 addresses
 * deployed contract addresses on Gnosis Chain from
 * https://github.com/safe-global/safe-deployments/tree/2dc3a9913ccc7a524ccfdacd50a2f864f423cf2b/src/assets/v1.4.0
 */
library SafeSuiteLib {
    address internal constant SAFE_SimulateTxAccessor_ADDRESS = 0x3d4BA2E0884aa488718476ca2FB8Efc291A46199;
    address internal constant SAFE_SafeProxyFactory_ADDRESS = 0x4e1DCf7AD4e460CfD30791CCC4F9c8a4f820ec67;
    address internal constant SAFE_TokenCallbackHandler_ADDRESS = 0xeDCF620325E82e3B9836eaaeFdc4283E99Dd7562;
    address internal constant SAFE_CompatibilityFallbackHandler_ADDRESS = 0x2a15DE4410d4c8af0A7b6c12803120f43C42B820;
    address internal constant SAFE_CreateCall_ADDRESS = 0x9b35Af71d77eaf8d7e40252370304687390A1A52;
    address internal constant SAFE_MultiSend_ADDRESS = 0x38869bf66a61cF6bDB996A6aE40D5853Fd43B526;
    address internal constant SAFE_MultiSendCallOnly_ADDRESS = 0x9641d764fc13c8B624c04430C7356C1C7C8102e2;
    address internal constant SAFE_SignMessageLib_ADDRESS = 0x58FCe385Ed16beB4BCE49c8DF34c7d6975807520;
    address internal constant SAFE_Safe_ADDRESS = 0xc962E67D9490E154D81181879ddf4CD3b65D2132;
    string internal constant SAFE_VERSION = "1.4.0";
}
