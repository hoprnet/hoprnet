# Using A Script (Recommended)

The simplest way to get started with HOPR is to run our [script](https://github.com/hoprnet/hopr-sh) to install **hoprd**. The script will install all the required dependencies, including `node.js` version `16`. If you have [`nvm`](https://github.com/nvm-sh/nvm) installed, it will use it.

```eval_rst
.. ATTENTION::
   Please bear in mind that at the time of writing, ``hoprd`` only has been tested in version ``16``.

   If you are a MacOS M1 user, please refer to the `npm guide <./using-npm.html>`_, this script will not work for you.
```

## Setup and install HOPRd

Our script will ask you to accept our [Privacy Policy](https://hoprnet.org/privacy-policy), and afterwards prompt you for a version to install. You can use any of our [public releases](https://www.npmjs.com/package/@hoprnet/hoprd), but `latest` also works.

```bash
$ bash -c "$(curl -s https://raw.githubusercontent.com/hoprnet/hopr-sh/master/setup-hoprd.sh)"

██╗  ██╗ ██████╗ ██████╗ ██████╗
██║  ██║██╔═══██╗██╔══██╗██╔══██╗
███████║██║   ██║██████╔╝██████╔╝
██╔══██║██║   ██║██╔═══╝ ██╔══██╗
██║  ██║╚██████╔╝██║     ██║  ██║
╚═╝  ╚═╝ ╚═════╝ ╚═╝     ╚═╝  ╚═╝

By installing this node, you agree to our Privacy Policy found at https://hoprnet.org/privacy-policy
Do you agree to our Privacy Policy? [y/n]:y
Terrific!
Warning! Running this script repeatedly will cause you to have a new node address each time.
Would you like to run this script? [y/n]:y
What release are you installing? Format: X.XX.X (https://github.com/hoprnet/hoprnet/releases)
latest
```

You might need to restart your terminal for your computer to be able to find `hoprd` after the script completes installation.

### Ubuntu or Debian

Type following commands into your terminal, if you are using a VPS, log in into your VPS.

```bash
$ sudo apt install -y curl
$ curl https://raw.githubusercontent.com/hoprnet/hopr-sh/master/setup-hoprd.sh --output setup-hoprd.sh
$ chmod +x setup-hoprd.sh
$ ./setup-hoprd.sh
```

### macOS

Type following commands into your terminal. If you have not installed the XCode Command-line utils, please install them via:

```bash
$ xcode-select --install
```

Also check whether you have installed [Homebrew](https://brew.sh/) - the OSX package manager - and install it if it is not installed yet.

```bash
$ brew install curl
$ curl https://raw.githubusercontent.com/hoprnet/hopr-sh/master/setup-hoprd-macos.sh --output setup-hoprd.sh
$ chmod +x setup-hoprd.sh
$ ./setup-hoprd.sh
```

### One-liner

If you like to live dangerously and have no regards to safety and trust us (you shouldn’t) and want a one-liner script, here it is.

```bash
bash -c "$(curl -s https://raw.githubusercontent.com/hoprnet/hopr-sh/master/setup-hoprd.sh)"
```

(we even removed the `$` so you can copy and paste that on your terminal, you savage).
