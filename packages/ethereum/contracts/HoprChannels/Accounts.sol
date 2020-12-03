// SPDX-License-Identifier: LGPL-3.0-only
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
     * @param pubKeyA first half of the public key
     * @param pubKeyB second half of the public key
     * @param secret account's secret
     */
    function _initializeAccount(
        address account,
        uint256 pubKeyA,
        uint256 pubKeyB,
        bytes32 secret
    ) internal {
        require(pubKeyA != uint256(0), "account pubKeyA must not be empty");
        require(pubKeyB != uint256(0), "account pubKeyB must not be empty");
        require(ECDSA.pubKeyToEthereumAddress(pubKeyA, pubKeyB) == account, "public key does not match caller");

        _updateAccountSecret(account, secret);

        emit AccountInitialized(account, pubKeyA, pubKeyB);
    }

    /**
     * @dev Updates account's secret and counter,
     * then emits {AccountSecretUpdated} event.
     * @param account the address of the account
     * @param secret account's secret
     */
    function _updateAccountSecret(
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
        uint256 pubKeyA,
        uint256 pubKeyB
    );

    event AccountSecretUpdated(
        // @TODO: remove this and rely on `msg.sender`
        address indexed account,
        bytes32 secret
    );
}