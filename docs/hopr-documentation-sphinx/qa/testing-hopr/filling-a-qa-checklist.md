---
description: >-
  Instructions on how to execute a QA checklist for a specific release to
  support the development and testing of HOPR
---

# Filling a QA checklist

{% hint style="info" %}
The following instructions are meant to be followed by individuals interested in the development and support of the HOPR Association. Unless agreed previously, executing a QA Checklist does not involve an economical reward for the tester.
{% endhint %}

## Step 1 - Copy the latest master template.

Using your browser, go to our [QA master template](https://docs.google.com/spreadsheets/d/1DJzgFshwWoZE6MEM916WIYYuSmzePaBPIZV0jlIQHU4/edit#gid=947619674). If you have a Google Suite account, you can just click the “Master Template” sheet and select the option “copy to” &gt; “new spreadsheet”. A new spreadsheet will be copied into your Google Drive for you to fill in.

![Quickly copying our master template to a new spreadsheet.](../.gitbook/assets/image.png)

If you do not have a Google Suite account or would prefer not to use it, you download the original .xlsx file and use another Excel editor to go to the next step.

![You can always download our Excel document and modify it with your favourite editor.](../.gitbook/assets/image.png)

## Step 2 - Get the latest releases.

To properly complete the QA checklist, you need to be able to run at least one of the following:

- **HOPR Chat \(source code, Docker image or executable binary\)**
- **HOPRd \(source code or HOPR PC\)**

In addition, you need to be connected to a network that has one of the following bots running:

- **Randobot**
- **Coverbot**

To obtain the latest releases and addresses of these bots per network, please go to our Releases page.

{% page-ref page="../../resources/releases.md" %}

## Step 3 - Record your screen and use HOPR

After gethering all the necessary information, make sure to start some recording software on your computer before launching the HOPR distribution you'll be using. For macOS, QuickTime player has screen recording capabilities. Users on Windows 10 can use the Xbox Game Bar on the binary you'll use.

Work through all the tasks in the template, and mark them with one of the following as you go along:

- **OK** - The application behaves as expected.
- **BUG** - The application does not behave as expected.

{% hint style="info" %}
To use HOPR, bear in mind you'll need both native and hopr tokens in the network you are testing. Reach out to the team on Discord and/or Telegram to support you with these currencies and/or follow our **Getting Started** instructions for more information.
{% endhint %}

## Step 4 - Report bugs and send checklist

After completing the checklist or reaching a point where you can no longer proceed due to a bug, make sure to note down all the bugs you encountered and submit a [Bug Report](https://github.com/hoprnet/hoprnet/issues/new?assignees=&labels=bug&template=bug-report.md&title=) in our GitHub repository.

Finally, send your recording via [WeTransfer](https://wetransfer.com/) and/or other file sharing application, alongside your checklist file to [qa@hoprnet.org](mailto:qa@hoprnet.org). Please include your name and HOPR version in your email.
