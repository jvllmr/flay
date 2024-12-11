from __future__ import annotations
from libcst import (
    CSTVisitor,
    MetadataWrapper,
)

from flay.common.module_spec import get_parent_package
from .node_remover import NodeRemover
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
from flay.common.util import safe_remove_dir

log = logging.getLogger(__name__)


def is_if_name_main(node: cst.CSTNode) -> bool:
    match node:
        case (
            cst.If(
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
            )
            | cst.If(
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
            )
        ):
            return True
        case _:
            return False


class ReferenceBumper(CSTVisitor):
    METADATA_DEPENENCIES = (FullyQualifiedNameProvider,)

    def __init__(self, references_counter: ReferencesCounter):
        self.references_counter = references_counter

    def on_visit(self, node: cst.CSTNode) -> bool:
        self.references_counter.maybe_increase(node)
        return True


class ReferencesCounter(CSTVisitor):
    METADATA_DEPENDENCIES = (FullyQualifiedNameProvider, ScopeProvider)

    def __init__(self, references_counts: dict[str, int]) -> None:
        self.references_counts: dict[str, int] = references_counts
        self.new_references_count = 0
        self.bumper = ReferenceBumper(self)
        super().__init__()

    def reset(self) -> None:
        self.new_references_count = 0

    def increase(self, fqn: QualifiedName | str) -> None:
        key = fqn if isinstance(fqn, str) else fqn.name
        old_reference_counts = self.references_counts[key]
        self.references_counts[key] = old_reference_counts + 1
        if old_reference_counts == 0:
            self.new_references_count += 1

    def maybe_increase(self, node: cst.CSTNode) -> None:
        fq_names = self.get_metadata(FullyQualifiedNameProvider, node, default=None)
        if not fq_names:
            return

        for fqn in fq_names:
            self.increase(fqn)
            log.debug(f"Increased references count for {fqn.name}")

    def has_references_for(self, node: cst.CSTNode) -> bool:
        fq_names = self.get_metadata(FullyQualifiedNameProvider, node, default=None)
        if not fq_names:
            return False

        for fqn in fq_names:
            if self.references_counts[fqn.name] > 0:
                return True

        return False

    def on_visit(self, node: cst.CSTNode) -> bool:
        match node:
            case (
                cst.ClassDef(body=body, decorators=decorators)
                | cst.FunctionDef(body=body, decorators=decorators)
            ):
                if decorators or self.has_references_for(node):
                    # TODO: accessing children this way with libcst is heavy
                    # stdlib ast solves this in less costly way, but then receiving FQNs is not simple
                    # -> only implement relevant visitors to bump references?
                    self.maybe_increase(node)
                    # NOTE: maybe discard unused ClassVars/instance attributes in the future
                    body.visit(self.bumper)
                return False
            case cst.Assign(targets=targets) | cst.AnnAssign(target=targets):
                if isinstance(targets, cst.BaseAssignTargetExpression):
                    targets = [targets]
                for target in targets:  # type: ignore[attr-defined]
                    if self.has_references_for(target):
                        self.maybe_increase(node)
                        node.visit(self.bumper)
                        break
                return True
            case cst.Call():
                scope = self.get_metadata(ScopeProvider, node, default=None)
                if not scope:
                    return True
                if scope is scope.globals:
                    self.maybe_increase(node)
                    node.visit(self.bumper)
                return False
            case cst.Module(body=body):
                for body_node in body:
                    if isinstance(body_node, cst.If) and is_if_name_main(body_node):
                        self.maybe_increase(node)

                        for accepted_node in body_node.body.body:
                            self.maybe_increase(accepted_node)
                            accepted_node.visit(self.bumper)
                return True
            case _:
                return True


def treeshake_package(
    source_dir: str, preserve_packages: t.Collection[str] | None = None
) -> dict[str, int]:
    stats: dict[str, int] = defaultdict(int)
    source_files: set[str] = set()
    known_module_specs: dict[str, str] = {}
    for path, dirs, files in os.walk(source_dir):
        for file in files:
            if file.endswith(".py") or file.endswith(".pyi"):
                file_path = f"{path}/{file}"
                source_files.add(file_path)
                known_module_specs[file_path] = ".".join(
                    file_path[len(source_dir) :].strip("/").split("/")[:-1]
                )

                if file.endswith("__init__.py"):
                    known_module_specs[path] = (
                        path[len(source_dir) :].strip("/").replace("/", ".")
                    )

    repo_manager = FullRepoManager(
        source_dir, paths=source_files, providers={FullyQualifiedNameProvider}
    )
    file_modules: dict[str, MetadataWrapper] = {}
    references_counts: dict[str, int] = defaultdict(int)
    new_references_count = 1
    for file_path in source_files:
        file_modules[file_path] = file_module = (
            repo_manager.get_metadata_wrapper_for_path(file_path)
        )

        # __main__.py should be preserved
        if file_path.endswith("__main__.py"):
            log.debug(f"File {file_path} will be preserved")
            fqnames = file_module.resolve(FullyQualifiedNameProvider)
            for fqns in fqnames.values():
                for fqn in fqns:
                    references_counts[fqn.name] = references_counts[fqn.name] + 1
                    new_references_count += 1

    references_counter = ReferencesCounter(references_counts)
    treeshake_iteration = 1
    # count references until no new references get added
    # NOTE: maybe follow imports instead? should be taken into consideration for future improvements
    while new_references_count:
        log.debug(f"Treeshake reference counter iteration {treeshake_iteration}")
        treeshake_iteration += 1
        references_counter.reset()
        for file_path, file_module in file_modules.items():
            file_module.visit(references_counter)
            # TODO: find out if this step is necessary; it could be that both dicts share the same identity
            references_counts |= references_counter.references_counts
            new_references_count = references_counter.new_references_count

    # remove nodes without references
    nodes_remover = NodeRemover(references_counts, set(known_module_specs.values()))
    for file_path, file_module in file_modules.items():
        module_spec = known_module_specs[file_path]
        parent_package = get_parent_package(module_spec)
        nodes_remover.parent_package = parent_package
        new_module = file_module.visit(nodes_remover)

        if not new_module.body and not file_path.endswith("__init__.py"):
            os.remove(file_path)
            log.debug(f"Removed file {file_path}")
            stats["Module"] += 1
            safe_remove_dir(file_path)

        else:
            with open(file_path, "w") as f:
                f.write(new_module.code)
            log.debug(f"Processed code of {file_path}")
    stats |= nodes_remover.stats

    # clean-up empty modules
    for file_path, file_module in sorted(file_modules.items(), key=lambda x: len(x[0])):
        if (
            not file_module.module.body
            and file_path.endswith("__init__.py")
            and len(os.listdir(os.path.dirname(file_path))) == 1
        ):
            os.remove(file_path)
            log.debug(f"Removed file {file_path}")
            stats["Module"] += 1
            safe_remove_dir(file_path)

    return stats
