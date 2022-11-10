// SPDX-License-Identifier: UNLICENSED
pragma solidity >=0.6.0 <0.9.0;

import "../src/HoprForwarder.sol";
import "./utils/ERC1820Registry.sol";

contract HoprForwarderTest is Test, ERC1820RegistryFixture {
    HoprForwarder public hoprForwarder;

    function setUp() public virtual override {
        super.setUp();
        hoprForwarder = new HoprForwarder();
    }
}