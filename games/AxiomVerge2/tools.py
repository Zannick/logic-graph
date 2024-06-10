from collections import defaultdict
import math
import os
from pprint import pprint
import sys

sys.path.append(os.path.abspath(os.path.join(os.path.dirname(__file__), '../..')))
from Compiler import GameLogic, treeToString
from Utils import construct_id

import igraph as ig
import leidenalg as la

AV2 = GameLogic('AxiomVerge2')
LOC_SPOTS = {spot['id'] for spot in AV2.spots()
             if ('locations' in spot and any('event' not in loc.get('tags', ()) for loc in spot['locations']))
             or ('hybrid' in spot and any('event' not in h.get('tags', ()) for h in spot['hybrid']))
             or ('actions' in spot and any('$visit' in a['do'] for a in spot['actions']))}

UNFLIPPABLE = ('Antarctica', 'Interior', 'Menu')

def get_spot_graph_coordinates(spot_id):
    spot = AV2.id_lookup[spot_id]
    if 'coord' in spot:    
        area = AV2.id_lookup[construct_id(spot['region'], spot['area'])]
        region = AV2.id_lookup[construct_id(spot['region'])]
        offset = area.get('graph_offset', region.get('graph_offset', [0, 0]))
        return (spot['coord'][0] + offset[0], spot['coord'][1] + offset[1])

def notable_spots_without_map_spot():
    spots = []
    for region in AV2.regions:
        if region['name'] in UNFLIPPABLE:
            continue
        for area in region['areas']:
            for spot in area['spots']:
                if 'coord' not in spot:
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
        if region['name'] in UNFLIPPABLE:
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


def confirm_flipsides():
    unmatched = set()
    unmatched_sets = []
    for spot in AV2.spots():
        if 'coord' not in spot or spot['fullname'] in unmatched:
            continue
        fl = spot['all_data']['flipside']
        if fl != 'SpotId::None':
            sset = set()
            while fl not in sset:
                sset.add(fl)
                if fl == 'SpotId::None':
                    break
                spot = AV2.id_lookup[construct_id(fl)]
                fl = spot['all_data']['flipside']
            unmatched.update(sset)
            unmatched_sets.add(sset)
    pprint(unmatched_sets)

# TODO: can we find nearby coordinates in the other realm to include in this?
# this may require numpy and scipy.spatial.cKDTree

# TODO: breach spots with non-event locations should have flipsides if possible
def notable_breach_exits_without_flipside():
    spots = []
    for region in AV2.regions:
        if 'Breach' not in region['name']:
            continue
        for area in region['areas']:
            for spot in area['spots']:
                if 'coord' not in spot:
                    continue
                if spot['all_data']['flipside'] != 'SpotId::None':
                    continue
                if spot.get('data', {}).get('flipside') == 'SpotId::None':
                    continue
                if 'actions' in spot:
                    spots.append(spot)
                elif 'locations' in spot and any('event' not in loc.get('tags', ()) for loc in spot['locations']):
                    spots.append(spot)
                elif 'hybrid' in spot and any('event' not in h.get('tags', ()) for h in spot['hybrid']):
                    spots.append(spot)
    pprint([
        (spot['fullname'], spot['coord'])
        for spot in spots
    ])

def make_igraph():
    # exclude the upgrades in the menu, since that's connected to everything
    edges = [(x, y, w) for ((x, y), w) in AV2.base_distances.items()
             if x != y and not x.startswith('Menu__Upgrade_Menu') ^ y.startswith('Menu__Upgrade_Menu')]
    g = ig.Graph.TupleList(edges, directed=True, edge_attrs=['weight'])
    for v in g.vs:
        name = v.attributes()['name']
        if c := get_spot_graph_coordinates(name):
            v.update_attributes(x=c[0], y=c[1], shape='circle' if name in LOC_SPOTS else 'hidden')
        else:
            v.update_attributes(x=32, y=32, shape='circle' if name in LOC_SPOTS else 'hidden')
    return g

def subgraph(g: ig.Graph):
    spots = LOC_SPOTS & {
        vertex.attributes()['name']
        for vertex in g.vs
    }
    return g.induced_subgraph(spots)

PARTITION_OPTIONS = {
    'mod': (la.ModularityVertexPartition, {}),
    'surprise': (la.SurpriseVertexPartition, {}),
    # others have resolution_parameter=1.0 by default
    # 'rb': (la.RBConfigurationVertexPartition, {}),  # same as mod at 1.0
    'rber': (la.RBERVertexPartition, {}),
    'cpm': (la.CPMVertexPartition, {}),
    'cpm2': (la.CPMVertexPartition, {'resolution_parameter': 2.0}),
    'cpm.5': (la.CPMVertexPartition, {'resolution_parameter': 0.5}),
    'rb2': (la.RBConfigurationVertexPartition, {'resolution_parameter': 2.0}),
    'rber2': (la.RBERVertexPartition, {'resolution_parameter': 2.0}),
    'rb.5': (la.RBConfigurationVertexPartition, {'resolution_parameter': 0.5}),
    'rber.5': (la.RBERVertexPartition, {'resolution_parameter': 0.5}),
}

def partition(g, p, **kwargs):
    part = la.find_partition(g, p, **kwargs)
    subgraphs = [subgraph(sg) for sg in part.subgraphs()]
    sglist = []
    for sg in subgraphs:
        vs = [v.attributes()['name'] for v in sg.vs]
        if vs:
            sglist.append(vs)
    return part, sglist

def partition_and_show_sub(g, p, filename='data/part.png', **kwargs):
    part, sglist = partition(g, p, **kwargs)
    ig.plot(part).save(os.path.join('.', filename))

    pprint(sglist)

def many_partitions(g):
    d = {
        k: partition(g, p, **kwargs)
        for (k, (p, kwargs)) in PARTITION_OPTIONS.items()
    }
    spot_lookup = defaultdict(dict)
    for (k, (_, sglist)) in d.items():
        for group in sglist:
            for spot in group:
                spot_lookup[spot][k] = (len(group), group)
    return d, spot_lookup

if __name__ == '__main__':
    print('Loaded game logic in var AV2')
    G = make_igraph()
    print('igraph for AV2 made in var G')
