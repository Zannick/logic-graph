from collections import defaultdict
from itertools import chain
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
            logging.error(f'Encountered exception rendering {self.name}: {self.ctxdict}')
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
        elif ctx.REF():
            args.append(self._getRefGetter(str(ctx.REF())[1:]))
        else:
            arg = f'{ctx.LIT() or ctx.INT() or ctx.FLOAT() or ""}'
            if arg:
                args.append(arg)
        if func.startswith('ctx.reset'):
            args.append('world')
        return f'{"!" if ctx.NOT() else ""}{func}({", ".join(args)})'

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
        rtype = inflection.camelize(REF_GETTER_TYPE.match(getter).group(1))
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
        return f'world.{ctx.SETTING()}'

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
    
    # There's no need to optimize for bitflags here, as the compiler can handle that! Hopefully.
    def visitItemList(self, ctx):
        helper_args = [self._getFuncAndArgs(helper) for helper in map(str, ctx.FUNC())]
        helpers = [f'{helper}({", ".join(args)})' for helper, args in helper_args]
        items = [self.visit(item) for item in ctx.item()]
        return f'{" && ".join(items + helpers)}'

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
        getter = self._getRefGetter(str(ctx.REF())[1:])
        values = [f'Item::{i}' for i in ctx.ITEM()]
        return f'matches!({getter}, {" | ".join(values)})'
    
    def visitRefStrInList(self, ctx):
        ref = str(ctx.REF())[1:]
        getter = self._getRefGetter(ref)
        rtype = self._getRefEnum(ref)
        values = [f'{rtype}::{inflection.camelize(str(lit)[1:-1])}' for lit in ctx.LIT()]
        return f'matches!({getter}, {" | ".join(values)})'
    
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
        matchcase, elsecase = ('false', 'true') if ctx.NOT() else ('true', 'false')
        per_type = [('(match ctx.position()' if pt == 'SpotId' else f'(match get_{pt.lower()[:-2]}(ctx.position())')
                    + ' {'
                    + ' | '.join(construct_place_id(pl) for pl in plist)
                    + f' => {matchcase}, _ => {elsecase} }})'
                    for pt, plist in places.items()
                    ]
        return ' || '.join(per_type)

    def visitRefInPlaceRef(self, ctx):
        ptype = self._getRefType(str(ctx.REF(1))[1:])
        eq = '!' if ctx.NOT() else '='
        ref = str(ctx.REF(0))[1:]
        get = f'{self._getRefGetter(ref)}'
        if ptype != 'SpotId':
            if self._isRefSpotId(ref):
                get = f'{get} != SpotId::None && get_{ptype[:-2].lower()}({get})'
            else:
                get = f'get_{ptype[:-2].lower()}({get})'
        return f'{get} {eq}= {self._getRefGetter(str(ctx.REF(1))[1:])}'
    
    def visitRefInPlaceName(self, ctx):
        pl = str(ctx.PLACE())[1:-1]
        ptype = getPlaceType(pl)
        eq = '!' if ctx.NOT() else '='
        ref = str(ctx.REF())[1:]
        get = f'{self._getRefGetter(ref)}'
        if ptype == 'SpotId':
            val = construct_spot_id(*place_to_names(pl))
        else:
            val = f'{ptype}::{construct_id(pl)}'
            if self._isRefSpotId(ref):
                get = f'{get} != SpotId::None && get_{ptype[:-2].lower()}({get})'
            else:
                get = f'get_{ptype[:-2].lower()}({get})'
        return f'{get} {eq}= {val}'

    def visitRefInFunc(self, ctx):
        func = str(ctx.invoke().FUNC())[1:]
        assert func in ('default', 'get_area', 'get_region')
        eq = '!' if ctx.NOT() else '='
        ref = str(ctx.REF())[1:]
        get = self._getRefGetter(ref)
        if func == 'default':
            return f'{get} {eq}= {self.visit(ctx.invoke())}'
        if self._isRefSpotId(ref):
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
        return f'{func}({", ".join(args)})'

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
        return f'ctx.{self._getRefRaw(str(ctx.REF())[1:])} {ctx.BINOP()}= {self.visit(ctx.num())};'

    def visitActionHelper(self, ctx):
        return self.visit(ctx.invoke()) + ';'
        
    def visitCondAction(self, ctx):
        return self._visitConditional(*chain(*zip(ctx.boolExpr(), ctx.actions())), else_case=False)



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
                    f'v.push_str(format!(", {usage}: {{}}", {var})); }} '
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
            args.append(str(self.code_writer.visit(ctx.value())))
        elif ctx.PLACE():
            places = [str(p)[1:-1] for p in ctx.PLACE()]
            args.extend(construct_place_id(pl) for pl in places)
        elif ctx.REF():
            args.append(self._getRefGetter(str(ctx.REF())[1:]))
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
            if ctx.REF():
                ref = str(ctx.REF())
                exp, tag = self._getRefExplainerAndTag(ref)
                if exp:
                    # Insert before the last element
                    lines[-1:-1] = [
                        f'let r = {self._getRefGetter(ref[1:])}',
                        exp,
                        f'refs.push("{tag}")',
                    ]
        elif func == 'Default::default':
            return '(Default::default(), vec![])'
        else:
            lines = [
                f'let res = {func}({", ".join(args)})',
                f'edict.insert("{ctx.getText()}", format!("{{:?}}", res))',
                f'({"!" if ctx.NOT() else ""}res, vec!["{ctx.getText()}"])',
            ]
            if ctx.REF():
                ref = str(ctx.REF())
                exp, tag = self._getRefExplainerAndTag(ref)
                if exp:
                    # Replace the last element
                    lines[-1:] = [
                        f'let r = {self._getRefGetter(ref[1:])}',
                        exp,
                        f'({"!" if ctx.NOT() else ""}res, vec!["{ctx.getText()}", "{tag}"])'
                    ]
        return f'{{ {"; ".join(lines)} }}'

    def _visitConditional(self, *args):
        cases = []
        while len(args) > 1:
            cond, then, *args = args
            cases.append("; ".join([
                f'let mut cond = {self.visit(cond)}',
                'refs.append(cond.1)',
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

    def visitRefEq(self, ctx):
        ref = str(ctx.REF())
        getter = self._getRefGetter(ref[1:])
        rval = f'Item::{ctx.ITEM()}' if ctx.ITEM() else f'ctx.{ctx.SETTING()}()'
        lines = [
            f'let left = {getter}',
            f'edict.insert("{ref}", format!("{{}}", left))',
            f'(left == {rval}, vec!["{ref}"])',
        ]
        return f'{{ {"; ".join(lines)} }}'

    def visitSetting(self, ctx):
        # TODO: dict settings?
        setting = ctx.SETTING().getText()
        lines = [
            f'let s = world.{ctx.SETTING()}',
            f'edict.insert("{setting}", format!("{{}}", s))',
            f'(s, vec!["{setting}"])',
        ]
        return f'{{ {"; ".join(lines)} }}'

    def visitArgument(self, ctx):
        ref = str(ctx.REF())
        getter = self._getRefGetter(ref[1:])
        exp, tag = self._getRefExplainerAndTag(ref, getter)
        if exp:
            return f'{{ let r = {getter}; {exp}; (r, vec!["{tag}"]) }}'
        return f'({getter}, vec![])'

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
                f'let s = ctx.{ctx.SETTING()}()',
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
        ref = str(ctx.REF())
        getter = self._getRefGetter(ref[1:])
        exp, tag = self._getRefExplainerAndTag(ref, getter)
        lines = [
            f'let r = {getter}' if getter[0] != '$' else f'let r = ctx.has({getter})',
            exp,
            f'(r, vec!["{tag}"])',
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
        if ctx.REF():
            ref = str(ctx.REF())
            getter = self._getRefGetter(ref[1:])
            exp, tag = self._getRefExplainerAndTag(ref, getter)
            if exp:
                lines = [
                    f'let r = {getter}',
                    exp,
                    f'(r, vec!["{tag}"])'
                ]
                return f'{{ {"; ".join(lines)} }}'
            return f'({getter}, vec![])'
        if ctx.SETTING():
            sstr = str(ctx.SETTING())
            lines.extend([
                f'let s = ctx.{ctx.SETTING()}()',
                f'edict.insert("{sstr}", format!("{{}}", s))',
                f'(s, vec!["{sstr}"])',
            ])
        # TODO: constants
        return self.visitChildren(ctx)

    def visitMathNum(self, ctx):
        lines = [
            f'let mut left = {self.visit(ctx.baseNum())}',
            f'let mut right = {self.visit(ctx.num())}',
            'left.1.append(&mut right)',
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
        ref = str(ctx.REF())
        getter = self._getRefGetter(ref[1:])
        values = [f'Item::{i}' for i in ctx.ITEM()]
        exp, tag = self._getRefExplainerAndTag(ref, getter)
        lines = [
            f'let r = {getter}',
            f'(matches!(r, {" | ".join(values)}), vec!["{tag}"])'
        ]
        if exp:
            # Insert before last line
            lines[-1:-1] = [exp]
        return f'{{ {"; ".join(lines)} }}'
    
    def visitRefStrInList(self, ctx):
        ref = str(ctx.REF())
        getter = self._getRefGetter(ref[1:])
        rtype = self._getRefEnum(ref[1:])
        values = [f'{rtype}::{inflection.camelize(str(lit)[1:-1])}' for lit in ctx.LIT()]
        exp, tag = self._getRefExplainerAndTag(ref, getter)
        if exp:
            return f'{{ let r = {getter}; {exp}; (matches!(r, {" | ".join(values)}), vec!["{tag}"]) }}'
        return f'(matches!({getter}, {" | ".join(values)}), vec!["{tag}"])'
    
    # TODO: other REF/SETTING rules

    def visitStr(self, ctx):
        if ctx.LIT() and self.rettype:
            return f'{self.rettype}::{inflection.camelize(str(ctx.LIT())[1:-1])}'
        return super().visitStr(ctx)

    def visitPerRefStr(self, ctx):
        ref = str(ctx.REF())
        getter = self._getRefGetter(ref[1:])
        enum = self._getRefEnum(ref[1:])
        cases = [f'{enum}::{str(c)[1:-1].capitalize()}' for c in ctx.LIT()] + [str(c) for c in ctx.INT()] + ['_']
        results = [str(self.visit(s, self.rettype)) for s in ctx.str_()]
        exp, tag = self._getRefExplainerAndTag(ref, getter)
        lines = [
            f'let r = {getter}',
            f'(match r {{ {", ".join(f"{c} => {r}" for c, r in zip(cases, results))} }}, vec!["{tag}"]',
        ]
        if exp:
            # Insert before the last line
            lines[-1:-1] = exp
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
        ref0 = str(ctx.REF(0))
        ref1 = str(ctx.REF(1))
        ptype = self._getRefType(ref1[1:])
        eq = '!' if ctx.NOT() else '='
        ref = str(ctx.REF(0))[1:]
        get0 = self._getRefGetter(ref0[1:])
        get1 = self._getRefGetter(ref1[1:])
        exp0, tag0 = self._getRefExplainerAndTag(ref0, get0, 'r0')
        exp1, tag1 = self._getRefExplainerAndTag(ref1, get1, 'r1')
        lines = [
            f'let r0 = {get0}{"; " + exp0 if exp0 else ""}',
            f'let r1 = {get1}{"; " + exp1 if exp1 else ""}',
        ]
        if ptype == 'SpotId':
            lines.append(f'(r0 {eq}= r1, vec!["{tag0}", "{tag1}"])')
        elif self._isRefSpotId(ref0[1:]):
            lines.append(f'(r0 != SpotId::None && get_{ptype[:-2].lower()}(r0) {eq}= r1, vec!["{tag0}", "{tag1}"])')
        else:
            lines.append(f'(get_{ptype[:-2].lower()}(r0) {eq}= r1, vec!["{tag0}", "{tag1}"])')
        return f'{{ {"; ".join(lines)} }}'
    
    def visitRefInPlaceName(self, ctx):
        pl = str(ctx.PLACE())[1:-1]
        ptype = getPlaceType(pl)
        eq = '!' if ctx.NOT() else '='
        ref = str(ctx.REF())
        get = self._getRefGetter(ref[1:])
        exp, tag = self._getRefExplainerAndTag(ref, get)
        lines = [
            f'let r = {get}',
        ]
        if exp:
            lines.append(exp)
        if ptype == 'SpotId':
            lines.append(f'(r {eq}= {construct_spot_id(*place_to_names(pl))}, vec!["{tag}"])')
        else:
            val = f'{ptype}::{construct_id(pl)}'
            if self._isRefSpotId(ref):
                lines.append(f'(r != SpotId::None && get_{ptype[:-2].lower()}(r) {eq}= {val}, vec!["{tag}"])')
            else:
                lines.append(f'(get_{ptype[:-2].lower()}(r) {eq}= {val}, vec!["{tag}"])')
        return f'{{ {"; ".join(lines)} }}'

    def visitRefInFunc(self, ctx):
        func = str(ctx.invoke().FUNC())[1:]
        assert func in ('default', 'get_area', 'get_region')
        eq = '!' if ctx.NOT() else '='
        ref = str(ctx.REF())
        get = self._getRefGetter(ref[1:])
        exp, tag = self._getRefExplainerAndTag(ref, get)
        ftag = ctx.invoke().getText()
        lines = [
            f'let mut refs = vec!["{tag}"]',
            f'let r = {get if func == "default" else f"{func}({get})"}',
            f'let mut f = {self.visit(ctx.invoke())}',
            f'edict.insert("{ftag}", format!("{{}}", f.0))',
            'refs.append(&mut f.1)',
        ]
        if exp:
            lines.append(exp)

        if func != 'default' and self._isRefSpotId(ref):
            return f'{{ {"; ".join(lines)}; (r != SpotId::None && r {eq}= f.0, refs) }}'
        return f'{{ {"; ".join(lines)}; (r {eq}= f.0, refs) }}'

    def visitFuncNum(self, ctx):
        func, args = self._getFuncAndArgs(str(ctx.FUNC()))
        if ctx.ITEM():
            args.append(f'Item::{ctx.ITEM()}')
        elif ctx.num():
            args.extend(str(self.code_writer.visit(n)) for n in ctx.num())
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

    def _getItemType(self, item):
        return get_int_type_for_max(self.item_max_counts[item])

    def _getObserverFunc(self, func):
        if func in BUILTINS:
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
        # We never need these to be observed.
        if func in ('$add_item', '$skip', '$visit', '$reset_area', '$reset_region'):
            return ''
        func, args = self._getFuncAndArgs(func)
        if items:
            args.extend(f'Item::{item}' for item in items)
            if func == 'ctx.collect':
                item = str(items[0])
                if item in self.collect_funcs:
                    func = f'rules::observe_action_{self.collect_funcs[item]['action_id']}'
                    return f'{func}(ctx, world, full_obs)'
                else:
                    return ''
        elif ctx.value():
            args.append(str(self.code_writer.visit(ctx.value())))
        elif ctx.PLACE():
            places = [str(p)[1:-1] for p in ctx.PLACE()]
            args.extend(construct_place_id(pl) for pl in places)
        elif ctx.REF():
            args.append(self._getRefGetter(str(ctx.REF())[1:]))
        else:
            arg = f'{ctx.LIT() or ctx.INT() or ctx.FLOAT() or ""}'
            if arg:
                args.append(arg)
        if ofunc := self._getObserverFunc(str(ctx.FUNC())):
            lines = []
            if ctx.REF():
                if obs := self._getRefObserver(str(ctx.REF())):
                    lines.append(obs)
            lines.append(f'{ofunc}({", ".join(args)}, full_obs)')
        elif func == 'ctx.count':
            lines = [
                f'full_obs.observe_{item.lower()}(IntegerObservation::Exact);'
                for item in items
            ]
            lines.append('/* TODO: handle count() at the comparison layer */')
            lines.append(f'{func}({", ".join(args)})')
        else:
            lines = []
            if ctx.REF():
                if obs := self._getRefObserver(str(ctx.REF())):
                    lines.append(obs)
            if func.startswith('rules::'):
                lines.append(f'{func}({", ".join(args)}, full_obs)')
            else:
                lines.append(f'{func}({", ".join(args)})')
        if len(lines) == 1:
            return lines[0]
        return f'{{ {" ".join(lines)} }}'

    def _visitConditional(self, *args, else_case=True):
        lines = []
        while len(args) > 1:
            cond, then, *args = args
            lines.append(f'if {self.visit(cond)} {{ {self.visit(then)} }}')
        if args:
            lines.append(f'{{ {self.visit(args[0])} }}')
        elif else_case:
            lines.append('{ false }')
        return ' else '.join(lines)
    
    def visitIfThenElse(self, ctx):
        return self._visitConditional(*ctx.boolExpr())

    def visitPyTernary(self, ctx):
        return self._visitConditional(ctx.boolExpr(1), ctx.boolExpr(0), ctx.boolExpr(2))

    def visitCmp(self, ctx):
        if ctx.value().REF():
            op = str(ctx.getChild(1))
            ref = str(ctx.value().REF())
            if obs := self._getRefObserver(ref, op=op, var='n'):
                lines = [
                    f'let n: i32 = {self.visit(ctx.num())}.into();',
                    obs,
                    f'i32::from({self._getRefGetter(ref[1:])}) {op} n'
                ]
                return f'{{ {" ".join(lines)} }}'
            # If we don't have an observation to make (i.e. a constant), fall through
        # Check for baseNum rather than mathNum to avoid annoyances
        if str(ctx.num()).startswith('$count('):
            lines = [
                f'let v: i32 = {self.visit(ctx.value())}.into();',
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
                f'let v: i32 = {self.visit(ctx.value())}.into();',
                f'let n: i32 = {self.visit(ctx.num())}.into();',
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

    def visitRefEq(self, ctx):
        ref = str(ctx.REF())
        getter = self._getRefGetter(ref[1:])
        lines = []
        if obs := self._getRefObserver(ref):
            lines.append(obs)
        if ctx.ITEM():
            lines.append(f'{getter} == Item::{ctx.ITEM()}')
        else:
            lines.append(f'{getter} == ctx.{ctx.SETTING()}()')
        if len(lines) == 1:
            return lines[0]
        return f'{{ {" ".join(lines)} }}'

    def visitSetting(self, ctx):
        # TODO: dict settings?
        return f'world.{ctx.SETTING()}'

    def visitArgument(self, ctx):
        ref = str(ctx.REF())
        getter = self._getRefGetter(ref[1:])
        if obs := self._getRefObserver(ref):
            return f'{{ {obs} {getter} }}'
        return getter

    def visitItemCount(self, ctx):
        if ctx.INT():
            val = str(ctx.INT())
        else:
            val = f'ctx.{ctx.SETTING()}()'
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
        ref = str(ctx.REF())
        getter = self._getRefGetter(ref[1:])
        if obs := self._getRefObserver(ref):
            return f'{{ {obs} {getter} }}'
        elif getter.startswith('data::'):
            return getter
        return f'/* TODO: runtime observe_item */ ctx.has({getter})'

    # There's no need to optimize for bitflags here, as the compiler can handle that! Hopefully.
    def visitItemList(self, ctx):
        # These visits create the observations necessary.
        helpers = [f'{self._getObserverFunc(helper)}(ctx, world, full_obs)' for helper in map(str, ctx.FUNC())]
        items = [f'({self.visit(item)})' for item in ctx.item()]
        return f'{" && ".join(items + helpers)}'

    def visitBaseNum(self, ctx):
        if ctx.INT():
            return str(ctx.INT())
        if ctx.REF():
            ref = str(ctx.REF())
            getter = self._getRefGetter(ref[1:])
            if obs := self._getRefObserver(ref):
                return f'{{ {obs} {getter} }}'
            return getter
        if ctx.SETTING():
            return f'ctx.{ctx::SETTING()}()'
        # TODO: constants
        return self.visitChildren(ctx)

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
        ref = str(ctx.REF())
        getter = self._getRefGetter(ref[1:])
        values = [f'Item::{i}' for i in ctx.ITEM()]
        match = f'matches!({getter}, {" | ".join(values)})'
        if obs := self._getRefObserver(ref):
            return f'{{ {obs} {match} }}'
        return match
    
    def visitRefStrInList(self, ctx):
        ref = str(ctx.REF())
        getter = self._getRefGetter(ref[1:])
        rtype = self._getRefEnum(ref[1:])
        values = [f'{rtype}::{inflection.camelize(str(lit)[1:-1])}' for lit in ctx.LIT()]
        match = f'matches!({getter}, {" | ".join(values)})'
        if obs := self._getRefObserver(ref):
            return f'{{ {obs} {match} }}'
        return match
    
    # TODO: other REF/SETTING rules
    # TODO: move unchanged functions to common class

    # unchanged
    def visitStr(self, ctx):
        if ctx.LIT() and self.rettype:
            return f'{self.rettype}::{inflection.camelize(str(ctx.LIT())[1:-1])}'
        return super().visitStr(ctx)

    def visitPerRefStr(self, ctx):
        ref = str(ctx.REF())
        enum = self._getRefEnum(ref[1:])
        cases = [f'{enum}::{str(c)[1:-1].capitalize()}' for c in ctx.LIT()] + [str(c) for c in ctx.INT()] + ['_']
        results = [str(self.visit(s, self.rettype)) for s in ctx.str_()]
        match = (f'match {self._getRefGetter(ref[1:])} {{ '
                 + ', '.join(f'{c} => {r}' for c, r in zip(cases, results))
                 + f' }}')
        if obs := self._getRefObserver(ref):
            return f'{{ {obs} {match} }}'
        return match

    # unchanged
    def visitSomewhere(self, ctx):
        places = defaultdict(list)
        for pl in ctx.PLACE():
            pl = str(pl)[1:-1]
            places[getPlaceType(pl)].append(pl)
        matchcase, elsecase = ('false', 'true') if ctx.NOT() else ('true', 'false')
        per_type = [('(match ctx.position()' if pt == 'SpotId' else f'(match get_{pt.lower()[:-2]}(ctx.position())')
                    + f' {{'
                    + ' | '.join(construct_place_id(pl) for pl in plist)
                    + f' => {matchcase}, _ => {elsecase} }})'
                    for pt, plist in places.items()
                    ]
        return ' || '.join(per_type)

    def visitRefInPlaceRef(self, ctx):
        ref1 = str(ctx.REF(0))
        ref2 = str(ctx.REF(1))
        ptype = self._getRefType(ref2[1:])
        eq = '!' if ctx.NOT() else '='
        get1 = self._getRefGetter(ref1[1:])
        get2 = self._getRefGetter(ref2[1:])
        if ptype != 'SpotId':
            if self._isRefSpotId(ref1[1:]):
                get1 = f'{get1} != SpotId::None && get_{ptype[:-2].lower()}({get1})'
            else:
                get1 = f'get_{ptype[:-2].lower()}({get1})'
        obs1 = self._getRefObserver(ref1)
        obs2 = self._getRefObserver(ref2)
        if obs1 or obs2:
            return f'{{ {obs1 or ''} {obs2 or ''} {get1} {eq}= {get2} }}'
        return f'{get1} {eq}= {get2}'

    def visitRefInPlaceName(self, ctx):
        pl = str(ctx.PLACE())[1:-1]
        ptype = getPlaceType(pl)
        eq = '!' if ctx.NOT() else '='
        ref = str(ctx.REF())
        get = f'{self._getRefGetter(ref[1:])}'
        if ptype == 'SpotId':
            val = construct_spot_id(*place_to_names(pl))
        else:
            val = f'{ptype}::{construct_id(pl)}'
            if self._isRefSpotId(ref[1:]):
                get = f'{get} != SpotId::None && get_{ptype[:-2].lower()}({get})'
            else:
                get = f'get_{ptype[:-2].lower()}({get})'
        if obs := self._getRefObserver(ref):
            return f'{{ {obs} {get} {eq}= {val} }}'
        return f'{get} {eq}= {val}'

    def visitRefInFunc(self, ctx):
        func = str(ctx.invoke().FUNC())[1:]
        assert func in ('default', 'get_area', 'get_region')
        eq = '!' if ctx.NOT() else '='
        ref = str(ctx.REF())
        get = self._getRefGetter(ref[1:])
        obs = self._getRefObserver(ref)
        if func == 'default':
            res = f'{get} {eq}= {self.visit(ctx.invoke())}'
            if obs:
                return f'{{ {obs} {res} }}'
            return res
        if self._isRefSpotId(ref[1:]):
            check = f'{get} != SpotId::None && '
        else:
            check = ''
        inv = f'{check}{func}({get}) {eq}= {self.visit(ctx.invoke())}'
        if obs:
            return f'{{ {obs} {inv} }}'
        return inv

    # unchanged
    def visitFuncNum(self, ctx):
        func, args = self._getFuncAndArgs(str(ctx.FUNC()))
        if ctx.ITEM():
            args.append(f'Item::{ctx.ITEM()}')
        elif ctx.num():
            args.extend(str(self.visit(n)) for n in ctx.num())
        return f'{func}({", ".join(args)})'

    ## Action-specific
    # We have to eliminate all the ctx mutations, we're only interested in conditions
    def visitActions(self, ctx):
        return ' '.join(map(str, (self.visit(ch) for ch in ctx.action())))

    def visitSet(self, ctx):
        var = str(ctx.REF(0))[1:]
        if ctx.num():
            val = self.visit(ctx.num())
        elif ctx.str_():
            val = self.visit(ctx.str_(), self._getRefEnum(var))
        else:
            return ''
        if 'full_obs' not in val:
            return ''
        return f'{{ let _set = {val}; }}'

    def visitAlter(self, ctx):
        val = self.visit(ctx.num())
        if 'full_obs' not in val:
            return ''
        return f'{{ let _alter = {val}; }}'

    def visitActionHelper(self, ctx):
        return self.visit(ctx.invoke()) + ';'
        
    def visitCondAction(self, ctx):
        return self._visitConditional(*chain(*zip(ctx.boolExpr(), ctx.actions())), else_case=False)
