from grammar import RulesParser, RulesVisitor

from Utils import construct_id, BUILTINS

class ContextVisitor(RulesVisitor):

    def __init__(self):
        self.ctxdict = {}
        self.name = ''
        self.errors = []

    def visit(self, tree, name:str ='', ctxdict=None):
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
    visitSet = visitAlter = visitMatchRefBool = visitRefInList = visitPerSettingInt = visitPerSettingStr = visitRefEq = visitBaseNum = visitArgument = visitOneArgument = _visitAnyRef
