use crate::lexical::{Token, TokenType, LiteralValue};
use crate::abstract_syntax_tree::ast;

// --- Parse Error Definition ---

/// Represents an error encountered during parsing.
#[derive(Debug, Clone)]
pub struct ParseError {
    /// The token that caused or is nearest to the error, if available.
    pub token: Option<Token>,
    /// A descriptive message for the error.
    pub message: String,
    /// The line number where the error occurred.
    pub line: usize,
}

impl ParseError {
    /// Creates a new `ParseError`.
    fn new(token: Option<&Token>, message: String, line: usize) -> Self {
        ParseError {
            token: token.cloned(),
            message,
            line,
        }
    }
}

// --- Parser Struct ---

/// Parses a sequence of `Token`s into an Abstract Syntax Tree (AST).
///
/// The parser implements a recursive descent strategy with precedence climbing
/// for expressions. It aims to produce a `ast::Program` or a list of `ParseError`s.
pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    /// Creates a new `Parser` instance with the given tokens.
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser {
            tokens,
            current: 0,
        }
    }

    /// Parses the entire program.
    ///
    /// This is the main entry point for the parser. It attempts to parse
    /// a series of declarations and statements until the end of the token stream.
    ///
    /// # Returns
    /// - `Ok(ast::Program)` if parsing is successful.
    /// - `Err(Vec<ParseError>)` if one or more parsing errors occur.
    pub fn parse_program(&mut self) -> Result<ast::Program, Vec<ParseError>> {
        let mut statements = Vec::new();
        let mut errors = Vec::new();

        while !self.is_at_end() {
            // Stop if we hit the final EOF token.
            // The scanner should place only one EOF token at the very end.
            if self.peek().token_type == TokenType::EndOfFile && self.current == self.tokens.len() -1 {
                break;
            }

            match self.declaration() {
                Ok(stmt) => statements.push(stmt),
                Err(e) => {
                    errors.push(e);
                    self.synchronize(); // Attempt to recover and parse next item
                }
            }
        }

        if errors.is_empty() {
            Ok(statements)
        } else {
            Err(errors)
        }
    }

    // --- Helper Methods: Token Navigation & Consumption ---

    /// Returns a reference to the current token without consuming it.
    fn peek(&self) -> &Token {
        // Assumes `tokens` always contains at least an EOF token.
        &self.tokens[self.current]
    }

    /// Returns the token that was just consumed.
    fn previous(&self) -> Token {
        // Assumes `current > 0`.
        self.tokens[self.current - 1].clone()
    }

    /// Checks if the parser has reached the end of the token stream (i.e., current token is EOF).
    fn is_at_end(&self) -> bool {
        self.peek().token_type == TokenType::EndOfFile
    }

    /// Consumes the current token and returns it, advancing the parser.
    fn advance(&mut self) -> Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous() // `previous()` clones and returns the consumed token.
    }

    /// Checks if the current token is of the specified `TokenType` without consuming it.
    fn check(&self, token_type: TokenType) -> bool {
        if self.is_at_end() {
            return false;
        }
        self.peek().token_type == token_type
    }

    /// Checks if the token *after* the current one is of the specified `TokenType`.
    fn check_next(&self, token_type: TokenType) -> bool {
        if self.is_at_end() || self.current + 1 >= self.tokens.len() {
            return false;
        }
        self.tokens[self.current + 1].token_type == token_type
    }

    /// If the current token matches any of the given `types`, consumes it and returns `true`.
    /// Otherwise, returns `false` without consuming.
    fn match_token(&mut self, types: &[TokenType]) -> bool {
        for token_type in types {
            if self.check(*token_type) {
                self.advance(); // Consumes the token
                return true;
            }
        }
        false
    }

    /// Consumes the current token if it matches the expected `token_type`.
    ///
    /// # Returns
    /// - `Ok(Token)` containing the consumed token if it matches.
    /// - `Err(ParseError)` with the given `message` if it does not match.
    fn consume(&mut self, token_type: TokenType, message: &str) -> Result<Token, ParseError> {
        if self.check(token_type) {
            Ok(self.advance()) 
        } else {
            let peeked_token = self.peek().clone();
            Err(ParseError::new(
                Some(&peeked_token),
                message.to_string(),
                peeked_token.line,
            ))
        }
    }

    /// Attempts to recover from a parse error by advancing tokens until a likely
    /// statement boundary or recognizable declaration keyword is found.
    fn synchronize(&mut self) {
        self.advance(); // Consume the token that caused the error.

        while !self.is_at_end() {
            // A semicolon often ends a statement.
            if self.previous().token_type == TokenType::Semicolon {
                return;
            }

            // Keywords that usually start a new statement or declaration.
            match self.peek().token_type {
                TokenType::Fn | TokenType::Let | TokenType::Struct | TokenType::Enum |
                TokenType::If | TokenType::While | TokenType::For | TokenType::Return |
                TokenType::Match | TokenType::LeftBrace | TokenType::RightBrace => return,
                _ => {} // Continue advancing.
            }
            self.advance();
        }
    }

    // --- Parsing Path Expressions ---

    /// Parses a path expression, e.g., `identifier` or `identifier::identifier` or `identifier.identifier`.
    /// The AST's `PathExpression` currently only stores segments.
    fn parse_path_expression(&mut self) -> Result<ast::PathExpression, ParseError> {
        let mut segments = Vec::new();
        segments.push(self.consume(TokenType::Identifier, "Expect identifier for path segment.")?);

        while self.check(TokenType::DoubleColon) || self.check(TokenType::Dot) {
            // Consume the separator (:: or .)
            if self.check(TokenType::DoubleColon) {
                self.advance(); 
            } else if self.check(TokenType::Dot) {
                self.advance(); 
            }
            segments.push(self.consume(TokenType::Identifier, "Expect identifier after '::' or '.'.")?);
        }
        Ok(ast::PathExpression { segments })
    }

    // --- Parsing Annotations ---

    /// Parses a list of zero or more annotations.
    fn parse_annotations_list(&mut self) -> Result<Vec<ast::AnnotationNode>, ParseError> {
        let mut annotations = Vec::new();
        while self.check(TokenType::At) {
            annotations.push(self.parse_annotation()?);
        }
        Ok(annotations)
    }

    /// Parses a single annotation, e.g., `@name`, `@name(arg1, arg2)`, or `@name = value`.
    fn parse_annotation(&mut self) -> Result<ast::AnnotationNode, ParseError> {
        let at_token = self.consume(TokenType::At, "Expect '@' for annotation.")?;
        let name_path = self.parse_path_expression()?; // Annotation name can be a path.
        
        let mut arguments: Option<Vec<ast::Expression>> = None;
        if self.match_token(&[TokenType::LeftParen]) { // Arguments like @foo(bar, baz)
            let mut args_vec = Vec::new();
            if !self.check(TokenType::RightParen) {
                 args_vec.push(self.expression()?);
                 while self.match_token(&[TokenType::Comma]) {
                    args_vec.push(self.expression()?);
                 }
            }
            self.consume(TokenType::RightParen, "Expect ')' after annotation arguments.")?;
            arguments = Some(args_vec);
        } else if self.match_token(&[TokenType::Equal]) { // Argument like @foo = bar
            let value_expr = self.expression()?;
            arguments = Some(vec![value_expr]); // Store as a single expression argument.
        }

        Ok(ast::AnnotationNode { at_token, name: name_path, arguments })
    }

    // --- Parsing Types ---

    /// Parses a type annotation.
    ///
    /// This is currently simplified and parses a path (for the type name)
    /// followed by an optional `?` for optional types.
    /// TODO: Expand to parse richer types (generics, modifiers, etc.) into `ast::Type`.
    fn parse_type_annotation(&mut self) -> Result<ast::TypeAnnotation, ParseError> {
        let name_path = self.parse_path_expression()?;
        let is_optional = self.match_token(&[TokenType::Question]);
        Ok(ast::TypeAnnotation { name: name_path, is_optional })
    }

    // --- Parsing Rules (Statements & Declarations) ---

    /// Parses a declaration or a statement.
    /// Declarations typically start with keywords like `fn`, `let`, `struct`, `enum`.
    fn declaration(&mut self) -> Result<ast::Stmt, ParseError> {
        let annotations = self.parse_annotations_list()?;
        let mut async_tok: Option<Token> = None;

        if self.check(TokenType::Async) { 
            async_tok = Some(self.advance()); // Consume 'async'
            if !self.check(TokenType::Fn) { // 'async' must be followed by 'fn'
                return Err(ParseError::new(async_tok.as_ref(), "Expect 'fn' after 'async'.".to_string(), async_tok.as_ref().unwrap().line));
            }
        }

        if self.match_token(&[TokenType::Fn]) {
            self.fn_declaration(annotations, async_tok) 
        } else if self.match_token(&[TokenType::Let]) {
            if async_tok.is_some() { // 'async' is not applicable to 'let'
                 return Err(ParseError::new(async_tok.as_ref(), "'async' not applicable to 'let' declaration.".to_string(), async_tok.as_ref().unwrap().line));
            }
            self.let_declaration(annotations)
        } else if self.match_token(&[TokenType::Struct]) {
            if async_tok.is_some() { // 'async' is not applicable to 'struct'
                 return Err(ParseError::new(async_tok.as_ref(), "'async' not applicable to 'struct' declaration.".to_string(), async_tok.as_ref().unwrap().line));
            }
            self.struct_declaration(annotations)
        } else if self.match_token(&[TokenType::Enum]) {
            if async_tok.is_some() { // 'async' is not applicable to 'enum'
                 return Err(ParseError::new(async_tok.as_ref(), "'async' not applicable to 'enum' declaration.".to_string(), async_tok.as_ref().unwrap().line));
            }
            self.enum_declaration(annotations)
        } else {
            // If annotations were parsed but no declaration keyword followed (and not an async-related error).
            if !annotations.is_empty() && async_tok.is_none() { 
                 let last_ann_token = annotations.last().map(|a| &a.at_token);
                 let line = last_ann_token.map_or_else(|| self.peek().line, |t| t.line);
                 return Err(ParseError::new(last_ann_token, "Expected declaration after annotations.".to_string(), line));
            }
            // If 'async' was parsed but not followed by 'fn' (already handled if 'fn' was checked).
            if async_tok.is_some() { 
                 return Err(ParseError::new(async_tok.as_ref(), "Expect 'fn' after 'async' (declaration not found).".to_string(), async_tok.as_ref().unwrap().line));
            }
            // Fallback to parsing a general statement.
            self.statement()
        }
    }

    /// Parses a function declaration: `annotations? (async)? fn IDENTIFIER (PARAMETERS) (-> TYPE_ANNOTATION)? { BODY }`
    fn fn_declaration(&mut self, annotations: Vec<ast::AnnotationNode>, async_token: Option<Token>) -> Result<ast::Stmt, ParseError> {
        let fn_token = self.previous(); // 'fn' token was consumed by `match_token` in `declaration`
        let name = self.consume(TokenType::Identifier, "Expect function name.")?;

        self.consume(TokenType::LeftParen, "Expect '(' after function name.")?;
        let params = self.parse_parameters()?;
        self.consume(TokenType::RightParen, "Expect ')' after parameters.")?;

        let mut return_type_arrow: Option<Token> = None;
        let mut return_type: Option<ast::TypeAnnotation> = None; 
        if self.match_token(&[TokenType::Arrow]) {
            return_type_arrow = Some(self.previous());
            // TODO: Parse a list of types for multiple return values if AST supports it.
            return_type = Some(self.parse_type_annotation()?);
        }

        let body_left_brace = self.consume(TokenType::LeftBrace, "Expect '{' before function body.")?;
        let body_statements = self.block_statement_list()?;
        let body_right_brace = self.consume(TokenType::RightBrace, "Expect '}' after function body.")?;
        
        Ok(ast::Stmt::FunctionDecl {
            annotations,
            async_token, 
            fn_token,
            name,
            params,
            return_type_arrow,
            return_type, // AST `FunctionDecl.return_type` is `Option<TypeAnnotation>`
            body_left_brace,
            body: body_statements,
            body_right_brace,
        })
    }

    /// Parses a list of parameters for a function: `( (IDENTIFIER : TYPE_ANNOTATION?),* )`
    fn parse_parameters(&mut self) -> Result<Vec<ast::Parameter>, ParseError> {
        let mut parameters = Vec::new();
        if !self.check(TokenType::RightParen) { // If there are parameters
            loop {
                let param_annotations = self.parse_annotations_list()?;
                let name = self.consume(TokenType::Identifier, "Expect parameter name.")?;
                let mut colon_token: Option<Token> = None;
                let mut type_ann: Option<ast::TypeAnnotation> = None;
                if self.match_token(&[TokenType::Colon]) {
                    colon_token = Some(self.previous());
                    type_ann = Some(self.parse_type_annotation()?);
                }
                parameters.push(ast::Parameter { annotations: param_annotations, name, colon_token, type_ann });

                if !self.match_token(&[TokenType::Comma]) { // If no comma, this must be the last parameter
                    break;
                }
                 if self.check(TokenType::RightParen) { // Allow trailing comma before ')'
                    break;
                }
            }
        }
        Ok(parameters)
    }

    /// Parses a struct declaration: `annotations? struct IDENTIFIER { FIELDS }`
    fn struct_declaration(&mut self, annotations: Vec<ast::AnnotationNode>) -> Result<ast::Stmt, ParseError> {
        let struct_token = self.previous(); // 'struct' token
        let name = self.consume(TokenType::Identifier, "Expect struct name.")?;
        let left_brace = self.consume(TokenType::LeftBrace, "Expect '{' before struct fields.")?;
        
        let mut fields = Vec::new();
        while !self.check(TokenType::RightBrace) && !self.is_at_end() {
            let field_annotations = self.parse_annotations_list()?;
            let field_name = self.consume(TokenType::Identifier, "Expect field name.")?;
            let colon_token = self.consume(TokenType::Colon, "Expect ':' after field name.")?;
            let type_ann = self.parse_type_annotation()?;
            fields.push(ast::FieldDecl {
                annotations: field_annotations,
                name: field_name,
                colon_token,
                type_ann,
            });
            if !self.match_token(&[TokenType::Comma]) {
                if !self.check(TokenType::RightBrace) { // If no comma, must be end of fields or error
                    return Err(ParseError::new(Some(self.peek()), "Expect ',' or '}' after field declaration.".to_string(), self.peek().line));
                }
                break; 
            }
            if self.check(TokenType::RightBrace) { // Allow trailing comma
                break;
            }
        }
        let right_brace = self.consume(TokenType::RightBrace, "Expect '}' after struct fields.")?;

        Ok(ast::Stmt::StructDecl {
            annotations,
            struct_token,
            name,
            left_brace,
            fields,
            right_brace,
        })
    }

    /// Parses an enum declaration: `annotations? enum IDENTIFIER { VARIANTS }`
    fn enum_declaration(&mut self, annotations: Vec<ast::AnnotationNode>) -> Result<ast::Stmt, ParseError> {
        let enum_token = self.previous(); // 'enum' token
        let name = self.consume(TokenType::Identifier, "Expect enum name.")?;
        let left_brace = self.consume(TokenType::LeftBrace, "Expect '{' before enum variants.")?;

        let mut variants = Vec::new();
        while !self.check(TokenType::RightBrace) && !self.is_at_end() {
            let variant_annotations = self.parse_annotations_list()?;
            let variant_name = self.consume(TokenType::Identifier, "Expect variant name.")?;
            
            let mut variant_fields_opt: Option<ast::VariantFields> = None; 
            if self.match_token(&[TokenType::LeftParen]) { // Tuple-like variant: Variant(Type1, Type2)
                let left_paren_token = self.previous();
                let mut types = Vec::new();
                if !self.check(TokenType::RightParen) {
                    loop {
                        types.push(self.parse_type_annotation()?);
                        if !self.match_token(&[TokenType::Comma]) {
                            break;
                        }
                        if self.check(TokenType::RightParen) { // Allow trailing comma
                           break;
                        }
                    }
                }
                let right_paren_token = self.consume(TokenType::RightParen, "Expect ')' after tuple variant types.")?;
                variant_fields_opt = Some(ast::VariantFields::Tuple(left_paren_token, types, right_paren_token));

            } else if self.match_token(&[TokenType::LeftBrace]) { // Struct-like variant: Variant { field1: Type1 }
                let left_brace_token = self.previous();
                let mut field_decls = Vec::new();
                 while !self.check(TokenType::RightBrace) && !self.is_at_end() {
                    let field_ann = self.parse_annotations_list()?;
                    let f_name = self.consume(TokenType::Identifier, "Expect field name in struct variant.")?;
                    let f_colon = self.consume(TokenType::Colon, "Expect ':' after field name.")?;
                    let f_type = self.parse_type_annotation()?;
                    field_decls.push(ast::FieldDecl{ annotations: field_ann, name: f_name, colon_token: f_colon, type_ann: f_type });
                    if !self.match_token(&[TokenType::Comma]) {
                        if !self.check(TokenType::RightBrace) {
                            return Err(ParseError::new(Some(self.peek()), "Expect ',' or '}' after struct variant field.".to_string(), self.peek().line));
                        }
                        break;
                    }
                    if self.check(TokenType::RightBrace) { // Allow trailing comma
                        break;
                    }
                }
                let right_brace_token = self.consume(TokenType::RightBrace, "Expect '}' after struct variant fields.")?;
                variant_fields_opt = Some(ast::VariantFields::Struct(left_brace_token, field_decls, right_brace_token));
            }

            variants.push(ast::VariantDecl {
                annotations: variant_annotations,
                name: variant_name,
                fields: variant_fields_opt,
            });

            if !self.match_token(&[TokenType::Comma]) { // If no comma, must be end of variants or error
                 if !self.check(TokenType::RightBrace) {
                    return Err(ParseError::new(Some(self.peek()), "Expect ',' or '}' after enum variant.".to_string(), self.peek().line));
                }
                break;
            }
            if self.check(TokenType::RightBrace) { // Allow trailing comma
                break;
            }
        }
        let right_brace = self.consume(TokenType::RightBrace, "Expect '}' after enum variants.")?;

        Ok(ast::Stmt::EnumDecl {
            annotations,
            enum_token,
            name,
            left_brace,
            variants,
            right_brace,
        })
    }

    /// Parses a let declaration: `annotations? let IDENTIFIER (: TYPE_ANNOTATION)? (= EXPRESSION)?;`
    fn let_declaration(&mut self, annotations: Vec<ast::AnnotationNode>) -> Result<ast::Stmt, ParseError> {
        let let_token = self.previous(); // 'let' token
        let name = self.consume(TokenType::Identifier, "Expect variable name.")?;

        let mut type_ann: Option<ast::TypeAnnotation> = None;
        if self.match_token(&[TokenType::Colon]) {
            type_ann = Some(self.parse_type_annotation()?);
        }
        
        let mut initializer: Option<Box<ast::Expression>> = None;
        if self.match_token(&[TokenType::Equal]) {
            initializer = Some(Box::new(self.expression()?));
        }

        self.consume(TokenType::Semicolon, "Expect ';' after variable declaration.")?;
        Ok(ast::Stmt::LetDecl {
            annotations,
            let_token,
            name,
            type_ann,
            initializer,
        })
    }

    /// Parses a statement.
    fn statement(&mut self) -> Result<ast::Stmt, ParseError> {
        if self.check(TokenType::If) {
            self.if_statement()
        } else if self.match_token(&[TokenType::LeftBrace]) { // Block statement
            let left_brace = self.previous();
            let statements = self.block_statement_list()?;
            let right_brace = self.consume(TokenType::RightBrace, "Expect '}' after block.")?;
            Ok(ast::Stmt::Block{ left_brace, statements, right_brace})
        } else if self.match_token(&[TokenType::Return]) {
            self.return_statement()
        } else if self.check(TokenType::While) {
            self.while_statement()
        } else if self.check(TokenType::For) {
            self.for_statement()
        }
        // TODO: Add Match statement if `match` can be a statement
        else {
            self.expression_statement()
        }
    }
    
    /// Parses a list of statements within a block.
    fn block_statement_list(&mut self) -> Result<Vec<ast::Stmt>, ParseError> {
        let mut statements = Vec::new();
        while !self.check(TokenType::RightBrace) && !self.is_at_end() {
            statements.push(self.declaration()?); // Declarations are allowed inside blocks
        }
        Ok(statements)
    }

    /// Parses an if statement: `if EXPRESSION STATEMENT (else if EXPRESSION STATEMENT)* (else STATEMENT)?`
    fn if_statement(&mut self) -> Result<ast::Stmt, ParseError> {
        let if_token = self.consume(TokenType::If, "Expect 'if'.")?;
        // Assuming conditions are not necessarily parenthesized, like Rust.
        let condition = self.expression()?;
        let then_branch = Box::new(self.statement()?); // `statement` can be a block or other statement

        let mut else_if_branches = Vec::new();
        // Loop for 'else if' blocks
        while self.check(TokenType::Else) && self.check_next(TokenType::If) {
            let else_tok = self.consume(TokenType::Else, "Expect 'else'.")?;
            let if_tok = self.consume(TokenType::If, "Expect 'if' after 'else'.")?;
            let else_if_condition = self.expression()?;
            let else_if_then_branch = Box::new(self.statement()?);
            else_if_branches.push(ast::ElseIfBranch {
                else_token: else_tok,
                if_token: if_tok,
                condition: Box::new(else_if_condition),
                then_branch: else_if_then_branch,
            });
        }

        let mut else_branch_opt: Option<ast::ElseBranch> = None;
        if self.match_token(&[TokenType::Else]) { // Check for a final 'else' block
            let else_tok = self.previous();
            let branch = Box::new(self.statement()?);
            else_branch_opt = Some(ast::ElseBranch{else_token: else_tok, branch});
        }

        Ok(ast::Stmt::IfStmt {
            if_token,
            condition: Box::new(condition),
            then_branch,
            else_if_branches,
            else_branch: else_branch_opt,
        })
    }

    /// Parses a while statement: `while EXPRESSION STATEMENT`
    fn while_statement(&mut self) -> Result<ast::Stmt, ParseError> {
        let while_token = self.consume(TokenType::While, "Expect 'while'.")?;
        let condition = self.expression()?;
        let body = Box::new(self.statement()?); 
        Ok(ast::Stmt::WhileStmt { while_token, condition: Box::new(condition), body })
    }

    /// Parses a for statement: `for IDENTIFIER in EXPRESSION STATEMENT`
    fn for_statement(&mut self) -> Result<ast::Stmt, ParseError> {
        let for_token = self.consume(TokenType::For, "Expect 'for'.")?;
        let variable = self.consume(TokenType::Identifier, "Expect variable name in for loop.")?;
        let in_token = self.consume(TokenType::In, "Expect 'in' keyword in for loop.")?;
        let iterable = self.expression()?;
        let body = Box::new(self.statement()?); 
        Ok(ast::Stmt::ForStmt { for_token, variable, in_token, iterable: Box::new(iterable), body })
    }

    /// Parses a return statement: `return (EXPRESSION)? ;`
    fn return_statement(&mut self) -> Result<ast::Stmt, ParseError> {
        let return_token = self.previous(); // 'return' token
        let mut value: Option<Box<ast::Expression>> = None;
        if !self.check(TokenType::Semicolon) { // If there's something after 'return' and before ';'
            // TODO: Handle multiple return values if your AST supports Vec<Expression> for ReturnStmt
            value = Some(Box::new(self.expression()?));
        }
        self.consume(TokenType::Semicolon, "Expect ';' after return value.")?;
        Ok(ast::Stmt::ReturnStmt { return_token, value })
    }

    /// Parses an expression statement: `EXPRESSION ;`
    fn expression_statement(&mut self) -> Result<ast::Stmt, ParseError> {
        let expr = self.expression()?;
        self.consume(TokenType::Semicolon, "Expect ';' after expression.")?;
        Ok(ast::Stmt::ExpressionStmt(Box::new(expr)))
    }

    // --- Parsing Rules (Expressions) ---

    /// Parses an expression (entry point for expression parsing).
    /// Currently delegates to `assignment`.
    fn expression(&mut self) -> Result<ast::Expression, ParseError> {
        self.assignment()
    }
    
    /// Parses an assignment expression: `lvalue = EXPRESSION` (right-associative).
    /// L-value is currently expected to be unwrappable to `ast::Postfix`.
    fn assignment(&mut self) -> Result<ast::Expression, ParseError> {
        let expr = self.logical_or_entry()?; // Parse the left-hand side (potential l-value)

        if self.match_token(&[TokenType::Equal]) { // If it's an assignment
            let equals_token = self.previous();
            let value = self.assignment()?; // Parse the right-hand side (recursive for a = b = c)

            // Attempt to convert the parsed `expr` (which is an ast::Expression)
            // into an ast::Postfix if it represents a valid l-value.
            // This unwrapping is specific to the current AST structure where Expression::Assign
            // expects a Box<Postfix> as its target.
            match expr {
                ast::Expression::LogicalOr(log_or_box) => {
                     match *log_or_box {
                        ast::LogicalOr::Next(log_and_box) => 
                        match *log_and_box {
                            ast::LogicalAnd::Next(eq_box) => 
                            match *eq_box {
                                ast::Equality::Next(comp_box) => 
                                match *comp_box {
                                    ast::Comparison::Next(term_box) => 
                                    match *term_box {
                                        ast::Term::Next(factor_box) => 
                                        match *factor_box {
                                            ast::Factor::Next(unary_box) => 
                                            match *unary_box {
                                                ast::Unary::Next(postfix_box) => {
                                                    // Successfully unwrapped to a Postfix expression.
                                                    return Ok(ast::Expression::Assign {
                                                        target: postfix_box, // postfix_box is Box<ast::Postfix>
                                                        equals_token,
                                                        value: Box::new(value),
                                                    });
                                                }
                                                _ => {} // Unary::Prefixed is not a valid l-value here
                                            },
                                            _ => {} // Factor::Binary is not a valid l-value here
                                        },
                                        _ => {} // Term::Binary is not a valid l-value here
                                    },
                                    _ => {} // Comparison::Binary is not a valid l-value here
                                },
                                _ => {} // Equality::Binary is not a valid l-value here
                            },
                            _ => {} // LogicalAnd::Binary is not a valid l-value here
                        },
                        _ => {} // LogicalOr::Binary is not a valid l-value here
                    }
                }
                // If `ast::Expression` had other variants that could be l-values (e.g., a direct Postfix variant),
                // they would need to be handled here.
                _ => {} 
            }
            
            // If the unwrapping failed, `expr` was not a simple Postfix target.
            return Err(ParseError::new(
                Some(&equals_token),
                "Invalid assignment target. Expected an identifier or property access.".to_string(),
                equals_token.line,
            ));
        }
        Ok(expr) // If no '=', it's just the expression parsed by logical_or_entry
    }

    /// Entry point for the precedence climbing chain for logical OR, returning `ast::Expression`.
    fn logical_or_entry(&mut self) -> Result<ast::Expression, ParseError> {
        let node = self.logical_or_node()?;
        Ok(ast::Expression::LogicalOr(Box::new(node)))
    }
    
    /// Parses logical OR expressions: `logical_and ( "or" logical_and )?`
    fn logical_or_node(&mut self) -> Result<ast::LogicalOr, ParseError> {
        let left_and = self.logical_and_node()?;
        if self.match_token(&[TokenType::Or]) {
            let operator = self.previous();
            let right_and = self.logical_and_node()?;
            Ok(ast::LogicalOr::Binary {
                left: Box::new(left_and), operator, right: Box::new(right_and),
            })
        } else {
            Ok(ast::LogicalOr::Next(Box::new(left_and)))
        }
    }

    /// Parses logical AND expressions: `equality ( "and" equality )?`
    fn logical_and_node(&mut self) -> Result<ast::LogicalAnd, ParseError> {
        let left_eq = self.equality_node()?;
        if self.match_token(&[TokenType::And]) {
            let operator = self.previous();
            let right_eq = self.equality_node()?;
            Ok(ast::LogicalAnd::Binary {
                left: Box::new(left_eq), operator, right: Box::new(right_eq),
            })
        } else {
            Ok(ast::LogicalAnd::Next(Box::new(left_eq)))
        }
    }
    
    /// Parses equality expressions: `comparison ( ( "!=" | "==" ) comparison )?`
    fn equality_node(&mut self) -> Result<ast::Equality, ParseError> {
        let left_comp = self.comparison_node()?;
        if self.match_token(&[TokenType::BangEqual, TokenType::EqualEqual]) {
            let operator = self.previous();
            let right_comp = self.comparison_node()?;
            Ok(ast::Equality::Binary {
                left: Box::new(left_comp), operator, right: Box::new(right_comp),
            })
        } else {
            Ok(ast::Equality::Next(Box::new(left_comp)))
        }
    }

    /// Parses comparison expressions: `term ( ( ">" | ">=" | "<" | "<=" ) term )?`
    fn comparison_node(&mut self) -> Result<ast::Comparison, ParseError> {
        let left_term = self.term_node()?;
        if self.match_token(&[TokenType::Greater, TokenType::GreaterEqual, TokenType::Less, TokenType::LessEqual]) {
            let operator = self.previous();
            let right_term = self.term_node()?;
            Ok(ast::Comparison::Binary {
                left: Box::new(left_term), operator, right: Box::new(right_term),
            })
        } else {
            Ok(ast::Comparison::Next(Box::new(left_term)))
        }
    }

    /// Parses term expressions (addition/subtraction): `factor ( ( "-" | "+" ) factor )?`
    fn term_node(&mut self) -> Result<ast::Term, ParseError> {
        let left_factor = self.factor_node()?;
        if self.match_token(&[TokenType::Minus, TokenType::Plus]) {
            let operator = self.previous();
            let right_factor = self.factor_node()?;
            Ok(ast::Term::Binary {
                left: Box::new(left_factor), operator, right: Box::new(right_factor),
            })
        } else {
            Ok(ast::Term::Next(Box::new(left_factor)))
        }
    }

    /// Parses factor expressions (multiplication/division): `unary ( ( "/" | "*" ) unary )?`
    fn factor_node(&mut self) -> Result<ast::Factor, ParseError> {
        let left_unary = self.unary()?; 
        if self.match_token(&[TokenType::Slash, TokenType::Star]) {
            let operator = self.previous();
            let right_unary = self.unary()?;
            Ok(ast::Factor::Binary {
                left: Box::new(left_unary), operator, right: Box::new(right_unary),
            })
        } else {
            Ok(ast::Factor::Next(Box::new(left_unary)))
        }
    }

    /// Parses unary expressions: `( "!" | "-" ) unary | postfix`
    fn unary(&mut self) -> Result<ast::Unary, ParseError> {
        if self.match_token(&[TokenType::Bang, TokenType::Minus]) {
            let operator = self.previous();
            let operand = self.unary()?; // Recursive call for chained unaries
            Ok(ast::Unary::Prefixed {
                operator,
                operand: Box::new(operand),
            })
        } else {
            // TODO: Parse `await` expressions here if `await` is a unary operator.
            // Example: if self.match_token(&[TokenType::Await]) { /* create ast::Await node */ }
            let postfix_expr = self.postfix()?;
            Ok(ast::Unary::Next(Box::new(postfix_expr)))
        }
    }

    /// Parses postfix expressions: `primary ( "(" arguments? ")" | "." IDENTIFIER ("?." IDENTIFIER)? )*`
    fn postfix(&mut self) -> Result<ast::Postfix, ParseError> {
        let mut expr = ast::Postfix::Item(Box::new(self.primary()?));

        loop {
            if self.match_token(&[TokenType::LeftParen]) { // Function call
                let left_paren = self.previous();
                let arguments = self.arguments()?;
                let right_paren = self.consume(TokenType::RightParen, "Expect ')' after arguments.")?;
                expr = ast::Postfix::FunctionCall {
                    callee: Box::new(expr),
                    left_paren,
                    arguments,
                    right_paren,
                };
            } else if self.match_token(&[TokenType::Dot]) { // Property access
                let dot_token = self.previous();
                // TODO: Implement proper optional chaining (`?.`) parsing.
                // This requires either a dedicated `QuestionDot` token from the lexer
                // or more sophisticated lookahead here.
                let optional_chain_token: Option<Token> = None; 
                let name = self.consume(TokenType::Identifier, "Expect property name after '.'")?;
                expr = ast::Postfix::PropertyGet {
                    object: Box::new(expr),
                    dot_token,
                    optional_chain_token, 
                    name,
                };
            } else {
                break;
            }
        }
        Ok(expr)
    }

    /// Parses a list of arguments for a function call.
    fn arguments(&mut self) -> Result<Vec<ast::Expression>, ParseError> {
        let mut args = Vec::new();
        if !self.check(TokenType::RightParen) { // If there are arguments
            loop {
                args.push(self.expression()?);
                if !self.match_token(&[TokenType::Comma]) {
                    break;
                }
                 if self.check(TokenType::RightParen) { // Allow trailing comma
                    break;
                }
            }
        }
        Ok(args)
    }

    /// Parses primary expressions: literals, identifiers, grouped expressions, struct literals, match expressions.
    fn primary(&mut self) -> Result<ast::Primary, ParseError> {
        let current_token_for_error = self.peek().clone(); 

        if self.match_token(&[TokenType::False]) {
            return Ok(ast::Primary::Literal(LiteralValue::Boolean(false)));
        }
        if self.match_token(&[TokenType::True]) {
            return Ok(ast::Primary::Literal(LiteralValue::Boolean(true)));
        }
        if self.match_token(&[TokenType::Null]) {
            return Ok(ast::Primary::Literal(LiteralValue::Null));
        }

        if self.check(TokenType::Number) || self.check(TokenType::String) {
            let token_with_literal = self.advance();
            match token_with_literal.literal { 
                Some(lv) => return Ok(ast::Primary::Literal(lv)),
                None => return Err(ParseError::new( // Should not happen if scanner is correct
                    Some(&token_with_literal),
                    "Expected literal value not found in token.".to_string(),
                    token_with_literal.line
                )),
            }
        }

        if self.check(TokenType::Identifier) {
            // Attempt to parse as a path first, as identifiers can be part of paths.
            let path_expr = self.parse_path_expression()?;
            
            // If it's a single identifier followed by '{', it's a struct literal.
            if self.check(TokenType::LeftBrace) && path_expr.segments.len() == 1 {
                // `parse_path_expression` consumed the identifier.
                // We need the original identifier token for struct_literal.
                // This requires `parse_path_expression` to return the first token if it's just one segment,
                // or we peek before calling it.
                // Let's adjust: peek for IDENTIFIER, then if LBRACE, it's struct literal.
                // Otherwise, try parsing as path.
                // Simpler: If path_expr has one segment and next is LBRACE, it's struct_literal.
                return self.struct_literal(path_expr.segments.into_iter().next().unwrap());
            } else if path_expr.segments.len() == 1 && !self.is_path_like_context_after_identifier() {
                // Single identifier, not starting a struct literal or a path with `::`
                return Ok(ast::Primary::Identifier(path_expr.segments.into_iter().next().unwrap()));
            } else {
                // Multi-segment path, or single identifier that might be an enum variant without data.
                return Ok(ast::Primary::EnumPath(path_expr));
            }
        }

        if self.match_token(&[TokenType::LeftParen]) { // Grouped expression
            let left_paren = self.previous();
            let expr = self.expression()?;
            let right_paren = self.consume(TokenType::RightParen, "Expect ')' after expression.")?;
            return Ok(ast::Primary::Grouping {
                left_paren,
                expression: Box::new(expr),
                right_paren,
            });
        }
        
        if self.check(TokenType::Match) { // Match expression
            return self.match_expression();
        }
        
        // TODO: Add Lambda expression parsing: `fn (PARAMS) (-> TYPE)? => EXPR` or `{ BODY }`

        Err(ParseError::new(
            Some(&current_token_for_error),
            "Expect expression.".to_string(),
            current_token_for_error.line,
        ))
    }

    /// Helper to check if the context after an identifier suggests it's part of a path.
    fn is_path_like_context_after_identifier(&self) -> bool {
        // This is called *after* an identifier (or path of length 1) has been parsed
        // and we are peeking at the *next* token.
        self.check(TokenType::DoubleColon) 
    }

    /// Parses a struct literal instantiation: `IDENTIFIER { field: value, ... }`
    fn struct_literal(&mut self, name: Token) -> Result<ast::Primary, ParseError> {
        let left_brace = self.consume(TokenType::LeftBrace, "Expect '{' for struct literal.")?;
        let mut fields = Vec::new();

        if !self.check(TokenType::RightBrace) {
            loop {
                let field_name = self.consume(TokenType::Identifier, "Expect field name in struct literal.")?;
                let colon_token = self.consume(TokenType::Colon, "Expect ':' after field name in struct literal.")?;
                let value = self.expression()?;
                fields.push(ast::StructLiteralField { name: field_name, colon_token, value: Box::new(value) });

                if !self.match_token(&[TokenType::Comma]) {
                    break; // No comma, so this must be the last field or an error
                }
                if self.check(TokenType::RightBrace) { // Allow trailing comma
                    break;
                }
            }
        }
        let right_brace = self.consume(TokenType::RightBrace, "Expect '}' after struct literal fields.")?;
        Ok(ast::Primary::StructLiteral { name, left_brace, fields, right_brace })
    }

    /// Parses a match expression: `match EXPRESSION { ARMS }`
    fn match_expression(&mut self) -> Result<ast::Primary, ParseError> {
        let match_token = self.consume(TokenType::Match, "Expect 'match' keyword.")?;
        let expression = self.expression()?; 
        let left_brace = self.consume(TokenType::LeftBrace, "Expect '{' after match expression.")?;
        
        let mut arms = Vec::new();
        while !self.check(TokenType::RightBrace) && !self.is_at_end() {
            let pattern = self.parse_match_pattern()?;
            let arrow_token = self.consume(TokenType::FatArrow, "Expect '=>' after match pattern.")?;
            let body = self.expression()?; 
            
            let mut comma_token_for_ast: Option<Token> = None;
            let mut consumed_comma_this_iteration = false;

            if self.match_token(&[TokenType::Comma]){ 
                comma_token_for_ast = Some(self.previous());
                consumed_comma_this_iteration = true;
            }
            arms.push(ast::MatchArm { pattern, arrow_token, body: Box::new(body), comma_token: comma_token_for_ast });
            
            if self.check(TokenType::RightBrace) { 
                break;
            }
            // If not at '}', and a comma was NOT consumed for this arm, it's an error.
            if !consumed_comma_this_iteration {
                 return Err(ParseError::new(Some(self.peek()), "Expect ',' or '}' after match arm.".to_string(), self.peek().line));
            }
        }
        let right_brace = self.consume(TokenType::RightBrace, "Expect '}' after match arms.")?;

        Ok(ast::Primary::MatchExpr { match_token, expression: Box::new(expression), left_brace, arms, right_brace })
    }

    /// Parses a pattern for a match arm.
    fn parse_match_pattern(&mut self) -> Result<ast::MatchPattern, ParseError> {
        let current_token_for_error = self.peek().clone();

        // Literals
        if self.match_token(&[TokenType::False]) {
            return Ok(ast::MatchPattern::Literal(LiteralValue::Boolean(false)));
        }
        if self.match_token(&[TokenType::True]) {
            return Ok(ast::MatchPattern::Literal(LiteralValue::Boolean(true)));
        }
        if self.match_token(&[TokenType::Null]) {
            return Ok(ast::MatchPattern::Literal(LiteralValue::Null));
        }
        if self.check(TokenType::Number) || self.check(TokenType::String) {
            let token_with_literal = self.advance();
             match token_with_literal.literal {
                Some(lv) => return Ok(ast::MatchPattern::Literal(lv)),
                None => return Err(ParseError::new(Some(&token_with_literal), "Expected literal value for pattern.".to_string(), token_with_literal.line)),
            }
        } 
        
        // Identifier, StructPattern, EnumVariantPattern
        if self.check(TokenType::Identifier) {
            let path_expr = self.parse_path_expression()?; // Path like MyStruct or MyEnum::Variant

            if self.check(TokenType::LeftBrace) { // Struct pattern: Path { ... }
                let left_brace = self.consume(TokenType::LeftBrace, "Expect '{' for struct pattern.")?;
                let mut fields = Vec::new();
                if !self.check(TokenType::RightBrace) {
                    loop {
                        // Allow field shorthand `name` or `name: pattern`
                        let field_name = self.consume(TokenType::Identifier, "Expect field name in struct pattern.")?;
                        let mut colon_and_pattern: Option<(Token, Box<ast::MatchPattern>)> = None;
                        if self.match_token(&[TokenType::Colon]) {
                            let colon_tok = self.previous();
                            let pattern = self.parse_match_pattern()?;
                            colon_and_pattern = Some((colon_tok, Box::new(pattern)));
                        }
                        fields.push(ast::StructPatternField { field_name, colon_and_pattern });

                        if !self.match_token(&[TokenType::Comma]) { break; }
                        if self.check(TokenType::RightBrace) { break; } // Trailing comma
                    }
                }
                let right_brace = self.consume(TokenType::RightBrace, "Expect '}' after struct pattern fields.")?;
                return Ok(ast::MatchPattern::StructPattern { name: path_expr, left_brace, fields, right_brace });
            
            } else if self.check(TokenType::LeftParen) { // Enum variant with tuple fields: Path(...)
                let left_paren = self.consume(TokenType::LeftParen, "Expect '(' for enum variant pattern fields.")?;
                let mut patterns = Vec::new();
                if !self.check(TokenType::RightParen) {
                    loop {
                        patterns.push(self.parse_match_pattern()?);
                        if !self.match_token(&[TokenType::Comma]) { break; }
                        if self.check(TokenType::RightParen) { break; } // Trailing comma
                    }
                }
                let right_paren = self.consume(TokenType::RightParen, "Expect ')' after enum variant pattern fields.")?;
                return Ok(ast::MatchPattern::EnumVariantPattern {
                    path: path_expr,
                    fields: Some(ast::EnumVariantPatternFields::Tuple(left_paren, patterns, right_paren)),
                });
            } else {
                // If it's a single segment path and no ( or { follows, it's an Identifier pattern (binding).
                // If it's a multi-segment path without ( or {, it's an EnumVariantPattern with no fields (unit-like variant).
                if path_expr.segments.len() == 1 {
                    return Ok(ast::MatchPattern::Identifier(path_expr.segments.into_iter().next().unwrap()));
                } else {
                    return Ok(ast::MatchPattern::EnumVariantPattern { path: path_expr, fields: None });
                }
            }
        }
        // TODO: Add Wildcard `_` pattern
        
                Err(ParseError::new(Some(&current_token_for_error), "Unsupported match pattern.".to_string(), current_token_for_error.line))
            }
        }
    