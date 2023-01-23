// SPDX-License-Identifier: GPL-3.0-only

pragma solidity ^0.8.0;

import '@openzeppelin/contracts/token/ERC20/IERC20.sol';
import '@openzeppelin/contracts/token/ERC20/utils/SafeERC20.sol';
import '@openzeppelin/contracts/token/ERC721/IERC721.sol';
import '@openzeppelin/contracts/security/ReentrancyGuard.sol';
import '@openzeppelin/contracts/access/AccessControlEnumerable.sol';
import '@openzeppelin/contracts/token/ERC721/extensions/ERC721URIStorage.sol';
import '@openzeppelin/contracts/token/ERC721/extensions/ERC721Enumerable.sol';
import './IHoprBoost.sol';
import '../utils/EnumerableStringSet.sol';

/**
 *
 */
contract HoprBoost is IHoprBoost, AccessControlEnumerable, ERC721URIStorage, ERC721Enumerable, ReentrancyGuard {
  using SafeERC20 for IERC20;
  using EnumerableStringSet for EnumerableStringSet.StringSet;

  bytes32 public constant MINTER_ROLE = keccak256('MINTER_ROLE');

  string private _baseTokenURI;
  EnumerableStringSet.StringSet private _boostType;
  mapping(uint256 => uint256) private _boostNumerator; // tokenId => boost factor numerator
  mapping(uint256 => uint256) private _redeemDeadline; // tokenId => deadline for redeeming a boost
  mapping(uint256 => uint256) private _boostTypeIndexOfId; // tokenId => boost type index

  event BoostMinted(uint256 indexed boostTypeIndex, uint256 indexed boostNumerator, uint256 indexed redeemDeadline);

  /**
   * @dev Set a new admin role. Set the new admin as a minter. Provide name and symbol for ERC721.
   * Update the ``_baseTokenURI``.
   * @param newAdmin address Address of the new admin account
   */
  constructor(address newAdmin, string memory baseTokenURI) ERC721('HOPR Boost NFT', 'HOPR Boost') {
    _baseTokenURI = baseTokenURI;
    _setupRole(DEFAULT_ADMIN_ROLE, newAdmin);
    _setupRole(MINTER_ROLE, newAdmin);
  }

  /**
   * @dev Returns the boost factor and the redeem deadline associated with ``tokenId``.
   * @param tokenId uint256 token Id of the boost.
   */
  function boostOf(uint256 tokenId) external view override returns (uint256, uint256) {
    return (_boostNumerator[tokenId], _redeemDeadline[tokenId]);
  }

  /**
   * @dev Returns the boost type index associated with ``tokenId``.
   * @param tokenId uint256 token Id of the boost.
   */
  function typeIndexOf(uint256 tokenId) external view override returns (uint256) {
    return _boostTypeIndexOfId[tokenId];
  }

  /**
   * @dev Returns the boost type associated with ``tokenId``.
   * @param tokenId uint256 token Id of the boost.
   */
  function typeOf(uint256 tokenId) external view override returns (string memory) {
    return _boostType.at(_boostTypeIndexOfId[tokenId]);
  }

  /**
   * @dev Returns the boost type name associated with ``typeIndex``.
   * @param typeIndex uint256 Index of the type.
   */
  function typeAt(uint256 typeIndex) external view returns (string memory) {
    return _boostType.at(typeIndex);
  }

  /**
   * @dev Admin can update the new base URI at any time.
   * @param baseTokenURI string Basic URI of the token.
   */
  function updateBaseURI(string memory baseTokenURI) external onlyRole(DEFAULT_ADMIN_ROLE) {
    _baseTokenURI = baseTokenURI;
  }

  /**
   * @dev Minter creates a new token for `to`. Its token ID will be automatically
   * assigned based on current `totalSupply`. (and available on the emitted {IERC721-Transfer} event).
   * Provide boost factor, boost type and boost rank. If needed, register a new class of boost,
   * with its "type", "rank" and "boost factor (numerator)"
   * The token URI is generated based on the base URI, boostType and boostRank.
   * @param to address Address of token holder that will receive the NFT token of the given "type"
   * @param boostType string Type of the boost
   * @param boostRank string Rank of the boost
   * @param boostNumerator uint256 Numerator of the boost factor. Its denominator is 1e12.
   * @param redeemDeadline uint256 Deadline for redeem a boost. Epoch Unix Timestamp.
   */
  function mint(
    address to,
    string memory boostType,
    string memory boostRank,
    uint256 boostNumerator,
    uint256 redeemDeadline
  ) external onlyRole(MINTER_ROLE) {
    // register boost type in the smart contract
    _boostType.add(boostType);
    uint256 boostTypeIndex = _boostType.indexOf(boostType);
    string memory _tokenURI = string(abi.encodePacked(boostType, '/', boostRank));

    _mintBoost(to, boostNumerator, redeemDeadline, boostTypeIndex, _tokenURI);

    emit BoostMinted(boostTypeIndex, boostNumerator, redeemDeadline);
  }

  /**
   * @dev Minter creates new tokens for `to` array. Its token ID will be automatically
   * assigned based on current `totalSupply`. (and available on the emitted {IERC721-Transfer} event).
   * Provide boost factor, boost type and boost rank. If needed, register a new class of boost,
   * with its "type", "rank" and "boost factor (numerator)"
   * The token URI is generated based on the base URI, boostType and boostRank.
   * @param to address[] Addresses of token holder that will receive the NFT token of the given "type"
   * @param boostType string Type of the boost
   * @param boostRank string Rank of the boost
   * @param boostNumerator uint256 Numerator of the boost factor. Its denominator is 1e12.
   * @param redeemDeadline uint256 Deadline for redeem a boost. Epoch Unix Timestamp
   */
  function batchMint(
    address[] calldata to,
    string calldata boostType,
    string calldata boostRank,
    uint256 boostNumerator,
    uint256 redeemDeadline
  ) external onlyRole(MINTER_ROLE) {
    // register boost type in the smart contract
    _boostType.add(boostType);
    uint256 boostTypeIndex = _boostType.indexOf(boostType);
    string memory _tokenURI = string(abi.encodePacked(boostType, '/', boostRank));

    for (uint256 index = 0; index < to.length; index++) {
      _mintBoost(to[index], boostNumerator, redeemDeadline, boostTypeIndex, _tokenURI);
    }

    emit BoostMinted(boostTypeIndex, boostNumerator, redeemDeadline);
  }

  /**
   * @dev Reclaim any ERC20 token being accidentally sent to the contract.
   * @param tokenAddress address ERC20 token address.
   */
  function reclaimErc20Tokens(address tokenAddress) external onlyRole(DEFAULT_ADMIN_ROLE) nonReentrant {
    IERC20(tokenAddress).safeTransfer(_msgSender(), IERC20(tokenAddress).balanceOf(address(this)));
  }

  /**
   * @dev Reclaim any ERC721 token being accidentally sent to the contract.
   * @param tokenAddress address ERC721 token address.
   */
  function reclaimErc721Tokens(address tokenAddress, uint256 tokenId)
    external
    onlyRole(DEFAULT_ADMIN_ROLE)
    nonReentrant
  {
    IERC721(tokenAddress).transferFrom(address(this), _msgSender(), tokenId);
  }

  /**
   * @dev See {IERC165-supportsInterface}.
   */
  function supportsInterface(bytes4 interfaceId)
    public
    view
    virtual
    override(IERC165, AccessControlEnumerable, ERC721, ERC721Enumerable)
    returns (bool)
  {
    return interfaceId == type(IHoprBoost).interfaceId || super.supportsInterface(interfaceId);
  }

  /**
   * @dev Inherit tokenURI from {ERC721} and {ERC721URIStorage}
   */
  function tokenURI(uint256 tokenId)
    public
    view
    override(IERC721Metadata, ERC721, ERC721URIStorage)
    returns (string memory)
  {
    return super.tokenURI(tokenId);
  }

  /**
   * @dev Inherit _baseURI from {ERC721} and {ERC721URIStorage}
   */
  function _baseURI() internal view override returns (string memory) {
    return _baseTokenURI;
  }

  /**
   * @dev Inherit _baseURI from {ERC721} and {ERC721Enumerable}
   */
  function _beforeTokenTransfer(
    address from,
    address to,
    uint256 tokenId
  ) internal virtual override(ERC721, ERC721Enumerable) {
    super._beforeTokenTransfer(from, to, tokenId);
  }

  /**
   * @dev Cannot burn the NFT token
   */
  function _burn(uint256 tokenId) internal virtual override(ERC721, ERC721URIStorage) {}

  /**
   * @dev Mint token. Generate token URI and save its URI.
   * @param to address Address of token holder
   * @param boostNumerator uint256 Numerator of the boost factor. Its denominator is 1e12.
   * @param redeemDeadline uint256 Deadline for redeem a boost. Epoch Unix Timestamp
   * @param boostTypeIndex uint256 Index of the boost type.
   * @param _tokenURI string URI of the boost.
   */
  function _mintBoost(
    address to,
    uint256 boostNumerator,
    uint256 redeemDeadline,
    uint256 boostTypeIndex,
    string memory _tokenURI
  ) private {
    // create token
    uint256 tokenId = totalSupply();
    _mint(to, tokenId);
    // save boost factor numerator
    _boostNumerator[tokenId] = boostNumerator;
    // save redeem deadline
    _redeemDeadline[tokenId] = redeemDeadline;
    // save boost type id
    _boostTypeIndexOfId[tokenId] = boostTypeIndex;
    // save tokenURI
    _setTokenURI(tokenId, _tokenURI);
  }
}
