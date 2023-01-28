import io
import itertools
import json
import os
import re
import unittest

from build import maybe_generate
maybe_generate(False)

from generated.RulesLexer import RulesLexer
from generated.RulesParser import RulesParser
from generated.RulesVisitor import RulesVisitor
from antlr4 import InputStream, CommonTokenStream


class StringVisitor(RulesVisitor):

    def visitAccessRule(self, ctx):
        try:
            if ctx.OR():
                return f'OR[ {self.visit(ctx.accessRule(0))} , {self.visit(ctx.accessRule(1))} ]'
            elif ctx.AND():
                return f'AND[ {self.visit(ctx.accessRule(0))} , {self.visit(ctx.accessRule(1))} ]'
            elif ctx.TRUE():
                return 'TRUE'
            elif ctx.FALSE():
                return 'FALSE'
            elif ctx.accessRule():
                return self.visit(ctx.accessRule(0))
            else:
                return super().visitAccessRule(ctx)
        except AttributeError as e:
            raise AttributeError(str(e) + '; ' + ' '.join(
                f'[{c.toStringTree(ruleNames = RulesParser.ruleNames)}]'
                for c in ctx.accessRule()))

    def visitMeta(self, ctx):
        lit = ctx.LIT()
        func = str(ctx.FUNC())[1:]
        return f'Meta:{func}( {str(lit) + " , " if lit else ""}{self.visit(ctx.accessRule())} )'

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

    def _visitCond(self, cond, then, el=None):
        p1 = f'IF( {self.visit(cond)} ) THEN{{ {self.visit(then)} }}'
        if el is None:
            return p1
        return p1 + f' ELSE{{ {self.visit(el)} }}'

    def visitIfThenElse(self, ctx):
        return self._visitConditional(*ctx.accessRule())

    def visitPyTernary(self, ctx):
        return self._visitConditional(ctx.accessRule(1), ctx.accessRule(0), ctx.accessRule(2))

    def visitCmp(self, ctx):
        return f'{self.visit(ctx.value())} {ctx.getChild(1)} {ctx.LIT() or self.visit(ctx.num())}'

    def visitFlagMatch(self, ctx):
        num = f'{self.visit(ctx.num())}'
        return f'({self.visit(ctx.value())} & {num}) == {num}'

    def visitRefEq(self, ctx):
        if ctx.ITEM():
            return f'Arg:{str(ctx.REF())[1:]} == Item:{ctx.ITEM()}'
        return f'Arg:{str(ctx.REF())[1:]} == Setting:{ctx.SETTING()}'

    def visitSetting(self, ctx):
        s = f'Setting:{ctx.SETTING()}'
        if ctx.LIT():
            s += f'[{ctx.LIT()}]'
        if ctx.NOT():
            return f'NOT[ {s} ]'
        return s

    def visitArgument(self, ctx):
        return f'Arg:{str(ctx.REF())[1:]}'

    def visitItemCount(self, ctx):
        if ctx.INT():
            return f'Items:{ctx.ITEM()}:{ctx.INT()}'
        return f'Items:{ctx.ITEM()}:{{Setting:{ctx.SETTING}}}'

    def visitOneItem(self, ctx):
        return f'Item:{ctx.ITEM()}'

    def visitOneLitItem(self, ctx):
        return f'Item:{str(ctx.LIT())[1:-1].replace(" ", "_")}'

    def visitOneArgument(self, ctx):
        return f'Arg:{str(ctx.REF())[1:]}'

    def visitNum(self, ctx):
        if ctx.INT():
            return str(ctx.INT())
        return f'Const:{ctx.CONST()}'


def parse(text):
    ts = InputStream(text)
    lexer = RulesLexer(ts)
    stream = CommonTokenStream(lexer)
    parser = RulesParser(stream)
    tree = parser.accessRule()
    return tree, parser


text = "(deadly_bonks != 'ohko' or Fairy)"
t, p = parse(text)

class TestGrammar(unittest.TestCase):

    def testMeta(self):
        text = '$here($can_use(Progressive_Hookshot)) or $is_child'
        t, p = parse(text)
        exp = '(accessRule (accessRule (meta $here ( (accessRule (invoke $can_use ( Progressive_Hookshot ))) ))) or (accessRule (invoke $is_child)))'
        self.assertEqual(exp, t.toStringTree(recog=p))

    def testSetting(self):
        text = "deadly_bonks != 'ohko' or Fairy"
        t, p = parse(text)
        # toStringTree can only get rule names
        exp = "(accessRule (accessRule (cmp (value deadly_bonks) != 'ohko')) or (accessRule (item Fairy)))"
        self.assertEqual(exp, t.toStringTree(recog=p))

    def testMetaFuncVisit(self):
        text = '$here($can_use(Progressive_Hookshot)) or $is_child'
        t, p = parse(text)
        exp = 'OR[ Meta:here( Func:can_use(Progressive_Hookshot) ) , Func:is_child() ]'
        self.assertEqual(exp, StringVisitor().visit(t))

    def testSettingVisit(self):
        text = "deadly_bonks != 'ohko' or Fairy"
        t, p = parse(text)
        exp = "OR[ Setting:deadly_bonks != 'ohko' , Item:Fairy ]"
        self.assertEqual(exp, StringVisitor().visit(t))


def read_ootr_logic_file(file_path):
    json_string = ""
    with io.open(file_path, 'r') as file:
        for line in file.readlines():
            json_string += line.split('#')[0].replace('\n', ' ')
    json_string = re.sub(' +', ' ', json_string)
    try:
        return json.loads(json_string)
    except json.JSONDecodeError as error:
        raise Exception("JSON parse error around text:\n" + \
                        json_string[error.pos-35:error.pos+35] + "\n" + \
                        "                                   ^^\n")

class TestOoTR(unittest.TestCase):
    ootr_dir = os.path.expanduser('~/OoT-Randomizer/data')
    raw = re.compile(r'\[[0-9 ]*\]')

    def testAll(self):
        helpers = read_ootr_logic_file(os.path.join(self.ootr_dir, 'LogicHelpers.json'))
        funcs = ['here', 'at', 'at_day', 'at_dampe_time', 'at_night',
                 'has_bottle', 'has_hearts', 'heart_count', 'has_medallions',
                 'has_stones', 'has_dungeon_rewards', 'has_item_goal', 'has_full_item_goal',
                 'has_all_item_goals', 'had_night_start', 'can_live_dmg', 'guarantee_hint',
                 'region_has_shortcuts', 'count_of', 'item_count']
        baseargs = ['age', 'tod', 'spot']

        rules = {}
        # process args for helper functions and add their names to funcs
        for s, rule in helpers.items():
            if s[0].isupper():
                continue
            if '(' in s:
                func, args = s[:-1].split('(', 1)
                args = [a.strip() for a in args.split(',')] + baseargs
            else:
                func = s
                args = baseargs
            funcs.append(func)
            r = re.sub(r'\b(' + '|'.join(args) + r')\b', r'@\1', rule)
            if r not in rules:
                rules[r] = func

        fn = re.compile(r'\b(' + '|'.join(funcs) + r')\b')
        ba = re.compile(r'\b(' + '|'.join(baseargs) + r')\b')

        # process args for all other rules
        folders = ['World', 'Glitched World']
        files = [os.path.join(self.ootr_dir, folder, f) for folder in folders
                 for f in os.listdir(os.path.join(self.ootr_dir, folder))]
        regions = itertools.chain.from_iterable(read_ootr_logic_file(f) for f in files)
        for reg in regions:
            for loc, r in itertools.chain(reg.get("locations", {}).items(),
                                          reg.get("events", {}).items()):
                r = ba.sub(r'@\1', r)
                if r not in rules:
                    rules[r] = loc
            for ex, r in reg.get("exits", {}).items():
                r = ba.sub(r'@\1', r)
                if r not in rules:
                    rules[r] = f'{reg["region_name"]} -> {ex}'

        for rule, name in rules.items():
            rule = fn.sub(r'$\1', rule)
            with self.subTest(name=name + '; ' + rule):
                t, p = parse(rule)
                self.assertIsNotNone(t, f'No parse tree from rule: {rule}')
                st = StringVisitor().visit(t)
                self.assertIsNotNone(st, f'No visit string returned for rule: {rule}\n  -> {t.toStringTree(recog=p)}')
                self.assertIsNone(self.raw.search(st), f'{rule}\n  ->  {st}')
