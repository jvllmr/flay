from __future__ import annotations
from pydantic import BaseModel, ConfigDict
from pydantic_settings import (
    BaseSettings,
    PydanticBaseSettingsSource,
    SettingsConfigDict,
    PyprojectTomlConfigSettingsSource,
    TomlConfigSettingsSource,
    YamlConfigSettingsSource,
    JsonConfigSettingsSource,
)


class FlayBaseModel(BaseModel):
    model_config = ConfigDict(from_attributes=True)


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
