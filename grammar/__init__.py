
from .build import maybe_generate
maybe_generate(False)

from .generated.RulesLexer import RulesLexer
from .generated.RulesParser import RulesParser
from .generated.RulesVisitor import RulesVisitor
from antlr4 import InputStream, CommonTokenStream
from antlr4.error.ErrorListener import ErrorListener


class CollectErrorListener(ErrorListener):
    def __init__(self, name, verbose):
        self.errors = []
        self.name = name
        self.verbose = verbose

    def syntaxError(self, recog, offendingSymbol, line, col, msg, e):
        err = f'{name}: at {line}:{col}: {msg}'
        if self.verbose:
            stack = recog.getRuleInvocationStack()
            err += f' ({offendingSymbol} in rule stack {stack.reverse()})'
        self.errors.append(err)


def make_parser(text):
    ts = InputStream(text)
    lexer = RulesLexer(ts)
    stream = CommonTokenStream(lexer)
    return RulesParser(stream)


def parseBoolExpr(text, name='', verbose=False):
    p = make_parser(text)
    tree = p.boolExpr()
    errl = CollectErrorListener(name, verbose)
    return tree, p, errl.errors


def parseAction(text, name='', verbose=False):
    p = make_parser(text)
    tree = p.Action()
    errl = CollectErrorListener(name, verbose)
    return tree, p, errl.errors


__all__ = ['RulesParser', 'RulesVisitor', 'parseBoolExpr', 'parseAction', 'make_parser']
