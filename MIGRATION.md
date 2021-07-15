# Migrating between releases

At the moment we DO NOT HAVE backward compatibility between releases. We attempt
to provide instructions on how to migrate your tokens between releases.

1. BACKUP YOUR DATABASES
2. DO NOT FUND HOPR NODES WITH TOKENS YOU CANNOT AFFORD TO LOSE

## Backing up your databases

HOPR stores information about open channels etc. in a database that lives by
default at `./db`

Since version `1.71` HOPR stores your encrypted private key in a file that by
default lives at `~/.hopr-identity`

Both of these should be backed up as without this data it is impossible to
access tokens that are locked in payment channels.

## Upgrading

The standard upgrade procedure should be to withdraw all funds to an external
wallet before upgrading software.

- Set your automatic channel strategy to `MANUAL`
- Close all open payment channels.
- Once all payment channels have closed, withdraw your funds to an external
  wallet.
- Update the software.
- Start the new software and observe the account address.
- Fund the new account.
