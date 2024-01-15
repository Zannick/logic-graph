from collections import defaultdict
from itertools import chain
import logging
import re

from grammar import RulesParser, RulesVisitor
from Utils import construct_id, construct_place_id, construct_spot_id, getPlaceType, place_to_names, BUILTINS

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
        return f'ctx.{self.ctxdict[ref]}'
    
    def _getRefSetter(self, ref):
        return f'ctx.set_{self.ctxdict[ref]}'

    def _getRefEnum(self, ref):
        return f'enums::{ref.capitalize()}'
    
    def _isRefSpotId(self, ref):
        if ref in self.context_types:
            return 'SpotId' == self.context_types[ref]
        if ref in self.data_types:
            return 'SpotId' == self.data_types[ref]
        # TODO: This probably needs to handle access funcs as well
        if func := self.action_funcs.get(self.name):
            if ref in func.get('args', {}):
                return 'SpotId' == func['args'][ref]
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
            return super().visit(tree)
        except:
            logging.error(f'Encountered exception rendering {self.name}: {self.ctxdict}')
            raise
        finally:
            self.rettype = last_rettype


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
        return f'matches!({getter}, {' | '.join(values)})'
    
    def visitRefStrInList(self, ctx):
        getter = self._getRefGetter(str(ctx.REF())[1:])
        rtype = inflection.camelize(REF_GETTER_TYPE.match(getter).group(1))
        values = [f'enums::{rtype}::{inflection.camelize(str(lit)[1:-1])}' for lit in ctx.LIT()]
        return f'matches!({getter}, {' | '.join(values)})'
    
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

    def visitRefInPlaceRef(self, ctx):
        ptype = self.context_types[str(ctx.REF(1))[1:]]
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
        eq = '!' if ctx.NOT() else '='
        ref = str(ctx.REF())[1:]
        get = self._getRefGetter(ref)
        if func == 'default':
            return f'{get} {eq}= {self.visit(ctx.invoke())}'
        assert func in ('get_area', 'get_region')
        if self._isRefSpotId(ref):
            check = f'{get} != SpotId && '
        else:
            check = ''
        return (f'{check}{func}({get}) '
                f'{eq}= {self.visit(ctx.invoke())}')

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
        func, args = self._getFuncAndArgs(str(ctx.FUNC()))
        if ctx.ITEM():
            args.append(f'Item::{ctx.ITEM()}')
        elif ctx.num():
            args.extend(str(self.visit(n)) for n in ctx.num())
        return f'{func}({", ".join(args)})'
        
    def visitActionHelper(self, ctx):
        return self.visit(ctx.invoke()) + ';'
        
    def visitCondAction(self, ctx):
        return self._visitConditional(*chain(*zip(ctx.boolExpr(), ctx.actions())), else_case=False)



class RustExplainerVisitor(RustBaseVisitor):

    def __init__(self, *args, **kwargs):
        super().__init__(*args, **kwargs)
        self.code_writer = RustVisitor(*args, **kwargs)

    def _getRefExplainer(self, ref):
        get = self._getRefGetter(ref[1:])
        # we don't want to explain builtins or arguments.
        # Arguments could differ by call
        if ref[0] == '$' or get[0] == '$':
            return None
        return f'edict.insert("{ref}", format!("{{:?}}", {get}))'

    def _getExplainerFunc(self, func):
        if func in BUILTINS:
            return None
        elif func in self.rules:
            return f'rexplain__{construct_id(func[1:])}!'
        else:
            return f'hexplain__{construct_id(func[1:])}!'

    # TODO: Remove the "or (false, vec![])" parts when all visits are supported
    def visit(self, ctx):
        val = super().visit(ctx)
        if val is None:
            return '(false, vec![])'
        return val

    def visitBoolExpr(self, ctx):
        try:
            if ctx.OR():
                lines = [
                    f'let mut left = {self.visit(ctx.boolExpr(0))}',
                    # short-circuit logic
                    'if left.0 { left } else { let mut right = ' + str(self.visit(ctx.boolExpr(1))),
                    'left.1.append(&mut right.1); (right.0, left.1) }',
                ]
            elif ctx.AND():
                lines = [
                    f'let mut left = {self.visit(ctx.boolExpr(0))}',
                    # short-circuit logic
                    'if !left.0 { left } else { let mut right = ' + str(self.visit(ctx.boolExpr(1))),
                    'left.1.append(&mut right.1); (right.0, left.1) }',
                ]
            elif ctx.TRUE():
                return f'(true, vec![])'
            elif ctx.FALSE():
                return f'(false, vec![])'
            elif ctx.boolExpr():
                return f'({self.visit(ctx.boolExpr(0))})'
            elif ctx.NOT():
                # TODO: Remove the "or (false, vec![])" parts
                lines = [
                    f'let val = {super().visitBoolExpr(ctx) or "(false, vec![])"}',
                    '(!val.0, val.1)'
                ]
            else:
                return super().visitBoolExpr(ctx) or "(false, vec![])"
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
        efunc = self._getExplainerFunc(str(ctx.FUNC()))
        if efunc:
            lines = [
                f'let (res, mut refs) = {efunc}({", ".join(args)}, edict)',
                f'edict.insert("{ctx.getText()}", format!("{{:?}}", res))',
                f'refs.push("{ctx.getText()}")',
                f'({"!" if ctx.NOT() else ""}res, refs)',
            ]
            if ctx.REF():
                ref = str(ctx.REF())
                if exp := self._getRefExplainer(ref):
                    # Insert before the last element
                    lines[-1:-1] = [
                        exp,
                        f'refs.push("{ref}")',
                    ]
        else:
            lines = [
                f'let res = {func}({", ".join(args)})',
                f'edict.insert("{ctx.getText()}", format!("{{:?}}", res))',
                f'({"!" if ctx.NOT() else ""}res, vec!["{ctx.getText()}"])',
            ]
            if ctx.REF():
                ref = str(ctx.REF())
                if exp := self._getRefExplainer(ref):
                    # Replace the last element
                    lines[-1:] = [
                        exp,
                        f'({"!" if ctx.NOT() else ""}res, vec!["{ctx.getText()}", "{ref}"])'
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
            f'let s = ctx.{ctx.SETTING()}()',
            f'edict.insert("{setting}", format!("{{}}", s))',
            f'(s, vec!["{setting}"])',
        ]
        return f'{{ {"; ".join(lines)} }}'

    def visitArgument(self, ctx):
        ref = str(ctx.REF())
        getter = self._getRefGetter(ref[1:])
        if exp := self._getRefExplainer(ref):
            return f'{{ {exp}; ({getter}, vec!["{ref}"]) }}'
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
            f'({"!" if ctx.NOT() else ''}h, vec!["{ctx.ITEM()}"])',
        ]
        return f'{{ {"; ".join(lines)} }}'

    def visitOneArgument(self, ctx):
        ref = str(ctx.REF())
        getter = self._getRefGetter(ref[1:])
        exp = self._getRefExplainer(ref)
        if exp is None:
            lines = [
                f'let h = ctx.has({getter})',
                # Calling this function multiple times with different args;
                # we could generate a string and leak it for the key, or we could just append all
                # the uses to the explainer.
                (f'if let Some(v) = edict.get_mut(&"{self.name}.{ref}") {{ '
                 f'v.push_str(format!(" {getter}: {{}}", h)); }} '
                 f'else {{ edict.insert("{self.name}.{ref}", format!("{getter}: {{}}", h)); }}'),
                f'(h, vec!["{self.name}.{ref}"])',
            ]
        else:
            lines = [
                f'let h = {getter}',
                exp,
                f'(h, vec!["{ref}"])',
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
            if exp := self._getRefExplainer(ref):
                lines = [
                    f'let r = {self._getRefGetter(ref[1:])}',
                    exp,
                    f'(r, vec!["{ref}"])'
                ]
                return f'{{ {"; ".join(lines)} }}'
            return f'({self._getRefGetter(str(ctx.REF())[1:])}, vec![])'
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

