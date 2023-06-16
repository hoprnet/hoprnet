import json


class Withdraw(object):
    def __init__(self, currency: str, amount: str, recipient: str) -> None:
        self.currency = currency
        self.amount = amount
        self.recipient = recipient

    def toJSON(self):
        return json.dumps(self, default=lambda o: o.__dict__, sort_keys=True, indent=4)
