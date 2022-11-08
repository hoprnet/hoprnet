---
id: start-here
title: Start here
---

# Start here

:::caution Warning
The HOPR client software (hoprd) is still under heavy development. Please do _not_ add funds to the node that you cannot lose.

For further questions, please visit our [Telegram channel](https://t.me/hoprnet)
:::

To use the HOPR network, you will need a HOPR node. Currently, the only way to do so is running `hoprd`, a node.js process that implements the HOPR protocol and effectively transforms the device you are running it in into a HOPR node. Please bear in mind that by simply installing `hoprd`, you are not making your computer a HOPR node. It is required you run the service as an application and have a working internet connection.

There are several ways to run `hoprd`: you can run it in your own device, install it on a virtual private server (VPS) or use a dedicated hardware device such as the AVADO HOPR Node PC, which has it as a package (Docker image).

## For VPS users

If you’re running your nodes on a VPS, make sure you’ve logged in to your server with the port forwarding feature.

```
ssh -L 3000:127.0.0.1:3000 <root or username>@<Your_server_ip>
```

`<root or username>` - replace with your server username

`<Your_server_ip>` - replace with your server IP

Example: `ssh -L 3000:127.0.0.1:3000 root@192.168.0.1`

## For Linux users

When you are starting your node, it will be running until you will close your terminal / command prompt window. It means it will not be running in background, to achieve that, you will need to use linux apps like: tmux or screen.

### What is Tmux or Screen?

Within one terminal window you can open multiple windows. Each window will contain its own, independently running terminal instance. This allows you to have multiple terminal commands and applications running visually next to each other without the need to open multiple terminals. On top of that tmux and screen keeps these windows in a session, which means your node will be able to run in the background.

### Using Tmux

First of all, check if tmux is installed on your linux OS. Run command:

```
tmux -V
```

It should output which version it is.

For example: `tmux 3.0a`

If it has no `tmux` on your OS, run this command to install it:

```
sudo apt install tmux
```

To run your node in background, open the session running command:

```
tmux
```

It will open a new window session inside the same terminal window. Now you can execute the command to run the node. From now on your node will be running in the background.

#### Main key combinations to use tmux:

```
CTRL + B and press key D
```

It will exit from a session window but it will not close the session itself.

```
tmux ls
```

It will list all your active sessions. The output should look similar to this:

```
0: 1 windows (created Wed Nov 24 08:26:20 2021)
```

You can see that it has one session which ID is 0.

```
tmux attach-session -t <number> or <session name>
```

It will enter the session, `<number>` or `<session name>` is the session ID or name.

Example: `tmux attach-session -t 0`

```
tmux kill-session -t <number> or <session name>
```

It will close your session and will back to terminal, `<number>` or `<session name>` is the session ID or name.

Example: `tmux kill-session -t 0`

:::info
More information about tmux you can read [here](https://linuxize.com/post/getting-started-with-tmux/).
:::

### Using Screen

Check if screen is installed on your linux OS. Run command:

```
screen --version
```

It should output which version it is.

For example: `Screen version 4.08.00 (GNU) 05-Feb-20`

If it has no `screen` on your OS, run this command to install it:

```
sudo apt install screen
```

To run your node in background, open the session running command:

```
screen
```

It will open a new window session inside the same terminal window. Press key `space bar` and now you can execute the command to run the node. From now on your node will be running in the background.

#### Main key combinations to use screen:

```
CTRL + A and press key D
```

It will exit from a session window but it will not close the session itself.

```
screen -ls
```

It will list all your active sessions.

```
Output
There is a screen on:
	5235.pts-0.hopr	(11/24/21 12:48:45)	(Detached)
1 Socket in /run/screen/S-root.
```

You can see that it has one session which ID is 5235.

```
screen -r <number> or <session name>
```

It will enter the session, `<number>` or `<session name>` is the session ID or name.

Example: `screen -r 5235`

```
screen -S <number> or <session name> -X quit
```

It will close your session and will back to terminal, `<number>` or `<session name>` is the session ID or name.

Example: `screen -S 5235 -X quit`

:::info
More information about screen you can read [here](https://linuxize.com/post/how-to-use-linux-screen/).
:::

## hoprd installation methods

We support multiple distribution mechanisms to install a `hoprd`:

**[Avado](using-avado)**

An [AVADO](https://ava.do/) plug-n-play device able to install a HOPR node as a DappNode package from their store.

**[Docker](using-docker)**

Using [Docker](https://www.docker.com/) you can run a `hoprd` within a container with a shared volume to store your node info.

Regardless of which way you install `hoprd`, you will access it and interact with it through your browser. By default, `hoprd` exposes an admin interface available on `localhost:3000`, although flags can change these settings.
