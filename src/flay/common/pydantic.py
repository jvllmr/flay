from __future__ import annotations
from pydantic_settings import (
    BaseSettings,
    PydanticBaseSettingsSource,
    SettingsConfigDict,
    PyprojectTomlConfigSettingsSource,
    TomlConfigSettingsSource,
    YamlConfigSettingsSource,
    JsonConfigSettingsSource,
)
from pydantic import BaseModel
import typing as t
import functools
from pydanclick.model import convert_to_click


class FlayBaseSettings(BaseSettings):
    model_config = SettingsConfigDict(
        from_attributes=True,
        extra="ignore",
        env_file=".env",
        env_prefix="FLAY_",
        case_sensitive=False,
        secrets_dir="/run/secrets",
        pyproject_toml_table_header=("tool", "flay"),
        toml_file="flay.toml",
        yaml_file=("flay.yaml", "flay.yml"),
        json_file="flay.json",
    )

    @classmethod
    def settings_customise_sources(
        cls,
        settings_cls: type[BaseSettings],
        init_settings: PydanticBaseSettingsSource,
        env_settings: PydanticBaseSettingsSource,
        dotenv_settings: PydanticBaseSettingsSource,
        file_secret_settings: PydanticBaseSettingsSource,
    ) -> tuple[PydanticBaseSettingsSource, ...]:
        return (
            init_settings,
            env_settings,
            dotenv_settings,
            file_secret_settings,
            PyprojectTomlConfigSettingsSource(settings_cls=settings_cls),
            TomlConfigSettingsSource(settings_cls=settings_cls),
            YamlConfigSettingsSource(settings_cls=settings_cls),
            JsonConfigSettingsSource(settings_cls=settings_cls),
        )


_TWrappedReturn = t.TypeVar("_TWrappedReturn", covariant=True)

_TBaseModel = t.TypeVar("_TBaseModel", bound=BaseModel)
_TBaseModelCon = t.TypeVar("_TBaseModelCon", bound=BaseModel, contravariant=True)


class _ConfigFromPydanticFunc(t.Protocol[_TBaseModelCon, _TWrappedReturn]):
    def __call__(self, config: _TBaseModelCon) -> _TWrappedReturn: ...


if t.TYPE_CHECKING:
    TWrapped = functools._Wrapped[
        [_TBaseModel],
        _TWrappedReturn,
        [],
        _TWrappedReturn,
    ]


def config_from_pydantic(
    model: type[_TBaseModel],
) -> t.Callable[
    [_ConfigFromPydanticFunc[_TBaseModel, _TWrappedReturn]],
    TWrapped[_TBaseModel, _TWrappedReturn],
]:
    options, _ = convert_to_click(model)

    def wrapper(
        f: _ConfigFromPydanticFunc[_TBaseModel, _TWrappedReturn],
    ) -> TWrapped[_TBaseModel, _TWrappedReturn]:
        if not hasattr(f, "__click_params__"):
            f.__click_params__ = []  # type: ignore[attr-defined]
        f.__click_params__.extend(reversed(options))  # type: ignore[attr-defined]

        @functools.wraps(f)
        def wrapped(**kwargs: t.Any) -> _TWrappedReturn:
            config = model(**kwargs)
            return f(config=config)

        return wrapped

    return wrapper
