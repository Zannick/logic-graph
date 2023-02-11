from grammar import RulesParser, RulesVisitor

from Utils import construct_id, BUILTINS

class RustVisitor(RulesVisitor):

    def __init__(self, ctxdict):
        self.ctxdict = ctxdict

    def _getRealRef(self, ref):
        return f'ctx.{self.ctxdict[ref]}' if ref in self.ctxdict else f'${ref}'

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
        f = str(ctx.FUNC())
        if f in BUILTINS:
            func = BUILTINS[f]
            c = ''
        else:
            func = 'helper__' + construct_id(f[1:]) + '!'
            c = 'ctx'
        if items:
            args = f'{", ".join("Item::" + str(item) for item in items)}'
        elif ctx.value():
            args = f'{self.visit(ctx.value())}'
        else:
            args = f'{ctx.LIT() or ctx.INT() or ctx.FLOAT() or ""}'
        return f'{"!" if ctx.NOT() else ""}{func}({c}{", " if c and args else ""}{args})'

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
        ref = self._getRealRef(str(ctx.REF())[1:])
        if ctx.ITEM():
            return f'{ref} == Item::{ctx.ITEM()}'
        return f'{ref} == ctx.{ctx.SETTING()}'

    def visitSetting(self, ctx):
        # TODO: dict settings?
        return f'{"!" if ctx.NOT() else ""}ctx.{ctx.SETTING()}'

    def visitArgument(self, ctx):
        ref = self._getRealRef(str(ctx.REF())[1:])
        return f'{"!" if ctx.NOT() else ""}{ref}'

    def visitItemCount(self, ctx):
        if ctx.INT():
            val = str(ctx.INT())
        else:
            val = f'ctx.{ctx.SETTING()}'
        return f'ctx.count(&Item::{ctx.ITEM()}) >= {val}'

    def visitOneItem(self, ctx):
        return f'ctx.has(&Item::{ctx.ITEM()})'

    def visitOneArgument(self, ctx):
        ref = self._getRealRef(str(ctx.REF())[1:])
        if ref.startswith('ctx'):
            return ref
        return f'ctx.has(&{ref})'

    def visitBaseNum(self, ctx):
        if ctx.INT():
            return str(ctx.INT())
        if ctx.REF():
            return self._getRealRef(str(ctx.REF())[1:])
        if ctx.SETTING():
            return f'ctx.{ctx::SETTING()}'
        # TODO: constants
        return self.visitChildren(ctx)

    def visitPerItemInt(self, ctx):
        cases = list(map(str, ctx.INT())) + ["_"]
        results = [str(self.visit(n)) for n in ctx.num()]
        return (f'match ctx.count(&Item::{ctx.ITEM()}) {{ '
                + ', '.join(f'{i} => {r}' for i, r in zip(cases, results))
                + '}')

    def visitRefInList(self, ctx):
        return (f'match {self._getRealRef(str(ctx.REF())[1:])} {{ '
                + '|'.join(f'Item::{i}' for i in ctx.ITEM())
                + ' => true, _ => false, }')
