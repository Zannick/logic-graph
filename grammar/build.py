import argparse
import os
import subprocess
import sys

grammar_dir = os.path.dirname(os.path.realpath(__file__))

def generate():
    antlr = ['antlr4', '-o', 'generated', '-Dlanguage=Python3', '-visitor', 'Rules.g4']
    print(' '.join(antlr))
    subprocess.run(antlr, cwd=grammar_dir, check=True)


def maybe_generate(recompile):
    if not os.path.exists(os.path.join(grammar_dir, 'generated/RulesVisitor.py')) or recompile:
        generate()


if __name__ == '__main__':
    argparser = argparse.ArgumentParser()
    argparser.add_argument('--recompile', action='store_true', help='Force a recompile of the grammar')

    args = argparser.parse_args()
    maybe_generate(args.recompile)

    
