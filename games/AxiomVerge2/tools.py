from collections import defaultdict
import math
import os
from pprint import pprint
import sys

sys.path.append(os.path.abspath(os.path.join(os.path.dirname(__file__), '../..')))
import Compiler

AV2 = Compiler.GameLogic('AxiomVerge2')

def notable_spots_without_map_spot():
    spots = []
    for area in AV2.areas():
        for spot in area['spots']:
            if 'coord' not in spot or spot['region'] in ('Interior', 'Menu'):
                continue
            if tnames := spot.get('tilenames'):
                if dmap_spots := area.get('datamap', {}).get('map_spot'):
                    if any(t in dmap_spots for t in tnames):
                        continue
            data = spot['all_data']
            if data['map_spot'] != 'SpotId::None':
                continue
            # Notability:
            # - if the spot has a flipside (might switch and then warp)
            # - if the spot has a non-event location, hybrid, or action
            # - if the spot is marked keep (might be used for recall then warp)
            if data['flipside'] != 'SpotId::None' or spot.get('keep'):
                spots.append(spot)
            elif 'actions' in spot:
                spots.append(spot)
            elif 'locations' in spot and any('event' not in loc.get('tags', ()) for loc in spot['locations']):
                spots.append(spot)
            elif 'hybrid' in spot and any('event' not in h.get('tags', ()) for h in spot['hybrid']):
                spots.append(spot)
    pprint([
        (s['fullname'], (a := int(math.floor(s['coord'][0])), b := int(math.floor(s['coord'][1])), a+1, b+1))
        for s in spots
    ])


def same_coordinates_no_flipside():
    by_coord = defaultdict(list)
    for spot in AV2.spots():
        if 'coord' not in spot or spot['region'] in ('Interior', 'Menu'):
            continue
        by_coord[tuple(spot['coord'])].append(spot)
    dups = [
        [s['fullname'] for s in same] for same in by_coord.values()
        if len(same) > 1 and any(s['all_data']['flipside'] == 'SpotId::None' for s in same)
    ]
    pprint(dups)


if __name__ == '__main__':
    print('Loaded game logic in var AV2')
