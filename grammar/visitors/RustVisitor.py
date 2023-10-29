from collections import defaultdict
from itertools import chain
import logging

from grammar import RulesParser, RulesVisitor
from Utils import construct_id, construct_place_id, construct_spot_id, getPlaceType, place_to_names, BUILTINS

import inflection


class RustVisitor(RulesVisitor):

    def __init__(self, context_types, action_funcs, ctxdict, data_keys, name):
        self.context_types = context_types
        self.action_funcs = action_funcs
        self.ctxdict = ctxdict
        self.data_keys = data_keys
        self.name = name
        self.rettype = None

    def _getRefGetter(self, ref):
        if ref in self.data_keys:
            return f'data::{ref}(ctx.position())'
        if ref in self.ctxdict:
            return f'ctx.{self.ctxdict[ref]}()'
        if func := self.action_funcs.get(self.name):
            if ref in func.get('args', {}):
                return ref
        return BUILTINS.get(ref, '$' + ref)
    
    def _getRefRaw(self, ref):
        return f'ctx.{self.ctxdict[ref]}'
    
    def _getRefSetter(self, ref):
        return f'ctx.set_{self.ctxdict[ref]}'

    def _getRefEnum(self, ref):
        return f'enums::{ref.capitalize()}'

    def _getFuncAndArgs(self, func):
        if func in BUILTINS:
            return BUILTINS[func] + '('
        else:
            return f'helper__{construct_id(func[1:])}!(ctx, '

    def visit(self, tree, rettype=None):
        last_rettype = self.rettype
        self.rettype = rettype
        try:
            return super().visit(tree)
        except:
            logging.error(f'Encountered exception rendering {self.name}: {self.ctxdict}')
            raise
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
            places = [str(p)[1:-1] for p in ctx.PLACE()]
            args = ', '.join(construct_place_id(pl) for pl in places)
        elif ctx.REF():
            args = self._getRefGetter(str(ctx.REF())[1:])
        else:
            args = f'{ctx.LIT() or ctx.INT() or ctx.FLOAT() or ""}'
            if not args:
                func = func[:-2]
        return f'{"!" if ctx.NOT() else ""}{func}{args})'

    def _visitConditional(self, *args, else_case=True):
        ret = []
        while len(args) > 1:
            cond, then, *args = args
            ret.append(f'if {self.visit(cond)} {{ {self.visit(then)} }}')
        if args:
            ret.append(f'{{ {self.visit(args[0])} }}')
        elif else_case:
            ret.append('{ false }')
        return ' else '.join(ret)

    def visitIfThenElse(self, ctx):
        return self._visitConditional(*ctx.boolExpr())

    def visitPyTernary(self, ctx):
        return self._visitConditional(ctx.boolExpr(1), ctx.boolExpr(0), ctx.boolExpr(2))

    def visitCmp(self, ctx):
        return f'Into::<i32>::into({self.visit(ctx.value())}) {ctx.getChild(1)} {self.visit(ctx.num())}.into()'
    
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
        return f'{ref} == ctx.{ctx.SETTING()}()'

    def visitSetting(self, ctx):
        # TODO: dict settings?
        return f'ctx.{ctx.SETTING()}()'

    def visitArgument(self, ctx):
        return self._getRefGetter(str(ctx.REF())[1:])

    def visitItemCount(self, ctx):
        if ctx.INT():
            val = str(ctx.INT())
        else:
            val = f'ctx.{ctx.SETTING()}()'
        return f'ctx.count(Item::{ctx.ITEM()}) >= {val}'

    def visitOneItem(self, ctx):
        return ('!' if ctx.NOT() else '') + f'ctx.has(Item::{ctx.ITEM()})'

    def visitOneArgument(self, ctx):
        ref = self._getRefGetter(str(ctx.REF())[1:])
        if ref.startswith('ctx.') or ref.startswith('data::'):
            return ref
        return f'ctx.has({ref})'
    
    # There's no need to optimize for bitflags here, as the compiler can handle that!
    def visitItemList(self, ctx):
        helpers = [self._getFuncAndArgs(helper)[:-2] + ')' for helper in map(str, ctx.FUNC())]
        items = [self.visit(item) for item in ctx.item()]
        return f'{" && ".join(helpers + items)}'

    def visitBaseNum(self, ctx):
        if ctx.INT():
            return str(ctx.INT())
        if ctx.REF():
            return self._getRefGetter(str(ctx.REF())[1:])
        if ctx.SETTING():
            return f'ctx.{ctx::SETTING()}()'
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
            places[getPlaceType(pl)].append(pl)
        per_type = [('(match ctx.position()' if pt == 'SpotId' else f'(match get_{pt.lower()[:-2]}(ctx.position())')
                    + ' {'
                    + ' | '.join(construct_place_id(pl) for pl in plist)
                    + ' => true, _ => false })'
                    for pt, plist in places.items()
                    ]
        return ('!' if ctx.NOT() else '') + ' || '.join(per_type)

    ## Action-specific
    def visitActions(self, ctx):
        return ' '.join(map(str, (self.visit(ch) for ch in ctx.action())))

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
            val = construct_place_id(pl)
        elif ctx.num():
            val = self.visit(ctx.num())
        else:
            val = self.visit(ctx.str_(), self._getRefEnum(var))
        return f'{self._getRefSetter(var)}({val});'

    def visitAlter(self, ctx):
        return f'{self._getRefRaw(str(ctx.REF())[1:])} {ctx.BINOP()}= {self.visit(ctx.num())};'

    def visitFuncNum(self, ctx):
        func = self._getFuncAndArgs(str(ctx.FUNC()))
        if ctx.ITEM():
            return f'{func}Item::{ctx.ITEM()})'
        elif ctx.num():
            return f'{func}{", ".join(self.visit(n) for n in ctx.num())})'
        else:
            return func[:-2] + ')'
        
    def visitActionHelper(self, ctx):
        return self.visit(ctx.invoke()) + ';'
        
    def visitCondAction(self, ctx):
        return self._visitConditional(*chain(*zip(ctx.boolExpr(), ctx.actions())), else_case=False)

    def visitRefInPlaceRef(self, ctx):
        ptype = self.context_types[str(ctx.REF(1))[1:]]
        eq = '!' if ctx.NOT() else '='
        get = f'{self._getRefGetter(str(ctx.REF(0))[1:])}'
        if ptype != 'SpotId':
            get = f'get_{ptype[:-2].lower()}({get})'
        return f'{get} {eq}= {self._getRefGetter(str(ctx.REF(1))[1:])}'
    
    def visitRefInPlaceName(self, ctx):
        pl = str(ctx.PLACE())[1:-1]
        ptype = getPlaceType(pl)
        eq = '!' if ctx.NOT() else '='
        get = f'{self._getRefGetter(str(ctx.REF())[1:])}'
        if ptype == 'SpotId':
            val = construct_spot_id(*place_to_names(pl))
        else:
            get = f'get_{ptype[:-2].lower()}({get})'
            val = f'{ptype}::{construct_id(pl)}'
        return f'{get} {eq}= {val}'

    def visitRefInFunc(self, ctx):
        func = str(ctx.invoke().FUNC())[1:]
        eq = '!' if ctx.NOT() else '='
        if func == 'default':
            return f'{self._getRefGetter(str(ctx.REF())[1:])} {eq}= {self.visit(ctx.invoke())}'
        assert func in ('get_area', 'get_region')
        return (f'{func}({self._getRefGetter(str(ctx.REF())[1:])}) '
                f'{eq}= {self.visit(ctx.invoke())}')
