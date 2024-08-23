from collections import defaultdict
from itertools import chain, zip_longest
import logging
import re

from grammar import RulesParser, RulesVisitor
from Utils import *

import inflection

REF_GETTER_TYPE = re.compile(r'(?:ctx\.|data::)([^(]*)\(')

class RustBaseVisitor(RulesVisitor):

    def __init__(self, rules, context_types, action_funcs, ctxdict, data_types, name):
        self.rules = rules
        self.context_types = context_types
        self.action_funcs = action_funcs
        self.ctxdict = ctxdict
        self.data_types = data_types
        self.name = name
        self.rettype = None

    def _getRefGetter(self, ref):
        if ref in self.data_types:
            return f'data::{ref}(ctx.position())'
        if ref in self.ctxdict:
            return f'ctx.{self.ctxdict[ref]}()'
        if func := self.action_funcs.get(self.name):
            if ref in func.get('args', {}):
                return ref
        return BUILTINS.get(ref, '$' + ref)
    
    def _getRefRaw(self, ref):
        return self.ctxdict.get(ref, ref)
    
    def _getRefSetter(self, ref):
        return f'ctx.set_{self.ctxdict[ref]}'

    def _getRefEnum(self, ref):
        return f'enums::{self.ctxdict[ref].capitalize()}'

    def _getRefType(self, ref):
        if isinstance(ref, RulesParser.RefContext):
            ref = str(ref.REF()[-1])[1:]
        if ref in self.context_types:
            return self.context_types[ref]
        if ref in self.data_types:
            return self.data_types[ref]
        # TODO: This probably needs to handle access funcs as well
        if func := self.action_funcs.get(self.name):
            if ref in func.get('args', {}):
                return func['args'][ref]

    def _isRefSpotId(self, ref):
        rtype = self._getRefType(ref)
        if rtype:
            return 'SpotId' == rtype
        return False

    def _getFuncAndArgs(self, func):
        if func in BUILTINS:
            if isinstance(BUILTINS[func], str):
                return (BUILTINS[func], [])
            else:
                return (BUILTINS[func][0], list(BUILTINS[func][1:]))
        elif func in self.rules:
            return (f'rule__{construct_id(func[1:])}!', ['ctx', 'world'])
        else:
            return (f'helper__{construct_id(func[1:])}!', ['ctx', 'world'])

    def visit(self, tree, rettype=None):
        last_rettype = self.rettype
        self.rettype = rettype
        try:
            ret = super().visit(tree)
            if ret is None:
                return 'todo!()'
            return ret
        except:
            logging.error(f'Encountered exception rendering {self.name}')
            raise
        finally:
            self.rettype = last_rettype

    def visitBoolExpr(self, ctx):
        ret = super().visitBoolExpr(ctx)
        if ret is None:
            return 'todo!()'
        return ret


class RustVisitor(RustBaseVisitor):

    def __init__(self, *args, **kwargs):
        super().__init__(*args, **kwargs)

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
        func, args = self._getFuncAndArgs(str(ctx.FUNC()))
        if items:
            args.extend(f'Item::{item}' for item in items)
            if func.startswith('ctx.collect'):
                args.append('world')
        elif ctx.value():
            args.append(str(self.visit(ctx.value())))
        elif ctx.PLACE():
            places = [str(p)[1:-1] for p in ctx.PLACE()]
            args.extend(construct_place_id(pl) for pl in places)
        elif ctx.ref():
            args.extend(self.visit(r) for r in ctx.ref())
        else:
            arg = f'{ctx.LIT() or ctx.INT() or ctx.FLOAT() or ""}'
            if arg:
                args.append(arg)
        if func.startswith('ctx.reset'):
            args.append('world')
        return f'{"!" if ctx.NOT() else ""}{func}({", ".join(args)})'

    def _visitConditional(self, *args, else_case='false'):
        ret = []
        while len(args) > 1:
            cond, then, *args = args
            ret.append(f'if {self.visit(cond)} {{ {self.visit(then)} }}')
        if args:
            ret.append(f'{{ {self.visit(args[0])} }}')
        elif else_case:
            ret.append(f'{{ {else_case} }}')
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
        m = REF_GETTER_TYPE.match(getter)
        assert m, f'rendered getter does not match pattern: {getter}, {ctx.value().toStringTree(ruleNames = RulesParser.ruleNames)}'
        rtype = inflection.camelize(m.group(1))
        return f'{getter} {ctx.getChild(1)} enums::{rtype}::{inflection.camelize(str(ctx.LIT())[1:-1])}'

    def visitFlagMatch(self, ctx):
        num = f'{self.visit(ctx.num())}'
        return f'({self.visit(ctx.value())} & {num}) == {num}'

    def visitRefEqSimple(self, ctx):
        ref = self.visit(ctx.ref())
        if ctx.ITEM():
            return f'{ref} {ctx.getChild(1)} Item::{ctx.ITEM()}'
        if ctx.PLACE():
            return f'{ref} {ctx.getChild(1)} {construct_place_id(str(ctx.PLACE()))}'
        return f'{ref} {ctx.getChild(1)} world.{ctx.SETTING()}'

    def _refEq(self, val1, val2, op, coerce=False):
        if coerce:
            return f'Into::<i32>::into({val1}) {op} {val2}.into()'
        return f'{val1} {op} {val2}'

    def visitRefEqRef(self, ctx):
        t = self._getRefType(ctx.ref(0))
        val1 = self.visit(ctx.ref(0))
        val2 = self.visit(ctx.ref(1))
        return self._refEq(val1, val2, ctx.getChild(1), coerce=t in int_types)

    def visitRefEqInvoke(self, ctx):
        t = self._getRefType(ctx.ref())
        val1 = self.visit(ctx.ref())
        val2 = self.visit(ctx.invoke())
        return self._refEq(val1, val2, ctx.getChild(1), coerce=t in int_types)

    def visitSetting(self, ctx):
        # TODO: dict settings?
        return f'world.{ctx.SETTING()}'

    def visitArgument(self, ctx):
        return self.visit(ctx.ref())

    def visitRef(self, ctx):
        ref = str(ctx.REF()[-1])[1:]
        if len(ctx.REF()) == 2:
            return f'data::{ref}({self._getRefGetter(str(ctx.REF(0))[1:])})'
        if ctx.PLACE():
            return f'data::{ref}({construct_place_id(str(ctx.PLACE()))})'
        return self._getRefGetter(ref)

    def visitPlace(self, ctx):
        if ctx.PLACE():
            return construct_place_id(str(ctx.PLACE()))
        return self.visit(ctx.ref())

    def visitItemCount(self, ctx):
        if ctx.INT():
            val = str(ctx.INT())
        else:
            val = f'world.{ctx.SETTING()}'
        return f'ctx.count(Item::{ctx.ITEM()}) >= {val}'

    def visitOneItem(self, ctx):
        return ('!' if ctx.NOT() else '') + f'ctx.has(Item::{ctx.ITEM()})'

    def visitOneArgument(self, ctx):
        ref = self.visit(ctx.ref())
        if ref.startswith('ctx.') or ref.startswith('data::'):
            return ref
        return f'ctx.has({ref})'
    
    # There's no need to optimize for bitflags here, as the compiler can handle that! Hopefully.
    def visitItemList(self, ctx):
        helper_args = [self._getFuncAndArgs(helper) for helper in map(str, ctx.FUNC())]
        helpers = [f'{helper}({", ".join(args)})' for helper, args in helper_args]
        items = [self.visit(item) for item in ctx.item()]
        return f'{" && ".join(items + helpers)}'

    def visitBaseNum(self, ctx):
        if ctx.INT():
            return str(ctx.INT())
        if ctx.FLOAT():
            return str(ctx.FLOAT())
        if ctx.SETTING():
            return f'world.{ctx::SETTING()}'
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

    def visitCondNum(self, ctx):
        return self._visitConditional(*[child for child in chain(*zip_longest(ctx.boolExpr(), ctx.num())) if child])
    
    def visitCondStr(self, ctx):
        return self._visitConditional(*[child for child in chain(*zip_longest(ctx.boolExpr(), ctx.str_())) if child])

    def visitRefInList(self, ctx):
        getter = self.visit(ctx.ref())
        values = [f'Item::{i}' for i in ctx.ITEM()]
        return f'matches!({getter}, {" | ".join(values)})'
    
    def visitRefStrInList(self, ctx):
        ref = str(ctx.ref().REF()[-1])
        rtype = self._getRefEnum(ref[1:])
        getter = self.visit(ctx.ref())
        values = [f'{rtype}::{inflection.camelize(str(lit)[1:-1])}' for lit in ctx.LIT()]
        return f'matches!({getter}, {" | ".join(values)})'
    
    # TODO: other REF/SETTING rules

    def visitStr(self, ctx):
        if ctx.LIT() and self.rettype:
            return f'{self.rettype}::{inflection.camelize(str(ctx.LIT())[1:-1])}'
        return self.visitChildren(ctx)

    def visitPerRefStr(self, ctx):
        ref = str(ctx.ref().REF()[-1])
        enum = self._getRefEnum(ref[1:])
        cases = [f'{enum}::{str(c)[1:-1].capitalize()}' for c in ctx.LIT()] + [str(c) for c in ctx.INT()] + ['_']
        results = [str(self.visit(s, self.rettype)) for s in ctx.str_()]
        return (f'match {self.visit(ctx.ref())} {{ '
                + ', '.join(f'{c} => {r}' for c, r in zip(cases, results))
                + '}')

    def visitSomewhere(self, ctx):
        places = defaultdict(list)
        for pl in ctx.PLACE():
            pl = str(pl)[1:-1]
            places[getPlaceType(pl)].append(pl)
        matchcase, elsecase = ('false', 'true') if ctx.NOT() else ('true', 'false')
        per_type = [('(match ctx.position()' if pt == 'SpotId' else f'(match get_{pt.lower()[:-2]}(ctx.position())')
                    + ' {'
                    + ' | '.join(construct_place_id(pl) for pl in plist)
                    + f' => {matchcase}, _ => {elsecase} }})'
                    for pt, plist in places.items()
                    ]
        return ' || '.join(per_type)

    def visitRefInPlaceRef(self, ctx):
        ptype = self._getRefType(ctx.ref(1))
        eq = '!' if ctx.NOT() else '='
        get = self.visit(ctx.ref(0))
        if ptype != 'SpotId':
            if self._isRefSpotId(ctx.ref(0)):
                get = f'{get} != SpotId::None && get_{ptype[:-2].lower()}({get})'
            else:
                get = f'get_{ptype[:-2].lower()}({get})'
        return f'{get} {eq}= {self.visit(ctx.ref(1))}'
    
    def visitRefInPlaceName(self, ctx):
        pl = str(ctx.PLACE())[1:-1]
        ptype = getPlaceType(pl)
        eq = '!' if ctx.NOT() else '='
        get = self.visit(ctx.ref())
        if ptype == 'SpotId':
            val = construct_spot_id(*place_to_names(pl))
        else:
            val = f'{ptype}::{construct_id(pl)}'
            if self._isRefSpotId(ctx.ref()):
                get = f'{get} != SpotId::None && get_{ptype[:-2].lower()}({get})'
            else:
                get = f'get_{ptype[:-2].lower()}({get})'
        return f'{get} {eq}= {val}'
    
    def visitRefInPlaceList(self, ctx):
        eq = '!' if ctx.NOT() else '='
        get = self.visit(ctx.ref())
        places = [str(pl)[1:-1] for pl in ctx.PLACE()]
        ptypes = [getPlaceType(pl) for pl in places]
        vals = [construct_spot_id(*place_to_names(pl))
                    if ptype == 'SpotId'
                    else f'{ptype}::{construct_id(pl)}'
                for (pl, ptype) in zip(places, ptypes)]
        all_types = set(ptypes)
        precheck = ''
        if len(all_types) == 1:
            ptype = all_types.pop()
            if ptype != 'SpotId':
                if self._isRefSpotId(ctx.ref()):
                    precheck = f'{get} != SpotId::None && '
                get = f'get_{ptype[:-2].lower()}({get})'
            return f'{precheck}{"!" if ctx.NOT() else ""}matches!({get}, {" | ".join(vals)})'
        
        if self._isRefSpotId(ctx.ref()):
            precheck = f'{get} != SpotId::None && '
        return f'{precheck}{" && ".join(
            f"get_{ptype[:-2].lower()}({get}) {eq}= {val}"
                if ptype != "SpotId"
                else f"{get} {eq}= {val}"
            for (val, ptype) in zip(vals, ptypes)
        )}'

    def visitRefInFunc(self, ctx):
        func = str(ctx.invoke().FUNC())[1:]
        assert func in ('default', 'get_area', 'get_region')
        eq = '!' if ctx.NOT() else '='
        get = self.visit(ctx.ref())
        if func == 'default':
            return f'{get} {eq}= {self.visit(ctx.invoke())}'
        if self._isRefSpotId(ctx.ref()):
            check = f'{get} != SpotId::None && '
        else:
            check = ''
        return (f'{check}{func}({get}) '
                f'{eq}= {self.visit(ctx.invoke())}')

    def visitFuncNum(self, ctx):
        func, args = self._getFuncAndArgs(str(ctx.FUNC()))
        if ctx.ITEM():
            args.append(f'Item::{ctx.ITEM()}')
        elif ctx.num():
            args.extend(str(self.visit(n)) for n in ctx.num())
        elif ctx.place():
            args.extend(str(self.visit(n)) for n in ctx.place())
        return f'{func}({", ".join(args)})'

    ## Action-specific
    def visitActions(self, ctx):
        return ' '.join(map(str, (self.visit(ch) for ch in ctx.action())))

    def visitSet(self, ctx):
        var = str(ctx.REF())[1:]
        if ctx.TRUE():
            val = 'true'
        elif ctx.FALSE():
            val = 'false'
        elif ctx.ref():
            val = self.visit(ctx.ref())
        elif ctx.PLACE():
            pl = str(ctx.PLACE())[1:-1]
            val = construct_place_id(pl)
        elif ctx.num():
            val = self.visit(ctx.num())
        else:
            val = self.visit(ctx.str_(), self._getRefEnum(var))
        return f'{self._getRefSetter(var)}({val});'

    def visitAlter(self, ctx):
        return f'ctx.{self._getRefRaw(str(ctx.REF())[1:])} {ctx.BINOP()}= {self.visit(ctx.num())};'
    
    def visitSwap(self, ctx):
        return f'std::mem::swap(&mut ctx.{self._getRefRaw(str(ctx.REF(0))[1:])}, &mut ctx.{self._getRefRaw(str(ctx.REF(1))[1:])});'

    def visitActionHelper(self, ctx):
        return self.visit(ctx.invoke()) + ';'
        
    def visitCondAction(self, ctx):
        if len(ctx.boolExpr()) == len(ctx.actions()):
            return self._visitConditional(*chain(*zip(ctx.boolExpr(), ctx.actions())), else_case=None)
        else:
            # explicit else case
            return self._visitConditional(*chain(*zip(ctx.boolExpr(), ctx.actions()[:-1]), ctx.actions()[-1:]), else_case=None)



class RustExplainerVisitor(RustBaseVisitor):

    def __init__(self, *args, **kwargs):
        super().__init__(*args, **kwargs)
        self.code_writer = RustVisitor(*args, **kwargs)

    def _getRefExplainerAndTag(self, ref, usage=None, var='r'):
        if usage is None:
            usage = self._getRefGetter(ref[1:])
        # we don't want to explain builtins or arguments.
        # Arguments could differ by call
        if ref in BUILTINS or ref[1:] in BUILTINS:
            return None, ref
        if usage[0] == '$':
            tag = f'{self.name}.{ref}'
            return (f'if let Some(v) = edict.get_mut(&"{tag}") {{ '
                    f'v.push_str(&format!(", {usage}: {{}}", {var})); }} '
                    f'else {{ edict.insert("{tag}", format!("{usage}: {{}}", {var})); }}', tag)
        elif ref[1:] in self.ctxdict:
            tag = ref[0] + self.ctxdict[ref[1:]]
        else:
            tag = ref
        return f'edict.insert("{tag}", format!("{{:?}}", {var}))', tag

    def _getExplainerFunc(self, func):
        if func in BUILTINS:
            return None
        elif func in self.rules:
            return f'rexplain__{construct_id(func[1:])}!'
        else:
            return f'hexplain__{construct_id(func[1:])}!'

    def visitBoolExpr(self, ctx):
        try:
            if ctx.OR():
                lines = [
                    f'let mut left = {self.visit(ctx.boolExpr(0))}',
                    # short-circuit logic
                    f'if left.0 {{ left }} else {{ let mut right = {self.visit(ctx.boolExpr(1))}',
                    'left.1.append(&mut right.1); (right.0, left.1) }',
                ]
            elif ctx.AND():
                lines = [
                    f'let mut left = {self.visit(ctx.boolExpr(0))}',
                    # short-circuit logic
                    f'if !left.0 {{ left }} else {{ let mut right = {self.visit(ctx.boolExpr(1))}',
                    'left.1.append(&mut right.1); (right.0, left.1) }',
                ]
            elif ctx.TRUE():
                return f'(true, vec![])'
            elif ctx.FALSE():
                return f'(false, vec![])'
            elif ctx.boolExpr():
                return f'({self.visit(ctx.boolExpr(0))})'
            elif ctx.NOT():
                lines = [
                    f'let val = {super().visitBoolExpr(ctx)}',
                    '(!val.0, val.1)'
                ]
            else:
                return super().visitBoolExpr(ctx)
            return f'{{ {"; ".join(lines)} }}'
        except AttributeError as e:
            raise AttributeError(str(e) + '; ' + ' '.join(
                f'[{c.toStringTree(ruleNames = RulesParser.ruleNames)}]'
                for c in ctx.boolExpr()))

    def visitInvoke(self, ctx):
        items = ctx.ITEM()
        func, args = self._getFuncAndArgs(str(ctx.FUNC()))
        if items:
            args.extend(f'Item::{item}' for item in items)
        elif ctx.value():
            # TODO: only visit once and include the explanation
            args.append(str(self.code_writer.visit(ctx.value())))
        elif ctx.PLACE():
            places = [str(p)[1:-1] for p in ctx.PLACE()]
            args.extend(construct_place_id(pl) for pl in places)
        elif ctx.ref():
            args.extend(self.code_writer.visit(r) for r in ctx.ref())
        else:
            arg = f'{ctx.LIT() or ctx.INT() or ctx.FLOAT() or ""}'
            if arg:
                args.append(arg)
        if efunc := self._getExplainerFunc(str(ctx.FUNC())):
            lines = [
                f'let (res, mut refs) = {efunc}({", ".join(args)}, edict)',
                f'edict.insert("{ctx.getText()}", format!("{{:?}}", res))',
                f'refs.push("{ctx.getText()}")',
                f'({"!" if ctx.NOT() else ""}res, refs)',
            ]
            if ctx.ref():
                # Insert before the last element
                lines[-1:-1] = [
                    f'let mut r = {self.visit(ctx.ref())}',
                    'refs.append(&mut r.1)',
                ]
        elif func == 'Default::default':
            return '(Default::default(), vec![])'
        else:
            lines = [
                f'let res = {func}({", ".join(args)})',
                f'edict.insert("{ctx.getText()}", format!("{{:?}}", res))',
                f'({"!" if ctx.NOT() else ""}res, vec!["{ctx.getText()}"])',
            ]
            if ctx.ref():
                # Replace the last element
                lines[-1:] = [
                    f'let mut r = {self.visit(ctx.ref())}',
                    f'let mut refs = vec!["{ctx.getText()}"]',
                    'refs.append(&mut r.1)',
                    f'({"!" if ctx.NOT() else ""}res, refs)'
                ]
        return f'{{ {"; ".join(lines)} }}'

    def _visitConditional(self, *args):
        cases = []
        while len(args) > 1:
            cond, then, *args = args
            cases.append("; ".join([
                f'let mut cond = {self.visit(cond)}',
                'refs.append(&mut cond.1)',
                'if cond.0 { let mut then = ' + str(self.visit(then)),
                'refs.append(&mut then.1)',
                '(then.0, refs) }',
            ]))
        if args:
            cases.append("; ".join([
                f'let mut then = {self.visit(then)}',
                'refs.append(&mut then.1)',
                '(then.0, refs)',
            ]))
        else:
            cases.append(' (false, refs)')
        return f'{{ let mut refs = Vec::new(); {" else { ".join(cases)} {" }" * (len(cases) - 1)} }}'

    def visitIfThenElse(self, ctx):
        return self._visitConditional(*ctx.boolExpr())

    def visitPyTernary(self, ctx):
        return self._visitConditional(ctx.boolExpr(1), ctx.boolExpr(0), ctx.boolExpr(2))

    def visitCmp(self, ctx):
        left_str = ctx.value().getText()
        right_str = ctx.num().getText()
        lines = [
            f'let mut refs = vec!["{left_str}", "{right_str}"]',
            f'let mut left = {self.visit(ctx.value())}',
            f'let mut right = {self.visit(ctx.num())}',
            f'edict.insert("{left_str}", format!("{{:?}}", left.0))',
            f'edict.insert("{right_str}", format!("{{:?}}", right.0))',
            'refs.append(&mut left.1)',
            'refs.append(&mut right.1)',
            f'(Into::<i32>::into(left.0) {ctx.getChild(1)} right.0.into(), refs)',
        ]
        return f'{{ {"; ".join(lines)} }}'

    def visitCmpStr(self, ctx):
        vstr = ctx.value().getText()
        # get the real getter to determine if this is a cvar/data
        getter = self.code_writer.visit(ctx.value())
        rtype = inflection.camelize(REF_GETTER_TYPE.match(getter).group(1))
        lines = [
            f'let mut refs = vec!["{vstr}"]',
            f'let mut left = {self.visit(ctx.value())}',
            f'let right = enums::{rtype}::{inflection.camelize(str(ctx.LIT())[1:-1])}',
            f'edict.insert("{vstr}", format!("{{}}", left.0))',
            'refs.append(&mut left.1)',
            f'(left.0 {ctx.getChild(1)} right, refs)',
        ]
        return f'{{ {"; ".join(lines)} }}'

    def visitFlagMatch(self, ctx):
        vstr = ctx.value().getText()
        nstr = ctx.num().getText()
        lines = [
            f'let mut refs = vec!["{vstr}", "{nstr}"]',
            f'let mut left = {self.visit(ctx.value())}',
            f'let mut right = {self.visit(ctx.num())}',
            f'edict.insert("{vstr}", format!("{{}}", left.0))',
            f'edict.insert("{nstr}", format!("{{}}", right.0))',
            'refs.append(&mut left.1)'
            'refs.append(&mut right.1)',
            f'((left & right) == right, vec!["{vstr}", "{nstr}"])'
        ]
        return f'{{ {"; ".join(lines)} }}'

    def visitRefEqSimple(self, ctx):
        rval = f'Item::{ctx.ITEM()}' if ctx.ITEM() else construct_place_id(str(ctx.PLACE())) if ctx.PLACE() else f'world.{ctx.SETTING()}'
        lines = [
            f'let left = {self.visit(ctx.ref())}',
            f'(left.0 {ctx.getChild(1)} {rval}, left.1)',
        ]
        return f'{{ {"; ".join(lines)} }}'

    def _refEq(self, val1, val2, op, coerce=False):
        lines = [
            f'let mut left = {val1}',
            f'let mut right = {val2}',
            'left.1.append(&mut right.1)',
        ]
        if coerce:
            lines.append(f'(Into::<i32>::into(left.0) {op} right.0.into(), left.1)')
        else:
            lines.append(f'(left.0 {op} right.0, left.1)')
        return f'{{ {"; ".join(lines)} }}'

    def visitRefEqRef(self, ctx):
        t = self._getRefType(ctx.ref(0))
        val1 = self.visit(ctx.ref(0))
        val2 = self.visit(ctx.ref(1))
        return self._refEq(val1, val2, ctx.getChild(1), coerce=t in int_types)

    def visitRefEqInvoke(self, ctx):
        t = self._getRefType(ctx.ref())
        val1 = self.visit(ctx.ref())
        val2 = self.visit(ctx.invoke())
        return self._refEq(val1, val2, ctx.getChild(1), coerce=t in int_types)

    def visitSetting(self, ctx):
        # TODO: dict settings?
        setting = ctx.SETTING().getText()
        lines = [
            f'let s = world.{ctx.SETTING()}',
            f'edict.insert("{setting}", format!("{{}}", s))',
            f'(s, vec!["{setting}"])',
        ]
        return f'{{ {"; ".join(lines)} }}'

    def visitRef(self, ctx):
        ref = str(ctx.REF()[-1])
        if len(ctx.REF()) == 2:
            ref0 = str(ctx.REF(0))
            getter = self._getRefGetter(ref0[1:])
            exp, tag = self._getRefExplainerAndTag(ref0, getter)
            lines = [
                f'let r = {getter}',
                f'let d = data::{ref[1:]}(r)',
                exp,
                f'edict.insert("{ctx.getText()}", format!("{{}}", d))',
                f'(d, vec!["{ref0}", "{ctx.getText()}"])',
            ]
            return f'{{ {"; ".join(lines)} }}'
        if ctx.PLACE():
            lines = [
                f'let d = data::{ref[1:]}({construct_place_id(str(ctx.PLACE()))})',
                f'edict.insert("{ctx.getText()}", format!("{{}}", d))',
                f'(d, vec!["{ctx.getText()}"])',
            ]
            return f'{{ {"; ".join(lines)} }}'

        getter = self._getRefGetter(ref[1:])
        exp, tag = self._getRefExplainerAndTag(ref, getter)
        if exp:
            return f'{{ let r = {getter}; {exp}; (r, vec!["{tag}"]) }}'
        return f'({getter}, vec![])'

    def visitPlace(self, ctx):
        if ctx.PLACE():
            return f'({construct_place_id(str(ctx.PLACE()))}, vec![])'
        return self.visit(ctx.ref())

    def visitItemCount(self, ctx):
        vstr = f'{ctx.ITEM()} count'
        lines = [
            f'let ct = ctx.count(Item::{ctx.ITEM()})',
            f'edict.insert("{vstr}", format!("{{}}", ct))',
        ]
        if ctx.INT():
            lines.append(f'(ct >= {ctx.INT()}, vec!["{vstr}"])')
        else:
            sstr = ctx.SETTING().getText()
            lines.extend([
                f'let s = world.{ctx.SETTING()}',
                f'edict.insert("{sstr}", format!("{{}}", s))',
                f'(ct >= s, vec!["{vstr}", "{sstr}"])',
            ])
        return f'{{ {"; ".join(lines)} }}'

    def visitOneItem(self, ctx):
        lines = [
            f'let h = ctx.has(Item::{ctx.ITEM()})',
            f'edict.insert("{ctx.ITEM()}", format!("{{}}", h))',
            f'({"!" if ctx.NOT() else ""}h, vec!["{ctx.ITEM()}"])',
        ]
        return f'{{ {"; ".join(lines)} }}'

    def visitOneArgument(self, ctx):
        getter = self.code_writer.visit(ctx.ref())
        if getter[0] != '$':
            return self.visit(ctx.ref())
        
        lines = [
            f'let r = {self.visit(ctx.ref())}',
            '(ctx.has(r.0), r.1)',
        ]
        return f'{{ {"; ".join(lines)} }}'
    
    # There's no need to optimize for bitflags here, as the compiler can handle that! Hopefully.
    def visitItemList(self, ctx):
        helpers = [f'{self._getExplainerFunc(helper)}(ctx, world, edict)' for helper in map(str, ctx.FUNC())]
        items = [self.visit(item) for item in ctx.item()]
        lines = [
            [
                # This tends to be one extra level of recursion than apparently necessary, but eh
                f'let mut h = {item}',
                f'refs.append(&mut h.1)',
                # short-circuit logic
                'if !h.0 { return (false, refs); }',
            ]
            for item in items + helpers
        ]
        lines[-1][-1] = '(h.0, refs)'
        return f'{{ let mut refs = Vec::new(); {"; ".join(chain.from_iterable(lines))} }}'

    def visitBaseNum(self, ctx):
        if ctx.INT():
            return f'({ctx.INT()}, vec![])'
        if ctx.FLOAT():
            return f'({ctx.FLOAT()}, vec![])'
        if ctx.SETTING():
            sstr = str(ctx.SETTING())
            lines = [
                f'let s = world.{ctx.SETTING()}',
                f'edict.insert("{sstr}", format!("{{}}", s))',
                f'(s, vec!["{sstr}"])',
            ]
            return f'{{ {"; ".join(lines)} }}'
        # TODO: constants
        return self.visitChildren(ctx)

    def visitMathNum(self, ctx):
        lines = [
            f'let mut left = {self.visit(ctx.baseNum())}',
            f'let mut right = {self.visit(ctx.num())}',
            'left.1.append(&mut right.1)',
            f'(left {ctx.BINOP()} right, left.1)'
        ]
        return f'{{ {"; ".join(lines)} }}'

    def visitPerItemInt(self, ctx):
        cases = list(map(str, ctx.INT())) + ['_']
        results = [str(self.visit(n)) for n in ctx.num()]
        vstr = f'{ctx.ITEM()} count'
        lines = [
            f'let mut refs = vec!["{vstr}"]',
            f'let ct = ctx.count(Item::{ctx.ITEM()})',
            f'edict.insert("{vstr}", format!("{{}}", ct))',
            ('let mut m = match ct { '
             + ', '.join(f'{i} => {r}' for i, r in zip(cases, results))
             + ', }'),
            'refs.append(&mut m.1)',
            'm'
        ]
        return f'{{ {"; ".join(lines)} }}'

    def visitRefInList(self, ctx):
        values = [f'Item::{i}' for i in ctx.ITEM()]
        getter = self.visit(ctx.ref())
        lines = [
            f'let r = {getter}',
            f'(matches!(r.0, {" | ".join(values)}), r.1)'
        ]
        return f'{{ {"; ".join(lines)} }}'
    
    def visitRefStrInList(self, ctx):
        ref = str(ctx.ref().REF()[-1])
        getter = self.visit(ctx.ref())
        rtype = self._getRefEnum(ref[1:])
        values = [f'{rtype}::{inflection.camelize(str(lit)[1:-1])}' for lit in ctx.LIT()]
        return f'{{ let r = {getter}; (matches!(r.0, {" | ".join(values)}), r.1) }}'
    
    # TODO: other REF/SETTING rules

    def visitStr(self, ctx):
        if ctx.LIT() and self.rettype:
            return f'{self.rettype}::{inflection.camelize(str(ctx.LIT())[1:-1])}'
        return super().visitStr(ctx)

    def visitPerRefStr(self, ctx):
        ref = str(ctx.ref().REF()[-1])
        getter = self.visit(ctx.ref())
        enum = self._getRefEnum(ref[1:])
        cases = [f'{enum}::{str(c)[1:-1].capitalize()}' for c in ctx.LIT()] + [str(c) for c in ctx.INT()] + ['_']
        results = [str(self.visit(s, self.rettype)) for s in ctx.str_()]
        lines = [
            f'let r = {getter}',
            f'(match r.0 {{ {", ".join(f"{c} => {r}" for c, r in zip(cases, results))} }}, r1)',
        ]
        return f'{{ {"; ".join(lines)} }}'

    def visitSomewhere(self, ctx):
        places = defaultdict(list)
        for pl in ctx.PLACE():
            pl = str(pl)[1:-1]
            places[getPlaceType(pl)].append(pl)
        matchcase, elsecase = ('false', 'true') if ctx.NOT() else ('true', 'false')
        per_type = [('match r' if pt == 'SpotId' else f'match get_{pt.lower()[:-2]}(r)')
                    + ' {'
                    + ' | '.join(construct_place_id(pl) for pl in plist)
                    + f' => {matchcase}, _ => {elsecase} }}'
                    for pt, plist in places.items()
                    ]
        exp, tag = self._getRefExplainerAndTag("^position", 'ctx.position()')
        return f'{{ let r = ctx.position(); {exp}; ({" || ".join(per_type)}, vec!["{tag}"]) }}'

    def visitRefInPlaceRef(self, ctx):    
        ptype = self._getRefType(ctx.ref(1))
        eq = '!' if ctx.NOT() else '='
        lines = [
            f'let mut r0 = {self.visit(ctx.ref(0))}',
            f'let mut r1 = {self.visit(ctx.ref(1))}',
            'r0.1.append(&mut r1.1)',
        ]
        if ptype == 'SpotId':
            lines.append(f'(r0.0 {eq}= r1.0, r0.1)')
        elif self._isRefSpotId(ctx.ref(0)):
            lines.append(f'(r0.0 != SpotId::None && get_{ptype[:-2].lower()}(r0.0) {eq}= r1.0, r0.1)')
        else:
            lines.append(f'(get_{ptype[:-2].lower()}(r0.0) {eq}= r1.0, r0.1)')
        return f'{{ {"; ".join(lines)} }}'
    
    def visitRefInPlaceName(self, ctx):
        pl = str(ctx.PLACE())[1:-1]
        ptype = getPlaceType(pl)
        eq = '!' if ctx.NOT() else '='
        lines = [
            f'let r = {self.visit(ctx.ref())}',
        ]
        if ptype == 'SpotId':
            lines.append(f'(r.0 {eq}= {construct_spot_id(*place_to_names(pl))}, r.1)')
        else:
            val = f'{ptype}::{construct_id(pl)}'
            if self._isRefSpotId(ctx.ref()):
                lines.append(f'(r.0 != SpotId::None && get_{ptype[:-2].lower()}(r.0) {eq}= {val}, r.1)')
            else:
                lines.append(f'(get_{ptype[:-2].lower()}(r.0) {eq}= {val}, r.1)')
        return f'{{ {"; ".join(lines)} }}'
    
    def visitRefInPlaceList(self, ctx):
        eq = '!' if ctx.NOT() else '='
        lines = [
            f'let r = {self.visit(ctx.ref())}',
        ]
        places = [str(pl)[1:-1] for pl in ctx.PLACE()]
        ptypes = [getPlaceType(pl) for pl in places]
        vals = [construct_spot_id(*place_to_names(pl))
                    if ptype == 'SpotId'
                    else f'{ptype}::{construct_id(pl)}'
                for (pl, ptype) in zip(places, ptypes)]
        all_ptypes = set(ptypes)
        precheck = ''
        if len(all_ptypes) == 1:
            ptype = all_ptypes.pop()
            if ptype == 'SpotId':
                lines.append(f'({"!" if ctx.NOT() else ""}matches!(r.0, {" | ".join(vals)}), r.1)')
                return f'{{ {"; ".join(lines)} }}'
            if self._isRefSpotId(ctx.ref()):
                precheck = 'r.0 != SpotId::None && '
            lines.append(f'({precheck}{"!" if ctx.NOT() else ""}matches!(get_{ptype[:-2].lower()}(r.0), {" | ".join(vals)}), r.1)')
        else:
            if self._isRefSpotId(ctx.ref()):
                precheck = f'r.0 != SpotId::None && '
            lines.append(f'({precheck}{" && ".join(
                f"get_{ptype[:-2].lower()}(r.0) {eq}= {val}"
                    if ptype != "SpotId"
                    else f"r.0 {eq}= {val}"
                for (val, ptype) in zip(vals, ptypes)
            )}, r.1)')

        return f'{{ {"; ".join(lines)} }}'

    def visitRefInFunc(self, ctx):
        func = str(ctx.invoke().FUNC())[1:]
        assert func in ('default', 'get_area', 'get_region')
        eq = '!' if ctx.NOT() else '='
        get = self.visit(ctx.ref())
        ftag = ctx.invoke().getText()
        lines = [
            f'let mut r = {get}',
            f'let res = {"r.0" if func == "default" else f"{func}(r.0)"}',
            f'let mut f = {self.visit(ctx.invoke())}',
            f'edict.insert("{ftag}", format!("{{}}", f.0))',
            'r.1.append(&mut f.1)',
        ]

        if func != 'default' and self._isRefSpotId(ctx.ref()):
            return f'{{ {"; ".join(lines)}; (res != SpotId::None && res {eq}= f.0, r.1) }}'
        return f'{{ {"; ".join(lines)}; (res {eq}= f.0, r.1) }}'

    def visitFuncNum(self, ctx):
        func, args = self._getFuncAndArgs(str(ctx.FUNC()))
        if ctx.ITEM():
            args.append(f'Item::{ctx.ITEM()}')
        elif ctx.num():
            args.extend(str(self.code_writer.visit(n)) for n in ctx.num())
        elif ctx.place():
            args.extend(str(self.code_writer.visit(n)) for n in ctx.place())
        tag = ctx.getText()
        lines = [
            f'let f = {func}({", ".join(args)})',
            f'edict.insert("{tag}", format!("{{}}", f))',
            f'(f, vec!["{tag}"])'
        ]
        return f'{{ {"; ".join(lines)} }}'


class RustObservationVisitor(RustBaseVisitor):
    def __init__(self, item_max_counts, collect_funcs, *args, **kwargs):
        super().__init__(*args, **kwargs)
        self.item_max_counts = item_max_counts
        self.collect_funcs = collect_funcs
        self.code_writer = RustVisitor(*args, **kwargs)
        self.resets = []

    def visit(self, tree, rettype=None):
        v = super().visit(tree, rettype=rettype)
        # if self.resets:
        #     return f'{{ {' '.join(self.resets) } {v} }}'
        return v

    def _getItemType(self, item):
        return get_int_type_for_max(self.item_max_counts[item])

    def _getObserverFunc(self, func):
        if ofunc := OBSERVER_BUILTINS.get(func):
            return ofunc
        elif func in BUILTINS:
            return None
        elif func in self.rules:
            return f'robserve__{construct_id(func[1:])}!'
        else:
            return f'hobserve__{construct_id(func[1:])}!'
        
    def _getItemObserver(self, item):
        if self._getItemType(item) == 'bool':
            return f'full_obs.observe_{item.lower()}();'
        return f'full_obs.observe_{item.lower()}(IntegerObservation::Exact);'

    def _getRefObserver(self, ref, op=None, var=None):
        if ref[0] == '^':
            ref = self._getRefRaw(ref[1:])
            if ty := self.context_types.get(ref):
                if ty in int_types:
                    cmp = f'{self._opToComparison(op, var, ty)}' if op else 'Exact'
                    return f'full_obs.observe_{ref}(IntegerObservation::{cmp});'
                else:
                    assert op not in ('>', '<', '>=', '<='), f'Int comparison of bool ref {ref} not allowed'
                    return f'full_obs.observe_{ref}();'

    def _opToComparison(self, op, v, t):
        if op == '>=' or op == '<':
            return f'Ge({v} as {t})'
        if op == '<=' or op == '>':
            return f'Le({v} as {t})'
        assert op == '==' or op == '!=', f'Invalid operand: "{op}"'
        return f'Eq({v} as {t})'

    def visitBoolExpr(self, ctx):
        try:
            if ctx.OR():
                return f'({self.visit(ctx.boolExpr(0))} || {self.visit(ctx.boolExpr(1))})'
            elif ctx.AND():
                return f'({self.visit(ctx.boolExpr(0))} && ({self.visit(ctx.boolExpr(1))}))'
            elif ctx.TRUE():
                return 'true'
            elif ctx.FALSE():
                return 'false'
            elif ctx.boolExpr():
                return f'({self.visit(ctx.boolExpr(0))})'
            elif ctx.NOT():
                return f'!({super().visitBoolExpr(ctx)})'
            else:
                return super().visitBoolExpr(ctx)
        except AttributeError as e:
            raise AttributeError(str(e) + '; ' + ' '.join(
                f'[{c.toStringTree(ruleNames = RulesParser.ruleNames)}]'
                for c in ctx.boolExpr()))
        
    def visitInvoke(self, ctx):
        items = ctx.ITEM()
        func = str(ctx.FUNC())
        ofunc = self._getObserverFunc(func)
        func, args = self._getFuncAndArgs(func)
        if items:
            args.extend(f'Item::{item}' for item in items)
            if func == 'ctx.collect':
                return f'{ofunc}(Item::{items[0]}, world, full_obs)'
        elif ctx.value():
            # settings and helper args shouldn't generally need to be observable
            args.append(str(self.code_writer.visit(ctx.value())))
        elif ctx.PLACE():
            places = [str(p)[1:-1] for p in ctx.PLACE()]
            args.extend(construct_place_id(pl) for pl in places)
        elif ctx.ref():
            args.extend(self.visit(ref) for ref in ctx.ref())
        else:
            arg = f'{ctx.LIT() or ctx.INT() or ctx.FLOAT() or ""}'
            if arg:
                args.append(arg)
        if func.startswith('ctx.reset'):
            args.append('world')
        if ofunc:
            return f'{ofunc}({", ".join(args)}, full_obs)'
        elif func == 'ctx.count':
            lines = [
                f'full_obs.observe_{item.lower()}(IntegerObservation::Exact);'
                for item in items
            ]
            lines.append('/* TODO: handle count() at the comparison layer */')
            lines.append(f'{func}({", ".join(args)})')
            return f'{{ {" ".join(lines)} }}'
        elif func.startswith('rules::'):
            return f'{func}({", ".join(args)}, full_obs)'
        else:
            return f'{func}({", ".join(args)})'

    def _visitConditional(self, *args, else_case='false'):
        lines = []
        while len(args) > 1:
            cond, then, *args = args
            lines.append(f'if {self.visit(cond)} {{ {self.visit(then)} }}')
        if args:
            lines.append(f'{{ {self.visit(args[0])} }}')
        elif else_case:
            lines.append(f'{{ {else_case} }}')
        return ' else '.join(lines)
    
    def visitIfThenElse(self, ctx):
        return self._visitConditional(*ctx.boolExpr())

    def visitPyTernary(self, ctx):
        return self._visitConditional(ctx.boolExpr(1), ctx.boolExpr(0), ctx.boolExpr(2))

    def visitCmp(self, ctx):
        if ctx.value().ref():
            op = str(ctx.getChild(1))
            ref = str(ctx.value().ref().REF()[-1])
            if obs := self._getRefObserver(ref, op=op, var='n'):
                lines = [
                    f'let n = {self.visit(ctx.num())} as i32;',
                    obs,
                    f'({self._getRefGetter(ref[1:])} as i32) {op} n'
                ]
                return f'{{ {" ".join(lines)} }}'
            # If we don't have an observation to make (i.e. a constant), fall through
        # Check for baseNum rather than mathNum to avoid annoyances
        if str(ctx.num()).startswith('$count('):
            lines = [
                f'let v = {self.visit(ctx.value())} as i32;',
            ]
            if ctx.num().baseNum():
                item = str(ctx.num().baseNum().funcNum().ITEM(0))
                ty = self._getItemType(item)
                if ty == 'bool':
                    lines.extend([
                        f'full_obs.observe_{item.lower()}();',
                        f'v {op} {self.code_writer.visit(ctx.num())}.into()'
                    ])
                else:
                    op = str(ctx.getChild(1))
                    lines.extend([
                        f'if v < {ty}::MAX as i32 {{',
                        # Mirror the op as this is the right operand
                        f'full_obs.observe_{item.lower()}(IntegerObservation::{self._opToComparison(mirror(op), 'v', ty)});',
                        f'}}',
                        f'v {op} {self.code_writer.visit(ctx.num())}.into()'
                    ])
            else:
                lines.extend([
                    '/* TODO: support $count in mathNum */',
                    f'v {ctx.getChild(1)} {self.visit(ctx.num())}.into()',
                ])
        else:
            # Observe exact values.
            lines = [
                f'let v = {self.visit(ctx.value())} as i32;',
                f'let n = {self.visit(ctx.num())} as i32;',
                f'v {ctx.getChild(1)} n',
            ]
        return f'{{ {" ".join(lines)} }}'

    # This could be easier if str enum values are required to be unique among all enums
    # otherwise we have to get the appropriate ref/setting enum
    def visitCmpStr(self, ctx):
        getter = self.code_writer.visit(ctx.value())
        rtype = inflection.camelize(REF_GETTER_TYPE.match(getter).group(1))
        lines = [
            f'let v = {self.visit(ctx.value())};',
            f'v {ctx.getChild(1)} enums::{rtype}::{inflection.camelize(str(ctx.LIT())[1:-1])}'
        ]
        return f'{{ {" ".join(lines)} }}'

    def visitFlagMatch(self, ctx):
        return f'{{ let n = {self.visit(ctx.num())}; ({self.visit(ctx.value())} & n) == n }}'

    def visitRefEqSimple(self, ctx):
        getter = self.visit(ctx.ref())
        if ctx.ITEM():
            return f'{{ let left = {getter}; left {ctx.getChild(1)} Item::{ctx.ITEM()} }}'
        elif ctx.PLACE():
            return f'{{ let left = {getter}; left {ctx.getChild(1)} {construct_place_id(str(ctx.PLACE()))} }}'
        else:
            return f'{{ let left = {getter}; left {ctx.getChild(1)} world.{ctx.SETTING()} }}'

    def visitRefEqRef(self, ctx):
        return f'{{ let left = {self.visit(ctx.ref(0))}; let right = {self.visit(ctx.ref(1))}; left {ctx.getChild(1)} right }}'

    def visitRefEqInvoke(self, ctx):
        return f'{{ let left = {self.visit(ctx.ref())}; let right = {self.visit(ctx.invoke())}; left {ctx.getChild(1)} right }}'

    def visitSetting(self, ctx):
        # TODO: dict settings?
        return f'world.{ctx.SETTING()}'

    def visitRef(self, ctx):
        if len(ctx.REF()) == 2:
            # Reading data based on ref0, so the only obs is ref0
            ref0 = str(ctx.REF(0))
            getter = self._getRefGetter(ref0[1:])
            if obs := self._getRefObserver(ref0):
                return f'{{ {obs} {self.code_writer.visit(ctx)} }}'
            return self.code_writer.visit(ctx)
        if ctx.PLACE():
            # No observations here, this is a constant
            return self.code_writer.visit(ctx)

        ref = str(ctx.REF()[-1])
        getter = self._getRefGetter(ref[1:])
        if obs := self._getRefObserver(ref):
            return f'{{ {obs} {getter} }}'
        return getter

    def visitPlace(self, ctx):
        if ctx.PLACE():
            return construct_place_id(str(ctx.PLACE()))
        return self.visit(ctx.ref())

    def visitItemCount(self, ctx):
        if ctx.INT():
            val = str(ctx.INT())
        else:
            val = f'world.{ctx.SETTING()}'
        item = str(ctx.ITEM())
        if self._getItemType(item) == 'bool':
            obs = f'full_obs.observe_{item.lower()}()'
        else:
            obs = f'full_obs.observe_{item.lower()}(IntegerObservation::Ge({val}))'
        return f'{{ {obs}; ctx.count(Item::{ctx.ITEM()}) >= {val} }}'

    def visitOneItem(self, ctx):
        item = str(ctx.ITEM())
        if self._getItemType(item) == 'bool':
            obs = f'full_obs.observe_{item.lower()}()'
        else:
            obs = f'full_obs.observe_{item.lower()}(IntegerObservation::Ge(1))'
        return f'{{ {obs}; {'!' if ctx.NOT() else ''}ctx.has(Item::{ctx.ITEM()}) }}'

    def visitOneArgument(self, ctx):
        getter = self.code_writer.visit(ctx.ref())
        if getter[0] != '$':
            return self.visit(ctx.ref())
        item = self.visit(ctx.ref())
        return f'{{ full_obs.observe_has_item({item}); ctx.has({item}) }}'

    # There's no need to optimize for bitflags here, as the compiler can handle that! Hopefully.
    def visitItemList(self, ctx):
        # These visits create the observations necessary.
        helpers = [f'{self._getObserverFunc(helper)}(ctx, world, full_obs)' for helper in map(str, ctx.FUNC())]
        items = [f'({self.visit(item)})' for item in ctx.item()]
        return f'{" && ".join(items + helpers)}'

    def visitMathNum(self, ctx):
        return f'{self.visit(ctx.baseNum())} {ctx.BINOP()} {self.visit(ctx.num())}'

    def visitPerItemInt(self, ctx):
        cases = list(map(str, ctx.INT())) + ['_']
        results = [str(self.visit(n)) for n in ctx.num()]
        item = str(ctx.ITEM())
        obs = self._getItemObserver(item)
        return (f'{{ {obs} match ctx.count(Item::{ctx.ITEM()}) {{ '
                + ', '.join(f'{i} => {r}' for i, r in zip(cases, results))
                + f'}} }}')

    def visitRefInList(self, ctx):
        values = [f'Item::{i}' for i in ctx.ITEM()]
        match = f'matches!({self.visit(ctx.ref())}, {" | ".join(values)})'
        return match
    
    def visitRefStrInList(self, ctx):
        ref = str(ctx.ref().REF()[-1])
        rtype = self._getRefEnum(ref[1:])
        values = [f'{rtype}::{inflection.camelize(str(lit)[1:-1])}' for lit in ctx.LIT()]
        match = f'matches!({self.visit(ctx.ref())}, {" | ".join(values)})'
        return match
    
    # TODO: other REF/SETTING rules
    # TODO: move unchanged functions to common class

    # unchanged
    visitBaseNum = RustVisitor.visitBaseNum
    visitStr = RustVisitor.visitStr
    visitPerRefStr = RustVisitor.visitPerRefStr
    visitSomewhere = RustVisitor.visitSomewhere
    visitRefInPlaceRef = RustVisitor.visitRefInPlaceRef
    visitRefInPlaceName = RustVisitor.visitRefInPlaceName
    visitRefInPlaceList = RustVisitor.visitRefInPlaceList
    visitRefInFunc = RustVisitor.visitRefInFunc
    visitFuncNum = RustVisitor.visitFuncNum

    ## Action-specific
    def visitActions(self, ctx):
        return ' '.join(map(str, (self.visit(ch) for ch in ctx.action())))

    def visitSet(self, ctx):
        var = str(ctx.REF())[1:]
        if ctx.TRUE():
            val = 'true'
        elif ctx.FALSE():
            val = 'false'
        elif ctx.ref():
            val = self.visit(ctx.ref())
        elif ctx.PLACE():
            pl = str(ctx.PLACE())[1:-1]
            val = construct_place_id(pl)
        elif ctx.num():
            val = self.visit(ctx.num())
        else:
            val = self.visit(ctx.str_(), self._getRefEnum(var))
        # Setting to a specific value means it does not matter what the value was before.
        return f'full_obs.clear_{self.ctxdict[var]}(); {self._getRefSetter(var)}({val});'

    def visitAlter(self, ctx):
        val = self.visit(ctx.num())
        if 'full_obs' not in val:
            return ''
        varname = self._getRefRaw(str(ctx.REF())[1:])
        return f'{{ let v = {val}; full_obs.observe_shift_{varname}(v); ctx.{varname} {ctx.BINOP()}= v; }}'
    
    def visitSwap(self, ctx):
        ref1 = str(ctx.REF(0))[1:]
        ref2 = str(ctx.REF(1))[1:]
        if ref2 < ref1:
            ref1, ref2 = ref2, ref1
        return f'full_obs.swap_{ref1}__{ref2}(); {self.code_writer.visit(ctx)}'

    def visitActionHelper(self, ctx):
        return self.visit(ctx.invoke()) + ';'
        
    def visitCondAction(self, ctx):
        if len(ctx.boolExpr()) == len(ctx.actions()):
            return self._visitConditional(*chain(*zip(ctx.boolExpr(), ctx.actions())), else_case=None)
        else:
            # explicit else case
            return self._visitConditional(*chain(*zip(ctx.boolExpr(), ctx.actions()[:-1]), ctx.actions()[-1:]), else_case=None)
