---
id: network-registry-monte-rosa
title: Network Registry
---

## What is the Network Registry?

The Network Registry is a list of nodes that are allowed to interact with other nodes on the network. If your node is not registered on the "Network Registry" smart contract. it will be unable to send messages or communicate with other nodes. This is a utility used by HOPR to scale and test the network at a controlled pace. 

This restriction on the access guarded by the "Network Registry" is only enabled in the staging or production environment by default. If you are running a cluster of HOPR nodes locally in the hardhat network, the "Network Registry" is not enabled. 

## Monte Rosa Release

The Network Registry is being used for the Monte Rosa release so please make sure you are eligible and have been registered before trying to participate in the release. In order to register your node for the Monte Rosa release you will need to first register your interest to participate in the release.

By completing the form you will be supplying your associated staking address which will be the main indicator of your eligibility to participate in this release. 

### Eligibility

From the list of interested participants an ordered waitlist will be generated. This is determined by your staking address, it will be given a score based on NFT holdings from previous testnets and staking seasons, and HOPR tokens currently staked. 

The waitlist will then be used to add users to the registry in blocks of 20, with most of the 20 being assigned to the next participants on the waitlist and a few being given at random to people further down the waitlist (so you always have a chance of being added even if you don't have the highest rank).

The current version of the waitlist and registered participants can be viewed here.

### Joining the Registry

If you have been chosen to join the registry, you will be sent an NFT to your provided staking account. From here you should first install your node and get it's peerID. You can find details here. 

Once you have both the NFT in your staking account and your peerID, you can visit our registration site and enter these details. They will be checked and if you are indeed eligible your node will be funded with mHOPR and you will be added to the Network Registry smart contract.

You can then proceed to our tutorial and begin using the hopr-admin!
