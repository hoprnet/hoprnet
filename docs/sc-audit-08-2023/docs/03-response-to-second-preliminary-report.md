## Response to Second Preliminary Report

All relevant test files have been appropriately updated in alignment with the findings outlined below.

### Findings

#### 5.1 Incorrect indexEvent Input

**Status:** <span style="background-color:#000058">Code Change</span>

**Description of Changes:**

- Signature of indexed event has been corrected

_Commit Hash:_ [40cbbfc5c6b7f72312ab666469a8d1af2f19dbf8](https://github.com/hoprnet/hoprnet/commit/40cbbfc5c6b7f72312ab666469a8d1af2f19dbf8)

#### 5.2 Winning Ticket Can Be Redeemed Multiple Times

**Status:** <span style="background-color:#000058">Code Change</span>

**Description of Changes:**

- Modified check of `ticketIndex` to ensure correct base is always used

_Commit Hash:_ [40cbbfc5c6b7f72312ab666469a8d1af2f19dbf8](https://github.com/hoprnet/hoprnet/commit/40cbbfc5c6b7f72312ab666469a8d1af2f19dbf8)
