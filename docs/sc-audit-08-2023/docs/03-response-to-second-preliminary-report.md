## Response to Second Preliminary Report

All relevant test files have been appropriately updated in alignment with the findings outlined below.

Updated Git commit hash under audit is:

```
83929b72730ff7264e7728567135fc8274693ed9
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

**Commit Hash:** [83929b72730ff7264e7728567135fc8274693ed9](https://github.com/hoprnet/hoprnet/commit/83929b72730ff7264e7728567135fc8274693ed9)
