// SPDX-License-Identifier: GPL-3.0-or-later
pragma solidity >=0.8.0 <0.9.0;

import "forge-std/Test.sol";

import { Safe } from "safe-contracts-1.4.1/Safe.sol";
import { SafeSuiteLibV141 } from "../../utils/SafeSuiteLibV141.sol";
import { ClonesUpgradeable } from "openzeppelin-contracts-upgradeable-4.9.2/proxy/ClonesUpgradeable.sol";

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
 * @title Migration Contract for Hopr Node Safe Upgrade
 * @notice This is a generic contract that facilitates HOPR Node Safe and Hopr module upgrades.
 * HOPR Node Safe version 1.0.0 is the first version of the Hopr Node Safe, which uses Safe.sol
 * as its implementation. This contract allows for the migration of
 * - the HOPR Node Safe singleton address to SafeL2.sol of version 1.4.1
 * - the HOPR Node Safe singleton address to SafeL2.sol of version 1.5.0
 * - the HOPR Module singleton address to HoprNodeManagementModule.sol of version 2.0.0
 */
contract HoprNodeSafeMigration {}