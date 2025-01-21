import asyncio
import logging
from typing import Optional, Any, Generator

from ddapi import Status, ServerTw
from prometheus_client import REGISTRY, Counter, Histogram, start_http_server, Gauge, push_to_gateway, CollectorRegistry

from modals import Config
from util import get_config

status = Status()
config: Config = get_config(Config)

logging.basicConfig(format='%(asctime)s:%(levelname)s:%(name)s: %(message)s', level=getattr(logging, config.log_level.upper()))
_log = logging.getLogger(__name__)

registry = CollectorRegistry()

count_request = Counter('count_request', 'request count', registry=registry)
request_latency_seconds = Histogram('request_latency_seconds', 'histogram', registry=registry)

labels = ["address", "gametype", "map", "name", "hasPassword"]
server_online = Gauge('server_online', 'ddnet server online', labels, registry=registry)
server_online_max = Gauge('server_online_max', 'ddnet server online max', labels, registry=registry)

server_online_per_ip = Gauge('server_online_per_ip', 'ddnet server online per ip', ["address"], registry=registry)
server_online_per_ip_max = Gauge('server_online_per_ip_max', 'ddnet server online per ip max', ["address"], registry=registry)

@request_latency_seconds.time()
async def status_request(_status: Status, addresses: list) -> Optional[Generator[ServerTw, Any, None]]:
    count_request.inc()
    request = await _status.server_list()
    if request is None:
        return

    return (
        server for server in request.servers
        if any(
            server.ip == ip and (port is None or server.port == port)
            for ip, port in addresses
        )
    )


async def main():
    addresses = [
        (ip_port[0], int(ip_port[1]) if len(ip_port) > 1 else None)
        for ip_port in (
            ip.split(":") for ip in config.address
        )
    ]

    if config.gateway_address is not None:
        _log.info('| Push gateway client scheduled')
    else:
        start_http_server(config.port, registry=registry)
        _log.info("| Starting http server")

    _log.info("| Starting")
    while True:
        result = await status_request(status, addresses)
        if result is None:
            await asyncio.sleep(config.sleep)
            continue

        for server in addresses:
            ip = server[0]

            server_online_per_ip.labels(ip).set(0)
            server_online_per_ip_max.labels(ip).set(0)

        for server in result:
            args = {
                "address": f"{server.ip}:{server.port}",
                "map": server.map.name,
                "hasPassword": str(server.hasPassword).lower(),
                "gametype": server.gameType.name,
                "name": server.name
            }

            server_online_per_ip.labels(server.ip).inc(server.numClients)
            server_online_per_ip_max.labels(server.ip).inc(server.maxClients)

            server_online.labels(**args).set(server.numClients)
            server_online_max.labels(**args).set(server.maxClients)

        if config.gateway_address is not None:
            push_to_gateway(config.gateway_address, job='ddnet-exporter', registry=registry)
        await asyncio.sleep(config.sleep)

if __name__ == '__main__':
    try:
        asyncio.run(main())
    except (KeyboardInterrupt, SystemExit):
        asyncio.run(status.close())