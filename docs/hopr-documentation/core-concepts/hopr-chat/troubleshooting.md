---
description: Common errors that might show in your work station when running HOPR Chat.
---

# Troubleshooting

## Running HOPR Chat

### When using the Nodejs binary in macOS, HOPR Chat does not open and shows a “unidentified developer” error.

![](../../.gitbook/assets/hopr-chat-command-cannot-be-openned_lowfi%20%282%29%20%281%29%20%281%29%20%281%29.png)

In macOS, most applications are scanned by **Gatekeeper,** a security feature that ensures software that runs in your computer has been signed properly. To work around this issue, you need to go to the “Security & Privacy” settings in your macOS and click on “Open Anyway” under the “Allow apps downloaded from” section.

![](../../.gitbook/assets/hopr-chat-command-cannot-be-openned_privacy-lowfi%20%282%29%20%281%29%20%281%29%20%281%29.png)

Since the version **HOPR Chat** that uses Nodejs to work has additional precompiled binaries, you might need to repeat this process multiple times with each prompt. **HOPR Chat** will not work properly until you have accepted all the requests from **Gatekeeper.** The following video should give you a breakdown of what that process looks like.

{% embed url="https://player.vimeo.com/video/431443429" caption="" %}

### When using the Docker image in Windows, HOPR Chat triggers a prompt inside my Windows PC about “Filesharing”.

**HOPR Chat** requires write access to a working directory to store important data in your computer. If you didn't started your command line with elevated privileges, you might be prompted to give access to your current working directory.

![Windows prompting access to write to your current directory](../../.gitbook/assets/image%20%282%29%20%282%29%20%281%29%20%281%29%20%281%29.png)

Clicking “Share It” will allow you to run **HOPR Chat** successfully.
