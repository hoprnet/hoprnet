---
id: what-is-metadata
title: What is Metadata? 
---

The [HOPR protocol](https://hoprnet.org) is special because it protects both your data AND your connection metadata. But what is this metadata, and why is protecting it so important?

Connection metadata is the data that’s generated when you do things online, like visit a website, use an app, or send a message. This metadata records information like who sent the data (in the form of an IP address), where they sent it (another IP), at what time, and how much data was sent. In short, it’s data about data.

On its own, this might not seem like much, but every single interaction online generates dozens of pieces of metadata, almost all public or easy to find. And if someone collects enough of your metadata, it won’t be long before they can build a clear picture of what you do online and learn a whole lot about your life offline. This is possible [even if the connection is end-to-end encrypted](https://arxiv.org/abs/2010.10294).

Like in the picture below, just because you can’t see inside the package, doesn’t mean you can’t work out exactly what’s in it.

![What is a metadata](/img/core/what_is_metadata.png)

Even though the package is wrapped, it’s still not hard to guess what’s inside!

## Why Does Connection Metadata Exist?

So if it’s such a problem, why not just stop all this metadata from being created, or at least make it private? Unfortunately it’s not that simple: public metadata is vital to how the Internet currently functions, a relic from a time when no-one could imagine how big the Internet would grow, or how malicious actors might abuse it.

In basic terms, think of it like mailing a letter. The contents can be sealed in an envelope, but to reach its destination the envelope needs to be clearly addressed. This address information can be read by anyone. If they want to, they can make a note of where the envelope is going, how big it was, and when it was sent. Over time, they can build a database of this information and start finding patterns. All without ever opening an envelope.

If I can see which stores you’re shopping at, which apps you’re using, and who you’re sending messages to, I don’t actually need to know the content of your messages or the full details of your purchases to infer a lot of information about you.

## Is It Really That Big A Deal?

But who is actually seeing this metadata? Every time you go online, dozens of different companies and services all see and potentially log this metadata. There’s internet service providers (ISPs), telecom companies, DNS servers that make the Internet work and content delivery networks (CDNs) such as Cloudflare that actually serve most of the web’s content. These services all gather and store information about you, most of the time, without your consent.

And the way modern web services are all interlinked means this list has only grown. For example, if you visit a website with an embedded YouTube video, then Google will be notified of your visit *even if you don’t click the video*. This information can easily be linked to your name via your IP address, which Google knows thanks to your Google account, and added to the detailed dossier that Google maintains on you. And it’s not just Google. The same thing happens with Facebook, or a blog hosting site like Medium, or instant messaging platforms. None of this requires cookies or any extra code, and changing your privacy settings won’t do anything to stop it. It’s just how the Internet works today.

And that’s before we even get into problems like hackers, government overreach and the huge problem of how to securely handle and protect metadata if you’re an online business, now that regulators have started to take notice.
