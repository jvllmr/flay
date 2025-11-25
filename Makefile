
install:
	uv sync --all-groups

test:
	uv run pytest

test_matrix:
	uv run nox

cov:
	uv run pytest --cov=flay --cov-report=html

dev: install
	uv run maturin develop

hook:
	python -m maturin_import_hook site install

.DEFAULT_GOAL := dev

.PHONY: install dev test test_matrix cov dev hook
