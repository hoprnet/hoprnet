pragma solidity ^0.5.0;

import "../../ERC20Token.sol";


contract ERC20SnapshotMock is ERC20Token {
    constructor(address initialAccount, uint256 initialBalance) public {
        _mint(initialAccount, initialBalance);
    }

    function mint(address account, uint256 amount) public {
        _mint(account, amount);
    }

    function burn(address account, uint256 amount) public {
        _burn(account, amount);
    }
}
