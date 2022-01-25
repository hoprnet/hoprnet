---
id: cover-traffic-nodes
title: Cover Traffic Nodes
---

One of HOPR’s innovations is to link cover traffic directly to staking: the more HOPR tokens you have, the more cover traffic you’ll receive, and the more HOPR tokens you’ll earn for relaying it.

This is useful for several reasons: the added consistent economic incentive encourages long-lived and reliable nodes, and baking staking payouts — an expected feature of many cryptocurrencies — into the cover traffic mechanism sidesteps some extremely complex problems related to knowing how much cover traffic to issue and who pays for it.

## An Ideal World

In a decentralized network like HOPR, ideally individual nodes would dynamically issue just the right amount of cover traffic to provide cover for the real users in the network. Sending too much takes up valuable bandwidth, while too little risks exposing user metadata. As user numbers go up, the need for cover traffic goes down, and vice versa.

Unfortunately, this raises several problems:

- First, who should pay? Cover traffic is arbitrary data which ideally wouldn’t need to exist. If individual nodes are responsible for issuing cover traffic, then they would have to bear the costs themselves. This runs contrary to the incentive design ethos discussed in earlier episodes, where network behaviour we want to encourage needs to be directly incentivized.

- Second, how do nodes decide how much cover traffic to issue? This is a slightly complicated point, and it’s worth taking a short detour to examine it.

## Network Fog

There’s a weird feature of decentralized networks which complicates almost every protocol design question: nodes never have the full picture. Nodes know about their peers, and they know information their peers share, but all this information becomes less and less reliable the more time that passes and the more steps removed from the original node we get.

This uncertainty affects everything from route planning to defending against attacks, and even basic concepts like joining the network. For cover traffic, because nodes have a foggy picture of the network at best, it’s impossible for individual nodes to assess how much cover traffic is needed. If nodes try to use the traffic they see directly as a barometer, we run into an unfortunate feedback loop, where cover traffic issued by one node triggers a cover traffic response in another.

Put another way: because cover traffic is indistinguishable from real traffic by design, there is no way for individual nodes to tell whether they’re seeing real traffic that needs extra cover or cover traffic itself.

## Cover Traffic Nodes

In its current version, HOPR circumvents both issues by having dedicated cover traffic nodes. Initially the HOPR Association will run these nodes itself, but in the medium-term anyone will be able to run a cover traffic node, provided they meet the requirements. By increasingly federating this responsibility, we ensure that the cover traffic system is reliable and mitigate against a theoretical (but extremely unlikely) attack where issuers of cover traffic collaborate to deanonymize the network by gathering information on all the traffic and subtracting the extra information they know about cover traffic by virtue of running a cover traffic node.

But even with dedicated cover traffic nodes resolving some problems inherent to cover traffic, there’s still a lot more design decisions to make. Cover traffic nodes need to send data through the network like any other node, which means they need to choose a route. This could be done based purely on stake, but if the route fails because nodes are offline, then we’re essentially burning tokens for nothing. But if we choose some other criterion, for example reliability, then nodes won’t be earning based on their stake.
