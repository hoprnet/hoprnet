// SPDX-License-Identifier: UNLICENSED
pragma solidity >=0.6.0 <0.9.0;

import "../mocks/ERC777SnapshotMock.sol";
import "../utils/ERC1820Registry.sol";

contract ERC777SnapshotTest is Test, ERC1820RegistryFixtureTest {
    ERC777SnapshotMock public erc777Snapshot;

    function setUp() public virtual override {
        super.setUp();
        // mock string memory name,
        // mock string memory symbol,
        // mock address initialAccount with address(1)
        // set uint256 initialBalance with 1000
        erc777Snapshot = new ERC777SnapshotMock("ERC 777 Token", "ERC777", address(1), 1000);
    }
}