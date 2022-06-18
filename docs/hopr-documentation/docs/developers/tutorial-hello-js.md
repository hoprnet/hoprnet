---
id: tutorial-hello-js
title: Hello World in Javascript
---

Now that you set up your HOPR cluster and familiarized yourself with how to connect to a HOPR node, let's build our first HOPR app.
This first example is deliberately kept simple and only requires basic Javascript.
If you're impatient, you can scroll all the way to the end of this example, copy the single HTML file that contains the full example and build the app of your dreams on top of the HOPR protocol already.

## Setting the scene

Before getting to the actual interface to the HOPR node in Javascript, let's set up a minimal HTML UI.

<iframe height="300" style="width: 100%;" scrolling="no" title="HOPR Mini Chat - part 1" src="https://codepen.io/SCBuergel/embed/BaYERPY?default-tab=html%2Cresult&editable=true" frameborder="no" loading="lazy" allowtransparency="true" allowfullscreen="true">
  See the Pen <a href="https://codepen.io/SCBuergel/pen/BaYERPY">
  HOPR Mini Chat - part 1</a> on <a href="https://codepen.io">CodePen</a>.
</iframe>

This minimal web app contains three textboxes to manage the required settings of a HOPR node.
As you've seen in the previous section, the HTTP API is required to send data to the HOPR node (e.g. to send messages),
the WS API is responsible for receiving data from the node (receiving messages and logs)
and the API token is used for authenticating your frontend application with the HOPR node.

Now we're ready to write some Javascript that connects the web UI with our HOPR node.
Let's start with a simple helper function that handles the HOPR REST API calls by formatting the correct headers with authentication and HTTP methods.

<iframe height="300" style="width: 100%;" scrolling="no" title="Untitled" src="https://codepen.io/SCBuergel/embed/gOvyRGB?default-tab=js&editable=true" frameborder="no" loading="lazy" allowtransparency="true" allowfullscreen="true">
  See the Pen <a href="https://codepen.io/SCBuergel/pen/gOvyRGB">
  Untitled</a> on <a href="https://codepen.io">CodePen</a>.
</iframe>

Note how we are using basic authentication in the HTTP header with the Base64 encoded API token (via `window.btoa()`).
The payload that is sent via the `fetch` function to the HOPR node as a HTTP `POST` request contains a JSON object.
The payload has to be passed as a string, otherwise you will receive an error.
The `.catch` handler of the promise that is returned by the `fetch` function is logging any errors to your browser console.
If you are not familiar with promises in Javascript, you can learn about them [here](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Promise).

Now we're finally ready to send our first Hello World message to another HOPR node! With all the preparations that we have done, that's not even 10 lines of Javascript.
<iframe height="300" style="width: 100%;" scrolling="no" title="HOPR Mini Chat - submitMessage" src="https://codepen.io/SCBuergel/embed/OJQGgee?default-tab=js&editable=true" frameborder="no" loading="lazy" allowtransparency="true" allowfullscreen="true">
  See the Pen <a href="https://codepen.io/SCBuergel/pen/OJQGgee">
  HOPR Mini Chat - submitMessage</a> on <a href="https://codepen.io">CodePen</a>.
</iframe>

As outline in the HOPR REST API documentation of the `/messages` endpoint, the method requires two parameters in the paylod:
`recipient` is the HOPR address of the recipient node and `body` is the string of the message text.

Now that we are done with sending a message, let's enable our UI to receive messages.

<iframe height="300" style="width: 100%;" scrolling="no" title="HOPR Mini Chat - ws handlers" src="https://codepen.io/SCBuergel/embed/ExQJvxb?default-tab=js&editable=true" frameborder="no" loading="lazy" allowtransparency="true" allowfullscreen="true">
  See the Pen <a href="https://codepen.io/SCBuergel/pen/ExQJvxb">
  HOPR Mini Chat - ws handlers</a> on <a href="https://codepen.io">CodePen</a>.
</iframe>

A global `ws` object maintains the websocket connection and closes it when we change the settings.
If you do not `close()` the websocket connection and call the `setup` function again, the app will render each message twice.
Similar to the HTTP API, the websocket connection needs to be authenticated with the HOPR node.
For the websocket connection this is done via URL search parameters.

The `handleReceivedMessage` is added to the websocket connection, so that it can process data that it receives from the HOPR node.
The data are received as an object from which we need to parse the `.data` field.
The resulting data object contains a `type`, `ts` and `message` field.
Since the HORP node is not just broadcasting received messages but also logs via the websocket API, we are only processing messages with a `type` field of value `message`.
The `.ts` now finally contains the timestamp of receiving the message and `.msg` contains the actual message text that the sender sent through the HOPR network to your web app.
Here we are simply appending an entry with the timestamp and the message text to our simple HTML page with all the messages that the node sent to our web app since the connection was established.

Calling the `setup` function initializes the websocket connection with default values that are set up in the HTML section upon page load.

We now just add some `onchange` handlers to the settings textboxes so that the `setup` function gets called again when some value changed.
We also call the `submitMessage` function in the `send` buttons `onclick` handler.

And with that we have completed our Hello World HOPR Chat web app.
You can even download the `.html` file and run it by opening the file from your local file system in your browser - no webserver, NodeJS etc required!

If you are trying to use the example with an ad blocker or Brave's shields up, you might be running into issues sending or receiving messages. In that case, consider disabling your ad blocker or Brave shields.

<iframe height="300" style="width: 100%;" scrolling="no" title="HOPR Mini Chat - complete" src="https://codepen.io/SCBuergel/embed/mdXgMEY?default-tab=js&editable=true" frameborder="no" loading="lazy" allowtransparency="true" allowfullscreen="true">
  See the Pen <a href="https://codepen.io/SCBuergel/pen/mdXgMEY">
  HOPR Mini Chat - complete</a> on <a href="https://codepen.io">CodePen</a>.
</iframe>

You can also [download the `Hello.html` file](https://github.com/SCBuergel/hopr-mini-chat/blob/main/public/Hello.html) and start hacking away.
Note that the `apiCall()` function was extended by an optional `isPost` parameter which has to be set to `true` to send `POST` requests, otherwise the requests will be `GET`.
That allows you to use other endpoints conveniently.
You could e.g. go ahead and try modifying the example to get the HORP node's own HOPR address via the REST API and use it as a default value for the `recipient` textbox.

