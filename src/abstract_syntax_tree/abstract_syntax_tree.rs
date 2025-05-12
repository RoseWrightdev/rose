pub mod ast {
    use crate::lexical::{Token, TokenType, LiteralValue};
    use std::fmt::Debug;

    // --- Path Expression ---
    #[derive(Debug, Clone, PartialEq)]
    pub struct PathExpression {

        pub segments: Vec<Token>, 
    }

    // --- AST Node Definitions for Declarations and Statements ---

    pub type Program = Vec<Stmt>;

    #[derive(Debug, Clone, PartialEq)]
    pub struct AnnotationNode {
        pub at_token: Token,
        pub name: PathExpression, 
        pub arguments: Option<Vec<Expression>>,
    }

    #[derive(Debug, Clone, PartialEq)]
    pub struct TypeAnnotation { // This is a simplified version.
                                // Consider replacing with a richer `Type` enum later.
        pub name: PathExpression,
        pub is_optional: bool,
    }
    
    #[derive(Debug, Clone, PartialEq)]
    pub struct Parameter {
        pub annotations: Vec<AnnotationNode>,
        pub name: Token,
        pub colon_token: Option<Token>,
        pub type_ann: Option<TypeAnnotation>, 
    }

    #[derive(Debug, Clone, PartialEq)]
    pub struct FieldDecl {
        pub annotations: Vec<AnnotationNode>,
        pub name: Token,
        pub colon_token: Token,
        pub type_ann: TypeAnnotation, 
    }

    #[derive(Debug, Clone, PartialEq)]
    pub enum VariantFields {
        Tuple(Token /* LeftParen */, Vec<TypeAnnotation>, Token /* RightParen */), 
        Struct(Token /* LeftBrace */, Vec<FieldDecl>, Token /* RightBrace */),
    }
    
    #[derive(Debug, Clone, PartialEq)]
    pub struct VariantDecl {
        pub annotations: Vec<AnnotationNode>,
        pub name: Token, // Variant name is a single Identifier Token
        pub fields: Option<VariantFields>,
    }

    #[derive(Debug, Clone, PartialEq)]
    pub struct ElseIfBranch {
        pub else_token: Token,
        pub if_token: Token,
        pub condition: Box<Expression>,
        pub then_branch: Box<Stmt>,
    }

    #[derive(Debug, Clone, PartialEq)]
    pub struct ElseBranch {
        pub else_token: Token,
        pub branch: Box<Stmt>,
    }

    #[derive(Debug, Clone, PartialEq)]
    pub enum Stmt {
        ExpressionStmt(Box<Expression>),
        LetDecl { 
            annotations: Vec<AnnotationNode>,
            let_token: Token, 
            name: Token,
            type_ann: Option<TypeAnnotation>, 
            initializer: Option<Box<Expression>>,
        },
        Block {
            left_brace: Token,
            statements: Vec<Stmt>,
            right_brace: Token,
        },
        IfStmt {
            if_token: Token,
            condition: Box<Expression>,
            then_branch: Box<Stmt>,
            else_if_branches: Vec<ElseIfBranch>,
            else_branch: Option<ElseBranch>,
        },
        WhileStmt {
            while_token: Token,
            condition: Box<Expression>,
            body: Box<Stmt>,
        },
        ForStmt {
            for_token: Token,
            variable: Token,
            in_token: Token,
            iterable: Box<Expression>,
            body: Box<Stmt>,
        },
        FunctionDecl {
            annotations: Vec<AnnotationNode>,
            async_token: Option<Token>,
            fn_token: Token,
            name: Token,
            params: Vec<Parameter>,
            return_type_arrow: Option<Token>,
            // TODO: Consider changing to Option<FunctionReturnTypes { types: Vec<Type> }> for richer types
            return_type: Option<TypeAnnotation>, 
            body_left_brace: Token,
            body: Vec<Stmt>,
            body_right_brace: Token,
        },
        ReturnStmt {
            return_token: Token,
            value: Option<Box<Expression>>, 
        },
        StructDecl {
            annotations: Vec<AnnotationNode>,
            struct_token: Token,
            name: Token, // Struct name is a single Identifier Token
            left_brace: Token,
            fields: Vec<FieldDecl>,
            right_brace: Token,
        },
        EnumDecl {
            annotations: Vec<AnnotationNode>,
            enum_token: Token,
            name: Token, // Enum name is a single Identifier Token
            left_brace: Token,
            variants: Vec<VariantDecl>,
            right_brace: Token,
        },
    }

    // --- Expression Hierarchy ---
    #[derive(Debug, Clone, PartialEq)]
    pub enum Expression {
        Assign { 
            target: Box<Postfix>, // The L-value, e.g., identifier or property access
            equals_token: Token,
            value: Box<Expression>, // The R-value
        },
        Conditional(Box<ConditionalExpressionContent>), // For ternary ?: (if supported)
        LogicalOr(Box<LogicalOr>), // Entry point for arithmetic/logical precedence chain
        // Other direct expression types from your richer language example could be added here,
        // e.g., Await(Box<Expression>), if they don't fit the Postfix/Primary chain.
    }
    
    #[derive(Debug, Clone, PartialEq)]
    pub struct ConditionalExpressionContent {
        pub condition_expr: Box<LogicalOr>, // Condition part
        pub question_token: Token,
        pub then_expr: Box<Expression>, // Result if true
        pub colon_token: Token,
        pub else_expr: Box<Expression>, // Result if false
    }

    #[derive(Debug, Clone, PartialEq)]
    pub enum LogicalOr {
        Binary { left: Box<LogicalAnd>, operator: Token, right: Box<LogicalAnd> },
        Next(Box<LogicalAnd>),
    }

    #[derive(Debug, Clone, PartialEq)]
    pub enum LogicalAnd {
        Binary { left: Box<Equality>, operator: Token, right: Box<Equality> },
        Next(Box<Equality>),
    }

    #[derive(Debug, Clone, PartialEq)]
    pub enum Equality {
        Binary { left: Box<Comparison>, operator: Token, right: Box<Comparison> },
        Next(Box<Comparison>),
    }

    #[derive(Debug, Clone, PartialEq)]
    pub enum Comparison {
        Binary { left: Box<Term>, operator: Token, right: Box<Term> },
        Next(Box<Term>),
    }

    #[derive(Debug, Clone, PartialEq)]
    pub enum Term {
        Binary { left: Box<Factor>, operator: Token, right: Box<Factor> },
        Next(Box<Factor>),
    }

    #[derive(Debug, Clone, PartialEq)]
    pub enum Factor {
        Binary { left: Box<Unary>, operator: Token, right: Box<Unary> },
        Next(Box<Unary>),
    }

    #[derive(Debug, Clone, PartialEq)]
    pub enum Unary {
        Prefixed { operator: Token, operand: Box<Unary> }, // e.g., !found, -5
        Next(Box<Postfix>), // Pass to Postfix expressions (calls, property access)
    }

    #[derive(Debug, Clone, PartialEq)]
    pub enum Postfix {
        FunctionCall {
            callee: Box<Postfix>, // Expression being called
            left_paren: Token,
            arguments: Vec<Expression>, // Arguments are full expressions
            right_paren: Token,
        },
        PropertyGet {
            object: Box<Postfix>, // Expression whose property is accessed
            dot_token: Token,
            optional_chain_token: Option<Token>, // For '?.'
            name: Token, // Identifier for the property name
        },
        Item(Box<Primary>), // Base of a postfix chain is a Primary expression
    }

    #[derive(Debug, Clone, PartialEq)]
    pub struct MatchArm {
        pub pattern: MatchPattern,
        pub arrow_token: Token, // Should be TokenType::FatArrow
        pub body: Box<Expression>, // Body of a match arm is an expression
        pub comma_token: Option<Token>, // For trailing comma or separating arms
    }

    #[derive(Debug, Clone, PartialEq)]
    pub enum MatchPattern {
        Literal(LiteralValue), 
        Identifier(Token), // Variable binding or simple enum variant
        StructPattern {
            name: PathExpression, // UPDATED: Was Token, now PathExpression for struct name in pattern
            left_brace: Token,
            fields: Vec<StructPatternField>,
            right_brace: Token,
        },
        EnumVariantPattern {
            path: PathExpression, // UPDATED: Was Vec<Token>, now PathExpression for Enum::Variant
            fields: Option<EnumVariantPatternFields>,
        },
        // Wildcard(Token), // If you add '_' as a token
        // OrPattern(Box<MatchPattern>, Token /* Or or | token */, Box<MatchPattern>),
    }

    #[derive(Debug, Clone, PartialEq)]
    pub struct StructPatternField {
        pub field_name: Token, // Identifier
        pub colon_and_pattern: Option<(Token /* Colon */, Box<MatchPattern>)>,
    }
    
    #[derive(Debug, Clone, PartialEq)]
    pub enum EnumVariantPatternFields {
        Tuple(Token /* LeftParen */, Vec<MatchPattern>, Token /* RightParen */),
        Struct(Token /* LeftBrace */, Vec<StructPatternField>, Token /* RightBrace */),
    }

    #[derive(Debug, Clone, PartialEq)]
    pub struct StructLiteralField {
        pub name: Token, /* Identifier (field name) */
        pub colon_token: Token,
        pub value: Box<Expression>,
    }

    #[derive(Debug, Clone, PartialEq)]
    pub enum Primary {
        Literal(LiteralValue), 
        Identifier(Token), // Variable usage
        Grouping {
            left_paren: Token,
            expression: Box<Expression>,
            right_paren: Token,
        },
        StructLiteral {
            name: Token, // Struct name for instantiation is a single Identifier Token
            left_brace: Token,
            fields: Vec<StructLiteralField>,
            right_brace: Token,
        },
        EnumPath(PathExpression), // UPDATED: Was Vec<Token>, now PathExpression for MyEnum::Variant as a value
        MatchExpr {
            match_token: Token,
            expression: Box<Expression>, // Expression being matched
            left_brace: Token,
            arms: Vec<MatchArm>,
            right_brace: Token,
        },
        Lambda { // Anonymous function
            fn_token: Token, 
            params: Vec<Parameter>,
            return_type_arrow: Option<Token>,
            return_type: Option<TypeAnnotation>, 
            body_expr: Box<Expression>,
        },
        // Array/List literals ([elem1, elem2]) could be here if tokens are available.
        // Await expression could be a primary expression if it's `await primary`
        // Or it could be a unary operator if `await` has higher precedence.
        // For JS-like `await`, it's often treated as a unary expression:
        // Expression::Await(Box<Expression>)
    }

    pub fn print_example_expression_construction() {
        let token_one_lhs = Token { 
            token_type: TokenType::Number, 
            lexeme: "1.0".to_string(), 
            literal: Some(LiteralValue::Float(1.0)),
            line: 1 
        };
        let token_one_rhs = Token { 
            token_type: TokenType::Number, 
            lexeme: "1.0".to_string(), 
            literal: Some(LiteralValue::Float(1.0)),
            line: 1 
        };
        let token_plus = Token { 
            token_type: TokenType::Plus, 
            lexeme: "+".to_string(), 
            literal: None,
            line: 1 
        };

        let primary_lhs = Primary::Literal(LiteralValue::Float(1.0));
        let postfix_lhs = Postfix::Item(Box::new(primary_lhs));
        let unary_lhs = Unary::Next(Box::new(postfix_lhs));
        let factor_lhs = Factor::Next(Box::new(unary_lhs));

        let primary_rhs = Primary::Literal(LiteralValue::Float(1.0));
        let postfix_rhs = Postfix::Item(Box::new(primary_rhs));
        let unary_rhs = Unary::Next(Box::new(postfix_rhs));
        let factor_rhs = Factor::Next(Box::new(unary_rhs));
        
        let term_expr = Term::Binary {
            left: Box::new(factor_lhs),
            operator: token_plus.clone(), 
            right: Box::new(factor_rhs),
        };

        let expr_node = Expression::LogicalOr(Box::new(
            LogicalOr::Next(Box::new(
                LogicalAnd::Next(Box::new(
                    Equality::Next(Box::new(
                        Comparison::Next(Box::new(term_expr))
                    ))
                ))
            ))
        ));

        println!("Constructed AST for '1.0 + 1.0':\n{:#?}", expr_node);

        let token_my_var = Token { 
            token_type: TokenType::Identifier, 
            lexeme: "my_var".to_string(), 
            literal: None, 
            line: 1
        };
        let token_equals = Token { 
            token_type: TokenType::Equal, 
            lexeme: "=".to_string(), 
            literal: None, 
            line: 1
        };
        
        let rhs_expr_1_plus_2 = Expression::LogicalOr(Box::new(
            LogicalOr::Next(Box::new(
                LogicalAnd::Next(Box::new(
                    Equality::Next(Box::new(
                        Comparison::Next(Box::new(
                            Term::Binary {
                                left: Box::new(Factor::Next(Box::new(Unary::Next(Box::new(Postfix::Item(Box::new(Primary::Literal(LiteralValue::Integer(1))))))))),
                                operator: token_plus.clone(), 
                                right: Box::new(Factor::Next(Box::new(Unary::Next(Box::new(Postfix::Item(Box::new(Primary::Literal(LiteralValue::Integer(2))))))))),
                            }
                        ))
                    ))
                ))
            ))
        ));

        let assignment_expr = Expression::Assign {
            target: Box::new(Postfix::Item(Box::new(Primary::Identifier(token_my_var.clone())))), 
            equals_token: token_equals.clone(), 
            value: Box::new(rhs_expr_1_plus_2),
        };
        println!("\nConstructed AST for 'my_var = 1 + 2':\n{:#?}", assignment_expr);
    }
}
