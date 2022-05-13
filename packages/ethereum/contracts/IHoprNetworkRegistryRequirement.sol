// SPDX-License-Identifier: GPL-3.0
pragma solidity ^0.8.0;

interface IHoprNetworkRegistryRequirement {
  function isRequirementFulfilled(address account) external view returns (bool);
}
