grammar Rules;

accessRule
    : '(' accessRule ')'
    | accessRule AND accessRule
    | accessRule OR accessRule
    // Ordering is important!
    | invoke  // a FUNC on a primitive
    | meta  // a FUNC on an accessRule
    | cond
    | cmp
    | flagMatch
    | refEq
    | item
    | value
    | TRUE
    | FALSE
    ;

// might remove this as those rules need to be separate for a traversal graph anyway
meta    : FUNC '(' LIT ',' accessRule ')'
        | FUNC '(' accessRule ')'
        ;

invoke  : NOT? FUNC '(' ITEM (',' ITEM)* ')'   // must be 1+ items, 0 handled below
        | NOT? FUNC '(' value ')'
        | NOT? FUNC '(' LIT ')'
        | NOT? FUNC '(' INT ')'
        | NOT? FUNC '(' FLOAT ')'
        | NOT? FUNC '()'? // essentially a call with no arguments
        ;

cond    : IF '(' accessRule ')' '{' accessRule '}'
          ( ELSE '{' accessRule '}' )?                      # IfThenElse
        | '(' accessRule IF accessRule ELSE accessRule ')'  # PyTernary
        ;

cmp : value '==' num
    | value '==' LIT
    | value '!=' num
    | value '!=' LIT
    | value '>=' num
    | value '<=' num
    | value '<' num
    | value '>' num
    ;

flagMatch : value '&' num ;
refEq : REF '==' ( ITEM | SETTING ) ;

num : INT | CONST ;

value   : NOT? SETTING ('[' ( LIT | ITEM ) ']')?   # Setting
        | REF                           # Argument
        ;

item    : ( ITEM '{' ( INT | SETTING ) '}'
          | '(' ITEM ',' ( INT | SETTING ) ')'
          )     # ItemCount
        | ITEM  # OneItem
        | LIT   # OneLitItem  // I don't like it and I introduced it
        | REF   # OneArgument
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

ITEM    : [A-Z][a-z][a-zA-Z_0-9]+ ;
SETTING : [a-z][a-z_0-9]+ ;
REF     : '@' [a-z_]+ ;
FUNC    : '$' [A-Za-z_]+ ;
LIT     : '\'' (~'\'' | '\\\'' )* '\'';
CONST   : [A-Z][A-Z_]+ ;
INT     : '-'? [0-9]+ ;
FLOAT   : '-'? [0-9]+[.][0-9]+ ;
WS      : [ \t\r\n]+ -> skip ;
