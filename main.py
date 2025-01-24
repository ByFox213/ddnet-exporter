import asyncio
import logging

from ddapi import DDnetApi
from prometheus_client import start_http_server, pushadd_to_gateway

from modals import Config
from registry import REGISTRY
from util import get_config, status_request, update_metrics

dd = DDnetApi()

logging.basicConfig(format='%(asctime)s:%(levelname)s:%(name)s: %(message)s', level=logging.INFO)
_log = logging.getLogger(__name__)


async def main():
    config: Config = get_config(Config)
    if config is None:
        config: Config = Config()

    addresses = [
        (ip_port[0], int(ip_port[1]) if len(ip_port) > 1 else None)
        for ip_port in (
            ip.split(":") for ip in config.address
        )
    ] if config.address is not None else None

    if config.gateway_address is not None:
        _log.info('| Push gateway client scheduled')
    else:
        start_http_server(config.port, registry=REGISTRY)
        _log.info("| Starting http server")

    while True:
        servers = []

        async for addr, server in status_request(dd, addresses):
            if server is None:
                await asyncio.sleep(config.sleep)
                break

            args = {
                "ip": addr[0],
                "address": f"{addr[0]}:{addr[1]}",
                "map": server.info.map.name,
                "hasPassword": str(server.info.passworded).lower(),
                "gametype": server.info.game_type,
                "name": server.info.name,
                "max_clients": server.info.max_clients,
                "online": len(server.info.clients)
            }

            servers.append(args)

        logging.debug("update metrics")
        await update_metrics(servers)

        if config.gateway_address is not None:
            pushadd_to_gateway(config.gateway_address, job='ddnet-exporter', registry=REGISTRY)
        await asyncio.sleep(config.sleep)

if __name__ == '__main__':
    try:
        asyncio.run(main())
    except (KeyboardInterrupt, SystemExit):
        asyncio.run(dd.close())
