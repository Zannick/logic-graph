from collections import namedtuple
from itertools import compress

from Utils import typenameof

BitFlagGroup = namedtuple("BitFlagGroup", ['size', 'vars', 'defaults'])

class BitFlagProcessor(object):

    def __init__(self, context_values, settings, item_max_counts):
        self.context_values = context_values
        self.settings = settings
        self.item_max_counts = item_max_counts
        self.flag_groups = []
        self.varmap = {}

    def process(self):
        context_vars = [c for c, val in self.context_values.items() if typenameof(val) == 'bool']
        settings = sorted(s for s, t in self.settings.items() if t['type'] == 'bool')
        items = sorted(i for i, n in self.item_max_counts.items() if n == 1)
        everything = context_vars + settings + items
        while len(everything) > 32:
            sl, everything = everything[:32], everything[32:]
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