from collections import defaultdict
import logging

import Utils
from Utils import construct_id

from grammar import RulesVisitor


class SettingVisitor(RulesVisitor):

    def __init__(self, context_types, settings):
        self.context_types = context_types
        self.settings = settings
        self.setting_options = defaultdict(set)
        self.name = ''
        self.ctxdict = {}
        self.errors = []

    def _getFullRef(self, ref: str):
        return self.ctxdict.get(ref, '$' + ref)

    def _checkSetting(self, s: str):
        if s not in self.settings:
            self.errors.append(f'Unrecognized setting in rule {self.name}: {s}')

    def visit(self, tree, name:str ='', ctxdict=None):
        self.name = name
        self.ctxdict = ctxdict or {}
        try:
            ret = super().visit(tree)
        finally:
            self.name = ''
            self.ctxdict = {}
        return ret

    def _checkType(self, setting: str, type: str):
        if self.settings[setting]['type'] != type:
            self.errors.append(f'Rule {self.name} uses {setting} as {type} '
                               'but {setting} is defined as {self.settings[setting]["type"]}')

    def _perSetting(self, ctx):
        s = str(ctx.SETTING())
        self._checkSetting(s)
        if ctx.INT():
            self._checkType(s, 'int')
            self.setting_options[s] |= {int(str(i)) for i in ctx.INT()}
        elif ctx.LIT():
            self._setType(s, 'str')
            self.setting_options[s].update(map(str, ctx.LIT()))
        return self.visitChildren(ctx)
    visitPerSettingInt = visitPerSettingStr = visitPerSettingBool = _perSetting

    def visitRefEq(self, ctx):
        ref = self._getFullRef(str(ctx.REF())[1:])
        s = str(ctx.SETTING())
        self._checkSetting(s)
        self.setting_options[s].add('^' + ref)
        return self.visitChildren(ctx)

    def visitSetting(self, ctx):
        s = str(ctx.SETTING())
        self._checkSetting(s)
        self.setting_options[s].add('?')
        return self.visitChildren(ctx)

    def visitItemCount(self, ctx):
        if ctx.SETTING():
            s = str(ctx.SETTING())
            self._checkSetting(s)
            self._checkType(s, 'int')
            self.setting_options[s].add('?')
        return self.visitChildren(ctx)
