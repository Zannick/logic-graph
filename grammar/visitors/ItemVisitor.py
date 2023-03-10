from collections import Counter, defaultdict
import logging
import re

import Utils
from Utils import construct_id

from grammar import RulesVisitor


class ItemVisitor(RulesVisitor):

    def __init__(self, settings, vanilla_items):
        self.item_uses = Counter()
        self.item_max_counts = defaultdict(int)
        self.settings = settings
        self.vanilla_items = vanilla_items
        self.name = ''
        self.errors = []

    def visit(self, tree, name=''):
        self.name = name
        try:
            return super().visit(tree)
        finally:
            self.name = ''

    def _count_items(self, ctx):
        if self.name.startswith('objectives'):
            for item in ctx.ITEM():
                it = str(item)
                if it not in self.vanilla_items:
                    logging.warning("%s references undefined item %r and may be impossible", self.name, it)
                self.item_uses[it] += 1
                self.item_max_counts[it] = max(1, self.item_max_counts[it])
        else:
            for item in ctx.ITEM():
                it = str(item)
                self.item_uses[it] += 1
                self.item_max_counts[it] = max(1, self.item_max_counts[it])
        return self.visitChildren(ctx)

    visitInvoke = visitRefInList = visitMatchRefBool = _count_items

    def _switch_count(self, ctx):
        it = str(ctx.ITEM())
        self.item_uses[it] += 1
        mc = max(int(str(x)) for x in ctx.INT())
        self.item_max_counts[it] = max(mc, self.item_max_counts[it])
        return self.visitChildren(ctx)

    visitPerItemBool = visitPerItemNum = visitPerItemStr = _switch_count

    # These will either need to check for the items used in the calls,
    # or the rules could be removed. (Other rules using REF don't use count,
    # so it's sufficient to count the provided item as 1 in the calling rule.)
    def _switch_warn(self, ctx):
        # TODO: check the type of REF, we don't need to warn if the type is not Item
        if ctx.REF() and ctx.INT():
            logging.warning('Rule %r checks for count of ref: not supported', self.name)
        return self.visitChildren(ctx)

    visitPerRefInt = visitPerRefStr = _switch_warn

    def _count_one(self, ctx):
        if ctx.ITEM():
            it = str(ctx.ITEM())
            self.item_uses[it] += 1
            self.item_max_counts[it] = max(1, self.item_max_counts[it])
        return self.visitChildren(ctx)

    visitRefEq = visitFuncNum = visitValue = visitOneItem = _count_one

    def visitItemCount(self, ctx):
        it = str(ctx.ITEM())
        self.item_uses[it] += 1
        if ctx.SETTING():
            s = str(ctx.SETTING())
            if sd := self.settings.get(s):
                if sd['type'] != 'int':
                    self.errors.append(f'Rule {self.name} uses setting {s} as int, but it is {sd["type"]}')
                    return self.visitChildren(ctx)

                m = sd.get('max', 1024)
                if 'max' not in sd:
                    logging.getLogger('').warning('Rule %r looks at a setting value of item %s, setting max to 1024', self.name, it)
                self.item_max_counts[it] = max(m, self.item_max_counts[it])
            # There would be an error added here but it is taken care of by SettingVisitor
        else:
            ct = int(str(ctx.INT()))
            self.item_max_counts[it] = max(ct, self.item_max_counts[it])
        return self.visitChildren(ctx)

    def visitOneLitItem(self, ctx):
        it = construct_id(ctx.LIT())
        self.item_uses[it] += 1
        self.item_max_counts[it] = max(1, self.item_max_counts[it])
        return self.visitChildren(ctx)
