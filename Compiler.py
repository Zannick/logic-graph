import argparse
from collections import namedtuple, Counter, defaultdict
from functools import cache, cached_property, partial
import itertools
import logging
import math
import os
import re
import subprocess
import sys
import yaml
# TODO: pyspellchecker to check for issues with item names

logging.basicConfig(level=logging.INFO, format='{relativeCreated:09.2f} {levelname}: {message}', style='{')

import antlr4
import inflection
import jinja2

from grammar import parseRule, parseAction, ParseResult
from grammar.visitors import *
from TestProcessor import TestProcessor
from Utils import *

templates_dir = os.path.join(base_dir, 'games', 'templates')

MAIN_FILENAME = 'Game.yaml'
GAME_FIELDS = {'name', 'objectives', 'movements', 'warps', 'actions', 'time',
               'start', 'load', 'helpers', 'collect', 'settings', '_filename'}
REGION_FIELDS = {'name', 'short', 'here'}
AREA_FIELDS = {'name', 'enter', 'exits', 'spots', 'here'}
SPOT_FIELDS = {'name', 'coord', 'actions', 'locations', 'exits', 'hybrid', 'local'}
LOCATION_FIELDS = {'name', 'item', 'req', 'canon'}
SETTING_FIELDS = {'type', 'max', 'opts', 'default'}
MOVEMENT_DIMS = {'free', 'xy', 'x', 'y'}
TRIGGER_RULES = {'enter', 'load', 'reset'}

ON_ENTRY_ARGS = {'newpos': 'SpotId'}

typed_name = re.compile(r'(?P<name>\$?[\w\s]+)(?::(?P<type>\w+))?')
TypedVar = namedtuple('TypedVar', ['name', 'type'])


def load_data_from_file(file: str):
    try:
        with open(file) as f:
            res = list(yaml.safe_load_all(f))
            for r in res:
                r['_filename'] = os.path.basename(file)
            return res
    except Exception as e:
        raise Exception(f'Error reading from {file}') from e


def load_game_yaml(game_dir: str):
    yfiles = [file for file in os.listdir(game_dir)
              if file.endswith('.yaml')]
    game_file = os.path.join(game_dir, MAIN_FILENAME)
    if MAIN_FILENAME not in yfiles:
        raise Exception(f'Game not found: expecting {game_file}')
    yfiles.remove(MAIN_FILENAME)
    with open(os.path.join(game_dir, MAIN_FILENAME)) as f:
        game = yaml.safe_load(f)
    unexp = game.keys() - GAME_FIELDS
    if unexp:
        raise Exception(f'Unexpected top-level fields in {game_file}: {", ".join(sorted(unexp))}')
    game['regions'] = list(itertools.chain.from_iterable(
        load_data_from_file(os.path.join(game_dir, file))
        for file in sorted(yfiles)))
    test_dir = os.path.join(game_dir, 'tests')
    if os.path.exists(test_dir):
        testfiles = [file for file in os.listdir(test_dir)
                     if file.endswith('.yaml')]
        game['tests'] = list(itertools.chain.from_iterable(
            load_data_from_file(os.path.join(test_dir, file))
            for file in sorted(testfiles)))
    return game

def _parseExpression(logic: str, name: str, category: str, sep:str=':') -> ParseResult:
    rule = 'boolExpr'
    # TODO: turn the whole thing into a regex
    if m := typed_name.match(name):
        rule = m.group('type') or rule
        name = m.group('name')
    return parseRule(rule, logic, name=f'{category}{sep}{name}')


def get_func_rule(helper_key:str) -> str:
    if m := typed_name.match(helper_key):
        return m.group('type') or 'boolExpr'
    return 'boolExpr'


def get_func_name(helper_key: str) -> str:
    if m := typed_name.match(helper_key):
        return m.group('name')
    return helper_key

def get_arg_with_type(arg: str) -> str:
    if m := typed_name.match(arg):
        return TypedVar(m.group('name'), m.group('type'))
    return TypedVar(arg, '')

def get_func_args(helper_key: str) -> list[str]:
    if '(' in helper_key:
        return [get_arg_with_type(arg) for arg in helper_key[:-1].split('(', 1)[1].split(',')]
    return []


def get_int_type_for_max(count: int) -> str:
    if count == 1:
        return 'bool'
    if count < 128:
        return 'i8'
    if count < 32768:
        return 'i16'
    return 'i32'


def trim_type_prefix(s: str) -> str:
    if '::' in s:
        return s[s.index('::') + 2:]
    return s


def str_to_rusttype(val: str, t: str) -> str:
    if t.startswith('enums::'):
        return f'{t}::{inflection.camelize(val)}'
    if isinstance(val, str) and '::' in val:
        return f'{t}::{trim_type_prefix(val)}'
    if 'Id' in t:
        return f'{t}::{construct_id(val)}'
    if t == 'bool':
        return str(val).lower()
    return val


def treeToString(tree: antlr4.ParserRuleContext):
    return StringVisitor().visit(tree)


def get_spot_reference(target, source):
    local = [source['region'], source['area'],
             source.get('spot') or source.get('name')]
    targ = target.split('>')
    # targ length = 1 (just spot) => leave 2 (reg/area), 2 (spot+area) => leave 1 (region)
    # 3 => 0.
    res = local[:-len(targ)] + [t.strip() for t in targ]
    return construct_id(*res)

def get_exit_target(ex):
    return get_spot_reference(ex['to'], ex)

class GameLogic(object):

    def __init__(self, game: str):
        self.game = game
        self.package = inflection.underscore(game)
        self.game_dir = os.path.join(base_dir, 'games', game)
        self._errors = []

        self._info = load_game_yaml(self.game_dir)
        self.tests = self._info.get('tests', [])
        self.game_name = self._info['name']
        self.helpers = {
            get_func_name(name): {
                'args': get_func_args(name),
                'pr': _parseExpression(logic, name, 'helpers'),
                'rule': get_func_rule(name),
            }
            for name, logic in self._info['helpers'].items()
        }

        self.allowed_funcs = self.helpers.keys() | BUILTINS.keys()
        self.access_funcs = {}
        self.action_funcs = {}
        self.objectives = {}
        for name, logic in self._info['objectives'].items():
            pr = _parseExpression(logic, name, 'objectives')
            self.objectives[name] = {'pr': pr}
            self.objectives[name]['access_id'] = self.make_funcid(self.objectives[name])
        self.default_objective = list(self._info['objectives'].keys())[0]

        self.collect = {}
        for name, logic in self._info.get('collect', {}).items():
            pr = parseAction(logic, 'collect:' + name)
            self.collect[name] = {'act': pr}
            self.collect[name]['action_id'] = self.make_funcid(self.collect[name], 'act')

        # these are {name: {...}} dicts
        self.movements = self._info['movements']
        self.time = self._info['time']
        if 'default' not in self.movements:
            self._errors.append(f'No default movement defined')
        for name, info in self.movements.items():
            if 'req' in info:
                info['pr'] = _parseExpression(info['req'], name, 'movements')
                info['access_id'] = self.make_funcid(info)
                if name == 'default':
                    self._errors.append(f'Cannot define req for default movement')

        self.id_lookup = {}
        self.process_regions()
        self.process_warps()
        self.process_global_actions()
        self._errors.extend(itertools.chain.from_iterable(pr.errors for pr in self.all_parse_results()))

        self.process_times()
        self.process_settings()
        self.process_tests()


    def process_regions(self):
        self.canon_places = defaultdict(list)
        # regions/areas/etc are dicts {name: blah, req: blah} (at whatever level)
        self.regions = self._info['regions']
        for region in self.regions:
            rname = region.get('short', region['name'])
            region['id'] = construct_id(rname)
            self.id_lookup[region['id']] = region
            region['loc_ids'] = []
            if 'on_entry' in region:
                region['act'] = parseAction(
                        region['on_entry'], name=f'{region["fullname"]}:on_entry')
                region['action_id'] = self.make_funcid(region, 'act', ON_ENTRY_ARGS)
            for area in region['areas']:
                aname = area['name']
                area['region'] = rname
                area['id'] = construct_id(rname, aname)
                self.id_lookup[area['id']] = area
                area['fullname'] = f'{rname} > {aname}'
                area['spot_ids'] = []
                area['loc_ids'] = []
                if 'on_entry' in area:
                    area['act'] = parseAction(
                            area['on_entry'], name=f'{area["fullname"]}:on_entry')
                    area['action_id'] = self.make_funcid(area, 'act', ON_ENTRY_ARGS)

                for spot in area['spots']:
                    sname = spot['name']
                    spot['area'] = aname
                    spot['region'] = rname
                    spot['id'] = construct_id(rname, aname, sname)
                    self.id_lookup[spot['id']] = spot
                    spot['fullname'] = f'{rname} > {aname} > {sname}'
                    area['spot_ids'].append(spot['id'])
                    spot['loc_ids'] = []
                    spot['exit_ids'] = []
                    spot['action_ids'] = []
                    # hybrid spots are exits but have names
                    for loc in spot.get('locations', []) + spot.get('hybrid', []):
                        loc['spot'] = sname
                        loc['area'] = aname
                        loc['region'] = rname
                        loc['id'] = construct_id(rname, aname, sname, loc['name'])
                        self.id_lookup[loc['id']] = loc
                        spot['loc_ids'].append(loc['id'])
                        area['loc_ids'].append(loc['id'])
                        region['loc_ids'].append(loc['id'])
                        loc['fullname'] = f'{spot["fullname"]}: {loc["name"]}'
                        if 'canon' in loc:
                            self.canon_places[loc['canon']].append(loc)
                        if 'req' in loc:
                            loc['pr'] = _parseExpression(
                                    loc['req'], loc['name'], spot['fullname'], ': ')
                            loc['access_id'] = self.make_funcid(loc)
                    # We need a counter for exits in case of alternates
                    ec = Counter()
                    for eh in spot.get('exits', []):
                        eh['spot'] = sname
                        eh['area'] = aname
                        eh['region'] = rname
                        ec[eh['to']] += 1
                        eh['id'] = construct_id(rname, aname, sname, 'ex',
                                                f'{eh["to"]}_{ec[eh["to"]]}')
                        self.id_lookup[eh['id']] = eh
                        spot['exit_ids'].append(eh['id'])
                        eh['fullname'] = f'{spot["fullname"]} ==> {eh["to"]} ({ec[eh["to"]]})'
                        if 'req' in eh:
                            eh['pr'] = _parseExpression(
                                    eh['req'], eh['to'], spot['fullname'], ' ==> ')
                            eh['access_id'] = self.make_funcid(eh)
                    for act in spot.get('actions', ()):
                        act['spot'] = sname
                        act['area'] = aname
                        act['region'] = rname
                        act['id'] = construct_id(rname, aname, sname, act['name'])
                        self.id_lookup[act['id']] = act
                        spot['action_ids'].append(act['id'])
                        act['fullname'] = f'{spot["fullname"]}: {act["name"]}'
                        if 'req' in act:
                            act['pr'] = _parseExpression(
                                    act['req'], act['name'] + ' req', spot['fullname'], ': ')
                            act['access_id'] = self.make_funcid(act)
                        act['act'] = parseAction(
                                act['do'], name=f'{act["fullname"]}:do')
                        act['action_id'] = self.make_funcid(act, 'act')


    def process_times(self):
        for point in self.all_points():
            if 'time' not in point:
                point['time'] = max(
                        (v for k,v in self.time.items() if k in point.get('tags', [])),
                        default=self.time['default'])
            if 'item' in point and 'to' in point and 'item_time' not in point:
                point['item_time'] = max(
                        (v for k,v in self.time.items() if k in point.get('tags', [])),
                        default=self.time['default'])


    def process_warps(self):
        self.warps = self._info['warps']
        for name, info in self.warps.items():
            info['name'] = inflection.camelize(name)
            info['id'] = construct_id(info['name'])
            if 'time' not in info:
                self._errors.append(f'Warp {name} requires explicit "time" setting')
            if info['to'].startswith('^'):
                val = info['to'][1:]
                if val not in self.context_types:
                    self._errors.append(f'Warp {name} goes to undefined ctx dest: ^{val}')
                elif self.context_types[val] != 'SpotId':
                    self._errors.append(f'Warp {name} goes to invalid ctx dest: ^{val} (of type {self.context_types[val]})')
                info['target_id'] = f'ctx.{val}()'
            else:
                id = construct_id(info['to'])
                if not any(info['id'] == id for info in self.spots()):
                    self._errors.append(f'Warp {name} goes to unrecognized spot: {info["to"]}')
                info['target_id'] = 'SpotId::' + id
            if 'req' in info:
                info['pr'] = _parseExpression(info['req'], name, 'warps')
                info['access_id'] = self.make_funcid(info)
            if 'do' in info:
                info['act'] = parseAction(
                        info['do'], name=f'{info["name"]}:do')
                info['action_id'] = self.make_funcid(info, 'act')


    def process_global_actions(self):
        self.global_actions = self._info.get('actions', [])
        for act in self.global_actions:
            name = act['name']
            act['id'] = construct_id('Global', name)
            if 'req' not in act and 'price' not in act:
                self._errors.append(f'Global actions must have req or price: {name}')
            elif 'req' in act:
                act['pr'] = _parseExpression(
                        act['req'], name + ' req', 'actions', ': ')
                act['access_id'] = self.make_funcid(act)
            act['act'] = parseAction(
                    act['do'], name=f'{name}:do')
            act['action_id'] = self.make_funcid(act, 'act')


    def process_settings(self):
        # Check settings
        def _visit(visitor, reverse=False):
            if not reverse:
                for pr in self.nonpoint_parse_results():
                    visitor.visit(pr.tree, pr.name, self.get_default_ctx())
            for pt in self.all_points():
                if 'pr' in pt:
                    visitor.visit(pt['pr'].tree, pt['pr'].name, self.get_local_ctx(pt))
                if 'act' in pt:
                    visitor.visit(pt['act'].tree, pt['act'].name, self.get_local_ctx(pt))
            if reverse:
                for pr in self.nonpoint_parse_results():
                    visitor.visit(pr.tree, pr.name, self.get_default_ctx())
            self._errors.extend(visitor.errors)

        sv = SettingVisitor(self.context_types, self.settings)
        _visit(sv)
        self.used_settings = sv.setting_options

        for s in self.settings.keys() - self.used_settings.keys():
            logging.warning(f'Did not find usage of setting {s}')

        hv = HelperVisitor(self.helpers, self.context_types, self.settings)
        _visit(hv, True)

        cv = ContextVisitor(self.context_types, self.context_values)
        _visit(cv)
        self.context_str_values = cv.values

    def process_tests(self):
        tp = TestProcessor(self.all_items, self.context_types, self.context_str_values,
                           self.settings, self.id_lookup)
        self._errors.extend(tp.process_tests(self.tests))


    @cached_property
    def movements_by_type(self):
        d = defaultdict(list)
        for m, info in self.movements.items():
            found = False
            # 'x' and 'y' can be on the same movement
            for mt in MOVEMENT_DIMS:
                if mt in info:
                    d[mt].append(m)
                    found = True
            if not found:
                self._errors.append(f'Movement {m} does not define a movement dimension: '
                                         f'must be one of {", ".join(MOVEMENT_DIMS)}')

            if m != 'default' and 'req' not in info:
                self._errors.append(f'Movement {m} must have a req')
        return d


    def movement_time(self, mset, a, b, jumps=0, jumps_down=0):
        times = []
        xtimes = []
        ytimes = []
        for m in mset + ('default',):
            if s := self.movements[m].get('free'):
                times.append(math.sqrt(a**2 + b**2) / s)
                continue
            if s := self.movements[m].get('xy'):
                times.append((abs(a) + abs(b)) / s)
                continue
            if sx := self.movements[m].get('x'):
                xtimes.append(abs(a) / sx)
            # x, y, fall: not mutually exclusive
            if sy := self.movements[m].get('y'):
                ytimes.append(abs(b) / sy)
            if sfall := self.movements[m].get('fall'):
                # fall speed must be the same direction as "down"
                if (t := b / sfall) > 0:
                    t += jumps_down * self.movements[m].get('jump_down', 0)
                    ytimes.append(t)
                elif jumps and t < 0 and (sjump := self.movements[m].get('jump')):
                    # Direction is negative but jumps is just time taken
                    ytimes.append(jumps * sjump)
        if xtimes and ytimes:
            times.append(max(min(xtimes), min(ytimes)))
        elif xtimes and b == 0:
            times.append(min(xtimes))
        elif ytimes and a == 0:
            times.append(min(ytimes))
        return min(times, default=None)


    @cached_property
    def movement_sets(self):
        # Possible relevant movement sets:
        # 1. any movement on its own
        # 2. any 'x' or 'x+y' with any 'y' or 'x+y'
        # -- free and xy are not compatible with x and y alone (could they be?)
        # All movement sets:
        # - any combination of available movements (2^n) only needs to consider these subsets
        #   to find which is the best option for any travel between two points
        # for a distance of (a,b):
        # - free: sqrt(a**2 + b**2)/s
        # - xy: (a+b)/s
        # - x+y: max(a/s_x, b/s_y)
        # But is it consistent for all travel?
        # - obviously the fastest free is faster than other frees, etc.
        # - if s_free > s_xy then free is always faster than xy. This should also be true
        #   at lower s_free but it becomes dependent on (a,b); so the answer is no overall.
        s = {(m['name'],) for m in self.movements}
        for xm in self.movements_by_type.get('x', []):
            for ym in self.movement_by_type.get('y', []):
                s.add((xm, ym))
        return s


    @cached_property
    def non_default_movements(self):
        return sorted(m for m in self.movements if m != 'default')


    def spot_distance(self, sp1, sp2):
        coords = [sp1['coord'], sp2['coord']]
        jumps = [0]
        jumps_down = [0]
        for lcl in sp1.get('local', []):
            if lcl.get('to') == sp2['name']:
                # We could have more overrides here, like dist
                if thru := lcl.get('thru'):
                    if isinstance(thru, str):
                        self._errors.append(f'Invalid thru from {sp1["name"]} to {sp2["name"]}: {thru!r} '
                                                    f'(Did you mean [{thru}] ?)')
                        break
                    if not isinstance(thru, list) or not thru:
                        self._errors.append(f'Invalid thru from {sp1["name"]} to {sp2["name"]}: {thru}')
                        break
                    if all(isinstance(t, list) for t in thru):
                        coords[1:1] = thru
                    elif len(thru) == 2 and all(isinstance(t, (int, float)) for t in thru):
                        coords[1:1] = [thru]
                    else:
                        self._errors.append(f'Mismatched length or types in thru '
                                                    f'from {sp1["name"]} to {sp2["name"]}: {thru}')
                        break
                if j := lcl.get('jumps'):
                    if isinstance(j, str):
                        self._errors.append(f'Invalid jumps from {sp1["name"]} to {sp2["name"]}: {j!r} '
                                                    f'(Did you mean [{j}] ?)')
                        break
                    if not isinstance(j, list):
                        j = [j]
                    if len(j) != len(coords) - 1:
                        self._errors.append(f'Jumps list from {sp1["name"]} to {sp2["name"]} '
                                                    f'must match path length 1+thru = {len(coords) - 1} but was {len(j)}')
                        break
                    jumps[:] = j
                else:
                    jumps *= len(coords) - 1
                if j := lcl.get('jumps_down'):
                    if isinstance(j, str):
                        self._errors.append(f'Invalid jumps from {sp1["name"]} to {sp2["name"]}: {j!r} '
                                                    f'(Did you mean [{j}] ?)')
                        break
                    if not isinstance(j, list):
                        j = [j]
                    if len(j) != len(coords) - 1:
                        self._errors.append(f'Jumps_down list from {sp1["name"]} to {sp2["name"]} '
                                                    f'must match path length 1+thru={len(coords) - 1}: {len(j)}')
                        break
                    jumps_down[:] = j
                else:
                    jumps_down *= len(coords) - 1
                break
            
        else:
            # spots must explicitly declare connections
            return ([], [], [])
        
        return (coords, jumps, jumps_down)
        

    @cached_property
    def local_distances(self):
        # create a distances table: (spot, spot) -> [(x, y), ...]
        d = defaultdict(list)
        for a in self.areas():
            errors = []
            for sp in a['spots']:
                if c := sp.get('coord'):
                    if isinstance(c, str):
                        errors.append(f'Invalid coord for {sp["fullname"]}: {c!r} '
                                      f'(did you mean [{c}] ?)')
                    elif not isinstance(c, (list, tuple)) or len(c) != 2:
                        errors.append(f'Invalid coord for {sp["fullname"]}: {c}')
                elif sp.get('local'):
                    errors.append(f'Expected coord for spot {sp["fullname"]} with local rules')
            if errors:
                self._errors.extend(errors)
                break
            spot_errors = set()
            for sp1, sp2 in itertools.permutations(a['spots'], 2):
                if 'coord' not in sp1 or 'local' not in sp1:
                    continue
                if 'coord' not in sp2:
                    if sp2['name'] not in spot_errors and any(link["to"] == sp2['name'] for link in sp1['local']):
                        spot_errors.add(sp2['name'])
                        self._errors.append(f'Expected coord for spot {sp["fullname"]} used in local rules')
                    continue
                coords, jumps, jumps_down = self.spot_distance(sp1, sp2)
                if not coords:
                    continue
                for ((sx, sy), (cx, cy)), j, jd in zip(itertools.pairwise(coords), jumps, jumps_down):
                    d[(sp1['id'], sp2['id'])].append(
                            (abs(cx - sx), cy - sy, j, jd))
        return d


    @cached_property
    def movement_tables(self):
        # create a movement table for each movement "combo"
        # (generally we'll use only 1 free, or 1 xy, or 1x, or 1x+1y, at a time,
        #  but we can't guarantee which is best for all situations.
        #  It might be simplest to determine which movements we have available in
        #  the area we're in, and then look up the travel time from that.)
        table = {}
        for mset in itertools.chain.from_iterable(
                itertools.combinations(self.non_default_movements, r)
                for r in range(0, len(self.non_default_movements) + 1)):
            key = tuple(m in mset for m in self.non_default_movements)
            table[key] = local_time = {}
            for k, dlist in self.local_distances.items():
                times = [self.movement_time(mset, a, b, j, jd) for a,b, j, jd in dlist]
                if all(t is not None for t in times):
                    local_time[k] = times
        return table


    def make_funcid(self, info, prkey:str='pr', extra_fields=None):
        pr = info[prkey]
        d = self.action_funcs if pr.parser.ruleNames[pr.tree.getRuleIndex()] == 'actions' else self.access_funcs
        if '^_' in str(pr.text):
            id = construct_id(str(pr.name).lower())
            assert id not in d
            d[id] = info
            if extra_fields:
                d[id]['args'] = extra_fields
            return id

        id = construct_id(str(pr.name) if '^_' in str(pr.text) else str(pr.text)).lower()
        if id not in d:
            d[id] = {prkey: info[prkey]}
            if extra_fields:
                d[id]['args'] = extra_fields
            return id

        if d[id][prkey].text != pr.text:
            id = id + sum(1 for k in d if k.startswith(id))
            assert id not in d
            d[id] = {prkey: info[prkey]}
            if extra_fields:
                d[id]['args'] = extra_fields
        return id


    def areas(self):
        return itertools.chain.from_iterable(r['areas'] for r in self.regions)


    def spots(self):
        return itertools.chain.from_iterable(a['spots'] for a in self.areas())


    # Hybrids are both locations and exits, so they have to be returned here
    # for both in order to create the appropriate ids.
    def locations(self):
        return itertools.chain.from_iterable(s.get('locations', []) + s.get('hybrid', [])
                                             for s in self.spots())


    def exits(self):
        return itertools.chain.from_iterable(s.get('exits', []) + s.get('hybrid', [])
                                             for s in self.spots())


    def actions(self):
        return itertools.chain(
            self.global_actions,
            itertools.chain.from_iterable(s.get('actions', []) for s in self.spots()))


    def all_points(self):
        for region in self.regions:
            for area in region['areas']:
                for spot in area['spots']:
                    yield from spot.get('locations', ())
                    yield from spot.get('exits', ())
                    yield from spot.get('hybrid', ())
                    yield from spot.get('actions', ())


    def nonpoint_parse_results(self):
        yield from (info['pr'] for info in self.helpers.values())
        yield from (info['pr'] for info in self.objectives.values())
        yield from (info['act'] for info in self.collect.values())
        yield from (info['pr'] for info in self.movements.values() if 'pr' in info)
        yield from (info['pr'] for info in self.warps.values() if 'pr' in info)
        yield from (info['pr'] for info in self.global_actions if 'pr' in info)
        yield from (info['act'] for info in self.global_actions)


    def all_parse_results(self):
        yield from self.nonpoint_parse_results()
        for pt in self.all_points():
            if 'pr' in pt:
                yield pt['pr']
            if 'act' in pt:
                yield pt['act']


    @cached_property
    def all_connections(self):
        conns = set()
        for spot in self.spots():
            for ex in spot.get('exits', ()):
                conns.add((spot['id'], get_exit_target(ex)))
            for hybrid in spot.get('hybrid', ()):
                conns.add((spot['id'], get_exit_target(hybrid)))
        return conns
    

    @cached_property
    def adjacent_regions(self):
        conns = set()
        for r in self.regions:
            for a in r['areas']:
                for s in a['spots']:
                    for ex in s.get('exits', ()):
                        target = ex['to'].split('>')
                        if len(target) == 3:
                            t = target[0].strip()
                            if r['id'] < t:
                                conns.add((r['id'], t))
                            else:
                                conns.add((t, r['id']))
        return conns


    @cached_property
    def settings(self):
        sd = self._info.get('settings', {})

        def _apply_override(s, t, info, text):
            if declared := info.get('type'):
                if declared != t:
                    logging.warning(f'Setting {s} type {declared} overridden by {text} ({t})')
            info['type'] = t

        for s, info in sd.items():
            if disallowed := info.keys() - SETTING_FIELDS:
                self._errors.append(f'Unrecognized setting fields on setting {s}: {", ".join(disallowed)}')
                continue
            if m := info.get('max', 0):
                t = config_type(m)
                _apply_override(s, t, info, f'max: {m}')
                if t == 'int':
                    info['rust_type'] = get_int_type_for_max(m)
            elif opts := info.get('opts', ()):
                t, *types = {config_type(o) for o in opts}
                if types:
                    self._errors.append(f'Setting {s} options are mixed types: {t}, {", ".join(types)}')
                    continue
                _apply_override(s, t, info, f'opts, e.g. {opts[0]}')
                if t == 'int':
                    info['rust_type'] = get_int_type_for_max(max(opts))
            elif 'type' not in info:
                self._errors.append(f'Setting {s} must declare one of: type, max, opts')
                continue
            if 'rust_type' not in info:
                info['rust_type'] = ctx_types.get(info['type'], info['type'])

        return sd


    def check_all(self):
        # Check vanilla items
        for pt in self.all_points():
            if 'item' in pt and pt['item'] is None:
                self._errors.append(f'{pt["id"]} specified with empty item')
            elif 'item' in pt and pt['item'] != construct_id(pt['item']):
                self._errors.append(f'Invalid item name {pt["item"]!r} at {pt["id"]}; '
                         f'did you mean {construct_id(pt["item"])!r}?')
        # Check used functions
        for func in BUILTINS.keys() & self.helpers.keys():
            self._errors.append(f'Cannot use reserved name {func!r} as helper')
        for pr in self.all_parse_results():
            for t in pr.parser.getTokenStream().tokens:
                if pr.parser.symbolicNames[t.type] == 'FUNC' and t.text not in self.allowed_funcs:
                    self._errors.append(f'{pr.name}: Unrecognized function {t.text}')
        # Check exits
        spot_ids = {sp['id'] for sp in self.spots()}
        for ex in self.exits():
            if get_exit_target(ex) not in spot_ids:
                self._errors.append(f'Unrecognized destination spot in exit {ex["fullname"]}')
        for item in self.collect:
            if item != construct_id(item):
                self._errors.append(f'Invalid item name {item!r} as collect rule; '
                         f'did you mean {construct_id(item)!r}?')
            elif item not in self.all_items:
                self._errors.append(f'Unrecognized item {item!r} as collect rule')

    @cached_property
    def errors(self):
        # Do things that will fill _errors
        self.check_all()
        self.context_values
        self.local_distances
        self.context_resetters
        self.item_stats

        return self._errors


    @cached_property
    def vanilla_items(self):
        return {pt['item'] for pt in self.all_points()
                if 'item' in pt}


    @cached_property
    def rule_items(self):
        return {t.text
                for pr in self.all_parse_results()
                for t in pr.parser.getTokenStream().tokens
                if pr.parser.symbolicNames[t.type] == 'ITEM'}


    @cached_property
    def all_items(self):
        return sorted(self.vanilla_items | self.rule_items)


    @cached_property
    def item_stats(self):
        visitor = ItemVisitor(self.settings, self.vanilla_items)
        for pr in self.all_parse_results():
            visitor.visit(pr.tree, name=pr.name)
        self._errors.extend(visitor.errors)
        return visitor.item_uses, visitor.item_max_counts


    def item_uses(self):
        return self.item_stats[0]


    def item_max_counts(self):
        return self.item_stats[1]


    @cached_property
    def unused_items(self):
        return self.all_items - self.item_max_counts().keys() - self.collect.keys()


    @cached_property
    def context_values(self):
        def _check_types(v1, v2, ctx, *names, here=False):
            t1 = typenameof(v1)
            t2 = typenameof(v2)
            # here overrides may look like a higher-order place type
            if here:
                if t1 == 'SpotId' and t2 in ('AreaId', 'RegionId'):
                    return
                if len(names) == 1 and t1 == 'AreaId' and t2 == 'RegionId':
                    return
            if t1 != t2:
                self._errors.append(
                    f'context value type mismatch: {ctx} defined as {v1} ({t1}) '
                    f'and reused in {" > ".join(names)} as {v2} ({t2})')
        
        # self._info: start
        # regions/areas: here, start, enter
        gc = dict(self._info['start'])
        def _check_shadow(ctx, *names):
            if len(names) == 2:
                pc = construct_id(names[0], 'ctx', ctx[1:]).lower()
                if pc in gc:
                    self._errors.append(
                        f'Context parameter {ctx} in {" > ".join(names)} hides '
                        f'parameter {ctx} in {names[0]}')

        def _handle_start(ctx, val, *names):
            if ctx[0] == '_':
                _check_shadow(ctx, *names)
                ctx = construct_id(*names, 'ctx', ctx[1:]).lower()
            if ctx in gc:
                self._errors.append(
                    f'Duplicate context parameter {ctx} in {" > ".join(names)}: '
                    'not allowed in "start" section')
            else:
                gc[ctx] = val

        def _handle_triggers(ctx, val, *names):
            if ctx[0] == '_':
                _check_shadow(ctx, *names)
                ctx = construct_id(*names, 'ctx', ctx[1:]).lower()
            if ctx in gc:
                _check_types(gc[ctx], val, ctx, *names)
            else:
                gc[ctx] = val

        def _handle_here(ctx, val, *names):
            if ctx[0] == '_':
                self._errors.append(
                    f'"here" overrides cannot be local: {" > ".join(names)} {ctx}')
            elif ctx not in gc:
                self._errors.append(
                    f'"here" overrides must be predefined: {" > ".join(names)} {ctx}')
            else:
                _check_types(gc[ctx], val, ctx, *names, here=True)

        for region in self.regions:
            for ctx, val in region.get('start', {}).items():
                _handle_start(ctx, val, region['name'])
            for ctx, val in itertools.chain.from_iterable(
                    region.get(trigger, {}).items()
                    for trigger in TRIGGER_RULES):
                _handle_triggers(ctx, val, region['name'])
            for ctx, val in region.get('here', {}).items():
                _handle_here(ctx, val, region['name'])
        # Areas must be handled second to check for shadowing
        for area in self.areas():
            for ctx, val in area.get('start', {}).items():
                _handle_start(ctx, val, area['region'], area['name'])
            for ctx, val in itertools.chain.from_iterable(
                    area.get(trigger, {}).items()
                    for trigger in TRIGGER_RULES):
                _handle_triggers(ctx, val, area['region'], area['name'])
            for ctx, val in area.get('here', {}).items():
                _handle_here(ctx, val, area['region'], area['name'])

        return gc


    @cached_property
    def context_types(self):
        d = {'position': 'SpotId'}
        for ctx, val in self.context_values.items():
            t = typenameof(val)
            if t == 'ENUM':
                t = 'enums::' + ctx.capitalize()
            d[ctx] = t
        return d

    def get_default_ctx(self):
        return {c: c for c in self.context_values if '__ctx__' not in c}

    def get_local_ctx(self, info):
        d = self.get_default_ctx()
        if 'region' not in info:
            return d
        area = info.get('area') or info['name']

        levels = [construct_id(info['region']).lower(),
                  construct_id(info['region'], area).lower()]
        for cname in self.context_values:
            if '__ctx__' not in cname:
                continue

            pref, local = cname.split('__ctx_', 1)  # intentionally leave one _ in
            if pref in levels:
                d[local] = cname
        return d

    @cached_property
    def context_here_overrides(self):
        d = {c: {'region': defaultdict(dict), 'area': defaultdict(dict)}
             for c in self.context_types}
        for r in self.regions:
            localctx = self.get_local_ctx(r)
            if here := r.get('here'):
                for k, v in here.items():
                    t = self.context_types[localctx[k]]
                    if t == 'SpotId':
                        v = f'{r["name"]} > {v}'
                    d[localctx[k]]['region'][r['id']] = str_to_rusttype(v, t)
        for a in self.areas():
            localctx = self.get_local_ctx(a)
            if here := a.get('here'):
                for k, v in here.items():
                    t = self.context_types[localctx[k]]
                    if t == 'SpotId':
                        v = f'{a["fullname"]} > {v}'
                    d[localctx[k]]['area'][a['id']] = str_to_rusttype(v, t)
        return d

    @cached_property
    def context_trigger_rules(self):
        d = {trigger: {'region': defaultdict(dict), 'area': defaultdict(dict)}
             for trigger in TRIGGER_RULES}
        for r in self.regions:
            localctx = self.get_local_ctx(r)
            for trigger in TRIGGER_RULES:
                if e := r.get(trigger):
                    for k, v in e.items():
                        d[trigger]['region'][r['id']][localctx[k]] = str_to_rusttype(v, self.context_types[localctx[k]])
        for a in self.areas():
            localctx = self.get_local_ctx(a)
            for trigger in TRIGGER_RULES:
                if e := a.get(trigger):
                    for k, v in e.items():
                        d[trigger]['area'][a['id']][localctx[k]] = str_to_rusttype(v, self.context_types[localctx[k]])
        return d


    @cached_property
    def context_resetters(self):
        d = {'region': defaultdict(list), 'area': defaultdict(list)}
        for r in self.regions:
            for other_name in r.get('resets', ()):
                if '>' in other_name:
                    self._errors.append(f'Region {r["name"]} may only reset other regions: {other_name!r}')
                    break
                if other_name == r['name']:
                    self._errors.append(f'Use "enter" rule instead of a self-reset in region {r["name"]}')
                    break
                other = construct_id(other_name)
                if other not in self.id_lookup:
                    self._errors.append(f'Unrecognized region in {r["name"]} resets: {other_name!r}')
                d['region'][r['id']].append(other)
        for a in self.areas():
            for other_name in a.get('resets', ()):
                if other_name.count('>') > 1:
                    self._errors.append(f'Area {a["name"]} cannot reset non-Areas: {other_name!r}')
                    break
                if '>' not in other_name:
                    other = construct_id(a['region'], other_name)
                    if other not in self.id_lookup and construct_id(other_name) in self.id_lookup:
                        self._errors.append(f'Area {a["name"]} cannot reset Regions: {other_name!r} '
                                                 f'(would be interpreted as \'{a["region"]} > {other_name}\' if it exists)')
                        break
                else:
                    other = construct_id(other_name)
                if other not in self.id_lookup:
                    self._errors.append(f'Unrecognized area in {a["name"]} resets: {other_name!r}')
                    break
                d['area'][a['id']].append(other)
        return d

    @cached_property
    def context_position_watchers(self):
        d = {'region': set(), 'area': set()}
        d['region'].update(self.context_trigger_rules['enter']['region'].keys())
        d['area'].update(self.context_trigger_rules['enter']['area'].keys())
        d['region'].update(self.context_resetters['region'].keys())
        d['area'].update(self.context_resetters['area'].keys())
        d['region'].update(r['id'] for r in self.regions if 'act' in r)
        d['area'].update(a['id'] for a in self.areas() if 'act' in a)
        return d

    
    @cached_property
    def price_types(self):
        return [ctx for ctx, val in self._info['start'].items()
                if typenameof(val) == 'i32']


    def prToRust(self, pr, info, id=None):
        return RustVisitor(self.context_types,
                           self.action_funcs,
                           self.get_local_ctx(info),
                           id or pr.name).visit(pr.tree)


    def actToHasEffect(self, pr, info, id=None):
        return ActionHasEffectVisitor(self.context_types,
                                      self.action_funcs,
                                      self.get_local_ctx(info),
                                      id or pr.name).visit(pr.tree)


    def render(self):
        env = jinja2.Environment(loader=jinja2.FileSystemLoader(templates_dir),
                                 line_statement_prefix='%%',
                                 line_comment_prefix='%#')
        env.filters.update({
            'actToHasEffect': self.actToHasEffect,
            'camelize': inflection.camelize,
            'construct_id': construct_id,
            'construct_test_name': construct_test_name,
            'escape_ctx': partial(re.compile(r'\bctx\b').sub, '$ctx'),
            'get_exit_target': get_exit_target,
            'get_int_type_for_max': get_int_type_for_max,
            'get_spot_reference': get_spot_reference,
            'prToRust': self.prToRust,
            'str_to_rusttype': str_to_rusttype,
            'treeToString': treeToString,
            'trim_type_prefix': trim_type_prefix,
        })
        # Access cached_properties to ensure they're in the template vars
        self.unused_items
        self.context_types
        self.context_values
        self.price_types
        self.movement_tables
        self.context_trigger_rules
        self.context_position_watchers
        self.context_here_overrides
        self.all_connections
        self.adjacent_regions
        files = {
            '.': ['Cargo.toml'],
            'data': ['digraph.dot', 'digraph.mmd'],
            'src': ['lib.rs', 'items.rs', 'helpers.rs', 'graph.rs', 'context.rs',
                    'prices.rs', 'rules.rs', 'movements.rs', 'settings.rs'],
            'benches': ['bench.rs'],
            'bin': ['main.rs'],
        }
        rustfiles = []
        for dirname, fnames in files.items():
            os.makedirs(os.path.join(self.game_dir, dirname), exist_ok=True)
            for tname in fnames:
                template = env.get_template(tname + '.jinja')
                name = os.path.join(self.game_dir, dirname, tname)
                if name.endswith('.rs'):
                    rustfiles.append(name)
                with open(name, 'w') as f:
                    f.write(template.render(gl=self, **self.__dict__))

        test_template = env.get_template('tests.rs.jinja')
        for test in self.tests:
            name = os.path.join(self.game_dir, 'tests', inflection.underscore(test['name']) + '.rs')
            rustfiles.append(name)
            with open(name, 'w') as f:
                f.write(test_template.render(gl=self, test_file=test, **self.__dict__))

        cmd = ['rustfmt'] + rustfiles
        subprocess.run(cmd)


if __name__ == '__main__':
    cmd = argparse.ArgumentParser()
    cmd.add_argument('game', help='Which game to build the graph for')
    args = cmd.parse_args()

    gl = GameLogic(args.game)
    if gl.errors:
        print('\n'.join(gl.errors))
        print(f'Encountered {len(gl.errors)} error(s); exiting before codegen.')
        sys.exit(1)
    gl.render()
