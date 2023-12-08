import logging
import os
import re
from typing import Any

base_dir = os.path.dirname(os.path.realpath(__file__))
logging.basicConfig(level=logging.INFO, format='{relativeCreated:09.2f} {levelname}: {message}', style='{')

# To be replaced with standard functions instead of helpers
BUILTINS = {
    '$max': 'std::cmp::max',
    '$min': 'std::cmp::min',
    '$all_spot_checks': 'ctx.all_spot_checks',
    '$all_area_checks': 'ctx.all_area_checks',
    '$all_region_checks': 'ctx.all_region_checks',
    '$reset_region': 'ctx.reset_region',
    '$reset_area': 'ctx.reset_area',
    '$get_region': 'get_region',
    '$get_area': 'get_area',
    # TODO: Add a collect_from builtin. Note we need the world for this.
    '$skip': 'ctx.skip',
    '$visit': 'ctx.visit',
    '$count': 'ctx.count',
    '$add_item': 'ctx.add_item',
    '$default': 'Default::default',
    # warning: be careful not to introduce infinite loops in collect rules!
    '$collect': 'ctx.collect',
}

OPS = {
    '==': 'eq',
    '!=': 'ne',
    '>': 'gt',
    '<': 'lt',
    '>=': 'ge',
    '<=': 'lt',
    '=': 'set',
    r'\+': 'add',
    r'\+=': 'incr',
    '-': 'sub',
    '-=': 'decr',
}

disallowed_chars = re.compile(r'[^A-Za-z_0-9]')
punct = re.compile(r'[,./| -]+')
nested = re.compile(r'[({\[:]')
ops = re.compile(r'(?!=)|'.join(OPS.keys()) + r'(?!=)')
def ops_replace(m):
    return OPS[re.escape(m.group(0))]

def escape_ops(text: str) -> str:
    return ops.sub(ops_replace, text)


def construct_id(*args: list[str]) -> str:
    return '__'.join(disallowed_chars.sub('', punct.sub('_', s))
                     for a in args
                     for s in nested.split(a))

def construct_spot_id(*args: list[str]) -> str:
    return f'SpotId::{construct_id(*args)}'

def place_to_names(pl: str) -> list[str]:
    names = pl.split('>')
    return [n.strip() for n in names]

def construct_place_id(pl: str) -> str:
    pt = getPlaceType(pl)
    if pt == 'SpotId':
        return construct_spot_id(*place_to_names(pl))
    else:
        return f'{pt}::{construct_id(pl)}'

def construct_test_name(test_dict):
    if 'name' in test_dict:
        return test_dict['name']
    return '_'.join(
        construct_id(k) + '_' + (construct_test_name(v) if isinstance(v, dict)
                                 else construct_id(*v) if isinstance(v, (list, tuple))
                                 else construct_id(str(v)))
        for k, v in test_dict.items()
    )


def n1(tuples):
    for a, *_ in tuples:
        yield a

def n2(tuples):
    for _, b, *_ in tuples:
        yield b


def config_type(val: Any) -> str:
    if isinstance(val, str):
        if '::' in val:
            return val[:val.index('::')]
        depth = val.count('>')
        if depth == 1:
            return 'AreaId'
        if depth == 2:
            return 'SpotId'
        return 'str'
    if isinstance(val, bool):
        return 'bool'
    if isinstance(val, int):
        return 'int'
    if isinstance(val, float):
        return 'float'
    return type(val).__name__


PLACE_TYPES = ['RegionId', 'AreaId', 'SpotId', 'LocationId']
def getPlaceType(place):
    return PLACE_TYPES[place.count(">")]

ctx_types = {
    'Id': 'SpotId',
    # arguably anything that's a string will be an enum instead
    # but we have to organize all the possible values
    'str': 'ENUM',
    'int': 'i32',
    'float': 'f32',
}
def typenameof(val: Any) -> str:
    rname = config_type(val)
    return ctx_types.get(rname, rname)

int_types = ['i8', 'i16', 'i32']

def fits_in_expected_int(t, expected):
    if t in int_types and expected in int_types:
        return int_types.index(t) <= int_types.index(expected)
    return False


def field_size(max_value: int):
    return max(8, (max_value.bit_length() / 8) * 8)
