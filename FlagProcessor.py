from collections import namedtuple

from Utils import typenameof

BitFlagGroup = namedtuple("BitFlagGroup", ['size', 'vars', 'defaults'])
GroupRange = namedtuple("GroupRange", ['start_group', 'start_index', 'end_group', 'end_index'])

MAX_GROUP_SIZE = 64

class BitFlagProcessor(object):

    def __init__(self, context_values, settings, item_max_counts, locations):
        self.context_values = context_values
        self.settings = settings
        self.item_max_counts = item_max_counts
        self.locations = locations
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
        context_vars = [c for c, val in self.context_values.items() if typenameof(val) == 'bool']
        settings = sorted(s for s, t in self.settings.items() if t['type'] == 'bool')
        items = sorted(i for i, n in self.item_max_counts.items() if n == 1)
        visits = sorted('VISITED_' + loc['id'] for loc in self.locations)
        skips = sorted('SKIPPED_' + loc['id'] for loc in self.locations)
        basic = context_vars + settings + items
        basic_len = len(basic)
        everything = basic + visits + skips
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
        vmax, vmaxindex = divmod(basic_len + len(self.locations) - 1, MAX_GROUP_SIZE)
        # the group + index the first SKIPPED flag falls in
        smin, smindex = divmod(basic_len + len(self.locations), MAX_GROUP_SIZE)
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

    def process_place_groups(self, regions):
        for region in regions:
            if not region['loc_ids']:
                continue
            self.visit_region_groups[region['id']] = self.get_groups_for_place('VISITED_', region['loc_ids'])
            self.skip_region_groups[region['id']] = self.get_groups_for_place('SKIPPED_', region['loc_ids'])
            for area in region['areas']:
                if not area['loc_ids']:
                    continue
                self.visit_area_groups[area['id']] = self.get_groups_for_place('VISITED_', area['loc_ids'])
                self.skip_area_groups[area['id']] = self.get_groups_for_place('SKIPPED_', area['loc_ids'])
                for spot in area['spots']:
                    if not spot['loc_ids']:
                        continue
                    self.visit_spot_groups[spot['id']] = self.get_groups_for_place('VISITED_', spot['loc_ids'])
                    self.skip_spot_groups[spot['id']] = self.get_groups_for_place('SKIPPED_', spot['loc_ids'])