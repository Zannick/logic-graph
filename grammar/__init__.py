from collections import namedtuple

from .build import maybe_generate
maybe_generate(False)

from .generated.RulesLexer import RulesLexer
from .generated.RulesParser import RulesParser
from .generated.RulesVisitor import RulesVisitor
import antlr4
from antlr4 import InputStream, CommonTokenStream
from antlr4.error.ErrorListener import ErrorListener


class CollectErrorListener(ErrorListener):
    def __init__(self, name: str, verbose: bool):
        self.errors: list[str] = []
        self.name: str = name
        self.verbose: bool = verbose

    def syntaxError(self, recog, offendingSymbol, line, col, msg, e):
        err = f'{self.name}: at {line}:{col}: {msg}'
        if self.verbose:
            stack = recog.getRuleInvocationStack()
            err += f' ({offendingSymbol} in rule stack {stack.reverse()})'
        self.errors.append(err)


ParseResult = namedtuple('ParseResult', ['name', 'text', 'tree', 'parser', 'errors'])

def make_parser(text) -> antlr4.Parser:
    ts = InputStream(str(text))
    lexer = RulesLexer(ts)
    stream = CommonTokenStream(lexer)
    return RulesParser(stream)


def parseBoolExpr(text, name='', verbose=False) -> ParseResult:
    p = make_parser(text)
    errl = CollectErrorListener(name, verbose)
    p.removeErrorListeners()
    p.addErrorListener(errl)
    tree = p.boolExpr()
    return ParseResult(name, text, tree, p, errl.errors)


def parseNum(text, name='', verbose=False) -> ParseResult:
    p = make_parser(text)
    errl = CollectErrorListener(name, verbose)
    p.removeErrorListeners()
    p.addErrorListener(errl)
    tree = p.num()
    return ParseResult(name, text, tree, p, errl.errors)


def parseAction(text, name='', verbose=False) -> ParseResult:
    p = make_parser(text)
    errl = CollectErrorListener(name, verbose)
    p.removeErrorListeners()
    p.addErrorListener(errl)
    tree = p.actions()
    return ParseResult(name, text, tree, p, errl.errors)

def parseItemList(text, name='', verbose=False) -> ParseResult:
    p = make_parser(text)
    errl = CollectErrorListener(name, verbose)
    p.removeErrorListeners()
    p.addErrorListener(errl)
    tree = p.itemList()
    return ParseResult(name, text, tree, p, errl.errors)


def parseRule(rule: str, text: str, name:str='', verbose:bool=False) -> ParseResult:
    if rule == 'boolExpr':
        return parseBoolExpr(text, name=name, verbose=verbose)
    if rule == 'num':
        return parseNum(text, name=name, verbose=verbose)
    if rule == 'action':
        return parseAction(text, name=name, verbose=verbose)
    if rule == 'itemList':
        return parseItemList(text, name=name, verbose=verbose)
    raise Exception(f'Unrecognized parse rule {rule!r} on name {name!r}')


__all__ = ['RulesParser', 'RulesVisitor',
           'parseBoolExpr', 'parseNum', 'parseAction', 'parseRule',
           'make_parser', 'ParseResult']
