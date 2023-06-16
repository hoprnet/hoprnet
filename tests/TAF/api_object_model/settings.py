from requests import Response
from enum import Enum
from strenum import StrEnum

from .root_api_model import RootApiModel

from ..test_data.urls import Urls
from ..services.rest_api_service import RestApiService


class Setting(StrEnum):
    AUTO_REDEEM_TICKETS = ("autoRedeemTickets",)
    INCLUDE_RECIPIENT = ("includeRecipient",)
    STRATEGY = "strategy"


class StrategySetting(Enum):
    PASSIVE = (0,)
    PROMISCUOUS = 1


class Settings(RootApiModel):
    def __init__(self) -> None:
        super().__init__()

    def get_settings(self, nodeIndex) -> str:
        """ """
        restService = RestApiService(self.get_auth_token())
        data = restService.get_request(nodeIndex, Urls.SETTINGS_GET_NODE_SETTINGS)
        return data

    def set_setting(self, nodeIndex: int, setting: Setting, value: str) -> bool:
        """
        Change this node's setting value. Check Settings schema to learn more about each setting
        and the type of value it expects.
        """
        url = self.get_rest_url(nodeIndex, Urls.SETTINGS_UPDATE_NODE_SETTING_VALUE)
        restService = RestApiService(self.get_auth_token())
        response: Response = restService.put_request(url)
        if response.status_code != 204:
            # Setting set succesfully
            return True
        else:
            self.handle_http_error(response)
        return False

    def is_auto_redemption_tickets(self, nodeIndex: int) -> bool:
        """
        Check if auto redemption setting is set on a specific node.
        :nodeIndex: The index of the node to which the setting should be applied
        """
        restService = RestApiService(self.get_auth_token())
        data = restService.get_request(nodeIndex, Urls.SETTINGS_GET_NODE_SETTINGS)
        return bool(data[Setting.AUTO_REDEEM_TICKETS.lower()])

    def set_auto_redemption_tickets(self, nodeIndex: int, value: bool) -> bool:
        """
        :nodeIndex: The index of the node to which the setting should be applied
        :value: True or False
        """
        return self.set_setting(nodeIndex, Setting.AUTO_REDEEM_TICKETS.value, value)

    def is_include_recipient(self, nodeIndex: int, value: bool) -> bool:
        """ """

    def set_include_recipient(self, nodeIndex: int, value: bool) -> bool:
        """
        :nodeIndex: The index of the node to which the setting should be applied
        :value: True or False
        """
        return self.set_setting(nodeIndex, Setting.INCLUDE_RECIPIENT.value, value)
