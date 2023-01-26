grammar Rules;

accessRule
    : '(' accessRule ')'
    | accessRule AND accessRule
    | accessRule OR accessRule
    | meta
    | invoke
    | cond
    | cmp
    | item
    | value
    | TRUE
    | FALSE
    ;

meta    : FUNC '(' LIT ',' accessRule ')'
        | FUNC '(' accessRule ')'
        ;

invoke  : FUNC '(' ITEM (',' ITEM)* ')'   // must be 1+ items, 0 handled below
        | FUNC '(' value ')'
        | FUNC '(' LIT ')'
        | FUNC '(' INT ')'
        | FUNC '()'? // essentially a call with no arguments
        ;

cond    : IF '(' accessRule ')' '{' accessRule '}'
          ( ELSE '{' accessRule '}' )?                      # IfThenElse
        | '(' accessRule IF accessRule ELSE accessRule ')'  # PyTernary
        ;

cmp : value '==' int    # IntEq
    | value '==' LIT    # LitEq
    | value '!=' int    # IntNeq
    | value '!=' LIT    # LitNeq
    | value '>=' int    # Geq
    | value '<=' int    # Leq
    | value '<' int     # Lt
    | value '>' int     # Gt
    | value '&' int     # FlagMatch
    | REF '==' ITEM     # RefEq
    ;

int : INT | CONST ;

value   : SETTING '[' LIT ']'   # SettingSubscript
        | SETTING               # Setting
        | NOT SETTING           # NotSetting
        | REF                   # Argument
        ;

item    : ( ITEM '{' INT '}'
          | '(' ITEM ',' INT ')' )  # ItemCount
        | ITEM                      # OneItem
        | REF                       # OneArgument
        ;

/** Lexer rules (tokens) */
AND     : 'AND' | 'and' | '&&' ;
OR      : 'OR' | 'or' | '||' ;
NOT     : 'NOT' | 'not' | '!' ;
TRUE    : 'TRUE' | 'true' | 'True' ;
FALSE   : 'FALSE' | 'false' | 'False' ;
IF      : 'IF' | 'if' ;
ELSE    : 'ELSE' | 'else' ;
IN      : 'IN' | 'in' ;

ITEM    : [A-Z][a-z][a-zA-Z_]+ ;
SETTING : [a-z][a-z_]+ ;
REF     : '@' [a-z_]+ ;
FUNC    : '$' [A-Za-z_]+ ;
LIT     : '\'' (~'\'' | '\\\'' )* '\'';
CONST   : [A-Z][A-Z_]+ ;
INT     : '-'? [0-9]+ ;
WS      : [ \t\r\n]+ -> skip ;
