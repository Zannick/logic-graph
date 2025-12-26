from dataclasses import dataclass
import logging
from typing import Any, Dict, List, Optional, Set

from Utils import config_type, ctx_types, get_int_type_for_max


@dataclass(kw_only=True)
class ContextInfo[T]:
    """Defined and inferred details about a context parameter or setting.

    T should be the *Python* type of the possible values, i.e. str for any enums."""
    
    value_type: str
    rust_type: str
    default: T
    values: Set[T]  # for enums
    # the value set (either a specific value or another variable of the same type)
    # maps to a list of the actions and triggers that can set it
    setters: Dict[T | str, List[str]]


@dataclass(kw_only=True)
class NumericContextInfo[I: int | float](ContextInfo[I]): 
    min: I
    max: I
    # track modifiers that change it by a certain amount?


def make_context_info[T](name: str, category: str, value_type: Optional[str], default: Optional[T], opts: Optional[List[T]], max_value: Optional[T]) -> ContextInfo[T]:
    if max_value:
        if value_type and value_type != T.__name__:
            logging.warning(f'{category} {name} type {value_type} overridden by max: {max_value} ({T.__name__})')
        value_type = config_type(max_value)
        if value_type == 'float':
            return NumericContextInfo[float](value_type=value_type, rust_type='f32', default=default or 0.0,
                                             min=0.0, max=max_value,
                                             values=set(), setters={})
        else:
            return NumericContextInfo[int](value_type=value_type, rust_type=get_int_type_for_max(max_value), default=default or 0,
                                           min=0, max=max_value,
                                           values=set(), setters={})
    elif opts:
        t, *types = {config_type(o) for o in opts}
        if types:
            raise ValueError(f'{category} {name} options are mixed types: {t}, {", ".join(types)}')
        if value_type and value_type != t:
            logging.warning(f'{category} {name} type {value_type} overridden by opts, e.g. {opts[0]} ({t})')
        if t == 'int':
            return NumericContextInfo[int](value_type=t, rust_type=get_int_type_for_max(max(opts)), default=default or opts[0])
        if t == 'float':
            return NumericContextInfo[float](value_type=t, rust_type='f32', default=default or opts[0])
        rt = ctx_types.get(t, t)
        if rt == 'ENUM':
            rt = 'enums::' + name.capitalize()
        return ContextInfo[str](value_type=t, rust_type=rt, default=default or opts[0], values=frozenset(opts), setters={})
    elif not value_type:
        raise ValueError(f'{category} {name} must declare one of: type, max, opts')
    
    if value_type == 'bool' or default == False:
        return ContextInfo[bool](value_type=value_type, rust_type='bool', default=default or False, values=frozenset((True, False)), setters={})
    if value_type == 'SpotId':
        return ContextInfo[str](value_type=value_type, rust_type=value_type, default=default or 'SpotId::None', values=set(), setters={})
    if value_type == 'int':
        return NumericContextInfo[int](value_type=value_type, rust_type='u32', default=default or 0, values=set(), setters={})
    if value_type == 'float':
        return NumericContextInfo[float](value_type=value_type, rust_type='f32', default=default or 0.0, values=set(), setters={})
    if default is None:
        raise ValueError(f'{category} {name} defined as {value_type} which requires a default')
    return ContextInfo[str](value_type=value_type, rust_type=value_type, default=default, values=set(), setters={})


@dataclass(kw_only=True)
class DataInfo[T]:
    """Defined and inferred details about a data parameter.

    T should be the *Python* type of the parameter values, i.e. str for any enums."""
    value_type: str
    rust_type: str
    default: T
    values: Set[T]
    data: Dict[str, T]  # the map of overrides

