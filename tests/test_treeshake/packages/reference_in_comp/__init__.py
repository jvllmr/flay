import re
import cfgv # type: ignore
import pre_commit.constants as C # type: ignore
from pre_commit.clientlib import _entry, NotAllowed, MANIFEST_HOOK_DICT, _COMMON_HOOK_WARNINGS # type: ignore
# code from pre_commit.clientlib


_meta = (
    (
        'check-hooks-apply', (
            ('name', 'Check hooks apply to the repository'),
            ('files', f'^{re.escape(C.CONFIG_FILE)}$'),
            ('entry', _entry('check_hooks_apply')),
        ),
    ),
    (
        'check-useless-excludes', (
            ('name', 'Check for useless excludes'),
            ('files', f'^{re.escape(C.CONFIG_FILE)}$'),
            ('entry', _entry('check_useless_excludes')),
        ),
    ),
    (
        'identity', (
            ('name', 'identity'),
            ('verbose', True),
            ('entry', _entry('identity')),
        ),
    ),
)



META_HOOK_DICT = cfgv.Map(
    'Hook', 'id',
    cfgv.Required('id', cfgv.check_string),
    cfgv.Required('id', cfgv.check_one_of(tuple(k for k, _ in _meta))),
    # language must be system
    cfgv.Optional('language', cfgv.check_one_of({'system'}), 'system'),
    # entry cannot be overridden
    NotAllowed('entry', cfgv.check_any),
    *(
        # default to the hook definition for the meta hooks
        cfgv.ConditionalOptional(key, cfgv.check_any, value, 'id', hook_id)
        for hook_id, values in _meta
        for key, value in values
    ),
    *(
        # default to the "manifest" parsing
        cfgv.OptionalNoDefault(item.key, item.check_fn)
        # these will always be defaulted above
        if item.key in {'name', 'language', 'entry'} else
        item
        for item in MANIFEST_HOOK_DICT.items
    ),
    *_COMMON_HOOK_WARNINGS,
)


def main() -> None:
    print(META_HOOK_DICT)

if __name__ == "__main__":
    main()
