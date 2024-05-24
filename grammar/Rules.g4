grammar Rules;

// TODO: rename boolExpr?
boolExpr
    : '(' boolExpr ')'
    | boolExpr AND boolExpr
    | boolExpr OR boolExpr
    // Ordering is important!
    | invoke  // a FUNC on a primitive
    | meta  // a FUNC on an boolExpr
    | switchBool
    | cond
    | refEq
    | cmp
    | cmpStr
    | flagMatch
    | itemList
    | item
    | NOT? value
    | somewhere
    | refSomewhere
    | TRUE
    | FALSE
    ;

actions : action (';' action)* ';'?;

// TODO? a "^here" builtin ref for the spot it's defined in, but is that just position?
// TODO: a "cycle" action for ints/enums
action  : REF '=' ( TRUE | FALSE | PLACE | ref | str | num )    # Set
        | REF BINOP '=' num                                     # Alter
        | invoke                                                # ActionHelper
        | IF '(' boolExpr ')' '{' actions '}'
          ( ELSE IF '(' boolExpr ')' '{' actions '}' )*
          ( ELSE '{' actions '}' )?                             # CondAction
        | SWAP REF ',' REF                                      # Swap
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
        | NOT? FUNC '(' PLACE (',' PLACE)* ')'
        | NOT? FUNC '(' ref (',' ref)* ')'
        | NOT? FUNC ('(' ')')? // essentially a call with no arguments
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


switchBool
        : PER ITEM '{' ( INT '=>' boolExpr ',' )+
                        '_' '=>' boolExpr ','? '}'      # PerItemBool
        | PER SETTING '{' 
            ( ( INT '=>' boolExpr ',' )+
            | ( LIT '=>' boolExpr ',' )+ )
            '_' '=>' boolExpr ','? '}'                  # PerSettingBool
        | PER ref '{' ( ITEM ( '|' ITEM )* '=>' boolExpr ',' )+
                        '_' '=>' boolExpr ','? '}'      # MatchRefBool
        // simpler match expression where all results are true/false
        | ref IN '[' ITEM ( ',' ITEM )+ ']'             # RefInList
        | ref IN '[' LIT ( ',' LIT )+ ']'               # RefStrInList
        ;
switchNum   : PER ITEM '{' ( INT '=>' num ',' )+ '_' '=>' num ','? '}'  # PerItemInt
            | PER ref '{'
                ( ( INT '=>' num ',' )+
                | ( LIT '=>' num ',' )+ )
                '_' '=>' num ','? '}'                                   # PerRefInt
            | PER SETTING '{'
                ( ( INT '=>' num ',' )+
                | ( LIT '=>' num ',' )+ )
                '_' '=>' num ','? '}'                                   # PerSettingInt
            ;
switchStr   : PER ITEM '{' ( INT '=>' str ',' )+ '_' '=>' str ','? '}'  # PerItemStr
            | PER ref '{'
                ( ( INT '=>' str ',' )+
                | ( LIT '=>' str ',' )+ )
                '_' '=>' str ','? '}'                                   # PerRefStr
            | PER SETTING '{'
                ( ( INT '=>' str ',' )+
                | ( LIT '=>' str ',' )+ )
                '_' '=>' str ','? '}'                                   # PerSettingStr
            ;

cmp : value '==' num
    | value '!=' num
    | value '>=' num
    | value '<=' num
    | value '<' num
    | value '>' num
    ;

cmpStr  : value '==' LIT
        | value '!=' LIT
        ;

flagMatch : value '&' num ;
refEq   : ( ref '==' ( ITEM | SETTING )
          | ref '!=' ( ITEM | SETTING )
          )                             # RefEqSimple
        | ( ref '==' ref
          | ref '!=' ref
          )                             # RefEqRef
        | ( ref '==' invoke
          | ref '!=' invoke
          )                             # RefEqInvoke
        ;

// Specifically where a function is expected to return an integer
funcNum : FUNC '(' ITEM ')'
        | FUNC '(' num ( ',' num )* ')'
        | FUNC ('(' ')')?
        ;

mathNum : baseNum BINOP num ;

num : baseNum | mathNum ;

baseNum : INT | CONST | SETTING | ref | value | switchNum | funcNum | condNum ;

value   : SETTING ('[' ( LIT | ITEM ) ']')?     # Setting
        | ref                                   # Argument
        ;

itemList : '[' (FUNC | item) (',' (FUNC | item))* ']';

item    : ( ITEM '{' ( INT | SETTING ) '}'
          | '(' ITEM ',' ( INT | SETTING ) ')'
          )             # ItemCount
        | NOT? ITEM     # OneItem
        | LIT           # OneLitItem  // I don't like it and I introduced it
        | ref           # OneArgument
        ;

str : LIT | value | condStr | switchStr ;

somewhere : NOT? WITHIN PLACE
          | NOT? WITHIN '(' PLACE (',' PLACE)* ')';

refSomewhere : ref NOT? WITHIN ref      # RefInPlaceRef
             | ref NOT? WITHIN PLACE    # RefInPlaceName
             | ref NOT? WITHIN '(' PLACE (',' PLACE)* ')'  # RefInPlaceList
             | ref NOT? WITHIN invoke   # RefInFunc
             ;

ref : REF
    | '@' REF REF
    | '@' PLACE REF
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
PER     : 'PER' | 'per' | 'MATCH' | 'match' ;
WITHIN  : 'WITHIN' | 'within' ;
SWAP    : 'SWAP' | 'swap' ;

ITEM    : [A-Z][a-z][a-zA-Z_0-9]+ ;
SETTING : [a-z][a-zA-Z_0-9]+ ;
REF     : '^' [a-z_][a-zA-Z_0-9.]+ ;
FUNC    : '$' [A-Za-z_][A-Za-z_0-9]+ ;
PLACE   : '`' [A-Z][A-Za-z_0-9'> ]+ '`';
LIT     : '\'' (~'\'' | '\\\'' )* '\'';
CONST   : [A-Z][A-Z_0-9]+ ;
INT     : '-'? [0-9]+ ;
FLOAT   : '-'? [0-9]+[.][0-9]+ ;
BINOP   : [-+*/] ;
WS      : [ \t\r\n]+ -> skip ;
