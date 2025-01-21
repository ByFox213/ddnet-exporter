from pydantic import BaseModel, Field


class Config(BaseModel):
    gateway_address: str = None
    push_gateway_schedule_timer: int = Field(10)
    port: int = Field(8000)
    address: list[str]
    log_level: str = Field("INFO")
    sleep: int = Field(10)
