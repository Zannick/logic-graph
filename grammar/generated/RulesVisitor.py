# Generated from Rules.g4 by ANTLR 4.12.0
from antlr4 import *
if __name__ is not None and "." in __name__:
    from .RulesParser import RulesParser
else:
    from RulesParser import RulesParser

# This class defines a complete generic visitor for a parse tree produced by RulesParser.

class RulesVisitor(ParseTreeVisitor):

    # Visit a parse tree produced by RulesParser#boolExpr.
    def visitBoolExpr(self, ctx:RulesParser.BoolExprContext):
        return self.visitChildren(ctx)


    # Visit a parse tree produced by RulesParser#actions.
    def visitActions(self, ctx:RulesParser.ActionsContext):
        return self.visitChildren(ctx)


    # Visit a parse tree produced by RulesParser#Set.
    def visitSet(self, ctx:RulesParser.SetContext):
        return self.visitChildren(ctx)


    # Visit a parse tree produced by RulesParser#Alter.
    def visitAlter(self, ctx:RulesParser.AlterContext):
        return self.visitChildren(ctx)


    # Visit a parse tree produced by RulesParser#meta.
    def visitMeta(self, ctx:RulesParser.MetaContext):
        return self.visitChildren(ctx)


    # Visit a parse tree produced by RulesParser#invoke.
    def visitInvoke(self, ctx:RulesParser.InvokeContext):
        return self.visitChildren(ctx)


    # Visit a parse tree produced by RulesParser#IfThenElse.
    def visitIfThenElse(self, ctx:RulesParser.IfThenElseContext):
        return self.visitChildren(ctx)


    # Visit a parse tree produced by RulesParser#PyTernary.
    def visitPyTernary(self, ctx:RulesParser.PyTernaryContext):
        return self.visitChildren(ctx)


    # Visit a parse tree produced by RulesParser#condNum.
    def visitCondNum(self, ctx:RulesParser.CondNumContext):
        return self.visitChildren(ctx)


    # Visit a parse tree produced by RulesParser#condStr.
    def visitCondStr(self, ctx:RulesParser.CondStrContext):
        return self.visitChildren(ctx)


    # Visit a parse tree produced by RulesParser#PerItemBool.
    def visitPerItemBool(self, ctx:RulesParser.PerItemBoolContext):
        return self.visitChildren(ctx)


    # Visit a parse tree produced by RulesParser#PerSettingBool.
    def visitPerSettingBool(self, ctx:RulesParser.PerSettingBoolContext):
        return self.visitChildren(ctx)


    # Visit a parse tree produced by RulesParser#MatchRefBool.
    def visitMatchRefBool(self, ctx:RulesParser.MatchRefBoolContext):
        return self.visitChildren(ctx)


    # Visit a parse tree produced by RulesParser#RefInList.
    def visitRefInList(self, ctx:RulesParser.RefInListContext):
        return self.visitChildren(ctx)


    # Visit a parse tree produced by RulesParser#PerItemInt.
    def visitPerItemInt(self, ctx:RulesParser.PerItemIntContext):
        return self.visitChildren(ctx)


    # Visit a parse tree produced by RulesParser#PerRefInt.
    def visitPerRefInt(self, ctx:RulesParser.PerRefIntContext):
        return self.visitChildren(ctx)


    # Visit a parse tree produced by RulesParser#PerSettingInt.
    def visitPerSettingInt(self, ctx:RulesParser.PerSettingIntContext):
        return self.visitChildren(ctx)


    # Visit a parse tree produced by RulesParser#PerItemStr.
    def visitPerItemStr(self, ctx:RulesParser.PerItemStrContext):
        return self.visitChildren(ctx)


    # Visit a parse tree produced by RulesParser#PerRefStr.
    def visitPerRefStr(self, ctx:RulesParser.PerRefStrContext):
        return self.visitChildren(ctx)


    # Visit a parse tree produced by RulesParser#PerSettingStr.
    def visitPerSettingStr(self, ctx:RulesParser.PerSettingStrContext):
        return self.visitChildren(ctx)


    # Visit a parse tree produced by RulesParser#cmp.
    def visitCmp(self, ctx:RulesParser.CmpContext):
        return self.visitChildren(ctx)


    # Visit a parse tree produced by RulesParser#cmpStr.
    def visitCmpStr(self, ctx:RulesParser.CmpStrContext):
        return self.visitChildren(ctx)


    # Visit a parse tree produced by RulesParser#flagMatch.
    def visitFlagMatch(self, ctx:RulesParser.FlagMatchContext):
        return self.visitChildren(ctx)


    # Visit a parse tree produced by RulesParser#refEq.
    def visitRefEq(self, ctx:RulesParser.RefEqContext):
        return self.visitChildren(ctx)


    # Visit a parse tree produced by RulesParser#funcNum.
    def visitFuncNum(self, ctx:RulesParser.FuncNumContext):
        return self.visitChildren(ctx)


    # Visit a parse tree produced by RulesParser#mathNum.
    def visitMathNum(self, ctx:RulesParser.MathNumContext):
        return self.visitChildren(ctx)


    # Visit a parse tree produced by RulesParser#num.
    def visitNum(self, ctx:RulesParser.NumContext):
        return self.visitChildren(ctx)


    # Visit a parse tree produced by RulesParser#baseNum.
    def visitBaseNum(self, ctx:RulesParser.BaseNumContext):
        return self.visitChildren(ctx)


    # Visit a parse tree produced by RulesParser#Setting.
    def visitSetting(self, ctx:RulesParser.SettingContext):
        return self.visitChildren(ctx)


    # Visit a parse tree produced by RulesParser#Argument.
    def visitArgument(self, ctx:RulesParser.ArgumentContext):
        return self.visitChildren(ctx)


    # Visit a parse tree produced by RulesParser#ItemCount.
    def visitItemCount(self, ctx:RulesParser.ItemCountContext):
        return self.visitChildren(ctx)


    # Visit a parse tree produced by RulesParser#OneItem.
    def visitOneItem(self, ctx:RulesParser.OneItemContext):
        return self.visitChildren(ctx)


    # Visit a parse tree produced by RulesParser#OneLitItem.
    def visitOneLitItem(self, ctx:RulesParser.OneLitItemContext):
        return self.visitChildren(ctx)


    # Visit a parse tree produced by RulesParser#OneArgument.
    def visitOneArgument(self, ctx:RulesParser.OneArgumentContext):
        return self.visitChildren(ctx)


    # Visit a parse tree produced by RulesParser#str.
    def visitStr(self, ctx:RulesParser.StrContext):
        return self.visitChildren(ctx)



del RulesParser