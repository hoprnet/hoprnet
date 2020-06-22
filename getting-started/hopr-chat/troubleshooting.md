---
description: Common errors that might show in your work station when running HOPR Chat.
---

# Troubleshooting

## Running HOPR Chat

### When using the Nodejs binary in macOS, HOPR Chat does not open and shows a “unidentified developer” error.

![](../../.gitbook/assets/hopr-chat-command-cannot-be-openned_lowfi.png)

In macOS, most applications are scanned by **Gatekeeper,** a security feature that ensures software that runs in your computer has been signed properly. To work around this issue, you need to go to the “Security & Privacy” settings in your macOS and click on “Open Anyway” under the “Allow apps downloaded from” section.

![](../../.gitbook/assets/hopr-chat-command-cannot-be-openned_privacy-lowfi.png)

Since the version **HOPR Chat** that uses Nodejs to work has additional precompiled binaries, you might need to repeat this process multiple times with each prompt. **HOPR Chat** will not work properly until you have accepted all the requests from **Gatekeeper.** The following video should give you a breakdown of what that process looks like.

{% embed url="https://player.vimeo.com/video/431443429" %}



[  
  
](https://vimeo.com/431443429

)

