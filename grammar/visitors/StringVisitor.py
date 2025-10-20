from grammar import RulesParser, RulesVisitor


class StringVisitor(RulesVisitor):

    def visitBoolExpr(self, ctx):
        try:
            if ctx.OR():
                return f'OR[ {self.visit(ctx.boolExpr(0))} , {self.visit(ctx.boolExpr(1))} ]'
            elif ctx.AND():
                return f'AND[ {self.visit(ctx.boolExpr(0))} , {self.visit(ctx.boolExpr(1))} ]'
            elif ctx.TRUE():
                return 'TRUE'
            elif ctx.FALSE():
                return 'FALSE'
            elif ctx.boolExpr():
                return self.visit(ctx.boolExpr(0))
            else:
                return super().visitBoolExpr(ctx)
        except AttributeError as e:
            raise AttributeError(str(e) + '; ' + ' '.join(
                f'[{c.toStringTree(ruleNames = RulesParser.ruleNames)}]'
                for c in ctx.boolExpr()))

    def visitMeta(self, ctx):
        lit = ctx.LIT()
        func = str(ctx.FUNC())[1:]
        return f'Meta:{func}( {str(lit) + " , " if lit else ""}{self.visit(ctx.boolExpr())} )'

    def visitInvoke(self, ctx):
        items = ctx.ITEM()
        func = str(ctx.FUNC())[1:]
        s = f'Func:{func}'
        if items:
            s += f'({" , ".join(map(str, items))})'
        elif ctx.value():
            s += f'({self.visit(ctx.value())})'
        else:
            s += f'({ctx.LIT() or ctx.INT() or ctx.FLOAT() or ""})'
        if ctx.NOT():
            return f'NOT[ {s} ]'
        return s

    def _visitConditional(self, *args):
        ret = []
        while len(args) > 1:
            cond, then, *args = args
            ret.append(f'IF( {self.visit(cond)} ) THEN{{ {self.visit(then)} }}')
        if args:
            return ' ELSE '.join(ret) + f' ELSE{{ {self.visit(args[0])} }}'
        return ' ELSE '.join(ret)

    def visitIfThenElse(self, ctx):
        return self._visitConditional(*ctx.boolExpr())

    def visitPyTernary(self, ctx):
        return self._visitConditional(ctx.boolExpr(1), ctx.boolExpr(0), ctx.boolExpr(2))

    def visitCmp(self, ctx):
        return f'{self.visit(ctx.value())} {ctx.getChild(1)} {self.visit(ctx.num())}'

    def visitCmpStr(self, ctx):
        return f'{self.visit(ctx.value())} {ctx.getChild(1)} {ctx.LIT()}'

    def visitFlagMatch(self, ctx):
        num = f'{self.visit(ctx.num())}'
        return f'({self.visit(ctx.value())} & {num}) == {num}'

    def visitRefEqSimple(self, ctx):
        if ctx.ITEM():
            return f'Arg:{self.visit(ctx.ref())} {ctx.getChild(1)} Item:{ctx.ITEM()}'
        return f'Arg:{self.visit(ctx.ref())} {ctx.getChild(1)} Setting:{ctx.SETTING()}'

    def visitRefEqRef(self, ctx):
        return f'Arg:{self.visit(ctx.ref(0))} {ctx.getChild(1)} Arg:{self.visit(ctx.ref(1))}'

    def visitRefEqInvoke(self, ctx):
        return f'Arg:{self.visit(ctx.ref())} {ctx.getChild(1)} {self.visit(ctx.invoke())}'

    def visitSetting(self, ctx):
        s = f'Setting:{ctx.SETTING()}'
        if ctx.LIT():
            s += f'[{ctx.LIT()}]'
        return s

    def visitArgument(self, ctx):
        return f'Arg:{self.visit(ctx.ref())}'

    def visitRef(self, ctx):
        ref = str(ctx.REF()[-1])[1:]
        if len(ctx.REF()) == 2:
            return f'[Ref:{ref} At Ref:{str(ctx.REF(0))[1:]}]'
        if ctx.PLACE():
            return f'[Ref:{ref} At:{str(ctx.PLACE())[1:-1]}]'
        return ref

    def visitItemCount(self, ctx):
        if ctx.INT():
            return f'Items:{ctx.ITEM()}:{ctx.INT()}'
        return f'Items:{ctx.ITEM()}:{{Setting:{ctx.SETTING()}}}'

    def visitOneItem(self, ctx):
        return f'Item:{ctx.ITEM()}'

    def visitOneLitItem(self, ctx):
        return f'Item:{str(ctx.LIT())[1:-1].replace(" ", "_")}'

    def visitOneArgument(self, ctx):
        return f'OneArg:{self.visit(ctx.ref())}'

    def visitBaseNum(self, ctx):
        if ctx.INT():
            return str(ctx.INT())
        if ctx.CONST():
            return f'Const:{ctx.CONST()}'
        if ctx.ref():
            return f'Arg:{self.visit(ctx.ref())}'
        if ctx.SETTING():
            return f'Setting:{ctx.SETTING()}'
        return super().visitBaseNum(ctx)

    def visitPerItemInt(self, ctx):
        cases = list(map(str, ctx.INT())) + ["_"]
        results = [str(self.visit(n)) for n in ctx.num()]
        return f'Item:{ctx.ITEM()}{{' + '; '.join(f'{i} => {r}' for i,r in zip(cases, results)) + '}'

    def visitRefInList(self, ctx):
        return f'(Arg:{self.visit(ctx.ref())} IN [{"|".join(map(str, ctx.ITEM()))}])'

    def visitItemList(self, ctx):
        els = list(map(str, ctx.FUNC())) + [self.visit(item) for item in ctx.item()]
        return f'[{", ".join(els)}]'
