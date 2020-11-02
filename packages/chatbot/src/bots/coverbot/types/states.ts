export enum NodeStates {
  adminModeDisabled = 'ADMIN_MODE_DISABLED',
  adminCommandReceived = 'ADMIN_COMMAND_RECEIVED',
  newUnverifiedNode = 'UNVERIFIED',
  xdaiBalanceFailed = 'FAILED_XDAI_BALANCE_VERIFICATION',
  xdaiBalanceSucceeded = 'SUCCEEDED_XDAI_BALANCE_VERIFICATION',
  relayingNodeFailed = 'FAILED_RELAYING_PACKET',
  relayingNodeInProgress = 'IN_PROGRESS_RELAYING_PACKET',
  relayingNodeSucceded = 'SUCCEEDED_RELAYING_PACKET',
  onlineNode = 'ONLINE',
  verifiedNode = 'VERIFIED',
}

export enum VerifyTweetStates {
  tweetInvalid = 'INVALID_TWEET',
  tweetVerificationFailed = 'FAILED_TWITTER_VERIFICATION',
  tweetVerificationInProgress = 'IN_PROGRESS_TWITTER_VERIFICATION',
  tweetVerificationSucceeded = 'SUCCEEDED_TWITTER_VERIFICATION',
}
