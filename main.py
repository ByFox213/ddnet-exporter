import asyncio
import logging

from ddapi import DDnetApi, Server
from prometheus_client import start_http_server, Gauge, pushadd_to_gateway

from modals import Config
from util import REGISTRY, get_config, status_request

dd = DDnetApi()

logging.basicConfig(format='%(asctime)s:%(levelname)s:%(name)s: %(message)s', level=logging.INFO)
_log = logging.getLogger(__name__)

labels = ["address", "gametype", "map", "name", "hasPassword", "max_clients"]
server_online = Gauge(
    'server_online',
    'ddnet server online',
    labels,
    registry=REGISTRY
)
server_online_per_ip = Gauge(
    'server_online_per_ip',
    'ddnet server online per ip',
    ["address"],
    registry=REGISTRY
)



async def update_metrics(server: Server, addr: tuple[str, str]) -> None:
    args = {
        "address": f"{addr[0]}:{addr[1]}",
        "map": server.info.map.name,
        "hasPassword": str(server.info.passworded).lower(),
        "gametype": server.info.game_type,
        "name": server.info.name,
        "max_clients": server.info.max_clients
    }
    online = len(server.info.clients)

    server_online_per_ip.labels(addr[0]).inc(online)
    server_online.labels(**args).set(online)


async def main():
    config: Config = get_config(Config)
    if config is None:
        return ValueError("config is None")

    addresses = [
        (ip_port[0], int(ip_port[1]) if len(ip_port) > 1 else None)
        for ip_port in (
            ip.split(":") for ip in config.address
        )
    ]

    if config.gateway_address is not None:
        _log.info('| Push gateway client scheduled')
    else:
        start_http_server(config.port, registry=REGISTRY)
        _log.info("| Starting http server")

    while True:
        server_online_per_ip.clear()
        server_online.clear()

        async for addr, server in status_request(dd, addresses):
            if server is None:
                await asyncio.sleep(config.sleep)
                break

            await update_metrics(server, addr)

        if config.gateway_address is not None:
            pushadd_to_gateway(config.gateway_address, job='ddnet-exporter', registry=REGISTRY)
        await asyncio.sleep(config.sleep)

if __name__ == '__main__':
    try:
        asyncio.run(main())
    except (KeyboardInterrupt, SystemExit):
        asyncio.run(dd.close())
