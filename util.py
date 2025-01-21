import logging

import yaml


_log = logging.getLogger(__name__)


def get_config(modal):
    with open('config.yaml', encoding="utf-8") as fh:
        data = yaml.load(fh, Loader=yaml.FullLoader)
    _yaml = modal(**data) if data is not None else None
    if _yaml is not None:
        _log.info("config loaded from yaml")
        return _yaml