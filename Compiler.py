import argparse
from collections import namedtuple
import itertools
import os
import yaml

base_dir = os.path.dirname(os.path.realpath(__file__))

from grammar import parseRule, parseAction

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


def load_game_yaml(game):
    game_dir = os.path.join(base_dir, 'games', game)
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


class GameLogic(object):

    def __init__(self, gameinfo):
        self.errors = []
        self._info = gameinfo
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
            for area in region['areas']:
                aname = area['name']
                for e in area.get('exits', ()):
                    if 'req' in e:
                        e['pr'] = _parseExpression(
                                e['req'], e['to'], f'{rname} > {aname}', ' ==> ')
                        self.errors.extend(e['pr'].errors)

                for spot in area['spots']:
                    sname = spot['name']
                    fullname = f'{rname} > {aname} > {sname}'
                    for loc in spot.get('locations', ()):
                        if 'req' in loc:
                            loc['pr'] = _parseExpression(
                                    loc['req'], loc['name'], fullname, ' ')
                            self.errors.extend(loc['pr'].errors)
                    for eh in spot.get('exits', []) + spot.get('hybrid', []):
                        if 'req' in eh:
                            eh['pr'] = _parseExpression(
                                    eh['req'], eh['to'], fullname, ' ==> ')
                            self.errors.extend(eh['pr'].errors)

                    for act in spot.get('actions', ()):
                        if 'req' in act:
                            act['pr'] = _parseExpression(
                                    act['req'], act['name'] + ' req', fullname, ' ')
                            self.errors.extend(act['pr'].errors)
                        act['act'] = parseAction(
                                act['do'], name=f'{fullname} {act["name"]}:do')
                        self.errors.extend(act['act'].errors)



if __name__ == '__main__':
    cmd = argparse.ArgumentParser()
    cmd.add_argument('game', help='Which game to build the graph for')
    args = cmd.parse_args()

    # Things we need to do:
    # load the game's yaml files

    game = load_game_yaml(args.game)
    gl = GameLogic(game)
    if gl.errors:
        print('\n'.join(gl.errors))

    # Error checking:
    # - Check for unsupported info, bad indents, etc
    # - Parse all rules
    # - Check functions called
    # determine the list of all items in the game
    # build context type
    # build graph
