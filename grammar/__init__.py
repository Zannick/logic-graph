from collections import namedtuple

from .build import maybe_generate
maybe_generate(False)

from .generated.RulesLexer import RulesLexer
from .generated.RulesParser import RulesParser
from .generated.RulesVisitor import RulesVisitor
from .StringVisitor import StringVisitor
from antlr4 import InputStream, CommonTokenStream
from antlr4.error.ErrorListener import ErrorListener


class CollectErrorListener(ErrorListener):
    def __init__(self, name, verbose):
        self.errors = []
        self.name = name
        self.verbose = verbose

    def syntaxError(self, recog, offendingSymbol, line, col, msg, e):
        err = f'{self.name}: at {line}:{col}: {msg}'
        if self.verbose:
            stack = recog.getRuleInvocationStack()
            err += f' ({offendingSymbol} in rule stack {stack.reverse()})'
        self.errors.append(err)


ParseResult = namedtuple('ParseResult', ['text', 'tree', 'parser', 'errors'])

def make_parser(text):
    ts = InputStream(str(text))
    lexer = RulesLexer(ts)
    stream = CommonTokenStream(lexer)
    return RulesParser(stream)


def parseBoolExpr(text, name='', verbose=False):
    p = make_parser(text)
    errl = CollectErrorListener(name, verbose)
    p.removeErrorListeners()
    p.addErrorListener(errl)
    tree = p.boolExpr()
    return ParseResult(text, tree, p, errl.errors)


def parseNum(text, name='', verbose=False):
    p = make_parser(text)
    errl = CollectErrorListener(name, verbose)
    p.removeErrorListeners()
    p.addErrorListener(errl)
    tree = p.num()
    return ParseResult(text, tree, p, errl.errors)


def parseAction(text, name='', verbose=False):
    p = make_parser(text)
    errl = CollectErrorListener(name, verbose)
    p.removeErrorListeners()
    p.addErrorListener(errl)
    tree = p.action()
    return ParseResult(text, tree, p, errl.errors)


def parseRule(rule, text, name='', verbose=False):
    if rule == 'boolExpr':
        return parseBoolExpr(text, name=name, verbose=verbose)
    if rule == 'num':
        return parseNum(text, name=name, verbose=verbose)
    if rule == 'action':
        return parseAction(text, name=name, verbose=verbose)
    raise Exception(f'Unrecognized parse rule {rule!r} on name {name!r}')


__all__ = ['RulesParser', 'RulesVisitor',
           'parseBoolExpr', 'parseNum', 'parseAction', 'parseRule',
           'make_parser', 'ParseResult', 'StringVisitor']
