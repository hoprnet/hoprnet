// SPDX-License-Identifier: GPL-3.0
pragma solidity ^0.8.0;

import 'openzeppelin-contracts-4.4.2/utils/introspection/IERC1820Registry.sol';
import 'openzeppelin-contracts-4.4.2/token/ERC777/IERC777Recipient.sol';
import 'openzeppelin-contracts-4.4.2/token/ERC20/IERC20.sol';
import 'openzeppelin-contracts-4.4.2/token/ERC20/utils/SafeERC20.sol';

// https://github.com/omni/omnibridge/blob/c814f686487c50462b132b9691fd77cc2de237d3/contracts/interfaces/IERC677.sol
interface IERC677 is IERC20 {
  event Transfer(address indexed from, address indexed to, uint256 value, bytes data);

  function transferAndCall(
    address to,
    uint256 value,
    bytes calldata data
  ) external returns (bool);

  function increaseAllowance(address spender, uint256 addedValue) external returns (bool);

  function decreaseAllowance(address spender, uint256 subtractedValue) external returns (bool);
}

contract HoprWrapperProxy is IERC777Recipient {
  using SafeERC20 for IERC20;

  IERC1820Registry public constant ERC1820_REGISTRY = IERC1820Registry(0x1820a4B7618BdE71Dce8cdc73aAB6C95905faD24);
  bytes32 public constant TOKENS_RECIPIENT_INTERFACE_HASH = keccak256('ERC777TokensRecipient');
  address public constant WXHOPR_TOKEN = 0xD4fdec44DB9D44B8f2b6d529620f9C0C7066A2c1;
  address public constant XHOPR_TOKEN = 0xD057604A14982FE8D88c5fC25Aac3267eA142a08;
  address public constant WRAPPER = 0x097707143e01318734535676cfe2e5cF8b656ae8;
  address payable public constant XDAI_MULTISIG = payable(0x5E1c4e7004B7411bA27Dc354330fab31147DFeF1);

  event FowardedTo(address to, uint256 amount);
  event FowardedFrom(address from, uint256 amount);

  /**
   * @dev register this contract to ERC1820 registry
   */
  constructor() {
    ERC1820_REGISTRY.setInterfaceImplementer(address(this), TOKENS_RECIPIENT_INTERFACE_HASH, address(this));
  }

  /**
   * @dev ERC677 hook. Token holder can send their xHOPR with `transferAndCall` to this contract.
   * @param _from address Address of tokens sender. It should be from the multisig
   * @param _value uint256 token amount being transferred
   */
  function onTokenTransfer(
    address _from,
    uint256 _value,
    // solhint-disable-next-line no-unused-vars
    bytes memory
  ) external returns (bool) {
    require(msg.sender == XHOPR_TOKEN, 'HoprWrapperProxy: Only accept xHOPR token');
    require(_from == XDAI_MULTISIG, 'HoprWrapperProxy: Only accept xHOPR from the Asso MultiSig');
    // forward it to the wrapper
    emit FowardedTo(WRAPPER, _value);
    IERC677(XHOPR_TOKEN).transferAndCall(WRAPPER, _value, '');
    return true;
  }

  /**
   * @dev token recipient hook for ERC777
   * @param from address token holder address.
   * @param to address recipient address
   * @param amount uint256 amount of tokens to transfer
   */
  function tokensReceived(
    address,
    address from,
    address to,
    uint256 amount,
    bytes calldata,
    bytes calldata
  ) external override {
    require(msg.sender == WXHOPR_TOKEN, 'HoprWrapperProxy: Only accept WXHOPR_TOKEN token');
    // require(from == WRAPPER, 'HoprWrapperProxy: Only accept WXHOPR token from the wrapper');
    require(to == address(this), 'HoprWrapperProxy: Must be sending tokens to this proxy');
    emit FowardedFrom(from, amount);
    IERC20(WXHOPR_TOKEN).safeTransfer(XDAI_MULTISIG, amount);
  }

  /**
   * @dev Recover any residual ERC20 token/ETH to the multisig
   * @param token ERC20 token address that is sent to this contract. Address 0 for ETH
   */
  function recoverTokens(address token) external {
    if (token == address(0)) {
      XDAI_MULTISIG.transfer(address(this).balance);
    } else {
      IERC20(token).safeTransfer(XDAI_MULTISIG, IERC20(token).balanceOf(address(this)));
    }
  }
}
