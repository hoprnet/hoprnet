// SPDX-License-Identifier: GPL-3.0-or-later
pragma solidity 0.8.19;

library PrecompileUtils {
    /**
     * Checks if given address is a Foundry precompile
     * see https://book.getfoundry.sh/misc/precompile-registry
     */
    function isPrecompileAddress(address maybePrecompile) public pure returns (bool) {
        address[12] memory precompiles = [
            0x0000000000000000000000000000000000000001, //	ECRecover
            0x0000000000000000000000000000000000000002, //	SHA-256
            0x0000000000000000000000000000000000000003, //	RIPEMD-160
            0x0000000000000000000000000000000000000004, //	Identity
            0x0000000000000000000000000000000000000005, //	ModExp
            0x0000000000000000000000000000000000000006, //	ECAdd
            0x0000000000000000000000000000000000000007, //	ECMul
            0x0000000000000000000000000000000000000008, //  ECPairing
            0x0000000000000000000000000000000000000009, //  Blake2F
            0x4e59b44847b379578588920cA78FbF26c0B4956C, // Create2Deployer
            0x7109709ECfa91a80626fF3989D68f67F5b1DD12D, // VM
            0x000000000000000000636F6e736F6c652e6c6f67 // Console
        ];

        for (uint256 i = 0; i < precompiles.length; i++) {
            if (maybePrecompile == precompiles[i]) {
                return true;
            }
        }

        return false;
    }
}
