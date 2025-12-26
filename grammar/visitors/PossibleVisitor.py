from enum import Enum
import logging
import operator

from Utils import BUILTINS, construct_id, int_types, get_area, get_region, getPlaceType

from grammar import RulesParser, RulesVisitor

optable = {
    '==': operator.eq,
    '!=': operator.ne,
    '>=': operator.ge,
    '>': operator.gt,
    '<=': operator.le,
    '<': operator.lt,
    '-': operator.sub,
    '+': operator.add,
    '*': operator.mul,
    '/': operator.floordiv,
}

class Result(Enum):
    FALSE = 0
    TRUE = 1
    UNCERTAIN = 2

    # overload bitwise ops for easier management
    def __or__(self, other):
        if self == Result.TRUE or other == Result.TRUE:
            return Result.TRUE
        if self == Result.FALSE:
            return other
        # self == UNCERTAIN and other != TRUE
        return Result.UNCERTAIN
    
    def __and__(self, other):
        if self == Result.FALSE or other == Result.FALSE:
            return Result.FALSE
        if self == Result.TRUE:
            return other
        # self == UNCERTAIN and other != FALSE
        return Result.UNCERTAIN
    
    def __invert__(self):
        if self == Result.TRUE:
            return Result.FALSE
        if self == Result.FALSE:
            return Result.TRUE
        return Result.UNCERTAIN


class PossibleVisitor(RulesVisitor):
    
    def __init__(self, helpers, rules, context_types, data_types, data_defaults, data_values):
        self.helpers = helpers
        self.rules = rules
        self.context_types = context_types
        self.data_types = data_types
        self.data_defaults = data_defaults
        self.data_values = data_values
        self.name = ''
        self.spot_id = None
        self.local_args = {}
        self.errors = []
        self.rettype = None

    def examine(self, ctx, spot_id, name=''):
        """Returns True if the ctx tree might evaluate to True at this spot_id."""        
        self.name = name
        self.spot_id = spot_id
        try:
            result = self.visit(ctx, bool)
        finally:
            self.name = ''
            self.spot_id = None
        return result != Result.FALSE

    def visit(self, tree, rettype=None, local_args=None):
        last_rettype = self.rettype
        self.rettype = rettype
        last_args = self.local_args
        self.local_args = local_args or {}
        try:
            ret = super().visit(tree)
            assert ret != None, f'Missing visitor implementation: {tree.toStringTree(ruleNames = RulesParser.ruleNames)}'
            return ret
        except:
            logging.error(f'Encountered exception examining {self.name} at {self.spot_id}')
            raise
        finally:
            self.rettype = last_rettype
            self.local_args = last_args

    def visitRef(self, ctx):
        ref = str(ctx.REF()[-1])[1:]
        ref_spot = self.spot_id
        if ref in self.local_args:
            return self.local_args[ref]
        if ref not in self.data_values:
            return Result.UNCERTAIN
        if len(ctx.REF()) == 2:
            ref0 = str(ctx.REF(0))[1:]
            if ref0 not in self.data_values:
                return Result.UNCERTAIN
            ref_spot = construct_id(self.data_values[ref0].get(self.spot_id) or self.data_defaults[ref0])
        elif ctx.PLACE():
            ref_spot = construct_id(str(ctx.PLACE())[1:-1])
        val = self.data_values[ref].get(ref_spot) or self.data_defaults[ref]
        if val == True:
            return Result.TRUE
        if val == False:
            return Result.FALSE
        return val
    
    def visitBoolExpr(self, ctx):
        try:
            if ctx.OR():
                return self.visit(ctx.boolExpr(0)) | self.visit(ctx.boolExpr(1))
            elif ctx.AND():
                return self.visit(ctx.boolExpr(0)) & self.visit(ctx.boolExpr(1))
            elif ctx.TRUE():
                return Result.TRUE
            elif ctx.FALSE():
                return Result.FALSE
            elif ctx.boolExpr():
                return self.visit(ctx.boolExpr(0))
            elif ctx.NOT():
                return ~super().visitBoolExpr(ctx)
            else:
                return super().visitBoolExpr(ctx)
        except AttributeError as e:
            raise AttributeError(str(e) + '; ' + ' '.join(
                f'[{c.toStringTree(ruleNames = RulesParser.ruleNames)}]'
                for c in ctx.boolExpr()))

    def visitInvoke(self, ctx):
        func = str(ctx.FUNC())
        if func in BUILTINS:
            # there are only a few options: get_area, get_region, default
            # if func == 'get_area'
            if func == '$default':
                if not self.rettype:
                    logging.warning(f'No rettype for $default invocation: {self.name}')
                    return Result.UNCERTAIN
                if self.rettype == bool:
                    return Result.FALSE
                if self.rettype == 'SpotId':
                    return 'SpotId::None'
                if self.rettype in int_types:
                    return 0
                val = self.data_defaults.get(self.rettype)
                if not val:
                    logging.warning(f'No default found for rettype={self.rettype}')
                    return Result.UNCERTAIN
                return val
            if func not in ('get_area', 'get_region'):
                return Result.UNCERTAIN
            func = get_area if func == 'get_area' else get_region
            if ctx.ref():
                pl = self.visit(ctx.ref())
                if isinstance(pl, str):
                    ret = func(pl)
                    return ~ret if ctx.NOT() else ret
            if ctx.PLACE():
                ret = func(str(ctx.PLACE())[1:-1])
                return ~ret if ctx.NOT() else ret
            return Result.UNCERTAIN
        elif func in self.rules:
            # Unless we check every variant
            return Result.UNCERTAIN
        elif func in self.helpers:
            helper = self.helpers[func]
            args = []
            if ctx.ITEM():
                args.extend(map(str, ctx.ITEM()))
            elif ctx.value():
                val = self.visit(ctx.value())
                if val == Result.UNCERTAIN:
                    return Result.UNCERTAIN
                args.append(val)
            elif ctx.PLACE():
                args.append(str(ctx.PLACE())[1:-1])
            elif ctx.ref():
                ref = self.visit(ctx.ref())
                if ref == Result.UNCERTAIN:
                    return Result.UNCERTAIN
                args.append(ref)
            else:
                arg = f'{ctx.LIT() or ctx.INT() or ctx.FLOAT() or ""}'
                if arg:
                    args.append(arg)
            args = {
                arg['name']: val
                for (arg, val) in zip(helper['args'], args)
            }
            ret = self.visit(helper['pr'].tree, local_args=args)
            return ~ret if ctx.NOT() else ret
        return Result.UNCERTAIN

    def _visitConditional(self, *args):
        ret = None
        while len(args) > 1:
            cond, then, *args = args
            cond = self.visit(cond)
            if cond == Result.FALSE:
                continue
            r2 = self.visit(then)
            if ret is None:
                ret = r2
            elif ret != r2:
                return Result.UNCERTAIN
            if cond == Result.TRUE:
                return ret
        if args:
            r2 = self.visit(args[0])
            if ret is None:
                return r2
            if ret != r2:
                return Result.UNCERTAIN
        return ret
    
    def visitIfThenElse(self, ctx):
        return self._visitConditional(*ctx.boolExpr())

    def visitPyTernary(self, ctx):
        return self._visitConditional(ctx.boolExpr(1), ctx.boolExpr(0), ctx.boolExpr(2))

    def visitCmp(self, ctx):
        left = self.visit(ctx.value(), 'int')
        if left == Result.UNCERTAIN:
            return Result.UNCERTAIN
        right = self.visit(ctx.num(), 'int')
        if right == Result.UNCERTAIN:
            return Result.UNCERTAIN
        op = optable.get(str(ctx.getChild(1)))
        if not op:
            return Result.UNCERTAIN
        left = int(left)
        right = int(right)
        return Result(op(left, right))

    def visitCmpStr(self, ctx):
        val = self.visit(ctx.value())
        lit = str(ctx.LIT())[1:-1]
        if val == Result.UNCERTAIN:
            return Result.UNCERTAIN
        if str(ctx.getChild(1)) == '==':
            return Result(val == lit)
        else:
            return Result(val != lit)

    def visitFlagMatch(self, ctx):
        num = self.visit(ctx.num())
        val = self.visit(ctx.value())
        if num == Result.UNCERTAIN or val == Result.UNCERTAIN:
            return Result.UNCERTAIN
        return Result((val & num) == num)

    def visitRefEqSimple(self, ctx):
        ref = self.visit(ctx.ref())
        if ref == Result.UNCERTAIN:
            return Result.UNCERTAIN
        if ctx.ITEM():
            item = str(ctx.ITEM())
            if str(ctx.getChild(1)) == '==':
                return Result(ref == item)
            else:
                return Result(ref != item)
        # setting
        return Result.UNCERTAIN

    def _refEq(self, val1, val2, op, ints=False):
        if val1 == Result.UNCERTAIN or val2 == Result.UNCERTAIN:
            return Result.UNCERTAIN
        if op == '==':
            return Result(val1 == val2)
        else:
            return Result(val1 != val2)

    def visitRefEqRef(self, ctx):
        val1 = self.visit(ctx.ref(0))
        val2 = self.visit(ctx.ref(1))
        return self._refEq(val1, val2, str(ctx.getChild(1)))

    def visitRefEqInvoke(self, ctx):
        val1 = self.visit(ctx.ref())
        refname = str(ctx.ref().REF()[-1])[1:]
        val2 = self.visit(ctx.invoke(), rettype=self.context_types.get(refname, self.data_types.get(refname)))
        return self._refEq(val1, val2, str(ctx.getChild(1)))


    def _alwaysUncertain(self, _):
        return Result.UNCERTAIN
    visitSetting = visitItemCount = visitOneItem = visitItemList = visitPerItemInt = _alwaysUncertain

    def visitArgument(self, ctx):
        return self.visit(ctx.ref())

    def visitOneArgument(self, ctx):
        ref = self.visit(ctx.ref())
        refname = str(ctx.ref().REF()[-1])[1:]
        # TODO: Can there even be data or context typed as Item?
        if refname in self.data_types and self.data_types[refname] != 'Item':
            return ref
        return Result.UNCERTAIN

    def visitBaseNum(self, ctx):
        if ctx.INT():
            return str(ctx.INT())
        if ctx.ref():
            return self.visit(ctx.ref())
        if ctx.SETTING():
            return Result.UNCERTAIN
        # TODO: constants
        return self.visitChildren(ctx)

    def visitMathNum(self, ctx):
        left = self.visit(ctx.baseNum())
        right = self.visit(ctx.num())
        if left == Result.UNCERTAIN or right == Result.UNCERTAIN:
            return Result.UNCERTAIN
        left = int(left)
        right = int(right)
        op = optable.get(str(ctx.BINOP()))
        if not op:
            return Result.UNCERTAIN
        return op(left, right)

    def visitRefInList(self, ctx):
        ref = self.visit(ctx.ref())
        if ref == Result.UNCERTAIN:
            return Result.UNCERTAIN
        return Result(ref in map(str, ctx.ITEM()))
    
    def visitRefStrInList(self, ctx):
        ref = self.visit(ctx.ref())
        if ref == Result.UNCERTAIN:
            return Result.UNCERTAIN
        return Result(ref in (str(lit)[1:-1] for lit in ctx.LIT()))

    def visitStr(self, ctx):
        if ctx.LIT():
            return str(ctx.LIT())[1:-1]
        return super().visitStr(ctx)

    def visitPerRefStr(self, ctx):
        ref = self.visit(ctx.ref())
        if ref == Result.UNCERTAIN:
            return Result.UNCERTAIN
        cases = [str(c)[1:-1] for c in ctx.LIT()] + [str(c) for c in ctx.INT()]
        for case, rstr in zip(cases, ctx.str_()):
            if ref == case:
                return self.visit(rstr)
        return self.visit(ctx.str_()[-1])

    def visitSomewhere(self, ctx):
        ret = Result.TRUE
        if ctx.NOT():
            ret = Result.FALSE
        for pl in ctx.PLACE():
            plid = construct_id(str(pl)[1:-1])
            if self.spot_id.startswith(plid):
                return ret
        return ~ret

    def visitRefInPlaceRef(self, ctx):
        ref = self.visit(ctx.ref(0))
        place = self.visit(ctx.ref(1))
        if ref == Result.UNCERTAIN or place == Result.UNCERTAIN:
            return Result.UNCERTAIN
        if ctx.NOT():
            return Result(not ref.startswith(place))
        return Result(ref.startswith(place))
    
    def visitRefInPlaceName(self, ctx):
        ref = self.visit(ctx.ref())
        place = str(ctx.PLACE())[1:-1]
        if ref == Result.UNCERTAIN:
            return Result.UNCERTAIN
        if ctx.NOT():
            return Result(not ref.startswith(place))
        return Result(ref.startswith(place))

    def visitRefInFunc(self, ctx):
        ref = self.visit(ctx.ref())
        if ref == Result.UNCERTAIN:
            return Result.UNCERTAIN
        refname = str(ctx.ref().REF()[-1])[1:]
        place = self.visit(ctx.invoke(), rettype=self.data_types.get(refname))
        if place == Result.UNCERTAIN:
            return Result.UNCERTAIN
        if ctx.NOT():
            return Result(not ref.startswith(place))
        return Result(ref.startswith(place))

    def visitFuncNum(self, ctx):
        args = []
        if ctx.ITEM():
            return Result.UNCERTAIN
        elif ctx.num():
            args = [self.visit(n) for n in ctx.num()]
            if any(a == Result.UNCERTAIN for a in args):
                return Result.UNCERTAIN
        func = str(ctx.FUNC())
        if func in self.helpers:
            helper = self.helpers[func]
            args = {
                arg['name']: val
                for (arg, val) in zip(helper['args'], args)
            }
            return self.visit(helper['pr'].tree, local_args=args)
        # Visit the func with local values set for the args
        if func in BUILTINS:
            if func == 'max':
                return max(map(int, args))
            if func == 'min':
                return min(map(int, args))
            if func == 'default':
                return 0
        return Result.UNCERTAIN
