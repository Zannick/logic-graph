# Generated from Rules.g4 by ANTLR 4.12.0
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
        4,1,40,734,2,0,7,0,2,1,7,1,2,2,7,2,2,3,7,3,2,4,7,4,2,5,7,5,2,6,7,
        6,2,7,7,7,2,8,7,8,2,9,7,9,2,10,7,10,2,11,7,11,2,12,7,12,2,13,7,13,
        2,14,7,14,2,15,7,15,2,16,7,16,2,17,7,17,2,18,7,18,2,19,7,19,2,20,
        7,20,2,21,7,21,2,22,7,22,2,23,7,23,1,0,1,0,1,0,1,0,1,0,1,0,1,0,1,
        0,1,0,1,0,1,0,1,0,1,0,1,0,1,0,3,0,64,8,0,1,0,1,0,1,0,1,0,1,0,3,0,
        71,8,0,1,0,1,0,1,0,1,0,1,0,1,0,5,0,79,8,0,10,0,12,0,82,9,0,1,1,1,
        1,1,1,5,1,87,8,1,10,1,12,1,90,9,1,1,1,3,1,93,8,1,1,2,1,2,1,2,1,2,
        1,2,1,2,1,2,1,2,3,2,103,8,2,1,2,1,2,1,2,1,2,1,2,1,2,1,2,1,2,1,2,
        1,2,1,2,1,2,1,2,1,2,1,2,1,2,1,2,1,2,1,2,1,2,1,2,5,2,126,8,2,10,2,
        12,2,129,9,2,1,2,1,2,1,2,1,2,1,2,3,2,136,8,2,3,2,138,8,2,1,3,1,3,
        1,3,1,3,1,3,1,3,1,3,1,3,1,3,1,3,1,3,1,3,3,3,152,8,3,1,4,3,4,155,
        8,4,1,4,1,4,1,4,1,4,1,4,5,4,162,8,4,10,4,12,4,165,9,4,1,4,1,4,3,
        4,169,8,4,1,4,1,4,1,4,1,4,1,4,1,4,3,4,177,8,4,1,4,1,4,1,4,1,4,1,
        4,3,4,184,8,4,1,4,1,4,1,4,1,4,1,4,3,4,191,8,4,1,4,1,4,1,4,1,4,1,
        4,3,4,198,8,4,1,4,1,4,1,4,1,4,1,4,3,4,205,8,4,1,4,1,4,1,4,1,4,1,
        4,3,4,212,8,4,1,4,1,4,1,4,3,4,217,8,4,3,4,219,8,4,1,5,1,5,1,5,1,
        5,1,5,1,5,1,5,1,5,1,5,1,5,1,5,1,5,1,5,1,5,1,5,1,5,5,5,237,8,5,10,
        5,12,5,240,9,5,1,5,1,5,1,5,1,5,1,5,3,5,247,8,5,1,5,1,5,1,5,1,5,1,
        5,1,5,1,5,1,5,3,5,257,8,5,1,6,1,6,1,6,1,6,1,6,1,6,1,6,1,6,1,6,1,
        6,1,6,1,6,1,6,1,6,1,6,1,6,5,6,275,8,6,10,6,12,6,278,9,6,1,6,1,6,
        1,6,1,6,1,6,3,6,285,8,6,1,7,1,7,1,7,1,7,1,7,1,7,1,7,1,7,1,7,1,7,
        1,7,1,7,1,7,1,7,1,7,1,7,5,7,303,8,7,10,7,12,7,306,9,7,1,7,1,7,1,
        7,1,7,1,7,3,7,313,8,7,1,8,1,8,1,8,1,8,1,8,1,8,1,8,1,8,4,8,323,8,
        8,11,8,12,8,324,1,8,1,8,1,8,1,8,3,8,331,8,8,1,8,1,8,1,8,1,8,1,8,
        1,8,1,8,1,8,1,8,1,8,4,8,343,8,8,11,8,12,8,344,1,8,1,8,1,8,1,8,1,
        8,4,8,352,8,8,11,8,12,8,353,3,8,356,8,8,1,8,1,8,1,8,1,8,3,8,362,
        8,8,1,8,1,8,1,8,1,8,1,8,1,8,1,8,1,8,5,8,372,8,8,10,8,12,8,375,9,
        8,1,8,1,8,1,8,1,8,4,8,381,8,8,11,8,12,8,382,1,8,1,8,1,8,1,8,3,8,
        389,8,8,1,8,1,8,1,8,1,8,1,8,1,8,1,8,1,8,4,8,399,8,8,11,8,12,8,400,
        1,8,3,8,404,8,8,1,9,1,9,1,9,1,9,1,9,1,9,1,9,1,9,4,9,414,8,9,11,9,
        12,9,415,1,9,1,9,1,9,1,9,3,9,422,8,9,1,9,1,9,1,9,1,9,1,9,1,9,1,9,
        1,9,1,9,1,9,4,9,434,8,9,11,9,12,9,435,1,9,1,9,1,9,1,9,1,9,4,9,443,
        8,9,11,9,12,9,444,3,9,447,8,9,1,9,1,9,1,9,1,9,3,9,453,8,9,1,9,1,
        9,1,9,1,9,1,9,1,9,1,9,1,9,1,9,1,9,4,9,465,8,9,11,9,12,9,466,1,9,
        1,9,1,9,1,9,1,9,4,9,474,8,9,11,9,12,9,475,3,9,478,8,9,1,9,1,9,1,
        9,1,9,3,9,484,8,9,1,9,1,9,3,9,488,8,9,1,10,1,10,1,10,1,10,1,10,1,
        10,1,10,1,10,4,10,498,8,10,11,10,12,10,499,1,10,1,10,1,10,1,10,3,
        10,506,8,10,1,10,1,10,1,10,1,10,1,10,1,10,1,10,1,10,1,10,1,10,4,
        10,518,8,10,11,10,12,10,519,1,10,1,10,1,10,1,10,1,10,4,10,527,8,
        10,11,10,12,10,528,3,10,531,8,10,1,10,1,10,1,10,1,10,3,10,537,8,
        10,1,10,1,10,1,10,1,10,1,10,1,10,1,10,1,10,1,10,1,10,4,10,549,8,
        10,11,10,12,10,550,1,10,1,10,1,10,1,10,1,10,4,10,558,8,10,11,10,
        12,10,559,3,10,562,8,10,1,10,1,10,1,10,1,10,3,10,568,8,10,1,10,1,
        10,3,10,572,8,10,1,11,1,11,1,11,1,11,1,11,1,11,1,11,1,11,1,11,1,
        11,1,11,1,11,1,11,1,11,1,11,1,11,1,11,1,11,1,11,1,11,1,11,1,11,1,
        11,1,11,3,11,598,8,11,1,12,1,12,1,12,1,12,1,12,1,12,1,12,1,12,3,
        12,608,8,12,1,13,1,13,1,13,1,13,1,14,1,14,1,14,1,14,1,15,1,15,1,
        15,1,15,1,15,1,15,1,15,1,15,1,15,5,15,627,8,15,10,15,12,15,630,9,
        15,1,15,1,15,1,15,1,15,1,15,3,15,637,8,15,3,15,639,8,15,1,16,1,16,
        1,16,1,16,1,17,1,17,3,17,647,8,17,1,18,1,18,1,18,1,18,1,18,1,18,
        1,18,1,18,3,18,657,8,18,1,19,1,19,1,19,1,19,3,19,663,8,19,1,19,3,
        19,666,8,19,1,20,1,20,1,20,1,20,1,20,1,20,1,20,1,20,1,20,3,20,677,
        8,20,1,20,3,20,680,8,20,1,20,1,20,1,20,3,20,685,8,20,1,21,1,21,1,
        21,1,21,3,21,691,8,21,1,22,3,22,694,8,22,1,22,1,22,1,22,3,22,699,
        8,22,1,22,1,22,1,22,1,22,1,22,5,22,706,8,22,10,22,12,22,709,9,22,
        1,22,3,22,712,8,22,1,23,1,23,3,23,716,8,23,1,23,1,23,1,23,1,23,3,
        23,722,8,23,1,23,1,23,1,23,1,23,3,23,728,8,23,1,23,1,23,3,23,732,
        8,23,1,23,0,1,0,24,0,2,4,6,8,10,12,14,16,18,20,22,24,26,28,30,32,
        34,36,38,40,42,44,46,0,3,1,0,30,31,2,0,30,30,35,35,2,0,31,31,37,
        37,837,0,70,1,0,0,0,2,83,1,0,0,0,4,137,1,0,0,0,6,151,1,0,0,0,8,218,
        1,0,0,0,10,256,1,0,0,0,12,258,1,0,0,0,14,286,1,0,0,0,16,403,1,0,
        0,0,18,487,1,0,0,0,20,571,1,0,0,0,22,597,1,0,0,0,24,607,1,0,0,0,
        26,609,1,0,0,0,28,613,1,0,0,0,30,638,1,0,0,0,32,640,1,0,0,0,34,646,
        1,0,0,0,36,656,1,0,0,0,38,665,1,0,0,0,40,684,1,0,0,0,42,690,1,0,
        0,0,44,711,1,0,0,0,46,731,1,0,0,0,48,49,6,0,-1,0,49,50,5,1,0,0,50,
        51,3,0,0,0,51,52,5,2,0,0,52,71,1,0,0,0,53,71,3,8,4,0,54,71,3,6,3,
        0,55,71,3,16,8,0,56,71,3,10,5,0,57,71,3,22,11,0,58,71,3,24,12,0,
        59,71,3,26,13,0,60,71,3,28,14,0,61,71,3,40,20,0,62,64,5,22,0,0,63,
        62,1,0,0,0,63,64,1,0,0,0,64,65,1,0,0,0,65,71,3,38,19,0,66,71,3,44,
        22,0,67,71,3,46,23,0,68,71,5,23,0,0,69,71,5,24,0,0,70,48,1,0,0,0,
        70,53,1,0,0,0,70,54,1,0,0,0,70,55,1,0,0,0,70,56,1,0,0,0,70,57,1,
        0,0,0,70,58,1,0,0,0,70,59,1,0,0,0,70,60,1,0,0,0,70,61,1,0,0,0,70,
        63,1,0,0,0,70,66,1,0,0,0,70,67,1,0,0,0,70,68,1,0,0,0,70,69,1,0,0,
        0,71,80,1,0,0,0,72,73,10,16,0,0,73,74,5,20,0,0,74,79,3,0,0,17,75,
        76,10,15,0,0,76,77,5,21,0,0,77,79,3,0,0,16,78,72,1,0,0,0,78,75,1,
        0,0,0,79,82,1,0,0,0,80,78,1,0,0,0,80,81,1,0,0,0,81,1,1,0,0,0,82,
        80,1,0,0,0,83,88,3,4,2,0,84,85,5,3,0,0,85,87,3,4,2,0,86,84,1,0,0,
        0,87,90,1,0,0,0,88,86,1,0,0,0,88,89,1,0,0,0,89,92,1,0,0,0,90,88,
        1,0,0,0,91,93,5,3,0,0,92,91,1,0,0,0,92,93,1,0,0,0,93,3,1,0,0,0,94,
        95,5,32,0,0,95,102,5,4,0,0,96,103,5,23,0,0,97,103,5,24,0,0,98,103,
        5,34,0,0,99,103,5,32,0,0,100,103,3,42,21,0,101,103,3,34,17,0,102,
        96,1,0,0,0,102,97,1,0,0,0,102,98,1,0,0,0,102,99,1,0,0,0,102,100,
        1,0,0,0,102,101,1,0,0,0,103,138,1,0,0,0,104,105,5,32,0,0,105,106,
        5,39,0,0,106,107,5,4,0,0,107,138,3,34,17,0,108,138,3,8,4,0,109,110,
        5,25,0,0,110,111,5,1,0,0,111,112,3,0,0,0,112,113,5,2,0,0,113,114,
        5,5,0,0,114,115,3,2,1,0,115,127,5,6,0,0,116,117,5,26,0,0,117,118,
        5,25,0,0,118,119,5,1,0,0,119,120,3,0,0,0,120,121,5,2,0,0,121,122,
        5,5,0,0,122,123,3,2,1,0,123,124,5,6,0,0,124,126,1,0,0,0,125,116,
        1,0,0,0,126,129,1,0,0,0,127,125,1,0,0,0,127,128,1,0,0,0,128,135,
        1,0,0,0,129,127,1,0,0,0,130,131,5,26,0,0,131,132,5,5,0,0,132,133,
        3,2,1,0,133,134,5,6,0,0,134,136,1,0,0,0,135,130,1,0,0,0,135,136,
        1,0,0,0,136,138,1,0,0,0,137,94,1,0,0,0,137,104,1,0,0,0,137,108,1,
        0,0,0,137,109,1,0,0,0,138,5,1,0,0,0,139,140,5,33,0,0,140,141,5,1,
        0,0,141,142,5,35,0,0,142,143,5,7,0,0,143,144,3,0,0,0,144,145,5,2,
        0,0,145,152,1,0,0,0,146,147,5,33,0,0,147,148,5,1,0,0,148,149,3,0,
        0,0,149,150,5,2,0,0,150,152,1,0,0,0,151,139,1,0,0,0,151,146,1,0,
        0,0,152,7,1,0,0,0,153,155,5,22,0,0,154,153,1,0,0,0,154,155,1,0,0,
        0,155,156,1,0,0,0,156,157,5,33,0,0,157,158,5,1,0,0,158,163,5,30,
        0,0,159,160,5,7,0,0,160,162,5,30,0,0,161,159,1,0,0,0,162,165,1,0,
        0,0,163,161,1,0,0,0,163,164,1,0,0,0,164,166,1,0,0,0,165,163,1,0,
        0,0,166,219,5,2,0,0,167,169,5,22,0,0,168,167,1,0,0,0,168,169,1,0,
        0,0,169,170,1,0,0,0,170,171,5,33,0,0,171,172,5,1,0,0,172,173,3,38,
        19,0,173,174,5,2,0,0,174,219,1,0,0,0,175,177,5,22,0,0,176,175,1,
        0,0,0,176,177,1,0,0,0,177,178,1,0,0,0,178,179,5,33,0,0,179,180,5,
        1,0,0,180,181,5,35,0,0,181,219,5,2,0,0,182,184,5,22,0,0,183,182,
        1,0,0,0,183,184,1,0,0,0,184,185,1,0,0,0,185,186,5,33,0,0,186,187,
        5,1,0,0,187,188,5,37,0,0,188,219,5,2,0,0,189,191,5,22,0,0,190,189,
        1,0,0,0,190,191,1,0,0,0,191,192,1,0,0,0,192,193,5,33,0,0,193,194,
        5,1,0,0,194,195,5,38,0,0,195,219,5,2,0,0,196,198,5,22,0,0,197,196,
        1,0,0,0,197,198,1,0,0,0,198,199,1,0,0,0,199,200,5,33,0,0,200,201,
        5,1,0,0,201,202,5,34,0,0,202,219,5,2,0,0,203,205,5,22,0,0,204,203,
        1,0,0,0,204,205,1,0,0,0,205,206,1,0,0,0,206,207,5,33,0,0,207,208,
        5,1,0,0,208,209,5,32,0,0,209,219,5,2,0,0,210,212,5,22,0,0,211,210,
        1,0,0,0,211,212,1,0,0,0,212,213,1,0,0,0,213,216,5,33,0,0,214,215,
        5,1,0,0,215,217,5,2,0,0,216,214,1,0,0,0,216,217,1,0,0,0,217,219,
        1,0,0,0,218,154,1,0,0,0,218,168,1,0,0,0,218,176,1,0,0,0,218,183,
        1,0,0,0,218,190,1,0,0,0,218,197,1,0,0,0,218,204,1,0,0,0,218,211,
        1,0,0,0,219,9,1,0,0,0,220,221,5,25,0,0,221,222,5,1,0,0,222,223,3,
        0,0,0,223,224,5,2,0,0,224,225,5,5,0,0,225,226,3,0,0,0,226,238,5,
        6,0,0,227,228,5,26,0,0,228,229,5,25,0,0,229,230,5,1,0,0,230,231,
        3,0,0,0,231,232,5,2,0,0,232,233,5,5,0,0,233,234,3,0,0,0,234,235,
        5,6,0,0,235,237,1,0,0,0,236,227,1,0,0,0,237,240,1,0,0,0,238,236,
        1,0,0,0,238,239,1,0,0,0,239,246,1,0,0,0,240,238,1,0,0,0,241,242,
        5,26,0,0,242,243,5,5,0,0,243,244,3,0,0,0,244,245,5,6,0,0,245,247,
        1,0,0,0,246,241,1,0,0,0,246,247,1,0,0,0,247,257,1,0,0,0,248,249,
        5,1,0,0,249,250,3,0,0,0,250,251,5,25,0,0,251,252,3,0,0,0,252,253,
        5,26,0,0,253,254,3,0,0,0,254,255,5,2,0,0,255,257,1,0,0,0,256,220,
        1,0,0,0,256,248,1,0,0,0,257,11,1,0,0,0,258,259,5,25,0,0,259,260,
        5,1,0,0,260,261,3,0,0,0,261,262,5,2,0,0,262,263,5,5,0,0,263,264,
        3,34,17,0,264,276,5,6,0,0,265,266,5,26,0,0,266,267,5,25,0,0,267,
        268,5,1,0,0,268,269,3,0,0,0,269,270,5,2,0,0,270,271,5,5,0,0,271,
        272,3,34,17,0,272,273,5,6,0,0,273,275,1,0,0,0,274,265,1,0,0,0,275,
        278,1,0,0,0,276,274,1,0,0,0,276,277,1,0,0,0,277,284,1,0,0,0,278,
        276,1,0,0,0,279,280,5,26,0,0,280,281,5,5,0,0,281,282,3,34,17,0,282,
        283,5,6,0,0,283,285,1,0,0,0,284,279,1,0,0,0,284,285,1,0,0,0,285,
        13,1,0,0,0,286,287,5,25,0,0,287,288,5,1,0,0,288,289,3,0,0,0,289,
        290,5,2,0,0,290,291,5,5,0,0,291,292,3,42,21,0,292,304,5,6,0,0,293,
        294,5,26,0,0,294,295,5,25,0,0,295,296,5,1,0,0,296,297,3,0,0,0,297,
        298,5,2,0,0,298,299,5,5,0,0,299,300,3,42,21,0,300,301,5,6,0,0,301,
        303,1,0,0,0,302,293,1,0,0,0,303,306,1,0,0,0,304,302,1,0,0,0,304,
        305,1,0,0,0,305,312,1,0,0,0,306,304,1,0,0,0,307,308,5,26,0,0,308,
        309,5,5,0,0,309,310,3,42,21,0,310,311,5,6,0,0,311,313,1,0,0,0,312,
        307,1,0,0,0,312,313,1,0,0,0,313,15,1,0,0,0,314,315,5,28,0,0,315,
        316,5,30,0,0,316,322,5,5,0,0,317,318,5,37,0,0,318,319,5,8,0,0,319,
        320,3,0,0,0,320,321,5,7,0,0,321,323,1,0,0,0,322,317,1,0,0,0,323,
        324,1,0,0,0,324,322,1,0,0,0,324,325,1,0,0,0,325,326,1,0,0,0,326,
        327,5,9,0,0,327,328,5,8,0,0,328,330,3,0,0,0,329,331,5,7,0,0,330,
        329,1,0,0,0,330,331,1,0,0,0,331,332,1,0,0,0,332,333,5,6,0,0,333,
        404,1,0,0,0,334,335,5,28,0,0,335,336,5,31,0,0,336,355,5,5,0,0,337,
        338,5,37,0,0,338,339,5,8,0,0,339,340,3,0,0,0,340,341,5,7,0,0,341,
        343,1,0,0,0,342,337,1,0,0,0,343,344,1,0,0,0,344,342,1,0,0,0,344,
        345,1,0,0,0,345,356,1,0,0,0,346,347,5,35,0,0,347,348,5,8,0,0,348,
        349,3,0,0,0,349,350,5,7,0,0,350,352,1,0,0,0,351,346,1,0,0,0,352,
        353,1,0,0,0,353,351,1,0,0,0,353,354,1,0,0,0,354,356,1,0,0,0,355,
        342,1,0,0,0,355,351,1,0,0,0,356,357,1,0,0,0,357,358,5,9,0,0,358,
        359,5,8,0,0,359,361,3,0,0,0,360,362,5,7,0,0,361,360,1,0,0,0,361,
        362,1,0,0,0,362,363,1,0,0,0,363,364,5,6,0,0,364,404,1,0,0,0,365,
        366,5,28,0,0,366,367,5,32,0,0,367,380,5,5,0,0,368,373,5,30,0,0,369,
        370,5,10,0,0,370,372,5,30,0,0,371,369,1,0,0,0,372,375,1,0,0,0,373,
        371,1,0,0,0,373,374,1,0,0,0,374,376,1,0,0,0,375,373,1,0,0,0,376,
        377,5,8,0,0,377,378,3,0,0,0,378,379,5,7,0,0,379,381,1,0,0,0,380,
        368,1,0,0,0,381,382,1,0,0,0,382,380,1,0,0,0,382,383,1,0,0,0,383,
        384,1,0,0,0,384,385,5,9,0,0,385,386,5,8,0,0,386,388,3,0,0,0,387,
        389,5,7,0,0,388,387,1,0,0,0,388,389,1,0,0,0,389,390,1,0,0,0,390,
        391,5,6,0,0,391,404,1,0,0,0,392,393,5,32,0,0,393,394,5,27,0,0,394,
        395,5,11,0,0,395,398,5,30,0,0,396,397,5,7,0,0,397,399,5,30,0,0,398,
        396,1,0,0,0,399,400,1,0,0,0,400,398,1,0,0,0,400,401,1,0,0,0,401,
        402,1,0,0,0,402,404,5,12,0,0,403,314,1,0,0,0,403,334,1,0,0,0,403,
        365,1,0,0,0,403,392,1,0,0,0,404,17,1,0,0,0,405,406,5,28,0,0,406,
        407,5,30,0,0,407,413,5,5,0,0,408,409,5,37,0,0,409,410,5,8,0,0,410,
        411,3,34,17,0,411,412,5,7,0,0,412,414,1,0,0,0,413,408,1,0,0,0,414,
        415,1,0,0,0,415,413,1,0,0,0,415,416,1,0,0,0,416,417,1,0,0,0,417,
        418,5,9,0,0,418,419,5,8,0,0,419,421,3,34,17,0,420,422,5,7,0,0,421,
        420,1,0,0,0,421,422,1,0,0,0,422,423,1,0,0,0,423,424,5,6,0,0,424,
        488,1,0,0,0,425,426,5,28,0,0,426,427,5,32,0,0,427,446,5,5,0,0,428,
        429,5,37,0,0,429,430,5,8,0,0,430,431,3,34,17,0,431,432,5,7,0,0,432,
        434,1,0,0,0,433,428,1,0,0,0,434,435,1,0,0,0,435,433,1,0,0,0,435,
        436,1,0,0,0,436,447,1,0,0,0,437,438,5,35,0,0,438,439,5,8,0,0,439,
        440,3,34,17,0,440,441,5,7,0,0,441,443,1,0,0,0,442,437,1,0,0,0,443,
        444,1,0,0,0,444,442,1,0,0,0,444,445,1,0,0,0,445,447,1,0,0,0,446,
        433,1,0,0,0,446,442,1,0,0,0,447,448,1,0,0,0,448,449,5,9,0,0,449,
        450,5,8,0,0,450,452,3,34,17,0,451,453,5,7,0,0,452,451,1,0,0,0,452,
        453,1,0,0,0,453,454,1,0,0,0,454,455,5,6,0,0,455,488,1,0,0,0,456,
        457,5,28,0,0,457,458,5,31,0,0,458,477,5,5,0,0,459,460,5,37,0,0,460,
        461,5,8,0,0,461,462,3,34,17,0,462,463,5,7,0,0,463,465,1,0,0,0,464,
        459,1,0,0,0,465,466,1,0,0,0,466,464,1,0,0,0,466,467,1,0,0,0,467,
        478,1,0,0,0,468,469,5,35,0,0,469,470,5,8,0,0,470,471,3,34,17,0,471,
        472,5,7,0,0,472,474,1,0,0,0,473,468,1,0,0,0,474,475,1,0,0,0,475,
        473,1,0,0,0,475,476,1,0,0,0,476,478,1,0,0,0,477,464,1,0,0,0,477,
        473,1,0,0,0,478,479,1,0,0,0,479,480,5,9,0,0,480,481,5,8,0,0,481,
        483,3,34,17,0,482,484,5,7,0,0,483,482,1,0,0,0,483,484,1,0,0,0,484,
        485,1,0,0,0,485,486,5,6,0,0,486,488,1,0,0,0,487,405,1,0,0,0,487,
        425,1,0,0,0,487,456,1,0,0,0,488,19,1,0,0,0,489,490,5,28,0,0,490,
        491,5,30,0,0,491,497,5,5,0,0,492,493,5,37,0,0,493,494,5,8,0,0,494,
        495,3,42,21,0,495,496,5,7,0,0,496,498,1,0,0,0,497,492,1,0,0,0,498,
        499,1,0,0,0,499,497,1,0,0,0,499,500,1,0,0,0,500,501,1,0,0,0,501,
        502,5,9,0,0,502,503,5,8,0,0,503,505,3,42,21,0,504,506,5,7,0,0,505,
        504,1,0,0,0,505,506,1,0,0,0,506,507,1,0,0,0,507,508,5,6,0,0,508,
        572,1,0,0,0,509,510,5,28,0,0,510,511,5,32,0,0,511,530,5,5,0,0,512,
        513,5,37,0,0,513,514,5,8,0,0,514,515,3,42,21,0,515,516,5,7,0,0,516,
        518,1,0,0,0,517,512,1,0,0,0,518,519,1,0,0,0,519,517,1,0,0,0,519,
        520,1,0,0,0,520,531,1,0,0,0,521,522,5,35,0,0,522,523,5,8,0,0,523,
        524,3,42,21,0,524,525,5,7,0,0,525,527,1,0,0,0,526,521,1,0,0,0,527,
        528,1,0,0,0,528,526,1,0,0,0,528,529,1,0,0,0,529,531,1,0,0,0,530,
        517,1,0,0,0,530,526,1,0,0,0,531,532,1,0,0,0,532,533,5,9,0,0,533,
        534,5,8,0,0,534,536,3,42,21,0,535,537,5,7,0,0,536,535,1,0,0,0,536,
        537,1,0,0,0,537,538,1,0,0,0,538,539,5,6,0,0,539,572,1,0,0,0,540,
        541,5,28,0,0,541,542,5,31,0,0,542,561,5,5,0,0,543,544,5,37,0,0,544,
        545,5,8,0,0,545,546,3,42,21,0,546,547,5,7,0,0,547,549,1,0,0,0,548,
        543,1,0,0,0,549,550,1,0,0,0,550,548,1,0,0,0,550,551,1,0,0,0,551,
        562,1,0,0,0,552,553,5,35,0,0,553,554,5,8,0,0,554,555,3,42,21,0,555,
        556,5,7,0,0,556,558,1,0,0,0,557,552,1,0,0,0,558,559,1,0,0,0,559,
        557,1,0,0,0,559,560,1,0,0,0,560,562,1,0,0,0,561,548,1,0,0,0,561,
        557,1,0,0,0,562,563,1,0,0,0,563,564,5,9,0,0,564,565,5,8,0,0,565,
        567,3,42,21,0,566,568,5,7,0,0,567,566,1,0,0,0,567,568,1,0,0,0,568,
        569,1,0,0,0,569,570,5,6,0,0,570,572,1,0,0,0,571,489,1,0,0,0,571,
        509,1,0,0,0,571,540,1,0,0,0,572,21,1,0,0,0,573,574,3,38,19,0,574,
        575,5,13,0,0,575,576,3,34,17,0,576,598,1,0,0,0,577,578,3,38,19,0,
        578,579,5,14,0,0,579,580,3,34,17,0,580,598,1,0,0,0,581,582,3,38,
        19,0,582,583,5,15,0,0,583,584,3,34,17,0,584,598,1,0,0,0,585,586,
        3,38,19,0,586,587,5,16,0,0,587,588,3,34,17,0,588,598,1,0,0,0,589,
        590,3,38,19,0,590,591,5,17,0,0,591,592,3,34,17,0,592,598,1,0,0,0,
        593,594,3,38,19,0,594,595,5,18,0,0,595,596,3,34,17,0,596,598,1,0,
        0,0,597,573,1,0,0,0,597,577,1,0,0,0,597,581,1,0,0,0,597,585,1,0,
        0,0,597,589,1,0,0,0,597,593,1,0,0,0,598,23,1,0,0,0,599,600,3,38,
        19,0,600,601,5,13,0,0,601,602,5,35,0,0,602,608,1,0,0,0,603,604,3,
        38,19,0,604,605,5,14,0,0,605,606,5,35,0,0,606,608,1,0,0,0,607,599,
        1,0,0,0,607,603,1,0,0,0,608,25,1,0,0,0,609,610,3,38,19,0,610,611,
        5,19,0,0,611,612,3,34,17,0,612,27,1,0,0,0,613,614,5,32,0,0,614,615,
        5,13,0,0,615,616,7,0,0,0,616,29,1,0,0,0,617,618,5,33,0,0,618,619,
        5,1,0,0,619,620,5,30,0,0,620,639,5,2,0,0,621,622,5,33,0,0,622,623,
        5,1,0,0,623,628,3,34,17,0,624,625,5,7,0,0,625,627,3,34,17,0,626,
        624,1,0,0,0,627,630,1,0,0,0,628,626,1,0,0,0,628,629,1,0,0,0,629,
        631,1,0,0,0,630,628,1,0,0,0,631,632,5,2,0,0,632,639,1,0,0,0,633,
        636,5,33,0,0,634,635,5,1,0,0,635,637,5,2,0,0,636,634,1,0,0,0,636,
        637,1,0,0,0,637,639,1,0,0,0,638,617,1,0,0,0,638,621,1,0,0,0,638,
        633,1,0,0,0,639,31,1,0,0,0,640,641,3,36,18,0,641,642,5,39,0,0,642,
        643,3,34,17,0,643,33,1,0,0,0,644,647,3,36,18,0,645,647,3,32,16,0,
        646,644,1,0,0,0,646,645,1,0,0,0,647,35,1,0,0,0,648,657,5,37,0,0,
        649,657,5,36,0,0,650,657,5,31,0,0,651,657,5,32,0,0,652,657,3,38,
        19,0,653,657,3,18,9,0,654,657,3,30,15,0,655,657,3,12,6,0,656,648,
        1,0,0,0,656,649,1,0,0,0,656,650,1,0,0,0,656,651,1,0,0,0,656,652,
        1,0,0,0,656,653,1,0,0,0,656,654,1,0,0,0,656,655,1,0,0,0,657,37,1,
        0,0,0,658,662,5,31,0,0,659,660,5,11,0,0,660,661,7,1,0,0,661,663,
        5,12,0,0,662,659,1,0,0,0,662,663,1,0,0,0,663,666,1,0,0,0,664,666,
        5,32,0,0,665,658,1,0,0,0,665,664,1,0,0,0,666,39,1,0,0,0,667,668,
        5,30,0,0,668,669,5,5,0,0,669,670,7,2,0,0,670,677,5,6,0,0,671,672,
        5,1,0,0,672,673,5,30,0,0,673,674,5,7,0,0,674,675,7,2,0,0,675,677,
        5,2,0,0,676,667,1,0,0,0,676,671,1,0,0,0,677,685,1,0,0,0,678,680,
        5,22,0,0,679,678,1,0,0,0,679,680,1,0,0,0,680,681,1,0,0,0,681,685,
        5,30,0,0,682,685,5,35,0,0,683,685,5,32,0,0,684,676,1,0,0,0,684,679,
        1,0,0,0,684,682,1,0,0,0,684,683,1,0,0,0,685,41,1,0,0,0,686,691,5,
        35,0,0,687,691,3,38,19,0,688,691,3,14,7,0,689,691,3,20,10,0,690,
        686,1,0,0,0,690,687,1,0,0,0,690,688,1,0,0,0,690,689,1,0,0,0,691,
        43,1,0,0,0,692,694,5,22,0,0,693,692,1,0,0,0,693,694,1,0,0,0,694,
        695,1,0,0,0,695,696,5,29,0,0,696,712,5,34,0,0,697,699,5,22,0,0,698,
        697,1,0,0,0,698,699,1,0,0,0,699,700,1,0,0,0,700,701,5,29,0,0,701,
        702,5,1,0,0,702,707,5,34,0,0,703,704,5,7,0,0,704,706,5,34,0,0,705,
        703,1,0,0,0,706,709,1,0,0,0,707,705,1,0,0,0,707,708,1,0,0,0,708,
        710,1,0,0,0,709,707,1,0,0,0,710,712,5,2,0,0,711,693,1,0,0,0,711,
        698,1,0,0,0,712,45,1,0,0,0,713,715,5,32,0,0,714,716,5,22,0,0,715,
        714,1,0,0,0,715,716,1,0,0,0,716,717,1,0,0,0,717,718,5,29,0,0,718,
        732,5,32,0,0,719,721,5,32,0,0,720,722,5,22,0,0,721,720,1,0,0,0,721,
        722,1,0,0,0,722,723,1,0,0,0,723,724,5,29,0,0,724,732,5,34,0,0,725,
        727,5,32,0,0,726,728,5,22,0,0,727,726,1,0,0,0,727,728,1,0,0,0,728,
        729,1,0,0,0,729,730,5,29,0,0,730,732,3,8,4,0,731,713,1,0,0,0,731,
        719,1,0,0,0,731,725,1,0,0,0,732,47,1,0,0,0,83,63,70,78,80,88,92,
        102,127,135,137,151,154,163,168,176,183,190,197,204,211,216,218,
        238,246,256,276,284,304,312,324,330,344,353,355,361,373,382,388,
        400,403,415,421,435,444,446,452,466,475,477,483,487,499,505,519,
        528,530,536,550,559,561,567,571,597,607,628,636,638,646,656,662,
        665,676,679,684,690,693,698,707,711,715,721,727,731
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
    RULE_item = 20
    RULE_str = 21
    RULE_somewhere = 22
    RULE_refSomewhere = 23

    ruleNames =  [ "boolExpr", "actions", "action", "meta", "invoke", "cond", 
                   "condNum", "condStr", "switchBool", "switchNum", "switchStr", 
                   "cmp", "cmpStr", "flagMatch", "refEq", "funcNum", "mathNum", 
                   "num", "baseNum", "value", "item", "str", "somewhere", 
                   "refSomewhere" ]

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
        self.checkVersion("4.12.0")
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
            self.state = 70
            self._errHandler.sync(self)
            la_ = self._interp.adaptivePredict(self._input,1,self._ctx)
            if la_ == 1:
                self.state = 49
                self.match(RulesParser.T__0)
                self.state = 50
                self.boolExpr(0)
                self.state = 51
                self.match(RulesParser.T__1)
                pass

            elif la_ == 2:
                self.state = 53
                self.invoke()
                pass

            elif la_ == 3:
                self.state = 54
                self.meta()
                pass

            elif la_ == 4:
                self.state = 55
                self.switchBool()
                pass

            elif la_ == 5:
                self.state = 56
                self.cond()
                pass

            elif la_ == 6:
                self.state = 57
                self.cmp()
                pass

            elif la_ == 7:
                self.state = 58
                self.cmpStr()
                pass

            elif la_ == 8:
                self.state = 59
                self.flagMatch()
                pass

            elif la_ == 9:
                self.state = 60
                self.refEq()
                pass

            elif la_ == 10:
                self.state = 61
                self.item()
                pass

            elif la_ == 11:
                self.state = 63
                self._errHandler.sync(self)
                _la = self._input.LA(1)
                if _la==22:
                    self.state = 62
                    self.match(RulesParser.NOT)


                self.state = 65
                self.value()
                pass

            elif la_ == 12:
                self.state = 66
                self.somewhere()
                pass

            elif la_ == 13:
                self.state = 67
                self.refSomewhere()
                pass

            elif la_ == 14:
                self.state = 68
                self.match(RulesParser.TRUE)
                pass

            elif la_ == 15:
                self.state = 69
                self.match(RulesParser.FALSE)
                pass


            self._ctx.stop = self._input.LT(-1)
            self.state = 80
            self._errHandler.sync(self)
            _alt = self._interp.adaptivePredict(self._input,3,self._ctx)
            while _alt!=2 and _alt!=ATN.INVALID_ALT_NUMBER:
                if _alt==1:
                    if self._parseListeners is not None:
                        self.triggerExitRuleEvent()
                    _prevctx = localctx
                    self.state = 78
                    self._errHandler.sync(self)
                    la_ = self._interp.adaptivePredict(self._input,2,self._ctx)
                    if la_ == 1:
                        localctx = RulesParser.BoolExprContext(self, _parentctx, _parentState)
                        self.pushNewRecursionContext(localctx, _startState, self.RULE_boolExpr)
                        self.state = 72
                        if not self.precpred(self._ctx, 16):
                            from antlr4.error.Errors import FailedPredicateException
                            raise FailedPredicateException(self, "self.precpred(self._ctx, 16)")
                        self.state = 73
                        self.match(RulesParser.AND)
                        self.state = 74
                        self.boolExpr(17)
                        pass

                    elif la_ == 2:
                        localctx = RulesParser.BoolExprContext(self, _parentctx, _parentState)
                        self.pushNewRecursionContext(localctx, _startState, self.RULE_boolExpr)
                        self.state = 75
                        if not self.precpred(self._ctx, 15):
                            from antlr4.error.Errors import FailedPredicateException
                            raise FailedPredicateException(self, "self.precpred(self._ctx, 15)")
                        self.state = 76
                        self.match(RulesParser.OR)
                        self.state = 77
                        self.boolExpr(16)
                        pass

             
                self.state = 82
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
            self.state = 83
            self.action()
            self.state = 88
            self._errHandler.sync(self)
            _alt = self._interp.adaptivePredict(self._input,4,self._ctx)
            while _alt!=2 and _alt!=ATN.INVALID_ALT_NUMBER:
                if _alt==1:
                    self.state = 84
                    self.match(RulesParser.T__2)
                    self.state = 85
                    self.action() 
                self.state = 90
                self._errHandler.sync(self)
                _alt = self._interp.adaptivePredict(self._input,4,self._ctx)

            self.state = 92
            self._errHandler.sync(self)
            _la = self._input.LA(1)
            if _la==3:
                self.state = 91
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
            self.state = 137
            self._errHandler.sync(self)
            la_ = self._interp.adaptivePredict(self._input,9,self._ctx)
            if la_ == 1:
                localctx = RulesParser.SetContext(self, localctx)
                self.enterOuterAlt(localctx, 1)
                self.state = 94
                self.match(RulesParser.REF)
                self.state = 95
                self.match(RulesParser.T__3)
                self.state = 102
                self._errHandler.sync(self)
                la_ = self._interp.adaptivePredict(self._input,6,self._ctx)
                if la_ == 1:
                    self.state = 96
                    self.match(RulesParser.TRUE)
                    pass

                elif la_ == 2:
                    self.state = 97
                    self.match(RulesParser.FALSE)
                    pass

                elif la_ == 3:
                    self.state = 98
                    self.match(RulesParser.PLACE)
                    pass

                elif la_ == 4:
                    self.state = 99
                    self.match(RulesParser.REF)
                    pass

                elif la_ == 5:
                    self.state = 100
                    self.str_()
                    pass

                elif la_ == 6:
                    self.state = 101
                    self.num()
                    pass


                pass

            elif la_ == 2:
                localctx = RulesParser.AlterContext(self, localctx)
                self.enterOuterAlt(localctx, 2)
                self.state = 104
                self.match(RulesParser.REF)
                self.state = 105
                self.match(RulesParser.BINOP)
                self.state = 106
                self.match(RulesParser.T__3)
                self.state = 107
                self.num()
                pass

            elif la_ == 3:
                localctx = RulesParser.ActionHelperContext(self, localctx)
                self.enterOuterAlt(localctx, 3)
                self.state = 108
                self.invoke()
                pass

            elif la_ == 4:
                localctx = RulesParser.CondActionContext(self, localctx)
                self.enterOuterAlt(localctx, 4)
                self.state = 109
                self.match(RulesParser.IF)
                self.state = 110
                self.match(RulesParser.T__0)
                self.state = 111
                self.boolExpr(0)
                self.state = 112
                self.match(RulesParser.T__1)
                self.state = 113
                self.match(RulesParser.T__4)
                self.state = 114
                self.actions()
                self.state = 115
                self.match(RulesParser.T__5)
                self.state = 127
                self._errHandler.sync(self)
                _alt = self._interp.adaptivePredict(self._input,7,self._ctx)
                while _alt!=2 and _alt!=ATN.INVALID_ALT_NUMBER:
                    if _alt==1:
                        self.state = 116
                        self.match(RulesParser.ELSE)
                        self.state = 117
                        self.match(RulesParser.IF)
                        self.state = 118
                        self.match(RulesParser.T__0)
                        self.state = 119
                        self.boolExpr(0)
                        self.state = 120
                        self.match(RulesParser.T__1)
                        self.state = 121
                        self.match(RulesParser.T__4)
                        self.state = 122
                        self.actions()
                        self.state = 123
                        self.match(RulesParser.T__5) 
                    self.state = 129
                    self._errHandler.sync(self)
                    _alt = self._interp.adaptivePredict(self._input,7,self._ctx)

                self.state = 135
                self._errHandler.sync(self)
                _la = self._input.LA(1)
                if _la==26:
                    self.state = 130
                    self.match(RulesParser.ELSE)
                    self.state = 131
                    self.match(RulesParser.T__4)
                    self.state = 132
                    self.actions()
                    self.state = 133
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
            self.state = 151
            self._errHandler.sync(self)
            la_ = self._interp.adaptivePredict(self._input,10,self._ctx)
            if la_ == 1:
                self.enterOuterAlt(localctx, 1)
                self.state = 139
                self.match(RulesParser.FUNC)
                self.state = 140
                self.match(RulesParser.T__0)
                self.state = 141
                self.match(RulesParser.LIT)
                self.state = 142
                self.match(RulesParser.T__6)
                self.state = 143
                self.boolExpr(0)
                self.state = 144
                self.match(RulesParser.T__1)
                pass

            elif la_ == 2:
                self.enterOuterAlt(localctx, 2)
                self.state = 146
                self.match(RulesParser.FUNC)
                self.state = 147
                self.match(RulesParser.T__0)
                self.state = 148
                self.boolExpr(0)
                self.state = 149
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

        def PLACE(self):
            return self.getToken(RulesParser.PLACE, 0)

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
            self.state = 218
            self._errHandler.sync(self)
            la_ = self._interp.adaptivePredict(self._input,21,self._ctx)
            if la_ == 1:
                self.enterOuterAlt(localctx, 1)
                self.state = 154
                self._errHandler.sync(self)
                _la = self._input.LA(1)
                if _la==22:
                    self.state = 153
                    self.match(RulesParser.NOT)


                self.state = 156
                self.match(RulesParser.FUNC)
                self.state = 157
                self.match(RulesParser.T__0)
                self.state = 158
                self.match(RulesParser.ITEM)
                self.state = 163
                self._errHandler.sync(self)
                _la = self._input.LA(1)
                while _la==7:
                    self.state = 159
                    self.match(RulesParser.T__6)
                    self.state = 160
                    self.match(RulesParser.ITEM)
                    self.state = 165
                    self._errHandler.sync(self)
                    _la = self._input.LA(1)

                self.state = 166
                self.match(RulesParser.T__1)
                pass

            elif la_ == 2:
                self.enterOuterAlt(localctx, 2)
                self.state = 168
                self._errHandler.sync(self)
                _la = self._input.LA(1)
                if _la==22:
                    self.state = 167
                    self.match(RulesParser.NOT)


                self.state = 170
                self.match(RulesParser.FUNC)
                self.state = 171
                self.match(RulesParser.T__0)
                self.state = 172
                self.value()
                self.state = 173
                self.match(RulesParser.T__1)
                pass

            elif la_ == 3:
                self.enterOuterAlt(localctx, 3)
                self.state = 176
                self._errHandler.sync(self)
                _la = self._input.LA(1)
                if _la==22:
                    self.state = 175
                    self.match(RulesParser.NOT)


                self.state = 178
                self.match(RulesParser.FUNC)
                self.state = 179
                self.match(RulesParser.T__0)
                self.state = 180
                self.match(RulesParser.LIT)
                self.state = 181
                self.match(RulesParser.T__1)
                pass

            elif la_ == 4:
                self.enterOuterAlt(localctx, 4)
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
                self.match(RulesParser.INT)
                self.state = 188
                self.match(RulesParser.T__1)
                pass

            elif la_ == 5:
                self.enterOuterAlt(localctx, 5)
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
                self.match(RulesParser.FLOAT)
                self.state = 195
                self.match(RulesParser.T__1)
                pass

            elif la_ == 6:
                self.enterOuterAlt(localctx, 6)
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
                self.match(RulesParser.PLACE)
                self.state = 202
                self.match(RulesParser.T__1)
                pass

            elif la_ == 7:
                self.enterOuterAlt(localctx, 7)
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
                self.match(RulesParser.REF)
                self.state = 209
                self.match(RulesParser.T__1)
                pass

            elif la_ == 8:
                self.enterOuterAlt(localctx, 8)
                self.state = 211
                self._errHandler.sync(self)
                _la = self._input.LA(1)
                if _la==22:
                    self.state = 210
                    self.match(RulesParser.NOT)


                self.state = 213
                self.match(RulesParser.FUNC)
                self.state = 216
                self._errHandler.sync(self)
                la_ = self._interp.adaptivePredict(self._input,20,self._ctx)
                if la_ == 1:
                    self.state = 214
                    self.match(RulesParser.T__0)
                    self.state = 215
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
            self.state = 256
            self._errHandler.sync(self)
            token = self._input.LA(1)
            if token in [25]:
                localctx = RulesParser.IfThenElseContext(self, localctx)
                self.enterOuterAlt(localctx, 1)
                self.state = 220
                self.match(RulesParser.IF)
                self.state = 221
                self.match(RulesParser.T__0)
                self.state = 222
                self.boolExpr(0)
                self.state = 223
                self.match(RulesParser.T__1)
                self.state = 224
                self.match(RulesParser.T__4)
                self.state = 225
                self.boolExpr(0)
                self.state = 226
                self.match(RulesParser.T__5)
                self.state = 238
                self._errHandler.sync(self)
                _alt = self._interp.adaptivePredict(self._input,22,self._ctx)
                while _alt!=2 and _alt!=ATN.INVALID_ALT_NUMBER:
                    if _alt==1:
                        self.state = 227
                        self.match(RulesParser.ELSE)
                        self.state = 228
                        self.match(RulesParser.IF)
                        self.state = 229
                        self.match(RulesParser.T__0)
                        self.state = 230
                        self.boolExpr(0)
                        self.state = 231
                        self.match(RulesParser.T__1)
                        self.state = 232
                        self.match(RulesParser.T__4)
                        self.state = 233
                        self.boolExpr(0)
                        self.state = 234
                        self.match(RulesParser.T__5) 
                    self.state = 240
                    self._errHandler.sync(self)
                    _alt = self._interp.adaptivePredict(self._input,22,self._ctx)

                self.state = 246
                self._errHandler.sync(self)
                la_ = self._interp.adaptivePredict(self._input,23,self._ctx)
                if la_ == 1:
                    self.state = 241
                    self.match(RulesParser.ELSE)
                    self.state = 242
                    self.match(RulesParser.T__4)
                    self.state = 243
                    self.boolExpr(0)
                    self.state = 244
                    self.match(RulesParser.T__5)


                pass
            elif token in [1]:
                localctx = RulesParser.PyTernaryContext(self, localctx)
                self.enterOuterAlt(localctx, 2)
                self.state = 248
                self.match(RulesParser.T__0)
                self.state = 249
                self.boolExpr(0)
                self.state = 250
                self.match(RulesParser.IF)
                self.state = 251
                self.boolExpr(0)
                self.state = 252
                self.match(RulesParser.ELSE)
                self.state = 253
                self.boolExpr(0)
                self.state = 254
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
            self.state = 258
            self.match(RulesParser.IF)
            self.state = 259
            self.match(RulesParser.T__0)
            self.state = 260
            self.boolExpr(0)
            self.state = 261
            self.match(RulesParser.T__1)
            self.state = 262
            self.match(RulesParser.T__4)
            self.state = 263
            self.num()
            self.state = 264
            self.match(RulesParser.T__5)
            self.state = 276
            self._errHandler.sync(self)
            _alt = self._interp.adaptivePredict(self._input,25,self._ctx)
            while _alt!=2 and _alt!=ATN.INVALID_ALT_NUMBER:
                if _alt==1:
                    self.state = 265
                    self.match(RulesParser.ELSE)
                    self.state = 266
                    self.match(RulesParser.IF)
                    self.state = 267
                    self.match(RulesParser.T__0)
                    self.state = 268
                    self.boolExpr(0)
                    self.state = 269
                    self.match(RulesParser.T__1)
                    self.state = 270
                    self.match(RulesParser.T__4)
                    self.state = 271
                    self.num()
                    self.state = 272
                    self.match(RulesParser.T__5) 
                self.state = 278
                self._errHandler.sync(self)
                _alt = self._interp.adaptivePredict(self._input,25,self._ctx)

            self.state = 284
            self._errHandler.sync(self)
            la_ = self._interp.adaptivePredict(self._input,26,self._ctx)
            if la_ == 1:
                self.state = 279
                self.match(RulesParser.ELSE)
                self.state = 280
                self.match(RulesParser.T__4)
                self.state = 281
                self.num()
                self.state = 282
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
            self.state = 286
            self.match(RulesParser.IF)
            self.state = 287
            self.match(RulesParser.T__0)
            self.state = 288
            self.boolExpr(0)
            self.state = 289
            self.match(RulesParser.T__1)
            self.state = 290
            self.match(RulesParser.T__4)
            self.state = 291
            self.str_()
            self.state = 292
            self.match(RulesParser.T__5)
            self.state = 304
            self._errHandler.sync(self)
            _alt = self._interp.adaptivePredict(self._input,27,self._ctx)
            while _alt!=2 and _alt!=ATN.INVALID_ALT_NUMBER:
                if _alt==1:
                    self.state = 293
                    self.match(RulesParser.ELSE)
                    self.state = 294
                    self.match(RulesParser.IF)
                    self.state = 295
                    self.match(RulesParser.T__0)
                    self.state = 296
                    self.boolExpr(0)
                    self.state = 297
                    self.match(RulesParser.T__1)
                    self.state = 298
                    self.match(RulesParser.T__4)
                    self.state = 299
                    self.str_()
                    self.state = 300
                    self.match(RulesParser.T__5) 
                self.state = 306
                self._errHandler.sync(self)
                _alt = self._interp.adaptivePredict(self._input,27,self._ctx)

            self.state = 312
            self._errHandler.sync(self)
            _la = self._input.LA(1)
            if _la==26:
                self.state = 307
                self.match(RulesParser.ELSE)
                self.state = 308
                self.match(RulesParser.T__4)
                self.state = 309
                self.str_()
                self.state = 310
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
            self.state = 403
            self._errHandler.sync(self)
            la_ = self._interp.adaptivePredict(self._input,39,self._ctx)
            if la_ == 1:
                localctx = RulesParser.PerItemBoolContext(self, localctx)
                self.enterOuterAlt(localctx, 1)
                self.state = 314
                self.match(RulesParser.PER)
                self.state = 315
                self.match(RulesParser.ITEM)
                self.state = 316
                self.match(RulesParser.T__4)
                self.state = 322 
                self._errHandler.sync(self)
                _la = self._input.LA(1)
                while True:
                    self.state = 317
                    self.match(RulesParser.INT)
                    self.state = 318
                    self.match(RulesParser.T__7)
                    self.state = 319
                    self.boolExpr(0)
                    self.state = 320
                    self.match(RulesParser.T__6)
                    self.state = 324 
                    self._errHandler.sync(self)
                    _la = self._input.LA(1)
                    if not (_la==37):
                        break

                self.state = 326
                self.match(RulesParser.T__8)
                self.state = 327
                self.match(RulesParser.T__7)
                self.state = 328
                self.boolExpr(0)
                self.state = 330
                self._errHandler.sync(self)
                _la = self._input.LA(1)
                if _la==7:
                    self.state = 329
                    self.match(RulesParser.T__6)


                self.state = 332
                self.match(RulesParser.T__5)
                pass

            elif la_ == 2:
                localctx = RulesParser.PerSettingBoolContext(self, localctx)
                self.enterOuterAlt(localctx, 2)
                self.state = 334
                self.match(RulesParser.PER)
                self.state = 335
                self.match(RulesParser.SETTING)
                self.state = 336
                self.match(RulesParser.T__4)
                self.state = 355
                self._errHandler.sync(self)
                token = self._input.LA(1)
                if token in [37]:
                    self.state = 342 
                    self._errHandler.sync(self)
                    _la = self._input.LA(1)
                    while True:
                        self.state = 337
                        self.match(RulesParser.INT)
                        self.state = 338
                        self.match(RulesParser.T__7)
                        self.state = 339
                        self.boolExpr(0)
                        self.state = 340
                        self.match(RulesParser.T__6)
                        self.state = 344 
                        self._errHandler.sync(self)
                        _la = self._input.LA(1)
                        if not (_la==37):
                            break

                    pass
                elif token in [35]:
                    self.state = 351 
                    self._errHandler.sync(self)
                    _la = self._input.LA(1)
                    while True:
                        self.state = 346
                        self.match(RulesParser.LIT)
                        self.state = 347
                        self.match(RulesParser.T__7)
                        self.state = 348
                        self.boolExpr(0)
                        self.state = 349
                        self.match(RulesParser.T__6)
                        self.state = 353 
                        self._errHandler.sync(self)
                        _la = self._input.LA(1)
                        if not (_la==35):
                            break

                    pass
                else:
                    raise NoViableAltException(self)

                self.state = 357
                self.match(RulesParser.T__8)
                self.state = 358
                self.match(RulesParser.T__7)
                self.state = 359
                self.boolExpr(0)
                self.state = 361
                self._errHandler.sync(self)
                _la = self._input.LA(1)
                if _la==7:
                    self.state = 360
                    self.match(RulesParser.T__6)


                self.state = 363
                self.match(RulesParser.T__5)
                pass

            elif la_ == 3:
                localctx = RulesParser.MatchRefBoolContext(self, localctx)
                self.enterOuterAlt(localctx, 3)
                self.state = 365
                self.match(RulesParser.PER)
                self.state = 366
                self.match(RulesParser.REF)
                self.state = 367
                self.match(RulesParser.T__4)
                self.state = 380 
                self._errHandler.sync(self)
                _la = self._input.LA(1)
                while True:
                    self.state = 368
                    self.match(RulesParser.ITEM)
                    self.state = 373
                    self._errHandler.sync(self)
                    _la = self._input.LA(1)
                    while _la==10:
                        self.state = 369
                        self.match(RulesParser.T__9)
                        self.state = 370
                        self.match(RulesParser.ITEM)
                        self.state = 375
                        self._errHandler.sync(self)
                        _la = self._input.LA(1)

                    self.state = 376
                    self.match(RulesParser.T__7)
                    self.state = 377
                    self.boolExpr(0)
                    self.state = 378
                    self.match(RulesParser.T__6)
                    self.state = 382 
                    self._errHandler.sync(self)
                    _la = self._input.LA(1)
                    if not (_la==30):
                        break

                self.state = 384
                self.match(RulesParser.T__8)
                self.state = 385
                self.match(RulesParser.T__7)
                self.state = 386
                self.boolExpr(0)
                self.state = 388
                self._errHandler.sync(self)
                _la = self._input.LA(1)
                if _la==7:
                    self.state = 387
                    self.match(RulesParser.T__6)


                self.state = 390
                self.match(RulesParser.T__5)
                pass

            elif la_ == 4:
                localctx = RulesParser.RefInListContext(self, localctx)
                self.enterOuterAlt(localctx, 4)
                self.state = 392
                self.match(RulesParser.REF)
                self.state = 393
                self.match(RulesParser.IN)
                self.state = 394
                self.match(RulesParser.T__10)
                self.state = 395
                self.match(RulesParser.ITEM)
                self.state = 398 
                self._errHandler.sync(self)
                _la = self._input.LA(1)
                while True:
                    self.state = 396
                    self.match(RulesParser.T__6)
                    self.state = 397
                    self.match(RulesParser.ITEM)
                    self.state = 400 
                    self._errHandler.sync(self)
                    _la = self._input.LA(1)
                    if not (_la==7):
                        break

                self.state = 402
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
            self.state = 487
            self._errHandler.sync(self)
            la_ = self._interp.adaptivePredict(self._input,50,self._ctx)
            if la_ == 1:
                localctx = RulesParser.PerItemIntContext(self, localctx)
                self.enterOuterAlt(localctx, 1)
                self.state = 405
                self.match(RulesParser.PER)
                self.state = 406
                self.match(RulesParser.ITEM)
                self.state = 407
                self.match(RulesParser.T__4)
                self.state = 413 
                self._errHandler.sync(self)
                _la = self._input.LA(1)
                while True:
                    self.state = 408
                    self.match(RulesParser.INT)
                    self.state = 409
                    self.match(RulesParser.T__7)
                    self.state = 410
                    self.num()
                    self.state = 411
                    self.match(RulesParser.T__6)
                    self.state = 415 
                    self._errHandler.sync(self)
                    _la = self._input.LA(1)
                    if not (_la==37):
                        break

                self.state = 417
                self.match(RulesParser.T__8)
                self.state = 418
                self.match(RulesParser.T__7)
                self.state = 419
                self.num()
                self.state = 421
                self._errHandler.sync(self)
                _la = self._input.LA(1)
                if _la==7:
                    self.state = 420
                    self.match(RulesParser.T__6)


                self.state = 423
                self.match(RulesParser.T__5)
                pass

            elif la_ == 2:
                localctx = RulesParser.PerRefIntContext(self, localctx)
                self.enterOuterAlt(localctx, 2)
                self.state = 425
                self.match(RulesParser.PER)
                self.state = 426
                self.match(RulesParser.REF)
                self.state = 427
                self.match(RulesParser.T__4)
                self.state = 446
                self._errHandler.sync(self)
                token = self._input.LA(1)
                if token in [37]:
                    self.state = 433 
                    self._errHandler.sync(self)
                    _la = self._input.LA(1)
                    while True:
                        self.state = 428
                        self.match(RulesParser.INT)
                        self.state = 429
                        self.match(RulesParser.T__7)
                        self.state = 430
                        self.num()
                        self.state = 431
                        self.match(RulesParser.T__6)
                        self.state = 435 
                        self._errHandler.sync(self)
                        _la = self._input.LA(1)
                        if not (_la==37):
                            break

                    pass
                elif token in [35]:
                    self.state = 442 
                    self._errHandler.sync(self)
                    _la = self._input.LA(1)
                    while True:
                        self.state = 437
                        self.match(RulesParser.LIT)
                        self.state = 438
                        self.match(RulesParser.T__7)
                        self.state = 439
                        self.num()
                        self.state = 440
                        self.match(RulesParser.T__6)
                        self.state = 444 
                        self._errHandler.sync(self)
                        _la = self._input.LA(1)
                        if not (_la==35):
                            break

                    pass
                else:
                    raise NoViableAltException(self)

                self.state = 448
                self.match(RulesParser.T__8)
                self.state = 449
                self.match(RulesParser.T__7)
                self.state = 450
                self.num()
                self.state = 452
                self._errHandler.sync(self)
                _la = self._input.LA(1)
                if _la==7:
                    self.state = 451
                    self.match(RulesParser.T__6)


                self.state = 454
                self.match(RulesParser.T__5)
                pass

            elif la_ == 3:
                localctx = RulesParser.PerSettingIntContext(self, localctx)
                self.enterOuterAlt(localctx, 3)
                self.state = 456
                self.match(RulesParser.PER)
                self.state = 457
                self.match(RulesParser.SETTING)
                self.state = 458
                self.match(RulesParser.T__4)
                self.state = 477
                self._errHandler.sync(self)
                token = self._input.LA(1)
                if token in [37]:
                    self.state = 464 
                    self._errHandler.sync(self)
                    _la = self._input.LA(1)
                    while True:
                        self.state = 459
                        self.match(RulesParser.INT)
                        self.state = 460
                        self.match(RulesParser.T__7)
                        self.state = 461
                        self.num()
                        self.state = 462
                        self.match(RulesParser.T__6)
                        self.state = 466 
                        self._errHandler.sync(self)
                        _la = self._input.LA(1)
                        if not (_la==37):
                            break

                    pass
                elif token in [35]:
                    self.state = 473 
                    self._errHandler.sync(self)
                    _la = self._input.LA(1)
                    while True:
                        self.state = 468
                        self.match(RulesParser.LIT)
                        self.state = 469
                        self.match(RulesParser.T__7)
                        self.state = 470
                        self.num()
                        self.state = 471
                        self.match(RulesParser.T__6)
                        self.state = 475 
                        self._errHandler.sync(self)
                        _la = self._input.LA(1)
                        if not (_la==35):
                            break

                    pass
                else:
                    raise NoViableAltException(self)

                self.state = 479
                self.match(RulesParser.T__8)
                self.state = 480
                self.match(RulesParser.T__7)
                self.state = 481
                self.num()
                self.state = 483
                self._errHandler.sync(self)
                _la = self._input.LA(1)
                if _la==7:
                    self.state = 482
                    self.match(RulesParser.T__6)


                self.state = 485
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
            self.state = 571
            self._errHandler.sync(self)
            la_ = self._interp.adaptivePredict(self._input,61,self._ctx)
            if la_ == 1:
                localctx = RulesParser.PerItemStrContext(self, localctx)
                self.enterOuterAlt(localctx, 1)
                self.state = 489
                self.match(RulesParser.PER)
                self.state = 490
                self.match(RulesParser.ITEM)
                self.state = 491
                self.match(RulesParser.T__4)
                self.state = 497 
                self._errHandler.sync(self)
                _la = self._input.LA(1)
                while True:
                    self.state = 492
                    self.match(RulesParser.INT)
                    self.state = 493
                    self.match(RulesParser.T__7)
                    self.state = 494
                    self.str_()
                    self.state = 495
                    self.match(RulesParser.T__6)
                    self.state = 499 
                    self._errHandler.sync(self)
                    _la = self._input.LA(1)
                    if not (_la==37):
                        break

                self.state = 501
                self.match(RulesParser.T__8)
                self.state = 502
                self.match(RulesParser.T__7)
                self.state = 503
                self.str_()
                self.state = 505
                self._errHandler.sync(self)
                _la = self._input.LA(1)
                if _la==7:
                    self.state = 504
                    self.match(RulesParser.T__6)


                self.state = 507
                self.match(RulesParser.T__5)
                pass

            elif la_ == 2:
                localctx = RulesParser.PerRefStrContext(self, localctx)
                self.enterOuterAlt(localctx, 2)
                self.state = 509
                self.match(RulesParser.PER)
                self.state = 510
                self.match(RulesParser.REF)
                self.state = 511
                self.match(RulesParser.T__4)
                self.state = 530
                self._errHandler.sync(self)
                token = self._input.LA(1)
                if token in [37]:
                    self.state = 517 
                    self._errHandler.sync(self)
                    _la = self._input.LA(1)
                    while True:
                        self.state = 512
                        self.match(RulesParser.INT)
                        self.state = 513
                        self.match(RulesParser.T__7)
                        self.state = 514
                        self.str_()
                        self.state = 515
                        self.match(RulesParser.T__6)
                        self.state = 519 
                        self._errHandler.sync(self)
                        _la = self._input.LA(1)
                        if not (_la==37):
                            break

                    pass
                elif token in [35]:
                    self.state = 526 
                    self._errHandler.sync(self)
                    _la = self._input.LA(1)
                    while True:
                        self.state = 521
                        self.match(RulesParser.LIT)
                        self.state = 522
                        self.match(RulesParser.T__7)
                        self.state = 523
                        self.str_()
                        self.state = 524
                        self.match(RulesParser.T__6)
                        self.state = 528 
                        self._errHandler.sync(self)
                        _la = self._input.LA(1)
                        if not (_la==35):
                            break

                    pass
                else:
                    raise NoViableAltException(self)

                self.state = 532
                self.match(RulesParser.T__8)
                self.state = 533
                self.match(RulesParser.T__7)
                self.state = 534
                self.str_()
                self.state = 536
                self._errHandler.sync(self)
                _la = self._input.LA(1)
                if _la==7:
                    self.state = 535
                    self.match(RulesParser.T__6)


                self.state = 538
                self.match(RulesParser.T__5)
                pass

            elif la_ == 3:
                localctx = RulesParser.PerSettingStrContext(self, localctx)
                self.enterOuterAlt(localctx, 3)
                self.state = 540
                self.match(RulesParser.PER)
                self.state = 541
                self.match(RulesParser.SETTING)
                self.state = 542
                self.match(RulesParser.T__4)
                self.state = 561
                self._errHandler.sync(self)
                token = self._input.LA(1)
                if token in [37]:
                    self.state = 548 
                    self._errHandler.sync(self)
                    _la = self._input.LA(1)
                    while True:
                        self.state = 543
                        self.match(RulesParser.INT)
                        self.state = 544
                        self.match(RulesParser.T__7)
                        self.state = 545
                        self.str_()
                        self.state = 546
                        self.match(RulesParser.T__6)
                        self.state = 550 
                        self._errHandler.sync(self)
                        _la = self._input.LA(1)
                        if not (_la==37):
                            break

                    pass
                elif token in [35]:
                    self.state = 557 
                    self._errHandler.sync(self)
                    _la = self._input.LA(1)
                    while True:
                        self.state = 552
                        self.match(RulesParser.LIT)
                        self.state = 553
                        self.match(RulesParser.T__7)
                        self.state = 554
                        self.str_()
                        self.state = 555
                        self.match(RulesParser.T__6)
                        self.state = 559 
                        self._errHandler.sync(self)
                        _la = self._input.LA(1)
                        if not (_la==35):
                            break

                    pass
                else:
                    raise NoViableAltException(self)

                self.state = 563
                self.match(RulesParser.T__8)
                self.state = 564
                self.match(RulesParser.T__7)
                self.state = 565
                self.str_()
                self.state = 567
                self._errHandler.sync(self)
                _la = self._input.LA(1)
                if _la==7:
                    self.state = 566
                    self.match(RulesParser.T__6)


                self.state = 569
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
            self.state = 597
            self._errHandler.sync(self)
            la_ = self._interp.adaptivePredict(self._input,62,self._ctx)
            if la_ == 1:
                self.enterOuterAlt(localctx, 1)
                self.state = 573
                self.value()
                self.state = 574
                self.match(RulesParser.T__12)
                self.state = 575
                self.num()
                pass

            elif la_ == 2:
                self.enterOuterAlt(localctx, 2)
                self.state = 577
                self.value()
                self.state = 578
                self.match(RulesParser.T__13)
                self.state = 579
                self.num()
                pass

            elif la_ == 3:
                self.enterOuterAlt(localctx, 3)
                self.state = 581
                self.value()
                self.state = 582
                self.match(RulesParser.T__14)
                self.state = 583
                self.num()
                pass

            elif la_ == 4:
                self.enterOuterAlt(localctx, 4)
                self.state = 585
                self.value()
                self.state = 586
                self.match(RulesParser.T__15)
                self.state = 587
                self.num()
                pass

            elif la_ == 5:
                self.enterOuterAlt(localctx, 5)
                self.state = 589
                self.value()
                self.state = 590
                self.match(RulesParser.T__16)
                self.state = 591
                self.num()
                pass

            elif la_ == 6:
                self.enterOuterAlt(localctx, 6)
                self.state = 593
                self.value()
                self.state = 594
                self.match(RulesParser.T__17)
                self.state = 595
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
            self.state = 607
            self._errHandler.sync(self)
            la_ = self._interp.adaptivePredict(self._input,63,self._ctx)
            if la_ == 1:
                self.enterOuterAlt(localctx, 1)
                self.state = 599
                self.value()
                self.state = 600
                self.match(RulesParser.T__12)
                self.state = 601
                self.match(RulesParser.LIT)
                pass

            elif la_ == 2:
                self.enterOuterAlt(localctx, 2)
                self.state = 603
                self.value()
                self.state = 604
                self.match(RulesParser.T__13)
                self.state = 605
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
            self.state = 609
            self.value()
            self.state = 610
            self.match(RulesParser.T__18)
            self.state = 611
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
            self.state = 613
            self.match(RulesParser.REF)
            self.state = 614
            self.match(RulesParser.T__12)
            self.state = 615
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
            self.state = 638
            self._errHandler.sync(self)
            la_ = self._interp.adaptivePredict(self._input,66,self._ctx)
            if la_ == 1:
                self.enterOuterAlt(localctx, 1)
                self.state = 617
                self.match(RulesParser.FUNC)
                self.state = 618
                self.match(RulesParser.T__0)
                self.state = 619
                self.match(RulesParser.ITEM)
                self.state = 620
                self.match(RulesParser.T__1)
                pass

            elif la_ == 2:
                self.enterOuterAlt(localctx, 2)
                self.state = 621
                self.match(RulesParser.FUNC)
                self.state = 622
                self.match(RulesParser.T__0)
                self.state = 623
                self.num()
                self.state = 628
                self._errHandler.sync(self)
                _la = self._input.LA(1)
                while _la==7:
                    self.state = 624
                    self.match(RulesParser.T__6)
                    self.state = 625
                    self.num()
                    self.state = 630
                    self._errHandler.sync(self)
                    _la = self._input.LA(1)

                self.state = 631
                self.match(RulesParser.T__1)
                pass

            elif la_ == 3:
                self.enterOuterAlt(localctx, 3)
                self.state = 633
                self.match(RulesParser.FUNC)
                self.state = 636
                self._errHandler.sync(self)
                la_ = self._interp.adaptivePredict(self._input,65,self._ctx)
                if la_ == 1:
                    self.state = 634
                    self.match(RulesParser.T__0)
                    self.state = 635
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
            self.state = 640
            self.baseNum()
            self.state = 641
            self.match(RulesParser.BINOP)
            self.state = 642
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
            self.state = 646
            self._errHandler.sync(self)
            la_ = self._interp.adaptivePredict(self._input,67,self._ctx)
            if la_ == 1:
                self.enterOuterAlt(localctx, 1)
                self.state = 644
                self.baseNum()
                pass

            elif la_ == 2:
                self.enterOuterAlt(localctx, 2)
                self.state = 645
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
            self.state = 656
            self._errHandler.sync(self)
            la_ = self._interp.adaptivePredict(self._input,68,self._ctx)
            if la_ == 1:
                self.enterOuterAlt(localctx, 1)
                self.state = 648
                self.match(RulesParser.INT)
                pass

            elif la_ == 2:
                self.enterOuterAlt(localctx, 2)
                self.state = 649
                self.match(RulesParser.CONST)
                pass

            elif la_ == 3:
                self.enterOuterAlt(localctx, 3)
                self.state = 650
                self.match(RulesParser.SETTING)
                pass

            elif la_ == 4:
                self.enterOuterAlt(localctx, 4)
                self.state = 651
                self.match(RulesParser.REF)
                pass

            elif la_ == 5:
                self.enterOuterAlt(localctx, 5)
                self.state = 652
                self.value()
                pass

            elif la_ == 6:
                self.enterOuterAlt(localctx, 6)
                self.state = 653
                self.switchNum()
                pass

            elif la_ == 7:
                self.enterOuterAlt(localctx, 7)
                self.state = 654
                self.funcNum()
                pass

            elif la_ == 8:
                self.enterOuterAlt(localctx, 8)
                self.state = 655
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
            self.state = 665
            self._errHandler.sync(self)
            token = self._input.LA(1)
            if token in [31]:
                localctx = RulesParser.SettingContext(self, localctx)
                self.enterOuterAlt(localctx, 1)
                self.state = 658
                self.match(RulesParser.SETTING)
                self.state = 662
                self._errHandler.sync(self)
                la_ = self._interp.adaptivePredict(self._input,69,self._ctx)
                if la_ == 1:
                    self.state = 659
                    self.match(RulesParser.T__10)
                    self.state = 660
                    _la = self._input.LA(1)
                    if not(_la==30 or _la==35):
                        self._errHandler.recoverInline(self)
                    else:
                        self._errHandler.reportMatch(self)
                        self.consume()
                    self.state = 661
                    self.match(RulesParser.T__11)


                pass
            elif token in [32]:
                localctx = RulesParser.ArgumentContext(self, localctx)
                self.enterOuterAlt(localctx, 2)
                self.state = 664
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
        self.enterRule(localctx, 40, self.RULE_item)
        self._la = 0 # Token type
        try:
            self.state = 684
            self._errHandler.sync(self)
            la_ = self._interp.adaptivePredict(self._input,73,self._ctx)
            if la_ == 1:
                localctx = RulesParser.ItemCountContext(self, localctx)
                self.enterOuterAlt(localctx, 1)
                self.state = 676
                self._errHandler.sync(self)
                token = self._input.LA(1)
                if token in [30]:
                    self.state = 667
                    self.match(RulesParser.ITEM)
                    self.state = 668
                    self.match(RulesParser.T__4)
                    self.state = 669
                    _la = self._input.LA(1)
                    if not(_la==31 or _la==37):
                        self._errHandler.recoverInline(self)
                    else:
                        self._errHandler.reportMatch(self)
                        self.consume()
                    self.state = 670
                    self.match(RulesParser.T__5)
                    pass
                elif token in [1]:
                    self.state = 671
                    self.match(RulesParser.T__0)
                    self.state = 672
                    self.match(RulesParser.ITEM)
                    self.state = 673
                    self.match(RulesParser.T__6)
                    self.state = 674
                    _la = self._input.LA(1)
                    if not(_la==31 or _la==37):
                        self._errHandler.recoverInline(self)
                    else:
                        self._errHandler.reportMatch(self)
                        self.consume()
                    self.state = 675
                    self.match(RulesParser.T__1)
                    pass
                else:
                    raise NoViableAltException(self)

                pass

            elif la_ == 2:
                localctx = RulesParser.OneItemContext(self, localctx)
                self.enterOuterAlt(localctx, 2)
                self.state = 679
                self._errHandler.sync(self)
                _la = self._input.LA(1)
                if _la==22:
                    self.state = 678
                    self.match(RulesParser.NOT)


                self.state = 681
                self.match(RulesParser.ITEM)
                pass

            elif la_ == 3:
                localctx = RulesParser.OneLitItemContext(self, localctx)
                self.enterOuterAlt(localctx, 3)
                self.state = 682
                self.match(RulesParser.LIT)
                pass

            elif la_ == 4:
                localctx = RulesParser.OneArgumentContext(self, localctx)
                self.enterOuterAlt(localctx, 4)
                self.state = 683
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
        self.enterRule(localctx, 42, self.RULE_str)
        try:
            self.state = 690
            self._errHandler.sync(self)
            token = self._input.LA(1)
            if token in [35]:
                self.enterOuterAlt(localctx, 1)
                self.state = 686
                self.match(RulesParser.LIT)
                pass
            elif token in [31, 32]:
                self.enterOuterAlt(localctx, 2)
                self.state = 687
                self.value()
                pass
            elif token in [25]:
                self.enterOuterAlt(localctx, 3)
                self.state = 688
                self.condStr()
                pass
            elif token in [28]:
                self.enterOuterAlt(localctx, 4)
                self.state = 689
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
        self.enterRule(localctx, 44, self.RULE_somewhere)
        self._la = 0 # Token type
        try:
            self.state = 711
            self._errHandler.sync(self)
            la_ = self._interp.adaptivePredict(self._input,78,self._ctx)
            if la_ == 1:
                self.enterOuterAlt(localctx, 1)
                self.state = 693
                self._errHandler.sync(self)
                _la = self._input.LA(1)
                if _la==22:
                    self.state = 692
                    self.match(RulesParser.NOT)


                self.state = 695
                self.match(RulesParser.WITHIN)
                self.state = 696
                self.match(RulesParser.PLACE)
                pass

            elif la_ == 2:
                self.enterOuterAlt(localctx, 2)
                self.state = 698
                self._errHandler.sync(self)
                _la = self._input.LA(1)
                if _la==22:
                    self.state = 697
                    self.match(RulesParser.NOT)


                self.state = 700
                self.match(RulesParser.WITHIN)
                self.state = 701
                self.match(RulesParser.T__0)
                self.state = 702
                self.match(RulesParser.PLACE)
                self.state = 707
                self._errHandler.sync(self)
                _la = self._input.LA(1)
                while _la==7:
                    self.state = 703
                    self.match(RulesParser.T__6)
                    self.state = 704
                    self.match(RulesParser.PLACE)
                    self.state = 709
                    self._errHandler.sync(self)
                    _la = self._input.LA(1)

                self.state = 710
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
        self.enterRule(localctx, 46, self.RULE_refSomewhere)
        self._la = 0 # Token type
        try:
            self.state = 731
            self._errHandler.sync(self)
            la_ = self._interp.adaptivePredict(self._input,82,self._ctx)
            if la_ == 1:
                localctx = RulesParser.RefInPlaceRefContext(self, localctx)
                self.enterOuterAlt(localctx, 1)
                self.state = 713
                self.match(RulesParser.REF)
                self.state = 715
                self._errHandler.sync(self)
                _la = self._input.LA(1)
                if _la==22:
                    self.state = 714
                    self.match(RulesParser.NOT)


                self.state = 717
                self.match(RulesParser.WITHIN)
                self.state = 718
                self.match(RulesParser.REF)
                pass

            elif la_ == 2:
                localctx = RulesParser.RefInPlaceNameContext(self, localctx)
                self.enterOuterAlt(localctx, 2)
                self.state = 719
                self.match(RulesParser.REF)
                self.state = 721
                self._errHandler.sync(self)
                _la = self._input.LA(1)
                if _la==22:
                    self.state = 720
                    self.match(RulesParser.NOT)


                self.state = 723
                self.match(RulesParser.WITHIN)
                self.state = 724
                self.match(RulesParser.PLACE)
                pass

            elif la_ == 3:
                localctx = RulesParser.RefInFuncContext(self, localctx)
                self.enterOuterAlt(localctx, 3)
                self.state = 725
                self.match(RulesParser.REF)
                self.state = 727
                self._errHandler.sync(self)
                _la = self._input.LA(1)
                if _la==22:
                    self.state = 726
                    self.match(RulesParser.NOT)


                self.state = 729
                self.match(RulesParser.WITHIN)
                self.state = 730
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
                return self.precpred(self._ctx, 16)
         

            if predIndex == 1:
                return self.precpred(self._ctx, 15)
         




