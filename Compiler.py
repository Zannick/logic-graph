import argparse
from collections import namedtuple, Counter, defaultdict
from functools import cache, cached_property, partial
import itertools
import logging
import math
import os
import pathlib
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
from FlagProcessor import BitFlagProcessor
from Utils import *

templates_dir = os.path.join(base_dir, 'games', 'templates')

MAIN_FILENAME = 'Game.yaml'
GAME_FIELDS = {'name', 'objectives', 'base_movements', 'movements', 'exit_movements',
               'warps', 'actions', 'time', 'context', 'start', 'load', 'data',
               'rules', 'helpers', 'collect', 'settings', 'special', '_filename'}
REGION_FIELDS = {'name', 'short', 'data', 'graph_offset'}
AREA_FIELDS = {'name', 'enter', 'spots', 'data', 'map'}
SPOT_FIELDS = {'name', 'coord', 'actions', 'locations', 'exits', 'hybrid', 'local', 'data',
               'keep', 'enter'}
LOCATION_FIELDS = {'name', 'item', 'req', 'canon'}
TYPEHINT_FIELDS = {'type', 'max', 'opts', 'default'}
MOVEMENT_DIMS = {'free', 'xy', 'x', 'y'}
TRIGGER_RULES = ['enter', 'load', 'reset']

ON_ENTRY_ARGS = {'newpos': 'SpotId'}

SPOT_NON_FIELDS = {
    inflection.pluralize(n) if n != inflection.pluralize(n) else inflection.singularize(n): n
    for n in SPOT_FIELDS
}

RULES_EXAMPLE = """
rules:
  $victory:
    default: Victory
"""

typed_name = re.compile(r'(?P<name>\$?[^:()]+)(?::(?P<type>\w+))?')
TypedVar = namedtuple('TypedVar', ['name', 'type'])
TypedRule = namedtuple('TypedRule', ['rule', 'args', 'variants'])


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
    if 'rules' not in game or not any(r for r in game['rules'] if r.startswith('$victory')):
        raise Exception(f'Must define top-level field "rules" with "$victory" entry in {game_file}, e.g.\n{RULES_EXAMPLE}')
    game['regions'] = list(itertools.chain.from_iterable(
        load_data_from_file(os.path.join(game_dir, file))
        for file in sorted(yfiles)))
    return game

def _parseExpression(logic: str, name: str, category: str, sep:str=':', rule:str='boolExpr') -> ParseResult:
    # TODO: turn the whole thing into a regex
    if m := typed_name.match(name):
        rule = m.group('type') or rule
        name = m.group('name')
    return parseRule(rule, logic, name=f'{category}{sep}{name}')


def get_func_rule(helper_key:str, default='boolExpr') -> str:
    if m := typed_name.match(helper_key):
        return m.group('type') or default
    return default


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
        return [get_arg_with_type(arg.strip()) for arg in helper_key[:-1].split('(', 1)[1].split(',')]
    return []


def trim_type_prefix(s: str) -> str:
    if '::' in s:
        return s[s.index('::') + 2:]
    return s


def str_to_rusttype(val: str, t: str) -> str:
    if t.startswith('enums::'):
        return f'{t}::{inflection.camelize(val)}'
    if isinstance(val, str) and '::' in val:
        return f'{t}::{trim_type_prefix(val)}'
    if t == 'SpotId':
        return construct_place_id(val)
    if 'Id' in t:
        return f'{t}::{construct_id(val)}'
    if t == 'bool':
        return str(val).lower()
    return val


def treeToString(tree: antlr4.ParserRuleContext):
    return StringVisitor().visit(tree)


def get_spot_reference_names(target, source):
    local = [source['region'], source.get('area') or source.get('name'),
             source.get('spot') or source.get('name')]
    targ = target.split('>')
    # targ length = 1 (just spot) => leave 2 (reg/area), 2 (spot+area) => leave 1 (region)
    # 3 => 0.
    return local[:-len(targ)] + [t.strip() for t in targ]

def get_spot_reference(target, source):
    return construct_id(*get_spot_reference_names(target, source))

def get_map_reference(tilename, source):
    return construct_id('map', *get_spot_reference_names(tilename, source))

def get_exit_target(ex):
    return get_spot_reference(ex['to'], ex)

def get_exit_target_id(ex):
    return construct_spot_id(*get_spot_reference_names(ex['to'], ex))


class GameLogic(object):

    def __init__(self, game: str):
        if '/' in game or '\\' in game:
            path = pathlib.PurePath(game)
            game = path.parts[1] if path.parts[0] == 'games' else path.parts[0]
        self.game = game
        self.package = inflection.underscore(game)
        self.game_dir = os.path.join(base_dir, 'games', game)
        self._errors = []

        self._info = load_game_yaml(self.game_dir)
        self.game_name = self._info['name']
        self.helpers = {
            get_func_name(name): {
                'args': get_func_args(name),
                'pr': _parseExpression(logic, name, 'helpers'),
                'rule': get_func_rule(name),
            }
            for name, logic in self._info.get('helpers', {}).items()
        }
        self.rules = {}
        for key, variants in self._info['rules'].items():
            name = get_func_name(key)
            rule = get_func_rule(key, 'itemList')
            self.rules[name] = TypedRule(rule, (), {
                variant: {
                    'pr': _parseExpression(logic, f'{name}_{variant}', 'rules', rule=rule),
                }
                for variant, logic in variants.items()
            })

        self.allowed_funcs = self.helpers.keys() | self.rules.keys() | BUILTINS.keys()
        self.access_funcs = {}
        self.action_funcs = {}
        for typed_rule in self.rules.values():
            for details in typed_rule.variants.values():
                details['func_id'] = self.make_funcid(details)

        self.collect = {}
        for name, logic in self._info.get('collect', {}).items():
            pr = parseAction(logic, 'collect:' + name)
            self.collect[name] = {'act': pr}
            self.collect[name]['action_id'] = self.make_funcid(self.collect[name], 'act')

        # these are {name: {...}} dicts
        self.base_movements = self._info['base_movements']
        self.movements = self._info.get('movements', {})
        self.exit_movements = self._info.get('exit_movements', {})
        for md in self.base_movements[1:]:
            if 'data' not in md:
                self._errors.append(f'base movements beyond the first must have data restrictions')
        if overlap := self.movements.keys() & self.exit_movements.keys():
            self._errors.append(f'Movement/exit_movement names cannot overlap: {overlap.join(", ")}')
        self.all_movements = dict(self.exit_movements)
        self.all_movements.update(self.movements)

        self.time = self._info['time']
        for name, info in self.movements.items():
            if 'req' in info:
                info['pr'] = _parseExpression(info['req'], name, 'movements')
                info['access_id'] = self.make_funcid(info)
            else:
                self._errors.append(f'movement {name} must have req or be in base_movements/exit_movements')

        self.id_lookup = {}
        self.special = self._info.get('special', {})
        self.data = self._info.get('data', {})
        self.data_defaults = self._info.get('data', {})
        self.map_defs = self._info.get('map', {})
        self.named_spots = set()
        self.process_regions()
        self.process_canon()
        self.process_context()
        self.process_area_maps()
        self.process_warps()
        self.process_global_actions()
        self._errors.extend(itertools.chain.from_iterable(pr.errors for pr in self.all_parse_results()))

        self.process_exit_movements()
        self.process_times()
        self.process_parsed_code()
        self.process_items()
        self.process_bitflags()
        self.process_special()


    def process_regions(self):
        # TODO: move interesting tags to Game.yaml for customization
        interesting_tags = ['interior', 'exterior']

        self.canon_places = defaultdict(list)
        # regions/areas/etc are dicts {name: blah, req: blah} (at whatever level)
        self.regions = self._info['regions']
        num_locs = 0
        for region in self.regions:
            rname = region.get('short', region['name'])
            region['id'] = construct_id(rname)
            self.id_lookup[region['id']] = region
            region['loc_ids'] = []
            region['all_data'] = dict(self.data)
            region['all_data'].update(region.get('data', {}))
            if 'on_entry' in region:
                region['act'] = parseAction(
                        region['on_entry'], name=f'{region["name"]}:on_entry')
                region['action_id'] = self.make_funcid(region, 'act', 'on_entry', ON_ENTRY_ARGS)
            if c := region.get('graph_offset'):
                self._validate_pair(c, f'graph offset for {region["name"]}')
            for area in region['areas']:
                aname = area['name']
                area['region'] = rname
                area['id'] = construct_id(rname, aname)
                self.id_lookup[area['id']] = area
                area['fullname'] = f'{rname} > {aname}'
                area['spot_ids'] = []
                area['loc_ids'] = []
                area['all_data'] = dict(region['all_data'])
                area['all_data'].update(area.get('data', {}))
                if 'on_entry' in area:
                    area['act'] = parseAction(
                            area['on_entry'], name=f'{area["fullname"]}:on_entry')
                    area['action_id'] = self.make_funcid(area, 'act', 'on_entry', ON_ENTRY_ARGS)
                if c := area.get('graph_offset'):
                    self._validate_pair(c, f'graph offset for {area["fullname"]}')

                for spot in area['spots']:
                    sname = spot['name']
                    fullname = f'{rname} > {aname} > {sname}'
                    unexp = spot.keys() - SPOT_FIELDS
                    for uk in unexp:
                        if uk in SPOT_NON_FIELDS:
                            logging.warning(f'Unknown field {uk!r} in {fullname} (did you mean {SPOT_NON_FIELDS[uk]!r}?)')
                        else:
                            logging.warning(f'Unknown field {uk!r} in {fullname}')

                    spot['area'] = aname
                    spot['region'] = rname
                    spot['id'] = construct_id(rname, aname, sname)
                    self.id_lookup[spot['id']] = spot
                    spot['fullname'] = fullname
                    area['spot_ids'].append(spot['id'])
                    spot['loc_ids'] = []
                    spot['exit_ids'] = []
                    spot['action_ids'] = []
                    spot['all_data'] = dict(area['all_data'])
                    spot['all_data'].update(spot.get('data', {}))
                    spot['base_movement'] = self.spot_base_movement(spot['all_data'])
                    if 'on_entry' in spot:
                        spot['act'] = parseAction(
                                spot['on_entry'], name=f'{spot["fullname"]}:on_entry')
                        spot['action_id'] = self.make_funcid(spot, 'act', 'on_entry', ON_ENTRY_ARGS)
                    if all_to_update := area.get('all'):
                        if lcl := all_to_update.get('local'):
                            if 'local' in spot:
                                spot['local'].extend(lcl)
                            else:
                                spot['local'] = list(lcl)
                    # hybrid spots are exits but have names
                    for loc in spot.get('locations', []) + spot.get('hybrid', []):
                        loc['spot'] = sname
                        loc['area'] = aname
                        loc['region'] = rname
                        if 'name' not in loc:
                            self._errors.append(f'Location in {spot["fullname"]} requires name')
                            loc["fullname"] = f'{spot["fullname"]} > Location without name'
                            continue
                        loc['id'] = construct_id(rname, aname, sname, loc['name'])
                        self.id_lookup[loc['id']] = loc
                        spot['loc_ids'].append(loc['id'])
                        area['loc_ids'].append(loc['id'])
                        region['loc_ids'].append(loc['id'])
                        loc['fullname'] = f'{spot["fullname"]} > {loc["name"]}'
                        if 'canon' in loc:
                            self.canon_places[construct_id(loc['canon'])].append(loc)
                            loc['canon_id'] = construct_id(loc['canon'])
                        if 'req' in loc:
                            loc['pr'] = _parseExpression(
                                    loc['req'], loc['name'], spot['fullname'], ': ')
                            loc['access_id'] = self.make_funcid(loc)
                        if 'penalties' in loc:
                            self._handle_penalties(loc, spot['fullname'])
                        if 'maps' in loc:
                            loc['tiles'] = [get_map_reference(tilename, loc) for tilename in loc['maps']]
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
                        dest = eh['to']
                        if dest.startswith('^'):
                            if d := spot.get('data', {}).get(dest[1:]):
                                if self.data_types[dest[1:]] != 'SpotId':
                                    self._errors.append(f'Exit {eh["fullname"]} exits to non-spot data: {dest}')
                                else:
                                    dest = d
                            else:
                                self._errors.append(f'Exit {eh["fullname"]} attempts exit to ctx but only data is supported: {dest}')
                        # Limit to in-Area by marking exits across Areas as keep
                        # Maybe later we can try changing to in-Region or global
                        eh['keep'] = '>' in dest or ('tags' in eh and any(t in interesting_tags for t in eh['tags']))
                        if 'req' in eh:
                            eh['pr'] = _parseExpression(
                                    eh['req'], eh['to'], spot['fullname'], ' ==> ')
                            eh['access_id'] = self.make_funcid(eh)
                        if 'penalties' in eh:
                            self._handle_penalties(eh, spot['fullname'])
                        if 'maps' in eh:
                            eh['tiles'] = [get_map_reference(tilename, eh) for tilename in eh['maps']]
                        eh['to'] = dest
                    for act in spot.get('actions', ()):
                        act['spot'] = sname
                        act['area'] = aname
                        act['region'] = rname
                        act['id'] = construct_id(rname, aname, sname, act['name'])
                        self.id_lookup[act['id']] = act
                        spot['action_ids'].append(act['id'])
                        act['fullname'] = f'{spot["fullname"]} > {act["name"]}'
                        if 'req' in act:
                            act['pr'] = _parseExpression(
                                    act['req'], act['name'] + ' req', spot['fullname'], ': ')
                            act['access_id'] = self.make_funcid(act)
                        if 'penalties' in act:
                            self._handle_penalties(act, spot['fullname'])
                        if 'maps' in act:
                            act['tiles'] = [get_map_reference(tilename, act) for tilename in act['maps']]
                        act['act'] = parseAction(
                                act['do'], name=f'{act["fullname"]}:do')
                        act['action_id'] = self.make_funcid(act, 'act', 'do')
                        if 'after' in act:
                            act['act_post'] = parseAction(
                                    act['after'], name=f'{act["name"]}:after')
                            act['after_id'] = self.make_funcid(act, 'act_post', 'after')
                        if 'to' in act:
                            dest = act['to']
                            if dest.startswith('^'):
                                if d := spot.get('data', {}).get(dest[1:]):
                                    if self.data_types[dest[1:]] != 'SpotId':
                                        self._errors.append(f'Action {act["fullname"]} moves to non-spot data: {dest}')
                                    else:
                                        act['to'] = d

            num_locs += len(region['loc_ids'])
        self.num_locations = num_locs


    def _handle_penalties(self, info, category:str):
        for i, pen in enumerate(info['penalties']):
            penaltyname = f'penalty{i + 1}'
            infoname = info['fullname'] if 'fullname' in info else info['name']
            pen['id'] = construct_id(info['id'], penaltyname)
            pen['pr'] = _parseExpression(
                pen['when'], f'{infoname} ({penaltyname})', category, ': ')
            pen['access_id'] = self.make_funcid_from(info, pen['pr'])


    def process_canon(self):
        for loc in self.locations():
            if 'canon' not in loc:
                cname = f'Loc_{loc["id"]}'
                if cname in self.canon_places:
                    self._errors.append(f'Cannot use canon name {cname!r} which collides with default canon name for {loc["fullname"]}')
                else:
                    self.canon_places[cname] = [loc]
                    loc['canon_id'] = cname


    def process_exit_movements(self):
        for spot in self.spots():
            for exit in spot.get('exits', []) + spot.get('hybrid', []):
                if 'time' not in exit and 'movement' in exit:
                    if 'to' not in exit:
                        # check_all will add an error for this
                        continue
                    target = get_exit_target(exit)
                    if target not in self.id_lookup:
                        # check_all will add an error for this
                        continue
                    dest = self.id_lookup[get_exit_target(exit)]
                    if 'coord' not in spot:
                        self._errors.append(f'Expected coord for spot {spot["fullname"]} used in exit with movement: {exit["fullname"]}')
                        continue
                    elif 'coord' not in dest:
                        self._errors.append(f'Expected coord for dest {dest["fullname"]} used in exit with movement: {exit["fullname"]}')
                        continue

                    base = spot['base_movement']
                    sx, sy = spot['coord']
                    tx, ty = dest['coord']
                    jumps = exit.get('jumps', 0)
                    jumps_down = exit.get('jumps_down', 0)
                    if exit['movement'] == 'base':
                        exit['time'] = self.movement_time([], base, abs(tx - sx), ty - sy, jumps, jumps_down)
                    elif (m := exit['movement']) in self.all_movements:
                        exit['time'] = self.movement_time([m], base, abs(tx - sx), ty - sy, jumps, jumps_down)
                    else:
                        self._errors.append(f'Unrecognized movement type in exit {exit["fullname"]}: {m!r}')
                        continue
                    if exit['time'] is None:
                        self._errors.append(f'Unable to determine movement time for exit {exit["fullname"]}: missing jumps?')


    def process_times(self):
        """Adds default times if time is not present, and constant-time penalties."""
        for point in self.all_points():
            if 'time' not in point:
                point['time'] = max(
                        (self.time[k] for k in point.get('tags', []) if k in self.time),
                        default=self.time['default'])
            if point['time'] is None:
                continue
            if 'item' in point and 'to' in point and 'item_time' not in point:
                point['item_time'] = max(
                        (self.time[k] for k in point.get('item_tags', []) if k in self.time),
                        default=self.time.get('hybrid_item_default', self.time['default']))
            if tags := point.get('penalty_tags'):
                penalty = 0
                for tag in tags:
                    if tag[0] == '-':
                        if tag[1:] not in self.time:
                            logging.warning(f'Unrecognized tag {tag[1:]!r} in {point["fullname"]}')
                        else:
                            penalty -= self.time[tag[1:]]
                    elif tag not in self.time:
                        logging.warning(f'Unrecognized tag {tag!r} in {point["fullname"]}')
                    else:
                        penalty += self.time[tag]
                if penalty < 0:
                    self._errors.append(f'Total penalties must be positive: {point["fullname"]} penalty tags: {tags} total {penalty}')
                else:
                    point['time'] += penalty

        for act in self.global_actions:
            if 'time' not in act:
                act['time'] = max(
                        (self.time[k] for k in act.get('tags', []) if k in self.time),
                        default=self.time['default'])
            if tags := act.get('penalty_tags'):
                for tag in tags:
                    act['time'] += self.time.get(tag, 0)


    def process_warps(self):
        self.warps = self._info['warps']
        for name, info in self.warps.items():
            info['name'] = inflection.camelize(name)
            info['id'] = construct_id(info['name'])
            if 'time' not in info:
                self._errors.append(f'Warp {name} requires explicit "time" setting')
            if info['to'].startswith('^'):
                val = info['to'][1:]
                if vtype := self.context_types.get(val):
                    if vtype != 'SpotId':
                        self._errors.append(f'Warp {name} goes to invalid ctx dest: ^{val} (of type {vtype})')
                    info['target_id'] = f'ctx.{val}()'
                elif vtype := self.data_types.get(val):
                    if vtype != 'SpotId':
                        self._errors.append(f'Warp {name} goes to invalid data dest: ^{val} (of type {vtype})')
                    info['target_id'] = f'data::{val}(ctx.position())'
                else:
                    self._errors.append(f'Warp {name} goes to undefined ctx dest: ^{val}')
            else:
                id = construct_id(info['to'])
                if not any(info['id'] == id for info in self.spots()):
                    self._errors.append(f'Warp {name} goes to unrecognized spot: {info["to"]}')
                info['target_id'] = self.target_id_from_id(id)
            if 'req' in info:
                info['pr'] = _parseExpression(info['req'], name, 'warps')
                info['access_id'] = self.make_funcid(info)
            if 'before' in info:
                info['act_pre'] = parseAction(
                        info['before'], name=f'{info["name"]}:before')
                info['before_id'] = self.make_funcid(info, 'act_pre', 'before')
            if 'after' in info:
                info['act_post'] = parseAction(
                        info['after'], name=f'{info["name"]}:after')
                info['after_id'] = self.make_funcid(info, 'act_post', 'after')
            if 'penalties' in info:
                self._handle_penalties(info, 'warps')


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

            if 'after' in act:
                act['act_post'] = parseAction(
                        act['after'], name=f'{act["name"]}:after')
                act['after_id'] = self.make_funcid(act, 'act_post', 'after')


    def process_area_maps(self):
        for area in self.areas():
            if 'map' not in area:
                continue
            map_defs = area['map']
            if isinstance(map_defs, str):
                area['tiles'] = [construct_id('map', area['id'].lower(), map_defs)]
                continue
            elif isinstance(map_defs, (list, tuple)):
                area['tiles'] = [construct_id('map', area['id'].lower(), tile) for tile in map_defs]
                continue
            elif not isinstance(map_defs, dict):
                self._errors.append(f'Invalid map entry for {area["fullname"]}: must be dict')
                continue

            tile_defs = []
            for tile, box in map_defs.items():
                if self._validate_box(box, f'{area["fullname"]} map tile "{tile}"'):
                    tile_defs.append((construct_id('map', area['id'].lower(), tile), tile, box))

            for spot in area['spots']:
                if 'coord' not in spot:
                    continue
                c1, c2 = spot['coord']
                tiles = []
                short_names = []
                for (tile, tsname, box) in tile_defs:
                    if box[0] <= c1 <= box[2] and box[1] <= c2 <= box[3]:
                        tiles.append(tile)
                        short_names.append(tsname)
                if tiles:
                    spot['tiles'] = sorted(tiles)
                    spot['tilenames'] = sorted(short_names)


    def process_parsed_code(self):
        # Check settings
        def _visit(visitor, reverse=False):
            if not reverse:
                for info in self.helpers.values():
                    visitor.visit(info['pr'].tree, info['pr'].name, self.get_default_ctx(), dict(info['args']))
                for pr in self.nonpoint_parse_results():
                    visitor.visit(pr.tree, pr.name, self.get_default_ctx())
            for pt in self.all_points():
                if 'pr' in pt:
                    visitor.visit(pt['pr'].tree, pt['pr'].name, self.get_local_ctx(pt))
                if 'act' in pt:
                    visitor.visit(pt['act'].tree, pt['act'].name, self.get_local_ctx(pt))
            if reverse:
                for info in self.helpers.values():
                    visitor.visit(info['pr'].tree, info['pr'].name, self.get_default_ctx(), dict(info['args']))
                for pr in self.nonpoint_parse_results():
                    visitor.visit(pr.tree, pr.name, self.get_default_ctx())
            self._errors.extend(visitor.errors)

        sv = SettingVisitor(self.context_types, self.settings)
        _visit(sv)
        self.used_settings = sv.setting_options

        for s in self.settings.keys() - self.used_settings.keys():
            logging.warning(f'Did not find usage of setting {s}')

        hv = HelperVisitor(self.helpers, self.rules, self.context_types, self.data_types, self.settings)
        _visit(hv, True)

        cv = ContextVisitor(self.context_types, self.context_values,
                            self.data_types, self.data_values, self.data_defaults)
        _visit(cv)
        self.context_str_values = cv.values
        self.swap_pairs = cv.swap_pairs
        self.named_spots.update(cv.named_spots)

    def process_bitflags(self):
        self.bfp = BitFlagProcessor(self.context_values, self.settings, self.item_max_counts, self.canon_places)
        self.bfp.process()

    def process_special(self):
        if sc := self.special.get('graph_scale'):
            self._validate_scale(sc, 'graph_scale')
        if sc := self.special.get('map_scale'):
            self._validate_scale(sc, 'map_scale')
        if p := self.special.get('map_min'):
            self._validate_pair(p, 'map_min')
            self._validate_all_numeric(p, 'map_min')
        if t := self.special.get('graph_exclude_tags'):
            self._validate_list(t, 'graph_exclude_tags')

    def _validate_scale(self, sc, name):
        if not self._validate_pair(sc, name):
            pass
        elif sc[0] == 0 or sc[1] == 0:
            self._errors.append(f'Invalid {name}: 0 not allowed: {sc}')
        else:
            return self._validate_all_numeric(sc, name)
        return False

    def _validate_all_numeric(self, p, name):
        for x in p:
            if not isinstance(x, (int, float)):
                self._errors.append(f'Invalid {name}: elements must be numeric: {x}')
                return False
        return True

    def _validate_pair(self, p, name):
        if isinstance(p, str):
            self._errors.append(f'Invalid {name}: {p!r} '
                                f'(did you mean [{p}] ?)')
        elif not isinstance(p, (list, tuple)) or len(p) != 2:
            self._errors.append(f'Invalid {name}: {p!r}')
        else:
            return True
        return False
    
    def _validate_box(self, box, name):
        if isinstance(box, str):
            self._errors.append(f'Invalid {name}: {box!r} '
                                f'(did you mean [{box}] ?)')
        elif not isinstance(box, (list, tuple)) or len(box) != 4:
            self._errors.append(f'Invalid {name}: {box!r}')
        else:
            return self._validate_all_numeric(box, name)
        return False

    def _validate_list(self, t, name):
        if isinstance(t, str):
            self._errors.append(f'Invalid {name}: {t!r} '
                                f'(did you mean [{t}] ?)')
        elif not isinstance(t, (list, tuple)):
            self._errors.append(f'Invalid {name}: {t!r}')
        else:
            return True
        return False
    
    def exclude_by_tag(self, info):
        if exc := self.special.get('graph_exclude_tags'):
            if tags := info.get('tags'):
                return any(x in exc for x in tags)
        return False

    def spot_base_movement(self, spot_data):
        d = dict(self.base_movements[0])
        for md in self.base_movements[1:]:
            if 'data' in md and all(d in spot_data and spot_data[d] == v for d, v in md['data'].items()):
                # Later movements override previous ones
                d.update(md)
        if 'data' in d:
            del d['data']
        return d

    @cache
    def region_id_from_id(self, id):
        return construct_id(self.id_lookup[id]['region'])

    @cache
    def target_id_from_id(self, spot_id):
        return f'SpotId::{spot_id}'

    @cached_property
    def movements_by_type(self):
        """Returns a mapping of movement type to movement names (excluding exit-movements)."""
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

        return d


    def movement_time(self, mset, base, a, b, jumps=0, jumps_down=0, jmvmt=None):
        times = []
        xtimes = []
        ytimes = []
        defallt = base.get('fall', 0)
        dejumpt = base.get('jump', 0)
        dejumpdownt = base.get('jump_down', 0)
        mp = [(m, self.all_movements[m]) for m in mset]
        for m, mvmt in mp + [('base', base)]:
            # TODO: This is all cacheable (per pair of spots, per movement type, per pair of points)
            # instead of calculating the times lists for a,b for m, once per powerset of movements
            if s := mvmt.get('free'):
                times.append(math.sqrt(a**2 + b**2) / s)
                continue
            if s := mvmt.get('xy'):
                times.append((abs(a) + abs(b)) / s)
                continue
            if sx := mvmt.get('x'):
                xtimes.append(abs(a) / sx)
            # x, y, fall: not mutually exclusive
            if sy := mvmt.get('y'):
                ytimes.append(abs(b) / sy)
            if sfall := mvmt.get('fall', defallt):
                # fall speed must be the same direction as "down"
                if (t := b / sfall) > 0:
                    t += jumps_down * mvmt.get('jump_down', dejumpdownt)
                    ytimes.append(t)
                elif (jumps and t < 0 and (jmvmt is None or m == jmvmt)
                        and (sjump := mvmt.get('jump', dejumpt))):
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
        """Returns a set of movement tuples that might be considered at the same time.

        Possible relevant movement sets:
          1. any movement on its own
          2. any 'x' or 'x+y' with any 'y' or 'x+y'

        Exit-only movements are not considered at all here.
        """
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
        return sorted(m for m in self.movements)


    def spot_distance(self, sp1, sp2):
        coords = [sp1['coord'], sp2['coord']]
        jumps = [0]
        jumps_down = [0]
        jump_mvmt = None
        for lcl in sp1.get('local', []):
            if lcl.get('to') == sp2['name']:
                # We could have more overrides here, like dist
                if thru := lcl.get('thru'):
                    if isinstance(thru, str):
                        self._errors.append(f'Invalid thru from {sp1["fullname"]} to {sp2["name"]}: {thru!r} '
                                            f'(Did you mean [{thru}] ?)')
                        break
                    if not isinstance(thru, list) or not thru:
                        self._errors.append(f'Invalid thru from {sp1["fullname"]} to {sp2["name"]}: {thru}')
                        break
                    if all(isinstance(t, list) for t in thru):
                        coords[1:1] = thru
                    elif len(thru) == 2 and all(isinstance(t, (int, float)) for t in thru):
                        coords[1:1] = [thru]
                    else:
                        self._errors.append(f'Mismatched length or types in thru '
                                            f'from {sp1["fullname"]} to {sp2["name"]}: {thru}')
                        break
                if j := lcl.get('jumps'):
                    if isinstance(j, str):
                        self._errors.append(f'Invalid jumps from {sp1["fullname"]} to {sp2["name"]}: {j!r} '
                                            f'(Did you mean [{j}] ?)')
                        break
                    if not isinstance(j, list):
                        j = [j]
                    if len(j) != len(coords) - 1:
                        self._errors.append(f'Jumps list from {sp1["fullname"]} to {sp2["name"]} '
                                            f'must match path length 1+thru = {len(coords) - 1} but was {len(j)}')
                        break
                    jumps[:] = j
                else:
                    jumps *= len(coords) - 1
                if j := lcl.get('jumps_down'):
                    if isinstance(j, str):
                        self._errors.append(f'Invalid jumps from {sp1["fullname"]} to {sp2["name"]}: {j!r} '
                                            f'(Did you mean [{j}] ?)')
                        break
                    if not isinstance(j, list):
                        j = [j]
                    if len(j) != len(coords) - 1:
                        self._errors.append(f'Jumps_down list from {sp1["fullname"]} to {sp2["name"]} '
                                            f'must match path length 1+thru={len(coords) - 1}: {len(j)}')
                        break
                    jumps_down[:] = j
                else:
                    jumps_down *= len(coords) - 1
                # TODO: It might be more reasonable to just have a list of allowed movement types?
                if m := lcl.get('jump_movement'):
                    if m not in self.all_movements:
                        self._errors.append(f'Unrecognized movement type from {sp1["fullname"]} to {sp2["name"]}: {m}')
                        break
                    jump_mvmt = m
                break
            
        else:
            # spots must explicitly declare connections
            return ([], [], [], None)
        
        return (coords, jumps, jumps_down, jump_mvmt)
        

    @cached_property
    def local_distances(self):
        # create a distances table: (spot, spot) -> [(x, y), ...]
        d = defaultdict(list)
        for a in self.areas():
            errors = []
            for sp in a['spots']:
                if c := sp.get('coord'):
                    self._validate_pair(c, f'coord for {sp["fullname"]}')
                elif sp.get('local'):
                    errors.append(f'Expected coord for spot {sp["fullname"]} with local rules')
            if errors:
                self._errors.extend(errors)
                break
            coord_errors = set()
            spot_names = set(sp['name'] for sp in a['spots'])
            for sp1 in a['spots']:
                if 'coord' not in sp1 or 'local' not in sp1:
                    continue
                if any(link.get('to') is None for link in sp1['local']):
                    self._errors.append(f'Expected "to:" in all local movement for spot {sp1["fullname"]}')
                    continue
                unrecognized = set(link['to'] for link in sp1['local']) - spot_names
                if unrecognized:
                    self._errors.append(f'Unrecognized destinations in local movement for spot {sp1["fullname"]}: {sorted(unrecognized)}')

            for sp1, sp2 in itertools.permutations(a['spots'], 2):
                if 'coord' not in sp1 or 'local' not in sp1:
                    continue
                if 'coord' not in sp2:
                    if sp2['name'] not in coord_errors and any(link["to"] == sp2['name'] for link in sp1['local']):
                        coord_errors.add(sp2['name'])
                        self._errors.append(f'Expected coord for spot {sp2["fullname"]} used in local rules')
                    continue
                coords, jumps, jumps_down, jmvmt = self.spot_distance(sp1, sp2)
                if not coords:
                    continue
                for ((sx, sy), (cx, cy)), j, jd in zip(itertools.pairwise(coords), jumps, jumps_down):
                    d[(sp1['id'], sp2['id'])].append(
                            (abs(cx - sx), cy - sy, j, jd, jmvmt))
        return d


    @cached_property
    def movement_tables(self):
        # create a movement table for each movement "combo"
        # (generally we'll use only 1 free, or 1 xy, or 1x, or 1x+1y, at a time,
        #  but we can't guarantee which is best for all situations.
        #  It might be simplest to determine which movements we have available in
        #  the area we're in, and then look up the travel time from that.)
        table = {}
        impossible = Counter()
        for mset in itertools.chain.from_iterable(
                itertools.combinations(self.non_default_movements, r)
                for r in range(0, len(self.non_default_movements) + 1)):
            key = tuple(m in mset for m in self.non_default_movements)
            table[key] = local_time = {}
            for k, dlist in self.local_distances.items():
                base = self.id_lookup[k[0]]['base_movement']
                times = [self.movement_time(mset, base, a, b, j, jd, jmvmt) for a,b, j, jd, jmvmt in dlist]
                if all(t is not None for t in times):
                    local_time[k] = times
                else:
                    impossible[k] += 1
        for k, val in impossible.items():
            if val == 2 ** len(self.non_default_movements):
                logging.warning(f'Base movement is not possible: {self.id_lookup[k[0]]["fullname"]}'
                                f' --> {self.id_lookup[k[1]]["name"]}')
        return table

    def iter_movement_set_keys(self):
        for mset in itertools.chain.from_iterable(
                itertools.combinations(self.non_default_movements, r)
                for r in range(0, len(self.non_default_movements) + 1)):
            yield tuple(m in mset for m in self.non_default_movements)

    @cached_property
    def movements_rev_lookup(self):
        # Precalculating which movement types we need available for what movement times
        # so this will look like (sp1, sp2) -> (base movement time, [(mkey, time)])
        base, *mkeys = list(self.iter_movement_set_keys())
        table = {k: (sum(times), []) for k, times in self.movement_tables[base].items()}
        def is_subset(x, y):
            return all(b or not a for a, b in zip(x, y))
        for mkey in mkeys:
            for k, times in self.movement_tables[mkey].items():
                t = sum(times)
                if k not in table:
                    table[k] = (-1, [])
                if table[k][0] < 0 or (t < table[k][0] and not any(st < t for v, st in table[k][1] if is_subset(v, mkey))):
                    table[k][1].append((mkey, t))
        return table

    
    @cached_property
    def base_distances(self):
        # initial conditions: (x,y) -> t according to the best movement
        table = {k: sum(t)
                 for k, t in self.movement_tables[tuple(True for _ in self.non_default_movements)].items()}

        def _update(key, val):
            if key in table:
                table[key] = min(table[key], val)
            else:
                table[key] = val

        # every exit
        # every warp with base_movement: true
        # every action with a "to" field
        # every warp/global action with a "to" field to a data value that's a valid spot
        warp_dests = []
        data_dests = []
        examiner = PossibleVisitor(self.helpers, self.rules, self.context_types,
                                   self.data_types, self.data_defaults, self.data_values)
        for w in self.warps.values():
            if w.get('base_movement') and w['to'][0] != '^':
                warp_dests.append((construct_id(w['to']), w['time']))
            elif w['to'][0] == '^' and w['to'][1:] in self.data_values:
                data_dests.append((w['to'][1:], w))
        for act in self.global_actions:
            if 'to' in act and act['to'][0] == '^' and act['to'][1:] in self.data_values:
                data_dests.append((act['to'][1:], act))
        for s in self.spots():
            table[(s['id'], s['id'])] = 0
            for ex in s.get('exits', []) + s.get('hybrid', []):
                key = (s['id'], get_exit_target(ex))
                if 'time' not in ex:
                    raise Exception(f'"time" not defined for exit {ex["fullname"]}')
                _update(key, float(ex['time']))
            for act in s.get('actions', []):
                if 'to' in act:
                    if not act['to'].startswith('^'):
                        key = (s['id'], get_exit_target(act))
                        _update(key, act['time'])
                    elif act['to'][1:] in self.data_values:
                        if dest := self.data_values[act['to'][1:]].get(s['id']):
                            if dest != 'SpotId::None' and dest != s['fullname']:
                                key = (s['id'], construct_id(dest))
                                _update(key, act['time'])
            for w, t in warp_dests:
                if s['id'] == w:
                    continue
                if 'pr' not in w or examiner.examine(w['pr'].tree, s['id'], w['name']):
                    key = (s['id'], w)
                    _update(key, t)
            for d, info in data_dests:
                if dest := self.data_values[d].get(s['id']):
                    if (dest != 'SpotId::None' and dest != s['fullname'] and
                            ('pr' not in info or examiner.examine(info['pr'].tree, s['id'], info['name']))):
                        key = (s['id'], construct_id(dest))
                        _update(key, info['time'])

        return table


    def make_funcid(self, info, prkey:str='pr', field:str='req', extra_fields=None):
        return self.make_funcid_from(info, info[prkey], field=field, extra_fields=extra_fields)

    def make_funcid_from(self, info, pr, field:str='req', extra_fields=None):
        ruletype = pr.parser.ruleNames[pr.tree.getRuleIndex()]
        d = self.action_funcs if ruletype == 'actions' else self.access_funcs
        if '^_' in str(pr.text):
            id = construct_id(info['id'].lower(), field)
            assert id not in d, f'trying to generate multiple functions named {id}: {info}'
            d[id] = {ruletype: pr, 'region': info['region']}
            if 'area' in info:
                d[id]['area'] = info['area']
            if extra_fields:
                d[id]['args'] = extra_fields
            return id

        id = construct_id(str(pr.name) if '^_' in str(pr.text) else escape_ops(str(pr.text))).lower()
        if id not in d:
            d[id] = {ruletype: pr}
            if extra_fields:
                d[id]['args'] = extra_fields
            return id

        if ruletype not in d[id]:
            raise Exception(f'func {id} missing {ruletype}: Is it redefined from {d[id].keys()}?')

        if d[id][ruletype].text != pr.text:
            logging.info(f'Rules with same id but different text: {id}({ruletype}) = {d[id][ruletype].text!r} but '
                         f'this pr = {pr.text!r}')
            id = id + '__' + str(sum(1 for k in d if k.startswith(id)))
            assert id not in d, f'duplicate even after counting: {id}'
            d[id] = {ruletype: pr}
            if extra_fields:
                d[id]['args'] = extra_fields
        return id


    def areas(self):
        return itertools.chain.from_iterable(r['areas'] for r in self.regions)


    def spots(self):
        return itertools.chain.from_iterable(a['spots'] for a in self.areas())


    def interesting_spots(self):
        return filter(
            lambda s: s.get('keep') or 'locations' in s or 'actions' in s or 'hybrid' in s
                or s['fullname'] in self.named_spots
                or any(e['keep'] for e in s.get('exits', ())),
            self.spots())

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

    def get_area(self, spot_id):
        spot = self.id_lookup[spot_id]
        return self.id_lookup[construct_id(spot['region'], spot['area'])]


    def nonpoint_parse_results(self):
        yield from (info['pr'] for rule in self.rules.values() for info in rule.variants.values())
        yield from (info['act'] for info in self.collect.values())
        yield from (info['pr'] for info in self.movements.values() if 'pr' in info)
        yield from (info['pr'] for info in self.warps.values() if 'pr' in info)
        yield from (info['pr'] for info in self.global_actions if 'pr' in info)
        yield from (info['act'] for info in self.global_actions)
        yield from (info['act_pre'] for info in self.warps.values() if 'act_pre' in info)
        yield from (info['act_post'] for info in self.warps.values() if 'act_post' in info)


    def all_parse_results(self):
        yield from (info['pr'] for info in self.helpers.values())
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
        conns = defaultdict(set)
        for r in self.regions:
            for a in r['areas']:
                for s in a['spots']:
                    for ex in s.get('exits', ()):
                        # This adjacency is only important for graphs, so if we exclude exits from the graph,
                        # we can exclude them here
                        if self.exclude_by_tag(ex):
                            continue
                        target = ex['to'].split('>')
                        if len(target) == 3:
                            t = construct_id(target[0].strip())
                            conns[t].add(r['id'])
                            conns[r['id']].add(t)
            if r['id'] not in conns:
                conns[r['id']] = set()
        return conns


    @cached_property
    def region_colors(self):
        d = {}
        NUMCOLORS = 7
        nextcolor = 0
        left = []
        for rid, neighbors in sorted(self.adjacent_regions.items(), key=lambda x: len(x[1])):
            ncolors = set(d[n] for n in neighbors if n in d)
            if nextcolor not in ncolors:
                d[rid] = nextcolor
                nextcolor = (nextcolor + 1) % NUMCOLORS
            else:
                left.append((rid, neighbors))
        for rid, neighbors in left:
            ncolors = set(d[n] for n in neighbors if n in d)
            if len(ncolors) >= NUMCOLORS:
                d[rid] = NUMCOLORS
            for offset in range(0, NUMCOLORS):
                nc = (nextcolor + offset) % NUMCOLORS
                if nc not in ncolors:
                    d[rid] = nc
                    nextcolor = nc
                    break
            else:
                d[rid] = NUMCOLORS
        return d


    def handle_typehint_config(self, category, d):
        def _apply_override(s, t, info, text):
            if declared := info.get('type'):
                if declared != t:
                    logging.warning(f'{category} {s} type {declared} overridden by {text} ({t})')
            info['type'] = t

        for s, info in d.items():
            if disallowed := info.keys() - TYPEHINT_FIELDS:
                self._errors.append(f'Unrecognized fields on {category} {s}: {", ".join(disallowed)}')
                continue
            if m := info.get('max', 0):
                t = config_type(m)
                _apply_override(s, t, info, f'max: {m}')
                if t == 'int':
                    info['rust_type'] = get_int_type_for_max(m)
            elif opts := info.get('opts', ()):
                t, *types = {config_type(o) for o in opts}
                if types:
                    self._errors.append(f'{category} {s} options are mixed types: {t}, {", ".join(types)}')
                    continue
                _apply_override(s, t, info, f'opts, e.g. {opts[0]}')
                if t == 'int':
                    info['rust_type'] = get_int_type_for_max(max(opts))
            elif 'type' not in info:
                self._errors.append(f'{category} {s} must declare one of: type, max, opts')
                continue
            if 'rust_type' not in info:
                info['rust_type'] = ctx_types.get(info['type'], info['type'])

        return d

    @cached_property
    def settings(self):
        sd = self._info.get('settings', {})

        return self.handle_typehint_config('Setting', sd)


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
        for func in BUILTINS.keys() & self.rules.keys():
            self._errors.append(f'Cannot use reserved name {func!r} as rule name')
        for pr in self.all_parse_results():
            for t in pr.parser.getTokenStream().tokens:
                if pr.parser.symbolicNames[t.type] == 'FUNC' and t.text not in self.allowed_funcs:
                    self._errors.append(f'{pr.name}: Unrecognized function {t.text}')
        # Check exits
        spot_ids = {sp['id'] for sp in self.spots()}
        for ex in self.exits():
            if 'to' not in ex:
                self._errors.append(f'No destination defined for {ex["fullname"]}')
            elif get_exit_target(ex) not in spot_ids:
                self._errors.append(f'Unrecognized destination in exit {ex["fullname"]}')
        for spot in self.spots():
            for act in spot.get('actions', []):
                if 'to' in act and not act['to'].startswith('^') and get_exit_target(act) not in spot_ids:
                    self._errors.append(f'Unrecognized destination in action {act["fullname"]}: {act["to"]}')
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
        self.context_trigger_rules

        return self._errors


    @cached_property
    def vanilla_items(self):
        return {pt['item'] for pt in self.all_points()
                if 'item' in pt}


    @cached_property
    def items_used_in_rules(self):
        return {t.text
                for pr in self.all_parse_results()
                for t in pr.parser.getTokenStream().tokens
                if pr.parser.symbolicNames[t.type] == 'ITEM'}


    @cached_property
    def all_items(self):
        return sorted(self.vanilla_items | self.items_used_in_rules)


    def process_items(self):
        visitor = ItemVisitor(self.rules, self.settings, self.vanilla_items)
        for pr in self.all_parse_results():
            visitor.visit(pr.tree, name=pr.name)
        self._errors.extend(visitor.errors)
        self.item_uses = visitor.item_uses
        self.item_max_counts = visitor.item_max_counts
        self.items_by_source = visitor.items_by_source
        self.rule_items = {
            name: {
                variant: dict(self.items_by_source[f'rules:{name}_{variant}'])
                for variant in rule.variants
            }
            for name, rule in self.rules.items()
        }
        self._source_refs = visitor.source_refs

        def _get_all_refs(sourcename):
            refs = visitor.source_refs[sourcename]
            checked = set()
            while diff := refs - checked:
                next = diff.pop()
                checked.add(next)
                if next in visitor.source_refs:
                    refs |= visitor.source_refs[next]
            return refs
        
        general_items = set(self.items_by_source['general'].keys())
        for ref in _get_all_refs('general'):
            general_items |= self.items_by_source[ref].keys()

        for rule, variants in self.rule_items.items():
            for variant, item_maxes in variants.items():
                for ref in _get_all_refs(f'rules:{rule}_{variant}'):
                    for item, ct in self.items_by_source[ref].items():
                        if item in item_maxes:
                            item_maxes[item] = max(item_maxes[item], ct)
                        else:
                            item_maxes[item] = ct
        
        general_unused = set(self.all_items) - general_items - self.collect.keys() - self.items_by_source['general'].keys()
        self.unused_by_rule = {
            rule: {
                variant: general_unused - variant_items.keys()
                for variant, variant_items in variants.items()
            }
            for rule, variants in self.rule_items.items()
        }
        self.victory_rule_refs = {
            variant: [ref[6:] for ref in _get_all_refs(f'rules:$victory_{variant}') if ref.startswith('rules:')]
            for variant in self.rules['$victory'].variants
        }
        self.item_locations = defaultdict(list)
        for loc in self.locations():
            if 'item' not in loc:
                self._errors.append(f'Expected item at location {loc["fullname"]}')
                continue
            self.item_locations[loc['item']].append(loc['id'])


    @cached_property
    def unused_items(self):
        return self.all_items - self.item_max_counts.keys() - self.collect.keys()
    

    @cached_property
    def undefined_items(self):
        return self.all_items - self.item_locations.keys() - set(self.special.get('unplaced_items', ()))


    def process_context(self):
        def _check_types(v1, v2, ctx, category, *names, local=False):
            t1 = typenameof(v1)
            t2 = typenameof(v2)
            if local:
                if t1 == 'SpotId' and t2 in ('AreaId', 'ENUM'):
                    return
                if len(names) == 1 and t1 == 'AreaId' and t2 == 'ENUM':
                    return
            if t1 != t2:
                self._errors.append(
                    f'context value type mismatch: {ctx} defined as {v1} ({t1}) '
                    f'and reused in {" > ".join(names)} in "{category}" section as {v2} ({t2})')

        def _check_data(ctx, val, category, *names, data=False):
            if data:
                if ctx in self.data:
                    _check_types(self._info['data'][ctx], val, ctx, category, *names, local=True)
                else:
                    self._errors.append(
                        f'context data field {ctx} used in {" > ".join(names)} in '
                        f'"{category}" section must have a global default value set')
            else:
                if ctx in self.data:
                    self._errors.append(
                        f'context category mismatch: {ctx} defined as data but used in '
                        f'{" > ".join(names)} in "{category}" section')

        # self._info: start, data
        # regions/areas: start, enter, data
        gc = dict(self._info['start'])
        for ctx, hints in self.context_type_hints.items():
            if d := hints.get('default'):
                gc[ctx] = d
            elif hints['type'] == 'int':
                gc[ctx] = 0
            elif hints['type'] == 'bool':
                gc[ctx] = False

        for area in self.areas():
            if 'map' not in area:
                continue
            map_defs = area['map']
            if isinstance(map_defs, str):
                map_defs = [map_defs]
            for tilename in map_defs:
                k = construct_id('map', area['id'].lower(), tilename)
                if k in gc:
                    self._errors.append(f'Name conflict: cannot define "{k}" and map tile "{tilename}" in {area["fullname"]}')
                else:
                    gc[k] = False

        def _check_shadow(ctx, category, *names):
            _check_data(ctx, val, category, *names)
            if len(names) == 2:
                pc = construct_id(names[0], 'ctx', ctx[1:]).lower()
                if pc in gc:
                    self._errors.append(
                        f'Context parameter {ctx} in {" > ".join(names)} "{category}" hides '
                        f'parameter {ctx} in {names[0]}')

        def _handle_start(ctx, val, category, *names):
            _check_data(ctx, val, category, *names)
            if ctx[0] == '_':
                _check_shadow(ctx, *names)
                ctx = construct_id(*names, 'ctx', ctx[1:]).lower()
            if ctx in gc:
                self._errors.append(
                    f'Duplicate context parameter {ctx} in {" > ".join(names)}: '
                    f'not allowed in "{category}" section')
            else:
                gc[ctx] = val

        def _handle_triggers(ctx, val, category, *names):
            _check_data(ctx, val, category, *names)
            if ctx[0] == '_':
                _check_shadow(ctx, *names)
                ctx = construct_id(*names, 'ctx', ctx[1:]).lower()
            if ctx in gc:
                _check_types(gc[ctx], val, ctx, category, *names)
            else:
                gc[ctx] = val

        for region in self.regions:
            for ctx, val in region.get('start', {}).items():
                _handle_start(ctx, val, 'start', region['name'])
            for trigger in TRIGGER_RULES:
                for ctx, val in region.get(trigger, {}).items():
                    _handle_triggers(ctx, val, trigger, region['name'])
            for ctx, val in region.get('data', {}).items():
                _check_data(ctx, val, 'data', region['name'], data=True)
        # Areas must be handled second to check for shadowing
        for area in self.areas():
            for ctx, val in area.get('start', {}).items():
                _handle_start(ctx, val, 'start', area['region'], area['name'])
            for trigger in TRIGGER_RULES:
                for ctx, val in area.get(trigger, {}).items():
                    _handle_triggers(ctx, val, trigger, area['region'], area['name'])
            for ctx, val in area.get('data', {}).items():
                _check_data(ctx, val, 'data', area['region'], area['name'], data=True)

        self.context_values = gc


    @cached_property
    def context_type_hints(self):
        return self.handle_typehint_config('Context', self._info.get('context', {}))

    @cached_property
    def context_types(self):
        d = {'position': 'SpotId'}
        for ctx, hints in self.context_type_hints.items():
            t = hints['rust_type']
            if t == 'ENUM':
                t = 'enums::' + ctx.capitalize()
            d[ctx] = t
        for ctx, val in self.context_values.items():
            if ctx not in d:
                t = typenameof(val)
                if t == 'ENUM':
                    t = 'enums::' + ctx.capitalize()
                d[ctx] = t
        return d

    @cached_property
    def data_types(self):
        d = {}
        for ctx, val in self.data_defaults.items():
            t = typenameof(val)
            if t == 'ENUM':
                t = 'enums::' + ctx.capitalize()
            d[ctx] = t
        return d

    def get_default_ctx(self):
        return {c: c for c in itertools.chain(self.context_values, self.data_defaults)
                if '__ctx__' not in c}

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

    def translate_ctx(self, ctx, info):
        if 'region' not in info or ctx[0] != '_':
            return ctx
        area = info.get('area') or info['name']

        poss = [construct_id(info['region'], 'ctx', ctx[1:]).lower(),
                construct_id(info['region'], area, 'ctx', ctx[1:]).lower()]
        for cname in poss:
            if cname in self.context_values:
                return cname
        return ctx

    @cached_property
    def data_values(self):
        # data name -> spot id -> value
        d = {c: {} for c in self.data_defaults}
        def get_first(datamap, tilenames):
            for tile in tilenames:
                if tile in datamap:
                    return datamap[tile]

        errors = set()
        def handle_place(c, source, val):
            if self.data_types[c] == 'SpotId':
                names = get_spot_reference_names(val, source)
                sp = ' > '.join(names)
                if construct_id(sp) not in self.id_lookup:
                    errors.add(f'Unknown spot {sp!r} in {source["fullname"]} data {c!r}')
                self.named_spots.add(sp)
                return sp
            if self.data_types[c] == 'AreaId':
                names = get_spot_reference_names(val + '>', source)
                return ' > '.join(names[:2])
            return val

        for r in self.regions:
            for a in r['areas']:
                for s in a['spots']:
                    for c, cdict in d.items():
                        if c in s.get('data', {}):
                            cdict[s['id']] = handle_place(c, s, s['data'][c])
                            continue
                        elif c in a.get('datamap', {}) and 'tilenames' in s:
                            val = get_first(a['datamap'][c], s['tilenames'])
                            if val is not None:
                                cdict[s['id']] = handle_place(c, s, val)
                                continue
                        if c in a.get('data', {}):
                            cdict[s['id']] = handle_place(c, a, a['data'][c])
                        elif c in r.get('data', {}):
                            cdict[s['id']] = r['data'][c]
        self._errors.extend(sorted(errors))
        return d

    @cached_property
    def context_trigger_rules(self):
        d = {trigger: {'region': defaultdict(dict), 'area': defaultdict(dict), 'spot': defaultdict(dict)}
             for trigger in TRIGGER_RULES}

        def _add_rules(place, ptype):
            localctx = self.get_local_ctx(place)
            for trigger in TRIGGER_RULES:
                if e := place.get(trigger):
                    for k, v in e.items():
                        if k not in localctx:
                            self._errors.append(f'Undefined ctx property ^{k} in {place["name"]}:{trigger}')
                            continue
                        d[trigger][ptype][place['id']][localctx[k]] = str_to_rusttype(v, self.context_types[localctx[k]])

        for r in self.regions:
            _add_rules(r, 'region')
            for a in r['areas']:
                _add_rules(a, 'area')
                for s in a['spots']:
                    _add_rules(s, 'spot')

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
        d = {'region': set(), 'area': set(), 'spot': set()}
        d['region'].update(self.context_trigger_rules['enter']['region'].keys())
        d['area'].update(self.context_trigger_rules['enter']['area'].keys())
        d['spot'].update(self.context_trigger_rules['enter']['spot'].keys())
        d['region'].update(self.context_resetters['region'].keys())
        d['area'].update(self.context_resetters['area'].keys())
        d['region'].update(r['id'] for r in self.regions if 'act' in r or 'tiles' in r)
        d['area'].update(a['id'] for a in self.areas() if 'act' in a or 'tiles' in a)
        d['spot'].update(s['id'] for s in self.spots() if 'act' in s or 'tiles' in s)
        return d

    
    @cached_property
    def default_price_type(self):
        for ctx, hints in self.context_type_hints.items():
            if hints['type'] == 'int':
                return ctx
        for ctx, val in self._info['start'].items():
            if typenameof(val) == 'i32':
                return ctx


    @cached_property
    def price_types(self):
        ints = {ctx: hints['rust_type'] for ctx, hints in self.context_type_hints.items()
                if hints['type'] == 'int'}
        starts = {ctx: 'i32' for ctx, val in self._info['start'].items()
                  if typenameof(val) == 'i32' and ctx not in ints}
        return ints | starts


    def prToRust(self, pr, info, id=None):
        return RustVisitor(self.rules,
                           self.context_types,
                           self.action_funcs,
                           self.get_local_ctx(info),
                           self.data_types,
                           id or pr.name).visit(pr.tree)
    

    def prToRustExplain(self, pr, info, id=None):
        return RustExplainerVisitor(self.rules,
                                    self.context_types,
                                    self.action_funcs,
                                    self.get_local_ctx(info),
                                    self.data_types,
                                    id or pr.name).visit(pr.tree)
    
    def prToRustObserve(self, pr, info, id=None):
        return RustObservationVisitor(self.item_max_counts,
                                      self.collect,
                                      self.rules,
                                      self.context_types,
                                      self.action_funcs,
                                      self.get_local_ctx(info),
                                      self.data_types,
                                      id or pr.name).visit(pr.tree)

    def render(self):
        env = jinja2.Environment(loader=jinja2.FileSystemLoader(templates_dir),
                                 line_statement_prefix='%%',
                                 line_comment_prefix='%#')
        env.filters.update({
            'camelize': inflection.camelize,
            'construct_id': construct_id,
            'construct_place_id': construct_place_id,
            'construct_test_name': construct_test_name,
            'escape_ctx': partial(re.compile(r'\b(ctx|world|edict|full_obs)\b').sub, r'$\1'),
            'field_size': field_size,
            'get_area': self.get_area,
            'get_exit_target': get_exit_target,
            'get_exit_target_id': get_exit_target_id,
            'get_int_type_for_max': get_int_type_for_max,
            'get_spot_reference': get_spot_reference,
            'hex': hex,
            'prToRust': self.prToRust,
            'prToRustExplain': self.prToRustExplain,
            'prToRustObserve': self.prToRustObserve,
            'region_id_from_id': self.region_id_from_id,
            'str_to_rusttype': str_to_rusttype,
            'target_id_from_id': self.target_id_from_id,
            'translate_ctx': self.translate_ctx,
            'treeToString': treeToString,
            'trim_type_prefix': trim_type_prefix,
        })
        env.tests.update({
            'exclude_by_tag': self.exclude_by_tag,
        })
        # Access cached_properties to ensure they're in the template vars
        self.unused_items
        self.context_types
        self.default_price_type
        self.price_types
        self.movement_tables
        self.movements_rev_lookup
        self.base_distances
        self.context_trigger_rules
        self.context_position_watchers
        self.all_connections
        self.region_colors
        files = {
            '.': ['Cargo.toml'],
            'data': ['digraph.dot', 'digraph.mmd', 'graph_map.sh'],
            'src': ['lib.rs', 'items.rs', 'helpers.rs', 'graph.rs', 'graph_enums.rs', 'context.rs',
                    'observe.rs', 'prices.rs', 'rules.rs', 'movements.rs', 'settings.rs'],
            'benches': ['bench.rs'],
            'bin': ['main.rs'],
            'tests': ['unittest.rs'],
        }
        rustfiles = []
        for dirname, fnames in files.items():
            os.makedirs(os.path.join(self.game_dir, dirname), exist_ok=True)
            for tname in fnames:
                template = env.get_template(tname + '.jinja')
                name = os.path.join(self.game_dir, dirname, tname)
                if name.endswith('.rs') and tname not in ('lib.rs', 'context.rs'):
                    rustfiles.append(name)
                with open(name, 'w', encoding='utf-8') as f:
                    f.write(template.render(gl=self, int_types=int_types, **self.__dict__))

        cmd = ['rustfmt'] + rustfiles
        subprocess.run(cmd)


if __name__ == '__main__':
    cmd = argparse.ArgumentParser()
    cmd.add_argument('game', help='Which game to build the graph for')
    cmd.add_argument('--noparse', action='store_true')
    args = cmd.parse_args()

    gl = GameLogic(args.game)
    if gl.errors:
        print('\n'.join(gl.errors))
        print(f'Encountered {len(gl.errors)} error(s); exiting before codegen.')
        sys.exit(1)

    if gl.undefined_items:
        logging.warning(f'Unplaced items: {", ".join(sorted(gl.undefined_items))}')

    if not args.noparse:
        logging.info(f'Rendering {gl.game} graph: {len(list(gl.spots()))} spots, '
                    f'{sum(len(r["loc_ids"]) for r in gl.regions)} locations '
                    f'({len(gl.canon_places)} canon locations), '
                    f'{len(list(gl.actions()))} actions, {len(gl.all_items)} items, '
                    f'{len(gl.helpers)} helpers, {len(gl.context_types)} context properties, '
                    f'{len(gl.warps)} warps, {sum(len(rule.variants) for rule in gl.rules.values())} rule variants')
        gl.render()
        logging.info(f'Render complete.')
    else:
        logging.info(f'Constructed {gl.game} graph in variable `gl`')
