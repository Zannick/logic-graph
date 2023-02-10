import logging
import os
import re

base_dir = os.path.dirname(os.path.realpath(__file__))
logging.basicConfig(level=logging.INFO, format='{relativeCreated:09.2f} {levelname}: {message}', style='{')

# To be replaced with standard functions instead of helpers
BUILTINS = {
    '$max' : 'cmp::max',
    '$min' : 'cmp::min',
    '$all_checks' : 'ctx.all_checks',
}

disallowed_chars = re.compile(r'[^A-Za-z_0-9]')
def construct_id(*args):
    return '__'.join(disallowed_chars.sub('', a.replace(' ', '_')) for a in args)

def n1(tuples):
    for a, *b in tuples:
        yield a
