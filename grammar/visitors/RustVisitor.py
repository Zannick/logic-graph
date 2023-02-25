from grammar import RulesParser, RulesVisitor

from Utils import construct_id, BUILTINS

_placePrefix = ['RegionId', 'AreaId', 'SpotId']

class RustVisitor(RulesVisitor):

    def __init__(self, ctxdict, name):
        self.ctxdict = ctxdict
        self.name = name

    def _getRefGetter(self, ref):
        if ref in self.ctxdict:
            return f'ctx.{self.ctxdict[ref]}()'
        return BUILTINS.get(ref, '$' + ref)
    
    def _getRefSetter(self, ref):
        return f'ctx.{self.ctxdict[ref]}'

    def _getFuncAndArgs(self, func):
        if func in BUILTINS:
            return BUILTINS[func] + '('
        else:
            return f'helper__{construct_id(func[1:])}!(ctx, '

    def visitBoolExpr(self, ctx):
        try:
            if ctx.OR():
                return f'({self.visit(ctx.boolExpr(0))} || {self.visit(ctx.boolExpr(1))})'
            elif ctx.AND():
                return f'({self.visit(ctx.boolExpr(0))} && {self.visit(ctx.boolExpr(1))})'
            elif ctx.TRUE():
                return f'true'
            elif ctx.FALSE():
                return f'false'
            elif ctx.boolExpr():
                return f'({self.visit(ctx.boolExpr(0))})'
            else:
                return super().visitBoolExpr(ctx)
        except AttributeError as e:
            raise AttributeError(str(e) + '; ' + ' '.join(
                f'[{c.toStringTree(ruleNames = RulesParser.ruleNames)}]'
                for c in ctx.boolExpr()))

    def visitInvoke(self, ctx):
        items = ctx.ITEM()
        func = self._getFuncAndArgs(str(ctx.FUNC()))
        if items:
            args = f'{", ".join("Item::" + str(item) for item in items)}'
        elif ctx.value():
            args = f'{self.visit(ctx.value())}'
        elif ctx.PLACE():
            pl = str(ctx.PLACE())[1:-1]
            args = f'{_placePrefix[pl.count(">")]}::{construct_id(pl)}'
        else:
            args = f'{ctx.LIT() or ctx.INT() or ctx.FLOAT() or ""}'
            if not args:
                func = func[:-2]
        return f'{"!" if ctx.NOT() else ""}{func}{args})'

    def _visitConditional(self, *args):
        ret = []
        while len(args) > 1:
            cond, then, *args = args
            ret.append(f'if {self.visit(cond)} {{ {self.visit(then)} }}')
        if args:
            ret.append(f'{{ {self.visit(args[0])} }}')
        else:
            ret.append('{ false }')
        return ' else '.join(ret)

    def visitIfThenElse(self, ctx):
        return self._visitConditional(*ctx.boolExpr())

    def visitPyTernary(self, ctx):
        return self._visitConditional(ctx.boolExpr(1), ctx.boolExpr(0), ctx.boolExpr(2))

    def visitCmp(self, ctx):
        return f'{self.visit(ctx.value())} {ctx.getChild(1)} {self.visit(ctx.num())}'

    def visitFlagMatch(self, ctx):
        num = f'{self.visit(ctx.num())}'
        return f'({self.visit(ctx.value())} & {num}) == {num}'

    def visitRefEq(self, ctx):
        ref = self._getRefGetter(str(ctx.REF())[1:])
        if ctx.ITEM():
            return f'{ref} == Item::{ctx.ITEM()}'
        return f'{ref} == ctx.{ctx.SETTING()}'

    def visitSetting(self, ctx):
        # TODO: dict settings?
        return f'{"!" if ctx.NOT() else ""}ctx.{ctx.SETTING()}'

    def visitArgument(self, ctx):
        ref = self._getRefGetter(str(ctx.REF())[1:])
        return f'{"!" if ctx.NOT() else ""}{ref}'

    def visitItemCount(self, ctx):
        if ctx.INT():
            val = str(ctx.INT())
        else:
            val = f'ctx.{ctx.SETTING()}'
        return f'ctx.count(Item::{ctx.ITEM()}) >= {val}'

    def visitOneItem(self, ctx):
        return f'ctx.has(Item::{ctx.ITEM()})'

    def visitOneArgument(self, ctx):
        ref = self._getRefGetter(str(ctx.REF())[1:])
        if ref.startswith('ctx'):
            return ref
        return f'ctx.has({ref})'

    def visitBaseNum(self, ctx):
        if ctx.INT():
            return str(ctx.INT())
        if ctx.REF():
            return self._getRefGetter(str(ctx.REF())[1:])
        if ctx.SETTING():
            return f'ctx.{ctx::SETTING()}'
        # TODO: constants
        return self.visitChildren(ctx)

    def visitMathNum(self, ctx):
        return f'{self.visit(ctx.baseNum())} {ctx.BINOP()} {self.visit(ctx.num())}'

    def visitPerItemInt(self, ctx):
        cases = list(map(str, ctx.INT())) + ["_"]
        results = [str(self.visit(n)) for n in ctx.num()]
        return (f'match ctx.count(Item::{ctx.ITEM()}) {{ '
                + ', '.join(f'{i} => {r}' for i, r in zip(cases, results))
                + '}')

    def visitRefInList(self, ctx):
        return (f'match {self._getRefGetter(str(ctx.REF())[1:])} {{ '
                + '|'.join(f'Item::{i}' for i in ctx.ITEM())
                + ' => true, _ => false, }')

    ## Action-specific
    def visitActions(self, ctx):
        return ' '.join(self.visit(ch) for ch in ctx.action())

    def visitSet(self, ctx):
        var = str(ctx.REF(0))[1:]
        if ctx.TRUE():
            val = 'true'
        elif ctx.FALSE():
            val = 'false'
        elif len(ctx.REF()) > 1:
            val = self._getRefGetter(str(ctx.REF(1))[1:])
        elif ctx.PLACE():
            pl = str(ctx.PLACE())[1:-1]
            val = f'{_placePrefix[pl.count(">")]}::{construct_id(pl)}'
        else:
            val = self.visit(ctx.str_() or ctx.num())
        return f'{self._getRefSetter(var)} = {val};'

    def visitAlter(self, ctx):
        return f'{self._getRefSetter(str(ctx.REF())[1:])} {ctx.BINOP()}= {self.visit(ctx.num())};'

    def visitFuncNum(self, ctx):
        func = self._getFuncAndArgs(str(ctx.FUNC()))
        if ctx.ITEM():
            return f'{func}Item::{ctx.Item()})'
        elif ctx.num():
            return f'{func}{", ".join(self.visit(n) for n in ctx.num())})'
        else:
            return func[:-2] + ')'
