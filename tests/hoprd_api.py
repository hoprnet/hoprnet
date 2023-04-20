import httpx
import json
import logging


class HoprdAPI(object):
    def __init__(self, api_url, api_token):
        self._api_url = api_url
        self._api_token = api_token

    async def __aenter__(self):
        return self

    async def __aexit__(self, exc_type, exc_val, exc_tb):
        pass

    async def call_api(self, path, method, data: bytes, timeout=600.0):
        async with httpx.AsyncClient(timeout=timeout) as client:
            request = httpx.Request(
                method,
                f"{self._api_url}/api/v2/{path}",
                headers={
                    "X-Auth-Token": f"{self._api_token}",
                    "Content-Type": "application/json",
                },
                content=data,
            )

            try:
                response = await client.send(request)
                response.raise_for_status()
            except httpx.HTTPError as exc:
                logging.error(f"HTTP Exception for {exc.request.url} - {exc}")
            finally:
                return response

    async def withdraw(self, currency, amount, recipient):
        data = json.dumps(
            {
                "currency": f"{currency}",
                "amount": f"{amount}",
                "recipient": f"{recipient}",
            }
        )
        return await self.call_api("account/withdraw", "POST", bytes(data, "utf-8"))

    async def balance(self):
        data = json.dumps({})
        return await self.call_api("account/balances", "GET", bytes(data, "utf-8"))

    async def set_alias(self, peer_id, alias):
        data = json.dumps({"peerId": f"{peer_id}", "alias": f"{alias}"})
        return await self.call_api("aliases", "POST", bytes(data, "utf-8"))

    async def get_alias(self, alias):
        data = json.dumps({})
        return await self.call_api(f"aliases/{alias}", "GET", bytes(data, "utf-8"))

    async def remove_alias(self, alias):
        data = json.dumps({})
        return await self.call_api(f"aliases/{alias}", "DELETE", bytes(data, "utf-8"))

    async def get_aliases(self):
        data = json.dumps({})
        return await self.call_api("aliases", "GET", bytes(data, "utf-8"))

    async def set_setting(self, key, value):
        data = json.dumps({"settingValue": f"{value}"})
        return await self.call_api(f"settings/{key}", "GET", bytes(data, "utf-8"))

    async def get_settings(self):
        data = json.dumps({})
        return await self.call_api("settings", "GET", bytes(data, "utf-8"))

    async def get_all_channels(self, include_closed: bool):
        data = json.dumps({})
        return await self.call_api(f"/channels?includingClosed=${include_closed}", "GET", bytes(data, "utf-8"))

    async def get_tickets_in_channel(self, include_closed: bool):
        data = json.dumps({})
        return await self.call_api(f"/channels?includingClosed=${include_closed}", "GET", bytes(data, "utf-8"))

    async def redeem_tickets_in_channel(self, peer_id):
        """Redeeming tickets can take up to 5 minutes"""
        data = json.dumps({})
        return await self.call_api(f"/channels/{peer_id}/tickets/redeem", "POST", bytes(data, "utf-8"))

    async def redeem_tickets(self):
        """Redeeming tickets can take up to 5 minutes"""
        data = json.dumps({})
        return await self.call_api("tickets/redeem", "POST", bytes(data, "utf-8"))

    async def ping(self, peer_id):
        data = json.dumps(
            {
                "peerId": f"{peer_id}",
            }
        )
        return await self.call_api("node/ping", "POST", bytes(data, "utf-8"))

    async def peers(self):
        data = json.dumps({})
        return await self.call_api("node/peers", "GET", bytes(data, "utf-8"))

    async def get_address(self):
        data = json.dumps({})
        return await self.call_api("account/addresses", "GET", bytes(data, "utf-8"))

    async def send_message(self, destination, message, hops):
        assert isinstance(hops, list)

        data = json.dumps({"body": message, "path": hops, "recipient": destination})
        return await self.call_api("messages", "POST", bytes(data, "utf-8"))
