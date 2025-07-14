from typing import TYPE_CHECKING, TypeVar
from collections import ChainMap

KT = TypeVar('KT')
VT = TypeVar('VT')

# from pydantic._internal._generics
if TYPE_CHECKING:

    class DeepChainMap(ChainMap[KT, VT]):
        ...
else:

    class DeepChainMap(ChainMap):
        """Variant of ChainMap that allows direct updates to inner scopes.\n\n        Taken from https://docs.python.org/3/library/collections.html#collections.ChainMap,\n        with some light modifications for this use case.\n        """

        def clear(self) -> None:
            for mapping in self.maps:
                mapping.clear()

        def __setitem__(self, key: KT, value: VT) -> None:
            for mapping in self.maps:
                mapping[key] = value

        def __delitem__(self, key: KT) -> None:
            hit = False
            for mapping in self.maps:
                if key in mapping:
                    del mapping[key]
                    hit = True
            if not hit:
                raise KeyError(key)

def main() -> None:
    deep_chain_map: DeepChainMap[str, str] = DeepChainMap({})
    print(deep_chain_map)

if __name__ == "__main__":
    main()
