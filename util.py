import logging
import re
from collections.abc import AsyncGenerator

from ddapi import DDnetApi, Server

from registry import server_online_per_ip, server_online, request_latency_seconds

_log = logging.getLogger(__name__)
ipv4_regex = re.compile(r"(?<=udp://)(\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3})")
ipv6_regex = re.compile(r"(?<=udp://)(\[(?:[0-9a-fA-F]{1,4}:){7}[0-9a-fA-F]{1,4}]|(?:\d{1,3}\.){3}\d{1,3})")


async def update_metrics(servers: list[dict]) -> None:
    _log.debug("updating metrics")
    server_online_per_ip.clear()
    server_online.clear()

    for args in servers:
        addr = args.pop("ip")
        online = args.pop("online")

        server_online_per_ip.labels(addr).inc(online)
        server_online.labels(**args).set(online)


def get_address(address: str) -> tuple:
    ip = ipv4_regex.findall(address) or ipv6_regex.findall(address)
    if not ip:
        return ()
    port = address.split(":")[-1]
    return ip[0], port


@request_latency_seconds.time()
async def status_request(_dd: DDnetApi, addresses: list) -> AsyncGenerator[Server] | None:
    _log.debug("send request")
    request = await _dd.master()
    if request is None:
        _log.info("request - None, an empty list is sent")
        yield [], None

    _log.debug("servers count: %s", len(request.servers))
    for server in request.servers:
        for address in server.addresses:
            addr = get_address(address)
            if not addr:
                continue

            if addresses is None:
                yield addr, server
                continue

            for ip, port in addresses:
                if addr[0] == ip and (port is None or addr[1] == port):
                    yield addr, server

    yield [], None
