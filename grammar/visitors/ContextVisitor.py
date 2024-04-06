from grammar import RulesParser, RulesVisitor


class ContextVisitor(RulesVisitor):

    def __init__(self, context_types, context_values, data_types, data_values, data_defaults):
        self.ctxdict = {}
        self.context_types = context_types
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
        self.ref = ''

    def visit(self, tree, name:str ='', ctxdict=None):
        if self.name:
            # For recursive cases
            return super().visit(tree)

        self.name = name
        self.ctxdict = ctxdict or {}
        try:
            return super().visit(tree)
        finally:
            self.name = ''
            self.ctxdict = {}

    def _checkRef(self, ref):
        if ref not in self.ctxdict and not self.name.startswith('helpers'):
            self.errors.append(f'Undefined ctx property ^{ref} in non-helper {self.name}')

    def _visitAnyRef(self, ctx):
        if ctx.REF():
            self._checkRef(str(ctx.REF())[1:])
            self.visitChildren(ctx)
    visitMatchRefBool = visitRefInList = visitPerSettingInt = visitRefEq = visitBaseNum = visitArgument = visitOneArgument = _visitAnyRef

    # Anything that could return a str needs to be visited and return a collection of options
    # plus anything that compares a ref to a str should update that ref

    def visitStr(self, ctx: RulesParser.StrContext):
        if ctx.LIT():
            return {str(ctx.LIT())[1:-1]}
        return super().visitStr(ctx)

    def visitCmpStr(self, ctx: RulesParser.CmpStrContext):
        if not ctx.value().REF():
            return super().visitCmpStr(ctx)
        
        ref = str(ctx.value().REF())[1:]
        self._checkRef(ref)
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
            ref = str(ctx.REF())[1:]
            self._checkRef(ref)
            self.values[ref].update(str(s)[1:-1] for s in ctx.LIT())
        return self._getAllStrReturns(ctx)

    def visitPerSettingStr(self, ctx: RulesParser.PerSettingStrContext):
        return self._getAllStrReturns(ctx)
    
    def visitRefStrInList(self, ctx: RulesParser.RefStrInListContext):
        ref = str(ctx.REF())[1:]
        self._checkRef(ref)
        self.values[ref].update(self._getAllLitReturns(ctx))

    def visitSet(self, ctx):
        ref = str(ctx.REF(0))[1:]
        self._checkRef(ref)
        if ctx.str_():
            self.values[ref].update(self.visit(ctx.str_()))
        elif len(ctx.REF()) > 1:
            ref2 = str(ctx.REF(1))[1:]
            self._checkRef(ref2)
            # TODO: check that the types match
        else:
            self.visitChildren(ctx)

    def visitAlter(self, ctx):
        ref = str(ctx.REF())[1:]
        self._checkRef(ref)
        self.visitChildren(ctx)
        # TODO: check that the var is an int type

    def visitSwap(self, ctx):
        ref1 = str(ctx.REF(0))[1:]
        ref2 = str(ctx.REF(1))[1:]
        self._checkRef(ref1)
        self._checkRef(ref2)
        self.visitChildren()
        # TODO: check that the types match
