from enum import Enum


class TargetEnvironment(Enum):
    """ """

    ANVIL = (0,)
    DOCKER_COMPOSE = (1,)
    KUBERNETES = 2
