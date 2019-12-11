pragma solidity ^0.5.3;

import "@openzeppelin/contracts/drafts/ERC20Snapshot.sol";
import "@openzeppelin/contracts/token/ERC20/ERC20Mintable.sol";
import "@openzeppelin/contracts/token/ERC20/ERC20Burnable.sol";
import "@openzeppelin/contracts/token/ERC20/ERC20Detailed.sol";

contract HoprToken is ERC20Snapshot, ERC20Mintable, ERC20Burnable, ERC20Detailed {
    constructor () public
    ERC20Detailed("HOPR", "HOPR", 18) {
        _mint(msg.sender, 100000000);
    }
}