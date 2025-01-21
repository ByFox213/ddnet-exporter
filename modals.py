from pydantic import BaseModel, Field


class Config(BaseModel):
    gateway_address: str = None
    address: list[str]
    log_level: str = Field("INFO")
    sleep: int = Field(10)
