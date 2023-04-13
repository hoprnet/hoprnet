import asyncio
import pytest
from tests.conftest import DEFAULT_API_TOKEN, check_socket
from hoprd_api import HoprdAPI


LOCALHOST = "127.0.0.1"
HTTP_STATUS_CODE_OK = 200
HTTP_STATUS_CODE_SEND_MESSAGE_OK = 202


@pytest.mark.asyncio
async def test_stress_hoprd_send_message_should_send_sequential_messages_without_errors(setup_7_nodes, cmd_line_args):
    STRESS_SEQUENTIAL_MESSAGE_COUNT = cmd_line_args["stress_seq_request_count"]

    alice = HoprdAPI(f"http://{LOCALHOST}:{setup_7_nodes['1']['api_port']}", DEFAULT_API_TOKEN)
    bob = HoprdAPI(f"http://{LOCALHOST}:{setup_7_nodes['2']['api_port']}", DEFAULT_API_TOKEN)
    hops = []  # zero hop

    bob_address = await bob.get_address()
    assert bob_address.status_code == HTTP_STATUS_CODE_OK
    bob_address = bob_address.json()["hopr"]

    for _ in range(20):
        if bob_address in [i["peerId"] for i in (await alice.peers()).json()["connected"]]:
            break
        else:
            await asyncio.sleep(1)

    assert bob_address in [i["peerId"] for i in (await alice.peers()).json()["connected"]]

    expected = [HTTP_STATUS_CODE_SEND_MESSAGE_OK] * STRESS_SEQUENTIAL_MESSAGE_COUNT
    actual = [
        (await alice.send_message(bob_address, f"message #{i}", hops)).status_code
        for i in range(STRESS_SEQUENTIAL_MESSAGE_COUNT)
    ]
    assert expected == actual


@pytest.mark.asyncio
async def test_stress_hoprd_send_message_should_send_parallel_messages_without_errors(setup_7_nodes, cmd_line_args):
    STRESS_PARALLEL_MESSAGE_COUNT = cmd_line_args["stress_par_request_count"]

    alice = HoprdAPI(f"http://{LOCALHOST}:{setup_7_nodes['1']['api_port']}", DEFAULT_API_TOKEN)
    roger = HoprdAPI(f"http://{LOCALHOST}:{setup_7_nodes['3']['api_port']}", DEFAULT_API_TOKEN)
    hops = []  # zero hop

    roger_address = await roger.get_address()
    assert roger_address.status_code == HTTP_STATUS_CODE_OK
    roger_address = roger_address.json()["hopr"]

    for _ in range(20):
        if roger_address in [i["peerId"] for i in (await alice.peers()).json()["connected"]]:
            break
        else:
            await asyncio.sleep(1)

    assert roger_address in [i["peerId"] for i in (await alice.peers()).json()["connected"]]

    expected = [HTTP_STATUS_CODE_SEND_MESSAGE_OK] * STRESS_PARALLEL_MESSAGE_COUNT
    actual = await asyncio.gather(
        *[alice.send_message(roger_address, f"message #{i}", hops) for i in range(STRESS_PARALLEL_MESSAGE_COUNT)]
    )
    actual = [i.status_code for i in actual]

    assert expected == actual
