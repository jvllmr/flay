# from cfgv
import collections

from cfgv import _get_check_conditional, _apply_default_conditional_optional, _remove_default_conditional_optional, _check_optional # type: ignore
ConditionalOptional = collections.namedtuple(
    'ConditionalOptional',
    (
        'key', 'check_fn', 'default', 'condition_key', 'condition_value',
        'ensure_absent',
    ),
)
ConditionalOptional.__new__.__defaults__ = (False,)
ConditionalOptional.check = _get_check_conditional(_check_optional) # type: ignore
ConditionalOptional.apply_default = _apply_default_conditional_optional # type: ignore
ConditionalOptional.remove_default = _remove_default_conditional_optional # type: ignore


def main() -> None:
    key = "abc"
    hook_id = "hallo"
    value = "test"
    optional = ConditionalOptional(key, cfgv.check_any, value, 'id', hook_id) # type: ignore
    print(optional)


if __name__ == "__main__":
    main()
