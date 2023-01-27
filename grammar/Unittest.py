import unittest

from build import maybe_generate
maybe_generate(False)

from generated.RulesLexer import RulesLexer
from generated.RulesParser import RulesParser
from generated.RulesVisitor import RulesVisitor
from antlr4 import InputStream, CommonTokenStream


class StringVisitor(RulesVisitor):

    def visitAccessRule(self, ctx):
        if ctx.OR():
            return f'OR[ {self.visit(ctx.accessRule(0))} , {self.visit(ctx.accessRule(1))} ]'
        elif ctx.AND():
            return f'AND[ {self.visit(ctx.accessRule(0))} , {self.visit(ctx.accessRule(1))} ]'
        else:
            return super().visitAccessRule(ctx)

    def visitLitNeq(self, ctx):
        return f'{self.visit(ctx.value())} != {ctx.LIT()}'

    def visitSetting(self, ctx):
        return f'Setting:{ctx.SETTING()}'

    def visitOneItem(self, ctx):
        return f'Item:{ctx.ITEM()}'


def parse(text):
    ts = InputStream(text)
    lexer = RulesLexer(ts)
    stream = CommonTokenStream(lexer)
    parser = RulesParser(stream)
    tree = parser.accessRule()
    return tree, parser


text = "deadly_bonks != 'ohko' or Fairy"
t, p = parse(text)

class TestGrammar(unittest.TestCase):

    def test_Meta(self):
        text = '$here($can_use(Progressive_Hookshot)) or $is_child'
        t, p = parse(text)
        exp = '(accessRule (accessRule (meta $here ( (accessRule (invoke $can_use ( Progressive_Hookshot ))) ))) or (accessRule (invoke $is_child)))'
        self.assertEqual(exp, t.toStringTree(recog=p))

    def test_Setting(self):
        text = "deadly_bonks != 'ohko' or Fairy"
        t, p = parse(text)
        # toStringTree can only get rule names
        exp = "(accessRule (accessRule (cmp (value deadly_bonks) != 'ohko')) or (accessRule (item Fairy)))"
        self.assertEqual(exp, t.toStringTree(recog=p))

    def test_SettingVisit(self):
        text = "deadly_bonks != 'ohko' or Fairy"
        t, p = parse(text)
        exp = "OR[ Setting:deadly_bonks != 'ohko' , Item:Fairy ]"
        self.assertEqual(exp, StringVisitor().visit(t))
