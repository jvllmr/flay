from pre_commit import cmd_output_b # type: ignore

def _log_and_exit(msg: str, ret_code: int, exc: BaseException, formatted: str) -> None:
    _, git_version_b, _ = cmd_output_b('git', '--version', check=False)
    git_version = git_version_b.decode(errors='backslashreplace').rstrip()


def main() -> None:
    _log_and_exit() # type: ignore

if __name__ == "__main__":
    main()
