// SPDX-License-Identifier: GPL-3.0-only

pragma solidity ^0.8.0;

import 'openzeppelin-contracts-4.4.2/token/ERC20/IERC20.sol';
import 'openzeppelin-contracts-4.4.2/token/ERC20/utils/SafeERC20.sol';
import 'openzeppelin-contracts-4.4.2/token/ERC777/IERC777Recipient.sol';
import 'openzeppelin-contracts-4.4.2/token/ERC721/IERC721Receiver.sol';
import 'openzeppelin-contracts-4.4.2/security/ReentrancyGuard.sol';
import 'openzeppelin-contracts-4.4.2/access/Ownable.sol';
import 'openzeppelin-contracts-4.4.2/utils/introspection/IERC1820Registry.sol';
import 'openzeppelin-contracts-4.4.2/utils/math/Math.sol';
import './IHoprBoost.sol';

/**
 *
 */
contract HoprStake is Ownable, IERC777Recipient, IERC721Receiver, ReentrancyGuard {
  using SafeERC20 for IERC20;
  using Math for uint256;

  struct Account {
    uint256 actualLockedTokenAmount; // The amount of LOCK_TOKEN being actually locked to the contract.
    // Those tokens can be withdrawn after “UNLOCK_START”
    uint256 virtualLockedTokenAmount; // The amount of LOCK_TOKEN token being virtually locked to the contract.
    // This field is only relevant to seed investors. Those tokens cannot be withdrawn after “UNLOCK_START”.
    uint256 lastSyncTimestamp; // Timestamp at which any “Account” attribute gets synced for the last time.
    uint256 cumulatedRewards; // Rewards accredited to the account at “lastSyncTimestamp”.
    uint256 claimedRewards; // Rewards claimed by the account.
  }

  uint256 public constant BASIC_START = 1627387200; // Block timestamp at which incentive program starts for accounts that stake real LOCK_TOKEN. Default value is 1627387200 (July 27th 2021 14:00 CET).
  uint256 public constant SEED_START = 1630065600; // Block timestamp at which incentive program starts for seed investors that promise to stake their unreleased tokens. Default value is 1630065600 (August 27th 2021 14:00 CET).
  uint256 public constant PROGRAM_END = 1642424400; // Block timestamp at which incentive program ends. From this timestamp on, tokens can be unlocked. Default value is 1642424400 (Jan 17th 2022 14:00 CET).
  uint256 public constant FACTOR_DENOMINATOR = 1e12; // Denominator of the “Basic reward factor”. Default value is 1e12.
  uint256 public constant BASIC_FACTOR_NUMERATOR = 5787; // Numerator of the “Basic reward factor”, for all accounts (except for seed investors) that participate in the program. Default value is 5787, which corresponds to 5.787/1e9 per second. Its associated denominator is FACTOR_DENOMINATOR.
  uint256 public constant SEED_FACTOR_NUMERATOR = 7032; // Numerator of the "Seed reward factor”, for all accounts (except for seed investors) that participate in the program. Default value is 7032, which corresponds to 7.032/1e9 per second. Its associated denominator is FACTOR_DENOMINATOR.
  uint256 public constant BOOST_CAP = 1e24; // Cap on actual locked tokens for receiving additional boosts.
  address public LOCK_TOKEN = 0xD057604A14982FE8D88c5fC25Aac3267eA142a08; // Token that HOPR holders need to lock to the contract: xHOPR address.
  address public REWARD_TOKEN = 0xD4fdec44DB9D44B8f2b6d529620f9C0C7066A2c1; // Token that HOPR holders can claim as rewards: wxHOPR address

  IHoprBoost public nftContract; // Address of the NFT smart contract.
  mapping(address => mapping(uint256 => uint256)) public redeemedNft; // Redeemed NFT per account, structured as “account -> index -> NFT tokenId”.
  mapping(address => uint256) public redeemedNftIndex; // The last index of redeemed NFT of an account. It defines the length of the “redeemedBoostToken mapping.
  mapping(address => mapping(uint256 => uint256)) public redeemedFactor; // Redeemed boost factor per account, structured as “account -> index -> NFT tokenId”.
  mapping(address => uint256) public redeemedFactorIndex; // The last index of redeemed boost factor factor of an account. It defines the length of the “redeemedFactor” mapping.

  mapping(address => Account) public accounts; // It stores the locked token amount, earned and claimed rewards per account.
  uint256 public totalLocked; // Total amount of tokens being locked in the incentive program. Virtual token locks are not taken into account.
  uint256 public availableReward; // Total amount of reward tokens currently available in the lock.

  // setup ERC1820
  IERC1820Registry private constant ERC1820_REGISTRY = IERC1820Registry(0x1820a4B7618BdE71Dce8cdc73aAB6C95905faD24);
  bytes32 private constant TOKENS_RECIPIENT_INTERFACE_HASH = keccak256('ERC777TokensRecipient');

  event Sync(address indexed account, uint256 indexed increment);
  event Staked(address indexed account, uint256 indexed actualAmount, uint256 indexed virtualAmount);
  event Released(address indexed account, uint256 indexed actualAmount, uint256 indexed virtualAmount);
  event RewardFueled(uint256 indexed amount);
  event Redeemed(address indexed account, uint256 indexed boostTokenId, bool indexed factorRegistered);
  event Claimed(address indexed account, uint256 indexed rewardAmount);

  /**
   * @dev Provide NFT contract address. Transfer owner role to the new owner address.
   * At deployment, it also registers the lock contract as an ERC777 recipient.
   * @param _nftAddress address Address of the NFT contract.
   * @param _newOwner address Address of the new owner. This new owner can reclaim any ERC20 and ERC721 token being accidentally sent to the lock contract.
   */
  constructor(
    address _nftAddress,
    address _newOwner,
    address _lockToken,
    address _rewardToken
  ) {
    // implement in favor of testing
    uint256 chainId;
    assembly {
      chainId := chainid()
    }
    if (chainId != 100) {
      LOCK_TOKEN = _lockToken;
      REWARD_TOKEN = _rewardToken;
    }
    nftContract = IHoprBoost(_nftAddress);
    transferOwnership(_newOwner);
    ERC1820_REGISTRY.setInterfaceImplementer(address(this), TOKENS_RECIPIENT_INTERFACE_HASH, address(this));
  }

  /**
   * @dev ERC677 hook. Token holders can send their tokens with `transferAndCall` to the stake contract.
   * After PROGRAM_END, it refuses tokens; Before PROGRAM_END, it accepts tokens xHOPR token, sync
   * Account state, and update totalLocked.
   * @param _from address Address of tokens sender
   * @param _value uint256 token amount being transferred
   */
  function onTokenTransfer(
    address _from,
    uint256 _value,
    bytes memory
  ) external returns (bool) {
    require(msg.sender == LOCK_TOKEN, 'HoprStake: Only accept LOCK_TOKEN in staking');
    require(block.timestamp <= PROGRAM_END, 'HoprStake: Program ended, cannot stake anymore.');

    _sync(_from);
    accounts[_from].actualLockedTokenAmount += _value;
    totalLocked += _value;
    emit Staked(_from, _value, 0);

    return true;
  }

  /**
   * @dev ERC777 hook. To receive wxHOPR to fuel the reward pool with `send()` method. It updates the availableReward by tokenAmount.
   * @param from address token holder address
   * @param to address recipient address
   * @param amount uint256 amount of tokens to transfer
   */
  function tokensReceived(
    address,
    address from,
    address to,
    uint256 amount,
    bytes calldata,
    bytes calldata
  ) external override {
    require(msg.sender == REWARD_TOKEN, 'HoprStake: Sender must be wxHOPR token');
    require(to == address(this), 'HoprStake: Must be sending tokens to HoprStake contract');
    require(from == owner(), 'HoprStake: Only accept owner to provide rewards');
    availableReward += amount;
    emit RewardFueled(amount);
  }

  /**
   * @dev Whenever a boost `tokenId` token is transferred to this contract via {IERC721-safeTransferFrom}
   * when redeeming, this function is called. Boost factor associated with the
   * It must return its Solidity selector to confirm the token transfer upon success.
   * @param from address token holder address
   * @param tokenId uint256 amount of tokens to transfer
   */
  function onERC721Received(
    address,
    address from,
    uint256 tokenId,
    bytes calldata
  ) external override returns (bytes4) {
    require(_msgSender() == address(nftContract), 'HoprStake: Cannot SafeTransferFrom tokens other than HoprBoost.');
    require(block.timestamp <= PROGRAM_END, 'HoprStake: Program ended, cannot redeem boosts.');
    // Account memory account = accounts[from];
    _sync(from);

    // redeem NFT
    redeemedNft[from][redeemedNftIndex[from]] = tokenId;
    redeemedNftIndex[from] += 1;

    // update boost factor
    uint256 typeId = nftContract.typeIndexOf(tokenId);
    (uint256 factor, uint256 deadline) = nftContract.boostOf(tokenId);
    require(block.timestamp <= deadline, 'HoprStake: Cannot redeem an expired boost.');

    uint256 boostIndex = redeemedFactorIndex[from];
    uint256 index = 0;
    for (index; index < boostIndex; index++) {
      // loop through redeemed factors, replace the factor of the same type, if the current factor is larger.
      uint256 redeemedId = redeemedFactor[from][index];
      (uint256 redeemedFactorValue, ) = nftContract.boostOf(redeemedId);

      if (nftContract.typeIndexOf(redeemedId) == typeId) {
        if (redeemedFactorValue < factor) {
          redeemedFactor[from][index] = tokenId;
        }
        emit Redeemed(from, tokenId, redeemedFactorValue < factor);
        break;
      }
    }
    if (index == boostIndex) {
      // new type being redeemed.
      redeemedFactor[from][boostIndex] = tokenId;
      redeemedFactorIndex[from] += 1;
      emit Redeemed(from, tokenId, true);
    }

    return IERC721Receiver(address(this)).onERC721Received.selector;
  }

  /**
   * @dev Only owner can call this function to store virtual lock for seed investors.
   * If the investor hasn't locked any token in this account, create an "Account" with {0, caps[i], block.timestamp, 0, 0}.
   * If the investor has locked some tokens in this account, update its “virtualLockedTokenAmount”.
   * This function can be called at anytime of the program.
   * @param investors address[] Array of seed investors accounts.
   * @param caps uint256[] Array of their virtually locked tokens.
   */
  function lock(address[] calldata investors, uint256[] calldata caps) external onlyOwner {
    require(block.timestamp <= PROGRAM_END, 'HoprStake: Program ended, cannot stake anymore.');
    require(investors.length == caps.length, 'HoprStake: Length does not match');

    for (uint256 index = 0; index < investors.length; index++) {
      address investor = investors[index];
      _sync(investor);
      accounts[investor].virtualLockedTokenAmount += caps[index];
      // accounts[investor].lastSyncTimestamp = block.timestamp;
      emit Staked(investor, 0, caps[index]);
    }
  }

  /**
   * @dev Manually sync account's reward states
   * @notice public function of ``_sync``.
   * @param account address Account whose stake rewards will be synced.
   */
  function sync(address account) external {
    _sync(account);
  }

  /**
   * @dev Sync rewards and claim them
   * @notice public function of ``_sync`` + ``_claim``
   * @param account address Account whose stake rewards will be synced and claimed.
   */
  function claimRewards(address account) public {
    _sync(account);
    _claim(account);
  }

  /**
   * @dev Unlock staking
   * @param account address Account that staked tokens.
   */
  function unlock(address account) external {
    require(block.timestamp > PROGRAM_END, 'HoprStake: Program is ongoing, cannot unlock stake.');
    uint256 actualStake = accounts[account].actualLockedTokenAmount;
    uint256 virtualStake = accounts[account].virtualLockedTokenAmount;
    _sync(account);
    accounts[account].actualLockedTokenAmount = 0;
    accounts[account].virtualLockedTokenAmount = 0;
    totalLocked -= actualStake;
    _claim(account);
    // unlock actual staked tokens
    IERC20(LOCK_TOKEN).safeTransfer(account, actualStake);
    // unlock redeemed NFTs
    for (uint256 index = 0; index < redeemedNftIndex[account]; index++) {
      nftContract.transferFrom(address(this), account, redeemedNft[account][index]);
    }
    emit Released(account, actualStake, virtualStake);
  }

  /**
   * @dev Reclaim any ERC20 token being accidentally sent to the contract.
   * @param tokenAddress address ERC20 token address.
   */
  function reclaimErc20Tokens(address tokenAddress) external onlyOwner nonReentrant {
    uint256 difference;
    if (tokenAddress == LOCK_TOKEN) {
      difference = IERC20(LOCK_TOKEN).balanceOf(address(this)) - totalLocked;
    } else {
      difference = IERC20(tokenAddress).balanceOf(address(this));
    }
    IERC20(tokenAddress).safeTransfer(owner(), difference);
  }

  /**
   * @dev Reclaim any ERC721 token being accidentally sent to the contract.
   * @param tokenAddress address ERC721 token address.
   */
  function reclaimErc721Tokens(address tokenAddress, uint256 tokenId) external onlyOwner nonReentrant {
    IHoprBoost(tokenAddress).transferFrom(address(this), owner(), tokenId);
  }

  /**
   * @dev Returns the increment of cumulated rewards during the “lastSyncTimestamp” and current block.timestamp.
   * @param _account address Address of the account whose rewards will be calculated.
   */
  function getCumulatedRewardsIncrement(address _account) public view returns (uint256) {
    return _getCumulatedRewardsIncrement(_account);
  }

  /**
   * @dev Calculates the increment of cumulated rewards during the “lastSyncTimestamp” and block.timestamp.
   * current block timestamp and lastSyncTimestamp are confined in [BASIC_START, PROGRAMEND] for basic and boosted lockup,
   * and [SEED_START, PROGRAMEND] for seed investors.
   * @param _account address Address of the account whose rewards will be calculated.
   */
  function _getCumulatedRewardsIncrement(address _account) private view returns (uint256) {
    Account memory account = accounts[_account];
    if (block.timestamp <= BASIC_START || account.lastSyncTimestamp >= PROGRAM_END) {
      // skip calculation and return directly 0;
      return 0;
    }
    // Per second gain, for basic lock-up.
    uint256 gainPerSec = account.actualLockedTokenAmount * BASIC_FACTOR_NUMERATOR;

    // Per second gain, for additional boost, applicable to amount under BOOST_CAP
    for (uint256 index = 0; index < redeemedFactorIndex[_account]; index++) {
      uint256 tokenId = redeemedFactor[_account][index];
      (uint256 boost, ) = nftContract.boostOf(tokenId);
      gainPerSec += (account.actualLockedTokenAmount.min(BOOST_CAP)) * boost;
    }

    return
      (gainPerSec *
        (block.timestamp.max(BASIC_START).min(PROGRAM_END) -
          account.lastSyncTimestamp.max(BASIC_START).min(PROGRAM_END)) +
        account.virtualLockedTokenAmount *
        SEED_FACTOR_NUMERATOR *
        (// Per second gain, for seed investor lock-up
        block.timestamp.max(SEED_START).min(PROGRAM_END) -
          account.lastSyncTimestamp.max(SEED_START).min(PROGRAM_END))) / FACTOR_DENOMINATOR;
  }

  /**
   * @dev Update “lastSyncTimestamp” with the current block timestamp and update “cumulatedRewards” with _getCumulatedRewardsIncrement(account)
   * @param _account address Address of the account whose rewards will be calculated.
   */
  function _sync(address _account) private {
    uint256 increment = _getCumulatedRewardsIncrement(_account);
    accounts[_account].cumulatedRewards += increment;
    accounts[_account].lastSyncTimestamp = block.timestamp;
    emit Sync(_account, increment);
  }

  /**
   * @dev Claim rewards for staking.
   * @param _account address Address of the staking account.
   */
  function _claim(address _account) private {
    Account memory account = accounts[_account];
    // update states
    uint256 amount = account.cumulatedRewards - account.claimedRewards;
    require(amount > 0, 'HoprStake: Nothing to claim');
    accounts[_account].claimedRewards = accounts[_account].cumulatedRewards;
    require(availableReward >= amount, 'HoprStake: Insufficient reward pool.');
    availableReward -= amount;
    // send rewards to the account.
    IERC20(REWARD_TOKEN).safeTransfer(_account, amount);
    emit Claimed(_account, amount);
  }
}
