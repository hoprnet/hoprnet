// SPDX-License-Identifier: GPL-3.0
pragma solidity ^0.6.0;

import "../utils/ECDSA.sol";

contract Accounts {
    /**
     * @dev An account struct
     */
    struct Account {
        bytes32 secret; // account's hashed secret
        uint256 counter; // increases everytime 'secret' is changed
    }

    /**
     * @dev Stored accounts keyed by their address
     */
    mapping(address => Account) public accounts;

    /**
     * @dev Initializes an account,
     * stores it's public key, secret and counter,
     * then emits {AccountInitialized} and {AccountSecretUpdated} events.
     * @param account the address of the account
     * @param pubKeyFirstHalf first half of the public key
     * @param pubKeySecondHalf second half of the public key
     * @param secret account's secret
     */
    function _initializeAccount(
        address account,
        uint256 pubKeyFirstHalf,
        uint256 pubKeySecondHalf,
        bytes32 secret
    ) internal {
        require(account != address(0), "account must not be empty");
        require(pubKeyFirstHalf != uint256(0), "pubKeyFirstHalf must not be empty");
        require(pubKeySecondHalf != uint256(0), "pubKeySecondHalf must not be empty");
        // require(secret != bytes32(0), "secret must not be empty");

        require(
            ECDSA.pubKeyToEthereumAddress(pubKeyFirstHalf, pubKeySecondHalf) == account,
            "public key does not match account"
        );

        _updateAccount(account, secret);

        emit AccountInitialized(account, pubKeyFirstHalf, pubKeySecondHalf);
    }

    /**
     * @dev Updates account's secret and counter,
     * then emits {AccountSecretUpdated} event.
     * @param account the address of the account
     * @param secret account's secret
     */
    function _updateAccount(
        address account,
        bytes32 secret
    ) internal {
        require(secret != bytes32(0), "secret must not be empty");

        Account storage accountData = accounts[account];
        require(secret != accountData.secret, "secret must not be the same as before");

        accountData.secret = secret;
        accountData.counter += 1;

        emit AccountSecretUpdated(account, secret);
    }

    event AccountInitialized(
        // @TODO: remove this and rely on `msg.sender`
        address indexed account,
        uint256 pubKeyFirstHalf,
        uint256 pubKeySecondHalf
    );

    event AccountSecretUpdated(
        // @TODO: remove this and rely on `msg.sender`
        address indexed account,
        bytes32 secret
    );
}