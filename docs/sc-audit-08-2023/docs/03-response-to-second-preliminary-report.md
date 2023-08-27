## Response to Second Preliminary Report

All relevant test files have been appropriately updated in alignment with the findings outlined below.

Updated Git commit hash under audit is:

```
4ff57c0a1ace5be01f60642e62da1dc56fed5709
```

### Findings

#### 5.1 Incorrect indexEvent Input

**Status:** <span style="background-color:#4cbb17">Code Change</span>

**Description of Changes:**

- Signature of indexed event has been corrected

**Commit Hash:** [40cbbfc5c6b7f72312ab666469a8d1af2f19dbf8](https://github.com/hoprnet/hoprnet/commit/40cbbfc5c6b7f72312ab666469a8d1af2f19dbf8)

#### 5.2 Winning Ticket Can Be Redeemed Multiple Times

**Status:** <span style="background-color:#4cbb17">Code Change</span>

**Description of Changes:**

- Modified check of `ticketIndex` to ensure correct base is always used

**Commit Hash:** [40cbbfc5c6b7f72312ab666469a8d1af2f19dbf8](https://github.com/hoprnet/hoprnet/commit/40cbbfc5c6b7f72312ab666469a8d1af2f19dbf8)

### Informational

#### 7.2 Order of Evaluation Can Be Enforced

**Status:** <span style="background-color:#4cbb17">Code Change</span>

**Description of Changes:**

- Enforced order of evaluation through the use of brackets

**Commit Hash:** [1113eb9f054f84a1ff10cb6b406cb3e36ddc61d6](https://github.com/hoprnet/hoprnet/commit/1113eb9f054f84a1ff10cb6b406cb3e36ddc61d6)

### Others

#### 1. Add getter functions for properties of internal struct `role`

**Commit Hash:** [4ff57c0a1ace5be01f60642e62da1dc56fed5709](https://github.com/hoprnet/hoprnet/commit/4ff57c0a1ace5be01f60642e62da1dc56fed5709)

**Description of Changes:**

- Added `tryGetTarget`, `getTargets` and `getGranularPermissions` functions to get properties of internal struct `role`
