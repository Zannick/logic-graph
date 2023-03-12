from collections import defaultdict

from grammar import RulesParser, RulesVisitor
from Utils import construct_id, BUILTINS

import inflection

_placeType = ['Region', 'Area', 'Spot']

class RustVisitor(RulesVisitor):

    def __init__(self, ctxdict, name):
        self.ctxdict = ctxdict
        self.name = name
        self.rettype = None

    def _getRefGetter(self, ref):
        if ref in self.ctxdict:
            return f'ctx.{self.ctxdict[ref]}()'
        return BUILTINS.get(ref, '$' + ref)
    
    def _getRefSetter(self, ref):
        return f'ctx.{self.ctxdict[ref]}'

    def _getRefEnum(self, ref):
        return f'enums::{ref.capitalize()}'

    def _getFuncAndArgs(self, func):
        if func in BUILTINS:
            return BUILTINS[func] + '('
        else:
            return f'helper__{construct_id(func[1:])}!(ctx, '

    def _getPlaceType(self, place):
        return _placeType[place.count(">")]

    def visit(self, tree, rettype=None):
        last_rettype = self.rettype
        self.rettype = rettype
        try:
            return super().visit(tree)
        finally:
            self.rettype = last_rettype

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
            elif ctx.NOT():
                return '!' + super().visitBoolExpr(ctx)
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
            args = f'{self._getPlaceType(pl)}Id::{construct_id(pl)}'
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
    
    # This could be easier if str enum values are required to be unique among all enums
    # otherwise we have to get the appropriate ref/setting enum
    def visitCmpStr(self, ctx):
        getter = self.visit(ctx.value())
        rtype = inflection.camelize(getter[4:-2])
        return f'{getter} {ctx.getChild(1)} enums::{rtype}::{inflection.camelize(str(ctx.LIT())[1:-1])}'

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
        return f'ctx.{ctx.SETTING()}'

    def visitArgument(self, ctx):
        return self._getRefGetter(str(ctx.REF())[1:])

    def visitItemCount(self, ctx):
        if ctx.INT():
            val = str(ctx.INT())
        else:
            val = f'ctx.{ctx.SETTING()}'
        return f'ctx.count(Item::{ctx.ITEM()}) >= {val}'

    def visitOneItem(self, ctx):
        return ('!' if ctx.NOT() else '') + f'ctx.has(Item::{ctx.ITEM()})'

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
        cases = list(map(str, ctx.INT())) + ['_']
        results = [str(self.visit(n)) for n in ctx.num()]
        return (f'match ctx.count(Item::{ctx.ITEM()}) {{ '
                + ', '.join(f'{i} => {r}' for i, r in zip(cases, results))
                + '}')

    def visitRefInList(self, ctx):
        return (f'match {self._getRefGetter(str(ctx.REF())[1:])} {{ '
                + '|'.join(f'Item::{i}' for i in ctx.ITEM())
                + ' => true, _ => false, }')
    
    # TODO: other REF/SETTING rules

    def visitStr(self, ctx):
        if ctx.LIT() and self.rettype:
            return f'{self.rettype}::{inflection.camelize(str(ctx.LIT())[1:-1])}'
        return super().visitStr(ctx)

    def visitPerRefStr(self, ctx):
        ref = str(ctx.REF())[1:]
        enum = self._getRefEnum(ref)
        cases = [f'{enum}::{str(c)[1:-1].capitalize()}' for c in ctx.LIT()] + [str(c) for c in ctx.INT()] + ['_']
        results = [str(self.visit(s, self.rettype)) for s in ctx.str_()]
        return (f'match {self._getRefGetter(ref)} {{ '
                + ', '.join(f'{c} => {r}' for c, r in zip(cases, results))
                + '}')

    def visitSomewhere(self, ctx):
        places = defaultdict(list)
        for pl in ctx.PLACE():
            pl = str(pl)[1:-1]
            places[self._getPlaceType(pl)].append(pl)
        per_type = [f'(match get_{pt.lower()}(ctx.position()) {{'
                    + ' | '.join(f'{pt}Id::{construct_id(pl)}' for pl in plist)
                    + ' => true, _ => false })'
                    for pt, plist in places.items()
                    ]
        return ('!' if ctx.NOT() else '') + ' || '.join(per_type)

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
            val = f'{self._getPlaceType(pl)}Id::{construct_id(pl)}'
        elif ctx.num():
            val = self.visit(ctx.num())
        else:
            val = self.visit(ctx.str_(), self._getRefEnum(var))
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
        

class ActionHasEffectVisitor(RustVisitor):

    def __init__(self, ctxdict, name):
        super().__init__(ctxdict, name)

    def visitActions(self, ctx):
        return ' || '.join(self.visit(ch) for ch in ctx.action())

    def visitSet(self, ctx):
        return super().visitSet(ctx)[:-1].replace(' = ', ' != ')

    def visitAlter(self, ctx):
        op = str(ctx.BINOP())
        if op in ('+', '-'):
            return f'0 != {self.visit(ctx.num())}'
        return f'1 != {self.visit(ctx.num())}'

    def visitActionHelper(self, ctx: RulesParser.ActionHelperContext):
        assert str(ctx.invoke().FUNC()) not in BUILTINS
        return self.visit(ctx.invoke()).replace('helper', 'helper_has_effect')
