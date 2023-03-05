import logging
import os
import re

base_dir = os.path.dirname(os.path.realpath(__file__))
logging.basicConfig(level=logging.INFO, format='{relativeCreated:09.2f} {levelname}: {message}', style='{')

# To be replaced with standard functions instead of helpers
BUILTINS = {
    '$max': 'std::cmp::max',
    '$min': 'std::cmp::min',
    '$all_spot_checks': 'ctx.all_spot_checks',
    '$all_area_checks': 'ctx.all_area_checks',
    '$all_region_checks': 'ctx.all_region_checks',
}

disallowed_chars = re.compile(r'[^A-Za-z_0-9]')
punct = re.compile(r'[,./| -]+')
nested = re.compile(r'[({\[:]')
def construct_id(*args: list[str]) -> str:
    return '__'.join(disallowed_chars.sub('', punct.sub('_', s))
                     for a in args
                     for s in nested.split(a))

def n1(tuples):
    for a, *b in tuples:
        yield a
