// SPDX-License-Identifier: GPL-3.0-or-later
pragma solidity 0.8.19;

import "openzeppelin-contracts/utils/Multicall.sol";

import { HoprMultiSig } from "./MultiSig.sol";
import { HoprNodeSafeRegistry } from "./node-stake/NodeSafeRegistry.sol";

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
 */
contract HoprAnnouncements is Multicall, HoprMultiSig, HoprAnnouncementsEvents {
    constructor(HoprNodeSafeRegistry safeRegistry) {
        setNodeSafeRegistry(safeRegistry);
    }

    function bindKeysSafe(
        address self,
        bytes32 ed25519_sig_0,
        bytes32 ed25519_sig_1,
        bytes32 ed25519_pub_key
    )
        external
        HoprMultiSig.onlySafe(self)
    {
        _bindKeysInternal(self, ed25519_sig_0, ed25519_sig_1, ed25519_pub_key);
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
        address self,
        bytes32 ed25519_sig_0,
        bytes32 ed25519_sig_1,
        bytes32 ed25519_pub_key,
        string calldata baseMultiaddr
    )
        external
        HoprMultiSig.onlySafe(self)
    {
        _bindKeysInternal(self, ed25519_sig_0, ed25519_sig_1, ed25519_pub_key);
        _announceInternal(self, baseMultiaddr);
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

    function announceSafe(address self, string calldata baseMultiaddr) external HoprMultiSig.onlySafe(self) {
        _announceInternal(self, baseMultiaddr);
    }

    function announce(string calldata baseMultiaddr) external HoprMultiSig.noSafeSet() {
        _announceInternal(msg.sender, baseMultiaddr);
    }

    function revokeSafe(address self) external HoprMultiSig.onlySafe(self) {
        _revokeInternal(self);
    }

    function revoke() external HoprMultiSig.noSafeSet() {
        _revokeInternal(msg.sender);
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
        address self,
        bytes32 ed25519_sig_0,
        bytes32 ed25519_sig_1,
        bytes32 ed25519_pub_key
    )
        internal
    {
        emit KeyBinding(ed25519_sig_0, ed25519_sig_1, ed25519_pub_key, self);
    }

    /**
     * [optional] Announces a base mutliaddress for a node
     *
     * @dev Turns a node into a public relay node (PRN)
     *
     * @param baseMultiaddr base multiaddress of the node
     */
    function _announceInternal(address self, string calldata baseMultiaddr) internal {
        emit AddressAnnouncement(self, baseMultiaddr);
    }

    /**
     * Opts out from acting as a public relay node (PRN)
     */
    function _revokeInternal(address self) internal {
        emit RevokeAnnouncement(self);
    }
}
