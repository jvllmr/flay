from __future__ import annotations
from libcst import (
    CSTVisitor,
    MetadataWrapper,
    CSTTransformer,
    CSTNodeT,
    RemovalSentinel,
    RemoveFromParent,
    Module,
)
from libcst.metadata import (
    FullyQualifiedNameProvider,
    FullRepoManager,
    ScopeProvider,
    QualifiedName,
)
import libcst as cst
import os
from collections import defaultdict
import typing as t
import logging

log = logging.getLogger(__name__)


def is_if_name_main(node: cst.CSTNode) -> bool:
    match node:
        case cst.If(
            test=cst.Comparison(
                left=cst.Name(
                    value="__name__",
                ),
                comparisons=[
                    cst.ComparisonTarget(
                        operator=cst.Equal(),
                        comparator=cst.SimpleString(
                            value='"__main__"',
                        ),
                    ),
                ],
            )
        ):
            return True
        case cst.If(
            test=cst.Comparison(
                left=cst.SimpleString(
                    value='"__main__"',
                ),
                comparisons=[
                    cst.ComparisonTarget(
                        operator=cst.Equal(),
                        comparator=cst.Name(
                            value="__name__",
                        ),
                    ),
                ],
            )
        ):
            return True
    return False


class ReferencesCounter(CSTVisitor):
    METADATA_DEPENDENCIES = (FullyQualifiedNameProvider, ScopeProvider)

    def __init__(self, references_counts: dict[str, int]) -> None:
        self.references_counts: dict[str, int] = references_counts
        self.new_references_count = 0

        super().__init__()

    def reset(self) -> None:
        self.new_references_count = 0

    def increase(self, fqn: QualifiedName | str) -> None:
        key = fqn if isinstance(fqn, str) else fqn.name
        old_reference_counts = self.references_counts[key]
        self.references_counts[key] = old_reference_counts + 1
        if old_reference_counts == 0:
            self.new_references_count += 1

    def visit_Module(self, node: Module) -> t.Literal[True]:
        for body_node in node.body:
            if isinstance(body_node, cst.If) and is_if_name_main(body_node):
                for accepted_node in body_node.body.body:
                    fq_names = self.get_metadata(
                        FullyQualifiedNameProvider, accepted_node, default=None
                    )
                    if not fq_names:
                        continue

                    for fqn in fq_names:
                        self.increase(fqn)

        return True


class NodeRemover(CSTTransformer):
    METADATA_DEPENDENCIES = (FullyQualifiedNameProvider,)

    def __init__(self, references_counts: dict[str, int]):
        self.references_counts = references_counts
        super().__init__()

    def on_leave(
        self, original_node: CSTNodeT, updated_node: CSTNodeT
    ) -> RemovalSentinel | CSTNodeT:
        if isinstance(
            updated_node,
            (cst.Name, cst.Attribute, cst.Subscript, cst.Call, cst.SimpleString),
        ):
            return updated_node  # type: ignore
        fq_names = self.get_metadata(FullyQualifiedNameProvider, original_node, None)
        if not fq_names:
            return updated_node

        for fqn in fq_names:
            if self.references_counts[fqn.name] > 0:
                return updated_node
        log.debug(f"Removed {set(fqn.name for fqn in fq_names)}")
        return RemoveFromParent()


def treeshake_package(
    source_dir: str, preserve_packages: t.Collection[str] | None = None
) -> None:
    source_files: set[str] = set()
    for path, dirs, files in os.walk(source_dir):
        for file in files:
            if file.endswith(".py") or file.endswith(".pyi"):
                source_files.add(f"{path}/{file}")
    repo_manager = FullRepoManager(
        source_dir, paths=source_files, providers={FullyQualifiedNameProvider}
    )
    file_modules: dict[str, MetadataWrapper] = {}
    references_counts: dict[str, int] = defaultdict(int)
    new_references_count = 0
    for file_path in source_files:
        file_modules[file_path] = file_module = (
            repo_manager.get_metadata_wrapper_for_path(file_path)
        )

        # __main__.py should be preserved
        if file_path.endswith("__main__.py"):
            fqnames = file_module.resolve(FullyQualifiedNameProvider)
            for fqns in fqnames.values():
                for fqn in fqns:
                    references_counts[fqn.name] = references_counts[fqn.name] + 1
                    new_references_count += 1

    references_counter = ReferencesCounter(references_counts)

    # count references until no new references get added
    # NOTE: maybe follow imports instead? should be taken into consideration for future improvements
    while new_references_count:
        references_counter.reset()
        for file_path, file_module in file_modules.items():
            file_module.visit(references_counter)
            # TODO: find out if this step is necessary; it could be that both dicts share the same identity
            references_counts |= references_counter.references_counts
            new_references_count = references_counter.new_references_count
    print(references_counts)

    # remove nodes without references
    nodes_remover = NodeRemover(references_counts)
    for file_path, file_module in file_modules.items():
        new_module = file_module.visit(nodes_remover)
        if not new_module.body:
            os.remove(file_path)
            log.debug(f"Removed file {file_path}")

            directory_path = os.path.dirname(file_path)
            while directory_path:
                if not os.listdir(directory_path):
                    os.rmdir(directory_path)
                    log.debug(f"Removed directory {directory_path}")
                    if directory_path != "/":
                        directory_path = os.path.dirname(directory_path)
                        continue
                break

        else:
            with open(file_path, "w") as f:
                f.write(new_module.code)
            log.debug(f"Cleared code from {file_path}")
