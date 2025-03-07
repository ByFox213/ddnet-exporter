import enum
import logging

from pydantic import BaseModel, field_validator


class Levels(int, enum.Enum):  # noqa: WPS600
    critical = 50
    fatal = 50
    error = 40
    warning = 30
    info = 20
    debug = 10


class StringLevels(str, enum.Enum):  # noqa: WPS600
    critical = 'CRITICAL'
    fatal = 'FATAL'
    error = 'ERROR'
    warning = 'WARNING'
    info = 'INFO'
    debug = 'DEBUG'


class LogsRenderer(str, enum.Enum):
    text = 'TEXT'
    json = 'JSON'


class Config(BaseModel):
    level: Levels | StringLevels
    time_format: str = 'utc'
    utc: bool = True
    record_format: str = ''
    call_site: bool = True
    renderer: LogsRenderer = LogsRenderer.text

    @field_validator('level', mode='before')
    @classmethod
    def string_level_upper(cls, level: Levels | StringLevels) -> str | int:  # noqa: N805
        log = logging.getLogger("root")
        if isinstance(level, str):
            log.setLevel(getattr(logging, level.upper()))
            return level.upper()
        log.setLevel(level)
        return level

    @field_validator('renderer', mode='before')
    @classmethod
    def string_renderer_upper(cls, renderer: LogsRenderer) -> str | int:  # noqa: N805
        if isinstance(renderer, str):
            return renderer.upper()
        return renderer

    class Config:
        extras = 'allow'
        use_enum_values = True
