import logging
import os

import yaml as ym
from pydantic import BaseModel, Field

from configs import LoggingConfig, StringLevels

_log = logging.getLogger(__name__)


class Config(BaseModel):
    logging: LoggingConfig = LoggingConfig(level=StringLevels.info)
    port: int = Field(8000)
    gateway_address: str = None
    address: list[str] = None
    sleep: int = Field(10)
    _default_config: bool = False

    def addresses(self) -> list[tuple[str, int | None]]:
        return [
            (ip_and_port[0], int(ip_and_port[1]) if len(ip_and_port) > 1 else None)
            for ip_and_port in (ip.split(":") for ip in self.address)
        ]

    @staticmethod
    def load() -> "Config":
        if os.path.exists("config.yaml"):
            _log.debug("yaml loading, modal=%s", Config)
            with open("config.yaml", encoding="utf-8") as fh:
                data = ym.load(fh, Loader=ym.FullLoader)
            yaml = Config(**data)
            _log.debug("yaml loaded")
            return yaml
        _log.debug("env loading, modal=%s", Config)
        env = Config(**{k.lower(): v for k, v in os.environ.items()})
        _log.debug("env loaded")
        return env
