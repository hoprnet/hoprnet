// SPDX-License-Identifier: GPL-3.0
pragma solidity 0.8.30;

import { Ownable2Step, Ownable } from "openzeppelin-contracts-5.4.0/access/Ownable2Step.sol";

type WinProb is uint56;

using { equal as == } for WinProb global;

function equal(WinProb a, WinProb b) pure returns (bool) {
    return WinProb.unwrap(a) == WinProb.unwrap(b);
}

abstract contract HoprWinningProbabilityOracleEvents {
    // emitted when winning probability was updated
    event WinProbUpdated(WinProb oldWinProb, WinProb newWinProb);
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
 * @title HoprWinningProbabilityOracle
 * @dev Oracle which defines the current minimum winning probability used in a HOPR network.
 * Exposes a single function to set a new global minimum winning probability set by the contract owner.
 *
 * The winning probability is written as IEEE 754 double precision floating point number.
 *
 * The current winning probability can be read via `currentWinProb()`.
 *
 * An update of the winning probablility triggers an event `WinProbUpdated`.
 * The winning probability `WinProb` is stored in uint56, the same as in the HoprChannels contract.
 */
contract HoprWinningProbabilityOracle is Ownable2Step, HoprWinningProbabilityOracleEvents {
    WinProb public currentWinProb;

    // when new winning probability is equal to old winning probability
    error WinProbMustNotBeSame();

    /**
     * @param _newOwner Address of the new owner.
     * @param _initialWinProb Initial winning probability to set.
     */
    constructor(address _newOwner, WinProb _initialWinProb) Ownable(_newOwner) {
        _setWinProb(_initialWinProb);
    }

    /**
     * @dev Owner can set a new winning probability.
     * @param _newWinProb new winning probability, must not be the same as what is already set
     */
    function setWinProb(WinProb _newWinProb) external onlyOwner {
        if (_newWinProb == currentWinProb) {
            revert WinProbMustNotBeSame();
        }
        _setWinProb(_newWinProb);
    }

    /**
     * @dev Set a new winning probability.
     * @param _newWinProb new winning probability, must not be the same as what is already set
     */
    function _setWinProb(WinProb _newWinProb) internal {
        WinProb oldWinProb = currentWinProb;
        currentWinProb = _newWinProb;

        emit WinProbUpdated(oldWinProb, currentWinProb);
    }
}
