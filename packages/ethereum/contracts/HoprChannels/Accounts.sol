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
     * @param publicKeyA first half of the public key
     * @param publicKeyB second half of the public key
     * @param secret account's secret
     */
    function _initializeAccount(
        address account,
        uint256 publicKeyA,
        uint256 publicKeyB,
        bytes32 secret
    ) internal {
        require(publicKeyA != uint256(0), "account publicKeyA must not be empty");
        require(publicKeyB != uint256(0), "account publicKeyB must not be empty");
        require(ECDSA.pubKeyToEthereumAddress(publicKeyA, publicKeyB) == account, "public key does not match caller");

        _updateAccountSecret(account, secret);

        emit AccountInitialized(account, publicKeyA, publicKeyB);
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
        uint256 publicKeyA,
        uint256 publicKeyB
    );

    event AccountSecretUpdated(
        // @TODO: remove this and rely on `msg.sender`
        address indexed account,
        bytes32 secret
    );
}