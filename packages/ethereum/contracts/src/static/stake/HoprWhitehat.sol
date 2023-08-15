// SPDX-License-Identifier: GPL-3.0-only

pragma solidity ^0.8.0;

import 'openzeppelin-contracts-4.4.2/access/Ownable.sol';
import 'openzeppelin-contracts-4.4.2/token/ERC777/IERC777Recipient.sol';
import 'openzeppelin-contracts-4.4.2/token/ERC721/IERC721Receiver.sol';
import 'openzeppelin-contracts-4.4.2/utils/introspection/ERC1820Implementer.sol';
import 'openzeppelin-contracts-4.4.2/security/ReentrancyGuard.sol';
import './HoprStake.sol';
import './HoprBoost.sol';
import './mocks/ERC677Mock.sol';
import './mocks/ERC777Mock.sol';

/*
  CHECKLIST:

  STEP -1. flatten contract
  STEP 0. deploy HoprWhitehat
  STEP 1. transfer ownership of HoprStake to this newOwnerContract
  STEP 2. find user with minimal amount of locked tokens for testing purposes
  STEP 3. obtain the amount of rewards which that user is entitled to
  STEP 4. fund newOwnerContract with corresponding amount of wxHOPR
  STEP 5. user needs to follow procedure A

  PROCEDURE PARTICIPANTS:

  W - HoprWhitehat contract
  H - HoprStake contract
  S - account which has stake locked in H
  C - account calling the gimmeToken/0 function
  O - account which is owner of W

  PROCEDURE A (2 manual steps):

  1. S calls contract function `prepare` of W
  2. S calls contract function `gimmeToken` of W
  3. [W-gimmeToken] sends wxHopr to H
  4. [W-gimmeToken] calls `unlock` of H
  5. [W-gimmeToken -> H-unlock-_claim] performs `safeTransfer` of wxHopr to S
  6. [W-gimmeToken -> H-unlock-_claim -> S-W_tokensReceived] calls `reclaimErc20Tokens` of H
  7. [W-gimmeToken -> H-unlock-_claim -> S-W_tokensReceived -> H-reclaimErc20Tokens] performs `safeTransfer` of xHopr to H
  8. [W-gimmeToken -> H-unlock] transfers redeemed nfts
  8. DONE

  PROCEDURE B (1 manual step):

  1. S calls contract function `prepare` of W
  2. O calls contract function `gimmeToken` of W with S as parameter
  3. [W-gimmeToken] sends wxHopr to H
  4. [W-gimmeToken] calls `unlock` of H
  5. [W-gimmeToken -> H-unlock-_claim] performs `safeTransfer` of wxHopr to S
  6. [W-gimmeToken -> H-unlock-_claim -> S-W_tokensReceived] calls `reclaimErc20Tokens` of H
  7. [W-gimmeToken -> H-unlock-_claim -> S-W_tokensReceived -> H-reclaimErc20Tokens] performs `safeTransfer` of xHopr to H
  8. [W-gimmeToken -> H-unlock] transfers redeemed nfts
  8. DONE
*/

contract HoprWhitehat is Ownable, IERC777Recipient, IERC721Receiver, ERC1820Implementer, ReentrancyGuard {
  using SafeERC20 for IERC20;

  // utility variable used to refer to the caller
  address public currentCaller;
  // determine if function calls are processed
  bool public isActive;
  // rescued xhopr amount
  uint256 public rescuedXHoprAmount;

  // instantiated references to the contracts used in Stake Season 1
  HoprBoost public myHoprBoost = HoprBoost(0x43d13D7B83607F14335cF2cB75E87dA369D056c7);
  HoprStake public myHoprStake = HoprStake(0x912F4d6607160256787a2AD40dA098Ac2aFE57AC);
  ERC777Mock public wxHopr = ERC777Mock(0xD4fdec44DB9D44B8f2b6d529620f9C0C7066A2c1);
  ERC677Mock public xHopr = ERC677Mock(0xD057604A14982FE8D88c5fC25Aac3267eA142a08);

  IERC1820Registry private constant ERC1820_REGISTRY = IERC1820Registry(0x1820a4B7618BdE71Dce8cdc73aAB6C95905faD24);
  bytes32 private constant TOKENS_RECIPIENT_INTERFACE_HASH = keccak256('ERC777TokensRecipient');
  bytes32 private constant ERC1820_ACCEPT_MAGIC = keccak256('ERC1820_ACCEPT_MAGIC');

  event RequestedGimme(address indexed account, uint256 indexed entitledReward);
  event Called777Hook(address indexed contractAddress, address indexed from, uint256 indexed amount);
  event Called777HookForFunding(address indexed contractAddress, address indexed from, uint256 indexed amount);
  event Received677(address indexed contractAddress, address indexed from, uint256 indexed amount);
  event ReclaimedBoost(address indexed account, uint256 indexed tokenId);

  /**
   * @dev Provide NFT contract address. Transfer owner role to the new owner address.
   * At deployment, it also registers the lock contract as an ERC777 recipient.
   * @param _newOwner address Address of the new owner. This new owner can reclaim any ERC20 and ERC721 token being accidentally sent to the lock contract.
   * @param _myHoprBoost address Address of the mock boost contract.
   * @param _myHoprStake address Address of the mock stake contract.
   * @param _xHopr address Address of the mock xHopr contract.
   * @param _wxHopr address Address of the mock wxHopr contract.
   */
  constructor(
    address _newOwner,
    address _myHoprBoost,
    address _myHoprStake,
    address _xHopr,
    address _wxHopr
  ) {
    // implement in favor of testing
    uint256 chainId;
    assembly {
      chainId := chainid()
    }
    if (chainId != 100) {
      myHoprBoost = HoprBoost(_myHoprBoost);
      myHoprStake = HoprStake(_myHoprStake);
      xHopr = ERC677Mock(_xHopr);
      wxHopr = ERC777Mock(_wxHopr);
    }
    // keep deactivated at creation, requires manual activation by owner
    isActive = false;
    ERC1820_REGISTRY.setInterfaceImplementer(address(this), TOKENS_RECIPIENT_INTERFACE_HASH, address(this));
    transferOwnership(_newOwner);
  }

  /**
   * override implementation check
   */
  function canImplementInterfaceForAddress(bytes32 interfaceHash, address)
    public
    view
    virtual
    override
    returns (bytes32)
  {
    return interfaceHash == TOKENS_RECIPIENT_INTERFACE_HASH ? ERC1820_ACCEPT_MAGIC : bytes32(0x00);
  }

  // entry function to be called by users who can unlock their tokens (users who have rewards)
  function gimmeToken() external nonReentrant {
    require(isActive, 'Whitehat is not active');
    // ensure STEP 1
    require(myHoprStake.owner() == address(this), 'HoprStake needs to transfer ownership');
    // ensure STEP 2
    require(
      ERC1820_REGISTRY.getInterfaceImplementer(msg.sender, TOKENS_RECIPIENT_INTERFACE_HASH) == address(this),
      'Caller has to set this contract as ERC1820 interface'
    );

    // store caller to be used throughout the call
    currentCaller = msg.sender;
    // updates the rewards inside the accounts mapping struct
    myHoprStake.sync(currentCaller);

    (
      ,
      ,
      ,
      uint256 cumulatedRewards,
      uint256 claimedRewards
    ) = myHoprStake.accounts(currentCaller);
    uint256 stakerEntitledReward = cumulatedRewards - claimedRewards;
    emit RequestedGimme(currentCaller, stakerEntitledReward);

    // send rewards to HoprStake to make sure claim within unlock works
    wxHopr.send(address(myHoprStake), stakerEntitledReward, '0x0');
    // unlock xHOPR
    myHoprStake.unlock(currentCaller);
  }

  // entry function to be called by users who can unlock their tokens (users who have rewards)
  function gimmeTokenFor(address staker) external onlyOwner {
    // contract must be the recipient of
    require(myHoprStake.owner() == address(this), 'HoprStake needs to transfer ownership');
    // check 1820 implementation
    require(
      ERC1820_REGISTRY.getInterfaceImplementer(staker, TOKENS_RECIPIENT_INTERFACE_HASH) == address(this),
      'Caller has to set this contract as ERC1820 interface'
    );
    // store the caller for other hook functions
    currentCaller = staker;
    // update caller's account (claimable rewards)
    myHoprStake.sync(currentCaller); // updates the rewards inside the accounts mapping struct
    // solhint-disable-next-line no-unused-vars
    (
      ,
      ,
      ,
      uint256 cumulatedRewards,
      uint256 claimedRewards
    ) = myHoprStake.accounts(currentCaller);
    uint256 stakerEntitledReward = cumulatedRewards - claimedRewards;
    emit RequestedGimme(currentCaller, stakerEntitledReward);
    // fund reward to Stake contract
    wxHopr.send(address(myHoprStake), stakerEntitledReward, '0x0');
    // unlock xHOPR
    myHoprStake.unlock(currentCaller);
  }

  // ERC777 fallback (wxHOPR aka reward tokens)
  function tokensReceived(
    address,
    address from,
    address to,
    uint256 amount,
    bytes calldata,
    bytes calldata
  ) external override {
    if (isActive) {
      require(msg.sender == address(wxHopr), 'can only be called from wxHOPR');
      if (from == address(myHoprStake)) {
        require(to == currentCaller, 'must send ERC777 tokens to the caller of gimmeToken');
        emit Called777Hook(msg.sender, from, amount);
        // controlled-reentrancy starts here
        if (xHopr.balanceOf(address(myHoprStake)) > myHoprStake.totalLocked()) {
          myHoprStake.reclaimErc20Tokens(address(xHopr));
        }
      } else {
        emit Called777HookForFunding(msg.sender, from, amount);
      }
    }
  }

  /**
   * @dev ERC677 fallback (xHOPR aka token that users stake and unlock)
   * No need to reclaim NFTs for caller upon receiving xHOPR
   */
  function onTokenTransfer(
    address _from,
    uint256 _value,
    bytes memory
  ) external returns (bool) {
    if (msg.sender == address(xHopr)) {
      rescuedXHoprAmount += _value;
    }
    emit Received677(msg.sender, _from, _value);
    return true;
  }

  /**
   * ERC721 hook. Allow contract to receive 721
   */
  function onERC721Received(
    address,
    address,
    uint256,
    bytes calldata
  ) view external override returns (bytes4) {
    return IERC721Receiver(address(this)).onERC721Received.selector;
  }

  /**
   * @dev Transfer the ownership of HoprStake contract back to multisig
   */
  function transferBackOwnership(address multisig) external onlyOwner {
    myHoprStake.transferOwnership(multisig);
  }

  /**
   * @dev rescue all the NFTs of a locked staker account
   * Forward it to the original owner.
   */
  function ownerRescueBoosterNft(address stakerAddress, uint256 tokenId) external onlyOwner {
    myHoprStake.reclaimErc721Tokens(address(myHoprBoost), tokenId);
    // reclaim erc721 of the lockedAddress
    emit ReclaimedBoost(stakerAddress, tokenId);
    // forward the 721 to the original staker
    myHoprBoost.safeTransferFrom(address(this), stakerAddress, tokenId);
  }

  /**
   * @dev rescue all the NFTs of a locked staker account
   * Forward it to the original owner.
   */
  function ownerRescueBoosterNftInBatch(address stakerAddress) external onlyOwner {
    uint256 numBoost = myHoprStake.redeemedNftIndex(stakerAddress);
    for (uint256 index = 0; index < numBoost; index++) {
      uint256 tokenId = myHoprStake.redeemedNft(stakerAddress, index);
      myHoprStake.reclaimErc721Tokens(address(myHoprBoost), tokenId);
      // reclaim erc721 of the lockedAddress
      emit ReclaimedBoost(stakerAddress, tokenId);
      // forward the 721 to the original staker
      myHoprBoost.safeTransferFrom(address(this), stakerAddress, tokenId);
    }
  }

  /**
   * @dev Reclaim any ERC20 token being accidentally sent to the contract.
   * @param tokenAddress address ERC20 token address.
   */
  function reclaimErc20Tokens(address tokenAddress) external onlyOwner {
    uint256 difference = IERC20(tokenAddress).balanceOf(address(this));
    IERC20(tokenAddress).safeTransfer(owner(), difference);
  }

  /**
   * @dev Reclaim any ERC721 token being accidentally sent to the contract.
   * @param tokenAddress address ERC721 token address.
   * @param tokenId id of the ERC721 token.
   */
  function reclaimErc721Tokens(address tokenAddress, uint256 tokenId) external onlyOwner {
    IHoprBoost(tokenAddress).transferFrom(address(this), owner(), tokenId);
  }

  /**
   * @dev Activate all contract functions.
   */
  function activate() public onlyOwner {
    require(!isActive, 'HoprWhitehat is already active');
    isActive = true;
  }

  /**
   * @dev Deactivate all contract functions.
   */
  function deactivate() public onlyOwner {
    require(isActive, 'HoprWhitehat is already not active');
    isActive = false;
  }
}
