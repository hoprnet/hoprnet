# DevRel Process

The purpose of this process is to streamline the interaction and resource sharing between HOPR's _tech_ team and other departments at HOPR.

## Comm Team

### Allocating tech's resources

Comm team is expected to create github issues in [this repository](https://github.com/hoprnet/hopr-devrel).

- should use [RFM](https://github.com/hoprnet/hopr-devrel/blob/main/.github/ISSUE_TEMPLATE/request-for-minting-hoprboost-nft.md) template for minting NFTs
- should use [Request Tech Work](https://github.com/hoprnet/hopr-devrel/blob/main/.github/ISSUE_TEMPLATE/request-for-tech-work.md) template for other work
- issues labelled as `high priority` will be treated as a [file-alarm](./development.md#fire-alarm)
- normal issues will be queued for work in next sprint planning
- issues should contain all information the author can provide, even if that means it includes more noise or possibly unrelated data
- tech members are encouraged to ask questions to get more information from the issue author
- issues should be created as early as possible to provide time for scheduling work

Once the issue is created, comm team must share the github issue in element channel `community` and tag both [representatives](./development.md#representatives).

### Usage of various communication tools

| Tool                                  | Usage                                |
| ------------------------------------- | ------------------------------------ |
| Element                               | _Internal_ team chat                 |
| [Github](https://www.github.com)      | Tracking software issues and PRs     |
| [Forum](https://forum.hoprnet.org)    | Main technical discussion space      |
| [Discord](https://discord.gg/dEAWC4G) | Direct technical topics to the forum |

### Tech recap

Tech recap is done once a sprint has finished.
The [tech representatives](./development.md#representatives) are required to create a list containing the most notable changes:

- A list of changes with a link to the issue or PR
- Highlight 2-3 most important changes
  - Short description of the change
  - Why is it important
  - Describe what parts of the code changed do (ex: what is the indexer)
- On new releases, mention testing was done & if something specific was tested
- Always try to link the original issue, if missing provide a description

This list can then be shared with the comm team.
In order to ensure that was then edited by the comm team is technically correct, the [tech representatives](./development.md#representatives) need to review the final text.

- comm team sends the content to the [tech representatives](./development.md#representatives)
- [tech representatives](./development.md#representatives) have one working day to give feedback back
