from __future__ import annotations
from collections import defaultdict
import libcst as cst
from libcst.metadata import FullyQualifiedNameProvider
import logging
import typing as t
from flay.common.logging import LazyStr, log_cst_code
from flay.common.libcst import get_import_from_absolute_module_spec

log = logging.getLogger(__name__)


THandlerReturn = (
    tuple[cst.RemovalSentinel, str | LazyStr]
    | tuple[cst.CSTNodeT, None | str | LazyStr]
)
THandler = t.Callable[
    ["NodeRemover", cst.CSTNodeT, cst.CSTNodeT], THandlerReturn[cst.CSTNodeT]
]


def default_handler(
    node_remover: NodeRemover, original_node: cst.CSTNodeT, updated_node: cst.CSTNodeT
) -> THandlerReturn[cst.CSTNodeT]:
    if node_remover.is_referenced(original_node):
        return updated_node, None
    fq_names = node_remover.get_metadata(
        FullyQualifiedNameProvider, original_node, None
    )
    node_remover.increase_stat(original_node)
    return cst.RemoveFromParent(), str({fqn.name for fqn in (fq_names or [])})


NODE_HANDLERS: dict[  # type: ignore[valid-type]
    type[cst.CSTNode],
    THandler[cst.CSTNodeT],
] = defaultdict(lambda: default_handler)


def import_handler(
    node_remover: NodeRemover, original_node: cst.Import, updated_node: cst.Import
) -> THandlerReturn[cst.Import]:
    def filter_(name: cst.ImportAlias) -> bool:
        return node_remover.is_referenced(name.evaluated_name)

    new_names = list(filter(filter_, updated_node.names))
    if not new_names:
        return cst.RemoveFromParent(), log_cst_code(updated_node)

    if len(new_names) != len(updated_node.names):
        new_node = updated_node.with_changes(names=new_names)
        return new_node, LazyStr(
            lambda: f"{log_cst_code(updated_node)} => {log_cst_code(new_node)}"
        )

    return updated_node, None


NODE_HANDLERS[cst.Import] = import_handler


def import_from_handler(
    node_remover: NodeRemover,
    original_node: cst.ImportFrom,
    updated_node: cst.ImportFrom,
) -> THandlerReturn[cst.ImportFrom]:
    if isinstance(updated_node.names, cst.ImportStar):
        return updated_node, None

    if node_remover.parent_package is None:
        log.warning(
            "Trying process an ImportFrom node removal without specified parent package"
        )
    module_specs = get_import_from_absolute_module_spec(
        updated_node, node_remover.parent_package
    )

    def filter_(name: cst.ImportAlias) -> bool:
        for module_spec in module_specs:
            if not node_remover.is_referenced(f"{module_spec}.{name.evaluated_name}"):
                return False
        return True

    new_names = list(filter(filter_, updated_node.names))

    if not new_names:
        return cst.RemoveFromParent(), log_cst_code(updated_node)

    if len(new_names) != len(updated_node.names):
        new_node = updated_node.with_changes(names=new_names)
        return new_node, LazyStr(
            lambda: f"{log_cst_code(updated_node)} => {log_cst_code(new_node)}"
        )

    return updated_node, None


NODE_HANDLERS[cst.ImportFrom] = import_from_handler


class NodeRemover(cst.CSTTransformer):
    METADATA_DEPENDENCIES = (FullyQualifiedNameProvider,)

    def __init__(self, references_counts: dict[str, int], known_modules: set[str]):
        self.references_counts = references_counts
        self.stats: dict[str, int] = defaultdict(int)
        self.parent_package: str | None = None
        self.known_modules = known_modules
        super().__init__()

    def _is_referenced_str(self, str_: str) -> bool:
        is_str_referenced = self.references_counts[str_] > 0
        if not is_str_referenced and str_ in self.known_modules:
            for key in self.references_counts.keys():
                if key.startswith(str_):
                    self.references_counts[str_] += 1
                    return True
        return is_str_referenced

    def is_referenced(self, node: cst.CSTNodeT | str) -> bool:
        if isinstance(node, str):
            return self._is_referenced_str(node)

        fq_names = self.get_metadata(FullyQualifiedNameProvider, node, None)
        if not fq_names:
            return True

        for fqn in fq_names:
            if self._is_referenced_str(fqn.name):
                return True
        return False

    def increase_stat(self, node: cst.CSTNodeT) -> None:
        self.stats[node.__class__.__name__] += 1

    def on_leave(
        self, original_node: cst.CSTNodeT, updated_node: cst.CSTNodeT
    ) -> cst.RemovalSentinel | cst.CSTNodeT:
        if isinstance(
            updated_node,
            (
                cst.Name,
                cst.Attribute,
                cst.Subscript,
                cst.Call,
                cst.SimpleString,
                cst.Module,
                cst.Decorator,
            ),
        ):
            return updated_node  # type: ignore

        handler = NODE_HANDLERS[type(updated_node)]

        new_node, removed_message = handler(self, original_node, updated_node)

        if isinstance(new_node, cst.RemovalSentinel):
            log.debug(f"Removed {removed_message}")
            return cst.RemoveFromParent()
        if removed_message is not None:
            log.debug(f"Partial remove {removed_message}")
        return new_node
