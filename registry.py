from prometheus_client import CollectorRegistry, Summary, Gauge

REGISTRY = CollectorRegistry()
request_latency_seconds = Summary(
    'request_latency_seconds',
    'histogram',
    registry=REGISTRY
)
server_online = Gauge(
    'server_online',
    'ddnet server online',
    ["address", "gametype", "map", "name", "hasPassword", "max_clients"],
    registry=REGISTRY
)
server_online_per_ip = Gauge(
    'server_online_per_ip',
    'ddnet server online per ip',
    ["address"],
    registry=REGISTRY
)
