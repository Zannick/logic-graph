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
}

disallowed_chars = re.compile(r'[^A-Za-z_0-9]')
punct = re.compile(r'[,./| -]+')
nested = re.compile(r'[({\[:]')
def construct_id(*args: list[str]) -> str:
    return '__'.join(disallowed_chars.sub('', punct.sub('_', s))
                     for a in args
                     for s in nested.split(a))

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
    for a, *b in tuples:
        yield a


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


PLACE_TYPES = ['RegionId', 'AreaId', 'SpotId']
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