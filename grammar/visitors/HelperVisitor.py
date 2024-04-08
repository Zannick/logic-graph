import logging

from Utils import BUILTINS, PLACE_TYPES, config_type

from grammar import RulesParser, RulesVisitor


class HelperVisitor(RulesVisitor):

    def __init__(self, helpers, rules, context_types, data_types, settings):
        self.helpers = helpers
        self.rules = rules
        self.context_types = context_types
        self.data_types = data_types
        self.settings = settings
        self.local_types = {}
        self.ctxdict = {}
        self.name = ''
        self.errors = []

    def visit(self, ctx, name='', ctxdict=None, local_types=None):
        self.name = name
        self.ctxdict = ctxdict or {}
        self.local_types = local_types or {}
        try:
            ret = super().visit(ctx)
        finally:
            self.name = ''
            self.ctxdict = {}
            self.local_types = {}
        return ret

    def _getFullRef(self, ref):
        return self.ctxdict.get(ref, ref)

    def _getValueType(self, valueCtx):
        if isinstance(valueCtx, RulesParser.SettingContext):
            s = str(valueCtx.SETTING())
            # TODO: if the setting is a dict
            return self.settings[s]['type']
        else:
            ref = self._getFullRef(str(valueCtx.REF())[1:])
            if ref in self.context_types:
                return self.context_types[ref]
            elif ref in self.data_types:
                return self.data_types[ref]
            elif ref in self.local_types:
                if self.local_types[ref] == '':
                    logging.warning(f"Rule {self.name} provides ref ^{ref} to functions but we don't know its type yet")
                return self.local_types[ref]
            else:
                self.errors.append(f'Unrecognized ctxvar in rule {self.name}: ^{ref}')
                return ''
        

    def visitInvoke(self, ctx):
        func = str(ctx.FUNC())
        if func in BUILTINS or func in self.rules:
            return self.visitChildren(ctx)
        if func not in self.helpers:
            self.errors.append(f'Unrecognized function {func} in rule {self.name}')
            return self.visitChildren(ctx)
        if args := self.helpers[func]['args']:
            if ctx.ITEM(): t = 'Item'
            elif ctx.LIT(): t = 'str'
            elif ctx.INT(): t = 'int'
            elif ctx.FLOAT(): t = 'float'
            elif ctx.PLACE():
                if len(ctx.PLACE()) != len(args):
                    self.errors.append(f'Rule {self.name} calls function {func} with incorrect number '
                                       f'of args, expected {len(args)}, got {len(ctx.PLACE())}')
                    return
                for i, (a, p) in enumerate(zip(args, ctx.PLACE())):
                    if not a.type:
                        self.errors.append(f'Function {func} must define arg {i} type in order to be called '
                                           f'with Places: rule {self.name}')
                    elif a.type not in PLACE_TYPES:
                        self.errors.append(f'Rule {self.name} calls function {func} with arg {i} PlaceId but '
                                           f'we saw other usage/definition with {a.type}')
                    else:
                        maxgt = PLACE_TYPES.index(a.type)
                        p = str(p)[1:-1]
                        t = config_type(p)
                        if t not in ['str', 'AreaId', 'SpotId']:
                            self.errors.append(f'Rule {self.name} calls function {func} with arg {i} as {t} '
                                               f'but we expected {a.type}')
                        elif p.count('>') != maxgt:
                            self.errors.append(f'Rule {self.name} calls function {func} with arg {i} as '
                                               f'invalid {a.type}: {p}')
                return
            elif ctx.value():
                t = self._getValueType(ctx.value())
                if not t:
                    return self.visitChildren(ctx)
            else:
                self.errors.append(f'Rule {self.name} calls function {func} with no/unrecognized args '
                                   'but args are expected')
                return self.visitChildren(ctx)

            if args[0].type and args[0].type != t:
                self.errors.append(f'Rule {self.name} calls function {func} with args of type {t} '
                                   f'but we saw other usage/definition with {args[0].type}')
            else:
                for i, a in enumerate(args):
                    args[i] = a._replace(type=t)

    def visitItemList(self, ctx):
        for func in ctx.FUNC():
            func = str(func)
            if func in BUILTINS:
                self.errors.append(f'Rule {self.name}: builtin {func!r} not supported in itemList')
            elif rule := self.rules.get(func):
                if rule.rule != 'itemList':
                    self.errors.append(f'Rule {self.name}: itemList only supports itemList helpers, '
                                    f'but rule {func!r} is of type {rule.rule!r}')
            elif func not in self.helpers:
                self.errors.append(f'Unrecognized function {func} in rule {self.name}')
            elif self.helpers[func]['rule'] != 'itemList':
                self.errors.append(f'Rule {self.name}: itemList only supports itemList helpers, '
                                   f'but helper {func!r} is of type {self.helpers[func]["rule"]!r}')
