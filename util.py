import logging
from collections.abc import AsyncGenerator

import yaml
from ddapi import DDnetApi, Server

from registry import server_online_per_ip, server_online, request_latency_seconds

_log = logging.getLogger(__name__)

def get_config(modal):
    with open('config.yaml', encoding="utf-8") as fh:
        data = yaml.load(fh, Loader=yaml.FullLoader)
    _yaml = modal(**data) if data is not None else None
    return _yaml


async def update_metrics(servers: list[dict]) -> None:
    server_online_per_ip.clear()
    server_online.clear()

    for args in servers:
        addr = args.pop("ip")
        online = args.pop("online")

        server_online_per_ip.labels(addr).inc(online)
        server_online.labels(**args).set(online)


@request_latency_seconds.time()
async def status_request(_dd: DDnetApi, addresses: list) -> AsyncGenerator[Server] | None:
    request = await _dd.master()
    if request is None:
        yield [], None

    for server in request.servers:
        addr = server.addresses[0].replace("tw-0.6+udp://", "").replace("tw-0.7+udp://", "").split(":")
        if addresses is None:
            yield addr, server
            continue

        for ip, port in addresses:
            if addr[0] == ip and (port is None or addr[1] == port):
                yield addr, server

    yield [], None
