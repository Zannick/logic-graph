from grammar import RulesParser, RulesVisitor

from Utils import construct_id, BUILTINS

class ContextVisitor(RulesVisitor):

    def __init__(self, context_types, context_values):
        self.ctxdict = {}
        self.context_types = context_types
        self.name = ''
        self.errors = []
        self.values = {
            ctx: [context_values[ctx]]
            for ctx, t in self.context_types.items()
            if t.startswith('enums::')
        }

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
    visitAlter = visitMatchRefBool = visitRefInList = visitPerSettingInt = visitPerSettingStr = visitRefEq = visitBaseNum = visitArgument = visitOneArgument = _visitAnyRef

    def visitSet(self, ctx):
        ref = str(ctx.REF(0))[1:]
        self._checkRef(ref)
        if ctx.str_():
            self.values[ref].append(self.visit(ctx.str_()))
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
