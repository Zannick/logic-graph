import argparse
from collections import namedtuple
import itertools
import logging
import os
import re
import yaml

base_dir = os.path.dirname(os.path.realpath(__file__))

from grammar import parseRule, parseAction, StringVisitor

MAIN_FILENAME = 'Game.yaml'
GAME_FIELDS = {'name', 'objectives', 'movements', 'warps', 'checks', 'start', 'load',
               'helpers', 'collect'}
# To be validated later
REGION_FIELDS = {'name', 'short', 'here'}
AREA_FIELDS = {'name', 'enter', 'exits', 'spots'}
SPOT_FIELDS = {'name', 'coord', 'actions', 'locations', 'exits', 'hybrid'}

# To be replaced with standard functions instead of helpers
BUILTINS = {'$max', '$min', '$all_checks'}

def load_regions_from_file(file):
    try:
        with open(file) as f:
            return list(yaml.safe_load_all(f))
    except Exception as e:
        raise Exception(f'Error reading from {file}') from e
    # TODO: validate fields


def load_game_yaml(game_dir):
    yfiles = [file for file in os.listdir(game_dir) if file.endswith('.yaml')]
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
        load_regions_from_file(os.path.join(game_dir, file))
        for file in sorted(yfiles)))
    return game


def _parseExpression(logic, name, category, sep=':'):
    rule = 'boolExpr'
    if ':' in name:
        rule, name = name.split(':', 1)
    return parseRule(rule, logic, name=f'{category}{sep}{name}')


disallowed_chars = re.compile(r'[^A-Za-z_0-9]')
def construct_id(*args):
    return '__'.join(disallowed_chars.sub('', a.replace(' ', '_')) for a in args)


class GameLogic(object):

    def __init__(self, game):
        self.errors = []
        self.game = game
        self.game_dir = os.path.join(base_dir, 'games', game)
        self._info = gameinfo = load_game_yaml(self.game_dir)
        self.helpers = {name: _parseExpression(logic, name, 'helpers')
                        for name, logic in gameinfo['helpers'].items()}
        self.objectives = {name: _parseExpression(logic, name, 'objectives')
                           for name, logic in gameinfo['objectives'].items()}
        self.collect = {name: parseAction(logic, name, 'collect')
                        for name, logic in gameinfo['collect'].items()}
        self.errors.extend(itertools.chain.from_iterable(
            v.errors
            for category in [self.helpers, self.objectives, self.collect]
            for v in category.values()))

        # these are {name: {...}} dicts
        self.movements = gameinfo['movements']
        for name, info in self.movements.items():
            if 'req' in info:
                info['pr'] = _parseExpression(info['req'], name, 'movements')
                self.errors.extend(info['pr'].errors)

        self.warps = gameinfo['warps']
        for name, info in self.warps.items():
            if 'req' in info:
                info['pr'] = _parseExpression(info['req'], name, 'warps')
                self.errors.extend(info['pr'].errors)

        # these are dicts {name: blah, req: blah} (at whatever level)
        self.regions = gameinfo['regions']
        for region in self.regions:
            rname = region.get('short', region['name'])
            region['id'] = construct_id(rname)
            for area in region['areas']:
                aname = area['name']
                area['region'] = rname
                area['id'] = construct_id(rname, aname)

                for e in area.get('exits', ()):
                    e['area'] = aname
                    e['region'] = rname
                    e['id'] = construct_id(rname, aname, 'ex', e['to'])
                    if 'req' in e:
                        e['pr'] = _parseExpression(
                                e['req'], e['to'], f'{rname} > {aname}', ' ==> ')
                        self.errors.extend(e['pr'].errors)

                for spot in area['spots']:
                    sname = spot['name']
                    spot['area'] = aname
                    spot['region'] = rname
                    spot['id'] = construct_id(rname, aname, sname)
                    fullname = f'{rname} > {aname} > {sname}'
                    for loc in spot.get('locations', ()):
                        loc['spot'] = sname
                        loc['area'] = aname
                        loc['region'] = rname
                        loc['id'] = construct_id(rname, aname, sname, loc['name'])
                        if 'req' in loc:
                            loc['pr'] = _parseExpression(
                                    loc['req'], loc['name'], fullname, ' ')
                            self.errors.extend(loc['pr'].errors)
                    for eh in spot.get('exits', []) + spot.get('hybrid', []):
                        eh['id'] = construct_id(rname, aname, sname, 'ex', eh['to'])
                        if 'req' in eh:
                            eh['pr'] = _parseExpression(
                                    eh['req'], eh['to'], fullname, ' ==> ')
                            self.errors.extend(eh['pr'].errors)

                    for act in spot.get('actions', ()):
                        act['id'] = construct_id(rname, aname, sname, act['name'])
                        if 'req' in act:
                            act['pr'] = _parseExpression(
                                    act['req'], act['name'] + ' req', fullname, ' ')
                            self.errors.extend(act['pr'].errors)
                        act['act'] = parseAction(
                                act['do'], name=f'{fullname} {act["name"]}:do')
                        self.errors.extend(act['act'].errors)


    def areas(self):
        return itertools.chain(r['areas'] for r in self.regions)


    def graph_rules(self):
        pass


    def all_rules(self):
        # We need to iterate over the rules, but with the ids
        # The items we have:
        # helpers, objectives, collect: {name -> ParseResult}
        # movements, warps: {name -> {'pr' -> ParseResult}? }
        # regions: it's complicated:
        #   [{'areas' -> [{
        #       'exits' -> [{'pr' -> ParseResult}]
        #       'spots' -> [{
        #           'locations'/'exits'/'hybrid' -> [{'pr' -> ParserResult}],
        #           'actions': [{'pr'/'act' -> ParseResult}]
        #       }]}]}]
        return (pr.tree for pr in itertools.chain(
            self.helpers.values(),
            self.objectives.values(),
            self.collect.values(),
            (info['pr'] for info in self.movements.values() if 'pr' in info),
            (info['pr'] for info in self.warps.values() if 'pr' in info),r))
            

    def emit_helpers(self):
        with open(os.path.join(self.game_dir, 'src', 'helpers.rs'), 'w') as f:
            f.write(f'//! AUTOGENERATED FOR {self.game} - DO NOT MODIFY\n'
                    f'//!\n'
                    f'//! Macro definitions for helpers.\n')
            for name, pr in self.helpers.items():
                args = []
                if '(' in name:
                    name, args = name.split('(', 1)
                    args = args[:-1].split(',')
                id = construct_id('helper', name)
                f.write(f'\n/// {name}\n'
                        f'/// {pr.text}\n'
                        f'#[macro_export]\n'
                        f'macro_rules! {id} {{\n'
                        f'    ({", ". join("$" + a + ":expr" for a in args)}) => {{{{\n'
                        f'        println!("{{}}", "{StringVisitor().visit(pr.tree)}");\n')
                for a in args:
                    f.write(f'        println!("{a} := {{}}", ${a});\n')
                f.write(f'    }}}}\n'
                        f'}}\n')


        
if __name__ == '__main__':
    cmd = argparse.ArgumentParser()
    cmd.add_argument('game', help='Which game to build the graph for')
    args = cmd.parse_args()

    # Things we need to do:
    # load the game's yaml files

    gl = GameLogic(args.game)
    if gl.errors:
        print('\n'.join(gl.errors))
    gl.emit_helpers()

    # Error checking:
    # - Check for unsupported info, bad indents, etc
    # - Parse all rules
    # - Check functions called
    # determine the list of all items in the game
    # build context type
    # build graph
