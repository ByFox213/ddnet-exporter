from pydantic import BaseModel, Field


class Config(BaseModel):
    gateway_address: str = None
    port: int = Field(8000)
    address: list[str] = None
    log_level: str = Field("INFO")
    sleep: int = Field(10)
