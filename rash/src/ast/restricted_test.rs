#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_restricted_ast_validation() {
        let ast = RestrictedAst {
            functions: vec![
                Function {
                    name: "main".to_string(),
                    params: vec![],
                    return_type: Type::Str,
                    body: vec![
                        Stmt::Let {
                            name: "x".to_string(),
                            value: Expr::Literal(Literal::U32(42)),
                        }
                    ],
                }
            ],
            entry_point: "main".to_string(),
        };

        assert!(ast.validate().is_ok());
    }

    #[test]
    fn test_missing_entry_point() {
        let ast = RestrictedAst {
            functions: vec![
                Function {
                    name: "helper".to_string(),
                    params: vec![],
                    return_type: Type::Str,
                    body: vec![],
                }
            ],
            entry_point: "main".to_string(),
        };

        assert!(ast.validate().is_err());
        assert!(ast.validate().unwrap_err().contains("Entry point function 'main' not found"));
    }

    #[test]
    fn test_function_validation() {
        let func = Function {
            name: "test".to_string(),
            params: vec![],
            return_type: Type::Str,
            body: vec![],
        };

        assert!(func.validate().is_err());
        assert!(func.validate().unwrap_err().contains("empty body"));
    }

    #[test]
    fn test_recursion_detection() {
        let ast = RestrictedAst {
            functions: vec![
                Function {
                    name: "recursive".to_string(),
                    params: vec![],
                    return_type: Type::Str,
                    body: vec![
                        Stmt::Expr(Expr::FunctionCall {
                            name: "recursive".to_string(),
                            args: vec![],
                        })
                    ],
                },
            ],
            entry_point: "recursive".to_string(),
        };

        assert!(ast.validate().is_err());
        assert!(ast.validate().unwrap_err().contains("Recursion detected"));
    }

    #[test]
    fn test_indirect_recursion_detection() {
        let ast = RestrictedAst {
            functions: vec![
                Function {
                    name: "a".to_string(),
                    params: vec![],
                    return_type: Type::Str,
                    body: vec![
                        Stmt::Expr(Expr::FunctionCall {
                            name: "b".to_string(),
                            args: vec![],
                        })
                    ],
                },
                Function {
                    name: "b".to_string(),
                    params: vec![],
                    return_type: Type::Str,
                    body: vec![
                        Stmt::Expr(Expr::FunctionCall {
                            name: "a".to_string(),
                            args: vec![],
                        })
                    ],
                },
            ],
            entry_point: "a".to_string(),
        };

        assert!(ast.validate().is_err());
        assert!(ast.validate().unwrap_err().contains("Recursion detected"));
    }

    #[test]
    fn test_type_validation() {
        assert!(Type::Bool.is_allowed());
        assert!(Type::U32.is_allowed());
        assert!(Type::Str.is_allowed());
        
        let result_type = Type::Result {
            ok_type: Box::new(Type::Str),
            err_type: Box::new(Type::Str),
        };
        assert!(result_type.is_allowed());

        let option_type = Type::Option {
            inner_type: Box::new(Type::U32),
        };
        assert!(option_type.is_allowed());
    }

    #[test]
    fn test_expression_validation() {
        let valid_expr = Expr::Binary {
            op: BinaryOp::Add,
            left: Box::new(Expr::Literal(Literal::U32(1))),
            right: Box::new(Expr::Literal(Literal::U32(2))),
        };
        assert!(valid_expr.validate().is_ok());

        let function_call = Expr::FunctionCall {
            name: "test".to_string(),
            args: vec![
                Expr::Literal(Literal::Str("hello".to_string())),
                Expr::Variable("x".to_string()),
            ],
        };
        assert!(function_call.validate().is_ok());
    }

    #[test]
    fn test_statement_validation() {
        let let_stmt = Stmt::Let {
            name: "x".to_string(),
            value: Expr::Literal(Literal::U32(42)),
        };
        assert!(let_stmt.validate().is_ok());

        let if_stmt = Stmt::If {
            condition: Expr::Literal(Literal::Bool(true)),
            then_block: vec![
                Stmt::Expr(Expr::Literal(Literal::Str("then".to_string()))),
            ],
            else_block: Some(vec![
                Stmt::Expr(Expr::Literal(Literal::Str("else".to_string()))),
            ]),
        };
        assert!(if_stmt.validate().is_ok());
    }

    #[test]
    fn test_function_call_collection() {
        let func = Function {
            name: "main".to_string(),
            params: vec![],
            return_type: Type::Str,
            body: vec![
                Stmt::Expr(Expr::FunctionCall {
                    name: "helper1".to_string(),
                    args: vec![],
                }),
                Stmt::Let {
                    name: "x".to_string(),
                    value: Expr::FunctionCall {
                        name: "helper2".to_string(),
                        args: vec![],
                    },
                },
            ],
        };

        let mut calls = Vec::new();
        func.collect_function_calls(&mut calls);
        
        assert_eq!(calls.len(), 2);
        assert!(calls.contains(&"helper1".to_string()));
        assert!(calls.contains(&"helper2".to_string()));
    }
}