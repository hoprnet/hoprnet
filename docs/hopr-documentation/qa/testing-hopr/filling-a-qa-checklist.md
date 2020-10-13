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

Using your browser, go to our [QA master template](https://docs.google.com/spreadsheets/d/1DJzgFshwWoZE6MEM916WIYYuSmzePaBPIZV0jlIQHU4/edit#gid=947619674). If you have a Google Suite account, you can just click under the sheet that reads “master template” and select the option “copy to” &gt; “new spreadsheet”. A new spreadsheet will copied into your Google Drive for you to fill.

![Quickly copying our master template to a new spreadsheet.](../../.gitbook/assets/image%20%2823%29.png)

In case you do not have a Google Suite account, you can always download the original .xlsx file and use another Excel editor to go to the next step.

![You can always download our Excel document and modify it with your favourite editor.](../../.gitbook/assets/image%20%2824%29.png)

## Step 2 - Get the latest releases.

To properly fill the QA checklist, you need to be able to run at least one of the following:

* **HOPR Chat \(source code, Docker image or executable binary\)**
* **HOPRd \(source code or HOPR PC\)**

In addition, you need to be connected to a network that has one of the following bots running:

* **Randobot**
* **Coverbot**

To obtain the latest releases and addresses of these bots per network, please go to our Releases page.

{% page-ref page="../../resources/releases.md" %}

## Step 3 - Record your screen and use HOPR

After you have all the information ready, make sure you start a recording software in your computer before starting to run the HOPR distribution you'll be using. For macOS, QuickTime player has screen recording capabilities, whereas Windows 10&gt; computers can use the Xbox Game Bar on the binary you'll use.

Follow-up all the tasks in the template, and fill them with one of the following

* **OK** - The application behaves as the instructions in the template expects it to.
* **BUG** - The application does not behave as instructed.

{% hint style="info" %}
For using HOPR, bear in mind you'll need both native and hopr tokens in the network you are testing. Reach the team at Discord and/or Telegram to support you with these currencies and/or follow our **Getting Started** instructions for more information.
{% endhint %}

## Step 4 - Report bugs and send checklist

Upon completion of the checklist, make sure to list down all the bugs that you have encountered and submit a [Bug Report](https://github.com/hoprnet/hoprnet/issues/new?assignees=&labels=bug&template=bug-report.md&title=) in our GitHub repository.

Finally, send your recording via [WeTransfer](https://wetransfer.com/) and/or other file sharing application, alongside your checklist file to [qa@hoprnet.org](mailto:qa@hoprnet.org). Please attach the name of the tester and the version in your email.



