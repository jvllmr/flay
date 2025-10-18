import importlib as importer
from clonf import clonf_click


abc = "abc"

aliased_module = importer.import_module("dynamic_imports.sub_module.aliased")

@clonf_click
def annotated() -> None:
    pass
