# Issue tracking (for team-members only)

Principles:

- Track our bugs / enhancements / milestones in one repository.
- Have the ability to filter them by:
  - label: bug / feature / p:chat / ..
  - milestone: if applicable, else it's considered in backlog
  - assigne: if applicable, else it's considered unassigned
  - project: if applicable

## Creating an issue

- A team-member, or an external contributor creates an issue using one of our [templates](.github/ISSUE_TEMPLATE), these templates may add some labels by default.
- All new issues will contain a 'needs review' label.
- A team-member may review the new issue and label it accordingly, then he/she may remove 'needs-review' label.
- (Optional) A team-member adds it to a milestone.
- (Optional) A team-member assignes a user.
- (Optional) A team-member adds it to a project.

## Why label them using p:\*

- We are using a monorepo
- Possibility to filter by `project`

## Why we add label 'needs review' by default

Makes it easy for an issue maintainer to filter by 'needs review' and review the issues.

temp
