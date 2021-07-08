// SPDX-License-Identifier: GPL-3.0
pragma solidity ^0.8;

/**
 * HoprRegistry is a temporary smart contract
 * to help us link an account's unclaimed tokens
 * in HoprDistributor with a running HOPR node.
 */
contract HoprRegistry {
    IHoprDistributor public distributor;

    // keep track of registered addresses
    address[] public registered;
    // map between claimant address and node address
    mapping(address => address) public links;

    // event will be primarily used for analytics
    event LinkCreated(address claimant, address node);

    /**
     * @param _distributor HoprDistributor address
     */
    constructor(IHoprDistributor _distributor) {
        distributor = _distributor;
    }

    /**
     * Sets a link between a claimant's address and a node's address.
     * Claimant (msg.sender) must have claimable tokens in schedules
     * "EarlyTokenBuyers" or "TeamAndAdvisors".
     * @param node node's address
     */
    function setLink(address node) external {
        require(
            distributor.getClaimable(msg.sender, "EarlyTokenBuyers") > 0 ||
            distributor.getClaimable(msg.sender, "TeamAndAdvisors") > 0,
            "Claimant has no claimable tokens"
        );
        require(node != address(0), "Node address is not valid");

        // first time registering
        if (links[msg.sender] == address(0)) {
            registered.push(msg.sender);
        }
        // update assigned address
        links[msg.sender] = node;
        emit LinkCreated(msg.sender, node);
    }

    /**
     * Inefficient way for getting all links.
     * @return returns all links
     */
    function getLinks() view external returns (address[2][] memory) {
        address[2][] memory result = new address[2][](registered.length);

        for (uint256 i=0; i<registered.length; i++) {
            address claimant = registered[i];
            address node = links[claimant];
            address[2] memory arr = [claimant, node];
            result[i] = arr;
        }

        return result;
    }
}

/**
 * HoprDistributor's required interface
 * HoprDistributor is written for solidity version ^0.6.0
 * which makes importing it impossible.
 */
interface IHoprDistributor {
    function getClaimable(address account, string calldata scheduleName) external view returns (uint128);
}
