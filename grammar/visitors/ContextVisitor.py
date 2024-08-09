from grammar import RulesParser, RulesVisitor


class ContextVisitor(RulesVisitor):

    def __init__(self, context_types, context_values, data_types, data_values, data_defaults):
        self.ctxdict = {}
        self.context_types = context_types
        self.local_types = {}
        self.data_types = data_types
        self.name = ''
        self.errors = []
        self.values = {
            ctx: {context_values[ctx]}
            for ctx, t in self.context_types.items()
            if t.startswith('enums::')
        } | {
            ctx: {data_defaults[ctx]} | set(data_values[ctx].values())
            for ctx, t in self.data_types.items()
            if t.startswith('enums::')
        }
        self.swap_pairs = set()
        self.named_spots = set()
        self.used_map_tiles = set()
        self.ref = ''

    def visit(self, tree, name:str ='', ctxdict=None, local_types=None):
        if self.name:
            # For recursive cases
            return super().visit(tree)

        self.name = name
        self.ctxdict = ctxdict or {}
        self.local_types = local_types or {}
        try:
            return super().visit(tree)
        finally:
            self.name = ''
            self.ctxdict = {}
            self.local_types = {}

    def _checkRef(self, ref):
        if ref not in self.ctxdict and ref not in self.local_types:
            self.errors.append(f'Undefined ctx property ^{ref} in {self.name}')
        if ref in self.ctxdict and ref.startswith('map__'):
            self.used_map_tiles.add(ref)

    def _getType(self, ref):
        if ref not in self.context_types and ref not in self.data_types and ref not in self.local_types:
            self.errors.append(f'Unknown type for ctx property ^{ref} in {self.name}')
            return None
        if ref in self.local_types:
            return self.local_types[ref]
        if ref in self.context_types:
            return self.context_types[ref]
        return self.data_types[ref]

    def _checkTypes(self, ref1, ref2):
        t1 = self._getType(ref1)
        t2 = self._getType(ref2)
        if not t1 or not t2:
            return

        if t1 != t2:
            self.errors.append(f'Type mismatch between ctx properties ^{ref1} ({t1}), '
                               f'^{ref2} ({t2}) in {self.name}')

    def visitInvoke(self, ctx):
        if ctx.PLACE():
            pl = str(ctx.PLACE())[1:-1]
            if pl.count('>') == 2:
                self.named_spots.add(pl)
        self.visitChildren(ctx)

    def visitRef(self, ctx):
        self._checkRef(str(ctx.REF()[-1])[1:])
        if len(ctx.REF()) == 2:
            ref0 = str(ctx.REF(0))[1:]
            self._checkRef(ref0)
            t = self._getType(ref0)
            # TODO: Add data lookups for areas and regions
            if t != 'SpotId':
                self.errors.append(f'Indirect data reference {ctx.getText()} in {self.name} '
                                   f'requires a SpotId but ^{ref0} is {t}')
        if ctx.PLACE():
            pl = str(ctx.PLACE())
            if pl.count('>') != 2:
                self.errors.append(f'Indirect data reference {ctx.getText()} in {self.name} '
                                   f'requires a Spot name')
            # we don't need to mark this spot as not-to-be-condensed

    # Anything that could return a str needs to be visited and return a collection of options
    # plus anything that compares a ref to a str should update that ref

    def visitStr(self, ctx: RulesParser.StrContext):
        if ctx.LIT():
            return {str(ctx.LIT())[1:-1]}
        return super().visitStr(ctx)

    def visitCmpStr(self, ctx: RulesParser.CmpStrContext):
        if not ctx.value().ref():
            return super().visitCmpStr(ctx)
        
        ref = str(ctx.value().ref().REF()[-1])[1:]
        self.visit(ctx.value().ref())
        self.values[ref].add(str(ctx.LIT())[1:-1])

    # value is SETTING or REF -- TODO: multiple REFs or SETTING+REF with the same enum type
    
    def _getAllStrReturns(self, ctx):
        s = set()
        for el in ctx.str_():
            s.update(self.visit(el))
        return s
    
    def _getAllLitReturns(self, ctx):
        s = set()
        for el in ctx.LIT():
            s.add(str(el)[1:-1])
        return s

    def visitCondStr(self, ctx: RulesParser.CondStrContext):
        for el in ctx.boolExpr():
            self.visit(el)
        return self._getAllStrReturns(ctx)
    
    def visitPerItemStr(self, ctx: RulesParser.PerItemStrContext):
        return self._getAllStrReturns(ctx)
    
    def visitPerRefStr(self, ctx: RulesParser.PerRefStrContext):
        if ctx.LIT():
            ref = str(ctx.ref().REF()[-1])[1:]
            self.visit(ctx.ref())
            self.values[ref].update(str(s)[1:-1] for s in ctx.LIT())
        return self._getAllStrReturns(ctx)

    def visitPerSettingStr(self, ctx: RulesParser.PerSettingStrContext):
        return self._getAllStrReturns(ctx)
    
    def visitRefStrInList(self, ctx: RulesParser.RefStrInListContext):
        ref = str(ctx.ref().REF()[-1])[1:]
        self.visit(ctx.ref())
        self.values[ref].update(self._getAllLitReturns(ctx))

    def visitSomewhere(self, ctx):
        if ctx.PLACE():
            pl = str(ctx.PLACE())[1:-1]
            if pl.count('>') == 2:
                self.named_spots.add(pl)
        self.visitChildren(ctx)

    visitRefSomewhere = visitSomewhere

    def visitSet(self, ctx):
        ref = str(ctx.REF())[1:]
        self._checkRef(ref)
        if ref in self.data_types:
            self.errors.append(f'Cannot modify data value ^{ref} in {self.name}')
            return
        if ctx.str_():
            self.values[ref].update(self.visit(ctx.str_()))
        elif ctx.ref():
            ref2 = str(ctx.ref().REF()[-1])[1:]
            self.visit(ctx.ref())
            self._checkTypes(ref, ref2)
        elif ctx.PLACE():
            pl = str(ctx.PLACE())[1:-1]
            if pl.count('>') == 2:
                self.named_spots.add(pl)
            self.visitChildren(ctx)
        else:
            self.visitChildren(ctx)

    def visitAlter(self, ctx):
        ref = str(ctx.REF())[1:]
        self._checkRef(ref)
        if ref in self.data_types:
            self.errors.append(f'Cannot modify data value ^{ref} in {self.name}')
            return
        self.visitChildren(ctx)
        # TODO: check that the var is an int type

    def visitSwap(self, ctx):
        ref1 = str(ctx.REF(0))[1:]
        ref2 = str(ctx.REF(1))[1:]
        self._checkRef(ref1)
        self._checkRef(ref2)
        if ref1 in self.data_types:
            self.errors.append(f'Cannot modify data value ^{ref1} in {self.name}')
            return
        if ref2 in self.data_types:
            self.errors.append(f'Cannot modify data value ^{ref2} in {self.name}')
            return
        self._checkTypes(ref1, ref2)
        self.swap_pairs.add((ref1, ref2) if ref1 <= ref2 else (ref2, ref1))
