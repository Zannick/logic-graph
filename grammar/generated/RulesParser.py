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
        4,1,41,783,2,0,7,0,2,1,7,1,2,2,7,2,2,3,7,3,2,4,7,4,2,5,7,5,2,6,7,
        6,2,7,7,7,2,8,7,8,2,9,7,9,2,10,7,10,2,11,7,11,2,12,7,12,2,13,7,13,
        2,14,7,14,2,15,7,15,2,16,7,16,2,17,7,17,2,18,7,18,2,19,7,19,2,20,
        7,20,2,21,7,21,2,22,7,22,2,23,7,23,2,24,7,24,1,0,1,0,1,0,1,0,1,0,
        1,0,1,0,1,0,1,0,1,0,1,0,1,0,1,0,1,0,1,0,1,0,3,0,67,8,0,1,0,1,0,1,
        0,1,0,1,0,3,0,74,8,0,1,0,1,0,1,0,1,0,1,0,1,0,5,0,82,8,0,10,0,12,
        0,85,9,0,1,1,1,1,1,1,5,1,90,8,1,10,1,12,1,93,9,1,1,1,3,1,96,8,1,
        1,2,1,2,1,2,1,2,1,2,1,2,1,2,1,2,3,2,106,8,2,1,2,1,2,1,2,1,2,1,2,
        1,2,1,2,1,2,1,2,1,2,1,2,1,2,1,2,1,2,1,2,1,2,1,2,1,2,1,2,1,2,1,2,
        5,2,129,8,2,10,2,12,2,132,9,2,1,2,1,2,1,2,1,2,1,2,3,2,139,8,2,1,
        2,1,2,1,2,1,2,3,2,145,8,2,1,3,1,3,1,3,1,3,1,3,1,3,1,3,1,3,1,3,1,
        3,1,3,1,3,3,3,159,8,3,1,4,3,4,162,8,4,1,4,1,4,1,4,1,4,1,4,5,4,169,
        8,4,10,4,12,4,172,9,4,1,4,1,4,3,4,176,8,4,1,4,1,4,1,4,1,4,1,4,1,
        4,3,4,184,8,4,1,4,1,4,1,4,1,4,1,4,3,4,191,8,4,1,4,1,4,1,4,1,4,1,
        4,3,4,198,8,4,1,4,1,4,1,4,1,4,1,4,3,4,205,8,4,1,4,1,4,1,4,1,4,1,
        4,5,4,212,8,4,10,4,12,4,215,9,4,1,4,1,4,3,4,219,8,4,1,4,1,4,1,4,
        1,4,1,4,5,4,226,8,4,10,4,12,4,229,9,4,1,4,1,4,3,4,233,8,4,1,4,1,
        4,1,4,3,4,238,8,4,3,4,240,8,4,1,5,1,5,1,5,1,5,1,5,1,5,1,5,1,5,1,
        5,1,5,1,5,1,5,1,5,1,5,1,5,1,5,5,5,258,8,5,10,5,12,5,261,9,5,1,5,
        1,5,1,5,1,5,1,5,3,5,268,8,5,1,5,1,5,1,5,1,5,1,5,1,5,1,5,1,5,3,5,
        278,8,5,1,6,1,6,1,6,1,6,1,6,1,6,1,6,1,6,1,6,1,6,1,6,1,6,1,6,1,6,
        1,6,1,6,5,6,296,8,6,10,6,12,6,299,9,6,1,6,1,6,1,6,1,6,1,6,3,6,306,
        8,6,1,7,1,7,1,7,1,7,1,7,1,7,1,7,1,7,1,7,1,7,1,7,1,7,1,7,1,7,1,7,
        1,7,5,7,324,8,7,10,7,12,7,327,9,7,1,7,1,7,1,7,1,7,1,7,3,7,334,8,
        7,1,8,1,8,1,8,1,8,1,8,1,8,1,8,1,8,4,8,344,8,8,11,8,12,8,345,1,8,
        1,8,1,8,1,8,3,8,352,8,8,1,8,1,8,1,8,1,8,1,8,1,8,1,8,1,8,1,8,1,8,
        4,8,364,8,8,11,8,12,8,365,1,8,1,8,1,8,1,8,1,8,4,8,373,8,8,11,8,12,
        8,374,3,8,377,8,8,1,8,1,8,1,8,1,8,3,8,383,8,8,1,8,1,8,1,8,1,8,1,
        8,1,8,1,8,1,8,5,8,393,8,8,10,8,12,8,396,9,8,1,8,1,8,1,8,1,8,4,8,
        402,8,8,11,8,12,8,403,1,8,1,8,1,8,1,8,3,8,410,8,8,1,8,1,8,1,8,1,
        8,1,8,1,8,1,8,1,8,4,8,420,8,8,11,8,12,8,421,1,8,1,8,1,8,1,8,1,8,
        1,8,1,8,4,8,431,8,8,11,8,12,8,432,1,8,3,8,436,8,8,1,9,1,9,1,9,1,
        9,1,9,1,9,1,9,1,9,4,9,446,8,9,11,9,12,9,447,1,9,1,9,1,9,1,9,3,9,
        454,8,9,1,9,1,9,1,9,1,9,1,9,1,9,1,9,1,9,1,9,1,9,4,9,466,8,9,11,9,
        12,9,467,1,9,1,9,1,9,1,9,1,9,4,9,475,8,9,11,9,12,9,476,3,9,479,8,
        9,1,9,1,9,1,9,1,9,3,9,485,8,9,1,9,1,9,1,9,1,9,1,9,1,9,1,9,1,9,1,
        9,1,9,4,9,497,8,9,11,9,12,9,498,1,9,1,9,1,9,1,9,1,9,4,9,506,8,9,
        11,9,12,9,507,3,9,510,8,9,1,9,1,9,1,9,1,9,3,9,516,8,9,1,9,1,9,3,
        9,520,8,9,1,10,1,10,1,10,1,10,1,10,1,10,1,10,1,10,4,10,530,8,10,
        11,10,12,10,531,1,10,1,10,1,10,1,10,3,10,538,8,10,1,10,1,10,1,10,
        1,10,1,10,1,10,1,10,1,10,1,10,1,10,4,10,550,8,10,11,10,12,10,551,
        1,10,1,10,1,10,1,10,1,10,4,10,559,8,10,11,10,12,10,560,3,10,563,
        8,10,1,10,1,10,1,10,1,10,3,10,569,8,10,1,10,1,10,1,10,1,10,1,10,
        1,10,1,10,1,10,1,10,1,10,4,10,581,8,10,11,10,12,10,582,1,10,1,10,
        1,10,1,10,1,10,4,10,590,8,10,11,10,12,10,591,3,10,594,8,10,1,10,
        1,10,1,10,1,10,3,10,600,8,10,1,10,1,10,3,10,604,8,10,1,11,1,11,1,
        11,1,11,1,11,1,11,1,11,1,11,1,11,1,11,1,11,1,11,1,11,1,11,1,11,1,
        11,1,11,1,11,1,11,1,11,1,11,1,11,1,11,1,11,3,11,630,8,11,1,12,1,
        12,1,12,1,12,1,12,1,12,1,12,1,12,3,12,640,8,12,1,13,1,13,1,13,1,
        13,1,14,1,14,1,14,1,14,1,15,1,15,1,15,1,15,1,15,1,15,1,15,1,15,1,
        15,5,15,659,8,15,10,15,12,15,662,9,15,1,15,1,15,1,15,1,15,1,15,3,
        15,669,8,15,3,15,671,8,15,1,16,1,16,1,16,1,16,1,17,1,17,3,17,679,
        8,17,1,18,1,18,1,18,1,18,1,18,1,18,1,18,1,18,3,18,689,8,18,1,19,
        1,19,1,19,1,19,3,19,695,8,19,1,19,3,19,698,8,19,1,20,1,20,1,20,3,
        20,703,8,20,1,20,1,20,1,20,3,20,708,8,20,5,20,710,8,20,10,20,12,
        20,713,9,20,1,20,1,20,1,21,1,21,1,21,1,21,1,21,1,21,1,21,1,21,1,
        21,3,21,726,8,21,1,21,3,21,729,8,21,1,21,1,21,1,21,3,21,734,8,21,
        1,22,1,22,1,22,1,22,3,22,740,8,22,1,23,3,23,743,8,23,1,23,1,23,1,
        23,3,23,748,8,23,1,23,1,23,1,23,1,23,1,23,5,23,755,8,23,10,23,12,
        23,758,9,23,1,23,3,23,761,8,23,1,24,1,24,3,24,765,8,24,1,24,1,24,
        1,24,1,24,3,24,771,8,24,1,24,1,24,1,24,1,24,3,24,777,8,24,1,24,1,
        24,3,24,781,8,24,1,24,0,1,0,25,0,2,4,6,8,10,12,14,16,18,20,22,24,
        26,28,30,32,34,36,38,40,42,44,46,48,0,3,1,0,31,32,2,0,31,31,36,36,
        2,0,32,32,38,38,894,0,73,1,0,0,0,2,86,1,0,0,0,4,144,1,0,0,0,6,158,
        1,0,0,0,8,239,1,0,0,0,10,277,1,0,0,0,12,279,1,0,0,0,14,307,1,0,0,
        0,16,435,1,0,0,0,18,519,1,0,0,0,20,603,1,0,0,0,22,629,1,0,0,0,24,
        639,1,0,0,0,26,641,1,0,0,0,28,645,1,0,0,0,30,670,1,0,0,0,32,672,
        1,0,0,0,34,678,1,0,0,0,36,688,1,0,0,0,38,697,1,0,0,0,40,699,1,0,
        0,0,42,733,1,0,0,0,44,739,1,0,0,0,46,760,1,0,0,0,48,780,1,0,0,0,
        50,51,6,0,-1,0,51,52,5,1,0,0,52,53,3,0,0,0,53,54,5,2,0,0,54,74,1,
        0,0,0,55,74,3,8,4,0,56,74,3,6,3,0,57,74,3,16,8,0,58,74,3,10,5,0,
        59,74,3,22,11,0,60,74,3,24,12,0,61,74,3,26,13,0,62,74,3,28,14,0,
        63,74,3,40,20,0,64,74,3,42,21,0,65,67,5,22,0,0,66,65,1,0,0,0,66,
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
        98,5,33,0,0,98,105,5,4,0,0,99,106,5,23,0,0,100,106,5,24,0,0,101,
        106,5,35,0,0,102,106,5,33,0,0,103,106,3,44,22,0,104,106,3,34,17,
        0,105,99,1,0,0,0,105,100,1,0,0,0,105,101,1,0,0,0,105,102,1,0,0,0,
        105,103,1,0,0,0,105,104,1,0,0,0,106,145,1,0,0,0,107,108,5,33,0,0,
        108,109,5,40,0,0,109,110,5,4,0,0,110,145,3,34,17,0,111,145,3,8,4,
        0,112,113,5,25,0,0,113,114,5,1,0,0,114,115,3,0,0,0,115,116,5,2,0,
        0,116,117,5,5,0,0,117,118,3,2,1,0,118,130,5,6,0,0,119,120,5,26,0,
        0,120,121,5,25,0,0,121,122,5,1,0,0,122,123,3,0,0,0,123,124,5,2,0,
        0,124,125,5,5,0,0,125,126,3,2,1,0,126,127,5,6,0,0,127,129,1,0,0,
        0,128,119,1,0,0,0,129,132,1,0,0,0,130,128,1,0,0,0,130,131,1,0,0,
        0,131,138,1,0,0,0,132,130,1,0,0,0,133,134,5,26,0,0,134,135,5,5,0,
        0,135,136,3,2,1,0,136,137,5,6,0,0,137,139,1,0,0,0,138,133,1,0,0,
        0,138,139,1,0,0,0,139,145,1,0,0,0,140,141,5,30,0,0,141,142,5,33,
        0,0,142,143,5,7,0,0,143,145,5,33,0,0,144,97,1,0,0,0,144,107,1,0,
        0,0,144,111,1,0,0,0,144,112,1,0,0,0,144,140,1,0,0,0,145,5,1,0,0,
        0,146,147,5,34,0,0,147,148,5,1,0,0,148,149,5,36,0,0,149,150,5,7,
        0,0,150,151,3,0,0,0,151,152,5,2,0,0,152,159,1,0,0,0,153,154,5,34,
        0,0,154,155,5,1,0,0,155,156,3,0,0,0,156,157,5,2,0,0,157,159,1,0,
        0,0,158,146,1,0,0,0,158,153,1,0,0,0,159,7,1,0,0,0,160,162,5,22,0,
        0,161,160,1,0,0,0,161,162,1,0,0,0,162,163,1,0,0,0,163,164,5,34,0,
        0,164,165,5,1,0,0,165,170,5,31,0,0,166,167,5,7,0,0,167,169,5,31,
        0,0,168,166,1,0,0,0,169,172,1,0,0,0,170,168,1,0,0,0,170,171,1,0,
        0,0,171,173,1,0,0,0,172,170,1,0,0,0,173,240,5,2,0,0,174,176,5,22,
        0,0,175,174,1,0,0,0,175,176,1,0,0,0,176,177,1,0,0,0,177,178,5,34,
        0,0,178,179,5,1,0,0,179,180,3,38,19,0,180,181,5,2,0,0,181,240,1,
        0,0,0,182,184,5,22,0,0,183,182,1,0,0,0,183,184,1,0,0,0,184,185,1,
        0,0,0,185,186,5,34,0,0,186,187,5,1,0,0,187,188,5,36,0,0,188,240,
        5,2,0,0,189,191,5,22,0,0,190,189,1,0,0,0,190,191,1,0,0,0,191,192,
        1,0,0,0,192,193,5,34,0,0,193,194,5,1,0,0,194,195,5,38,0,0,195,240,
        5,2,0,0,196,198,5,22,0,0,197,196,1,0,0,0,197,198,1,0,0,0,198,199,
        1,0,0,0,199,200,5,34,0,0,200,201,5,1,0,0,201,202,5,39,0,0,202,240,
        5,2,0,0,203,205,5,22,0,0,204,203,1,0,0,0,204,205,1,0,0,0,205,206,
        1,0,0,0,206,207,5,34,0,0,207,208,5,1,0,0,208,213,5,35,0,0,209,210,
        5,7,0,0,210,212,5,35,0,0,211,209,1,0,0,0,212,215,1,0,0,0,213,211,
        1,0,0,0,213,214,1,0,0,0,214,216,1,0,0,0,215,213,1,0,0,0,216,240,
        5,2,0,0,217,219,5,22,0,0,218,217,1,0,0,0,218,219,1,0,0,0,219,220,
        1,0,0,0,220,221,5,34,0,0,221,222,5,1,0,0,222,227,5,33,0,0,223,224,
        5,7,0,0,224,226,5,33,0,0,225,223,1,0,0,0,226,229,1,0,0,0,227,225,
        1,0,0,0,227,228,1,0,0,0,228,230,1,0,0,0,229,227,1,0,0,0,230,240,
        5,2,0,0,231,233,5,22,0,0,232,231,1,0,0,0,232,233,1,0,0,0,233,234,
        1,0,0,0,234,237,5,34,0,0,235,236,5,1,0,0,236,238,5,2,0,0,237,235,
        1,0,0,0,237,238,1,0,0,0,238,240,1,0,0,0,239,161,1,0,0,0,239,175,
        1,0,0,0,239,183,1,0,0,0,239,190,1,0,0,0,239,197,1,0,0,0,239,204,
        1,0,0,0,239,218,1,0,0,0,239,232,1,0,0,0,240,9,1,0,0,0,241,242,5,
        25,0,0,242,243,5,1,0,0,243,244,3,0,0,0,244,245,5,2,0,0,245,246,5,
        5,0,0,246,247,3,0,0,0,247,259,5,6,0,0,248,249,5,26,0,0,249,250,5,
        25,0,0,250,251,5,1,0,0,251,252,3,0,0,0,252,253,5,2,0,0,253,254,5,
        5,0,0,254,255,3,0,0,0,255,256,5,6,0,0,256,258,1,0,0,0,257,248,1,
        0,0,0,258,261,1,0,0,0,259,257,1,0,0,0,259,260,1,0,0,0,260,267,1,
        0,0,0,261,259,1,0,0,0,262,263,5,26,0,0,263,264,5,5,0,0,264,265,3,
        0,0,0,265,266,5,6,0,0,266,268,1,0,0,0,267,262,1,0,0,0,267,268,1,
        0,0,0,268,278,1,0,0,0,269,270,5,1,0,0,270,271,3,0,0,0,271,272,5,
        25,0,0,272,273,3,0,0,0,273,274,5,26,0,0,274,275,3,0,0,0,275,276,
        5,2,0,0,276,278,1,0,0,0,277,241,1,0,0,0,277,269,1,0,0,0,278,11,1,
        0,0,0,279,280,5,25,0,0,280,281,5,1,0,0,281,282,3,0,0,0,282,283,5,
        2,0,0,283,284,5,5,0,0,284,285,3,34,17,0,285,297,5,6,0,0,286,287,
        5,26,0,0,287,288,5,25,0,0,288,289,5,1,0,0,289,290,3,0,0,0,290,291,
        5,2,0,0,291,292,5,5,0,0,292,293,3,34,17,0,293,294,5,6,0,0,294,296,
        1,0,0,0,295,286,1,0,0,0,296,299,1,0,0,0,297,295,1,0,0,0,297,298,
        1,0,0,0,298,305,1,0,0,0,299,297,1,0,0,0,300,301,5,26,0,0,301,302,
        5,5,0,0,302,303,3,34,17,0,303,304,5,6,0,0,304,306,1,0,0,0,305,300,
        1,0,0,0,305,306,1,0,0,0,306,13,1,0,0,0,307,308,5,25,0,0,308,309,
        5,1,0,0,309,310,3,0,0,0,310,311,5,2,0,0,311,312,5,5,0,0,312,313,
        3,44,22,0,313,325,5,6,0,0,314,315,5,26,0,0,315,316,5,25,0,0,316,
        317,5,1,0,0,317,318,3,0,0,0,318,319,5,2,0,0,319,320,5,5,0,0,320,
        321,3,44,22,0,321,322,5,6,0,0,322,324,1,0,0,0,323,314,1,0,0,0,324,
        327,1,0,0,0,325,323,1,0,0,0,325,326,1,0,0,0,326,333,1,0,0,0,327,
        325,1,0,0,0,328,329,5,26,0,0,329,330,5,5,0,0,330,331,3,44,22,0,331,
        332,5,6,0,0,332,334,1,0,0,0,333,328,1,0,0,0,333,334,1,0,0,0,334,
        15,1,0,0,0,335,336,5,28,0,0,336,337,5,31,0,0,337,343,5,5,0,0,338,
        339,5,38,0,0,339,340,5,8,0,0,340,341,3,0,0,0,341,342,5,7,0,0,342,
        344,1,0,0,0,343,338,1,0,0,0,344,345,1,0,0,0,345,343,1,0,0,0,345,
        346,1,0,0,0,346,347,1,0,0,0,347,348,5,9,0,0,348,349,5,8,0,0,349,
        351,3,0,0,0,350,352,5,7,0,0,351,350,1,0,0,0,351,352,1,0,0,0,352,
        353,1,0,0,0,353,354,5,6,0,0,354,436,1,0,0,0,355,356,5,28,0,0,356,
        357,5,32,0,0,357,376,5,5,0,0,358,359,5,38,0,0,359,360,5,8,0,0,360,
        361,3,0,0,0,361,362,5,7,0,0,362,364,1,0,0,0,363,358,1,0,0,0,364,
        365,1,0,0,0,365,363,1,0,0,0,365,366,1,0,0,0,366,377,1,0,0,0,367,
        368,5,36,0,0,368,369,5,8,0,0,369,370,3,0,0,0,370,371,5,7,0,0,371,
        373,1,0,0,0,372,367,1,0,0,0,373,374,1,0,0,0,374,372,1,0,0,0,374,
        375,1,0,0,0,375,377,1,0,0,0,376,363,1,0,0,0,376,372,1,0,0,0,377,
        378,1,0,0,0,378,379,5,9,0,0,379,380,5,8,0,0,380,382,3,0,0,0,381,
        383,5,7,0,0,382,381,1,0,0,0,382,383,1,0,0,0,383,384,1,0,0,0,384,
        385,5,6,0,0,385,436,1,0,0,0,386,387,5,28,0,0,387,388,5,33,0,0,388,
        401,5,5,0,0,389,394,5,31,0,0,390,391,5,10,0,0,391,393,5,31,0,0,392,
        390,1,0,0,0,393,396,1,0,0,0,394,392,1,0,0,0,394,395,1,0,0,0,395,
        397,1,0,0,0,396,394,1,0,0,0,397,398,5,8,0,0,398,399,3,0,0,0,399,
        400,5,7,0,0,400,402,1,0,0,0,401,389,1,0,0,0,402,403,1,0,0,0,403,
        401,1,0,0,0,403,404,1,0,0,0,404,405,1,0,0,0,405,406,5,9,0,0,406,
        407,5,8,0,0,407,409,3,0,0,0,408,410,5,7,0,0,409,408,1,0,0,0,409,
        410,1,0,0,0,410,411,1,0,0,0,411,412,5,6,0,0,412,436,1,0,0,0,413,
        414,5,33,0,0,414,415,5,27,0,0,415,416,5,11,0,0,416,419,5,31,0,0,
        417,418,5,7,0,0,418,420,5,31,0,0,419,417,1,0,0,0,420,421,1,0,0,0,
        421,419,1,0,0,0,421,422,1,0,0,0,422,423,1,0,0,0,423,436,5,12,0,0,
        424,425,5,33,0,0,425,426,5,27,0,0,426,427,5,11,0,0,427,430,5,36,
        0,0,428,429,5,7,0,0,429,431,5,36,0,0,430,428,1,0,0,0,431,432,1,0,
        0,0,432,430,1,0,0,0,432,433,1,0,0,0,433,434,1,0,0,0,434,436,5,12,
        0,0,435,335,1,0,0,0,435,355,1,0,0,0,435,386,1,0,0,0,435,413,1,0,
        0,0,435,424,1,0,0,0,436,17,1,0,0,0,437,438,5,28,0,0,438,439,5,31,
        0,0,439,445,5,5,0,0,440,441,5,38,0,0,441,442,5,8,0,0,442,443,3,34,
        17,0,443,444,5,7,0,0,444,446,1,0,0,0,445,440,1,0,0,0,446,447,1,0,
        0,0,447,445,1,0,0,0,447,448,1,0,0,0,448,449,1,0,0,0,449,450,5,9,
        0,0,450,451,5,8,0,0,451,453,3,34,17,0,452,454,5,7,0,0,453,452,1,
        0,0,0,453,454,1,0,0,0,454,455,1,0,0,0,455,456,5,6,0,0,456,520,1,
        0,0,0,457,458,5,28,0,0,458,459,5,33,0,0,459,478,5,5,0,0,460,461,
        5,38,0,0,461,462,5,8,0,0,462,463,3,34,17,0,463,464,5,7,0,0,464,466,
        1,0,0,0,465,460,1,0,0,0,466,467,1,0,0,0,467,465,1,0,0,0,467,468,
        1,0,0,0,468,479,1,0,0,0,469,470,5,36,0,0,470,471,5,8,0,0,471,472,
        3,34,17,0,472,473,5,7,0,0,473,475,1,0,0,0,474,469,1,0,0,0,475,476,
        1,0,0,0,476,474,1,0,0,0,476,477,1,0,0,0,477,479,1,0,0,0,478,465,
        1,0,0,0,478,474,1,0,0,0,479,480,1,0,0,0,480,481,5,9,0,0,481,482,
        5,8,0,0,482,484,3,34,17,0,483,485,5,7,0,0,484,483,1,0,0,0,484,485,
        1,0,0,0,485,486,1,0,0,0,486,487,5,6,0,0,487,520,1,0,0,0,488,489,
        5,28,0,0,489,490,5,32,0,0,490,509,5,5,0,0,491,492,5,38,0,0,492,493,
        5,8,0,0,493,494,3,34,17,0,494,495,5,7,0,0,495,497,1,0,0,0,496,491,
        1,0,0,0,497,498,1,0,0,0,498,496,1,0,0,0,498,499,1,0,0,0,499,510,
        1,0,0,0,500,501,5,36,0,0,501,502,5,8,0,0,502,503,3,34,17,0,503,504,
        5,7,0,0,504,506,1,0,0,0,505,500,1,0,0,0,506,507,1,0,0,0,507,505,
        1,0,0,0,507,508,1,0,0,0,508,510,1,0,0,0,509,496,1,0,0,0,509,505,
        1,0,0,0,510,511,1,0,0,0,511,512,5,9,0,0,512,513,5,8,0,0,513,515,
        3,34,17,0,514,516,5,7,0,0,515,514,1,0,0,0,515,516,1,0,0,0,516,517,
        1,0,0,0,517,518,5,6,0,0,518,520,1,0,0,0,519,437,1,0,0,0,519,457,
        1,0,0,0,519,488,1,0,0,0,520,19,1,0,0,0,521,522,5,28,0,0,522,523,
        5,31,0,0,523,529,5,5,0,0,524,525,5,38,0,0,525,526,5,8,0,0,526,527,
        3,44,22,0,527,528,5,7,0,0,528,530,1,0,0,0,529,524,1,0,0,0,530,531,
        1,0,0,0,531,529,1,0,0,0,531,532,1,0,0,0,532,533,1,0,0,0,533,534,
        5,9,0,0,534,535,5,8,0,0,535,537,3,44,22,0,536,538,5,7,0,0,537,536,
        1,0,0,0,537,538,1,0,0,0,538,539,1,0,0,0,539,540,5,6,0,0,540,604,
        1,0,0,0,541,542,5,28,0,0,542,543,5,33,0,0,543,562,5,5,0,0,544,545,
        5,38,0,0,545,546,5,8,0,0,546,547,3,44,22,0,547,548,5,7,0,0,548,550,
        1,0,0,0,549,544,1,0,0,0,550,551,1,0,0,0,551,549,1,0,0,0,551,552,
        1,0,0,0,552,563,1,0,0,0,553,554,5,36,0,0,554,555,5,8,0,0,555,556,
        3,44,22,0,556,557,5,7,0,0,557,559,1,0,0,0,558,553,1,0,0,0,559,560,
        1,0,0,0,560,558,1,0,0,0,560,561,1,0,0,0,561,563,1,0,0,0,562,549,
        1,0,0,0,562,558,1,0,0,0,563,564,1,0,0,0,564,565,5,9,0,0,565,566,
        5,8,0,0,566,568,3,44,22,0,567,569,5,7,0,0,568,567,1,0,0,0,568,569,
        1,0,0,0,569,570,1,0,0,0,570,571,5,6,0,0,571,604,1,0,0,0,572,573,
        5,28,0,0,573,574,5,32,0,0,574,593,5,5,0,0,575,576,5,38,0,0,576,577,
        5,8,0,0,577,578,3,44,22,0,578,579,5,7,0,0,579,581,1,0,0,0,580,575,
        1,0,0,0,581,582,1,0,0,0,582,580,1,0,0,0,582,583,1,0,0,0,583,594,
        1,0,0,0,584,585,5,36,0,0,585,586,5,8,0,0,586,587,3,44,22,0,587,588,
        5,7,0,0,588,590,1,0,0,0,589,584,1,0,0,0,590,591,1,0,0,0,591,589,
        1,0,0,0,591,592,1,0,0,0,592,594,1,0,0,0,593,580,1,0,0,0,593,589,
        1,0,0,0,594,595,1,0,0,0,595,596,5,9,0,0,596,597,5,8,0,0,597,599,
        3,44,22,0,598,600,5,7,0,0,599,598,1,0,0,0,599,600,1,0,0,0,600,601,
        1,0,0,0,601,602,5,6,0,0,602,604,1,0,0,0,603,521,1,0,0,0,603,541,
        1,0,0,0,603,572,1,0,0,0,604,21,1,0,0,0,605,606,3,38,19,0,606,607,
        5,13,0,0,607,608,3,34,17,0,608,630,1,0,0,0,609,610,3,38,19,0,610,
        611,5,14,0,0,611,612,3,34,17,0,612,630,1,0,0,0,613,614,3,38,19,0,
        614,615,5,15,0,0,615,616,3,34,17,0,616,630,1,0,0,0,617,618,3,38,
        19,0,618,619,5,16,0,0,619,620,3,34,17,0,620,630,1,0,0,0,621,622,
        3,38,19,0,622,623,5,17,0,0,623,624,3,34,17,0,624,630,1,0,0,0,625,
        626,3,38,19,0,626,627,5,18,0,0,627,628,3,34,17,0,628,630,1,0,0,0,
        629,605,1,0,0,0,629,609,1,0,0,0,629,613,1,0,0,0,629,617,1,0,0,0,
        629,621,1,0,0,0,629,625,1,0,0,0,630,23,1,0,0,0,631,632,3,38,19,0,
        632,633,5,13,0,0,633,634,5,36,0,0,634,640,1,0,0,0,635,636,3,38,19,
        0,636,637,5,14,0,0,637,638,5,36,0,0,638,640,1,0,0,0,639,631,1,0,
        0,0,639,635,1,0,0,0,640,25,1,0,0,0,641,642,3,38,19,0,642,643,5,19,
        0,0,643,644,3,34,17,0,644,27,1,0,0,0,645,646,5,33,0,0,646,647,5,
        13,0,0,647,648,7,0,0,0,648,29,1,0,0,0,649,650,5,34,0,0,650,651,5,
        1,0,0,651,652,5,31,0,0,652,671,5,2,0,0,653,654,5,34,0,0,654,655,
        5,1,0,0,655,660,3,34,17,0,656,657,5,7,0,0,657,659,3,34,17,0,658,
        656,1,0,0,0,659,662,1,0,0,0,660,658,1,0,0,0,660,661,1,0,0,0,661,
        663,1,0,0,0,662,660,1,0,0,0,663,664,5,2,0,0,664,671,1,0,0,0,665,
        668,5,34,0,0,666,667,5,1,0,0,667,669,5,2,0,0,668,666,1,0,0,0,668,
        669,1,0,0,0,669,671,1,0,0,0,670,649,1,0,0,0,670,653,1,0,0,0,670,
        665,1,0,0,0,671,31,1,0,0,0,672,673,3,36,18,0,673,674,5,40,0,0,674,
        675,3,34,17,0,675,33,1,0,0,0,676,679,3,36,18,0,677,679,3,32,16,0,
        678,676,1,0,0,0,678,677,1,0,0,0,679,35,1,0,0,0,680,689,5,38,0,0,
        681,689,5,37,0,0,682,689,5,32,0,0,683,689,5,33,0,0,684,689,3,38,
        19,0,685,689,3,18,9,0,686,689,3,30,15,0,687,689,3,12,6,0,688,680,
        1,0,0,0,688,681,1,0,0,0,688,682,1,0,0,0,688,683,1,0,0,0,688,684,
        1,0,0,0,688,685,1,0,0,0,688,686,1,0,0,0,688,687,1,0,0,0,689,37,1,
        0,0,0,690,694,5,32,0,0,691,692,5,11,0,0,692,693,7,1,0,0,693,695,
        5,12,0,0,694,691,1,0,0,0,694,695,1,0,0,0,695,698,1,0,0,0,696,698,
        5,33,0,0,697,690,1,0,0,0,697,696,1,0,0,0,698,39,1,0,0,0,699,702,
        5,11,0,0,700,703,5,34,0,0,701,703,3,42,21,0,702,700,1,0,0,0,702,
        701,1,0,0,0,703,711,1,0,0,0,704,707,5,7,0,0,705,708,5,34,0,0,706,
        708,3,42,21,0,707,705,1,0,0,0,707,706,1,0,0,0,708,710,1,0,0,0,709,
        704,1,0,0,0,710,713,1,0,0,0,711,709,1,0,0,0,711,712,1,0,0,0,712,
        714,1,0,0,0,713,711,1,0,0,0,714,715,5,12,0,0,715,41,1,0,0,0,716,
        717,5,31,0,0,717,718,5,5,0,0,718,719,7,2,0,0,719,726,5,6,0,0,720,
        721,5,1,0,0,721,722,5,31,0,0,722,723,5,7,0,0,723,724,7,2,0,0,724,
        726,5,2,0,0,725,716,1,0,0,0,725,720,1,0,0,0,726,734,1,0,0,0,727,
        729,5,22,0,0,728,727,1,0,0,0,728,729,1,0,0,0,729,730,1,0,0,0,730,
        734,5,31,0,0,731,734,5,36,0,0,732,734,5,33,0,0,733,725,1,0,0,0,733,
        728,1,0,0,0,733,731,1,0,0,0,733,732,1,0,0,0,734,43,1,0,0,0,735,740,
        5,36,0,0,736,740,3,38,19,0,737,740,3,14,7,0,738,740,3,20,10,0,739,
        735,1,0,0,0,739,736,1,0,0,0,739,737,1,0,0,0,739,738,1,0,0,0,740,
        45,1,0,0,0,741,743,5,22,0,0,742,741,1,0,0,0,742,743,1,0,0,0,743,
        744,1,0,0,0,744,745,5,29,0,0,745,761,5,35,0,0,746,748,5,22,0,0,747,
        746,1,0,0,0,747,748,1,0,0,0,748,749,1,0,0,0,749,750,5,29,0,0,750,
        751,5,1,0,0,751,756,5,35,0,0,752,753,5,7,0,0,753,755,5,35,0,0,754,
        752,1,0,0,0,755,758,1,0,0,0,756,754,1,0,0,0,756,757,1,0,0,0,757,
        759,1,0,0,0,758,756,1,0,0,0,759,761,5,2,0,0,760,742,1,0,0,0,760,
        747,1,0,0,0,761,47,1,0,0,0,762,764,5,33,0,0,763,765,5,22,0,0,764,
        763,1,0,0,0,764,765,1,0,0,0,765,766,1,0,0,0,766,767,5,29,0,0,767,
        781,5,33,0,0,768,770,5,33,0,0,769,771,5,22,0,0,770,769,1,0,0,0,770,
        771,1,0,0,0,771,772,1,0,0,0,772,773,5,29,0,0,773,781,5,35,0,0,774,
        776,5,33,0,0,775,777,5,22,0,0,776,775,1,0,0,0,776,777,1,0,0,0,777,
        778,1,0,0,0,778,779,5,29,0,0,779,781,3,8,4,0,780,762,1,0,0,0,780,
        768,1,0,0,0,780,774,1,0,0,0,781,49,1,0,0,0,89,66,73,81,83,91,95,
        105,130,138,144,158,161,170,175,183,190,197,204,213,218,227,232,
        237,239,259,267,277,297,305,325,333,345,351,365,374,376,382,394,
        403,409,421,432,435,447,453,467,476,478,484,498,507,509,515,519,
        531,537,551,560,562,568,582,591,593,599,603,629,639,660,668,670,
        678,688,694,697,702,707,711,725,728,733,739,742,747,756,760,764,
        770,776,780
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
                      "IN", "PER", "WITHIN", "SWAP", "ITEM", "SETTING", 
                      "REF", "FUNC", "PLACE", "LIT", "CONST", "INT", "FLOAT", 
                      "BINOP", "WS" ]

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
    SWAP=30
    ITEM=31
    SETTING=32
    REF=33
    FUNC=34
    PLACE=35
    LIT=36
    CONST=37
    INT=38
    FLOAT=39
    BINOP=40
    WS=41

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


    class SwapContext(ActionContext):

        def __init__(self, parser, ctx:ParserRuleContext): # actually a RulesParser.ActionContext
            super().__init__(parser)
            self.copyFrom(ctx)

        def SWAP(self):
            return self.getToken(RulesParser.SWAP, 0)
        def REF(self, i:int=None):
            if i is None:
                return self.getTokens(RulesParser.REF)
            else:
                return self.getToken(RulesParser.REF, i)

        def enterRule(self, listener:ParseTreeListener):
            if hasattr( listener, "enterSwap" ):
                listener.enterSwap(self)

        def exitRule(self, listener:ParseTreeListener):
            if hasattr( listener, "exitSwap" ):
                listener.exitSwap(self)

        def accept(self, visitor:ParseTreeVisitor):
            if hasattr( visitor, "visitSwap" ):
                return visitor.visitSwap(self)
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
            self.state = 144
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

            elif la_ == 5:
                localctx = RulesParser.SwapContext(self, localctx)
                self.enterOuterAlt(localctx, 5)
                self.state = 140
                self.match(RulesParser.SWAP)
                self.state = 141
                self.match(RulesParser.REF)
                self.state = 142
                self.match(RulesParser.T__6)
                self.state = 143
                self.match(RulesParser.REF)
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
            self.state = 158
            self._errHandler.sync(self)
            la_ = self._interp.adaptivePredict(self._input,10,self._ctx)
            if la_ == 1:
                self.enterOuterAlt(localctx, 1)
                self.state = 146
                self.match(RulesParser.FUNC)
                self.state = 147
                self.match(RulesParser.T__0)
                self.state = 148
                self.match(RulesParser.LIT)
                self.state = 149
                self.match(RulesParser.T__6)
                self.state = 150
                self.boolExpr(0)
                self.state = 151
                self.match(RulesParser.T__1)
                pass

            elif la_ == 2:
                self.enterOuterAlt(localctx, 2)
                self.state = 153
                self.match(RulesParser.FUNC)
                self.state = 154
                self.match(RulesParser.T__0)
                self.state = 155
                self.boolExpr(0)
                self.state = 156
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

        def REF(self, i:int=None):
            if i is None:
                return self.getTokens(RulesParser.REF)
            else:
                return self.getToken(RulesParser.REF, i)

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
            self.state = 239
            self._errHandler.sync(self)
            la_ = self._interp.adaptivePredict(self._input,23,self._ctx)
            if la_ == 1:
                self.enterOuterAlt(localctx, 1)
                self.state = 161
                self._errHandler.sync(self)
                _la = self._input.LA(1)
                if _la==22:
                    self.state = 160
                    self.match(RulesParser.NOT)


                self.state = 163
                self.match(RulesParser.FUNC)
                self.state = 164
                self.match(RulesParser.T__0)
                self.state = 165
                self.match(RulesParser.ITEM)
                self.state = 170
                self._errHandler.sync(self)
                _la = self._input.LA(1)
                while _la==7:
                    self.state = 166
                    self.match(RulesParser.T__6)
                    self.state = 167
                    self.match(RulesParser.ITEM)
                    self.state = 172
                    self._errHandler.sync(self)
                    _la = self._input.LA(1)

                self.state = 173
                self.match(RulesParser.T__1)
                pass

            elif la_ == 2:
                self.enterOuterAlt(localctx, 2)
                self.state = 175
                self._errHandler.sync(self)
                _la = self._input.LA(1)
                if _la==22:
                    self.state = 174
                    self.match(RulesParser.NOT)


                self.state = 177
                self.match(RulesParser.FUNC)
                self.state = 178
                self.match(RulesParser.T__0)
                self.state = 179
                self.value()
                self.state = 180
                self.match(RulesParser.T__1)
                pass

            elif la_ == 3:
                self.enterOuterAlt(localctx, 3)
                self.state = 183
                self._errHandler.sync(self)
                _la = self._input.LA(1)
                if _la==22:
                    self.state = 182
                    self.match(RulesParser.NOT)


                self.state = 185
                self.match(RulesParser.FUNC)
                self.state = 186
                self.match(RulesParser.T__0)
                self.state = 187
                self.match(RulesParser.LIT)
                self.state = 188
                self.match(RulesParser.T__1)
                pass

            elif la_ == 4:
                self.enterOuterAlt(localctx, 4)
                self.state = 190
                self._errHandler.sync(self)
                _la = self._input.LA(1)
                if _la==22:
                    self.state = 189
                    self.match(RulesParser.NOT)


                self.state = 192
                self.match(RulesParser.FUNC)
                self.state = 193
                self.match(RulesParser.T__0)
                self.state = 194
                self.match(RulesParser.INT)
                self.state = 195
                self.match(RulesParser.T__1)
                pass

            elif la_ == 5:
                self.enterOuterAlt(localctx, 5)
                self.state = 197
                self._errHandler.sync(self)
                _la = self._input.LA(1)
                if _la==22:
                    self.state = 196
                    self.match(RulesParser.NOT)


                self.state = 199
                self.match(RulesParser.FUNC)
                self.state = 200
                self.match(RulesParser.T__0)
                self.state = 201
                self.match(RulesParser.FLOAT)
                self.state = 202
                self.match(RulesParser.T__1)
                pass

            elif la_ == 6:
                self.enterOuterAlt(localctx, 6)
                self.state = 204
                self._errHandler.sync(self)
                _la = self._input.LA(1)
                if _la==22:
                    self.state = 203
                    self.match(RulesParser.NOT)


                self.state = 206
                self.match(RulesParser.FUNC)
                self.state = 207
                self.match(RulesParser.T__0)
                self.state = 208
                self.match(RulesParser.PLACE)
                self.state = 213
                self._errHandler.sync(self)
                _la = self._input.LA(1)
                while _la==7:
                    self.state = 209
                    self.match(RulesParser.T__6)
                    self.state = 210
                    self.match(RulesParser.PLACE)
                    self.state = 215
                    self._errHandler.sync(self)
                    _la = self._input.LA(1)

                self.state = 216
                self.match(RulesParser.T__1)
                pass

            elif la_ == 7:
                self.enterOuterAlt(localctx, 7)
                self.state = 218
                self._errHandler.sync(self)
                _la = self._input.LA(1)
                if _la==22:
                    self.state = 217
                    self.match(RulesParser.NOT)


                self.state = 220
                self.match(RulesParser.FUNC)
                self.state = 221
                self.match(RulesParser.T__0)
                self.state = 222
                self.match(RulesParser.REF)
                self.state = 227
                self._errHandler.sync(self)
                _la = self._input.LA(1)
                while _la==7:
                    self.state = 223
                    self.match(RulesParser.T__6)
                    self.state = 224
                    self.match(RulesParser.REF)
                    self.state = 229
                    self._errHandler.sync(self)
                    _la = self._input.LA(1)

                self.state = 230
                self.match(RulesParser.T__1)
                pass

            elif la_ == 8:
                self.enterOuterAlt(localctx, 8)
                self.state = 232
                self._errHandler.sync(self)
                _la = self._input.LA(1)
                if _la==22:
                    self.state = 231
                    self.match(RulesParser.NOT)


                self.state = 234
                self.match(RulesParser.FUNC)
                self.state = 237
                self._errHandler.sync(self)
                la_ = self._interp.adaptivePredict(self._input,22,self._ctx)
                if la_ == 1:
                    self.state = 235
                    self.match(RulesParser.T__0)
                    self.state = 236
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
            self.state = 277
            self._errHandler.sync(self)
            token = self._input.LA(1)
            if token in [25]:
                localctx = RulesParser.IfThenElseContext(self, localctx)
                self.enterOuterAlt(localctx, 1)
                self.state = 241
                self.match(RulesParser.IF)
                self.state = 242
                self.match(RulesParser.T__0)
                self.state = 243
                self.boolExpr(0)
                self.state = 244
                self.match(RulesParser.T__1)
                self.state = 245
                self.match(RulesParser.T__4)
                self.state = 246
                self.boolExpr(0)
                self.state = 247
                self.match(RulesParser.T__5)
                self.state = 259
                self._errHandler.sync(self)
                _alt = self._interp.adaptivePredict(self._input,24,self._ctx)
                while _alt!=2 and _alt!=ATN.INVALID_ALT_NUMBER:
                    if _alt==1:
                        self.state = 248
                        self.match(RulesParser.ELSE)
                        self.state = 249
                        self.match(RulesParser.IF)
                        self.state = 250
                        self.match(RulesParser.T__0)
                        self.state = 251
                        self.boolExpr(0)
                        self.state = 252
                        self.match(RulesParser.T__1)
                        self.state = 253
                        self.match(RulesParser.T__4)
                        self.state = 254
                        self.boolExpr(0)
                        self.state = 255
                        self.match(RulesParser.T__5) 
                    self.state = 261
                    self._errHandler.sync(self)
                    _alt = self._interp.adaptivePredict(self._input,24,self._ctx)

                self.state = 267
                self._errHandler.sync(self)
                la_ = self._interp.adaptivePredict(self._input,25,self._ctx)
                if la_ == 1:
                    self.state = 262
                    self.match(RulesParser.ELSE)
                    self.state = 263
                    self.match(RulesParser.T__4)
                    self.state = 264
                    self.boolExpr(0)
                    self.state = 265
                    self.match(RulesParser.T__5)


                pass
            elif token in [1]:
                localctx = RulesParser.PyTernaryContext(self, localctx)
                self.enterOuterAlt(localctx, 2)
                self.state = 269
                self.match(RulesParser.T__0)
                self.state = 270
                self.boolExpr(0)
                self.state = 271
                self.match(RulesParser.IF)
                self.state = 272
                self.boolExpr(0)
                self.state = 273
                self.match(RulesParser.ELSE)
                self.state = 274
                self.boolExpr(0)
                self.state = 275
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
            self.state = 279
            self.match(RulesParser.IF)
            self.state = 280
            self.match(RulesParser.T__0)
            self.state = 281
            self.boolExpr(0)
            self.state = 282
            self.match(RulesParser.T__1)
            self.state = 283
            self.match(RulesParser.T__4)
            self.state = 284
            self.num()
            self.state = 285
            self.match(RulesParser.T__5)
            self.state = 297
            self._errHandler.sync(self)
            _alt = self._interp.adaptivePredict(self._input,27,self._ctx)
            while _alt!=2 and _alt!=ATN.INVALID_ALT_NUMBER:
                if _alt==1:
                    self.state = 286
                    self.match(RulesParser.ELSE)
                    self.state = 287
                    self.match(RulesParser.IF)
                    self.state = 288
                    self.match(RulesParser.T__0)
                    self.state = 289
                    self.boolExpr(0)
                    self.state = 290
                    self.match(RulesParser.T__1)
                    self.state = 291
                    self.match(RulesParser.T__4)
                    self.state = 292
                    self.num()
                    self.state = 293
                    self.match(RulesParser.T__5) 
                self.state = 299
                self._errHandler.sync(self)
                _alt = self._interp.adaptivePredict(self._input,27,self._ctx)

            self.state = 305
            self._errHandler.sync(self)
            la_ = self._interp.adaptivePredict(self._input,28,self._ctx)
            if la_ == 1:
                self.state = 300
                self.match(RulesParser.ELSE)
                self.state = 301
                self.match(RulesParser.T__4)
                self.state = 302
                self.num()
                self.state = 303
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
            self.state = 307
            self.match(RulesParser.IF)
            self.state = 308
            self.match(RulesParser.T__0)
            self.state = 309
            self.boolExpr(0)
            self.state = 310
            self.match(RulesParser.T__1)
            self.state = 311
            self.match(RulesParser.T__4)
            self.state = 312
            self.str_()
            self.state = 313
            self.match(RulesParser.T__5)
            self.state = 325
            self._errHandler.sync(self)
            _alt = self._interp.adaptivePredict(self._input,29,self._ctx)
            while _alt!=2 and _alt!=ATN.INVALID_ALT_NUMBER:
                if _alt==1:
                    self.state = 314
                    self.match(RulesParser.ELSE)
                    self.state = 315
                    self.match(RulesParser.IF)
                    self.state = 316
                    self.match(RulesParser.T__0)
                    self.state = 317
                    self.boolExpr(0)
                    self.state = 318
                    self.match(RulesParser.T__1)
                    self.state = 319
                    self.match(RulesParser.T__4)
                    self.state = 320
                    self.str_()
                    self.state = 321
                    self.match(RulesParser.T__5) 
                self.state = 327
                self._errHandler.sync(self)
                _alt = self._interp.adaptivePredict(self._input,29,self._ctx)

            self.state = 333
            self._errHandler.sync(self)
            _la = self._input.LA(1)
            if _la==26:
                self.state = 328
                self.match(RulesParser.ELSE)
                self.state = 329
                self.match(RulesParser.T__4)
                self.state = 330
                self.str_()
                self.state = 331
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
            self.state = 435
            self._errHandler.sync(self)
            la_ = self._interp.adaptivePredict(self._input,42,self._ctx)
            if la_ == 1:
                localctx = RulesParser.PerItemBoolContext(self, localctx)
                self.enterOuterAlt(localctx, 1)
                self.state = 335
                self.match(RulesParser.PER)
                self.state = 336
                self.match(RulesParser.ITEM)
                self.state = 337
                self.match(RulesParser.T__4)
                self.state = 343 
                self._errHandler.sync(self)
                _la = self._input.LA(1)
                while True:
                    self.state = 338
                    self.match(RulesParser.INT)
                    self.state = 339
                    self.match(RulesParser.T__7)
                    self.state = 340
                    self.boolExpr(0)
                    self.state = 341
                    self.match(RulesParser.T__6)
                    self.state = 345 
                    self._errHandler.sync(self)
                    _la = self._input.LA(1)
                    if not (_la==38):
                        break

                self.state = 347
                self.match(RulesParser.T__8)
                self.state = 348
                self.match(RulesParser.T__7)
                self.state = 349
                self.boolExpr(0)
                self.state = 351
                self._errHandler.sync(self)
                _la = self._input.LA(1)
                if _la==7:
                    self.state = 350
                    self.match(RulesParser.T__6)


                self.state = 353
                self.match(RulesParser.T__5)
                pass

            elif la_ == 2:
                localctx = RulesParser.PerSettingBoolContext(self, localctx)
                self.enterOuterAlt(localctx, 2)
                self.state = 355
                self.match(RulesParser.PER)
                self.state = 356
                self.match(RulesParser.SETTING)
                self.state = 357
                self.match(RulesParser.T__4)
                self.state = 376
                self._errHandler.sync(self)
                token = self._input.LA(1)
                if token in [38]:
                    self.state = 363 
                    self._errHandler.sync(self)
                    _la = self._input.LA(1)
                    while True:
                        self.state = 358
                        self.match(RulesParser.INT)
                        self.state = 359
                        self.match(RulesParser.T__7)
                        self.state = 360
                        self.boolExpr(0)
                        self.state = 361
                        self.match(RulesParser.T__6)
                        self.state = 365 
                        self._errHandler.sync(self)
                        _la = self._input.LA(1)
                        if not (_la==38):
                            break

                    pass
                elif token in [36]:
                    self.state = 372 
                    self._errHandler.sync(self)
                    _la = self._input.LA(1)
                    while True:
                        self.state = 367
                        self.match(RulesParser.LIT)
                        self.state = 368
                        self.match(RulesParser.T__7)
                        self.state = 369
                        self.boolExpr(0)
                        self.state = 370
                        self.match(RulesParser.T__6)
                        self.state = 374 
                        self._errHandler.sync(self)
                        _la = self._input.LA(1)
                        if not (_la==36):
                            break

                    pass
                else:
                    raise NoViableAltException(self)

                self.state = 378
                self.match(RulesParser.T__8)
                self.state = 379
                self.match(RulesParser.T__7)
                self.state = 380
                self.boolExpr(0)
                self.state = 382
                self._errHandler.sync(self)
                _la = self._input.LA(1)
                if _la==7:
                    self.state = 381
                    self.match(RulesParser.T__6)


                self.state = 384
                self.match(RulesParser.T__5)
                pass

            elif la_ == 3:
                localctx = RulesParser.MatchRefBoolContext(self, localctx)
                self.enterOuterAlt(localctx, 3)
                self.state = 386
                self.match(RulesParser.PER)
                self.state = 387
                self.match(RulesParser.REF)
                self.state = 388
                self.match(RulesParser.T__4)
                self.state = 401 
                self._errHandler.sync(self)
                _la = self._input.LA(1)
                while True:
                    self.state = 389
                    self.match(RulesParser.ITEM)
                    self.state = 394
                    self._errHandler.sync(self)
                    _la = self._input.LA(1)
                    while _la==10:
                        self.state = 390
                        self.match(RulesParser.T__9)
                        self.state = 391
                        self.match(RulesParser.ITEM)
                        self.state = 396
                        self._errHandler.sync(self)
                        _la = self._input.LA(1)

                    self.state = 397
                    self.match(RulesParser.T__7)
                    self.state = 398
                    self.boolExpr(0)
                    self.state = 399
                    self.match(RulesParser.T__6)
                    self.state = 403 
                    self._errHandler.sync(self)
                    _la = self._input.LA(1)
                    if not (_la==31):
                        break

                self.state = 405
                self.match(RulesParser.T__8)
                self.state = 406
                self.match(RulesParser.T__7)
                self.state = 407
                self.boolExpr(0)
                self.state = 409
                self._errHandler.sync(self)
                _la = self._input.LA(1)
                if _la==7:
                    self.state = 408
                    self.match(RulesParser.T__6)


                self.state = 411
                self.match(RulesParser.T__5)
                pass

            elif la_ == 4:
                localctx = RulesParser.RefInListContext(self, localctx)
                self.enterOuterAlt(localctx, 4)
                self.state = 413
                self.match(RulesParser.REF)
                self.state = 414
                self.match(RulesParser.IN)
                self.state = 415
                self.match(RulesParser.T__10)
                self.state = 416
                self.match(RulesParser.ITEM)
                self.state = 419 
                self._errHandler.sync(self)
                _la = self._input.LA(1)
                while True:
                    self.state = 417
                    self.match(RulesParser.T__6)
                    self.state = 418
                    self.match(RulesParser.ITEM)
                    self.state = 421 
                    self._errHandler.sync(self)
                    _la = self._input.LA(1)
                    if not (_la==7):
                        break

                self.state = 423
                self.match(RulesParser.T__11)
                pass

            elif la_ == 5:
                localctx = RulesParser.RefStrInListContext(self, localctx)
                self.enterOuterAlt(localctx, 5)
                self.state = 424
                self.match(RulesParser.REF)
                self.state = 425
                self.match(RulesParser.IN)
                self.state = 426
                self.match(RulesParser.T__10)
                self.state = 427
                self.match(RulesParser.LIT)
                self.state = 430 
                self._errHandler.sync(self)
                _la = self._input.LA(1)
                while True:
                    self.state = 428
                    self.match(RulesParser.T__6)
                    self.state = 429
                    self.match(RulesParser.LIT)
                    self.state = 432 
                    self._errHandler.sync(self)
                    _la = self._input.LA(1)
                    if not (_la==7):
                        break

                self.state = 434
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
            self.state = 519
            self._errHandler.sync(self)
            la_ = self._interp.adaptivePredict(self._input,53,self._ctx)
            if la_ == 1:
                localctx = RulesParser.PerItemIntContext(self, localctx)
                self.enterOuterAlt(localctx, 1)
                self.state = 437
                self.match(RulesParser.PER)
                self.state = 438
                self.match(RulesParser.ITEM)
                self.state = 439
                self.match(RulesParser.T__4)
                self.state = 445 
                self._errHandler.sync(self)
                _la = self._input.LA(1)
                while True:
                    self.state = 440
                    self.match(RulesParser.INT)
                    self.state = 441
                    self.match(RulesParser.T__7)
                    self.state = 442
                    self.num()
                    self.state = 443
                    self.match(RulesParser.T__6)
                    self.state = 447 
                    self._errHandler.sync(self)
                    _la = self._input.LA(1)
                    if not (_la==38):
                        break

                self.state = 449
                self.match(RulesParser.T__8)
                self.state = 450
                self.match(RulesParser.T__7)
                self.state = 451
                self.num()
                self.state = 453
                self._errHandler.sync(self)
                _la = self._input.LA(1)
                if _la==7:
                    self.state = 452
                    self.match(RulesParser.T__6)


                self.state = 455
                self.match(RulesParser.T__5)
                pass

            elif la_ == 2:
                localctx = RulesParser.PerRefIntContext(self, localctx)
                self.enterOuterAlt(localctx, 2)
                self.state = 457
                self.match(RulesParser.PER)
                self.state = 458
                self.match(RulesParser.REF)
                self.state = 459
                self.match(RulesParser.T__4)
                self.state = 478
                self._errHandler.sync(self)
                token = self._input.LA(1)
                if token in [38]:
                    self.state = 465 
                    self._errHandler.sync(self)
                    _la = self._input.LA(1)
                    while True:
                        self.state = 460
                        self.match(RulesParser.INT)
                        self.state = 461
                        self.match(RulesParser.T__7)
                        self.state = 462
                        self.num()
                        self.state = 463
                        self.match(RulesParser.T__6)
                        self.state = 467 
                        self._errHandler.sync(self)
                        _la = self._input.LA(1)
                        if not (_la==38):
                            break

                    pass
                elif token in [36]:
                    self.state = 474 
                    self._errHandler.sync(self)
                    _la = self._input.LA(1)
                    while True:
                        self.state = 469
                        self.match(RulesParser.LIT)
                        self.state = 470
                        self.match(RulesParser.T__7)
                        self.state = 471
                        self.num()
                        self.state = 472
                        self.match(RulesParser.T__6)
                        self.state = 476 
                        self._errHandler.sync(self)
                        _la = self._input.LA(1)
                        if not (_la==36):
                            break

                    pass
                else:
                    raise NoViableAltException(self)

                self.state = 480
                self.match(RulesParser.T__8)
                self.state = 481
                self.match(RulesParser.T__7)
                self.state = 482
                self.num()
                self.state = 484
                self._errHandler.sync(self)
                _la = self._input.LA(1)
                if _la==7:
                    self.state = 483
                    self.match(RulesParser.T__6)


                self.state = 486
                self.match(RulesParser.T__5)
                pass

            elif la_ == 3:
                localctx = RulesParser.PerSettingIntContext(self, localctx)
                self.enterOuterAlt(localctx, 3)
                self.state = 488
                self.match(RulesParser.PER)
                self.state = 489
                self.match(RulesParser.SETTING)
                self.state = 490
                self.match(RulesParser.T__4)
                self.state = 509
                self._errHandler.sync(self)
                token = self._input.LA(1)
                if token in [38]:
                    self.state = 496 
                    self._errHandler.sync(self)
                    _la = self._input.LA(1)
                    while True:
                        self.state = 491
                        self.match(RulesParser.INT)
                        self.state = 492
                        self.match(RulesParser.T__7)
                        self.state = 493
                        self.num()
                        self.state = 494
                        self.match(RulesParser.T__6)
                        self.state = 498 
                        self._errHandler.sync(self)
                        _la = self._input.LA(1)
                        if not (_la==38):
                            break

                    pass
                elif token in [36]:
                    self.state = 505 
                    self._errHandler.sync(self)
                    _la = self._input.LA(1)
                    while True:
                        self.state = 500
                        self.match(RulesParser.LIT)
                        self.state = 501
                        self.match(RulesParser.T__7)
                        self.state = 502
                        self.num()
                        self.state = 503
                        self.match(RulesParser.T__6)
                        self.state = 507 
                        self._errHandler.sync(self)
                        _la = self._input.LA(1)
                        if not (_la==36):
                            break

                    pass
                else:
                    raise NoViableAltException(self)

                self.state = 511
                self.match(RulesParser.T__8)
                self.state = 512
                self.match(RulesParser.T__7)
                self.state = 513
                self.num()
                self.state = 515
                self._errHandler.sync(self)
                _la = self._input.LA(1)
                if _la==7:
                    self.state = 514
                    self.match(RulesParser.T__6)


                self.state = 517
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
            self.state = 603
            self._errHandler.sync(self)
            la_ = self._interp.adaptivePredict(self._input,64,self._ctx)
            if la_ == 1:
                localctx = RulesParser.PerItemStrContext(self, localctx)
                self.enterOuterAlt(localctx, 1)
                self.state = 521
                self.match(RulesParser.PER)
                self.state = 522
                self.match(RulesParser.ITEM)
                self.state = 523
                self.match(RulesParser.T__4)
                self.state = 529 
                self._errHandler.sync(self)
                _la = self._input.LA(1)
                while True:
                    self.state = 524
                    self.match(RulesParser.INT)
                    self.state = 525
                    self.match(RulesParser.T__7)
                    self.state = 526
                    self.str_()
                    self.state = 527
                    self.match(RulesParser.T__6)
                    self.state = 531 
                    self._errHandler.sync(self)
                    _la = self._input.LA(1)
                    if not (_la==38):
                        break

                self.state = 533
                self.match(RulesParser.T__8)
                self.state = 534
                self.match(RulesParser.T__7)
                self.state = 535
                self.str_()
                self.state = 537
                self._errHandler.sync(self)
                _la = self._input.LA(1)
                if _la==7:
                    self.state = 536
                    self.match(RulesParser.T__6)


                self.state = 539
                self.match(RulesParser.T__5)
                pass

            elif la_ == 2:
                localctx = RulesParser.PerRefStrContext(self, localctx)
                self.enterOuterAlt(localctx, 2)
                self.state = 541
                self.match(RulesParser.PER)
                self.state = 542
                self.match(RulesParser.REF)
                self.state = 543
                self.match(RulesParser.T__4)
                self.state = 562
                self._errHandler.sync(self)
                token = self._input.LA(1)
                if token in [38]:
                    self.state = 549 
                    self._errHandler.sync(self)
                    _la = self._input.LA(1)
                    while True:
                        self.state = 544
                        self.match(RulesParser.INT)
                        self.state = 545
                        self.match(RulesParser.T__7)
                        self.state = 546
                        self.str_()
                        self.state = 547
                        self.match(RulesParser.T__6)
                        self.state = 551 
                        self._errHandler.sync(self)
                        _la = self._input.LA(1)
                        if not (_la==38):
                            break

                    pass
                elif token in [36]:
                    self.state = 558 
                    self._errHandler.sync(self)
                    _la = self._input.LA(1)
                    while True:
                        self.state = 553
                        self.match(RulesParser.LIT)
                        self.state = 554
                        self.match(RulesParser.T__7)
                        self.state = 555
                        self.str_()
                        self.state = 556
                        self.match(RulesParser.T__6)
                        self.state = 560 
                        self._errHandler.sync(self)
                        _la = self._input.LA(1)
                        if not (_la==36):
                            break

                    pass
                else:
                    raise NoViableAltException(self)

                self.state = 564
                self.match(RulesParser.T__8)
                self.state = 565
                self.match(RulesParser.T__7)
                self.state = 566
                self.str_()
                self.state = 568
                self._errHandler.sync(self)
                _la = self._input.LA(1)
                if _la==7:
                    self.state = 567
                    self.match(RulesParser.T__6)


                self.state = 570
                self.match(RulesParser.T__5)
                pass

            elif la_ == 3:
                localctx = RulesParser.PerSettingStrContext(self, localctx)
                self.enterOuterAlt(localctx, 3)
                self.state = 572
                self.match(RulesParser.PER)
                self.state = 573
                self.match(RulesParser.SETTING)
                self.state = 574
                self.match(RulesParser.T__4)
                self.state = 593
                self._errHandler.sync(self)
                token = self._input.LA(1)
                if token in [38]:
                    self.state = 580 
                    self._errHandler.sync(self)
                    _la = self._input.LA(1)
                    while True:
                        self.state = 575
                        self.match(RulesParser.INT)
                        self.state = 576
                        self.match(RulesParser.T__7)
                        self.state = 577
                        self.str_()
                        self.state = 578
                        self.match(RulesParser.T__6)
                        self.state = 582 
                        self._errHandler.sync(self)
                        _la = self._input.LA(1)
                        if not (_la==38):
                            break

                    pass
                elif token in [36]:
                    self.state = 589 
                    self._errHandler.sync(self)
                    _la = self._input.LA(1)
                    while True:
                        self.state = 584
                        self.match(RulesParser.LIT)
                        self.state = 585
                        self.match(RulesParser.T__7)
                        self.state = 586
                        self.str_()
                        self.state = 587
                        self.match(RulesParser.T__6)
                        self.state = 591 
                        self._errHandler.sync(self)
                        _la = self._input.LA(1)
                        if not (_la==36):
                            break

                    pass
                else:
                    raise NoViableAltException(self)

                self.state = 595
                self.match(RulesParser.T__8)
                self.state = 596
                self.match(RulesParser.T__7)
                self.state = 597
                self.str_()
                self.state = 599
                self._errHandler.sync(self)
                _la = self._input.LA(1)
                if _la==7:
                    self.state = 598
                    self.match(RulesParser.T__6)


                self.state = 601
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
            self.state = 629
            self._errHandler.sync(self)
            la_ = self._interp.adaptivePredict(self._input,65,self._ctx)
            if la_ == 1:
                self.enterOuterAlt(localctx, 1)
                self.state = 605
                self.value()
                self.state = 606
                self.match(RulesParser.T__12)
                self.state = 607
                self.num()
                pass

            elif la_ == 2:
                self.enterOuterAlt(localctx, 2)
                self.state = 609
                self.value()
                self.state = 610
                self.match(RulesParser.T__13)
                self.state = 611
                self.num()
                pass

            elif la_ == 3:
                self.enterOuterAlt(localctx, 3)
                self.state = 613
                self.value()
                self.state = 614
                self.match(RulesParser.T__14)
                self.state = 615
                self.num()
                pass

            elif la_ == 4:
                self.enterOuterAlt(localctx, 4)
                self.state = 617
                self.value()
                self.state = 618
                self.match(RulesParser.T__15)
                self.state = 619
                self.num()
                pass

            elif la_ == 5:
                self.enterOuterAlt(localctx, 5)
                self.state = 621
                self.value()
                self.state = 622
                self.match(RulesParser.T__16)
                self.state = 623
                self.num()
                pass

            elif la_ == 6:
                self.enterOuterAlt(localctx, 6)
                self.state = 625
                self.value()
                self.state = 626
                self.match(RulesParser.T__17)
                self.state = 627
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
            self.state = 639
            self._errHandler.sync(self)
            la_ = self._interp.adaptivePredict(self._input,66,self._ctx)
            if la_ == 1:
                self.enterOuterAlt(localctx, 1)
                self.state = 631
                self.value()
                self.state = 632
                self.match(RulesParser.T__12)
                self.state = 633
                self.match(RulesParser.LIT)
                pass

            elif la_ == 2:
                self.enterOuterAlt(localctx, 2)
                self.state = 635
                self.value()
                self.state = 636
                self.match(RulesParser.T__13)
                self.state = 637
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
            self.state = 641
            self.value()
            self.state = 642
            self.match(RulesParser.T__18)
            self.state = 643
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
            self.state = 645
            self.match(RulesParser.REF)
            self.state = 646
            self.match(RulesParser.T__12)
            self.state = 647
            _la = self._input.LA(1)
            if not(_la==31 or _la==32):
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
            self.state = 670
            self._errHandler.sync(self)
            la_ = self._interp.adaptivePredict(self._input,69,self._ctx)
            if la_ == 1:
                self.enterOuterAlt(localctx, 1)
                self.state = 649
                self.match(RulesParser.FUNC)
                self.state = 650
                self.match(RulesParser.T__0)
                self.state = 651
                self.match(RulesParser.ITEM)
                self.state = 652
                self.match(RulesParser.T__1)
                pass

            elif la_ == 2:
                self.enterOuterAlt(localctx, 2)
                self.state = 653
                self.match(RulesParser.FUNC)
                self.state = 654
                self.match(RulesParser.T__0)
                self.state = 655
                self.num()
                self.state = 660
                self._errHandler.sync(self)
                _la = self._input.LA(1)
                while _la==7:
                    self.state = 656
                    self.match(RulesParser.T__6)
                    self.state = 657
                    self.num()
                    self.state = 662
                    self._errHandler.sync(self)
                    _la = self._input.LA(1)

                self.state = 663
                self.match(RulesParser.T__1)
                pass

            elif la_ == 3:
                self.enterOuterAlt(localctx, 3)
                self.state = 665
                self.match(RulesParser.FUNC)
                self.state = 668
                self._errHandler.sync(self)
                la_ = self._interp.adaptivePredict(self._input,68,self._ctx)
                if la_ == 1:
                    self.state = 666
                    self.match(RulesParser.T__0)
                    self.state = 667
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
            self.state = 672
            self.baseNum()
            self.state = 673
            self.match(RulesParser.BINOP)
            self.state = 674
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
            self.state = 678
            self._errHandler.sync(self)
            la_ = self._interp.adaptivePredict(self._input,70,self._ctx)
            if la_ == 1:
                self.enterOuterAlt(localctx, 1)
                self.state = 676
                self.baseNum()
                pass

            elif la_ == 2:
                self.enterOuterAlt(localctx, 2)
                self.state = 677
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
            self.state = 688
            self._errHandler.sync(self)
            la_ = self._interp.adaptivePredict(self._input,71,self._ctx)
            if la_ == 1:
                self.enterOuterAlt(localctx, 1)
                self.state = 680
                self.match(RulesParser.INT)
                pass

            elif la_ == 2:
                self.enterOuterAlt(localctx, 2)
                self.state = 681
                self.match(RulesParser.CONST)
                pass

            elif la_ == 3:
                self.enterOuterAlt(localctx, 3)
                self.state = 682
                self.match(RulesParser.SETTING)
                pass

            elif la_ == 4:
                self.enterOuterAlt(localctx, 4)
                self.state = 683
                self.match(RulesParser.REF)
                pass

            elif la_ == 5:
                self.enterOuterAlt(localctx, 5)
                self.state = 684
                self.value()
                pass

            elif la_ == 6:
                self.enterOuterAlt(localctx, 6)
                self.state = 685
                self.switchNum()
                pass

            elif la_ == 7:
                self.enterOuterAlt(localctx, 7)
                self.state = 686
                self.funcNum()
                pass

            elif la_ == 8:
                self.enterOuterAlt(localctx, 8)
                self.state = 687
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
            self.state = 697
            self._errHandler.sync(self)
            token = self._input.LA(1)
            if token in [32]:
                localctx = RulesParser.SettingContext(self, localctx)
                self.enterOuterAlt(localctx, 1)
                self.state = 690
                self.match(RulesParser.SETTING)
                self.state = 694
                self._errHandler.sync(self)
                la_ = self._interp.adaptivePredict(self._input,72,self._ctx)
                if la_ == 1:
                    self.state = 691
                    self.match(RulesParser.T__10)
                    self.state = 692
                    _la = self._input.LA(1)
                    if not(_la==31 or _la==36):
                        self._errHandler.recoverInline(self)
                    else:
                        self._errHandler.reportMatch(self)
                        self.consume()
                    self.state = 693
                    self.match(RulesParser.T__11)


                pass
            elif token in [33]:
                localctx = RulesParser.ArgumentContext(self, localctx)
                self.enterOuterAlt(localctx, 2)
                self.state = 696
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
            self.state = 699
            self.match(RulesParser.T__10)
            self.state = 702
            self._errHandler.sync(self)
            token = self._input.LA(1)
            if token in [34]:
                self.state = 700
                self.match(RulesParser.FUNC)
                pass
            elif token in [1, 22, 31, 33, 36]:
                self.state = 701
                self.item()
                pass
            else:
                raise NoViableAltException(self)

            self.state = 711
            self._errHandler.sync(self)
            _la = self._input.LA(1)
            while _la==7:
                self.state = 704
                self.match(RulesParser.T__6)
                self.state = 707
                self._errHandler.sync(self)
                token = self._input.LA(1)
                if token in [34]:
                    self.state = 705
                    self.match(RulesParser.FUNC)
                    pass
                elif token in [1, 22, 31, 33, 36]:
                    self.state = 706
                    self.item()
                    pass
                else:
                    raise NoViableAltException(self)

                self.state = 713
                self._errHandler.sync(self)
                _la = self._input.LA(1)

            self.state = 714
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
            self.state = 733
            self._errHandler.sync(self)
            la_ = self._interp.adaptivePredict(self._input,79,self._ctx)
            if la_ == 1:
                localctx = RulesParser.ItemCountContext(self, localctx)
                self.enterOuterAlt(localctx, 1)
                self.state = 725
                self._errHandler.sync(self)
                token = self._input.LA(1)
                if token in [31]:
                    self.state = 716
                    self.match(RulesParser.ITEM)
                    self.state = 717
                    self.match(RulesParser.T__4)
                    self.state = 718
                    _la = self._input.LA(1)
                    if not(_la==32 or _la==38):
                        self._errHandler.recoverInline(self)
                    else:
                        self._errHandler.reportMatch(self)
                        self.consume()
                    self.state = 719
                    self.match(RulesParser.T__5)
                    pass
                elif token in [1]:
                    self.state = 720
                    self.match(RulesParser.T__0)
                    self.state = 721
                    self.match(RulesParser.ITEM)
                    self.state = 722
                    self.match(RulesParser.T__6)
                    self.state = 723
                    _la = self._input.LA(1)
                    if not(_la==32 or _la==38):
                        self._errHandler.recoverInline(self)
                    else:
                        self._errHandler.reportMatch(self)
                        self.consume()
                    self.state = 724
                    self.match(RulesParser.T__1)
                    pass
                else:
                    raise NoViableAltException(self)

                pass

            elif la_ == 2:
                localctx = RulesParser.OneItemContext(self, localctx)
                self.enterOuterAlt(localctx, 2)
                self.state = 728
                self._errHandler.sync(self)
                _la = self._input.LA(1)
                if _la==22:
                    self.state = 727
                    self.match(RulesParser.NOT)


                self.state = 730
                self.match(RulesParser.ITEM)
                pass

            elif la_ == 3:
                localctx = RulesParser.OneLitItemContext(self, localctx)
                self.enterOuterAlt(localctx, 3)
                self.state = 731
                self.match(RulesParser.LIT)
                pass

            elif la_ == 4:
                localctx = RulesParser.OneArgumentContext(self, localctx)
                self.enterOuterAlt(localctx, 4)
                self.state = 732
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
            self.state = 739
            self._errHandler.sync(self)
            token = self._input.LA(1)
            if token in [36]:
                self.enterOuterAlt(localctx, 1)
                self.state = 735
                self.match(RulesParser.LIT)
                pass
            elif token in [32, 33]:
                self.enterOuterAlt(localctx, 2)
                self.state = 736
                self.value()
                pass
            elif token in [25]:
                self.enterOuterAlt(localctx, 3)
                self.state = 737
                self.condStr()
                pass
            elif token in [28]:
                self.enterOuterAlt(localctx, 4)
                self.state = 738
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
            self.state = 760
            self._errHandler.sync(self)
            la_ = self._interp.adaptivePredict(self._input,84,self._ctx)
            if la_ == 1:
                self.enterOuterAlt(localctx, 1)
                self.state = 742
                self._errHandler.sync(self)
                _la = self._input.LA(1)
                if _la==22:
                    self.state = 741
                    self.match(RulesParser.NOT)


                self.state = 744
                self.match(RulesParser.WITHIN)
                self.state = 745
                self.match(RulesParser.PLACE)
                pass

            elif la_ == 2:
                self.enterOuterAlt(localctx, 2)
                self.state = 747
                self._errHandler.sync(self)
                _la = self._input.LA(1)
                if _la==22:
                    self.state = 746
                    self.match(RulesParser.NOT)


                self.state = 749
                self.match(RulesParser.WITHIN)
                self.state = 750
                self.match(RulesParser.T__0)
                self.state = 751
                self.match(RulesParser.PLACE)
                self.state = 756
                self._errHandler.sync(self)
                _la = self._input.LA(1)
                while _la==7:
                    self.state = 752
                    self.match(RulesParser.T__6)
                    self.state = 753
                    self.match(RulesParser.PLACE)
                    self.state = 758
                    self._errHandler.sync(self)
                    _la = self._input.LA(1)

                self.state = 759
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
            self.state = 780
            self._errHandler.sync(self)
            la_ = self._interp.adaptivePredict(self._input,88,self._ctx)
            if la_ == 1:
                localctx = RulesParser.RefInPlaceRefContext(self, localctx)
                self.enterOuterAlt(localctx, 1)
                self.state = 762
                self.match(RulesParser.REF)
                self.state = 764
                self._errHandler.sync(self)
                _la = self._input.LA(1)
                if _la==22:
                    self.state = 763
                    self.match(RulesParser.NOT)


                self.state = 766
                self.match(RulesParser.WITHIN)
                self.state = 767
                self.match(RulesParser.REF)
                pass

            elif la_ == 2:
                localctx = RulesParser.RefInPlaceNameContext(self, localctx)
                self.enterOuterAlt(localctx, 2)
                self.state = 768
                self.match(RulesParser.REF)
                self.state = 770
                self._errHandler.sync(self)
                _la = self._input.LA(1)
                if _la==22:
                    self.state = 769
                    self.match(RulesParser.NOT)


                self.state = 772
                self.match(RulesParser.WITHIN)
                self.state = 773
                self.match(RulesParser.PLACE)
                pass

            elif la_ == 3:
                localctx = RulesParser.RefInFuncContext(self, localctx)
                self.enterOuterAlt(localctx, 3)
                self.state = 774
                self.match(RulesParser.REF)
                self.state = 776
                self._errHandler.sync(self)
                _la = self._input.LA(1)
                if _la==22:
                    self.state = 775
                    self.match(RulesParser.NOT)


                self.state = 778
                self.match(RulesParser.WITHIN)
                self.state = 779
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
         




