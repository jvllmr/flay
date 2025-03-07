import transitive_star_import.imported # type: ignore

Loader = getattr(transitive_star_import.imported, "CSafeLoader", transitive_star_import.imported.SafeLoader)


if __name__ == "__main__":
    Loader()
