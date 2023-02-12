import io
import itertools
import json
import os
import re
import unittest

from . import *
from .visitors import StringVisitor


class TestGrammar(unittest.TestCase):

    def testMeta(self):
        text = '$here($can_use(Progressive_Hookshot)) or $is_child'
        _, t, p, e = parseBoolExpr(text)
        exp = '(boolExpr (boolExpr (meta $here ( (boolExpr (invoke $can_use ( Progressive_Hookshot ))) ))) or (boolExpr (invoke $is_child)))'
        self.assertEqual(exp, t.toStringTree(recog=p))

    def testSetting(self):
        text = "deadly_bonks != 'ohko' or Fairy"
        _, t, p, e = parseBoolExpr(text)
        # toStringTree can only get rule names
        exp = "(boolExpr (boolExpr (cmp (value deadly_bonks) != 'ohko')) or (boolExpr (item Fairy)))"
        self.assertEqual(exp, t.toStringTree(recog=p))

    def testMetaFuncVisit(self):
        text = '$here($can_use(Progressive_Hookshot)) or $is_child'
        _, t, p, e = parseBoolExpr(text)
        exp = 'OR[ Meta:here( Func:can_use(Progressive_Hookshot) ) , Func:is_child() ]'
        self.assertEqual(exp, StringVisitor().visit(t))

    def testSettingVisit(self):
        text = "deadly_bonks != 'ohko' or Fairy"
        _, t, p, e = parseBoolExpr(text)
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
            r = re.sub(r'\b(' + '|'.join(args) + r')\b', r'^\1', rule)
            if r not in rules:
                rules[r] = func

        fn = re.compile(r'\b(' + '|'.join(funcs) + r')\b')
        ba = re.compile(r'\b(' + '|'.join(baseargs) + r')\b')

        # process args for all other rules
        folders = ['World', 'Glitched World']
        files = [os.path.join(self.ootr_dir, folder, f) for folder in folders
                 for f in os.listdir(os.path.join(self.ootr_dir, folder))
                 if f.endswith('.json')]
        regions = itertools.chain.from_iterable(read_ootr_logic_file(f) for f in files)
        for reg in regions:
            for loc, r in itertools.chain(reg.get("locations", {}).items(),
                                          reg.get("events", {}).items()):
                r = ba.sub(r'^\1', r)
                if r not in rules:
                    rules[r] = loc
            for ex, r in reg.get("exits", {}).items():
                r = ba.sub(r'^\1', r)
                if r not in rules:
                    rules[r] = f'{reg["region_name"]} -> {ex}'

        for rule, name in rules.items():
            rule = fn.sub(r'$\1', rule)
            with self.subTest(name=name + '; ' + rule):
                _, t, p, e = parseBoolExpr(rule)
                self.assertIsNotNone(t, f'No parse tree from rule: {rule}')
                st = StringVisitor().visit(t)
                self.assertIsNotNone(st, f'No visit string returned for rule: {rule}\n  -> {t.toStringTree(recog=p)}')
                self.assertIsNone(self.raw.search(st), f'{rule}\n  ->  {st}')
