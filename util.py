import logging
from collections.abc import AsyncGenerator

import yaml
from ddapi import DDnetApi, Server
from prometheus_client import CollectorRegistry, Summary

_log = logging.getLogger(__name__)

REGISTRY = CollectorRegistry()
request_latency_seconds = Summary(
    'request_latency_seconds',
    'histogram',
    registry=REGISTRY
)

def get_config(modal):
    with open('config.yaml', encoding="utf-8") as fh:
        data = yaml.load(fh, Loader=yaml.FullLoader)
    _yaml = modal(**data) if data is not None else None
    return _yaml

@request_latency_seconds.time()
async def status_request(_dd: DDnetApi, addresses: list) -> AsyncGenerator[Server] | None:
    request = await _dd.master()
    if request is None:
        yield [], None

    for server in request.servers:
        for ip, port in addresses:
            addr = server.addresses[0].replace("tw-0.6+udp://", "").split(":")
            if addr[0] == ip and (port is None or addr[1] == port):
                yield addr, server

    yield [], None
