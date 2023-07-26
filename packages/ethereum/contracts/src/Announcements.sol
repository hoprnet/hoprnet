// SPDX-License-Identifier: GPL-3.0-or-later
pragma solidity 0.8.19;

import 'openzeppelin-contracts/utils/Multicall.sol';

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
 **/
contract HoprAnnouncements is Multicall {
  event KeyBinding(bytes32 ed25519_sig_0, bytes32 ed25519_sig_1, bytes32 ed25519_pub_key, address chain_key);

  event AddressAnnouncement4(address node, bytes4 ip4, bytes2 port);
  event AddressAnnouncement6(address node, bytes16 ip6, bytes2 port);

  event RevokeAnnouncement(address node);

  modifier onlySafe() {
    // check if NodeSafeRegistry entry exists
    _;
  }

  modifier noSafeSet() {
    // check if NodeSafeRegistry entry **does not** exist
    _;
  }

  function bindKeysSafe(
    address self,
    bytes32 ed25519_sig_0,
    bytes32 ed25519_sig_1,
    bytes32 ed25519_pub_key
  ) external onlySafe {
    _bindKeysInternal(self, ed25519_sig_0, ed25519_sig_1, ed25519_pub_key);
  }

  function bindKeys(
    bytes32 ed25519_sig_0,
    bytes32 ed25519_sig_1,
    bytes32 ed25519_pub_key
  ) external noSafeSet {
    _bindKeysInternal(msg.sender, ed25519_sig_0, ed25519_sig_1, ed25519_pub_key);
  }

  function bindKeysAnnounce4Safe(
    address self,
    bytes32 ed25519_sig_0,
    bytes32 ed25519_sig_1,
    bytes32 ed25519_pub_key,
    bytes4 ip,
    bytes2 port
  ) external onlySafe {
    _bindKeysInternal(self, ed25519_sig_0, ed25519_sig_1, ed25519_pub_key);
    _announce4Internal(self, ip, port);
  }

  /**
   * Convenience method to bind keys and announce a IPv4 address in one call.
   */
  function bindKeysAnnounce4(
    bytes32 ed25519_sig_0,
    bytes32 ed25519_sig_1,
    bytes32 ed25519_pub_key,
    bytes4 ip,
    bytes2 port
  ) external noSafeSet {
    _bindKeysInternal(msg.sender,  ed25519_sig_0, ed25519_sig_1, ed25519_pub_key);
    _announce4Internal(msg.sender, ip, port);
  }

  function bindKeysAnnounce6Safe(
    address self,
    bytes32 ed25519_sig_0,
    bytes32 ed25519_sig_1,
    bytes32 ed25519_pub_key,
    bytes16 ip,
    bytes2 port
  ) external onlySafe {
    _bindKeysInternal(self, ed25519_sig_0, ed25519_sig_1, ed25519_pub_key);
    _announce6Internal(self, ip, port);
  }

  /**
   * Convenience method to bind keys and announce a IPv6 address in one call.
   */
  function bindKeysAnnounce6(
    bytes32 ed25519_sig_0,
    bytes32 ed25519_sig_1,
    bytes32 ed25519_pub_key,
    bytes16 ip,
    bytes2 port
  ) external noSafeSet {
    _bindKeysInternal(msg.sender, ed25519_sig_0, ed25519_sig_1, ed25519_pub_key);
    _announce6Internal(msg.sender, ip, port);
  }

  function announce6Safe(address self, bytes16 ip, bytes2 port) external onlySafe {
    _announce6Internal(self, ip, port);
  }

  function announce6(bytes16 ip, bytes2 port) external noSafeSet {
    _announce6Internal(msg.sender, ip, port);
  }

  function announce4Safe(address self, bytes4 ip, bytes2 port) external onlySafe {
    _announce4Internal(self, ip, port);
  }

  function announce4(bytes4 ip, bytes2 port) external noSafeSet {
    _announce4Internal(msg.sender, ip, port);
  }

  function revokeSafe(address self) external onlySafe {
    _revokeInternal(self);
  }

  function revoke() external noSafeSet {
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
  ) internal {
    emit KeyBinding(ed25519_sig_0, ed25519_sig_1, ed25519_pub_key, self);
  }

  /**
   * [optional] Announces a IPv4 address with port for the node
   *
   * @dev Turns a node into a public relay node (PRN)
   *
   * @param ip the IPv4 address to announce
   * @param port the port to use
   */
  function _announce4Internal(address self, bytes4 ip, bytes2 port) internal {
    emit AddressAnnouncement4(self, ip, port);
  }

  /**
   * [optional] Announces a IPv6 address with port for the node
   *
   * @dev Turns a node into a public relay node (PRN)
   *
   * @param ip the IPv6 address to announce
   * @param port the port to use
   */
  function _announce6Internal(address self, bytes16 ip, bytes2 port) internal {
    emit AddressAnnouncement6(self, ip, port);
  }

  /**
   * Opts out from acting as a public relay node (PRN)
   */
  function _revokeInternal(address self) internal {
    emit RevokeAnnouncement(self);
  }
}
