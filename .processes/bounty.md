# Bounty Process

The purpose of this process is to follow a consistent way of creating, funding, publishing, and marketing bounties at HOPR.

## The bounty

A bounty usually serves multiple goals.

1. Takes away workload from the _tech_ team as it allows external developers improve the HOPR protocol and the various applications/tools we maintain.
2. Gives the _community_ team the opportunity to showcase HOPR's involvement in open source.
3. Allows us to build a relationship between HOPR and the bounty hunter (tech community).

The primary goal of a bounty creator is to take into account all 3 aspects, and depending on the bounty, adjust accordingly.

## How to

### Creating a bounty

- Create a GitHub issue using the [bounty template](../.github/ISSUE_TEMPLATE/bounty.md) in the repository where the bounty is most applicable.
- Within each section of the template, provide the most concise details you can.
- Feel free to expand on sections `Developing HOPR` and `What is HOPR?` with information more related to the bounty.
- Creator makes a comment in the PR with an estimate of:
  1. Prize of the bounty (in dollars).
  2. Time required to complete the bounty (in days).
- Bounty is sent to be reviewed by the [tech representatives](./development#tech-representantives), most importantly to make sure that:
  1. Bounty does not disturb tech's roadmap.
  2. Prize and time required make sense.

### Funding the bounty

The bounty _must_ be reviewed before receiving funds.

Depending on the medium used to post the bounty, the funds used may be different.
For most cases, use of HOPR tokens is encouraged.

- the funds are in crypto (HOPR, xDai, etc)
  1. Creator of the bounty [creates a wallet](./wallets.md) to be used as an operator wallet.
  2. Request of funds is done by emailing HOPR's accountant.
  3. Funds are sent to the wallet.

### Posting the bounty

Depending on the medium used the process may vary.

- via [Gitcoin](https://gitcoin.co/)

  1. Creator enters the GitHub issue link in Gitcoin's bounty creation portal.
  2. Creator sets prize of the bounty.
  3. Operator wallet is the one created in [Funding the bounty](#funding-the-bounty).

### Community

As soon as the bounty is ready to be promoted, the following must happen:

1. Creator pastes bounty URL within `bounties` channel in element.
2. Creator pastes bounty URL within `bounties (TBD)` channel in discord.
3. Community may use the bounty the way they see fit, with the help of the creator.

## Deadlines

While it's possible to create bounties which do not affect the tech teams roadmap, it's ideal, that we try to create bounties which are relevant to the upcoming roadmap.

After a bounty is [reviewed](#creating-a-bounty), two dates need to be set:

1. tech deadline: the deadline which the tech team requires this bounty to be completed.
2. pickup deadline: the deadline which the tech team will need to pick up this task so it's completed on time according to the roadmap.

The deadline set for the bounty hunter should be the `pickup deadline`, this way, it allows the tech team to fully avoid working on the task in the case it's completed or assist the bounty hunter on completing it. Either way, bounty hunter must be showing good progress throughout the time they are accepted to work on it.

## Creating a PR

In order to ensure that external contributions pass our CI tests, the bounty creator must:

1. Create a branch prefixed by `bounty/<name>` and push it.
2. Bounty hunter may create PRs that merge into that branch.
3. When work is complete, PR is merged into the branch, ensuring tests pass.

## Prizes and rewards

Depending on the prize's currency, there might be fluctuations in the price relative to the dollar amount.

- Price decrease: Bounty creator may choose to tip the bounty hunter with the difference, but not necessary.
- Price increase: Bounty creator is not allowed to change the prize, bounty hunter should receive set amount.

## Awarding the prize

Before the bounty is awarded, one of the [tech representatives](./development#tech-representantives) has to approve the PR which was created to resolve the bounty.

## The Bounty Program

The bounty program is a new initiative at HOPR created to kick-start and build a strong tech community.
Learn about the bounty program [here](https://github.com/hoprnet/hoprnet/issues/3285).
