import os

from Utils import *


TEST_FILE_FIELDS = {'name', 'all', 'tests', '_filename'}
TEST_SETUP_FIELDS = {'name', 'with', 'context', 'settings', 'visited', 'skipped', 'start'}
TEST_TYPES = {'can_obtain', 'cannot_obtain', 'can_reach', 'cannot_reach',
               'eventually_gets', 'path', 'requires', 'eventually_requires'}
TEST_REQUIRES_TYPES = {'to_reach', 'to_obtain'}
TEST_EV_REQUIRES_FIELDS = {'iteration_limit'}

class TestProcessor(object):

    def __init__(self, all_items, context_types, context_str_values, settings, id_lookup):
        self.all_items = all_items
        self.context_types = context_types
        self.context_str_values = context_str_values
        self.settings = settings
        self.id_lookup = id_lookup

    def process_tests(self, tests: list[dict]) -> list[str]:
        self.errors = []
        # go through the categories and check that items/contexts/settings etc are all defined
        for test_file in tests:
            if 'name' not in test_file:
                test_file['name'] = os.path.splitext(test_file['_filename'])[0]
            if unrecog := set(test_file.keys()) - TEST_FILE_FIELDS:
                self.errors.append(f'Unrecognized fields in {test_file["_filename"]}: {", ".join(sorted(unrecog))}')
            if a := test_file.get('all'):
                self._check_test_setup(a, f'{test_file["name"]}:all')
                if unrecog := set(a.keys()) - TEST_SETUP_FIELDS:
                    self.errors.append(f'Unrecognized or invalid fields in {test_file["name"]}:all: {", ".join(sorted(unrecog))}')
            tests = test_file.get('tests')
            if tests:
                if not isinstance(tests, (tuple, list)):
                    self.errors.append(f'Invalid "tests" entry in {test_file["name"]}, expected list but got {type(tests)}')
                    continue
                for test in tests:
                    src = f'{test_file["name"]}.{construct_test_name(test)}'
                    self._check_test_setup(test, src)
                    k = set(test.keys())
                    if unrecog := k - (TEST_SETUP_FIELDS | TEST_TYPES):
                        self.errors.append(f'Unrecognized fields in {src}: {", ".join(sorted(unrecog))}')
                    types = k & TEST_TYPES
                    if not types or len(types) > 1:
                        self.errors.append(f'Test at {src} must have exactly one test type, but has: '
                                           f'{", ".join(sorted(types)) or None}')
                    else:
                        ttype = types.pop()
                        self._check_test(ttype, test[ttype], src)
        return self.errors

    def _check_test_setup(self, setup_dict, src):
        if not isinstance(setup_dict, dict):
            self.errors.append(f'Invalid entry at {src}: expected dict but got {type(setup_dict)}')
            return
        if w := setup_dict.get('with'):
            if not isinstance(w, (tuple, list)):
                self.errors.append(f'Invalid "with" entry in {src}: expected list, got {type(w)}')
            else:
                for item in w:
                    self._check_item(item, src)
        if c := setup_dict.get('context'):
            if not isinstance(c, dict):
                self.errors.append(f'Invalid "context" entry in {src}: expected dict, got {type(w)}')
            else:
                for ckey, cval in c.items():
                    self._check_context(ckey, cval, src)
        if s := setup_dict.get('start'):
            self._check_spot(s, src)
        for stype in ['visited', 'skipped']:
            if s := setup_dict.get('stype'):
                if not isinstance(s, (tuple, list)):
                    self.errors.append(f'Invalid "{stype}" entry in {src}: expected list, got {type(s)}')
                else:
                    for spot in s:
                        self._check_spot(spot, src)

    def _check_test(self, testtype: str, testval: Any, src: str):
        if testtype in ('can_obtain', 'cannot_obtain', 'eventually_gets'):
            self._check_item(testval, src)
        elif testtype in ('can_reach', 'cannot_reach'):
            self._check_spot(testval, src)
        elif testtype == 'path':
            if not isinstance(testval, (tuple, list)):
                self.errors.append(f'Invalid path in {src}: expected list but got {type(testval)}')
            else:
                for i, spot in enumerate(testval):
                    self._check_spot(spot, f'{src}#{i}')
        elif testtype in ('requires', 'eventually_requires'):
            if not isinstance(testval, dict):
                self.errors.append(f'Invalid {testtype} at {src}: expected dict but got {type(testval)}')
            else:
                self._check_test_setup(testval, f'{src}.{testtype}')
                k = set(testval.keys())
                allowed = TEST_SETUP_FIELDS | TEST_REQUIRES_TYPES
                if testtype == 'eventually_requires':
                    allowed.update(TEST_EV_REQUIRES_FIELDS)
                if unrecog := k - allowed:
                    self.errors.append(f'Unrecognized/invalid {testtype} fields in {src}: {", ".join(sorted(unrecog))}')
                types = k & TEST_REQUIRES_TYPES
                if not types or len(types) > 1:
                    self.errors.append(f'Test at {src} {testtype} must have exactly one subtype, but has: '
                                        f'{", ".join(sorted(types)) or None}')
                else:
                    subtype = types.pop()
                    if subtype == 'to_obtain':
                        self._check_item(testval[subtype], src + '.' + subtype)
                    else:
                        self._check_spot(testval[subtype], src + '.' + subtype)

    def _check_item(self, item: str, src: str):
        if item not in self.all_items:
            self.errors.append(f'Unrecognized item in {src}: {item}')

    def _check_context(self, ckey: str, cval: Any, src: str):
        ctype = self.context_types.get(ckey)
        if not ctype:
            self.errors.append(f'Unrecognized context var in {src}: {ckey}')
        elif ctype.startswith('enums::'):
            if cval not in self.context_str_values[ckey]:
                self.errors.append(f'Invalid context enum value in {src}: {ckey}: {cval}')
        else:
            if typenameof(cval) != ctype:
                self.errors.append(f'Invalid context value in {src}: {ckey} expects {ctype} but {cval} is {typenameof(cval)}')

    def _check_setting(self, setting: str, val: Any, src: str):
        s = self.settings.get(setting)
        if not s:
            self.errors.append(f'Unrecognized setting in {src}: {setting}')
            return
        stype = s.get('rusttype')
        if not stype:
            return  # Error already reported
        if typenameof(val) != stype:
            self.errors.append(f'Invalid value of setting {setting} in {src}: expected {stype} but got {typenameof(val)}')
        elif m := s.get('max') and val > m:
            self.errors.append(f'Value too large for setting {setting} in {src}: max is {m} but got {val}')
        elif opts := s.get('opts') and val not in opts:
            self.errors.append(f'Unrecognized value for setting {setting} in {src}: expected one of {", ".join(opts)} but got {val}')

    def _check_spot(self, spot: str, src: str):
        if spot.count('>') != 2:
            self.errors.append(f'Invalid spot in {src}: {spot}')
        elif construct_id(spot) not in self.id_lookup:
            self.errors.append(f'Unrecognized spot in {src}: {spot}')

    def _check_leaf(self, type: str, leaf: str, src: str):
        if leaf.count('>') != 3:
            self.errors.append(f'Invalid {type} in {src}: {leaf}')
            return
        leaf_id = construct_id(leaf)
        if leaf_id not in self.id_lookup:
            self.errors.append(f'Unrecognized {type} in {src}: {leaf}')
            return
        if spot := self.id_lookup.get(construct_id(leaf[:leaf.rfind(' > ')])):
            othertype = 'location' if type == 'action' else 'action'
            my_id, o_id = 'loc_ids', 'action_ids'
            if type == 'action':
                my_id, o_id = o_id, my_id
            if leaf_id in spot[o_id] and leaf_id not in spot[my_id]:
                self.errors.append(f'Provided {othertype} but expected {type} in {src}: {leaf}')