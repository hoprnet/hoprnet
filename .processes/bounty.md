# Bounty Process

The purpose of this process is to follow a consistent way of creating, funding, publishing, and marketing bounties at HOPR.

## A Bounty

A bounty usually serves multiple purposes.

1. Takes away workload from the teams at HOPR as it allows external developers to improve the HOPR protocol and the various applications/tools we maintain.
2. Gives the COM-Team the opportunity to showcase HOPR's involvement in open source.
3. Allows us to build a relationship between HOPR and the bounty hunter (tech community).

The primary goal of a bounty creator is to take into account all 3 aspects, and depending on the bounty, adjust accordingly.

## Bounty Admin Process

### Creating the bounty

1. The bounty admin approaches the program owner with a bounty idea and the program owner approves it. Optionally the bounty admin appoints a bounty expert to assist with the technical content.
   - Sometimes the program owner may approaches a person with a bounty idea and asks that person to be the bounty admin.
2. The bounty admin creates a GitHub issue using the [bounty template](../.github/ISSUE_TEMPLATE/bounty.md) in the dedicated bounty repository **(Needs to be created)**
   - Within each section of the template, provide the most concise details you can.
   - Feel free to expand on sections `Developing HOPR` and `What is HOPR?` with information more related to the bounty.
3. The bounty admin assignes herself/himself to the bounty as well as the program owner
   - If the bounty admin appoints a bounty expert the bounty admin assigns the bounty expert to the issue as well.
4. The bounty admin assignes the dedicated bounty label to the issue.
5. The bounty admin makes a comment in the GitHub issue (tagging the program owner) with an estimate of:
   - The time required to complete the bounty (in hours).
   - A guestimate of the bounty reward (in USD) based on a rate of ~50 USD per hour.

### During the time the bounty hunter is working on the bounty

- The bounty admin may be asked (by the program owner) to answer some questions related to the bounty (posted by the bounty hunter) that the program owner cannot answer herself/himself. The bounty admin may consult with the bounty expert before answering the questions.
- In case of an emergency, the program owner might ask the bounty admin to get in touch with the bounty hunter directly to sort out the issue related to the bounty.

### After the bounty hunter submits the work

- The bounty admin (optionally: bounty expert) needs to review/test the submission and the bounty admin gives a reccomendation to the program owner. See xxxx

## Program Owner Process

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
  2. Creator sets prize of the bounty, organisation github handle to `hoprnet`, notification email to `tech@hoprnet.org`
  3. Operator wallet is the one created in [Funding the bounty](#funding-the-bounty).

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
4. If the bounty is a Dune Analytics query or dashboard, ensure that the full code of all queries got submitted via PR and the query/Dashboard got forked by the `hoprnet` Dune account.

## Prizes and rewards

Depending on the prize's currency, there might be fluctuations in the price relative to the dollar amount.

- Price decrease: Bounty creator may choose to tip the bounty hunter with the difference, but not necessary.
- Price increase: Bounty creator is not allowed to change the prize, bounty hunter should receive set amount.

## Awarding the prize

Before the bounty is awarded, one of the [tech representatives](./development#tech-representatives) has to approve the PR which was created to resolve the bounty.
