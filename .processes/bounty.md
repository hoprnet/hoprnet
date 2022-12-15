# Bounty Process

The purpose of this process is to follow a consistent way of creating, funding, publishing, and marketing bounties at HOPR. Please familiarize yourself with the [Program.md](https://github.com/hoprnet/bounties-hoprnet-org/blob/main/PROGRAM.md) document before you continue with the bounty process. 

## A Bounty

A bounty usually serves multiple purposes.

1. Takes away workload from the teams at HOPR as it allows external developers to improve the HOPR protocol and the various applications/tools hopr maintains.
2. Gives the COM-team the opportunity to showcase HOPR's involvement in open source.
3. Allows us to build a relationship between HOPR and the bounty hunter (tech community).

## Bounty Admin Process

### Creating the bounty

1. The bounty admin approaches the program owner with a bounty idea and the program owner approves it. Optionally the bounty admin appoints a bounty expert to assist with the technical content.
   - Sometimes the program owner may approach a person with a bounty idea and asks that person to be the bounty admin.
2. The bounty admin creates a GitHub issue using the [bounty template](../.github/ISSUE_TEMPLATE/bounty.md) in the dedicated [bounty repository](https://github.com/hoprnet/hopr-bounty-submission)
   - Within each section of the template, provide the most concise details you can.
   - Feel free to expand on sections `Developing HOPR` and `What is HOPR?` with information more related to the bounty.
3. The bounty admin assignes herself/himself to the bounty as well as the program owner.
   - If the bounty admin appoints a bounty expert the bounty admin assigns the bounty expert to the issue as well. 
4. The bounty admin assignes the dedicated bounty label to the issue.

5. The bounty admin makes a comment in the GitHub issue (tagging the program owner) with an estimate of:
   - The time required to complete the bounty (in hours).
   - A guestimate of the bounty reward (in USD) based on a rate of ~50 USD per hour.

### During the time the bounty hunter is working on the bounty

- The bounty admin may be asked (by the program owner) to answer some questions related to the bounty (posted by the bounty hunter) that the program owner cannot answer herself/himself. The bounty admin may consult with the bounty expert before answering the questions.
- The program owner might ask the bounty admin to get in touch with the bounty hunter directly to sort out the issue related to the bounty.

### After the bounty hunter submits the work

- The bounty admin (optionally: bounty expert) needs to review/test the submission and the bounty admin gives a [recommendation](#Recommendation-to-program-owner) to the program owner.

## Program Owner Process

### Posting the Bounty

1. Program owner approves the github issue created by the bounty admin.

2. Program owner requests 10% of the bounty reward from COM-Safe as the [Gitcoin](https://gitcoin.co/) fee for publishing the bounty.

3. Program owner publishes the bounty via [Gitcoin](https://gitcoin.co/)

4. Program owner reviews bounty applications and selects a bounty hunter

   - program owner might consult with the bounty admin (optinally: bounty expert) before making a decision

5. Program owner establishes a communication channel with the bounty hunter (Discord, Telegram, GitHub, etc..)

### During the time the bounty hunter is working on the bounty

- The program owner requests frequent progress updates from the bounty hunter. The program owner decides the update frequency which might vary from bounty to bounty and can be specified in the bounty template (optinally).
- The program owner answers questions that the bounty hunter might have or delegates the questions to the bounty admin (optinally: bounty expert)

### After the bounty hunter submits the work

- program owner asks the bounty admin (optinally: bounty expert) to review the submission and give a [recommendation](#Recommendation-to-program-owner)


## Posting the bounty on Gitcoin 

 - Program owner enters the GitHub issue link via Gitcoin's bounty creation portal.
 - Program owner sets prize of the bounty and organisation github handle to `hoprnet`. 

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

## Payment of Rewards

Bounty rewards are set in US Dollar. However, the program owner pays out the bounty in HOPR tokens reflecting the set UD Dollar amount.

- For example, consider the case where we choose 100 USD as the bounty reward and assume that at the time of publishing the bounty the HOPR token price equals 1 USD. A bounty hunter submits the work a month later and the program owner decides to pay out the bounty. In the meantime the HOPR token price drops to 0.5 USD. Therfore, the program owner pays out 200 HOPR token worth 100 USD at the time of payment.

## Recommendation to program owner

1. Bounty hunter submits a satisfactory solution that fullfills all deliverables outlined in the github issue within the specified deadlines (if any)

   - Pay out the promised bounty reward

2. Bounty hunter submits a solution of extraordinary quality exceeding the expectations outlined in the github issue

   - Pay out the promised bounty reward and tip the bounty hunter. **Maybe max 20% of the promised amount?**

3. Bounty hunter submits a dis-satisfactory solution but it technically fullfills the deliverables outlined in the github issue.

   - Ask the bounty hunter to implement changes where the changes need to be specified by the bounty admin (optinally: bounty expert) 
   - In case the bounty hunter refuses to implement the changes: Pay out the promised bounty reward.
   - Program owner and bounty admin schedule a **_what went wrong_** assessment of the github issue to minimize the probability that this happens again in the future.

4. Bounty hunter disappears
   - Program owner does **NOT** pay out the bounty reward.
   - Program owner checks with the bounty admin whether is makes sense to re-assign the bounty to someone else or whether the bounty gets canceled.
