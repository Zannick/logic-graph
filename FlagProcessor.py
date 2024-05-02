from collections import namedtuple

from Utils import typenameof

BitFlagGroup = namedtuple("BitFlagGroup", ['size', 'vars', 'defaults'])
GroupRange = namedtuple("GroupRange", ['start_group', 'start_index', 'end_group', 'end_index'])

MAX_GROUP_SIZE = 64

class BitFlagProcessor(object):

    def __init__(self, context_values, settings, item_max_counts, canon_places, unused_map_tiles):
        self.context_values = context_values
        self.settings = settings
        self.item_max_counts = item_max_counts
        self.canon_places = canon_places
        self.unused_map_tiles = unused_map_tiles
        self.flag_groups = []
        self.varmap = {}
        self.visit_groups = None
        self.skip_groups = None
        self.visit_spot_groups = {}
        self.visit_area_groups = {}
        self.visit_region_groups = {}
        self.skip_spot_groups = {}
        self.skip_area_groups = {}
        self.skip_region_groups = {}

    def process(self):
        context_vars = [c for c, val in self.context_values.items()
                        if typenameof(val) == 'bool' and c not in self.unused_map_tiles]
        items = sorted(i for i, n in self.item_max_counts.items() if n == 1)
        visits = sorted('VISITED_' + canon for canon in self.canon_places)
        basic = context_vars + items
        basic_len = len(basic)
        everything = basic + visits
        while len(everything) > MAX_GROUP_SIZE:
            sl, everything = everything[:MAX_GROUP_SIZE], everything[MAX_GROUP_SIZE:]
            self.flag_groups.append(BitFlagGroup(len(sl), sl, [
                e for e in sl if e in context_vars and self.context_values[e]]))
        if everything:
            size = len(everything) - 1
            size = max(8, 2 ** size.bit_length())
            self.flag_groups.append(BitFlagGroup(size, everything, [
                e for e in everything if e in context_vars and self.context_values[e]]))
        for i, fg in enumerate(self.flag_groups):
            self.varmap.update({
                v: i+1
                for v in fg.vars
            })
        
        # the group + index the first VISITED flag falls in
        vmin, vmindex = divmod(basic_len, MAX_GROUP_SIZE)
        # the group + index the last VISITED flag falls in
        vmax, vmaxindex = divmod(basic_len + len(self.canon_places) - 1, MAX_GROUP_SIZE)
        # the group + index the first SKIPPED flag falls in
        smin, smindex = divmod(basic_len + len(self.canon_places), MAX_GROUP_SIZE)
        # the group + index the last SKIPPED flag falls in
        smax = len(self.flag_groups) - 1
        smaxindex = (len(everything) - 1) % MAX_GROUP_SIZE
        self.visit_groups = GroupRange(vmin + 1, vmindex, vmax + 1, vmaxindex)
        self.skip_groups = GroupRange(smin + 1, smindex, smax + 1, smaxindex)

    def get_groups_for_place(self, mode, loc_ids):
        start = mode + min(loc_ids)
        end = mode + max(loc_ids)
        gmin = self.varmap[start]
        gmindex = self.flag_groups[gmin - 1].vars.index(start)
        gmax = self.varmap[end]
        gmaxindex = self.flag_groups[gmax - 1].vars.index(end)
        return GroupRange(gmin, gmindex, gmax, gmaxindex)
    
    # TODO: Checking visits for a region or area will need both the range of default canon names (if there is one)
    # plus all canon names for locations in the region/area
