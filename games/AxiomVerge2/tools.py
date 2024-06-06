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
    for region in AV2.regions:
        if region['name'] in ('Antarctica', 'Interior', 'Menu'):
            continue
        for area in region['areas']:
            for spot in area['spots']:
                if 'coord' not in spot:
                    continue
                by_coord[tuple(spot['coord'])].append(spot)
    dups = [
        [s['fullname'] for s in same] for same in by_coord.values()
        # Include all spots that don't have flipside set to default
        # explicitly setting it to default will exclude it so we can mark the passage deliberately one-way
        if len(same) > 1 and any(s['all_data']['flipside'] == 'SpotId::None'
                                 and 'flipside' not in s.get('data', {}) for s in same)
    ]
    pprint(dups)

# TODO: can we find nearby coordinates in the other realm to include in this?
# TODO: breach spots with non-event locations should have flipsides if possible
# TODO: confirm flipsides have each other as flipsides


if __name__ == '__main__':
    print('Loaded game logic in var AV2')
