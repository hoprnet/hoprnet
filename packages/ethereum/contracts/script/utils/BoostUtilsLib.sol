// SPDX-License-Identifier: GPL-3.0-or-later
pragma solidity >=0.8.0 <0.9.0;

import "../../src/static/stake/HoprBoost.sol";

/**
 * library to try catch typeAt() function of HoprBoost
 */
library BoostUtilsLib {
    /**
     * returns
     * hasIndex bool check if index is occupied
     * boostType string name of boost
     */
    function getBoostTypeAtIndex(
        address hoprBoostAddr,
        uint256 index
    )
        public
        view
        returns (bool hasIndex, string memory boostType)
    {
        HoprBoost hoprBoost = HoprBoost(hoprBoostAddr);

        // Option 1: use contract type
        try hoprBoost.typeAt(index) returns (string memory _boostType) {
            return (true, _boostType);
        } catch Panic(uint256 panicCode) {
            // panic code 0x32 (uint256 50): Index out of bounds.
            // See
            // https://docs.soliditylang.org/en/v0.8.13/control-structures.html#panic-via-assert-and-error-via-require
            if (panicCode != 50) {
                revert("Panic at getBoostTypeAtIndex");
                // TODO: return the actual panic code
                // return (false, string(bytes(abi.encodePacked(bytes32(panicCode)))));
            }
            return (false, "0x32");
        }

        // Option 2: pure static method
        // (bool successReadTypeAtIndex, bytes memory returndataReadTypeAtIndex) =
        // currentNetworkDetail.hoprBoostContractAddress.staticcall(abi.encodeWithSignature("typeAt()", index));
        // if (!successReadTypeAtIndex) {
        //     emit log_string("Cannot read Boost type at index");
        // } else {
        //     string memory boostType = abi.decode(returndataReadTypeAtIndex, (string));
        //     emit log_named_string("boostType", boostType);
        // }
    }
}
