grammar Rules;

// TODO: rename boolExpr?
boolExpr
    : '(' boolExpr ')'
    | boolExpr AND boolExpr
    | boolExpr OR boolExpr
    // Ordering is important!
    | invoke  // a FUNC on a primitive
    | meta  // a FUNC on an boolExpr
    | switch
    | cond
    | cmp
    | flagMatch
    | refEq
    | item
    | value
    | TRUE
    | FALSE
    ;

action  : REF '=' ( str | num | TRUE | FALSE )
        | REF BINOP '=' num
        ;

// might remove this as those rules need to be separate for a traversal graph anyway
meta    : FUNC '(' LIT ',' boolExpr ')'
        | FUNC '(' boolExpr ')'
        ;

invoke  : NOT? FUNC '(' ITEM (',' ITEM)* ')'   // must be 1+ items, 0 handled below
        | NOT? FUNC '(' value ')'
        | NOT? FUNC '(' LIT ')'
        | NOT? FUNC '(' INT ')'
        | NOT? FUNC '(' FLOAT ')'
        | NOT? FUNC '()'? // essentially a call with no arguments
        ;

cond    : IF '(' boolExpr ')' '{' boolExpr '}'
          ( ELSE IF '(' boolExpr ')' '{' boolExpr '}' )*
          ( ELSE '{' boolExpr '}' )?                    # IfThenElse
        | '(' boolExpr IF boolExpr ELSE boolExpr ')'    # PyTernary
        ;

condNum : IF '(' boolExpr ')' '{' num '}'
          ( ELSE IF '(' boolExpr ')' '{' num '}' )*
          ( ELSE '{' num '}' )?
        ;
condStr : IF '(' boolExpr ')' '{' str '}'
          ( ELSE IF '(' boolExpr ')' '{' str '}' )*
          ( ELSE '{' str '}' )?
        ;


// TODO: sub names?
switch  : PER ITEM '{' ( INT '=>' boolExpr ';' )+
                        '_' '=>' boolExpr ';'? '}'
        | PER SETTING '{' ( ( INT | LIT ) '=>' boolExpr ';' )+
                        '_' '=>' boolExpr ';'? '}'
        | MATCH REF '{' ( ITEM ( '|' ITEM )* '=>' boolExpr ';' )+
                            '_' '=>' boolExpr ';'? '}'
        // simpler match expression where all results are true/false
        | REF IN '[' ITEM ( ',' ITEM )+ ']'
        ;
switchNum   : PER ITEM '{' ( INT '=>' num ';' )+ '_' '=>' num ';'? '}'
            | PER ( REF | SETTING ) '{' ( ( INT | LIT ) '=>' num ';' )+
                                    '_' '=>' num ';'? '}'
            ;
switchStr   : PER ITEM '{' ( INT '=>' str ';' )+ '_' '=>' str ';'? '}'
            | PER ( REF | SETTING ) '{' ( ( INT | LIT ) '=>' str ';' )+
                                    '_' '=>' str ';'? '}'
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

// Specifically where a function is expected to return an integer
funcNum : FUNC '(' ITEM ')'
        | FUNC '(' num ( ',' num )* ')'
        ;

mathNum : baseNum BINOP num ;

num : baseNum | mathNum ;

baseNum : INT | CONST | SETTING | value | switchNum | funcNum | condNum ;

value   : NOT? SETTING ('[' ( LIT | ITEM ) ']')?    # Setting
        | NOT? REF                                  # Argument
        ;

item    : ( ITEM '{' ( INT | SETTING ) '}'
          | '(' ITEM ',' ( INT | SETTING ) ')'
          )     # ItemCount
        | ITEM  # OneItem
        | LIT   # OneLitItem  // I don't like it and I introduced it
        | REF   # OneArgument
        ;

str : LIT | value | condStr | switchStr ;

/** Lexer rules (tokens) */
AND     : 'AND' | 'and' | '&&' ;
OR      : 'OR' | 'or' | '||' ;
NOT     : 'NOT' | 'not' | '!' ;
TRUE    : 'TRUE' | 'true' | 'True' ;
FALSE   : 'FALSE' | 'false' | 'False' ;
IF      : 'IF' | 'if' ;
ELSE    : 'ELSE' | 'else' ;
IN      : 'IN' | 'in' ;
PER     : 'PER' | 'per' ;
MATCH   : 'MATCH' | 'match' ;

ITEM    : [A-Z][a-z][a-zA-Z_0-9]+ ;
SETTING : [a-z][a-zA-Z_0-9]+ ;
REF     : '^' [a-z_][a-zA-Z_0-9.]+ ;
FUNC    : '$' [A-Za-z_][A-Za-z_0-9]+ ;
LIT     : '\'' (~'\'' | '\\\'' )* '\'';
CONST   : [A-Z][A-Z_0-9]+ ;
INT     : '-'? [0-9]+ ;
FLOAT   : '-'? [0-9]+[.][0-9]+ ;
BINOP   : [-+*/] ;
WS      : [ \t\r\n]+ -> skip ;
