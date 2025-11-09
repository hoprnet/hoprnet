// SPDX-License-Identifier: GPL-3.0
pragma solidity 0.8.19;

import { Ownable } from "openzeppelin-contracts/access/Ownable.sol";

abstract contract HoprTicketPriceOracleEvents {
    // emitted when ticket price was updated
    event TicketPriceUpdated(uint256, uint256);
}

/**
 *    &&&&
 *    &&&&
 *    &&&&
 *    &&&&  &&&&&&&&&       &&&&&&&&&&&&          &&&&&&&&&&/   &&&&.&&&&&&&&&
 *    &&&&&&&&&   &&&&&   &&&&&&     &&&&&,     &&&&&    &&&&&  &&&&&&&&   &&&&
 *     &&&&&&      &&&&  &&&&#         &&&&   &&&&&       &&&&& &&&&&&     &&&&&
 *     &&&&&       &&&&/ &&&&           &&&& #&&&&        &&&&  &&&&&
 *     &&&&         &&&& &&&&&         &&&&  &&&&        &&&&&  &&&&&
 *     %%%%        /%%%%   %%%%%%   %%%%%%   %%%%  %%%%%%%%%    %%%%%
 *    %%%%%        %%%%      %%%%%%%%%%%    %%%%   %%%%%%       %%%%
 *                                          %%%%
 *                                          %%%%
 *                                          %%%%
 *
 * @title HoprTicketPriceOracle
 * @dev Oracle which defines the current ticket price used in a HOPR network.
 * Exposes a single function to set a new ticket price which will be used once
 * committed on-chain.
 * The current ticket price can be read via `currentTicketPrice()`.
 * An update of the ticket price triggers an event `TicketPriceUpdated`.
 *
 */
contract HoprTicketPriceOracle is Ownable, HoprTicketPriceOracleEvents {
    uint256 public currentTicketPrice;

    // when new ticket price is 0
    error TicketPriceMustNotBeZero();
    // when new ticket price is equal to old ticket price
    error TicketPriceMustNotBeSame();

    /**
     * @param _newOwner Address of the new owner.
     * @param _initialTicketPrice Initially set ticket price (in Wei).
     */
    constructor(address _newOwner, uint256 _initialTicketPrice) {
        transferOwnership(_newOwner);
        _setTicketPrice(_initialTicketPrice);
    }

    /**
     * @dev Owner can set a new ticket price (in Wei).
     * @param _newTicketPrice new ticket price, must be larger than 0 and not the same as
     * already set
     */
    function setTicketPrice(uint256 _newTicketPrice) external onlyOwner {
        _setTicketPrice(_newTicketPrice);
    }

    /**
     * @dev Set a new ticket price (in Wei).
     * @param _newTicketPrice new ticket price, must be larger than 0 and not the same as
     * already set
     */
    function _setTicketPrice(uint256 _newTicketPrice) internal {
        if (_newTicketPrice == 0) {
            revert TicketPriceMustNotBeZero();
        }
        if (_newTicketPrice == currentTicketPrice) {
            revert TicketPriceMustNotBeSame();
        }

        uint256 oldTicketPrice = currentTicketPrice;
        currentTicketPrice = _newTicketPrice;

        emit TicketPriceUpdated(oldTicketPrice, currentTicketPrice);
    }
}
