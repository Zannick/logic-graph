from functools import reduce

from grammar import RulesParser, RulesVisitor

from Utils import construct_id, BUILTINS

class ContextVisitor(RulesVisitor):

    def __init__(self, context_types, context_values):
        self.ctxdict = {}
        self.context_types = context_types
        self.name = ''
        self.errors = []
        self.values = {
            ctx: {context_values[ctx]}
            for ctx, t in self.context_types.items()
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
            ret = super().visit(tree)
        finally:
            self.name = ''
            self.ctxdict = {}
        return ret

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

    def visitCmpStr(self, ctx: RulesParser.CmpStrContext):
        if not ctx.value().REF():
            return super().visitCmpStr(ctx)
        
        ref = str(ctx.value().REF())[1:]
        self._checkRef(ref)
        self.values[ref].add(str(ctx.LIT()))

    # value is SETTING or REF -- TODO: multiple REFs or SETTING+REF with the same enum type
    
    def _getAllStrReturns(self, ctx):
        s = set()
        for el in ctx.str_():
            s.update(self.visit(el))
        return s

    def visitCondStr(self, ctx: RulesParser.CondStrContext):
        for el in ctx.boolExpr():
            self.visit(el)
        return self._getAllStrReturns(ctx)
    
    def visitPerItemStr(self, ctx: RulesParser.PerItemStrContext):
        return self._getAllStrReturns(ctx)
    
    def visitPerRefStr(self, ctx: RulesParser.PerRefStrContext):
        if ctx.LIT():
            ref = str(ctx.REF(0))[1:]
            self._checkRef(ref)
            self.values[ref].update(map(str, ctx.LIT()))
        return self._getAllStrReturns(ctx)

    def visitPerSettingStr(self, ctx: RulesParser.PerSettingStrContext):
        return self._getAllStrReturns(ctx)

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
        ref = str(ctx.REF(0))[1:]
        self._checkRef(ref)
        self.visitChildren(ctx)
        # TODO: check that the var is an int type
