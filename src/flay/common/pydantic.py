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
        cli_implicit_flags=True,
        cli_kebab_case=True,
        cli_prog_name="flay",
        cli_use_class_docs_for_groups=True,
        cli_ignore_unknown_args=True,
        cli_parse_args=True,
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
