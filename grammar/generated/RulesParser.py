# Generated from Rules.g4 by ANTLR 4.13.1
# encoding: utf-8
from antlr4 import *
from io import StringIO
import sys
if sys.version_info[1] > 5:
	from typing import TextIO
else:
	from typing.io import TextIO

def serializedATN():
    return [
        4,1,40,772,2,0,7,0,2,1,7,1,2,2,7,2,2,3,7,3,2,4,7,4,2,5,7,5,2,6,7,
        6,2,7,7,7,2,8,7,8,2,9,7,9,2,10,7,10,2,11,7,11,2,12,7,12,2,13,7,13,
        2,14,7,14,2,15,7,15,2,16,7,16,2,17,7,17,2,18,7,18,2,19,7,19,2,20,
        7,20,2,21,7,21,2,22,7,22,2,23,7,23,2,24,7,24,1,0,1,0,1,0,1,0,1,0,
        1,0,1,0,1,0,1,0,1,0,1,0,1,0,1,0,1,0,1,0,1,0,3,0,67,8,0,1,0,1,0,1,
        0,1,0,1,0,3,0,74,8,0,1,0,1,0,1,0,1,0,1,0,1,0,5,0,82,8,0,10,0,12,
        0,85,9,0,1,1,1,1,1,1,5,1,90,8,1,10,1,12,1,93,9,1,1,1,3,1,96,8,1,
        1,2,1,2,1,2,1,2,1,2,1,2,1,2,1,2,3,2,106,8,2,1,2,1,2,1,2,1,2,1,2,
        1,2,1,2,1,2,1,2,1,2,1,2,1,2,1,2,1,2,1,2,1,2,1,2,1,2,1,2,1,2,1,2,
        5,2,129,8,2,10,2,12,2,132,9,2,1,2,1,2,1,2,1,2,1,2,3,2,139,8,2,3,
        2,141,8,2,1,3,1,3,1,3,1,3,1,3,1,3,1,3,1,3,1,3,1,3,1,3,1,3,3,3,155,
        8,3,1,4,3,4,158,8,4,1,4,1,4,1,4,1,4,1,4,5,4,165,8,4,10,4,12,4,168,
        9,4,1,4,1,4,3,4,172,8,4,1,4,1,4,1,4,1,4,1,4,1,4,3,4,180,8,4,1,4,
        1,4,1,4,1,4,1,4,3,4,187,8,4,1,4,1,4,1,4,1,4,1,4,3,4,194,8,4,1,4,
        1,4,1,4,1,4,1,4,3,4,201,8,4,1,4,1,4,1,4,1,4,1,4,5,4,208,8,4,10,4,
        12,4,211,9,4,1,4,1,4,3,4,215,8,4,1,4,1,4,1,4,1,4,1,4,3,4,222,8,4,
        1,4,1,4,1,4,3,4,227,8,4,3,4,229,8,4,1,5,1,5,1,5,1,5,1,5,1,5,1,5,
        1,5,1,5,1,5,1,5,1,5,1,5,1,5,1,5,1,5,5,5,247,8,5,10,5,12,5,250,9,
        5,1,5,1,5,1,5,1,5,1,5,3,5,257,8,5,1,5,1,5,1,5,1,5,1,5,1,5,1,5,1,
        5,3,5,267,8,5,1,6,1,6,1,6,1,6,1,6,1,6,1,6,1,6,1,6,1,6,1,6,1,6,1,
        6,1,6,1,6,1,6,5,6,285,8,6,10,6,12,6,288,9,6,1,6,1,6,1,6,1,6,1,6,
        3,6,295,8,6,1,7,1,7,1,7,1,7,1,7,1,7,1,7,1,7,1,7,1,7,1,7,1,7,1,7,
        1,7,1,7,1,7,5,7,313,8,7,10,7,12,7,316,9,7,1,7,1,7,1,7,1,7,1,7,3,
        7,323,8,7,1,8,1,8,1,8,1,8,1,8,1,8,1,8,1,8,4,8,333,8,8,11,8,12,8,
        334,1,8,1,8,1,8,1,8,3,8,341,8,8,1,8,1,8,1,8,1,8,1,8,1,8,1,8,1,8,
        1,8,1,8,4,8,353,8,8,11,8,12,8,354,1,8,1,8,1,8,1,8,1,8,4,8,362,8,
        8,11,8,12,8,363,3,8,366,8,8,1,8,1,8,1,8,1,8,3,8,372,8,8,1,8,1,8,
        1,8,1,8,1,8,1,8,1,8,1,8,5,8,382,8,8,10,8,12,8,385,9,8,1,8,1,8,1,
        8,1,8,4,8,391,8,8,11,8,12,8,392,1,8,1,8,1,8,1,8,3,8,399,8,8,1,8,
        1,8,1,8,1,8,1,8,1,8,1,8,1,8,4,8,409,8,8,11,8,12,8,410,1,8,1,8,1,
        8,1,8,1,8,1,8,1,8,4,8,420,8,8,11,8,12,8,421,1,8,3,8,425,8,8,1,9,
        1,9,1,9,1,9,1,9,1,9,1,9,1,9,4,9,435,8,9,11,9,12,9,436,1,9,1,9,1,
        9,1,9,3,9,443,8,9,1,9,1,9,1,9,1,9,1,9,1,9,1,9,1,9,1,9,1,9,4,9,455,
        8,9,11,9,12,9,456,1,9,1,9,1,9,1,9,1,9,4,9,464,8,9,11,9,12,9,465,
        3,9,468,8,9,1,9,1,9,1,9,1,9,3,9,474,8,9,1,9,1,9,1,9,1,9,1,9,1,9,
        1,9,1,9,1,9,1,9,4,9,486,8,9,11,9,12,9,487,1,9,1,9,1,9,1,9,1,9,4,
        9,495,8,9,11,9,12,9,496,3,9,499,8,9,1,9,1,9,1,9,1,9,3,9,505,8,9,
        1,9,1,9,3,9,509,8,9,1,10,1,10,1,10,1,10,1,10,1,10,1,10,1,10,4,10,
        519,8,10,11,10,12,10,520,1,10,1,10,1,10,1,10,3,10,527,8,10,1,10,
        1,10,1,10,1,10,1,10,1,10,1,10,1,10,1,10,1,10,4,10,539,8,10,11,10,
        12,10,540,1,10,1,10,1,10,1,10,1,10,4,10,548,8,10,11,10,12,10,549,
        3,10,552,8,10,1,10,1,10,1,10,1,10,3,10,558,8,10,1,10,1,10,1,10,1,
        10,1,10,1,10,1,10,1,10,1,10,1,10,4,10,570,8,10,11,10,12,10,571,1,
        10,1,10,1,10,1,10,1,10,4,10,579,8,10,11,10,12,10,580,3,10,583,8,
        10,1,10,1,10,1,10,1,10,3,10,589,8,10,1,10,1,10,3,10,593,8,10,1,11,
        1,11,1,11,1,11,1,11,1,11,1,11,1,11,1,11,1,11,1,11,1,11,1,11,1,11,
        1,11,1,11,1,11,1,11,1,11,1,11,1,11,1,11,1,11,1,11,3,11,619,8,11,
        1,12,1,12,1,12,1,12,1,12,1,12,1,12,1,12,3,12,629,8,12,1,13,1,13,
        1,13,1,13,1,14,1,14,1,14,1,14,1,15,1,15,1,15,1,15,1,15,1,15,1,15,
        1,15,1,15,5,15,648,8,15,10,15,12,15,651,9,15,1,15,1,15,1,15,1,15,
        1,15,3,15,658,8,15,3,15,660,8,15,1,16,1,16,1,16,1,16,1,17,1,17,3,
        17,668,8,17,1,18,1,18,1,18,1,18,1,18,1,18,1,18,1,18,3,18,678,8,18,
        1,19,1,19,1,19,1,19,3,19,684,8,19,1,19,3,19,687,8,19,1,20,1,20,1,
        20,3,20,692,8,20,1,20,1,20,1,20,3,20,697,8,20,5,20,699,8,20,10,20,
        12,20,702,9,20,1,20,1,20,1,21,1,21,1,21,1,21,1,21,1,21,1,21,1,21,
        1,21,3,21,715,8,21,1,21,3,21,718,8,21,1,21,1,21,1,21,3,21,723,8,
        21,1,22,1,22,1,22,1,22,3,22,729,8,22,1,23,3,23,732,8,23,1,23,1,23,
        1,23,3,23,737,8,23,1,23,1,23,1,23,1,23,1,23,5,23,744,8,23,10,23,
        12,23,747,9,23,1,23,3,23,750,8,23,1,24,1,24,3,24,754,8,24,1,24,1,
        24,1,24,1,24,3,24,760,8,24,1,24,1,24,1,24,1,24,3,24,766,8,24,1,24,
        1,24,3,24,770,8,24,1,24,0,1,0,25,0,2,4,6,8,10,12,14,16,18,20,22,
        24,26,28,30,32,34,36,38,40,42,44,46,48,0,3,1,0,30,31,2,0,30,30,35,
        35,2,0,31,31,37,37,881,0,73,1,0,0,0,2,86,1,0,0,0,4,140,1,0,0,0,6,
        154,1,0,0,0,8,228,1,0,0,0,10,266,1,0,0,0,12,268,1,0,0,0,14,296,1,
        0,0,0,16,424,1,0,0,0,18,508,1,0,0,0,20,592,1,0,0,0,22,618,1,0,0,
        0,24,628,1,0,0,0,26,630,1,0,0,0,28,634,1,0,0,0,30,659,1,0,0,0,32,
        661,1,0,0,0,34,667,1,0,0,0,36,677,1,0,0,0,38,686,1,0,0,0,40,688,
        1,0,0,0,42,722,1,0,0,0,44,728,1,0,0,0,46,749,1,0,0,0,48,769,1,0,
        0,0,50,51,6,0,-1,0,51,52,5,1,0,0,52,53,3,0,0,0,53,54,5,2,0,0,54,
        74,1,0,0,0,55,74,3,8,4,0,56,74,3,6,3,0,57,74,3,16,8,0,58,74,3,10,
        5,0,59,74,3,22,11,0,60,74,3,24,12,0,61,74,3,26,13,0,62,74,3,28,14,
        0,63,74,3,40,20,0,64,74,3,42,21,0,65,67,5,22,0,0,66,65,1,0,0,0,66,
        67,1,0,0,0,67,68,1,0,0,0,68,74,3,38,19,0,69,74,3,46,23,0,70,74,3,
        48,24,0,71,74,5,23,0,0,72,74,5,24,0,0,73,50,1,0,0,0,73,55,1,0,0,
        0,73,56,1,0,0,0,73,57,1,0,0,0,73,58,1,0,0,0,73,59,1,0,0,0,73,60,
        1,0,0,0,73,61,1,0,0,0,73,62,1,0,0,0,73,63,1,0,0,0,73,64,1,0,0,0,
        73,66,1,0,0,0,73,69,1,0,0,0,73,70,1,0,0,0,73,71,1,0,0,0,73,72,1,
        0,0,0,74,83,1,0,0,0,75,76,10,17,0,0,76,77,5,20,0,0,77,82,3,0,0,18,
        78,79,10,16,0,0,79,80,5,21,0,0,80,82,3,0,0,17,81,75,1,0,0,0,81,78,
        1,0,0,0,82,85,1,0,0,0,83,81,1,0,0,0,83,84,1,0,0,0,84,1,1,0,0,0,85,
        83,1,0,0,0,86,91,3,4,2,0,87,88,5,3,0,0,88,90,3,4,2,0,89,87,1,0,0,
        0,90,93,1,0,0,0,91,89,1,0,0,0,91,92,1,0,0,0,92,95,1,0,0,0,93,91,
        1,0,0,0,94,96,5,3,0,0,95,94,1,0,0,0,95,96,1,0,0,0,96,3,1,0,0,0,97,
        98,5,32,0,0,98,105,5,4,0,0,99,106,5,23,0,0,100,106,5,24,0,0,101,
        106,5,34,0,0,102,106,5,32,0,0,103,106,3,44,22,0,104,106,3,34,17,
        0,105,99,1,0,0,0,105,100,1,0,0,0,105,101,1,0,0,0,105,102,1,0,0,0,
        105,103,1,0,0,0,105,104,1,0,0,0,106,141,1,0,0,0,107,108,5,32,0,0,
        108,109,5,39,0,0,109,110,5,4,0,0,110,141,3,34,17,0,111,141,3,8,4,
        0,112,113,5,25,0,0,113,114,5,1,0,0,114,115,3,0,0,0,115,116,5,2,0,
        0,116,117,5,5,0,0,117,118,3,2,1,0,118,130,5,6,0,0,119,120,5,26,0,
        0,120,121,5,25,0,0,121,122,5,1,0,0,122,123,3,0,0,0,123,124,5,2,0,
        0,124,125,5,5,0,0,125,126,3,2,1,0,126,127,5,6,0,0,127,129,1,0,0,
        0,128,119,1,0,0,0,129,132,1,0,0,0,130,128,1,0,0,0,130,131,1,0,0,
        0,131,138,1,0,0,0,132,130,1,0,0,0,133,134,5,26,0,0,134,135,5,5,0,
        0,135,136,3,2,1,0,136,137,5,6,0,0,137,139,1,0,0,0,138,133,1,0,0,
        0,138,139,1,0,0,0,139,141,1,0,0,0,140,97,1,0,0,0,140,107,1,0,0,0,
        140,111,1,0,0,0,140,112,1,0,0,0,141,5,1,0,0,0,142,143,5,33,0,0,143,
        144,5,1,0,0,144,145,5,35,0,0,145,146,5,7,0,0,146,147,3,0,0,0,147,
        148,5,2,0,0,148,155,1,0,0,0,149,150,5,33,0,0,150,151,5,1,0,0,151,
        152,3,0,0,0,152,153,5,2,0,0,153,155,1,0,0,0,154,142,1,0,0,0,154,
        149,1,0,0,0,155,7,1,0,0,0,156,158,5,22,0,0,157,156,1,0,0,0,157,158,
        1,0,0,0,158,159,1,0,0,0,159,160,5,33,0,0,160,161,5,1,0,0,161,166,
        5,30,0,0,162,163,5,7,0,0,163,165,5,30,0,0,164,162,1,0,0,0,165,168,
        1,0,0,0,166,164,1,0,0,0,166,167,1,0,0,0,167,169,1,0,0,0,168,166,
        1,0,0,0,169,229,5,2,0,0,170,172,5,22,0,0,171,170,1,0,0,0,171,172,
        1,0,0,0,172,173,1,0,0,0,173,174,5,33,0,0,174,175,5,1,0,0,175,176,
        3,38,19,0,176,177,5,2,0,0,177,229,1,0,0,0,178,180,5,22,0,0,179,178,
        1,0,0,0,179,180,1,0,0,0,180,181,1,0,0,0,181,182,5,33,0,0,182,183,
        5,1,0,0,183,184,5,35,0,0,184,229,5,2,0,0,185,187,5,22,0,0,186,185,
        1,0,0,0,186,187,1,0,0,0,187,188,1,0,0,0,188,189,5,33,0,0,189,190,
        5,1,0,0,190,191,5,37,0,0,191,229,5,2,0,0,192,194,5,22,0,0,193,192,
        1,0,0,0,193,194,1,0,0,0,194,195,1,0,0,0,195,196,5,33,0,0,196,197,
        5,1,0,0,197,198,5,38,0,0,198,229,5,2,0,0,199,201,5,22,0,0,200,199,
        1,0,0,0,200,201,1,0,0,0,201,202,1,0,0,0,202,203,5,33,0,0,203,204,
        5,1,0,0,204,209,5,34,0,0,205,206,5,7,0,0,206,208,5,34,0,0,207,205,
        1,0,0,0,208,211,1,0,0,0,209,207,1,0,0,0,209,210,1,0,0,0,210,212,
        1,0,0,0,211,209,1,0,0,0,212,229,5,2,0,0,213,215,5,22,0,0,214,213,
        1,0,0,0,214,215,1,0,0,0,215,216,1,0,0,0,216,217,5,33,0,0,217,218,
        5,1,0,0,218,219,5,32,0,0,219,229,5,2,0,0,220,222,5,22,0,0,221,220,
        1,0,0,0,221,222,1,0,0,0,222,223,1,0,0,0,223,226,5,33,0,0,224,225,
        5,1,0,0,225,227,5,2,0,0,226,224,1,0,0,0,226,227,1,0,0,0,227,229,
        1,0,0,0,228,157,1,0,0,0,228,171,1,0,0,0,228,179,1,0,0,0,228,186,
        1,0,0,0,228,193,1,0,0,0,228,200,1,0,0,0,228,214,1,0,0,0,228,221,
        1,0,0,0,229,9,1,0,0,0,230,231,5,25,0,0,231,232,5,1,0,0,232,233,3,
        0,0,0,233,234,5,2,0,0,234,235,5,5,0,0,235,236,3,0,0,0,236,248,5,
        6,0,0,237,238,5,26,0,0,238,239,5,25,0,0,239,240,5,1,0,0,240,241,
        3,0,0,0,241,242,5,2,0,0,242,243,5,5,0,0,243,244,3,0,0,0,244,245,
        5,6,0,0,245,247,1,0,0,0,246,237,1,0,0,0,247,250,1,0,0,0,248,246,
        1,0,0,0,248,249,1,0,0,0,249,256,1,0,0,0,250,248,1,0,0,0,251,252,
        5,26,0,0,252,253,5,5,0,0,253,254,3,0,0,0,254,255,5,6,0,0,255,257,
        1,0,0,0,256,251,1,0,0,0,256,257,1,0,0,0,257,267,1,0,0,0,258,259,
        5,1,0,0,259,260,3,0,0,0,260,261,5,25,0,0,261,262,3,0,0,0,262,263,
        5,26,0,0,263,264,3,0,0,0,264,265,5,2,0,0,265,267,1,0,0,0,266,230,
        1,0,0,0,266,258,1,0,0,0,267,11,1,0,0,0,268,269,5,25,0,0,269,270,
        5,1,0,0,270,271,3,0,0,0,271,272,5,2,0,0,272,273,5,5,0,0,273,274,
        3,34,17,0,274,286,5,6,0,0,275,276,5,26,0,0,276,277,5,25,0,0,277,
        278,5,1,0,0,278,279,3,0,0,0,279,280,5,2,0,0,280,281,5,5,0,0,281,
        282,3,34,17,0,282,283,5,6,0,0,283,285,1,0,0,0,284,275,1,0,0,0,285,
        288,1,0,0,0,286,284,1,0,0,0,286,287,1,0,0,0,287,294,1,0,0,0,288,
        286,1,0,0,0,289,290,5,26,0,0,290,291,5,5,0,0,291,292,3,34,17,0,292,
        293,5,6,0,0,293,295,1,0,0,0,294,289,1,0,0,0,294,295,1,0,0,0,295,
        13,1,0,0,0,296,297,5,25,0,0,297,298,5,1,0,0,298,299,3,0,0,0,299,
        300,5,2,0,0,300,301,5,5,0,0,301,302,3,44,22,0,302,314,5,6,0,0,303,
        304,5,26,0,0,304,305,5,25,0,0,305,306,5,1,0,0,306,307,3,0,0,0,307,
        308,5,2,0,0,308,309,5,5,0,0,309,310,3,44,22,0,310,311,5,6,0,0,311,
        313,1,0,0,0,312,303,1,0,0,0,313,316,1,0,0,0,314,312,1,0,0,0,314,
        315,1,0,0,0,315,322,1,0,0,0,316,314,1,0,0,0,317,318,5,26,0,0,318,
        319,5,5,0,0,319,320,3,44,22,0,320,321,5,6,0,0,321,323,1,0,0,0,322,
        317,1,0,0,0,322,323,1,0,0,0,323,15,1,0,0,0,324,325,5,28,0,0,325,
        326,5,30,0,0,326,332,5,5,0,0,327,328,5,37,0,0,328,329,5,8,0,0,329,
        330,3,0,0,0,330,331,5,7,0,0,331,333,1,0,0,0,332,327,1,0,0,0,333,
        334,1,0,0,0,334,332,1,0,0,0,334,335,1,0,0,0,335,336,1,0,0,0,336,
        337,5,9,0,0,337,338,5,8,0,0,338,340,3,0,0,0,339,341,5,7,0,0,340,
        339,1,0,0,0,340,341,1,0,0,0,341,342,1,0,0,0,342,343,5,6,0,0,343,
        425,1,0,0,0,344,345,5,28,0,0,345,346,5,31,0,0,346,365,5,5,0,0,347,
        348,5,37,0,0,348,349,5,8,0,0,349,350,3,0,0,0,350,351,5,7,0,0,351,
        353,1,0,0,0,352,347,1,0,0,0,353,354,1,0,0,0,354,352,1,0,0,0,354,
        355,1,0,0,0,355,366,1,0,0,0,356,357,5,35,0,0,357,358,5,8,0,0,358,
        359,3,0,0,0,359,360,5,7,0,0,360,362,1,0,0,0,361,356,1,0,0,0,362,
        363,1,0,0,0,363,361,1,0,0,0,363,364,1,0,0,0,364,366,1,0,0,0,365,
        352,1,0,0,0,365,361,1,0,0,0,366,367,1,0,0,0,367,368,5,9,0,0,368,
        369,5,8,0,0,369,371,3,0,0,0,370,372,5,7,0,0,371,370,1,0,0,0,371,
        372,1,0,0,0,372,373,1,0,0,0,373,374,5,6,0,0,374,425,1,0,0,0,375,
        376,5,28,0,0,376,377,5,32,0,0,377,390,5,5,0,0,378,383,5,30,0,0,379,
        380,5,10,0,0,380,382,5,30,0,0,381,379,1,0,0,0,382,385,1,0,0,0,383,
        381,1,0,0,0,383,384,1,0,0,0,384,386,1,0,0,0,385,383,1,0,0,0,386,
        387,5,8,0,0,387,388,3,0,0,0,388,389,5,7,0,0,389,391,1,0,0,0,390,
        378,1,0,0,0,391,392,1,0,0,0,392,390,1,0,0,0,392,393,1,0,0,0,393,
        394,1,0,0,0,394,395,5,9,0,0,395,396,5,8,0,0,396,398,3,0,0,0,397,
        399,5,7,0,0,398,397,1,0,0,0,398,399,1,0,0,0,399,400,1,0,0,0,400,
        401,5,6,0,0,401,425,1,0,0,0,402,403,5,32,0,0,403,404,5,27,0,0,404,
        405,5,11,0,0,405,408,5,30,0,0,406,407,5,7,0,0,407,409,5,30,0,0,408,
        406,1,0,0,0,409,410,1,0,0,0,410,408,1,0,0,0,410,411,1,0,0,0,411,
        412,1,0,0,0,412,425,5,12,0,0,413,414,5,32,0,0,414,415,5,27,0,0,415,
        416,5,11,0,0,416,419,5,35,0,0,417,418,5,7,0,0,418,420,5,35,0,0,419,
        417,1,0,0,0,420,421,1,0,0,0,421,419,1,0,0,0,421,422,1,0,0,0,422,
        423,1,0,0,0,423,425,5,12,0,0,424,324,1,0,0,0,424,344,1,0,0,0,424,
        375,1,0,0,0,424,402,1,0,0,0,424,413,1,0,0,0,425,17,1,0,0,0,426,427,
        5,28,0,0,427,428,5,30,0,0,428,434,5,5,0,0,429,430,5,37,0,0,430,431,
        5,8,0,0,431,432,3,34,17,0,432,433,5,7,0,0,433,435,1,0,0,0,434,429,
        1,0,0,0,435,436,1,0,0,0,436,434,1,0,0,0,436,437,1,0,0,0,437,438,
        1,0,0,0,438,439,5,9,0,0,439,440,5,8,0,0,440,442,3,34,17,0,441,443,
        5,7,0,0,442,441,1,0,0,0,442,443,1,0,0,0,443,444,1,0,0,0,444,445,
        5,6,0,0,445,509,1,0,0,0,446,447,5,28,0,0,447,448,5,32,0,0,448,467,
        5,5,0,0,449,450,5,37,0,0,450,451,5,8,0,0,451,452,3,34,17,0,452,453,
        5,7,0,0,453,455,1,0,0,0,454,449,1,0,0,0,455,456,1,0,0,0,456,454,
        1,0,0,0,456,457,1,0,0,0,457,468,1,0,0,0,458,459,5,35,0,0,459,460,
        5,8,0,0,460,461,3,34,17,0,461,462,5,7,0,0,462,464,1,0,0,0,463,458,
        1,0,0,0,464,465,1,0,0,0,465,463,1,0,0,0,465,466,1,0,0,0,466,468,
        1,0,0,0,467,454,1,0,0,0,467,463,1,0,0,0,468,469,1,0,0,0,469,470,
        5,9,0,0,470,471,5,8,0,0,471,473,3,34,17,0,472,474,5,7,0,0,473,472,
        1,0,0,0,473,474,1,0,0,0,474,475,1,0,0,0,475,476,5,6,0,0,476,509,
        1,0,0,0,477,478,5,28,0,0,478,479,5,31,0,0,479,498,5,5,0,0,480,481,
        5,37,0,0,481,482,5,8,0,0,482,483,3,34,17,0,483,484,5,7,0,0,484,486,
        1,0,0,0,485,480,1,0,0,0,486,487,1,0,0,0,487,485,1,0,0,0,487,488,
        1,0,0,0,488,499,1,0,0,0,489,490,5,35,0,0,490,491,5,8,0,0,491,492,
        3,34,17,0,492,493,5,7,0,0,493,495,1,0,0,0,494,489,1,0,0,0,495,496,
        1,0,0,0,496,494,1,0,0,0,496,497,1,0,0,0,497,499,1,0,0,0,498,485,
        1,0,0,0,498,494,1,0,0,0,499,500,1,0,0,0,500,501,5,9,0,0,501,502,
        5,8,0,0,502,504,3,34,17,0,503,505,5,7,0,0,504,503,1,0,0,0,504,505,
        1,0,0,0,505,506,1,0,0,0,506,507,5,6,0,0,507,509,1,0,0,0,508,426,
        1,0,0,0,508,446,1,0,0,0,508,477,1,0,0,0,509,19,1,0,0,0,510,511,5,
        28,0,0,511,512,5,30,0,0,512,518,5,5,0,0,513,514,5,37,0,0,514,515,
        5,8,0,0,515,516,3,44,22,0,516,517,5,7,0,0,517,519,1,0,0,0,518,513,
        1,0,0,0,519,520,1,0,0,0,520,518,1,0,0,0,520,521,1,0,0,0,521,522,
        1,0,0,0,522,523,5,9,0,0,523,524,5,8,0,0,524,526,3,44,22,0,525,527,
        5,7,0,0,526,525,1,0,0,0,526,527,1,0,0,0,527,528,1,0,0,0,528,529,
        5,6,0,0,529,593,1,0,0,0,530,531,5,28,0,0,531,532,5,32,0,0,532,551,
        5,5,0,0,533,534,5,37,0,0,534,535,5,8,0,0,535,536,3,44,22,0,536,537,
        5,7,0,0,537,539,1,0,0,0,538,533,1,0,0,0,539,540,1,0,0,0,540,538,
        1,0,0,0,540,541,1,0,0,0,541,552,1,0,0,0,542,543,5,35,0,0,543,544,
        5,8,0,0,544,545,3,44,22,0,545,546,5,7,0,0,546,548,1,0,0,0,547,542,
        1,0,0,0,548,549,1,0,0,0,549,547,1,0,0,0,549,550,1,0,0,0,550,552,
        1,0,0,0,551,538,1,0,0,0,551,547,1,0,0,0,552,553,1,0,0,0,553,554,
        5,9,0,0,554,555,5,8,0,0,555,557,3,44,22,0,556,558,5,7,0,0,557,556,
        1,0,0,0,557,558,1,0,0,0,558,559,1,0,0,0,559,560,5,6,0,0,560,593,
        1,0,0,0,561,562,5,28,0,0,562,563,5,31,0,0,563,582,5,5,0,0,564,565,
        5,37,0,0,565,566,5,8,0,0,566,567,3,44,22,0,567,568,5,7,0,0,568,570,
        1,0,0,0,569,564,1,0,0,0,570,571,1,0,0,0,571,569,1,0,0,0,571,572,
        1,0,0,0,572,583,1,0,0,0,573,574,5,35,0,0,574,575,5,8,0,0,575,576,
        3,44,22,0,576,577,5,7,0,0,577,579,1,0,0,0,578,573,1,0,0,0,579,580,
        1,0,0,0,580,578,1,0,0,0,580,581,1,0,0,0,581,583,1,0,0,0,582,569,
        1,0,0,0,582,578,1,0,0,0,583,584,1,0,0,0,584,585,5,9,0,0,585,586,
        5,8,0,0,586,588,3,44,22,0,587,589,5,7,0,0,588,587,1,0,0,0,588,589,
        1,0,0,0,589,590,1,0,0,0,590,591,5,6,0,0,591,593,1,0,0,0,592,510,
        1,0,0,0,592,530,1,0,0,0,592,561,1,0,0,0,593,21,1,0,0,0,594,595,3,
        38,19,0,595,596,5,13,0,0,596,597,3,34,17,0,597,619,1,0,0,0,598,599,
        3,38,19,0,599,600,5,14,0,0,600,601,3,34,17,0,601,619,1,0,0,0,602,
        603,3,38,19,0,603,604,5,15,0,0,604,605,3,34,17,0,605,619,1,0,0,0,
        606,607,3,38,19,0,607,608,5,16,0,0,608,609,3,34,17,0,609,619,1,0,
        0,0,610,611,3,38,19,0,611,612,5,17,0,0,612,613,3,34,17,0,613,619,
        1,0,0,0,614,615,3,38,19,0,615,616,5,18,0,0,616,617,3,34,17,0,617,
        619,1,0,0,0,618,594,1,0,0,0,618,598,1,0,0,0,618,602,1,0,0,0,618,
        606,1,0,0,0,618,610,1,0,0,0,618,614,1,0,0,0,619,23,1,0,0,0,620,621,
        3,38,19,0,621,622,5,13,0,0,622,623,5,35,0,0,623,629,1,0,0,0,624,
        625,3,38,19,0,625,626,5,14,0,0,626,627,5,35,0,0,627,629,1,0,0,0,
        628,620,1,0,0,0,628,624,1,0,0,0,629,25,1,0,0,0,630,631,3,38,19,0,
        631,632,5,19,0,0,632,633,3,34,17,0,633,27,1,0,0,0,634,635,5,32,0,
        0,635,636,5,13,0,0,636,637,7,0,0,0,637,29,1,0,0,0,638,639,5,33,0,
        0,639,640,5,1,0,0,640,641,5,30,0,0,641,660,5,2,0,0,642,643,5,33,
        0,0,643,644,5,1,0,0,644,649,3,34,17,0,645,646,5,7,0,0,646,648,3,
        34,17,0,647,645,1,0,0,0,648,651,1,0,0,0,649,647,1,0,0,0,649,650,
        1,0,0,0,650,652,1,0,0,0,651,649,1,0,0,0,652,653,5,2,0,0,653,660,
        1,0,0,0,654,657,5,33,0,0,655,656,5,1,0,0,656,658,5,2,0,0,657,655,
        1,0,0,0,657,658,1,0,0,0,658,660,1,0,0,0,659,638,1,0,0,0,659,642,
        1,0,0,0,659,654,1,0,0,0,660,31,1,0,0,0,661,662,3,36,18,0,662,663,
        5,39,0,0,663,664,3,34,17,0,664,33,1,0,0,0,665,668,3,36,18,0,666,
        668,3,32,16,0,667,665,1,0,0,0,667,666,1,0,0,0,668,35,1,0,0,0,669,
        678,5,37,0,0,670,678,5,36,0,0,671,678,5,31,0,0,672,678,5,32,0,0,
        673,678,3,38,19,0,674,678,3,18,9,0,675,678,3,30,15,0,676,678,3,12,
        6,0,677,669,1,0,0,0,677,670,1,0,0,0,677,671,1,0,0,0,677,672,1,0,
        0,0,677,673,1,0,0,0,677,674,1,0,0,0,677,675,1,0,0,0,677,676,1,0,
        0,0,678,37,1,0,0,0,679,683,5,31,0,0,680,681,5,11,0,0,681,682,7,1,
        0,0,682,684,5,12,0,0,683,680,1,0,0,0,683,684,1,0,0,0,684,687,1,0,
        0,0,685,687,5,32,0,0,686,679,1,0,0,0,686,685,1,0,0,0,687,39,1,0,
        0,0,688,691,5,11,0,0,689,692,5,33,0,0,690,692,3,42,21,0,691,689,
        1,0,0,0,691,690,1,0,0,0,692,700,1,0,0,0,693,696,5,7,0,0,694,697,
        5,33,0,0,695,697,3,42,21,0,696,694,1,0,0,0,696,695,1,0,0,0,697,699,
        1,0,0,0,698,693,1,0,0,0,699,702,1,0,0,0,700,698,1,0,0,0,700,701,
        1,0,0,0,701,703,1,0,0,0,702,700,1,0,0,0,703,704,5,12,0,0,704,41,
        1,0,0,0,705,706,5,30,0,0,706,707,5,5,0,0,707,708,7,2,0,0,708,715,
        5,6,0,0,709,710,5,1,0,0,710,711,5,30,0,0,711,712,5,7,0,0,712,713,
        7,2,0,0,713,715,5,2,0,0,714,705,1,0,0,0,714,709,1,0,0,0,715,723,
        1,0,0,0,716,718,5,22,0,0,717,716,1,0,0,0,717,718,1,0,0,0,718,719,
        1,0,0,0,719,723,5,30,0,0,720,723,5,35,0,0,721,723,5,32,0,0,722,714,
        1,0,0,0,722,717,1,0,0,0,722,720,1,0,0,0,722,721,1,0,0,0,723,43,1,
        0,0,0,724,729,5,35,0,0,725,729,3,38,19,0,726,729,3,14,7,0,727,729,
        3,20,10,0,728,724,1,0,0,0,728,725,1,0,0,0,728,726,1,0,0,0,728,727,
        1,0,0,0,729,45,1,0,0,0,730,732,5,22,0,0,731,730,1,0,0,0,731,732,
        1,0,0,0,732,733,1,0,0,0,733,734,5,29,0,0,734,750,5,34,0,0,735,737,
        5,22,0,0,736,735,1,0,0,0,736,737,1,0,0,0,737,738,1,0,0,0,738,739,
        5,29,0,0,739,740,5,1,0,0,740,745,5,34,0,0,741,742,5,7,0,0,742,744,
        5,34,0,0,743,741,1,0,0,0,744,747,1,0,0,0,745,743,1,0,0,0,745,746,
        1,0,0,0,746,748,1,0,0,0,747,745,1,0,0,0,748,750,5,2,0,0,749,731,
        1,0,0,0,749,736,1,0,0,0,750,47,1,0,0,0,751,753,5,32,0,0,752,754,
        5,22,0,0,753,752,1,0,0,0,753,754,1,0,0,0,754,755,1,0,0,0,755,756,
        5,29,0,0,756,770,5,32,0,0,757,759,5,32,0,0,758,760,5,22,0,0,759,
        758,1,0,0,0,759,760,1,0,0,0,760,761,1,0,0,0,761,762,5,29,0,0,762,
        770,5,34,0,0,763,765,5,32,0,0,764,766,5,22,0,0,765,764,1,0,0,0,765,
        766,1,0,0,0,766,767,1,0,0,0,767,768,5,29,0,0,768,770,3,8,4,0,769,
        751,1,0,0,0,769,757,1,0,0,0,769,763,1,0,0,0,770,49,1,0,0,0,88,66,
        73,81,83,91,95,105,130,138,140,154,157,166,171,179,186,193,200,209,
        214,221,226,228,248,256,266,286,294,314,322,334,340,354,363,365,
        371,383,392,398,410,421,424,436,442,456,465,467,473,487,496,498,
        504,508,520,526,540,549,551,557,571,580,582,588,592,618,628,649,
        657,659,667,677,683,686,691,696,700,714,717,722,728,731,736,745,
        749,753,759,765,769
    ]

class RulesParser ( Parser ):

    grammarFileName = "Rules.g4"

    atn = ATNDeserializer().deserialize(serializedATN())

    decisionsToDFA = [ DFA(ds, i) for i, ds in enumerate(atn.decisionToState) ]

    sharedContextCache = PredictionContextCache()

    literalNames = [ "<INVALID>", "'('", "')'", "';'", "'='", "'{'", "'}'", 
                     "','", "'=>'", "'_'", "'|'", "'['", "']'", "'=='", 
                     "'!='", "'>='", "'<='", "'<'", "'>'", "'&'" ]

    symbolicNames = [ "<INVALID>", "<INVALID>", "<INVALID>", "<INVALID>", 
                      "<INVALID>", "<INVALID>", "<INVALID>", "<INVALID>", 
                      "<INVALID>", "<INVALID>", "<INVALID>", "<INVALID>", 
                      "<INVALID>", "<INVALID>", "<INVALID>", "<INVALID>", 
                      "<INVALID>", "<INVALID>", "<INVALID>", "<INVALID>", 
                      "AND", "OR", "NOT", "TRUE", "FALSE", "IF", "ELSE", 
                      "IN", "PER", "WITHIN", "ITEM", "SETTING", "REF", "FUNC", 
                      "PLACE", "LIT", "CONST", "INT", "FLOAT", "BINOP", 
                      "WS" ]

    RULE_boolExpr = 0
    RULE_actions = 1
    RULE_action = 2
    RULE_meta = 3
    RULE_invoke = 4
    RULE_cond = 5
    RULE_condNum = 6
    RULE_condStr = 7
    RULE_switchBool = 8
    RULE_switchNum = 9
    RULE_switchStr = 10
    RULE_cmp = 11
    RULE_cmpStr = 12
    RULE_flagMatch = 13
    RULE_refEq = 14
    RULE_funcNum = 15
    RULE_mathNum = 16
    RULE_num = 17
    RULE_baseNum = 18
    RULE_value = 19
    RULE_itemList = 20
    RULE_item = 21
    RULE_str = 22
    RULE_somewhere = 23
    RULE_refSomewhere = 24

    ruleNames =  [ "boolExpr", "actions", "action", "meta", "invoke", "cond", 
                   "condNum", "condStr", "switchBool", "switchNum", "switchStr", 
                   "cmp", "cmpStr", "flagMatch", "refEq", "funcNum", "mathNum", 
                   "num", "baseNum", "value", "itemList", "item", "str", 
                   "somewhere", "refSomewhere" ]

    EOF = Token.EOF
    T__0=1
    T__1=2
    T__2=3
    T__3=4
    T__4=5
    T__5=6
    T__6=7
    T__7=8
    T__8=9
    T__9=10
    T__10=11
    T__11=12
    T__12=13
    T__13=14
    T__14=15
    T__15=16
    T__16=17
    T__17=18
    T__18=19
    AND=20
    OR=21
    NOT=22
    TRUE=23
    FALSE=24
    IF=25
    ELSE=26
    IN=27
    PER=28
    WITHIN=29
    ITEM=30
    SETTING=31
    REF=32
    FUNC=33
    PLACE=34
    LIT=35
    CONST=36
    INT=37
    FLOAT=38
    BINOP=39
    WS=40

    def __init__(self, input:TokenStream, output:TextIO = sys.stdout):
        super().__init__(input, output)
        self.checkVersion("4.13.1")
        self._interp = ParserATNSimulator(self, self.atn, self.decisionsToDFA, self.sharedContextCache)
        self._predicates = None




    class BoolExprContext(ParserRuleContext):
        __slots__ = 'parser'

        def __init__(self, parser, parent:ParserRuleContext=None, invokingState:int=-1):
            super().__init__(parent, invokingState)
            self.parser = parser

        def boolExpr(self, i:int=None):
            if i is None:
                return self.getTypedRuleContexts(RulesParser.BoolExprContext)
            else:
                return self.getTypedRuleContext(RulesParser.BoolExprContext,i)


        def invoke(self):
            return self.getTypedRuleContext(RulesParser.InvokeContext,0)


        def meta(self):
            return self.getTypedRuleContext(RulesParser.MetaContext,0)


        def switchBool(self):
            return self.getTypedRuleContext(RulesParser.SwitchBoolContext,0)


        def cond(self):
            return self.getTypedRuleContext(RulesParser.CondContext,0)


        def cmp(self):
            return self.getTypedRuleContext(RulesParser.CmpContext,0)


        def cmpStr(self):
            return self.getTypedRuleContext(RulesParser.CmpStrContext,0)


        def flagMatch(self):
            return self.getTypedRuleContext(RulesParser.FlagMatchContext,0)


        def refEq(self):
            return self.getTypedRuleContext(RulesParser.RefEqContext,0)


        def itemList(self):
            return self.getTypedRuleContext(RulesParser.ItemListContext,0)


        def item(self):
            return self.getTypedRuleContext(RulesParser.ItemContext,0)


        def value(self):
            return self.getTypedRuleContext(RulesParser.ValueContext,0)


        def NOT(self):
            return self.getToken(RulesParser.NOT, 0)

        def somewhere(self):
            return self.getTypedRuleContext(RulesParser.SomewhereContext,0)


        def refSomewhere(self):
            return self.getTypedRuleContext(RulesParser.RefSomewhereContext,0)


        def TRUE(self):
            return self.getToken(RulesParser.TRUE, 0)

        def FALSE(self):
            return self.getToken(RulesParser.FALSE, 0)

        def AND(self):
            return self.getToken(RulesParser.AND, 0)

        def OR(self):
            return self.getToken(RulesParser.OR, 0)

        def getRuleIndex(self):
            return RulesParser.RULE_boolExpr

        def enterRule(self, listener:ParseTreeListener):
            if hasattr( listener, "enterBoolExpr" ):
                listener.enterBoolExpr(self)

        def exitRule(self, listener:ParseTreeListener):
            if hasattr( listener, "exitBoolExpr" ):
                listener.exitBoolExpr(self)

        def accept(self, visitor:ParseTreeVisitor):
            if hasattr( visitor, "visitBoolExpr" ):
                return visitor.visitBoolExpr(self)
            else:
                return visitor.visitChildren(self)



    def boolExpr(self, _p:int=0):
        _parentctx = self._ctx
        _parentState = self.state
        localctx = RulesParser.BoolExprContext(self, self._ctx, _parentState)
        _prevctx = localctx
        _startState = 0
        self.enterRecursionRule(localctx, 0, self.RULE_boolExpr, _p)
        self._la = 0 # Token type
        try:
            self.enterOuterAlt(localctx, 1)
            self.state = 73
            self._errHandler.sync(self)
            la_ = self._interp.adaptivePredict(self._input,1,self._ctx)
            if la_ == 1:
                self.state = 51
                self.match(RulesParser.T__0)
                self.state = 52
                self.boolExpr(0)
                self.state = 53
                self.match(RulesParser.T__1)
                pass

            elif la_ == 2:
                self.state = 55
                self.invoke()
                pass

            elif la_ == 3:
                self.state = 56
                self.meta()
                pass

            elif la_ == 4:
                self.state = 57
                self.switchBool()
                pass

            elif la_ == 5:
                self.state = 58
                self.cond()
                pass

            elif la_ == 6:
                self.state = 59
                self.cmp()
                pass

            elif la_ == 7:
                self.state = 60
                self.cmpStr()
                pass

            elif la_ == 8:
                self.state = 61
                self.flagMatch()
                pass

            elif la_ == 9:
                self.state = 62
                self.refEq()
                pass

            elif la_ == 10:
                self.state = 63
                self.itemList()
                pass

            elif la_ == 11:
                self.state = 64
                self.item()
                pass

            elif la_ == 12:
                self.state = 66
                self._errHandler.sync(self)
                _la = self._input.LA(1)
                if _la==22:
                    self.state = 65
                    self.match(RulesParser.NOT)


                self.state = 68
                self.value()
                pass

            elif la_ == 13:
                self.state = 69
                self.somewhere()
                pass

            elif la_ == 14:
                self.state = 70
                self.refSomewhere()
                pass

            elif la_ == 15:
                self.state = 71
                self.match(RulesParser.TRUE)
                pass

            elif la_ == 16:
                self.state = 72
                self.match(RulesParser.FALSE)
                pass


            self._ctx.stop = self._input.LT(-1)
            self.state = 83
            self._errHandler.sync(self)
            _alt = self._interp.adaptivePredict(self._input,3,self._ctx)
            while _alt!=2 and _alt!=ATN.INVALID_ALT_NUMBER:
                if _alt==1:
                    if self._parseListeners is not None:
                        self.triggerExitRuleEvent()
                    _prevctx = localctx
                    self.state = 81
                    self._errHandler.sync(self)
                    la_ = self._interp.adaptivePredict(self._input,2,self._ctx)
                    if la_ == 1:
                        localctx = RulesParser.BoolExprContext(self, _parentctx, _parentState)
                        self.pushNewRecursionContext(localctx, _startState, self.RULE_boolExpr)
                        self.state = 75
                        if not self.precpred(self._ctx, 17):
                            from antlr4.error.Errors import FailedPredicateException
                            raise FailedPredicateException(self, "self.precpred(self._ctx, 17)")
                        self.state = 76
                        self.match(RulesParser.AND)
                        self.state = 77
                        self.boolExpr(18)
                        pass

                    elif la_ == 2:
                        localctx = RulesParser.BoolExprContext(self, _parentctx, _parentState)
                        self.pushNewRecursionContext(localctx, _startState, self.RULE_boolExpr)
                        self.state = 78
                        if not self.precpred(self._ctx, 16):
                            from antlr4.error.Errors import FailedPredicateException
                            raise FailedPredicateException(self, "self.precpred(self._ctx, 16)")
                        self.state = 79
                        self.match(RulesParser.OR)
                        self.state = 80
                        self.boolExpr(17)
                        pass

             
                self.state = 85
                self._errHandler.sync(self)
                _alt = self._interp.adaptivePredict(self._input,3,self._ctx)

        except RecognitionException as re:
            localctx.exception = re
            self._errHandler.reportError(self, re)
            self._errHandler.recover(self, re)
        finally:
            self.unrollRecursionContexts(_parentctx)
        return localctx


    class ActionsContext(ParserRuleContext):
        __slots__ = 'parser'

        def __init__(self, parser, parent:ParserRuleContext=None, invokingState:int=-1):
            super().__init__(parent, invokingState)
            self.parser = parser

        def action(self, i:int=None):
            if i is None:
                return self.getTypedRuleContexts(RulesParser.ActionContext)
            else:
                return self.getTypedRuleContext(RulesParser.ActionContext,i)


        def getRuleIndex(self):
            return RulesParser.RULE_actions

        def enterRule(self, listener:ParseTreeListener):
            if hasattr( listener, "enterActions" ):
                listener.enterActions(self)

        def exitRule(self, listener:ParseTreeListener):
            if hasattr( listener, "exitActions" ):
                listener.exitActions(self)

        def accept(self, visitor:ParseTreeVisitor):
            if hasattr( visitor, "visitActions" ):
                return visitor.visitActions(self)
            else:
                return visitor.visitChildren(self)




    def actions(self):

        localctx = RulesParser.ActionsContext(self, self._ctx, self.state)
        self.enterRule(localctx, 2, self.RULE_actions)
        self._la = 0 # Token type
        try:
            self.enterOuterAlt(localctx, 1)
            self.state = 86
            self.action()
            self.state = 91
            self._errHandler.sync(self)
            _alt = self._interp.adaptivePredict(self._input,4,self._ctx)
            while _alt!=2 and _alt!=ATN.INVALID_ALT_NUMBER:
                if _alt==1:
                    self.state = 87
                    self.match(RulesParser.T__2)
                    self.state = 88
                    self.action() 
                self.state = 93
                self._errHandler.sync(self)
                _alt = self._interp.adaptivePredict(self._input,4,self._ctx)

            self.state = 95
            self._errHandler.sync(self)
            _la = self._input.LA(1)
            if _la==3:
                self.state = 94
                self.match(RulesParser.T__2)


        except RecognitionException as re:
            localctx.exception = re
            self._errHandler.reportError(self, re)
            self._errHandler.recover(self, re)
        finally:
            self.exitRule()
        return localctx


    class ActionContext(ParserRuleContext):
        __slots__ = 'parser'

        def __init__(self, parser, parent:ParserRuleContext=None, invokingState:int=-1):
            super().__init__(parent, invokingState)
            self.parser = parser


        def getRuleIndex(self):
            return RulesParser.RULE_action

     
        def copyFrom(self, ctx:ParserRuleContext):
            super().copyFrom(ctx)



    class AlterContext(ActionContext):

        def __init__(self, parser, ctx:ParserRuleContext): # actually a RulesParser.ActionContext
            super().__init__(parser)
            self.copyFrom(ctx)

        def REF(self):
            return self.getToken(RulesParser.REF, 0)
        def BINOP(self):
            return self.getToken(RulesParser.BINOP, 0)
        def num(self):
            return self.getTypedRuleContext(RulesParser.NumContext,0)


        def enterRule(self, listener:ParseTreeListener):
            if hasattr( listener, "enterAlter" ):
                listener.enterAlter(self)

        def exitRule(self, listener:ParseTreeListener):
            if hasattr( listener, "exitAlter" ):
                listener.exitAlter(self)

        def accept(self, visitor:ParseTreeVisitor):
            if hasattr( visitor, "visitAlter" ):
                return visitor.visitAlter(self)
            else:
                return visitor.visitChildren(self)


    class CondActionContext(ActionContext):

        def __init__(self, parser, ctx:ParserRuleContext): # actually a RulesParser.ActionContext
            super().__init__(parser)
            self.copyFrom(ctx)

        def IF(self, i:int=None):
            if i is None:
                return self.getTokens(RulesParser.IF)
            else:
                return self.getToken(RulesParser.IF, i)
        def boolExpr(self, i:int=None):
            if i is None:
                return self.getTypedRuleContexts(RulesParser.BoolExprContext)
            else:
                return self.getTypedRuleContext(RulesParser.BoolExprContext,i)

        def actions(self, i:int=None):
            if i is None:
                return self.getTypedRuleContexts(RulesParser.ActionsContext)
            else:
                return self.getTypedRuleContext(RulesParser.ActionsContext,i)

        def ELSE(self, i:int=None):
            if i is None:
                return self.getTokens(RulesParser.ELSE)
            else:
                return self.getToken(RulesParser.ELSE, i)

        def enterRule(self, listener:ParseTreeListener):
            if hasattr( listener, "enterCondAction" ):
                listener.enterCondAction(self)

        def exitRule(self, listener:ParseTreeListener):
            if hasattr( listener, "exitCondAction" ):
                listener.exitCondAction(self)

        def accept(self, visitor:ParseTreeVisitor):
            if hasattr( visitor, "visitCondAction" ):
                return visitor.visitCondAction(self)
            else:
                return visitor.visitChildren(self)


    class SetContext(ActionContext):

        def __init__(self, parser, ctx:ParserRuleContext): # actually a RulesParser.ActionContext
            super().__init__(parser)
            self.copyFrom(ctx)

        def REF(self, i:int=None):
            if i is None:
                return self.getTokens(RulesParser.REF)
            else:
                return self.getToken(RulesParser.REF, i)
        def TRUE(self):
            return self.getToken(RulesParser.TRUE, 0)
        def FALSE(self):
            return self.getToken(RulesParser.FALSE, 0)
        def PLACE(self):
            return self.getToken(RulesParser.PLACE, 0)
        def str_(self):
            return self.getTypedRuleContext(RulesParser.StrContext,0)

        def num(self):
            return self.getTypedRuleContext(RulesParser.NumContext,0)


        def enterRule(self, listener:ParseTreeListener):
            if hasattr( listener, "enterSet" ):
                listener.enterSet(self)

        def exitRule(self, listener:ParseTreeListener):
            if hasattr( listener, "exitSet" ):
                listener.exitSet(self)

        def accept(self, visitor:ParseTreeVisitor):
            if hasattr( visitor, "visitSet" ):
                return visitor.visitSet(self)
            else:
                return visitor.visitChildren(self)


    class ActionHelperContext(ActionContext):

        def __init__(self, parser, ctx:ParserRuleContext): # actually a RulesParser.ActionContext
            super().__init__(parser)
            self.copyFrom(ctx)

        def invoke(self):
            return self.getTypedRuleContext(RulesParser.InvokeContext,0)


        def enterRule(self, listener:ParseTreeListener):
            if hasattr( listener, "enterActionHelper" ):
                listener.enterActionHelper(self)

        def exitRule(self, listener:ParseTreeListener):
            if hasattr( listener, "exitActionHelper" ):
                listener.exitActionHelper(self)

        def accept(self, visitor:ParseTreeVisitor):
            if hasattr( visitor, "visitActionHelper" ):
                return visitor.visitActionHelper(self)
            else:
                return visitor.visitChildren(self)



    def action(self):

        localctx = RulesParser.ActionContext(self, self._ctx, self.state)
        self.enterRule(localctx, 4, self.RULE_action)
        self._la = 0 # Token type
        try:
            self.state = 140
            self._errHandler.sync(self)
            la_ = self._interp.adaptivePredict(self._input,9,self._ctx)
            if la_ == 1:
                localctx = RulesParser.SetContext(self, localctx)
                self.enterOuterAlt(localctx, 1)
                self.state = 97
                self.match(RulesParser.REF)
                self.state = 98
                self.match(RulesParser.T__3)
                self.state = 105
                self._errHandler.sync(self)
                la_ = self._interp.adaptivePredict(self._input,6,self._ctx)
                if la_ == 1:
                    self.state = 99
                    self.match(RulesParser.TRUE)
                    pass

                elif la_ == 2:
                    self.state = 100
                    self.match(RulesParser.FALSE)
                    pass

                elif la_ == 3:
                    self.state = 101
                    self.match(RulesParser.PLACE)
                    pass

                elif la_ == 4:
                    self.state = 102
                    self.match(RulesParser.REF)
                    pass

                elif la_ == 5:
                    self.state = 103
                    self.str_()
                    pass

                elif la_ == 6:
                    self.state = 104
                    self.num()
                    pass


                pass

            elif la_ == 2:
                localctx = RulesParser.AlterContext(self, localctx)
                self.enterOuterAlt(localctx, 2)
                self.state = 107
                self.match(RulesParser.REF)
                self.state = 108
                self.match(RulesParser.BINOP)
                self.state = 109
                self.match(RulesParser.T__3)
                self.state = 110
                self.num()
                pass

            elif la_ == 3:
                localctx = RulesParser.ActionHelperContext(self, localctx)
                self.enterOuterAlt(localctx, 3)
                self.state = 111
                self.invoke()
                pass

            elif la_ == 4:
                localctx = RulesParser.CondActionContext(self, localctx)
                self.enterOuterAlt(localctx, 4)
                self.state = 112
                self.match(RulesParser.IF)
                self.state = 113
                self.match(RulesParser.T__0)
                self.state = 114
                self.boolExpr(0)
                self.state = 115
                self.match(RulesParser.T__1)
                self.state = 116
                self.match(RulesParser.T__4)
                self.state = 117
                self.actions()
                self.state = 118
                self.match(RulesParser.T__5)
                self.state = 130
                self._errHandler.sync(self)
                _alt = self._interp.adaptivePredict(self._input,7,self._ctx)
                while _alt!=2 and _alt!=ATN.INVALID_ALT_NUMBER:
                    if _alt==1:
                        self.state = 119
                        self.match(RulesParser.ELSE)
                        self.state = 120
                        self.match(RulesParser.IF)
                        self.state = 121
                        self.match(RulesParser.T__0)
                        self.state = 122
                        self.boolExpr(0)
                        self.state = 123
                        self.match(RulesParser.T__1)
                        self.state = 124
                        self.match(RulesParser.T__4)
                        self.state = 125
                        self.actions()
                        self.state = 126
                        self.match(RulesParser.T__5) 
                    self.state = 132
                    self._errHandler.sync(self)
                    _alt = self._interp.adaptivePredict(self._input,7,self._ctx)

                self.state = 138
                self._errHandler.sync(self)
                _la = self._input.LA(1)
                if _la==26:
                    self.state = 133
                    self.match(RulesParser.ELSE)
                    self.state = 134
                    self.match(RulesParser.T__4)
                    self.state = 135
                    self.actions()
                    self.state = 136
                    self.match(RulesParser.T__5)


                pass


        except RecognitionException as re:
            localctx.exception = re
            self._errHandler.reportError(self, re)
            self._errHandler.recover(self, re)
        finally:
            self.exitRule()
        return localctx


    class MetaContext(ParserRuleContext):
        __slots__ = 'parser'

        def __init__(self, parser, parent:ParserRuleContext=None, invokingState:int=-1):
            super().__init__(parent, invokingState)
            self.parser = parser

        def FUNC(self):
            return self.getToken(RulesParser.FUNC, 0)

        def LIT(self):
            return self.getToken(RulesParser.LIT, 0)

        def boolExpr(self):
            return self.getTypedRuleContext(RulesParser.BoolExprContext,0)


        def getRuleIndex(self):
            return RulesParser.RULE_meta

        def enterRule(self, listener:ParseTreeListener):
            if hasattr( listener, "enterMeta" ):
                listener.enterMeta(self)

        def exitRule(self, listener:ParseTreeListener):
            if hasattr( listener, "exitMeta" ):
                listener.exitMeta(self)

        def accept(self, visitor:ParseTreeVisitor):
            if hasattr( visitor, "visitMeta" ):
                return visitor.visitMeta(self)
            else:
                return visitor.visitChildren(self)




    def meta(self):

        localctx = RulesParser.MetaContext(self, self._ctx, self.state)
        self.enterRule(localctx, 6, self.RULE_meta)
        try:
            self.state = 154
            self._errHandler.sync(self)
            la_ = self._interp.adaptivePredict(self._input,10,self._ctx)
            if la_ == 1:
                self.enterOuterAlt(localctx, 1)
                self.state = 142
                self.match(RulesParser.FUNC)
                self.state = 143
                self.match(RulesParser.T__0)
                self.state = 144
                self.match(RulesParser.LIT)
                self.state = 145
                self.match(RulesParser.T__6)
                self.state = 146
                self.boolExpr(0)
                self.state = 147
                self.match(RulesParser.T__1)
                pass

            elif la_ == 2:
                self.enterOuterAlt(localctx, 2)
                self.state = 149
                self.match(RulesParser.FUNC)
                self.state = 150
                self.match(RulesParser.T__0)
                self.state = 151
                self.boolExpr(0)
                self.state = 152
                self.match(RulesParser.T__1)
                pass


        except RecognitionException as re:
            localctx.exception = re
            self._errHandler.reportError(self, re)
            self._errHandler.recover(self, re)
        finally:
            self.exitRule()
        return localctx


    class InvokeContext(ParserRuleContext):
        __slots__ = 'parser'

        def __init__(self, parser, parent:ParserRuleContext=None, invokingState:int=-1):
            super().__init__(parent, invokingState)
            self.parser = parser

        def FUNC(self):
            return self.getToken(RulesParser.FUNC, 0)

        def ITEM(self, i:int=None):
            if i is None:
                return self.getTokens(RulesParser.ITEM)
            else:
                return self.getToken(RulesParser.ITEM, i)

        def NOT(self):
            return self.getToken(RulesParser.NOT, 0)

        def value(self):
            return self.getTypedRuleContext(RulesParser.ValueContext,0)


        def LIT(self):
            return self.getToken(RulesParser.LIT, 0)

        def INT(self):
            return self.getToken(RulesParser.INT, 0)

        def FLOAT(self):
            return self.getToken(RulesParser.FLOAT, 0)

        def PLACE(self, i:int=None):
            if i is None:
                return self.getTokens(RulesParser.PLACE)
            else:
                return self.getToken(RulesParser.PLACE, i)

        def REF(self):
            return self.getToken(RulesParser.REF, 0)

        def getRuleIndex(self):
            return RulesParser.RULE_invoke

        def enterRule(self, listener:ParseTreeListener):
            if hasattr( listener, "enterInvoke" ):
                listener.enterInvoke(self)

        def exitRule(self, listener:ParseTreeListener):
            if hasattr( listener, "exitInvoke" ):
                listener.exitInvoke(self)

        def accept(self, visitor:ParseTreeVisitor):
            if hasattr( visitor, "visitInvoke" ):
                return visitor.visitInvoke(self)
            else:
                return visitor.visitChildren(self)




    def invoke(self):

        localctx = RulesParser.InvokeContext(self, self._ctx, self.state)
        self.enterRule(localctx, 8, self.RULE_invoke)
        self._la = 0 # Token type
        try:
            self.state = 228
            self._errHandler.sync(self)
            la_ = self._interp.adaptivePredict(self._input,22,self._ctx)
            if la_ == 1:
                self.enterOuterAlt(localctx, 1)
                self.state = 157
                self._errHandler.sync(self)
                _la = self._input.LA(1)
                if _la==22:
                    self.state = 156
                    self.match(RulesParser.NOT)


                self.state = 159
                self.match(RulesParser.FUNC)
                self.state = 160
                self.match(RulesParser.T__0)
                self.state = 161
                self.match(RulesParser.ITEM)
                self.state = 166
                self._errHandler.sync(self)
                _la = self._input.LA(1)
                while _la==7:
                    self.state = 162
                    self.match(RulesParser.T__6)
                    self.state = 163
                    self.match(RulesParser.ITEM)
                    self.state = 168
                    self._errHandler.sync(self)
                    _la = self._input.LA(1)

                self.state = 169
                self.match(RulesParser.T__1)
                pass

            elif la_ == 2:
                self.enterOuterAlt(localctx, 2)
                self.state = 171
                self._errHandler.sync(self)
                _la = self._input.LA(1)
                if _la==22:
                    self.state = 170
                    self.match(RulesParser.NOT)


                self.state = 173
                self.match(RulesParser.FUNC)
                self.state = 174
                self.match(RulesParser.T__0)
                self.state = 175
                self.value()
                self.state = 176
                self.match(RulesParser.T__1)
                pass

            elif la_ == 3:
                self.enterOuterAlt(localctx, 3)
                self.state = 179
                self._errHandler.sync(self)
                _la = self._input.LA(1)
                if _la==22:
                    self.state = 178
                    self.match(RulesParser.NOT)


                self.state = 181
                self.match(RulesParser.FUNC)
                self.state = 182
                self.match(RulesParser.T__0)
                self.state = 183
                self.match(RulesParser.LIT)
                self.state = 184
                self.match(RulesParser.T__1)
                pass

            elif la_ == 4:
                self.enterOuterAlt(localctx, 4)
                self.state = 186
                self._errHandler.sync(self)
                _la = self._input.LA(1)
                if _la==22:
                    self.state = 185
                    self.match(RulesParser.NOT)


                self.state = 188
                self.match(RulesParser.FUNC)
                self.state = 189
                self.match(RulesParser.T__0)
                self.state = 190
                self.match(RulesParser.INT)
                self.state = 191
                self.match(RulesParser.T__1)
                pass

            elif la_ == 5:
                self.enterOuterAlt(localctx, 5)
                self.state = 193
                self._errHandler.sync(self)
                _la = self._input.LA(1)
                if _la==22:
                    self.state = 192
                    self.match(RulesParser.NOT)


                self.state = 195
                self.match(RulesParser.FUNC)
                self.state = 196
                self.match(RulesParser.T__0)
                self.state = 197
                self.match(RulesParser.FLOAT)
                self.state = 198
                self.match(RulesParser.T__1)
                pass

            elif la_ == 6:
                self.enterOuterAlt(localctx, 6)
                self.state = 200
                self._errHandler.sync(self)
                _la = self._input.LA(1)
                if _la==22:
                    self.state = 199
                    self.match(RulesParser.NOT)


                self.state = 202
                self.match(RulesParser.FUNC)
                self.state = 203
                self.match(RulesParser.T__0)
                self.state = 204
                self.match(RulesParser.PLACE)
                self.state = 209
                self._errHandler.sync(self)
                _la = self._input.LA(1)
                while _la==7:
                    self.state = 205
                    self.match(RulesParser.T__6)
                    self.state = 206
                    self.match(RulesParser.PLACE)
                    self.state = 211
                    self._errHandler.sync(self)
                    _la = self._input.LA(1)

                self.state = 212
                self.match(RulesParser.T__1)
                pass

            elif la_ == 7:
                self.enterOuterAlt(localctx, 7)
                self.state = 214
                self._errHandler.sync(self)
                _la = self._input.LA(1)
                if _la==22:
                    self.state = 213
                    self.match(RulesParser.NOT)


                self.state = 216
                self.match(RulesParser.FUNC)
                self.state = 217
                self.match(RulesParser.T__0)
                self.state = 218
                self.match(RulesParser.REF)
                self.state = 219
                self.match(RulesParser.T__1)
                pass

            elif la_ == 8:
                self.enterOuterAlt(localctx, 8)
                self.state = 221
                self._errHandler.sync(self)
                _la = self._input.LA(1)
                if _la==22:
                    self.state = 220
                    self.match(RulesParser.NOT)


                self.state = 223
                self.match(RulesParser.FUNC)
                self.state = 226
                self._errHandler.sync(self)
                la_ = self._interp.adaptivePredict(self._input,21,self._ctx)
                if la_ == 1:
                    self.state = 224
                    self.match(RulesParser.T__0)
                    self.state = 225
                    self.match(RulesParser.T__1)


                pass


        except RecognitionException as re:
            localctx.exception = re
            self._errHandler.reportError(self, re)
            self._errHandler.recover(self, re)
        finally:
            self.exitRule()
        return localctx


    class CondContext(ParserRuleContext):
        __slots__ = 'parser'

        def __init__(self, parser, parent:ParserRuleContext=None, invokingState:int=-1):
            super().__init__(parent, invokingState)
            self.parser = parser


        def getRuleIndex(self):
            return RulesParser.RULE_cond

     
        def copyFrom(self, ctx:ParserRuleContext):
            super().copyFrom(ctx)



    class IfThenElseContext(CondContext):

        def __init__(self, parser, ctx:ParserRuleContext): # actually a RulesParser.CondContext
            super().__init__(parser)
            self.copyFrom(ctx)

        def IF(self, i:int=None):
            if i is None:
                return self.getTokens(RulesParser.IF)
            else:
                return self.getToken(RulesParser.IF, i)
        def boolExpr(self, i:int=None):
            if i is None:
                return self.getTypedRuleContexts(RulesParser.BoolExprContext)
            else:
                return self.getTypedRuleContext(RulesParser.BoolExprContext,i)

        def ELSE(self, i:int=None):
            if i is None:
                return self.getTokens(RulesParser.ELSE)
            else:
                return self.getToken(RulesParser.ELSE, i)

        def enterRule(self, listener:ParseTreeListener):
            if hasattr( listener, "enterIfThenElse" ):
                listener.enterIfThenElse(self)

        def exitRule(self, listener:ParseTreeListener):
            if hasattr( listener, "exitIfThenElse" ):
                listener.exitIfThenElse(self)

        def accept(self, visitor:ParseTreeVisitor):
            if hasattr( visitor, "visitIfThenElse" ):
                return visitor.visitIfThenElse(self)
            else:
                return visitor.visitChildren(self)


    class PyTernaryContext(CondContext):

        def __init__(self, parser, ctx:ParserRuleContext): # actually a RulesParser.CondContext
            super().__init__(parser)
            self.copyFrom(ctx)

        def boolExpr(self, i:int=None):
            if i is None:
                return self.getTypedRuleContexts(RulesParser.BoolExprContext)
            else:
                return self.getTypedRuleContext(RulesParser.BoolExprContext,i)

        def IF(self):
            return self.getToken(RulesParser.IF, 0)
        def ELSE(self):
            return self.getToken(RulesParser.ELSE, 0)

        def enterRule(self, listener:ParseTreeListener):
            if hasattr( listener, "enterPyTernary" ):
                listener.enterPyTernary(self)

        def exitRule(self, listener:ParseTreeListener):
            if hasattr( listener, "exitPyTernary" ):
                listener.exitPyTernary(self)

        def accept(self, visitor:ParseTreeVisitor):
            if hasattr( visitor, "visitPyTernary" ):
                return visitor.visitPyTernary(self)
            else:
                return visitor.visitChildren(self)



    def cond(self):

        localctx = RulesParser.CondContext(self, self._ctx, self.state)
        self.enterRule(localctx, 10, self.RULE_cond)
        try:
            self.state = 266
            self._errHandler.sync(self)
            token = self._input.LA(1)
            if token in [25]:
                localctx = RulesParser.IfThenElseContext(self, localctx)
                self.enterOuterAlt(localctx, 1)
                self.state = 230
                self.match(RulesParser.IF)
                self.state = 231
                self.match(RulesParser.T__0)
                self.state = 232
                self.boolExpr(0)
                self.state = 233
                self.match(RulesParser.T__1)
                self.state = 234
                self.match(RulesParser.T__4)
                self.state = 235
                self.boolExpr(0)
                self.state = 236
                self.match(RulesParser.T__5)
                self.state = 248
                self._errHandler.sync(self)
                _alt = self._interp.adaptivePredict(self._input,23,self._ctx)
                while _alt!=2 and _alt!=ATN.INVALID_ALT_NUMBER:
                    if _alt==1:
                        self.state = 237
                        self.match(RulesParser.ELSE)
                        self.state = 238
                        self.match(RulesParser.IF)
                        self.state = 239
                        self.match(RulesParser.T__0)
                        self.state = 240
                        self.boolExpr(0)
                        self.state = 241
                        self.match(RulesParser.T__1)
                        self.state = 242
                        self.match(RulesParser.T__4)
                        self.state = 243
                        self.boolExpr(0)
                        self.state = 244
                        self.match(RulesParser.T__5) 
                    self.state = 250
                    self._errHandler.sync(self)
                    _alt = self._interp.adaptivePredict(self._input,23,self._ctx)

                self.state = 256
                self._errHandler.sync(self)
                la_ = self._interp.adaptivePredict(self._input,24,self._ctx)
                if la_ == 1:
                    self.state = 251
                    self.match(RulesParser.ELSE)
                    self.state = 252
                    self.match(RulesParser.T__4)
                    self.state = 253
                    self.boolExpr(0)
                    self.state = 254
                    self.match(RulesParser.T__5)


                pass
            elif token in [1]:
                localctx = RulesParser.PyTernaryContext(self, localctx)
                self.enterOuterAlt(localctx, 2)
                self.state = 258
                self.match(RulesParser.T__0)
                self.state = 259
                self.boolExpr(0)
                self.state = 260
                self.match(RulesParser.IF)
                self.state = 261
                self.boolExpr(0)
                self.state = 262
                self.match(RulesParser.ELSE)
                self.state = 263
                self.boolExpr(0)
                self.state = 264
                self.match(RulesParser.T__1)
                pass
            else:
                raise NoViableAltException(self)

        except RecognitionException as re:
            localctx.exception = re
            self._errHandler.reportError(self, re)
            self._errHandler.recover(self, re)
        finally:
            self.exitRule()
        return localctx


    class CondNumContext(ParserRuleContext):
        __slots__ = 'parser'

        def __init__(self, parser, parent:ParserRuleContext=None, invokingState:int=-1):
            super().__init__(parent, invokingState)
            self.parser = parser

        def IF(self, i:int=None):
            if i is None:
                return self.getTokens(RulesParser.IF)
            else:
                return self.getToken(RulesParser.IF, i)

        def boolExpr(self, i:int=None):
            if i is None:
                return self.getTypedRuleContexts(RulesParser.BoolExprContext)
            else:
                return self.getTypedRuleContext(RulesParser.BoolExprContext,i)


        def num(self, i:int=None):
            if i is None:
                return self.getTypedRuleContexts(RulesParser.NumContext)
            else:
                return self.getTypedRuleContext(RulesParser.NumContext,i)


        def ELSE(self, i:int=None):
            if i is None:
                return self.getTokens(RulesParser.ELSE)
            else:
                return self.getToken(RulesParser.ELSE, i)

        def getRuleIndex(self):
            return RulesParser.RULE_condNum

        def enterRule(self, listener:ParseTreeListener):
            if hasattr( listener, "enterCondNum" ):
                listener.enterCondNum(self)

        def exitRule(self, listener:ParseTreeListener):
            if hasattr( listener, "exitCondNum" ):
                listener.exitCondNum(self)

        def accept(self, visitor:ParseTreeVisitor):
            if hasattr( visitor, "visitCondNum" ):
                return visitor.visitCondNum(self)
            else:
                return visitor.visitChildren(self)




    def condNum(self):

        localctx = RulesParser.CondNumContext(self, self._ctx, self.state)
        self.enterRule(localctx, 12, self.RULE_condNum)
        try:
            self.enterOuterAlt(localctx, 1)
            self.state = 268
            self.match(RulesParser.IF)
            self.state = 269
            self.match(RulesParser.T__0)
            self.state = 270
            self.boolExpr(0)
            self.state = 271
            self.match(RulesParser.T__1)
            self.state = 272
            self.match(RulesParser.T__4)
            self.state = 273
            self.num()
            self.state = 274
            self.match(RulesParser.T__5)
            self.state = 286
            self._errHandler.sync(self)
            _alt = self._interp.adaptivePredict(self._input,26,self._ctx)
            while _alt!=2 and _alt!=ATN.INVALID_ALT_NUMBER:
                if _alt==1:
                    self.state = 275
                    self.match(RulesParser.ELSE)
                    self.state = 276
                    self.match(RulesParser.IF)
                    self.state = 277
                    self.match(RulesParser.T__0)
                    self.state = 278
                    self.boolExpr(0)
                    self.state = 279
                    self.match(RulesParser.T__1)
                    self.state = 280
                    self.match(RulesParser.T__4)
                    self.state = 281
                    self.num()
                    self.state = 282
                    self.match(RulesParser.T__5) 
                self.state = 288
                self._errHandler.sync(self)
                _alt = self._interp.adaptivePredict(self._input,26,self._ctx)

            self.state = 294
            self._errHandler.sync(self)
            la_ = self._interp.adaptivePredict(self._input,27,self._ctx)
            if la_ == 1:
                self.state = 289
                self.match(RulesParser.ELSE)
                self.state = 290
                self.match(RulesParser.T__4)
                self.state = 291
                self.num()
                self.state = 292
                self.match(RulesParser.T__5)


        except RecognitionException as re:
            localctx.exception = re
            self._errHandler.reportError(self, re)
            self._errHandler.recover(self, re)
        finally:
            self.exitRule()
        return localctx


    class CondStrContext(ParserRuleContext):
        __slots__ = 'parser'

        def __init__(self, parser, parent:ParserRuleContext=None, invokingState:int=-1):
            super().__init__(parent, invokingState)
            self.parser = parser

        def IF(self, i:int=None):
            if i is None:
                return self.getTokens(RulesParser.IF)
            else:
                return self.getToken(RulesParser.IF, i)

        def boolExpr(self, i:int=None):
            if i is None:
                return self.getTypedRuleContexts(RulesParser.BoolExprContext)
            else:
                return self.getTypedRuleContext(RulesParser.BoolExprContext,i)


        def str_(self, i:int=None):
            if i is None:
                return self.getTypedRuleContexts(RulesParser.StrContext)
            else:
                return self.getTypedRuleContext(RulesParser.StrContext,i)


        def ELSE(self, i:int=None):
            if i is None:
                return self.getTokens(RulesParser.ELSE)
            else:
                return self.getToken(RulesParser.ELSE, i)

        def getRuleIndex(self):
            return RulesParser.RULE_condStr

        def enterRule(self, listener:ParseTreeListener):
            if hasattr( listener, "enterCondStr" ):
                listener.enterCondStr(self)

        def exitRule(self, listener:ParseTreeListener):
            if hasattr( listener, "exitCondStr" ):
                listener.exitCondStr(self)

        def accept(self, visitor:ParseTreeVisitor):
            if hasattr( visitor, "visitCondStr" ):
                return visitor.visitCondStr(self)
            else:
                return visitor.visitChildren(self)




    def condStr(self):

        localctx = RulesParser.CondStrContext(self, self._ctx, self.state)
        self.enterRule(localctx, 14, self.RULE_condStr)
        self._la = 0 # Token type
        try:
            self.enterOuterAlt(localctx, 1)
            self.state = 296
            self.match(RulesParser.IF)
            self.state = 297
            self.match(RulesParser.T__0)
            self.state = 298
            self.boolExpr(0)
            self.state = 299
            self.match(RulesParser.T__1)
            self.state = 300
            self.match(RulesParser.T__4)
            self.state = 301
            self.str_()
            self.state = 302
            self.match(RulesParser.T__5)
            self.state = 314
            self._errHandler.sync(self)
            _alt = self._interp.adaptivePredict(self._input,28,self._ctx)
            while _alt!=2 and _alt!=ATN.INVALID_ALT_NUMBER:
                if _alt==1:
                    self.state = 303
                    self.match(RulesParser.ELSE)
                    self.state = 304
                    self.match(RulesParser.IF)
                    self.state = 305
                    self.match(RulesParser.T__0)
                    self.state = 306
                    self.boolExpr(0)
                    self.state = 307
                    self.match(RulesParser.T__1)
                    self.state = 308
                    self.match(RulesParser.T__4)
                    self.state = 309
                    self.str_()
                    self.state = 310
                    self.match(RulesParser.T__5) 
                self.state = 316
                self._errHandler.sync(self)
                _alt = self._interp.adaptivePredict(self._input,28,self._ctx)

            self.state = 322
            self._errHandler.sync(self)
            _la = self._input.LA(1)
            if _la==26:
                self.state = 317
                self.match(RulesParser.ELSE)
                self.state = 318
                self.match(RulesParser.T__4)
                self.state = 319
                self.str_()
                self.state = 320
                self.match(RulesParser.T__5)


        except RecognitionException as re:
            localctx.exception = re
            self._errHandler.reportError(self, re)
            self._errHandler.recover(self, re)
        finally:
            self.exitRule()
        return localctx


    class SwitchBoolContext(ParserRuleContext):
        __slots__ = 'parser'

        def __init__(self, parser, parent:ParserRuleContext=None, invokingState:int=-1):
            super().__init__(parent, invokingState)
            self.parser = parser


        def getRuleIndex(self):
            return RulesParser.RULE_switchBool

     
        def copyFrom(self, ctx:ParserRuleContext):
            super().copyFrom(ctx)



    class PerItemBoolContext(SwitchBoolContext):

        def __init__(self, parser, ctx:ParserRuleContext): # actually a RulesParser.SwitchBoolContext
            super().__init__(parser)
            self.copyFrom(ctx)

        def PER(self):
            return self.getToken(RulesParser.PER, 0)
        def ITEM(self):
            return self.getToken(RulesParser.ITEM, 0)
        def boolExpr(self, i:int=None):
            if i is None:
                return self.getTypedRuleContexts(RulesParser.BoolExprContext)
            else:
                return self.getTypedRuleContext(RulesParser.BoolExprContext,i)

        def INT(self, i:int=None):
            if i is None:
                return self.getTokens(RulesParser.INT)
            else:
                return self.getToken(RulesParser.INT, i)

        def enterRule(self, listener:ParseTreeListener):
            if hasattr( listener, "enterPerItemBool" ):
                listener.enterPerItemBool(self)

        def exitRule(self, listener:ParseTreeListener):
            if hasattr( listener, "exitPerItemBool" ):
                listener.exitPerItemBool(self)

        def accept(self, visitor:ParseTreeVisitor):
            if hasattr( visitor, "visitPerItemBool" ):
                return visitor.visitPerItemBool(self)
            else:
                return visitor.visitChildren(self)


    class RefStrInListContext(SwitchBoolContext):

        def __init__(self, parser, ctx:ParserRuleContext): # actually a RulesParser.SwitchBoolContext
            super().__init__(parser)
            self.copyFrom(ctx)

        def REF(self):
            return self.getToken(RulesParser.REF, 0)
        def IN(self):
            return self.getToken(RulesParser.IN, 0)
        def LIT(self, i:int=None):
            if i is None:
                return self.getTokens(RulesParser.LIT)
            else:
                return self.getToken(RulesParser.LIT, i)

        def enterRule(self, listener:ParseTreeListener):
            if hasattr( listener, "enterRefStrInList" ):
                listener.enterRefStrInList(self)

        def exitRule(self, listener:ParseTreeListener):
            if hasattr( listener, "exitRefStrInList" ):
                listener.exitRefStrInList(self)

        def accept(self, visitor:ParseTreeVisitor):
            if hasattr( visitor, "visitRefStrInList" ):
                return visitor.visitRefStrInList(self)
            else:
                return visitor.visitChildren(self)


    class RefInListContext(SwitchBoolContext):

        def __init__(self, parser, ctx:ParserRuleContext): # actually a RulesParser.SwitchBoolContext
            super().__init__(parser)
            self.copyFrom(ctx)

        def REF(self):
            return self.getToken(RulesParser.REF, 0)
        def IN(self):
            return self.getToken(RulesParser.IN, 0)
        def ITEM(self, i:int=None):
            if i is None:
                return self.getTokens(RulesParser.ITEM)
            else:
                return self.getToken(RulesParser.ITEM, i)

        def enterRule(self, listener:ParseTreeListener):
            if hasattr( listener, "enterRefInList" ):
                listener.enterRefInList(self)

        def exitRule(self, listener:ParseTreeListener):
            if hasattr( listener, "exitRefInList" ):
                listener.exitRefInList(self)

        def accept(self, visitor:ParseTreeVisitor):
            if hasattr( visitor, "visitRefInList" ):
                return visitor.visitRefInList(self)
            else:
                return visitor.visitChildren(self)


    class PerSettingBoolContext(SwitchBoolContext):

        def __init__(self, parser, ctx:ParserRuleContext): # actually a RulesParser.SwitchBoolContext
            super().__init__(parser)
            self.copyFrom(ctx)

        def PER(self):
            return self.getToken(RulesParser.PER, 0)
        def SETTING(self):
            return self.getToken(RulesParser.SETTING, 0)
        def boolExpr(self, i:int=None):
            if i is None:
                return self.getTypedRuleContexts(RulesParser.BoolExprContext)
            else:
                return self.getTypedRuleContext(RulesParser.BoolExprContext,i)

        def INT(self, i:int=None):
            if i is None:
                return self.getTokens(RulesParser.INT)
            else:
                return self.getToken(RulesParser.INT, i)
        def LIT(self, i:int=None):
            if i is None:
                return self.getTokens(RulesParser.LIT)
            else:
                return self.getToken(RulesParser.LIT, i)

        def enterRule(self, listener:ParseTreeListener):
            if hasattr( listener, "enterPerSettingBool" ):
                listener.enterPerSettingBool(self)

        def exitRule(self, listener:ParseTreeListener):
            if hasattr( listener, "exitPerSettingBool" ):
                listener.exitPerSettingBool(self)

        def accept(self, visitor:ParseTreeVisitor):
            if hasattr( visitor, "visitPerSettingBool" ):
                return visitor.visitPerSettingBool(self)
            else:
                return visitor.visitChildren(self)


    class MatchRefBoolContext(SwitchBoolContext):

        def __init__(self, parser, ctx:ParserRuleContext): # actually a RulesParser.SwitchBoolContext
            super().__init__(parser)
            self.copyFrom(ctx)

        def PER(self):
            return self.getToken(RulesParser.PER, 0)
        def REF(self):
            return self.getToken(RulesParser.REF, 0)
        def boolExpr(self, i:int=None):
            if i is None:
                return self.getTypedRuleContexts(RulesParser.BoolExprContext)
            else:
                return self.getTypedRuleContext(RulesParser.BoolExprContext,i)

        def ITEM(self, i:int=None):
            if i is None:
                return self.getTokens(RulesParser.ITEM)
            else:
                return self.getToken(RulesParser.ITEM, i)

        def enterRule(self, listener:ParseTreeListener):
            if hasattr( listener, "enterMatchRefBool" ):
                listener.enterMatchRefBool(self)

        def exitRule(self, listener:ParseTreeListener):
            if hasattr( listener, "exitMatchRefBool" ):
                listener.exitMatchRefBool(self)

        def accept(self, visitor:ParseTreeVisitor):
            if hasattr( visitor, "visitMatchRefBool" ):
                return visitor.visitMatchRefBool(self)
            else:
                return visitor.visitChildren(self)



    def switchBool(self):

        localctx = RulesParser.SwitchBoolContext(self, self._ctx, self.state)
        self.enterRule(localctx, 16, self.RULE_switchBool)
        self._la = 0 # Token type
        try:
            self.state = 424
            self._errHandler.sync(self)
            la_ = self._interp.adaptivePredict(self._input,41,self._ctx)
            if la_ == 1:
                localctx = RulesParser.PerItemBoolContext(self, localctx)
                self.enterOuterAlt(localctx, 1)
                self.state = 324
                self.match(RulesParser.PER)
                self.state = 325
                self.match(RulesParser.ITEM)
                self.state = 326
                self.match(RulesParser.T__4)
                self.state = 332 
                self._errHandler.sync(self)
                _la = self._input.LA(1)
                while True:
                    self.state = 327
                    self.match(RulesParser.INT)
                    self.state = 328
                    self.match(RulesParser.T__7)
                    self.state = 329
                    self.boolExpr(0)
                    self.state = 330
                    self.match(RulesParser.T__6)
                    self.state = 334 
                    self._errHandler.sync(self)
                    _la = self._input.LA(1)
                    if not (_la==37):
                        break

                self.state = 336
                self.match(RulesParser.T__8)
                self.state = 337
                self.match(RulesParser.T__7)
                self.state = 338
                self.boolExpr(0)
                self.state = 340
                self._errHandler.sync(self)
                _la = self._input.LA(1)
                if _la==7:
                    self.state = 339
                    self.match(RulesParser.T__6)


                self.state = 342
                self.match(RulesParser.T__5)
                pass

            elif la_ == 2:
                localctx = RulesParser.PerSettingBoolContext(self, localctx)
                self.enterOuterAlt(localctx, 2)
                self.state = 344
                self.match(RulesParser.PER)
                self.state = 345
                self.match(RulesParser.SETTING)
                self.state = 346
                self.match(RulesParser.T__4)
                self.state = 365
                self._errHandler.sync(self)
                token = self._input.LA(1)
                if token in [37]:
                    self.state = 352 
                    self._errHandler.sync(self)
                    _la = self._input.LA(1)
                    while True:
                        self.state = 347
                        self.match(RulesParser.INT)
                        self.state = 348
                        self.match(RulesParser.T__7)
                        self.state = 349
                        self.boolExpr(0)
                        self.state = 350
                        self.match(RulesParser.T__6)
                        self.state = 354 
                        self._errHandler.sync(self)
                        _la = self._input.LA(1)
                        if not (_la==37):
                            break

                    pass
                elif token in [35]:
                    self.state = 361 
                    self._errHandler.sync(self)
                    _la = self._input.LA(1)
                    while True:
                        self.state = 356
                        self.match(RulesParser.LIT)
                        self.state = 357
                        self.match(RulesParser.T__7)
                        self.state = 358
                        self.boolExpr(0)
                        self.state = 359
                        self.match(RulesParser.T__6)
                        self.state = 363 
                        self._errHandler.sync(self)
                        _la = self._input.LA(1)
                        if not (_la==35):
                            break

                    pass
                else:
                    raise NoViableAltException(self)

                self.state = 367
                self.match(RulesParser.T__8)
                self.state = 368
                self.match(RulesParser.T__7)
                self.state = 369
                self.boolExpr(0)
                self.state = 371
                self._errHandler.sync(self)
                _la = self._input.LA(1)
                if _la==7:
                    self.state = 370
                    self.match(RulesParser.T__6)


                self.state = 373
                self.match(RulesParser.T__5)
                pass

            elif la_ == 3:
                localctx = RulesParser.MatchRefBoolContext(self, localctx)
                self.enterOuterAlt(localctx, 3)
                self.state = 375
                self.match(RulesParser.PER)
                self.state = 376
                self.match(RulesParser.REF)
                self.state = 377
                self.match(RulesParser.T__4)
                self.state = 390 
                self._errHandler.sync(self)
                _la = self._input.LA(1)
                while True:
                    self.state = 378
                    self.match(RulesParser.ITEM)
                    self.state = 383
                    self._errHandler.sync(self)
                    _la = self._input.LA(1)
                    while _la==10:
                        self.state = 379
                        self.match(RulesParser.T__9)
                        self.state = 380
                        self.match(RulesParser.ITEM)
                        self.state = 385
                        self._errHandler.sync(self)
                        _la = self._input.LA(1)

                    self.state = 386
                    self.match(RulesParser.T__7)
                    self.state = 387
                    self.boolExpr(0)
                    self.state = 388
                    self.match(RulesParser.T__6)
                    self.state = 392 
                    self._errHandler.sync(self)
                    _la = self._input.LA(1)
                    if not (_la==30):
                        break

                self.state = 394
                self.match(RulesParser.T__8)
                self.state = 395
                self.match(RulesParser.T__7)
                self.state = 396
                self.boolExpr(0)
                self.state = 398
                self._errHandler.sync(self)
                _la = self._input.LA(1)
                if _la==7:
                    self.state = 397
                    self.match(RulesParser.T__6)


                self.state = 400
                self.match(RulesParser.T__5)
                pass

            elif la_ == 4:
                localctx = RulesParser.RefInListContext(self, localctx)
                self.enterOuterAlt(localctx, 4)
                self.state = 402
                self.match(RulesParser.REF)
                self.state = 403
                self.match(RulesParser.IN)
                self.state = 404
                self.match(RulesParser.T__10)
                self.state = 405
                self.match(RulesParser.ITEM)
                self.state = 408 
                self._errHandler.sync(self)
                _la = self._input.LA(1)
                while True:
                    self.state = 406
                    self.match(RulesParser.T__6)
                    self.state = 407
                    self.match(RulesParser.ITEM)
                    self.state = 410 
                    self._errHandler.sync(self)
                    _la = self._input.LA(1)
                    if not (_la==7):
                        break

                self.state = 412
                self.match(RulesParser.T__11)
                pass

            elif la_ == 5:
                localctx = RulesParser.RefStrInListContext(self, localctx)
                self.enterOuterAlt(localctx, 5)
                self.state = 413
                self.match(RulesParser.REF)
                self.state = 414
                self.match(RulesParser.IN)
                self.state = 415
                self.match(RulesParser.T__10)
                self.state = 416
                self.match(RulesParser.LIT)
                self.state = 419 
                self._errHandler.sync(self)
                _la = self._input.LA(1)
                while True:
                    self.state = 417
                    self.match(RulesParser.T__6)
                    self.state = 418
                    self.match(RulesParser.LIT)
                    self.state = 421 
                    self._errHandler.sync(self)
                    _la = self._input.LA(1)
                    if not (_la==7):
                        break

                self.state = 423
                self.match(RulesParser.T__11)
                pass


        except RecognitionException as re:
            localctx.exception = re
            self._errHandler.reportError(self, re)
            self._errHandler.recover(self, re)
        finally:
            self.exitRule()
        return localctx


    class SwitchNumContext(ParserRuleContext):
        __slots__ = 'parser'

        def __init__(self, parser, parent:ParserRuleContext=None, invokingState:int=-1):
            super().__init__(parent, invokingState)
            self.parser = parser


        def getRuleIndex(self):
            return RulesParser.RULE_switchNum

     
        def copyFrom(self, ctx:ParserRuleContext):
            super().copyFrom(ctx)



    class PerRefIntContext(SwitchNumContext):

        def __init__(self, parser, ctx:ParserRuleContext): # actually a RulesParser.SwitchNumContext
            super().__init__(parser)
            self.copyFrom(ctx)

        def PER(self):
            return self.getToken(RulesParser.PER, 0)
        def REF(self):
            return self.getToken(RulesParser.REF, 0)
        def num(self, i:int=None):
            if i is None:
                return self.getTypedRuleContexts(RulesParser.NumContext)
            else:
                return self.getTypedRuleContext(RulesParser.NumContext,i)

        def INT(self, i:int=None):
            if i is None:
                return self.getTokens(RulesParser.INT)
            else:
                return self.getToken(RulesParser.INT, i)
        def LIT(self, i:int=None):
            if i is None:
                return self.getTokens(RulesParser.LIT)
            else:
                return self.getToken(RulesParser.LIT, i)

        def enterRule(self, listener:ParseTreeListener):
            if hasattr( listener, "enterPerRefInt" ):
                listener.enterPerRefInt(self)

        def exitRule(self, listener:ParseTreeListener):
            if hasattr( listener, "exitPerRefInt" ):
                listener.exitPerRefInt(self)

        def accept(self, visitor:ParseTreeVisitor):
            if hasattr( visitor, "visitPerRefInt" ):
                return visitor.visitPerRefInt(self)
            else:
                return visitor.visitChildren(self)


    class PerSettingIntContext(SwitchNumContext):

        def __init__(self, parser, ctx:ParserRuleContext): # actually a RulesParser.SwitchNumContext
            super().__init__(parser)
            self.copyFrom(ctx)

        def PER(self):
            return self.getToken(RulesParser.PER, 0)
        def SETTING(self):
            return self.getToken(RulesParser.SETTING, 0)
        def num(self, i:int=None):
            if i is None:
                return self.getTypedRuleContexts(RulesParser.NumContext)
            else:
                return self.getTypedRuleContext(RulesParser.NumContext,i)

        def INT(self, i:int=None):
            if i is None:
                return self.getTokens(RulesParser.INT)
            else:
                return self.getToken(RulesParser.INT, i)
        def LIT(self, i:int=None):
            if i is None:
                return self.getTokens(RulesParser.LIT)
            else:
                return self.getToken(RulesParser.LIT, i)

        def enterRule(self, listener:ParseTreeListener):
            if hasattr( listener, "enterPerSettingInt" ):
                listener.enterPerSettingInt(self)

        def exitRule(self, listener:ParseTreeListener):
            if hasattr( listener, "exitPerSettingInt" ):
                listener.exitPerSettingInt(self)

        def accept(self, visitor:ParseTreeVisitor):
            if hasattr( visitor, "visitPerSettingInt" ):
                return visitor.visitPerSettingInt(self)
            else:
                return visitor.visitChildren(self)


    class PerItemIntContext(SwitchNumContext):

        def __init__(self, parser, ctx:ParserRuleContext): # actually a RulesParser.SwitchNumContext
            super().__init__(parser)
            self.copyFrom(ctx)

        def PER(self):
            return self.getToken(RulesParser.PER, 0)
        def ITEM(self):
            return self.getToken(RulesParser.ITEM, 0)
        def num(self, i:int=None):
            if i is None:
                return self.getTypedRuleContexts(RulesParser.NumContext)
            else:
                return self.getTypedRuleContext(RulesParser.NumContext,i)

        def INT(self, i:int=None):
            if i is None:
                return self.getTokens(RulesParser.INT)
            else:
                return self.getToken(RulesParser.INT, i)

        def enterRule(self, listener:ParseTreeListener):
            if hasattr( listener, "enterPerItemInt" ):
                listener.enterPerItemInt(self)

        def exitRule(self, listener:ParseTreeListener):
            if hasattr( listener, "exitPerItemInt" ):
                listener.exitPerItemInt(self)

        def accept(self, visitor:ParseTreeVisitor):
            if hasattr( visitor, "visitPerItemInt" ):
                return visitor.visitPerItemInt(self)
            else:
                return visitor.visitChildren(self)



    def switchNum(self):

        localctx = RulesParser.SwitchNumContext(self, self._ctx, self.state)
        self.enterRule(localctx, 18, self.RULE_switchNum)
        self._la = 0 # Token type
        try:
            self.state = 508
            self._errHandler.sync(self)
            la_ = self._interp.adaptivePredict(self._input,52,self._ctx)
            if la_ == 1:
                localctx = RulesParser.PerItemIntContext(self, localctx)
                self.enterOuterAlt(localctx, 1)
                self.state = 426
                self.match(RulesParser.PER)
                self.state = 427
                self.match(RulesParser.ITEM)
                self.state = 428
                self.match(RulesParser.T__4)
                self.state = 434 
                self._errHandler.sync(self)
                _la = self._input.LA(1)
                while True:
                    self.state = 429
                    self.match(RulesParser.INT)
                    self.state = 430
                    self.match(RulesParser.T__7)
                    self.state = 431
                    self.num()
                    self.state = 432
                    self.match(RulesParser.T__6)
                    self.state = 436 
                    self._errHandler.sync(self)
                    _la = self._input.LA(1)
                    if not (_la==37):
                        break

                self.state = 438
                self.match(RulesParser.T__8)
                self.state = 439
                self.match(RulesParser.T__7)
                self.state = 440
                self.num()
                self.state = 442
                self._errHandler.sync(self)
                _la = self._input.LA(1)
                if _la==7:
                    self.state = 441
                    self.match(RulesParser.T__6)


                self.state = 444
                self.match(RulesParser.T__5)
                pass

            elif la_ == 2:
                localctx = RulesParser.PerRefIntContext(self, localctx)
                self.enterOuterAlt(localctx, 2)
                self.state = 446
                self.match(RulesParser.PER)
                self.state = 447
                self.match(RulesParser.REF)
                self.state = 448
                self.match(RulesParser.T__4)
                self.state = 467
                self._errHandler.sync(self)
                token = self._input.LA(1)
                if token in [37]:
                    self.state = 454 
                    self._errHandler.sync(self)
                    _la = self._input.LA(1)
                    while True:
                        self.state = 449
                        self.match(RulesParser.INT)
                        self.state = 450
                        self.match(RulesParser.T__7)
                        self.state = 451
                        self.num()
                        self.state = 452
                        self.match(RulesParser.T__6)
                        self.state = 456 
                        self._errHandler.sync(self)
                        _la = self._input.LA(1)
                        if not (_la==37):
                            break

                    pass
                elif token in [35]:
                    self.state = 463 
                    self._errHandler.sync(self)
                    _la = self._input.LA(1)
                    while True:
                        self.state = 458
                        self.match(RulesParser.LIT)
                        self.state = 459
                        self.match(RulesParser.T__7)
                        self.state = 460
                        self.num()
                        self.state = 461
                        self.match(RulesParser.T__6)
                        self.state = 465 
                        self._errHandler.sync(self)
                        _la = self._input.LA(1)
                        if not (_la==35):
                            break

                    pass
                else:
                    raise NoViableAltException(self)

                self.state = 469
                self.match(RulesParser.T__8)
                self.state = 470
                self.match(RulesParser.T__7)
                self.state = 471
                self.num()
                self.state = 473
                self._errHandler.sync(self)
                _la = self._input.LA(1)
                if _la==7:
                    self.state = 472
                    self.match(RulesParser.T__6)


                self.state = 475
                self.match(RulesParser.T__5)
                pass

            elif la_ == 3:
                localctx = RulesParser.PerSettingIntContext(self, localctx)
                self.enterOuterAlt(localctx, 3)
                self.state = 477
                self.match(RulesParser.PER)
                self.state = 478
                self.match(RulesParser.SETTING)
                self.state = 479
                self.match(RulesParser.T__4)
                self.state = 498
                self._errHandler.sync(self)
                token = self._input.LA(1)
                if token in [37]:
                    self.state = 485 
                    self._errHandler.sync(self)
                    _la = self._input.LA(1)
                    while True:
                        self.state = 480
                        self.match(RulesParser.INT)
                        self.state = 481
                        self.match(RulesParser.T__7)
                        self.state = 482
                        self.num()
                        self.state = 483
                        self.match(RulesParser.T__6)
                        self.state = 487 
                        self._errHandler.sync(self)
                        _la = self._input.LA(1)
                        if not (_la==37):
                            break

                    pass
                elif token in [35]:
                    self.state = 494 
                    self._errHandler.sync(self)
                    _la = self._input.LA(1)
                    while True:
                        self.state = 489
                        self.match(RulesParser.LIT)
                        self.state = 490
                        self.match(RulesParser.T__7)
                        self.state = 491
                        self.num()
                        self.state = 492
                        self.match(RulesParser.T__6)
                        self.state = 496 
                        self._errHandler.sync(self)
                        _la = self._input.LA(1)
                        if not (_la==35):
                            break

                    pass
                else:
                    raise NoViableAltException(self)

                self.state = 500
                self.match(RulesParser.T__8)
                self.state = 501
                self.match(RulesParser.T__7)
                self.state = 502
                self.num()
                self.state = 504
                self._errHandler.sync(self)
                _la = self._input.LA(1)
                if _la==7:
                    self.state = 503
                    self.match(RulesParser.T__6)


                self.state = 506
                self.match(RulesParser.T__5)
                pass


        except RecognitionException as re:
            localctx.exception = re
            self._errHandler.reportError(self, re)
            self._errHandler.recover(self, re)
        finally:
            self.exitRule()
        return localctx


    class SwitchStrContext(ParserRuleContext):
        __slots__ = 'parser'

        def __init__(self, parser, parent:ParserRuleContext=None, invokingState:int=-1):
            super().__init__(parent, invokingState)
            self.parser = parser


        def getRuleIndex(self):
            return RulesParser.RULE_switchStr

     
        def copyFrom(self, ctx:ParserRuleContext):
            super().copyFrom(ctx)



    class PerItemStrContext(SwitchStrContext):

        def __init__(self, parser, ctx:ParserRuleContext): # actually a RulesParser.SwitchStrContext
            super().__init__(parser)
            self.copyFrom(ctx)

        def PER(self):
            return self.getToken(RulesParser.PER, 0)
        def ITEM(self):
            return self.getToken(RulesParser.ITEM, 0)
        def str_(self, i:int=None):
            if i is None:
                return self.getTypedRuleContexts(RulesParser.StrContext)
            else:
                return self.getTypedRuleContext(RulesParser.StrContext,i)

        def INT(self, i:int=None):
            if i is None:
                return self.getTokens(RulesParser.INT)
            else:
                return self.getToken(RulesParser.INT, i)

        def enterRule(self, listener:ParseTreeListener):
            if hasattr( listener, "enterPerItemStr" ):
                listener.enterPerItemStr(self)

        def exitRule(self, listener:ParseTreeListener):
            if hasattr( listener, "exitPerItemStr" ):
                listener.exitPerItemStr(self)

        def accept(self, visitor:ParseTreeVisitor):
            if hasattr( visitor, "visitPerItemStr" ):
                return visitor.visitPerItemStr(self)
            else:
                return visitor.visitChildren(self)


    class PerRefStrContext(SwitchStrContext):

        def __init__(self, parser, ctx:ParserRuleContext): # actually a RulesParser.SwitchStrContext
            super().__init__(parser)
            self.copyFrom(ctx)

        def PER(self):
            return self.getToken(RulesParser.PER, 0)
        def REF(self):
            return self.getToken(RulesParser.REF, 0)
        def str_(self, i:int=None):
            if i is None:
                return self.getTypedRuleContexts(RulesParser.StrContext)
            else:
                return self.getTypedRuleContext(RulesParser.StrContext,i)

        def INT(self, i:int=None):
            if i is None:
                return self.getTokens(RulesParser.INT)
            else:
                return self.getToken(RulesParser.INT, i)
        def LIT(self, i:int=None):
            if i is None:
                return self.getTokens(RulesParser.LIT)
            else:
                return self.getToken(RulesParser.LIT, i)

        def enterRule(self, listener:ParseTreeListener):
            if hasattr( listener, "enterPerRefStr" ):
                listener.enterPerRefStr(self)

        def exitRule(self, listener:ParseTreeListener):
            if hasattr( listener, "exitPerRefStr" ):
                listener.exitPerRefStr(self)

        def accept(self, visitor:ParseTreeVisitor):
            if hasattr( visitor, "visitPerRefStr" ):
                return visitor.visitPerRefStr(self)
            else:
                return visitor.visitChildren(self)


    class PerSettingStrContext(SwitchStrContext):

        def __init__(self, parser, ctx:ParserRuleContext): # actually a RulesParser.SwitchStrContext
            super().__init__(parser)
            self.copyFrom(ctx)

        def PER(self):
            return self.getToken(RulesParser.PER, 0)
        def SETTING(self):
            return self.getToken(RulesParser.SETTING, 0)
        def str_(self, i:int=None):
            if i is None:
                return self.getTypedRuleContexts(RulesParser.StrContext)
            else:
                return self.getTypedRuleContext(RulesParser.StrContext,i)

        def INT(self, i:int=None):
            if i is None:
                return self.getTokens(RulesParser.INT)
            else:
                return self.getToken(RulesParser.INT, i)
        def LIT(self, i:int=None):
            if i is None:
                return self.getTokens(RulesParser.LIT)
            else:
                return self.getToken(RulesParser.LIT, i)

        def enterRule(self, listener:ParseTreeListener):
            if hasattr( listener, "enterPerSettingStr" ):
                listener.enterPerSettingStr(self)

        def exitRule(self, listener:ParseTreeListener):
            if hasattr( listener, "exitPerSettingStr" ):
                listener.exitPerSettingStr(self)

        def accept(self, visitor:ParseTreeVisitor):
            if hasattr( visitor, "visitPerSettingStr" ):
                return visitor.visitPerSettingStr(self)
            else:
                return visitor.visitChildren(self)



    def switchStr(self):

        localctx = RulesParser.SwitchStrContext(self, self._ctx, self.state)
        self.enterRule(localctx, 20, self.RULE_switchStr)
        self._la = 0 # Token type
        try:
            self.state = 592
            self._errHandler.sync(self)
            la_ = self._interp.adaptivePredict(self._input,63,self._ctx)
            if la_ == 1:
                localctx = RulesParser.PerItemStrContext(self, localctx)
                self.enterOuterAlt(localctx, 1)
                self.state = 510
                self.match(RulesParser.PER)
                self.state = 511
                self.match(RulesParser.ITEM)
                self.state = 512
                self.match(RulesParser.T__4)
                self.state = 518 
                self._errHandler.sync(self)
                _la = self._input.LA(1)
                while True:
                    self.state = 513
                    self.match(RulesParser.INT)
                    self.state = 514
                    self.match(RulesParser.T__7)
                    self.state = 515
                    self.str_()
                    self.state = 516
                    self.match(RulesParser.T__6)
                    self.state = 520 
                    self._errHandler.sync(self)
                    _la = self._input.LA(1)
                    if not (_la==37):
                        break

                self.state = 522
                self.match(RulesParser.T__8)
                self.state = 523
                self.match(RulesParser.T__7)
                self.state = 524
                self.str_()
                self.state = 526
                self._errHandler.sync(self)
                _la = self._input.LA(1)
                if _la==7:
                    self.state = 525
                    self.match(RulesParser.T__6)


                self.state = 528
                self.match(RulesParser.T__5)
                pass

            elif la_ == 2:
                localctx = RulesParser.PerRefStrContext(self, localctx)
                self.enterOuterAlt(localctx, 2)
                self.state = 530
                self.match(RulesParser.PER)
                self.state = 531
                self.match(RulesParser.REF)
                self.state = 532
                self.match(RulesParser.T__4)
                self.state = 551
                self._errHandler.sync(self)
                token = self._input.LA(1)
                if token in [37]:
                    self.state = 538 
                    self._errHandler.sync(self)
                    _la = self._input.LA(1)
                    while True:
                        self.state = 533
                        self.match(RulesParser.INT)
                        self.state = 534
                        self.match(RulesParser.T__7)
                        self.state = 535
                        self.str_()
                        self.state = 536
                        self.match(RulesParser.T__6)
                        self.state = 540 
                        self._errHandler.sync(self)
                        _la = self._input.LA(1)
                        if not (_la==37):
                            break

                    pass
                elif token in [35]:
                    self.state = 547 
                    self._errHandler.sync(self)
                    _la = self._input.LA(1)
                    while True:
                        self.state = 542
                        self.match(RulesParser.LIT)
                        self.state = 543
                        self.match(RulesParser.T__7)
                        self.state = 544
                        self.str_()
                        self.state = 545
                        self.match(RulesParser.T__6)
                        self.state = 549 
                        self._errHandler.sync(self)
                        _la = self._input.LA(1)
                        if not (_la==35):
                            break

                    pass
                else:
                    raise NoViableAltException(self)

                self.state = 553
                self.match(RulesParser.T__8)
                self.state = 554
                self.match(RulesParser.T__7)
                self.state = 555
                self.str_()
                self.state = 557
                self._errHandler.sync(self)
                _la = self._input.LA(1)
                if _la==7:
                    self.state = 556
                    self.match(RulesParser.T__6)


                self.state = 559
                self.match(RulesParser.T__5)
                pass

            elif la_ == 3:
                localctx = RulesParser.PerSettingStrContext(self, localctx)
                self.enterOuterAlt(localctx, 3)
                self.state = 561
                self.match(RulesParser.PER)
                self.state = 562
                self.match(RulesParser.SETTING)
                self.state = 563
                self.match(RulesParser.T__4)
                self.state = 582
                self._errHandler.sync(self)
                token = self._input.LA(1)
                if token in [37]:
                    self.state = 569 
                    self._errHandler.sync(self)
                    _la = self._input.LA(1)
                    while True:
                        self.state = 564
                        self.match(RulesParser.INT)
                        self.state = 565
                        self.match(RulesParser.T__7)
                        self.state = 566
                        self.str_()
                        self.state = 567
                        self.match(RulesParser.T__6)
                        self.state = 571 
                        self._errHandler.sync(self)
                        _la = self._input.LA(1)
                        if not (_la==37):
                            break

                    pass
                elif token in [35]:
                    self.state = 578 
                    self._errHandler.sync(self)
                    _la = self._input.LA(1)
                    while True:
                        self.state = 573
                        self.match(RulesParser.LIT)
                        self.state = 574
                        self.match(RulesParser.T__7)
                        self.state = 575
                        self.str_()
                        self.state = 576
                        self.match(RulesParser.T__6)
                        self.state = 580 
                        self._errHandler.sync(self)
                        _la = self._input.LA(1)
                        if not (_la==35):
                            break

                    pass
                else:
                    raise NoViableAltException(self)

                self.state = 584
                self.match(RulesParser.T__8)
                self.state = 585
                self.match(RulesParser.T__7)
                self.state = 586
                self.str_()
                self.state = 588
                self._errHandler.sync(self)
                _la = self._input.LA(1)
                if _la==7:
                    self.state = 587
                    self.match(RulesParser.T__6)


                self.state = 590
                self.match(RulesParser.T__5)
                pass


        except RecognitionException as re:
            localctx.exception = re
            self._errHandler.reportError(self, re)
            self._errHandler.recover(self, re)
        finally:
            self.exitRule()
        return localctx


    class CmpContext(ParserRuleContext):
        __slots__ = 'parser'

        def __init__(self, parser, parent:ParserRuleContext=None, invokingState:int=-1):
            super().__init__(parent, invokingState)
            self.parser = parser

        def value(self):
            return self.getTypedRuleContext(RulesParser.ValueContext,0)


        def num(self):
            return self.getTypedRuleContext(RulesParser.NumContext,0)


        def getRuleIndex(self):
            return RulesParser.RULE_cmp

        def enterRule(self, listener:ParseTreeListener):
            if hasattr( listener, "enterCmp" ):
                listener.enterCmp(self)

        def exitRule(self, listener:ParseTreeListener):
            if hasattr( listener, "exitCmp" ):
                listener.exitCmp(self)

        def accept(self, visitor:ParseTreeVisitor):
            if hasattr( visitor, "visitCmp" ):
                return visitor.visitCmp(self)
            else:
                return visitor.visitChildren(self)




    def cmp(self):

        localctx = RulesParser.CmpContext(self, self._ctx, self.state)
        self.enterRule(localctx, 22, self.RULE_cmp)
        try:
            self.state = 618
            self._errHandler.sync(self)
            la_ = self._interp.adaptivePredict(self._input,64,self._ctx)
            if la_ == 1:
                self.enterOuterAlt(localctx, 1)
                self.state = 594
                self.value()
                self.state = 595
                self.match(RulesParser.T__12)
                self.state = 596
                self.num()
                pass

            elif la_ == 2:
                self.enterOuterAlt(localctx, 2)
                self.state = 598
                self.value()
                self.state = 599
                self.match(RulesParser.T__13)
                self.state = 600
                self.num()
                pass

            elif la_ == 3:
                self.enterOuterAlt(localctx, 3)
                self.state = 602
                self.value()
                self.state = 603
                self.match(RulesParser.T__14)
                self.state = 604
                self.num()
                pass

            elif la_ == 4:
                self.enterOuterAlt(localctx, 4)
                self.state = 606
                self.value()
                self.state = 607
                self.match(RulesParser.T__15)
                self.state = 608
                self.num()
                pass

            elif la_ == 5:
                self.enterOuterAlt(localctx, 5)
                self.state = 610
                self.value()
                self.state = 611
                self.match(RulesParser.T__16)
                self.state = 612
                self.num()
                pass

            elif la_ == 6:
                self.enterOuterAlt(localctx, 6)
                self.state = 614
                self.value()
                self.state = 615
                self.match(RulesParser.T__17)
                self.state = 616
                self.num()
                pass


        except RecognitionException as re:
            localctx.exception = re
            self._errHandler.reportError(self, re)
            self._errHandler.recover(self, re)
        finally:
            self.exitRule()
        return localctx


    class CmpStrContext(ParserRuleContext):
        __slots__ = 'parser'

        def __init__(self, parser, parent:ParserRuleContext=None, invokingState:int=-1):
            super().__init__(parent, invokingState)
            self.parser = parser

        def value(self):
            return self.getTypedRuleContext(RulesParser.ValueContext,0)


        def LIT(self):
            return self.getToken(RulesParser.LIT, 0)

        def getRuleIndex(self):
            return RulesParser.RULE_cmpStr

        def enterRule(self, listener:ParseTreeListener):
            if hasattr( listener, "enterCmpStr" ):
                listener.enterCmpStr(self)

        def exitRule(self, listener:ParseTreeListener):
            if hasattr( listener, "exitCmpStr" ):
                listener.exitCmpStr(self)

        def accept(self, visitor:ParseTreeVisitor):
            if hasattr( visitor, "visitCmpStr" ):
                return visitor.visitCmpStr(self)
            else:
                return visitor.visitChildren(self)




    def cmpStr(self):

        localctx = RulesParser.CmpStrContext(self, self._ctx, self.state)
        self.enterRule(localctx, 24, self.RULE_cmpStr)
        try:
            self.state = 628
            self._errHandler.sync(self)
            la_ = self._interp.adaptivePredict(self._input,65,self._ctx)
            if la_ == 1:
                self.enterOuterAlt(localctx, 1)
                self.state = 620
                self.value()
                self.state = 621
                self.match(RulesParser.T__12)
                self.state = 622
                self.match(RulesParser.LIT)
                pass

            elif la_ == 2:
                self.enterOuterAlt(localctx, 2)
                self.state = 624
                self.value()
                self.state = 625
                self.match(RulesParser.T__13)
                self.state = 626
                self.match(RulesParser.LIT)
                pass


        except RecognitionException as re:
            localctx.exception = re
            self._errHandler.reportError(self, re)
            self._errHandler.recover(self, re)
        finally:
            self.exitRule()
        return localctx


    class FlagMatchContext(ParserRuleContext):
        __slots__ = 'parser'

        def __init__(self, parser, parent:ParserRuleContext=None, invokingState:int=-1):
            super().__init__(parent, invokingState)
            self.parser = parser

        def value(self):
            return self.getTypedRuleContext(RulesParser.ValueContext,0)


        def num(self):
            return self.getTypedRuleContext(RulesParser.NumContext,0)


        def getRuleIndex(self):
            return RulesParser.RULE_flagMatch

        def enterRule(self, listener:ParseTreeListener):
            if hasattr( listener, "enterFlagMatch" ):
                listener.enterFlagMatch(self)

        def exitRule(self, listener:ParseTreeListener):
            if hasattr( listener, "exitFlagMatch" ):
                listener.exitFlagMatch(self)

        def accept(self, visitor:ParseTreeVisitor):
            if hasattr( visitor, "visitFlagMatch" ):
                return visitor.visitFlagMatch(self)
            else:
                return visitor.visitChildren(self)




    def flagMatch(self):

        localctx = RulesParser.FlagMatchContext(self, self._ctx, self.state)
        self.enterRule(localctx, 26, self.RULE_flagMatch)
        try:
            self.enterOuterAlt(localctx, 1)
            self.state = 630
            self.value()
            self.state = 631
            self.match(RulesParser.T__18)
            self.state = 632
            self.num()
        except RecognitionException as re:
            localctx.exception = re
            self._errHandler.reportError(self, re)
            self._errHandler.recover(self, re)
        finally:
            self.exitRule()
        return localctx


    class RefEqContext(ParserRuleContext):
        __slots__ = 'parser'

        def __init__(self, parser, parent:ParserRuleContext=None, invokingState:int=-1):
            super().__init__(parent, invokingState)
            self.parser = parser

        def REF(self):
            return self.getToken(RulesParser.REF, 0)

        def ITEM(self):
            return self.getToken(RulesParser.ITEM, 0)

        def SETTING(self):
            return self.getToken(RulesParser.SETTING, 0)

        def getRuleIndex(self):
            return RulesParser.RULE_refEq

        def enterRule(self, listener:ParseTreeListener):
            if hasattr( listener, "enterRefEq" ):
                listener.enterRefEq(self)

        def exitRule(self, listener:ParseTreeListener):
            if hasattr( listener, "exitRefEq" ):
                listener.exitRefEq(self)

        def accept(self, visitor:ParseTreeVisitor):
            if hasattr( visitor, "visitRefEq" ):
                return visitor.visitRefEq(self)
            else:
                return visitor.visitChildren(self)




    def refEq(self):

        localctx = RulesParser.RefEqContext(self, self._ctx, self.state)
        self.enterRule(localctx, 28, self.RULE_refEq)
        self._la = 0 # Token type
        try:
            self.enterOuterAlt(localctx, 1)
            self.state = 634
            self.match(RulesParser.REF)
            self.state = 635
            self.match(RulesParser.T__12)
            self.state = 636
            _la = self._input.LA(1)
            if not(_la==30 or _la==31):
                self._errHandler.recoverInline(self)
            else:
                self._errHandler.reportMatch(self)
                self.consume()
        except RecognitionException as re:
            localctx.exception = re
            self._errHandler.reportError(self, re)
            self._errHandler.recover(self, re)
        finally:
            self.exitRule()
        return localctx


    class FuncNumContext(ParserRuleContext):
        __slots__ = 'parser'

        def __init__(self, parser, parent:ParserRuleContext=None, invokingState:int=-1):
            super().__init__(parent, invokingState)
            self.parser = parser

        def FUNC(self):
            return self.getToken(RulesParser.FUNC, 0)

        def ITEM(self):
            return self.getToken(RulesParser.ITEM, 0)

        def num(self, i:int=None):
            if i is None:
                return self.getTypedRuleContexts(RulesParser.NumContext)
            else:
                return self.getTypedRuleContext(RulesParser.NumContext,i)


        def getRuleIndex(self):
            return RulesParser.RULE_funcNum

        def enterRule(self, listener:ParseTreeListener):
            if hasattr( listener, "enterFuncNum" ):
                listener.enterFuncNum(self)

        def exitRule(self, listener:ParseTreeListener):
            if hasattr( listener, "exitFuncNum" ):
                listener.exitFuncNum(self)

        def accept(self, visitor:ParseTreeVisitor):
            if hasattr( visitor, "visitFuncNum" ):
                return visitor.visitFuncNum(self)
            else:
                return visitor.visitChildren(self)




    def funcNum(self):

        localctx = RulesParser.FuncNumContext(self, self._ctx, self.state)
        self.enterRule(localctx, 30, self.RULE_funcNum)
        self._la = 0 # Token type
        try:
            self.state = 659
            self._errHandler.sync(self)
            la_ = self._interp.adaptivePredict(self._input,68,self._ctx)
            if la_ == 1:
                self.enterOuterAlt(localctx, 1)
                self.state = 638
                self.match(RulesParser.FUNC)
                self.state = 639
                self.match(RulesParser.T__0)
                self.state = 640
                self.match(RulesParser.ITEM)
                self.state = 641
                self.match(RulesParser.T__1)
                pass

            elif la_ == 2:
                self.enterOuterAlt(localctx, 2)
                self.state = 642
                self.match(RulesParser.FUNC)
                self.state = 643
                self.match(RulesParser.T__0)
                self.state = 644
                self.num()
                self.state = 649
                self._errHandler.sync(self)
                _la = self._input.LA(1)
                while _la==7:
                    self.state = 645
                    self.match(RulesParser.T__6)
                    self.state = 646
                    self.num()
                    self.state = 651
                    self._errHandler.sync(self)
                    _la = self._input.LA(1)

                self.state = 652
                self.match(RulesParser.T__1)
                pass

            elif la_ == 3:
                self.enterOuterAlt(localctx, 3)
                self.state = 654
                self.match(RulesParser.FUNC)
                self.state = 657
                self._errHandler.sync(self)
                la_ = self._interp.adaptivePredict(self._input,67,self._ctx)
                if la_ == 1:
                    self.state = 655
                    self.match(RulesParser.T__0)
                    self.state = 656
                    self.match(RulesParser.T__1)


                pass


        except RecognitionException as re:
            localctx.exception = re
            self._errHandler.reportError(self, re)
            self._errHandler.recover(self, re)
        finally:
            self.exitRule()
        return localctx


    class MathNumContext(ParserRuleContext):
        __slots__ = 'parser'

        def __init__(self, parser, parent:ParserRuleContext=None, invokingState:int=-1):
            super().__init__(parent, invokingState)
            self.parser = parser

        def baseNum(self):
            return self.getTypedRuleContext(RulesParser.BaseNumContext,0)


        def BINOP(self):
            return self.getToken(RulesParser.BINOP, 0)

        def num(self):
            return self.getTypedRuleContext(RulesParser.NumContext,0)


        def getRuleIndex(self):
            return RulesParser.RULE_mathNum

        def enterRule(self, listener:ParseTreeListener):
            if hasattr( listener, "enterMathNum" ):
                listener.enterMathNum(self)

        def exitRule(self, listener:ParseTreeListener):
            if hasattr( listener, "exitMathNum" ):
                listener.exitMathNum(self)

        def accept(self, visitor:ParseTreeVisitor):
            if hasattr( visitor, "visitMathNum" ):
                return visitor.visitMathNum(self)
            else:
                return visitor.visitChildren(self)




    def mathNum(self):

        localctx = RulesParser.MathNumContext(self, self._ctx, self.state)
        self.enterRule(localctx, 32, self.RULE_mathNum)
        try:
            self.enterOuterAlt(localctx, 1)
            self.state = 661
            self.baseNum()
            self.state = 662
            self.match(RulesParser.BINOP)
            self.state = 663
            self.num()
        except RecognitionException as re:
            localctx.exception = re
            self._errHandler.reportError(self, re)
            self._errHandler.recover(self, re)
        finally:
            self.exitRule()
        return localctx


    class NumContext(ParserRuleContext):
        __slots__ = 'parser'

        def __init__(self, parser, parent:ParserRuleContext=None, invokingState:int=-1):
            super().__init__(parent, invokingState)
            self.parser = parser

        def baseNum(self):
            return self.getTypedRuleContext(RulesParser.BaseNumContext,0)


        def mathNum(self):
            return self.getTypedRuleContext(RulesParser.MathNumContext,0)


        def getRuleIndex(self):
            return RulesParser.RULE_num

        def enterRule(self, listener:ParseTreeListener):
            if hasattr( listener, "enterNum" ):
                listener.enterNum(self)

        def exitRule(self, listener:ParseTreeListener):
            if hasattr( listener, "exitNum" ):
                listener.exitNum(self)

        def accept(self, visitor:ParseTreeVisitor):
            if hasattr( visitor, "visitNum" ):
                return visitor.visitNum(self)
            else:
                return visitor.visitChildren(self)




    def num(self):

        localctx = RulesParser.NumContext(self, self._ctx, self.state)
        self.enterRule(localctx, 34, self.RULE_num)
        try:
            self.state = 667
            self._errHandler.sync(self)
            la_ = self._interp.adaptivePredict(self._input,69,self._ctx)
            if la_ == 1:
                self.enterOuterAlt(localctx, 1)
                self.state = 665
                self.baseNum()
                pass

            elif la_ == 2:
                self.enterOuterAlt(localctx, 2)
                self.state = 666
                self.mathNum()
                pass


        except RecognitionException as re:
            localctx.exception = re
            self._errHandler.reportError(self, re)
            self._errHandler.recover(self, re)
        finally:
            self.exitRule()
        return localctx


    class BaseNumContext(ParserRuleContext):
        __slots__ = 'parser'

        def __init__(self, parser, parent:ParserRuleContext=None, invokingState:int=-1):
            super().__init__(parent, invokingState)
            self.parser = parser

        def INT(self):
            return self.getToken(RulesParser.INT, 0)

        def CONST(self):
            return self.getToken(RulesParser.CONST, 0)

        def SETTING(self):
            return self.getToken(RulesParser.SETTING, 0)

        def REF(self):
            return self.getToken(RulesParser.REF, 0)

        def value(self):
            return self.getTypedRuleContext(RulesParser.ValueContext,0)


        def switchNum(self):
            return self.getTypedRuleContext(RulesParser.SwitchNumContext,0)


        def funcNum(self):
            return self.getTypedRuleContext(RulesParser.FuncNumContext,0)


        def condNum(self):
            return self.getTypedRuleContext(RulesParser.CondNumContext,0)


        def getRuleIndex(self):
            return RulesParser.RULE_baseNum

        def enterRule(self, listener:ParseTreeListener):
            if hasattr( listener, "enterBaseNum" ):
                listener.enterBaseNum(self)

        def exitRule(self, listener:ParseTreeListener):
            if hasattr( listener, "exitBaseNum" ):
                listener.exitBaseNum(self)

        def accept(self, visitor:ParseTreeVisitor):
            if hasattr( visitor, "visitBaseNum" ):
                return visitor.visitBaseNum(self)
            else:
                return visitor.visitChildren(self)




    def baseNum(self):

        localctx = RulesParser.BaseNumContext(self, self._ctx, self.state)
        self.enterRule(localctx, 36, self.RULE_baseNum)
        try:
            self.state = 677
            self._errHandler.sync(self)
            la_ = self._interp.adaptivePredict(self._input,70,self._ctx)
            if la_ == 1:
                self.enterOuterAlt(localctx, 1)
                self.state = 669
                self.match(RulesParser.INT)
                pass

            elif la_ == 2:
                self.enterOuterAlt(localctx, 2)
                self.state = 670
                self.match(RulesParser.CONST)
                pass

            elif la_ == 3:
                self.enterOuterAlt(localctx, 3)
                self.state = 671
                self.match(RulesParser.SETTING)
                pass

            elif la_ == 4:
                self.enterOuterAlt(localctx, 4)
                self.state = 672
                self.match(RulesParser.REF)
                pass

            elif la_ == 5:
                self.enterOuterAlt(localctx, 5)
                self.state = 673
                self.value()
                pass

            elif la_ == 6:
                self.enterOuterAlt(localctx, 6)
                self.state = 674
                self.switchNum()
                pass

            elif la_ == 7:
                self.enterOuterAlt(localctx, 7)
                self.state = 675
                self.funcNum()
                pass

            elif la_ == 8:
                self.enterOuterAlt(localctx, 8)
                self.state = 676
                self.condNum()
                pass


        except RecognitionException as re:
            localctx.exception = re
            self._errHandler.reportError(self, re)
            self._errHandler.recover(self, re)
        finally:
            self.exitRule()
        return localctx


    class ValueContext(ParserRuleContext):
        __slots__ = 'parser'

        def __init__(self, parser, parent:ParserRuleContext=None, invokingState:int=-1):
            super().__init__(parent, invokingState)
            self.parser = parser


        def getRuleIndex(self):
            return RulesParser.RULE_value

     
        def copyFrom(self, ctx:ParserRuleContext):
            super().copyFrom(ctx)



    class ArgumentContext(ValueContext):

        def __init__(self, parser, ctx:ParserRuleContext): # actually a RulesParser.ValueContext
            super().__init__(parser)
            self.copyFrom(ctx)

        def REF(self):
            return self.getToken(RulesParser.REF, 0)

        def enterRule(self, listener:ParseTreeListener):
            if hasattr( listener, "enterArgument" ):
                listener.enterArgument(self)

        def exitRule(self, listener:ParseTreeListener):
            if hasattr( listener, "exitArgument" ):
                listener.exitArgument(self)

        def accept(self, visitor:ParseTreeVisitor):
            if hasattr( visitor, "visitArgument" ):
                return visitor.visitArgument(self)
            else:
                return visitor.visitChildren(self)


    class SettingContext(ValueContext):

        def __init__(self, parser, ctx:ParserRuleContext): # actually a RulesParser.ValueContext
            super().__init__(parser)
            self.copyFrom(ctx)

        def SETTING(self):
            return self.getToken(RulesParser.SETTING, 0)
        def LIT(self):
            return self.getToken(RulesParser.LIT, 0)
        def ITEM(self):
            return self.getToken(RulesParser.ITEM, 0)

        def enterRule(self, listener:ParseTreeListener):
            if hasattr( listener, "enterSetting" ):
                listener.enterSetting(self)

        def exitRule(self, listener:ParseTreeListener):
            if hasattr( listener, "exitSetting" ):
                listener.exitSetting(self)

        def accept(self, visitor:ParseTreeVisitor):
            if hasattr( visitor, "visitSetting" ):
                return visitor.visitSetting(self)
            else:
                return visitor.visitChildren(self)



    def value(self):

        localctx = RulesParser.ValueContext(self, self._ctx, self.state)
        self.enterRule(localctx, 38, self.RULE_value)
        self._la = 0 # Token type
        try:
            self.state = 686
            self._errHandler.sync(self)
            token = self._input.LA(1)
            if token in [31]:
                localctx = RulesParser.SettingContext(self, localctx)
                self.enterOuterAlt(localctx, 1)
                self.state = 679
                self.match(RulesParser.SETTING)
                self.state = 683
                self._errHandler.sync(self)
                la_ = self._interp.adaptivePredict(self._input,71,self._ctx)
                if la_ == 1:
                    self.state = 680
                    self.match(RulesParser.T__10)
                    self.state = 681
                    _la = self._input.LA(1)
                    if not(_la==30 or _la==35):
                        self._errHandler.recoverInline(self)
                    else:
                        self._errHandler.reportMatch(self)
                        self.consume()
                    self.state = 682
                    self.match(RulesParser.T__11)


                pass
            elif token in [32]:
                localctx = RulesParser.ArgumentContext(self, localctx)
                self.enterOuterAlt(localctx, 2)
                self.state = 685
                self.match(RulesParser.REF)
                pass
            else:
                raise NoViableAltException(self)

        except RecognitionException as re:
            localctx.exception = re
            self._errHandler.reportError(self, re)
            self._errHandler.recover(self, re)
        finally:
            self.exitRule()
        return localctx


    class ItemListContext(ParserRuleContext):
        __slots__ = 'parser'

        def __init__(self, parser, parent:ParserRuleContext=None, invokingState:int=-1):
            super().__init__(parent, invokingState)
            self.parser = parser

        def FUNC(self, i:int=None):
            if i is None:
                return self.getTokens(RulesParser.FUNC)
            else:
                return self.getToken(RulesParser.FUNC, i)

        def item(self, i:int=None):
            if i is None:
                return self.getTypedRuleContexts(RulesParser.ItemContext)
            else:
                return self.getTypedRuleContext(RulesParser.ItemContext,i)


        def getRuleIndex(self):
            return RulesParser.RULE_itemList

        def enterRule(self, listener:ParseTreeListener):
            if hasattr( listener, "enterItemList" ):
                listener.enterItemList(self)

        def exitRule(self, listener:ParseTreeListener):
            if hasattr( listener, "exitItemList" ):
                listener.exitItemList(self)

        def accept(self, visitor:ParseTreeVisitor):
            if hasattr( visitor, "visitItemList" ):
                return visitor.visitItemList(self)
            else:
                return visitor.visitChildren(self)




    def itemList(self):

        localctx = RulesParser.ItemListContext(self, self._ctx, self.state)
        self.enterRule(localctx, 40, self.RULE_itemList)
        self._la = 0 # Token type
        try:
            self.enterOuterAlt(localctx, 1)
            self.state = 688
            self.match(RulesParser.T__10)
            self.state = 691
            self._errHandler.sync(self)
            token = self._input.LA(1)
            if token in [33]:
                self.state = 689
                self.match(RulesParser.FUNC)
                pass
            elif token in [1, 22, 30, 32, 35]:
                self.state = 690
                self.item()
                pass
            else:
                raise NoViableAltException(self)

            self.state = 700
            self._errHandler.sync(self)
            _la = self._input.LA(1)
            while _la==7:
                self.state = 693
                self.match(RulesParser.T__6)
                self.state = 696
                self._errHandler.sync(self)
                token = self._input.LA(1)
                if token in [33]:
                    self.state = 694
                    self.match(RulesParser.FUNC)
                    pass
                elif token in [1, 22, 30, 32, 35]:
                    self.state = 695
                    self.item()
                    pass
                else:
                    raise NoViableAltException(self)

                self.state = 702
                self._errHandler.sync(self)
                _la = self._input.LA(1)

            self.state = 703
            self.match(RulesParser.T__11)
        except RecognitionException as re:
            localctx.exception = re
            self._errHandler.reportError(self, re)
            self._errHandler.recover(self, re)
        finally:
            self.exitRule()
        return localctx


    class ItemContext(ParserRuleContext):
        __slots__ = 'parser'

        def __init__(self, parser, parent:ParserRuleContext=None, invokingState:int=-1):
            super().__init__(parent, invokingState)
            self.parser = parser


        def getRuleIndex(self):
            return RulesParser.RULE_item

     
        def copyFrom(self, ctx:ParserRuleContext):
            super().copyFrom(ctx)



    class OneLitItemContext(ItemContext):

        def __init__(self, parser, ctx:ParserRuleContext): # actually a RulesParser.ItemContext
            super().__init__(parser)
            self.copyFrom(ctx)

        def LIT(self):
            return self.getToken(RulesParser.LIT, 0)

        def enterRule(self, listener:ParseTreeListener):
            if hasattr( listener, "enterOneLitItem" ):
                listener.enterOneLitItem(self)

        def exitRule(self, listener:ParseTreeListener):
            if hasattr( listener, "exitOneLitItem" ):
                listener.exitOneLitItem(self)

        def accept(self, visitor:ParseTreeVisitor):
            if hasattr( visitor, "visitOneLitItem" ):
                return visitor.visitOneLitItem(self)
            else:
                return visitor.visitChildren(self)


    class OneArgumentContext(ItemContext):

        def __init__(self, parser, ctx:ParserRuleContext): # actually a RulesParser.ItemContext
            super().__init__(parser)
            self.copyFrom(ctx)

        def REF(self):
            return self.getToken(RulesParser.REF, 0)

        def enterRule(self, listener:ParseTreeListener):
            if hasattr( listener, "enterOneArgument" ):
                listener.enterOneArgument(self)

        def exitRule(self, listener:ParseTreeListener):
            if hasattr( listener, "exitOneArgument" ):
                listener.exitOneArgument(self)

        def accept(self, visitor:ParseTreeVisitor):
            if hasattr( visitor, "visitOneArgument" ):
                return visitor.visitOneArgument(self)
            else:
                return visitor.visitChildren(self)


    class ItemCountContext(ItemContext):

        def __init__(self, parser, ctx:ParserRuleContext): # actually a RulesParser.ItemContext
            super().__init__(parser)
            self.copyFrom(ctx)

        def ITEM(self):
            return self.getToken(RulesParser.ITEM, 0)
        def INT(self):
            return self.getToken(RulesParser.INT, 0)
        def SETTING(self):
            return self.getToken(RulesParser.SETTING, 0)

        def enterRule(self, listener:ParseTreeListener):
            if hasattr( listener, "enterItemCount" ):
                listener.enterItemCount(self)

        def exitRule(self, listener:ParseTreeListener):
            if hasattr( listener, "exitItemCount" ):
                listener.exitItemCount(self)

        def accept(self, visitor:ParseTreeVisitor):
            if hasattr( visitor, "visitItemCount" ):
                return visitor.visitItemCount(self)
            else:
                return visitor.visitChildren(self)


    class OneItemContext(ItemContext):

        def __init__(self, parser, ctx:ParserRuleContext): # actually a RulesParser.ItemContext
            super().__init__(parser)
            self.copyFrom(ctx)

        def ITEM(self):
            return self.getToken(RulesParser.ITEM, 0)
        def NOT(self):
            return self.getToken(RulesParser.NOT, 0)

        def enterRule(self, listener:ParseTreeListener):
            if hasattr( listener, "enterOneItem" ):
                listener.enterOneItem(self)

        def exitRule(self, listener:ParseTreeListener):
            if hasattr( listener, "exitOneItem" ):
                listener.exitOneItem(self)

        def accept(self, visitor:ParseTreeVisitor):
            if hasattr( visitor, "visitOneItem" ):
                return visitor.visitOneItem(self)
            else:
                return visitor.visitChildren(self)



    def item(self):

        localctx = RulesParser.ItemContext(self, self._ctx, self.state)
        self.enterRule(localctx, 42, self.RULE_item)
        self._la = 0 # Token type
        try:
            self.state = 722
            self._errHandler.sync(self)
            la_ = self._interp.adaptivePredict(self._input,78,self._ctx)
            if la_ == 1:
                localctx = RulesParser.ItemCountContext(self, localctx)
                self.enterOuterAlt(localctx, 1)
                self.state = 714
                self._errHandler.sync(self)
                token = self._input.LA(1)
                if token in [30]:
                    self.state = 705
                    self.match(RulesParser.ITEM)
                    self.state = 706
                    self.match(RulesParser.T__4)
                    self.state = 707
                    _la = self._input.LA(1)
                    if not(_la==31 or _la==37):
                        self._errHandler.recoverInline(self)
                    else:
                        self._errHandler.reportMatch(self)
                        self.consume()
                    self.state = 708
                    self.match(RulesParser.T__5)
                    pass
                elif token in [1]:
                    self.state = 709
                    self.match(RulesParser.T__0)
                    self.state = 710
                    self.match(RulesParser.ITEM)
                    self.state = 711
                    self.match(RulesParser.T__6)
                    self.state = 712
                    _la = self._input.LA(1)
                    if not(_la==31 or _la==37):
                        self._errHandler.recoverInline(self)
                    else:
                        self._errHandler.reportMatch(self)
                        self.consume()
                    self.state = 713
                    self.match(RulesParser.T__1)
                    pass
                else:
                    raise NoViableAltException(self)

                pass

            elif la_ == 2:
                localctx = RulesParser.OneItemContext(self, localctx)
                self.enterOuterAlt(localctx, 2)
                self.state = 717
                self._errHandler.sync(self)
                _la = self._input.LA(1)
                if _la==22:
                    self.state = 716
                    self.match(RulesParser.NOT)


                self.state = 719
                self.match(RulesParser.ITEM)
                pass

            elif la_ == 3:
                localctx = RulesParser.OneLitItemContext(self, localctx)
                self.enterOuterAlt(localctx, 3)
                self.state = 720
                self.match(RulesParser.LIT)
                pass

            elif la_ == 4:
                localctx = RulesParser.OneArgumentContext(self, localctx)
                self.enterOuterAlt(localctx, 4)
                self.state = 721
                self.match(RulesParser.REF)
                pass


        except RecognitionException as re:
            localctx.exception = re
            self._errHandler.reportError(self, re)
            self._errHandler.recover(self, re)
        finally:
            self.exitRule()
        return localctx


    class StrContext(ParserRuleContext):
        __slots__ = 'parser'

        def __init__(self, parser, parent:ParserRuleContext=None, invokingState:int=-1):
            super().__init__(parent, invokingState)
            self.parser = parser

        def LIT(self):
            return self.getToken(RulesParser.LIT, 0)

        def value(self):
            return self.getTypedRuleContext(RulesParser.ValueContext,0)


        def condStr(self):
            return self.getTypedRuleContext(RulesParser.CondStrContext,0)


        def switchStr(self):
            return self.getTypedRuleContext(RulesParser.SwitchStrContext,0)


        def getRuleIndex(self):
            return RulesParser.RULE_str

        def enterRule(self, listener:ParseTreeListener):
            if hasattr( listener, "enterStr" ):
                listener.enterStr(self)

        def exitRule(self, listener:ParseTreeListener):
            if hasattr( listener, "exitStr" ):
                listener.exitStr(self)

        def accept(self, visitor:ParseTreeVisitor):
            if hasattr( visitor, "visitStr" ):
                return visitor.visitStr(self)
            else:
                return visitor.visitChildren(self)




    def str_(self):

        localctx = RulesParser.StrContext(self, self._ctx, self.state)
        self.enterRule(localctx, 44, self.RULE_str)
        try:
            self.state = 728
            self._errHandler.sync(self)
            token = self._input.LA(1)
            if token in [35]:
                self.enterOuterAlt(localctx, 1)
                self.state = 724
                self.match(RulesParser.LIT)
                pass
            elif token in [31, 32]:
                self.enterOuterAlt(localctx, 2)
                self.state = 725
                self.value()
                pass
            elif token in [25]:
                self.enterOuterAlt(localctx, 3)
                self.state = 726
                self.condStr()
                pass
            elif token in [28]:
                self.enterOuterAlt(localctx, 4)
                self.state = 727
                self.switchStr()
                pass
            else:
                raise NoViableAltException(self)

        except RecognitionException as re:
            localctx.exception = re
            self._errHandler.reportError(self, re)
            self._errHandler.recover(self, re)
        finally:
            self.exitRule()
        return localctx


    class SomewhereContext(ParserRuleContext):
        __slots__ = 'parser'

        def __init__(self, parser, parent:ParserRuleContext=None, invokingState:int=-1):
            super().__init__(parent, invokingState)
            self.parser = parser

        def WITHIN(self):
            return self.getToken(RulesParser.WITHIN, 0)

        def PLACE(self, i:int=None):
            if i is None:
                return self.getTokens(RulesParser.PLACE)
            else:
                return self.getToken(RulesParser.PLACE, i)

        def NOT(self):
            return self.getToken(RulesParser.NOT, 0)

        def getRuleIndex(self):
            return RulesParser.RULE_somewhere

        def enterRule(self, listener:ParseTreeListener):
            if hasattr( listener, "enterSomewhere" ):
                listener.enterSomewhere(self)

        def exitRule(self, listener:ParseTreeListener):
            if hasattr( listener, "exitSomewhere" ):
                listener.exitSomewhere(self)

        def accept(self, visitor:ParseTreeVisitor):
            if hasattr( visitor, "visitSomewhere" ):
                return visitor.visitSomewhere(self)
            else:
                return visitor.visitChildren(self)




    def somewhere(self):

        localctx = RulesParser.SomewhereContext(self, self._ctx, self.state)
        self.enterRule(localctx, 46, self.RULE_somewhere)
        self._la = 0 # Token type
        try:
            self.state = 749
            self._errHandler.sync(self)
            la_ = self._interp.adaptivePredict(self._input,83,self._ctx)
            if la_ == 1:
                self.enterOuterAlt(localctx, 1)
                self.state = 731
                self._errHandler.sync(self)
                _la = self._input.LA(1)
                if _la==22:
                    self.state = 730
                    self.match(RulesParser.NOT)


                self.state = 733
                self.match(RulesParser.WITHIN)
                self.state = 734
                self.match(RulesParser.PLACE)
                pass

            elif la_ == 2:
                self.enterOuterAlt(localctx, 2)
                self.state = 736
                self._errHandler.sync(self)
                _la = self._input.LA(1)
                if _la==22:
                    self.state = 735
                    self.match(RulesParser.NOT)


                self.state = 738
                self.match(RulesParser.WITHIN)
                self.state = 739
                self.match(RulesParser.T__0)
                self.state = 740
                self.match(RulesParser.PLACE)
                self.state = 745
                self._errHandler.sync(self)
                _la = self._input.LA(1)
                while _la==7:
                    self.state = 741
                    self.match(RulesParser.T__6)
                    self.state = 742
                    self.match(RulesParser.PLACE)
                    self.state = 747
                    self._errHandler.sync(self)
                    _la = self._input.LA(1)

                self.state = 748
                self.match(RulesParser.T__1)
                pass


        except RecognitionException as re:
            localctx.exception = re
            self._errHandler.reportError(self, re)
            self._errHandler.recover(self, re)
        finally:
            self.exitRule()
        return localctx


    class RefSomewhereContext(ParserRuleContext):
        __slots__ = 'parser'

        def __init__(self, parser, parent:ParserRuleContext=None, invokingState:int=-1):
            super().__init__(parent, invokingState)
            self.parser = parser


        def getRuleIndex(self):
            return RulesParser.RULE_refSomewhere

     
        def copyFrom(self, ctx:ParserRuleContext):
            super().copyFrom(ctx)



    class RefInPlaceNameContext(RefSomewhereContext):

        def __init__(self, parser, ctx:ParserRuleContext): # actually a RulesParser.RefSomewhereContext
            super().__init__(parser)
            self.copyFrom(ctx)

        def REF(self):
            return self.getToken(RulesParser.REF, 0)
        def WITHIN(self):
            return self.getToken(RulesParser.WITHIN, 0)
        def PLACE(self):
            return self.getToken(RulesParser.PLACE, 0)
        def NOT(self):
            return self.getToken(RulesParser.NOT, 0)

        def enterRule(self, listener:ParseTreeListener):
            if hasattr( listener, "enterRefInPlaceName" ):
                listener.enterRefInPlaceName(self)

        def exitRule(self, listener:ParseTreeListener):
            if hasattr( listener, "exitRefInPlaceName" ):
                listener.exitRefInPlaceName(self)

        def accept(self, visitor:ParseTreeVisitor):
            if hasattr( visitor, "visitRefInPlaceName" ):
                return visitor.visitRefInPlaceName(self)
            else:
                return visitor.visitChildren(self)


    class RefInFuncContext(RefSomewhereContext):

        def __init__(self, parser, ctx:ParserRuleContext): # actually a RulesParser.RefSomewhereContext
            super().__init__(parser)
            self.copyFrom(ctx)

        def REF(self):
            return self.getToken(RulesParser.REF, 0)
        def WITHIN(self):
            return self.getToken(RulesParser.WITHIN, 0)
        def invoke(self):
            return self.getTypedRuleContext(RulesParser.InvokeContext,0)

        def NOT(self):
            return self.getToken(RulesParser.NOT, 0)

        def enterRule(self, listener:ParseTreeListener):
            if hasattr( listener, "enterRefInFunc" ):
                listener.enterRefInFunc(self)

        def exitRule(self, listener:ParseTreeListener):
            if hasattr( listener, "exitRefInFunc" ):
                listener.exitRefInFunc(self)

        def accept(self, visitor:ParseTreeVisitor):
            if hasattr( visitor, "visitRefInFunc" ):
                return visitor.visitRefInFunc(self)
            else:
                return visitor.visitChildren(self)


    class RefInPlaceRefContext(RefSomewhereContext):

        def __init__(self, parser, ctx:ParserRuleContext): # actually a RulesParser.RefSomewhereContext
            super().__init__(parser)
            self.copyFrom(ctx)

        def REF(self, i:int=None):
            if i is None:
                return self.getTokens(RulesParser.REF)
            else:
                return self.getToken(RulesParser.REF, i)
        def WITHIN(self):
            return self.getToken(RulesParser.WITHIN, 0)
        def NOT(self):
            return self.getToken(RulesParser.NOT, 0)

        def enterRule(self, listener:ParseTreeListener):
            if hasattr( listener, "enterRefInPlaceRef" ):
                listener.enterRefInPlaceRef(self)

        def exitRule(self, listener:ParseTreeListener):
            if hasattr( listener, "exitRefInPlaceRef" ):
                listener.exitRefInPlaceRef(self)

        def accept(self, visitor:ParseTreeVisitor):
            if hasattr( visitor, "visitRefInPlaceRef" ):
                return visitor.visitRefInPlaceRef(self)
            else:
                return visitor.visitChildren(self)



    def refSomewhere(self):

        localctx = RulesParser.RefSomewhereContext(self, self._ctx, self.state)
        self.enterRule(localctx, 48, self.RULE_refSomewhere)
        self._la = 0 # Token type
        try:
            self.state = 769
            self._errHandler.sync(self)
            la_ = self._interp.adaptivePredict(self._input,87,self._ctx)
            if la_ == 1:
                localctx = RulesParser.RefInPlaceRefContext(self, localctx)
                self.enterOuterAlt(localctx, 1)
                self.state = 751
                self.match(RulesParser.REF)
                self.state = 753
                self._errHandler.sync(self)
                _la = self._input.LA(1)
                if _la==22:
                    self.state = 752
                    self.match(RulesParser.NOT)


                self.state = 755
                self.match(RulesParser.WITHIN)
                self.state = 756
                self.match(RulesParser.REF)
                pass

            elif la_ == 2:
                localctx = RulesParser.RefInPlaceNameContext(self, localctx)
                self.enterOuterAlt(localctx, 2)
                self.state = 757
                self.match(RulesParser.REF)
                self.state = 759
                self._errHandler.sync(self)
                _la = self._input.LA(1)
                if _la==22:
                    self.state = 758
                    self.match(RulesParser.NOT)


                self.state = 761
                self.match(RulesParser.WITHIN)
                self.state = 762
                self.match(RulesParser.PLACE)
                pass

            elif la_ == 3:
                localctx = RulesParser.RefInFuncContext(self, localctx)
                self.enterOuterAlt(localctx, 3)
                self.state = 763
                self.match(RulesParser.REF)
                self.state = 765
                self._errHandler.sync(self)
                _la = self._input.LA(1)
                if _la==22:
                    self.state = 764
                    self.match(RulesParser.NOT)


                self.state = 767
                self.match(RulesParser.WITHIN)
                self.state = 768
                self.invoke()
                pass


        except RecognitionException as re:
            localctx.exception = re
            self._errHandler.reportError(self, re)
            self._errHandler.recover(self, re)
        finally:
            self.exitRule()
        return localctx



    def sempred(self, localctx:RuleContext, ruleIndex:int, predIndex:int):
        if self._predicates == None:
            self._predicates = dict()
        self._predicates[0] = self.boolExpr_sempred
        pred = self._predicates.get(ruleIndex, None)
        if pred is None:
            raise Exception("No predicate with index:" + str(ruleIndex))
        else:
            return pred(localctx, predIndex)

    def boolExpr_sempred(self, localctx:BoolExprContext, predIndex:int):
            if predIndex == 0:
                return self.precpred(self._ctx, 17)
         

            if predIndex == 1:
                return self.precpred(self._ctx, 16)
         




