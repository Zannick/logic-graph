# Generated from Rules.g4 by ANTLR 4.13.1
from antlr4 import *
if "." in __name__:
    from .RulesParser import RulesParser
else:
    from RulesParser import RulesParser

# This class defines a complete listener for a parse tree produced by RulesParser.
class RulesListener(ParseTreeListener):

    # Enter a parse tree produced by RulesParser#boolExpr.
    def enterBoolExpr(self, ctx:RulesParser.BoolExprContext):
        pass

    # Exit a parse tree produced by RulesParser#boolExpr.
    def exitBoolExpr(self, ctx:RulesParser.BoolExprContext):
        pass


    # Enter a parse tree produced by RulesParser#actions.
    def enterActions(self, ctx:RulesParser.ActionsContext):
        pass

    # Exit a parse tree produced by RulesParser#actions.
    def exitActions(self, ctx:RulesParser.ActionsContext):
        pass


    # Enter a parse tree produced by RulesParser#Set.
    def enterSet(self, ctx:RulesParser.SetContext):
        pass

    # Exit a parse tree produced by RulesParser#Set.
    def exitSet(self, ctx:RulesParser.SetContext):
        pass


    # Enter a parse tree produced by RulesParser#Alter.
    def enterAlter(self, ctx:RulesParser.AlterContext):
        pass

    # Exit a parse tree produced by RulesParser#Alter.
    def exitAlter(self, ctx:RulesParser.AlterContext):
        pass


    # Enter a parse tree produced by RulesParser#ActionHelper.
    def enterActionHelper(self, ctx:RulesParser.ActionHelperContext):
        pass

    # Exit a parse tree produced by RulesParser#ActionHelper.
    def exitActionHelper(self, ctx:RulesParser.ActionHelperContext):
        pass


    # Enter a parse tree produced by RulesParser#CondAction.
    def enterCondAction(self, ctx:RulesParser.CondActionContext):
        pass

    # Exit a parse tree produced by RulesParser#CondAction.
    def exitCondAction(self, ctx:RulesParser.CondActionContext):
        pass


    # Enter a parse tree produced by RulesParser#Swap.
    def enterSwap(self, ctx:RulesParser.SwapContext):
        pass

    # Exit a parse tree produced by RulesParser#Swap.
    def exitSwap(self, ctx:RulesParser.SwapContext):
        pass


    # Enter a parse tree produced by RulesParser#meta.
    def enterMeta(self, ctx:RulesParser.MetaContext):
        pass

    # Exit a parse tree produced by RulesParser#meta.
    def exitMeta(self, ctx:RulesParser.MetaContext):
        pass


    # Enter a parse tree produced by RulesParser#invoke.
    def enterInvoke(self, ctx:RulesParser.InvokeContext):
        pass

    # Exit a parse tree produced by RulesParser#invoke.
    def exitInvoke(self, ctx:RulesParser.InvokeContext):
        pass


    # Enter a parse tree produced by RulesParser#IfThenElse.
    def enterIfThenElse(self, ctx:RulesParser.IfThenElseContext):
        pass

    # Exit a parse tree produced by RulesParser#IfThenElse.
    def exitIfThenElse(self, ctx:RulesParser.IfThenElseContext):
        pass


    # Enter a parse tree produced by RulesParser#PyTernary.
    def enterPyTernary(self, ctx:RulesParser.PyTernaryContext):
        pass

    # Exit a parse tree produced by RulesParser#PyTernary.
    def exitPyTernary(self, ctx:RulesParser.PyTernaryContext):
        pass


    # Enter a parse tree produced by RulesParser#condNum.
    def enterCondNum(self, ctx:RulesParser.CondNumContext):
        pass

    # Exit a parse tree produced by RulesParser#condNum.
    def exitCondNum(self, ctx:RulesParser.CondNumContext):
        pass


    # Enter a parse tree produced by RulesParser#condStr.
    def enterCondStr(self, ctx:RulesParser.CondStrContext):
        pass

    # Exit a parse tree produced by RulesParser#condStr.
    def exitCondStr(self, ctx:RulesParser.CondStrContext):
        pass


    # Enter a parse tree produced by RulesParser#PerItemBool.
    def enterPerItemBool(self, ctx:RulesParser.PerItemBoolContext):
        pass

    # Exit a parse tree produced by RulesParser#PerItemBool.
    def exitPerItemBool(self, ctx:RulesParser.PerItemBoolContext):
        pass


    # Enter a parse tree produced by RulesParser#PerSettingBool.
    def enterPerSettingBool(self, ctx:RulesParser.PerSettingBoolContext):
        pass

    # Exit a parse tree produced by RulesParser#PerSettingBool.
    def exitPerSettingBool(self, ctx:RulesParser.PerSettingBoolContext):
        pass


    # Enter a parse tree produced by RulesParser#MatchRefBool.
    def enterMatchRefBool(self, ctx:RulesParser.MatchRefBoolContext):
        pass

    # Exit a parse tree produced by RulesParser#MatchRefBool.
    def exitMatchRefBool(self, ctx:RulesParser.MatchRefBoolContext):
        pass


    # Enter a parse tree produced by RulesParser#RefInList.
    def enterRefInList(self, ctx:RulesParser.RefInListContext):
        pass

    # Exit a parse tree produced by RulesParser#RefInList.
    def exitRefInList(self, ctx:RulesParser.RefInListContext):
        pass


    # Enter a parse tree produced by RulesParser#RefStrInList.
    def enterRefStrInList(self, ctx:RulesParser.RefStrInListContext):
        pass

    # Exit a parse tree produced by RulesParser#RefStrInList.
    def exitRefStrInList(self, ctx:RulesParser.RefStrInListContext):
        pass


    # Enter a parse tree produced by RulesParser#PerItemInt.
    def enterPerItemInt(self, ctx:RulesParser.PerItemIntContext):
        pass

    # Exit a parse tree produced by RulesParser#PerItemInt.
    def exitPerItemInt(self, ctx:RulesParser.PerItemIntContext):
        pass


    # Enter a parse tree produced by RulesParser#PerRefInt.
    def enterPerRefInt(self, ctx:RulesParser.PerRefIntContext):
        pass

    # Exit a parse tree produced by RulesParser#PerRefInt.
    def exitPerRefInt(self, ctx:RulesParser.PerRefIntContext):
        pass


    # Enter a parse tree produced by RulesParser#PerSettingInt.
    def enterPerSettingInt(self, ctx:RulesParser.PerSettingIntContext):
        pass

    # Exit a parse tree produced by RulesParser#PerSettingInt.
    def exitPerSettingInt(self, ctx:RulesParser.PerSettingIntContext):
        pass


    # Enter a parse tree produced by RulesParser#PerItemStr.
    def enterPerItemStr(self, ctx:RulesParser.PerItemStrContext):
        pass

    # Exit a parse tree produced by RulesParser#PerItemStr.
    def exitPerItemStr(self, ctx:RulesParser.PerItemStrContext):
        pass


    # Enter a parse tree produced by RulesParser#PerRefStr.
    def enterPerRefStr(self, ctx:RulesParser.PerRefStrContext):
        pass

    # Exit a parse tree produced by RulesParser#PerRefStr.
    def exitPerRefStr(self, ctx:RulesParser.PerRefStrContext):
        pass


    # Enter a parse tree produced by RulesParser#PerSettingStr.
    def enterPerSettingStr(self, ctx:RulesParser.PerSettingStrContext):
        pass

    # Exit a parse tree produced by RulesParser#PerSettingStr.
    def exitPerSettingStr(self, ctx:RulesParser.PerSettingStrContext):
        pass


    # Enter a parse tree produced by RulesParser#cmp.
    def enterCmp(self, ctx:RulesParser.CmpContext):
        pass

    # Exit a parse tree produced by RulesParser#cmp.
    def exitCmp(self, ctx:RulesParser.CmpContext):
        pass


    # Enter a parse tree produced by RulesParser#cmpStr.
    def enterCmpStr(self, ctx:RulesParser.CmpStrContext):
        pass

    # Exit a parse tree produced by RulesParser#cmpStr.
    def exitCmpStr(self, ctx:RulesParser.CmpStrContext):
        pass


    # Enter a parse tree produced by RulesParser#flagMatch.
    def enterFlagMatch(self, ctx:RulesParser.FlagMatchContext):
        pass

    # Exit a parse tree produced by RulesParser#flagMatch.
    def exitFlagMatch(self, ctx:RulesParser.FlagMatchContext):
        pass


    # Enter a parse tree produced by RulesParser#RefEqSimple.
    def enterRefEqSimple(self, ctx:RulesParser.RefEqSimpleContext):
        pass

    # Exit a parse tree produced by RulesParser#RefEqSimple.
    def exitRefEqSimple(self, ctx:RulesParser.RefEqSimpleContext):
        pass


    # Enter a parse tree produced by RulesParser#RefEqRef.
    def enterRefEqRef(self, ctx:RulesParser.RefEqRefContext):
        pass

    # Exit a parse tree produced by RulesParser#RefEqRef.
    def exitRefEqRef(self, ctx:RulesParser.RefEqRefContext):
        pass


    # Enter a parse tree produced by RulesParser#RefEqInvoke.
    def enterRefEqInvoke(self, ctx:RulesParser.RefEqInvokeContext):
        pass

    # Exit a parse tree produced by RulesParser#RefEqInvoke.
    def exitRefEqInvoke(self, ctx:RulesParser.RefEqInvokeContext):
        pass


    # Enter a parse tree produced by RulesParser#funcNum.
    def enterFuncNum(self, ctx:RulesParser.FuncNumContext):
        pass

    # Exit a parse tree produced by RulesParser#funcNum.
    def exitFuncNum(self, ctx:RulesParser.FuncNumContext):
        pass


    # Enter a parse tree produced by RulesParser#mathNum.
    def enterMathNum(self, ctx:RulesParser.MathNumContext):
        pass

    # Exit a parse tree produced by RulesParser#mathNum.
    def exitMathNum(self, ctx:RulesParser.MathNumContext):
        pass


    # Enter a parse tree produced by RulesParser#num.
    def enterNum(self, ctx:RulesParser.NumContext):
        pass

    # Exit a parse tree produced by RulesParser#num.
    def exitNum(self, ctx:RulesParser.NumContext):
        pass


    # Enter a parse tree produced by RulesParser#baseNum.
    def enterBaseNum(self, ctx:RulesParser.BaseNumContext):
        pass

    # Exit a parse tree produced by RulesParser#baseNum.
    def exitBaseNum(self, ctx:RulesParser.BaseNumContext):
        pass


    # Enter a parse tree produced by RulesParser#Setting.
    def enterSetting(self, ctx:RulesParser.SettingContext):
        pass

    # Exit a parse tree produced by RulesParser#Setting.
    def exitSetting(self, ctx:RulesParser.SettingContext):
        pass


    # Enter a parse tree produced by RulesParser#Argument.
    def enterArgument(self, ctx:RulesParser.ArgumentContext):
        pass

    # Exit a parse tree produced by RulesParser#Argument.
    def exitArgument(self, ctx:RulesParser.ArgumentContext):
        pass


    # Enter a parse tree produced by RulesParser#itemList.
    def enterItemList(self, ctx:RulesParser.ItemListContext):
        pass

    # Exit a parse tree produced by RulesParser#itemList.
    def exitItemList(self, ctx:RulesParser.ItemListContext):
        pass


    # Enter a parse tree produced by RulesParser#ItemCount.
    def enterItemCount(self, ctx:RulesParser.ItemCountContext):
        pass

    # Exit a parse tree produced by RulesParser#ItemCount.
    def exitItemCount(self, ctx:RulesParser.ItemCountContext):
        pass


    # Enter a parse tree produced by RulesParser#OneItem.
    def enterOneItem(self, ctx:RulesParser.OneItemContext):
        pass

    # Exit a parse tree produced by RulesParser#OneItem.
    def exitOneItem(self, ctx:RulesParser.OneItemContext):
        pass


    # Enter a parse tree produced by RulesParser#OneLitItem.
    def enterOneLitItem(self, ctx:RulesParser.OneLitItemContext):
        pass

    # Exit a parse tree produced by RulesParser#OneLitItem.
    def exitOneLitItem(self, ctx:RulesParser.OneLitItemContext):
        pass


    # Enter a parse tree produced by RulesParser#OneArgument.
    def enterOneArgument(self, ctx:RulesParser.OneArgumentContext):
        pass

    # Exit a parse tree produced by RulesParser#OneArgument.
    def exitOneArgument(self, ctx:RulesParser.OneArgumentContext):
        pass


    # Enter a parse tree produced by RulesParser#str.
    def enterStr(self, ctx:RulesParser.StrContext):
        pass

    # Exit a parse tree produced by RulesParser#str.
    def exitStr(self, ctx:RulesParser.StrContext):
        pass


    # Enter a parse tree produced by RulesParser#somewhere.
    def enterSomewhere(self, ctx:RulesParser.SomewhereContext):
        pass

    # Exit a parse tree produced by RulesParser#somewhere.
    def exitSomewhere(self, ctx:RulesParser.SomewhereContext):
        pass


    # Enter a parse tree produced by RulesParser#RefInPlaceRef.
    def enterRefInPlaceRef(self, ctx:RulesParser.RefInPlaceRefContext):
        pass

    # Exit a parse tree produced by RulesParser#RefInPlaceRef.
    def exitRefInPlaceRef(self, ctx:RulesParser.RefInPlaceRefContext):
        pass


    # Enter a parse tree produced by RulesParser#RefInPlaceName.
    def enterRefInPlaceName(self, ctx:RulesParser.RefInPlaceNameContext):
        pass

    # Exit a parse tree produced by RulesParser#RefInPlaceName.
    def exitRefInPlaceName(self, ctx:RulesParser.RefInPlaceNameContext):
        pass


    # Enter a parse tree produced by RulesParser#RefInPlaceList.
    def enterRefInPlaceList(self, ctx:RulesParser.RefInPlaceListContext):
        pass

    # Exit a parse tree produced by RulesParser#RefInPlaceList.
    def exitRefInPlaceList(self, ctx:RulesParser.RefInPlaceListContext):
        pass


    # Enter a parse tree produced by RulesParser#RefInFunc.
    def enterRefInFunc(self, ctx:RulesParser.RefInFuncContext):
        pass

    # Exit a parse tree produced by RulesParser#RefInFunc.
    def exitRefInFunc(self, ctx:RulesParser.RefInFuncContext):
        pass


    # Enter a parse tree produced by RulesParser#ref.
    def enterRef(self, ctx:RulesParser.RefContext):
        pass

    # Exit a parse tree produced by RulesParser#ref.
    def exitRef(self, ctx:RulesParser.RefContext):
        pass



del RulesParser