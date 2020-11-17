# Issue tracking (for team-members only)

Principles:

- Track our bugs / enhancements / milestones in one repository.
- Have the ability to filter them by:
  - label: bug / feature / p:chat / ..
  - milestone: if applicable, else it's considered in backlog
  - assigne: if applicable, else it's considered unassigned

## Creating an issue

- A team-member, or an external contributor creates an issue using one of our [templates](.github/ISSUE_TEMPLATE), these templates may add some labels by default.
- All new issues will contain a 's:new issue' label.
- A team-member may review the new issue and label it accordingly, see [labels](https://github.com/hoprnet/hoprnet/issues/labels) for descriptions for each label.
- Once a new issue is reviews, 's:new issue' label may be removed.
- (Optional) A team-member adds it to a milestone.
- (Optional) A team-member assignes a user.

## Why label them using p:\*

- We are using a monorepo
- Possibility to filter by `package`

## Why we add label 's:new issue' by default

Makes it easy for an issue maintainer to filter by 's:new issue' and review the issues.
