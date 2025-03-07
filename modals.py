import logging

import yaml
from pydantic import BaseModel, Field

from configs import LoggingConfig, StringLevels

_log = logging.getLogger("root")


class Config(BaseModel):
    logging: LoggingConfig = LoggingConfig(level=StringLevels.info)
    port: int = Field(8000)
    gateway_address: str = None
    address: list[str] = None
    sleep: int = Field(10)

    def addresses(self) -> list[tuple[str, int | None]]:
        return [
            (ip_and_port[0], int(ip_and_port[1]) if len(ip_and_port) > 1 else None)
            for ip_and_port in (
                ip.split(":") for ip in self.address
            )
        ]

    @staticmethod
    def yaml() -> 'Config':
        _log.debug("yaml loading, modal=%s", Config)
        with open('config.yaml', encoding="utf-8") as fh:
            data = yaml.load(fh, Loader=yaml.FullLoader)
        _yaml = Config(**data) if data is not None else Config()
        _log.debug("yaml loaded")
        return _yaml
