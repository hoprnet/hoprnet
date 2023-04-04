from tests.conftest import DEFAULT_API_TOKEN
import websocket
import json

LOCALHOST = "127.0.0.1"


def url(port):
    return f"ws://{LOCALHOST}:{port}/api/v2/messages/websocket"


def cookie():
    return f"X-Auth-Token={DEFAULT_API_TOKEN}"


def test_websocket_send_receive_messages(setup_7_nodes):
    ws1 = websocket.WebSocket()
    ws1.connect(url(setup_7_nodes["1"]["api_port"]), cookie=cookie())

    ws2 = websocket.WebSocket()
    ws1.connect(url(setup_7_nodes["4"]["api_port"]), cookie=cookie())

    peer1 = setup_7_nodes["1"]["peer_id"]
    peer2 = setup_7_nodes["4"]["peer_id"]

    body1 = "hello msg1 to peer2"
    body2 = "hello msg2 to peer1"

    msg1 = {"cmd": "sendmsg", "args": {"body": body1, "recipient": peer2, "hops": 3}}
    msg2 = {"cmd": "sendmsg", "args": {"body": body2, "recipient": peer1, "hops": 3}}

    ws1.send(json.dumps(msg1))
    ws2.send(json.dumps(msg2))

    inc_msg1 = ws2.recv()
    inc_msg2 = ws1.recv()

    assert json.loads(inc_msg1) == body1
    assert json.loads(inc_msg1) == body2
