// SPDX-License-Identifier: GPL-3.0-or-later
pragma solidity ^0.8.19;

import '@openzeppelin/contracts/utils/Multicall.sol';

contract HoprAnnouncements is Multicall {
  event KeyBindingOdd(bytes32 secp256k1_x, bytes32 ed25519_sig_0, bytes32 ed25519_sig_1, bytes32 ed25519_pub_key);
  event KeyBindingEven(bytes32 secp256k1_x, bytes32 ed25519_sig_0, bytes32 ed25519_sig_1, bytes32 ed25519_pub_key);

  event AddressAnnouncement4(address node, bytes4 ip4, bytes2 port);
  event AddressAnnouncement6(address node, bytes16 ip6, bytes2 port);

  event RevokeAnnouncement(address node);

  error PublicKeyDoesNotMatchSender(address pubkey, address sender);

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
   * @param secp256k1_x first component of the public key
   * @param secp256k1_y second component of the public key
   * @param ed25519_sig_0 first component of the EdDSA signature
   * @param ed25519_sig_1 second component of the EdDSA signature
   * @param ed25519_pub_key EdDSA public key
   */
  function bindKeys(
    bytes32 secp256k1_x,
    bytes32 secp256k1_y,
    bytes32 ed25519_sig_0,
    bytes32 ed25519_sig_1,
    bytes32 ed25519_pub_key
  ) external {
    address sender_addr = address(uint160(uint256(keccak256(abi.encodePacked(secp256k1_x, secp256k1_y)))));

    if (msg.sender != sender_addr) {
      revert PublicKeyDoesNotMatchSender({pubkey: sender_addr, sender: msg.sender});
    }

    if (uint256(secp256k1_y) % 2 == 1) {
      emit KeyBindingOdd(secp256k1_x, ed25519_sig_0, ed25519_sig_1, ed25519_pub_key);
    } else {
      emit KeyBindingEven(secp256k1_x, ed25519_sig_0, ed25519_sig_1, ed25519_pub_key);
    }
  }

  /**
   * [optional] Announces a IPv4 address with port for the node
   *
   * @dev Turns a node into a public relay node (PRN)
   *
   * @param ip the IPv4 address to announce
   * @param port the port to use
   */
  function announce4(bytes4 ip, bytes2 port) external {
    emit AddressAnnouncement4(msg.sender, ip, port);
  }

  /**
   * [optional] Announces a IPv6 address with port for the node
   *
   * @dev Turns a node into a public relay node (PRN)
   *
   * @param ip the IPv6 address to announce
   * @param port the port to use
   */
  function announce6(bytes16 ip, bytes2 port) external {
    emit AddressAnnouncement6(msg.sender, ip, port);
  }

  /**
   * Opts out from acting as a public relay node (PRN)
   */
  function revoke() external {
    emit RevokeAnnouncement(msg.sender);
  }

  /**
   * Convenience method to bind keys and announce a IPv4 address in one call.
   */
  function bindKeysAnnounce4(
    bytes32 secp256k1_x,
    bytes32 secp256k1_y,
    bytes32 ed25519_sig_0,
    bytes32 ed25519_sig_1,
    bytes32 ed25519_pub_key,
    bytes4 ip,
    bytes2 port
  ) external {
    (bool successBind, ) = address(this).delegatecall(
      abi.encodeCall(
        HoprAnnouncements.bindKeys,
        (secp256k1_x, secp256k1_y, ed25519_sig_0, ed25519_sig_1, ed25519_pub_key)
      )
    );
    if (successBind == false) {
      revert();
    }

    (bool successAnnounce4, ) = address(this).delegatecall(abi.encodeCall(HoprAnnouncements.announce4, (ip, port)));

    if (successAnnounce4 == false) {
      revert();
    }
  }

  /**
   * Convenience method to bind keys and announce a IPv6 address in one call.
   */
  function bindKeysAnnounce6(
    bytes32 secp256k1_x,
    bytes32 secp256k1_y,
    bytes32 ed25519_sig_0,
    bytes32 ed25519_sig_1,
    bytes32 ed25519_pub_key,
    bytes16 ip,
    bytes2 port
  ) external {
    (bool successBind, ) = address(this).delegatecall(
      abi.encodeCall(
        HoprAnnouncements.bindKeys,
        (secp256k1_x, secp256k1_y, ed25519_sig_0, ed25519_sig_1, ed25519_pub_key)
      )
    );
    if (successBind == false) {
      revert();
    }

    (bool successAnnounce6, ) = address(this).delegatecall(abi.encodeCall(HoprAnnouncements.announce6, (ip, port)));
    if (successAnnounce6 == false) {
      revert();
    }
  }
}
