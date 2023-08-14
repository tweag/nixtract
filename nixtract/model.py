from enum import Enum

from pydantic import BaseModel, Field


def snake_case_to_camel_case(name: str):
    parts = name.split("_")
    return "".join([parts[0]] + [part.capitalize() for part in parts[1:]])


class NixpkgsMetadata(BaseModel):
    """Derivation metadata defined by nixpkgs specifically."""

    pname: str | None = Field(
        default=None, description="The pname attribute of the Nix derivation"
    )
    version: str = Field(description="The derivation's version")
    broken: bool = Field(description="Flag indicating whether the derivation is broken")
    license: str = Field(description="The derivation's license")


class Output(BaseModel):
    """An output of a derivation, as specified for multi-output derivations."""

    name: str = Field(description="The output path's name (out, doc, dev, ...)")
    output_path: str = Field(description="The output path")

    class Config:
        alias_generator = snake_case_to_camel_case
        allow_population_by_field_name = True


class ParsedName(BaseModel):
    """The parsed output of the builtins.parseDrvName function."""

    name: str | None = Field(
        default=None, description="The derivation name of the Nix derivation"
    )
    version: str = Field(description="The version of the Nix derivation")


class BuildInputType(Enum):
    """The type of build input. In Nix there are three different types."""

    BUILD_INPUT = "build_input"
    PROPAGATED_BUILD_INPUT = "propagated_build_input"
    NATIVE_BUILD_INPUT = "native_build_input"


class BuildInput(BaseModel):
    """A build input to a Nix derivation"""

    attribute_path: str = Field(
        description="Attribute path from the flake derivation set",
    )
    build_input_type: BuildInputType = Field(
        description="The type of build input",
    )
    # None when it can't be built (e.g. wrong platform)
    output_path: str | None = Field(
        description="The output path of the input derivation",
    )

    class Config:
        use_enum_values = True
        alias_generator = snake_case_to_camel_case
        allow_population_by_field_name = True


class Derivation(BaseModel):
    """A Nix derivation, which is an evaluated (not realized) derivation."""

    attribute_path: str = Field(
        description="Attribute path from the flake derivation set",
    )
    derivation_path: str = Field(
        description="The derivation path of this derivation",
    )
    # None when it can't be built (e.g. wrong platform)
    output_path: str | None = Field(
        description="The output path of this derivation",
    )
    outputs: list[Output] = Field(
        description="A list of the derivation's output paths",
    )

    name: str = Field(
        description="The name of the derivation",
    )
    parsed_name: ParsedName | None = Field(
        default=None,
        description=(
            "The parsed derivation name and version of the derivation by Nix builtins"
        ),
    )
    nixpkgs_metadata: NixpkgsMetadata | None = Field(
        default=None,
        description="Optional metadata specific to derivations from nixpkgs",
    )
    build_inputs: list[BuildInput] = Field(
        description="The derivation's build inputs",
    )

    class Config:
        use_enum_values = True
        # serialize/deserialize JSON with camelCase
        alias_generator = snake_case_to_camel_case
        # use the snake_case attribute names in the model class as kwargs constructor
        allow_population_by_field_name = True
