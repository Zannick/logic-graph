from collections import defaultdict
import hashlib
import math
import os
import pathlib
from pprint import pprint
import sys
from typing import Any, Dict, List, Tuple

ROOT = (pathlib.Path(__file__).parent / '../..').resolve()
sys.path.append(str(ROOT))
SRCDIR = pathlib.Path(__file__).parent / 'src'
from grammar import RulesParser
from Compiler import GameLogic, get_exit_target, treeToString, LOCAL_REFERENCE_RE
from Utils import construct_id

import igraph as ig
import leidenalg as la
import networkx as nx

AV2 = GameLogic('AxiomVerge2')

LOC_SPOTS = {spot['id'] for spot in AV2.spots()
             if ('locations' in spot and any('event' not in loc.get('tags', ()) for loc in spot['locations']))
             or ('hybrid' in spot and any('event' not in h.get('tags', ()) for h in spot['hybrid']))
             or ('actions' in spot and any('$visit' in a['do'] for a in spot['actions']))}

UNFLIPPABLE = ('Antarctica', 'Interior', 'Menu')

class UnionFind:
    def __init__(self, ids):
        self.parent = {i: i for i in ids}
        self.size = {i: 1 for i in ids}

    def representative(self, sp):
        p = self.parent[sp]
        flatten = []
        while p != self.parent[p]:
            flatten.append(sp)
            sp = p
            p = self.parent[sp]
        for sp in flatten:
            self.parent[sp] = p
        return p
    
    def set_size(self, sp):
        return self.size[self.representative(sp)]
    
    def sets(self):
        return [(sp, self.size[sp]) for sp, p in self.parent.items() if sp == p]
    
    def nontrivial_sets(self):
        return [(x, y) for x,y in self.sets() if y > 1]
    
    def trivial_sets(self):
        return [x for x,y in self.sets() if y == 1]
    
    def union(self, sp1, sp2):
        p1 = self.representative(sp1)
        p2 = self.representative(sp2)

        if p1 == p2:
            return
        if self.size[p1] < self.size[p2]:
            self.parent[p1] = p2
            self.size[p2] += self.size[p1]
        else:
            self.parent[p2] = p1
            self.size[p1] += self.size[p2]

    def flatten(self):
        for sp in self.parent:
            self.representative(sp)

    def full_set(self, sp):
        self.flatten()
        common = self.parent[sp]
        return [sp for sp, p in self.parent.items() if p == common]

    def clique_map(self):
        cm = defaultdict(list)
        for sp, p in self.parent.items():
            cm[p].append(sp)
        return cm


def merge_free():
    uf = UnionFind([sp['id'] for sp in AV2.spots()])
    fd = AV2.free_distances
    for sp1, dmap in fd.items():
        for sp2 in dmap:
            if sp1 < sp2 and sp1 in fd.get(sp2, {}):
                uf.union(sp1, sp2)
    # Spots that go to a single union with any free edge can be merged into it
    # only if it has nothing interesting: no locations, no actions, no map tiles not in the union
    #for sp in AV2.spots():
    #    sp = sp['id']
    #    neighbors = {uf.representative(sp2) for sp2 in AV2.base_distances[sp]}
    #    if len(neighbors) == 1 and fd.get(sp):  # test fd is nonempty since fd is a subset of bd
    #        uf.union(sp, neighbors.pop())

    # Look for free cycles (above found 2-cycles)
    dg = nx.DiGraph()
    dg.add_nodes_from(uf.parent.values())
    for sp1, dmap in fd.items():
        s = uf.representative(sp1)
        dg.add_edges_from((s, uf.representative(t)) for t in dmap)
    for cycle in nx.simple_cycles(dg):
        for s in cycle[1:]:
            uf.union(cycle[0], s)
    return uf

# find all edges between cliques, so we can build a representative
# we might not need to analyze the context that much, if we can make a new ContextAccess type
# that contains bitflags/IntComps for access stuff

def make_edge_lists(uf: UnionFind):
    cliques = uf.clique_map()
    free_conns = defaultdict(set)
    edges = {}
    edge_texts = defaultdict(dict)
    count = 0
    trim = 0
    for c1, cset in cliques.items():
        ext = edges[c1] = defaultdict(list)
        for s1 in cset:
            if s1 in AV2.free_distances:
                free_conns[c1].update(uf.representative(s) for s in AV2.free_distances[s1])
                free_conns[c1].discard(c1)
            for s2, edge in AV2.edges_from(s1):
                # for now, skip context edges that aren't pre-calculated data
                if s2[0] == '^':
                    continue
                c2 = uf.representative(s2)
                if c2 == c1:
                    continue
                if 'req' not in edge:
                    continue
                text = edge['req']
                if '^_' in text:
                    sp = AV2.id_lookup[s1]
                    def replace(m):
                        return '^' + AV2.lookup_local_context(m.group(1), sp['region'], sp['area'])
                    text = LOCAL_REFERENCE_RE.sub(replace, text)
                    edge['reqlr'] = text
                    edge['pr'].tree.local_ctx = AV2.get_local_ctx(edge)
                else:
                    # Any shared objects may have this overwritten, but then again, they should all be None if shared
                    edge['pr'].tree.local_ctx = None
                ext[c2].append(edge['pr'].tree)
        # at this point we have all the edges from c1, now we just combine them. For each c2:
        # - if it's free, it's free.
        # - if there's more than one, combine it via OR
        # - but first split all the ones that are ORs to get the bool atoms, then we can deduplicate
        # - if any atoms are ANDs that contain other atoms, we can discard them: A OR (A AND B) => A
        # - if any are fully negations (may be hard to tell), the whole is true: A OR NOT A => True
        # - might also want to check for any rearranged atoms like A AND B is the same as B AND A
        for c2, atoms in ext.items():
            if not atoms:
                continue
            text = f'{c1} -> {c2}: OR[ {" , ".join(treeToString(atom, atom.local_ctx) for atom in atoms)} ]'
            clist = cfold(atoms)
            ocount = len(clist)
            clist = list(filter(None, clist))
            trim += ocount - len(clist)
            if not clist:
                print(text, '=> True')
                free_conns[c1].add(c2)
            elif clist == [False]:
                print(text, '=> False')
            else:
                if len(clist) == 1:
                    t2 = treeToString(clist[0], clist[0].local_ctx)
                else:
                    t2 = f'OR [ {" , ".join(treeToString(atom, atom.local_ctx) for atom in clist)} ]'
                if text != t2:
                    if 'None' in t2 or ocount > len(clist):
                        print(text, '=>', t2)
                    count += 1
                edge_texts[c1][c2] = t2

    print(f'{count} clique-clique edges improved, with {trim} trims')
    return free_conns, edges, edge_texts


def cfold(orlist: List[Dict[str, Any]]):
    atoms = []
    texts = set()
    queue = [c for c in orlist]
    empty = True

    def add_if_unique(atom):
        text = treeToString(atom, atom.local_ctx)
        if text not in texts:
            texts.add(text)
            atoms.append(atom)

    while queue:
        tree = queue.pop(0)
        if not isinstance(tree, RulesParser.BoolExprContext):
            add_if_unique(tree)
            continue
        if tree.TRUE():
            return []
        if tree.FALSE():
            empty = False
            continue
        if tree.OR():
            s1, s2 = tree.boolExpr()
            s1.local_ctx = s2.local_ctx = tree.local_ctx
            queue.append(s1)
            queue.append(s2)
            continue
        add_if_unique(tree)

    if not atoms:
        return [] if empty else [empty]

    # Now go back and check ANDs and NOTs
    for i, atom in enumerate(atoms):
        if not isinstance(atom, RulesParser.BoolExprContext):
            continue
        # If we have both A and NOT A in our OR atoms, the whole result (A or NOT A) is true.
        if atom.NOT():
            text = treeToString(atom.getChild(0), atom.local_ctx)
            if text in texts:
                return []
        # If we have an atom A in an AND expression, like A or (A AND B), we can remove the whole AND term
        # and just keep A.
        if atom.AND():
            atom.children = atom.children[::-1]
            rev = treeToString(atom, atom.local_ctx)
            atom.children = atom.children[::-1]
            # (A AND B) or (B AND A) => A AND B
            # Assuming we don't have A AND A ever
            if rev in texts:
                t1 = treeToString(atom.getChild(0), atom.local_ctx)
                t2 = treeToString(atom.getChild(1), atom.local_ctx)
                if t1 > t2:  # Only keep the one where t1 < t2
                    atoms[i] = None
                continue
            queue = list(atom.getChildren())
            while queue:
                c = queue.pop(0)
                if isinstance(c, RulesParser.BoolExprContext):
                    if c.AND():
                        queue.extend(c.getChildren())
                        continue
                    # for an OR, we have to test each of them
                    if c.OR():
                        # (A or B or ((A or B) and C)
                        if all(treeToString(orchild, atom.local_ctx) in texts for orchild in c.getChildren()):
                            atoms[i] = None
                            break
                text = treeToString(c, atom.local_ctx)
                if text in texts:
                    atoms[i] = None
    
    return atoms
    

def get_movement_cost(movement):
    if m := AV2.exit_movements.get(movement):
        if 'price_per_sec' in m or 'base_price' in m:
            return m.get('costs', AV2.default_price_type)
    return None

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

def find_combinable():
    for spot in AV2.spots():
        categories = defaultdict(list)
        for local in spot.get('local', ()):
            if 'thru' in local:
                continue
            cat = ('move', None, local['to'])
            categories[cat].append(f'Local movement {spot["fullname"]} -> {local["to"]}')
        for exit in spot.get('exits', ()):
            if 'price' in exit:
                cost = f'{exit["price"]} {exit.get('costs', AV2.default_price_type)}'
            else:
                cost = get_movement_cost(exit.get('movement'))
            cat = ('move', cost, exit['to'])
            categories[cat].append(exit['fullname'])
        for loc in spot.get('locations', []) + spot.get('hybrid', []):
            if 'price' in loc:
                cost = f'{loc["price"]} {loc.get('costs', AV2.default_price_type)}'
            else:
                cost = get_movement_cost(loc.get('movement'))
            cat = ('loc', cost, loc['item'], loc.get('canon'), loc.get('to'))
            categories[cat].append(loc['fullname'])
        for act in spot.get('actions', ()):
            if 'price' in act:
                cost = f'{act["price"]} {act.get('costs', AV2.default_price_type)}'
            else:
                cost = get_movement_cost(act.get('movement'))
            cat = ('act', cost, act.get('to'))
            categories[cat].append(act['fullname'])
        for cat, places in categories.items():
            if len(places) > 1:
                pprint((f'Combinable {cat}:', places))

def penalty_req_conflicts():
    odd = []
    for point in AV2.all_points():
        if 'req' in point and 'penalties' in point:
            if ' or ' not in point['req']:
                not_first = f'not {point['req'].split(' and ', 1)[0]}'
                if any(not_first in p.get('when', '') for p in point['penalties']):
                    odd.append(point['fullname'])
    pprint(odd)

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
        same for same in by_coord.values()
        # Include all spots that don't have flipside set to default
        # explicitly setting it to default will exclude it so we can mark the passage deliberately one-way
        if len(same) > 1 and any(s['all_data']['flipside'] == 'SpotId::None'
                                 and 'flipside' not in s.get('data', {}) for s in same)
    ]
    if not dups:
        print('All set!')
        return
    print('      data:\n        flipside: SpotId::None')
    to_print = {}
    for duplist in dups:
        first, second = duplist[:2]
        if 'flipside' not in first.get('data', {}):
            to_print[first['fullname']] = second['fullname']
        if 'flipside' not in second.get('data', {}):
            to_print[second['fullname']] = first['fullname']
    for spot in AV2.spots():
        name = spot['fullname']
        if flipside := to_print.get(name):
            print(f'{name}:\n      data:\n        flipside: {flipside}')


def confirm_flipsides():
    unmatched = set()
    unmatched_sets = []
    for spot in AV2.spots():
        if 'coord' not in spot or spot['fullname'] in unmatched:
            continue
        fl = spot['all_data']['flipside']
        if fl != 'SpotId::None':
            sset = {spot['fullname']}
            while fl not in sset and fl != 'SpotId::None':
                sset.add(fl)
                spot = AV2.id_lookup[construct_id(fl)]
                fl = spot['all_data']['flipside']
            if len(sset) != 2:
                unmatched.update(sset)
                unmatched_sets.append(sset)
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

def too_expensive():
    expensive_exits = []
    for exit in AV2.exits():
        if 'price' not in exit or exit.get('costs') != 'energy':
            continue
        if exit['price'] > 450:
            expensive_exits.append(exit)

    pprint([
        (exit['fullname'], exit['price'])
        for exit in expensive_exits
    ])

def missing_shockwave_price():
    missing = []
    for point in AV2.all_points():
        if 'req' in point and '$shockwave' in point['req'] and 'price' not in point:
            missing.append(point['fullname'])
    if not missing:
        print('All set!')
    else:
        pprint(missing)

def missing_reverse_exits():
    missing = []
    for exit in AV2.exits():
        t = exit.get('tags', ())
        if 'oneway' in t or 'req' in exit:
            continue
        if 'xshift' in t or 'xdoor' in t:
            exit_spot = construct_id(exit['region'], exit['area'], exit['spot'])
            sp = get_exit_target(exit)
            spot = AV2.id_lookup[sp]
            for ex in spot.get('exits', ()):
                if get_exit_target(ex) != exit_spot:
                    continue
                if 'xshift' in t and 'xshift' not in ex.get('tags', ()):
                    missing.append(f"{exit['fullname']} reverse: {ex['fullname']} missing xshift tag")
                elif 'xdoor' in t and 'xdoor' not in ex.get('tags', ()):
                    missing.append(f"{exit['fullname']} reverse: {ex['fullname']} missing xdoor tag")
                break
            else:
                missing.append(f"{exit['fullname']} missing a reverse edge from {spot['fullname']}")
    if not missing:
        print('All set!')
    else:
        pprint(missing)

def repeated_items():
    pprint({k: v for k, v in AV2.placed_item_counts.items() if v > 1})

def make_igraph():
    edges = [(x, y, w) for (x, table) in AV2.basic_distances.items() for y, w in table.items()]
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

def sglist_from_partition(part):
    subgraphs = [subgraph(sg) for sg in part.subgraphs()]
    sglist = []
    for sg in subgraphs:
        vs = [v.attributes()['name'] for v in sg.vs]
        if len(vs) > 1:
            sglist.append(vs)
    return sglist

def partition(g, p, **kwargs):
    part = la.find_partition(g, p, n_iterations=-1, **kwargs)
    sglist = sglist_from_partition(part)
    return part, sglist

def optimize(p, part, **kwargs):
    part2 = p.FromPartition(part, **kwargs)
    opt = la.Optimiser()
    fixed = {v.attributes()['name']
             for i, sg in enumerate(part.subgraphs())
             for v in sg.vs if part.total_weight_in_comm(i) < 1000}
    opt.optimise_partition(part2, n_iterations=-1,
                           is_membership_fixed=[v.attributes()['name'] in fixed for v in part2.graph.vs])
    return part2, sglist_from_partition(part2)

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

def hash_src_files():
    for fn in SRCDIR.glob('**/*.rs'):
        with fn.open('rb') as f:
            h = hashlib.file_digest(f, hashlib.sha256)
        print(h.hexdigest(), fn.relative_to(ROOT).as_posix())

if __name__ == '__main__':
    print('Loaded game logic in var AV2')
    if AV2.errors:
        print('\n'.join(AV2.errors))
    else:
        G = make_igraph()
        print('igraph for AV2 made in var G')
