// SPDX-License-Identifier: GPL-3.0-or-later
pragma solidity 0.8.30;

import { Multicall } from "openzeppelin-contracts-5.4.0/utils/Multicall.sol";
import { HoprMultiSig } from "./MultiSig.sol";
import { HoprLedger } from "./Ledger.sol";
import { HoprNodeSafeRegistry } from "./node-stake/NodeSafeRegistry.sol";
import { INDEX_SNAPSHOT_INTERVAL } from "./Channels.sol";
import { MAX_KEY_ID, KeyId, EnumerableKeyBindingSet, KeyBindingSet, KeyBindingWithSignature } from "./utils/EnumerableKeyBindingSet.sol";

error ZeroAddress(string reason);
error EmptyMultiaddr();

/// forge-lint:disable-next-item(mixed-case-variable)
abstract contract HoprAnnouncementsEvents {
    event KeyBinding(bytes32 ed25519_sig_0, bytes32 ed25519_sig_1, bytes32 ed25519_pub_key, address chain_key);

    /**
     * A node is announce with a multiaddress base which a peer can use to
     * construct a full p2p multiaddress.
     * Examples:
     *   /dns4/ams-2.bootstrap.libp2p.io/tcp/443/wss
     *   /ip6/2604:1380:2000:7a00::1/tcp/4001
     *   /ip4/147.75.83.83/tcp/4001
     *   /ip6/2604:1380:2000:7a00::1/udp/4001/quic
     *   /ip4/147.75.83.83/udp/4001/quic
     *   /dns6/ams-2.bootstrap.libp2p.io/tcp/443/wss
     */
    event AddressAnnouncement(address node, string baseMultiaddr);

    event RevokeAnnouncement(address node);
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
 * Publishes transport-layer information in the hopr network.
 *
 * Relay nodes MUST bind their off-chain keys to their on-chain identity
 * and announce a base multiaddress to be publicly reachable.
 * Edge nodes MUST bind their off-chain keys to their on-chain identity.
 * and they MAY announce a base multiaddress to be publicly reachable.
 * 
 * A key id is a 4 byte unsigned integer, which is incremented on each new key binding.
 * The key id is used to retrieve the off-chain keys and the chain-key.
 * A unique key id is bound to a set of off-chain key and a chain-key (Ethereum address)
 * A node MAY bind multiple off-chain keys to the same chain-key.
 * A node MUST NOT bind the same off-chain keys to multiple chain-keys.
 * Key ids cannot be re-used or overwritten.
 * Key id 0 is reserved and MUST NOT be used.
 * The range of valid key ids is [1, 2^32 - 1].
 *
 * The chain-key is used to retrieve the multiaddress base of a node.
 * By knowing the key id of a peer, a node can retrieve the off-chain keys and then the multiaddress base.
 */
/// forge-lint:disable-next-item(mixed-case-variable)
contract HoprAnnouncements is Multicall, HoprMultiSig, HoprAnnouncementsEvents, HoprLedger(INDEX_SNAPSHOT_INTERVAL) {
    using EnumerableKeyBindingSet for KeyBindingSet;

    // key bindings
    KeyBindingSet internal _keyBindings;
    // announcements: chain-key => base-multiaddr
    mapping(address => string) public multiaddrOf;

    constructor(HoprNodeSafeRegistry safeRegistry) {
        if (address(safeRegistry) == address(0)) {
            revert ZeroAddress({ reason: "safeRegistry must not be empty" });
        }
        setNodeSafeRegistry(safeRegistry);
    }

    function bindKeysSafe(
        address selfAddress,
        bytes32 ed25519_sig_0,
        bytes32 ed25519_sig_1,
        bytes32 ed25519_pub_key
    )
        external
        HoprMultiSig.onlySafe(selfAddress)
    {
        _bindKeysInternal(selfAddress, ed25519_sig_0, ed25519_sig_1, ed25519_pub_key);
    }

    function bindKeys(
        bytes32 ed25519_sig_0,
        bytes32 ed25519_sig_1,
        bytes32 ed25519_pub_key
    )
        external
        HoprMultiSig.noSafeSet()
    {
        _bindKeysInternal(msg.sender, ed25519_sig_0, ed25519_sig_1, ed25519_pub_key);
    }

    function bindKeysAnnounceSafe(
        address selfAddress,
        bytes32 ed25519_sig_0,
        bytes32 ed25519_sig_1,
        bytes32 ed25519_pub_key,
        string calldata baseMultiaddr
    )
        external
        HoprMultiSig.onlySafe(selfAddress)
    {
        _bindKeysInternal(selfAddress, ed25519_sig_0, ed25519_sig_1, ed25519_pub_key);
        _announceInternal(selfAddress, baseMultiaddr);
    }

    /**
     * Convenience method to bind keys and announce in one call.
     */
    function bindKeysAnnounce(
        bytes32 ed25519_sig_0,
        bytes32 ed25519_sig_1,
        bytes32 ed25519_pub_key,
        string calldata baseMultiaddr
    )
        external
        HoprMultiSig.noSafeSet()
    {
        _bindKeysInternal(msg.sender, ed25519_sig_0, ed25519_sig_1, ed25519_pub_key);
        _announceInternal(msg.sender, baseMultiaddr);
    }

    function announceSafe(
        address selfAddress,
        string calldata baseMultiaddr
    )
        external
        HoprMultiSig.onlySafe(selfAddress)
    {
        _announceInternal(selfAddress, baseMultiaddr);
    }

    function announce(string calldata baseMultiaddr) external HoprMultiSig.noSafeSet() {
        _announceInternal(msg.sender, baseMultiaddr);
    }

    function revokeSafe(address selfAddress) external HoprMultiSig.onlySafe(selfAddress) {
        _revokeInternal(selfAddress);
    }

    function revoke() external HoprMultiSig.noSafeSet() {
        _revokeInternal(msg.sender);
    }

    // View functions for key bindings
    // // --- The following mappings are for easier lookups ---
    // // keybindings: key-id => { offchain keys + chain-key }
    // mapping(KeyId => KeyBinding) keyBindingOf;  // This is similar to _values
    // // reverse lookup: pubkey => key-id
    // mapping(bytes32 => KeyId) keyIdOf;

    /**
     * @dev Returns the range of valid key ids.
     */
    function getKeyIdRange() external pure returns (uint32 minKeyId, uint32 maxKeyId) {
        return (0, MAX_KEY_ID);
    }
    /**
     * @dev Returns the number of key bindings.
     */
    function getKeyBindingCount() external view returns (uint256) {
        return _keyBindings.length();
    }

    /**
     * @dev Returns the list of all key bindings.
     *      The key id can be derived from the index in the array (starting from 0, capped at MAX_KEY_ID).
     * Note: this function is gas expensive.
     */
    function getAllKeyBindings() external view returns (KeyBindingWithSignature[] memory) {
        return _keyBindings._values;
    }


    function isOffchainKeyBound(bytes32 ed25519_pub_key) external view returns (bool) {
        return _keyBindings.contains(ed25519_pub_key);
    }

    function tryGetKeyBinding(bytes32 ed25519_pub_key) external view returns (bool, KeyId, KeyBindingWithSignature memory) {
        (bool success, uint256 possibleKeyId, KeyBindingWithSignature memory keyBinding) = _keyBindings.tryGet(ed25519_pub_key);
        return (success, KeyId.wrap(uint32(possibleKeyId)), keyBinding);
    }

    function getKeyBindingWithKeyId(KeyId keyId) external view returns (KeyBindingWithSignature memory) {
        uint256 index = uint256(uint32(KeyId.unwrap(keyId)));
        return _keyBindings.at(index);
    }

    function getOffchainKeyWithKeyId(KeyId keyId) external view returns (bytes32 ed25519_pub_key) {
        uint256 index = uint256(uint32(KeyId.unwrap(keyId)));
        return _keyBindings.at(index).ed25519_pub_key;
    }

    function getKeyIdWithOffchainKey(bytes32 ed25519_pub_key) external view returns (bool, KeyId) {
        (bool success, uint256 possibleKeyId, ) = _keyBindings.tryGet(ed25519_pub_key);
        return (success, KeyId.wrap(uint32(possibleKeyId)));
    }

    /**
     * [mandatory] Registers a node within the Hopr network and cross-signs on-chain and off-chain keys.
     *
     * Creates a link between an Ethereum, the corresponding secp256k1 public key,
     * a ed25519 EdDSA public key. By submitting the transaction, the caller provides
     * a secp256k1 signature of the ed25519 public key. Conversely, the EdDSA signature
     * signs the secp256k1 public key.
     *
     * @dev The verification of the ed25519 EdDSA signature happens off-chain.
     *
     * @dev Key binding and address announcements can happen in one call using `multicall`.
     *
     * @param ed25519_sig_0 first component of the EdDSA signature
     * @param ed25519_sig_1 second component of the EdDSA signature
     * @param ed25519_pub_key EdDSA public key
     */
    function _bindKeysInternal(
        address selfAddress,
        bytes32 ed25519_sig_0,
        bytes32 ed25519_sig_1,
        bytes32 ed25519_pub_key
    )
        internal
    {
        _keyBindings.add(KeyBindingWithSignature(
            ed25519_sig_0,
            ed25519_sig_1,
            ed25519_pub_key,
            selfAddress
        ));
        indexEvent(
            abi.encodePacked(KeyBinding.selector, ed25519_sig_0, ed25519_sig_1, ed25519_pub_key, selfAddress)
        );
        emit KeyBinding(ed25519_sig_0, ed25519_sig_1, ed25519_pub_key, selfAddress);
    }

    /**
     * [optional] Announces a base mutliaddress for a node
     *
     * @dev Turns a node into a public relay node (PRN)
     *
     * @param baseMultiaddr base multiaddress of the node
     */
    function _announceInternal(address selfAddress, string calldata baseMultiaddr) internal {
        if (bytes(baseMultiaddr).length == 0) {
            revert EmptyMultiaddr();
        }
        multiaddrOf[selfAddress] = baseMultiaddr;
        indexEvent(abi.encodePacked(AddressAnnouncement.selector, selfAddress, baseMultiaddr));
        emit AddressAnnouncement(selfAddress, baseMultiaddr);
    }

    /**
     * Opts out from acting as a public relay node (PRN)
     */
    function _revokeInternal(address selfAddress) internal {
        delete multiaddrOf[selfAddress];
        indexEvent(abi.encodePacked(RevokeAnnouncement.selector, selfAddress));
        emit RevokeAnnouncement(selfAddress);
    }
}
