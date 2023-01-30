import argparse
import itertools
import os
import yaml

base_dir = os.path.dirname(os.path.realpath(__file__))

from grammar.build import maybe_generate
maybe_generate(False)

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


# Maybe instead of collating all the rules, we should just update in-place
# with the trees, while we collect the errors

def collate_rules(gameinfo):
    boolExprs = {}
    actExprs = {}

    boolExprs.update({f'helpers:{o}': v for o, v in gameinfo['helpers'].items()})
    boolExprs.update({f'objectives:{o}': v for o, v in gameinfo['objectives'].items()})
    actExprs.update({f'collect:{o}': v for o, v in gameinfo['collect'].items()})
    for category in ['movements', 'warps']:
        boolExprs.update({f'{category}:{m}': v['req'] for m, v in gameinfo[category].items()
                          if 'req' in v})

    for region in gameinfo['regions']:
        rname = region.get('short', region['name'])
        for area in gameinfo['areas']:
            aname = area['name']
            if 'exits' in area:
                boolExprs.update({f'{rname} > {aname} ==> {e["to"]}': e['req']
                                  for e in area['exits']
                                  if 'req' in e})
            for spot in area['spots']:
                sname = spot['name']
                if 'locations' in spot:
                    boolExprs.update({f'{rname} > {aname} > {sname} {l["name"]}': l['req']
                                      for l in spot['locations']
                                      if 'req' in l})
                if 'exits' in spot:
                    boolExprs.update({f'{rname} > {aname} > {sname} ==> {e["to"]}': e['req']
                                      for e in spot['exits']
                                      if 'req' in e})
                if 'hybrid' in spot:
                    boolExprs.update({f'{rname} > {aname} > {sname} ==> {e["to"]}': e['req']
                                      for e in spot['hybrid']
                                      if 'req' in e})
                if 'actions' in spot:
                    boolExprs.update({f'{rname} > {aname} > {sname} {a["name"]}': a['req']
                                      for a in spot['actions']
                                      if 'req' in a})
                    actExprs.update({f'{rname} > {aname} > {sname} {a["name"]}': a['do']
                                     for a in spot['actions']})



if __name__ == '__main__':
    cmd = argparse.ArgumentParser()
    cmd.add_argument('game', help='Which game to build the graph for')
    args = cmd.parse_args()

    # Things we need to do:
    # load the game's yaml files

    game = load_game_yaml(args.game)

    # Error checking:
    # - Check for unsupported info, bad indents, etc
    # - Parse all rules
    # - Check functions called
    # determine the list of all items in the game
    # build context type
    # build graph
