// SPDX-License-Identifier: GPL-3.0-or-later
pragma solidity ^0.8;

/**
 * @title HoprNodeManagementModule interface
 */
interface IHoprNodeManagementModule {
    function isHoprNodeManagementModule() external view returns (bool);
    function isNode(address nodeAddress) external view returns (bool);
}
