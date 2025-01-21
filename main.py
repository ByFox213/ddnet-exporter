import asyncio
import logging
from typing import Optional, AsyncGenerator

from ddapi import DDnetApi, Server
from prometheus_client import Counter, Histogram, start_http_server, Gauge, push_to_gateway, CollectorRegistry

from modals import Config
from util import get_config

dd = DDnetApi()
config: Config = get_config(Config)

logging.basicConfig(format='%(asctime)s:%(levelname)s:%(name)s: %(message)s', level=getattr(logging, config.log_level.upper()))
_log = logging.getLogger(__name__)

registry = CollectorRegistry()

count_request = Counter('count_request', 'request count', registry=registry)
request_latency_seconds = Histogram('request_latency_seconds', 'histogram', registry=registry)

labels = ["address", "gametype"]
server_online = Gauge('server_online', 'ddnet server online', labels, registry=registry)
server_online_max = Gauge('server_online_max', 'ddnet server online max', labels, registry=registry)

server_online_per_ip = Gauge('server_online_per_ip', 'ddnet server online per ip', ["address"], registry=registry)
server_online_per_ip_max = Gauge('server_online_per_ip_max', 'ddnet server online per ip max', ["address"], registry=registry)

@request_latency_seconds.time()
async def status_request(_dd: DDnetApi, addresses: list) -> Optional[AsyncGenerator[Server, None]]:
    count_request.inc()
    request = await _dd.master()
    if request is None:
        yield [], None

    for server in request.servers:
        for ip, port in addresses:
            addr = server.addresses[0].replace("tw-0.6+udp://", "").split(":")
            if addr[0] == ip and (port is None or addr[1] == port):
                yield addr, server

    yield [], None


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

    while True:
        for server_ip in addresses:
            ip = server_ip[0]

            server_online_per_ip.labels(ip).set(0)
            server_online_per_ip_max.labels(ip).set(0)

        async for addr, server in status_request(dd, addresses):
            if server is None:
                await asyncio.sleep(config.sleep)
                break

            args = {
                "address": f"{addr[0]}:{addr[1]}",
                "gametype": server.info.game_type
            }
            ip = server.addresses[0].replace("tw-0.6+udp://", "")
            online = len(server.info.clients)

            server_online_per_ip.labels(ip).inc(online)
            server_online_per_ip_max.labels(ip).inc(server.info.max_clients)

            server_online.labels(**args).set(online)
            server_online_max.labels(**args).set(server.info.max_clients)

        if config.gateway_address is not None:
            push_to_gateway(config.gateway_address, job='ddnet-exporter', registry=registry)
        await asyncio.sleep(config.sleep)

if __name__ == '__main__':
    try:
        asyncio.run(main())
    except (KeyboardInterrupt, SystemExit):
        asyncio.run(dd.close())