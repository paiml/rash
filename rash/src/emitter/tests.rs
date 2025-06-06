use super::*;
use crate::ir::{Command, EffectSet, ShellIR, ShellValue};
use crate::models::Config;
use proptest::prelude::*;
use rstest::*;

#[test]
fn test_simple_let_emission() {
    let config = Config::default();
    let emitter = PosixEmitter::new(config);

    let ir = ShellIR::Let {
        name: "test_var".to_string(),
        value: ShellValue::String("hello world".to_string()),
        effects: EffectSet::pure(),
    };

    let result = emitter.emit(&ir).unwrap();
    assert!(result.contains("readonly test_var='hello world'"));
    assert!(result.contains("#!/bin/sh"));
    assert!(result.contains("set -euf"));
}

#[test]
fn test_command_emission() {
    let config = Config::default();
    let emitter = PosixEmitter::new(config);

    let cmd = Command {
        program: "echo".to_string(),
        args: vec![ShellValue::String("hello".to_string())],
    };

    let ir = ShellIR::Exec {
        cmd,
        effects: EffectSet::pure(),
    };

    let result = emitter.emit(&ir).unwrap();
    assert!(result.contains("echo hello"));
}

#[test]
fn test_if_statement_emission() {
    let config = Config::default();
    let emitter = PosixEmitter::new(config);

    let ir = ShellIR::If {
        test: ShellValue::Bool(true),
        then_branch: Box::new(ShellIR::Exec {
            cmd: Command {
                program: "echo".to_string(),
                args: vec![ShellValue::String("true branch".to_string())],
            },
            effects: EffectSet::pure(),
        }),
        else_branch: Some(Box::new(ShellIR::Exec {
            cmd: Command {
                program: "echo".to_string(),
                args: vec![ShellValue::String("false branch".to_string())],
            },
            effects: EffectSet::pure(),
        })),
    };

    let result = emitter.emit(&ir).unwrap();
    assert!(result.contains("if true; then"));
    assert!(result.contains("echo 'true branch'"));
    assert!(result.contains("else"));
    assert!(result.contains("echo 'false branch'"));
    assert!(result.contains("fi"));
}

#[test]
fn test_sequence_emission() {
    let config = Config::default();
    let emitter = PosixEmitter::new(config);

    let ir = ShellIR::Sequence(vec![
        ShellIR::Let {
            name: "greeting".to_string(),
            value: ShellValue::String("hello".to_string()),
            effects: EffectSet::pure(),
        },
        ShellIR::Exec {
            cmd: Command {
                program: "echo".to_string(),
                args: vec![ShellValue::Variable("greeting".to_string())],
            },
            effects: EffectSet::pure(),
        },
    ]);

    let result = emitter.emit(&ir).unwrap();
    assert!(result.contains("readonly greeting=hello"));
    assert!(result.contains("echo \"$greeting\""));
}

#[test]
fn test_exit_statement_emission() {
    let config = Config::default();
    let emitter = PosixEmitter::new(config);

    let ir = ShellIR::Exit {
        code: 1,
        message: Some("Error occurred".to_string()),
    };

    let result = emitter.emit(&ir).unwrap();
    assert!(result.contains("echo 'Error occurred' >&2"));
    assert!(result.contains("exit 1"));
}

#[test]
fn test_shell_value_emission() {
    let config = Config::default();
    let emitter = PosixEmitter::new(config);

    // String value
    let string_val = ShellValue::String("hello world".to_string());
    let result = emitter.emit_shell_value(&string_val).unwrap();
    assert_eq!(result, "'hello world'");

    // Boolean values
    let bool_val = ShellValue::Bool(true);
    let result = emitter.emit_shell_value(&bool_val).unwrap();
    assert_eq!(result, "true");

    let bool_val = ShellValue::Bool(false);
    let result = emitter.emit_shell_value(&bool_val).unwrap();
    assert_eq!(result, "false");

    // Variable reference
    let var_val = ShellValue::Variable("test_var".to_string());
    let result = emitter.emit_shell_value(&var_val).unwrap();
    assert_eq!(result, "\"$test_var\"");
}

#[test]
fn test_concatenation_emission() {
    let config = Config::default();
    let emitter = PosixEmitter::new(config);

    let concat_val = ShellValue::Concat(vec![
        ShellValue::String("Hello ".to_string()),
        ShellValue::Variable("name".to_string()),
        ShellValue::String("!".to_string()),
    ]);

    let result = emitter.emit_shell_value(&concat_val).unwrap();
    assert_eq!(result, "\"Hello ${name}!\"");
}

#[test]
fn test_command_substitution_emission() {
    let config = Config::default();
    let emitter = PosixEmitter::new(config);

    let cmd_subst = ShellValue::CommandSubst(Command {
        program: "date".to_string(),
        args: vec![ShellValue::String("+%Y".to_string())],
    });

    let result = emitter.emit_shell_value(&cmd_subst).unwrap();
    assert_eq!(result, "\"$(date '+%Y')\"");
}

#[test]
fn test_noop_emission() {
    let config = Config::default();
    let emitter = PosixEmitter::new(config);

    let ir = ShellIR::Noop;
    let result = emitter.emit(&ir).unwrap();
    assert!(result.contains("# noop"));
}

#[test]
fn test_header_and_footer_structure() {
    let config = Config::default();
    let emitter = PosixEmitter::new(config);

    let ir = ShellIR::Noop;
    let result = emitter.emit(&ir).unwrap();

    // Check header
    assert!(result.starts_with("#!/bin/sh"));
    assert!(result.contains("# Generated by Rash"));
    assert!(result.contains("set -euf"));
    assert!(result.contains("IFS=' \t\n'"));
    assert!(result.contains("export LC_ALL=C"));

    // Check runtime functions
    assert!(result.contains("rash_require()"));
    assert!(result.contains("rash_download_verified()"));

    // Check footer
    assert!(result.contains("main() {"));
    assert!(result.contains("trap 'rm -rf"));
    assert!(result.contains("main \"$@\""));
}

#[test]
fn test_runtime_functions_included() {
    let config = Config::default();
    let emitter = PosixEmitter::new(config);

    let ir = ShellIR::Noop;
    let result = emitter.emit(&ir).unwrap();

    // Verify essential runtime functions are present
    assert!(result.contains("rash_require() {"));
    assert!(result.contains("rash_download_verified() {"));

    // Verify they contain expected functionality
    assert!(result.contains("curl -fsSL"));
    assert!(result.contains("sha256sum"));
    assert!(result.contains("wget"));
}

#[test]
fn test_test_expression_emission() {
    let config = Config::default();
    let emitter = PosixEmitter::new(config);

    // Boolean true
    let result = emitter
        .emit_test_expression(&ShellValue::Bool(true))
        .unwrap();
    assert_eq!(result, "true");

    // Boolean false
    let result = emitter
        .emit_test_expression(&ShellValue::Bool(false))
        .unwrap();
    assert_eq!(result, "false");

    // Variable test
    let result = emitter
        .emit_test_expression(&ShellValue::Variable("var".to_string()))
        .unwrap();
    assert_eq!(result, "test -n \"$var\"");

    // String literal
    let result = emitter
        .emit_test_expression(&ShellValue::String("true".to_string()))
        .unwrap();
    assert_eq!(result, "true");

    let result = emitter
        .emit_test_expression(&ShellValue::String("false".to_string()))
        .unwrap();
    assert_eq!(result, "false");
}

// Test escape module functionality
#[test]
fn test_string_escaping() {
    use super::escape::*;

    // Simple strings don't need escaping
    assert_eq!(escape_shell_string("hello"), "hello");
    assert_eq!(escape_shell_string("simple123"), "simple123");

    // Strings with spaces need quotes
    assert_eq!(escape_shell_string("hello world"), "'hello world'");

    // Empty strings
    assert_eq!(escape_shell_string(""), "''");

    // Strings with single quotes
    assert_eq!(escape_shell_string("don't"), "'don'\"'\"'t'");
}

#[test]
fn test_variable_name_escaping() {
    use super::escape::*;

    // Valid identifiers
    assert_eq!(escape_variable_name("valid_name"), "valid_name");
    assert_eq!(escape_variable_name("_underscore"), "_underscore");
    assert_eq!(escape_variable_name("name123"), "name123");

    // Invalid characters converted to underscores
    assert_eq!(escape_variable_name("invalid-name"), "invalid_name");
    assert_eq!(escape_variable_name("123invalid"), "_23invalid");
    assert_eq!(escape_variable_name("my.var"), "my_var");
}

#[test]
fn test_command_name_escaping() {
    use super::escape::*;

    // Simple commands
    assert_eq!(escape_command_name("ls"), "ls");
    assert_eq!(escape_command_name("/bin/ls"), "/bin/ls");
    assert_eq!(escape_command_name("my-tool"), "my-tool");

    // Commands with spaces need quoting
    assert_eq!(escape_command_name("my command"), "'my command'");
}

// Property-based tests
proptest! {
    #![proptest_config(ProptestConfig::with_cases(1000))]

    #[test]
    fn test_string_escaping_preserves_content(s in ".*") {
        use super::escape::*;

        let escaped = escape_shell_string(&s);

        // Escaped strings should either be the original (if safe) or quoted
        if s.chars().all(|c| c.is_alphanumeric() || "_.-/+=:@".contains(c)) && !s.is_empty() {
            // Safe strings might be unquoted
            assert!(escaped == s || escaped == format!("'{s}'"));
        } else {
            // Unsafe strings should be quoted
            assert!(escaped.starts_with('\'') && escaped.ends_with('\'') || escaped == "''");
        }
    }

    #[test]
    fn test_variable_name_escaping_produces_valid_identifiers(name in "[a-zA-Z_][a-zA-Z0-9_-]*") {
        use super::escape::*;

        let escaped = escape_variable_name(&name);

        // Should start with letter or underscore
        assert!(escaped.chars().next().unwrap().is_alphabetic() || escaped.starts_with('_'));

        // Should only contain valid characters
        assert!(escaped.chars().all(|c| c.is_alphanumeric() || c == '_'));
    }

    /// Property: All shell values should emit valid shell code
    #[test]
    fn prop_shell_values_emit_valid_code(
        s in "[a-zA-Z0-9 _.-]{0,100}",
        b in prop::bool::ANY,
        var_name in "[a-zA-Z_][a-zA-Z0-9_]{0,20}"
    ) {
        let config = Config::default();
        let emitter = PosixEmitter::new(config);

        let test_values = vec![
            ShellValue::String(s),
            ShellValue::Bool(b),
            ShellValue::Variable(var_name),
        ];

        for value in test_values {
            let result = emitter.emit_shell_value(&value);
            prop_assert!(result.is_ok(), "Failed to emit shell value: {:?}", value);

            if let Ok(code) = result {
                // Generated code should not be empty
                prop_assert!(!code.trim().is_empty());

                // Should not contain unescaped dangerous characters
                prop_assert!(!code.contains("$(rm"), "Potential command injection in: {}", code);
                prop_assert!(!code.contains("; rm"), "Potential command injection in: {}", code);
            }
        }
    }

    /// Property: Commands should emit syntactically valid shell
    #[test]
    fn prop_commands_emit_valid_shell(
        cmd_name in "[a-zA-Z][a-zA-Z0-9_-]{0,20}",
        arg_count in 0usize..5usize
    ) {
        let config = Config::default();
        let emitter = PosixEmitter::new(config);

        let args: Vec<ShellValue> = (0..arg_count)
            .map(|i| ShellValue::String(format!("arg{i}")))
            .collect();

        let cmd = Command {
            program: cmd_name.clone(),
            args,
        };

        let ir = ShellIR::Exec {
            cmd,
            effects: EffectSet::pure(),
        };

        let result = emitter.emit(&ir);
        prop_assert!(result.is_ok(), "Failed to emit command: {}", cmd_name);

        if let Ok(shell_code) = result {
            // Should contain the command name
            prop_assert!(shell_code.contains(&cmd_name));

            // Should have balanced quotes
            let single_quotes = shell_code.chars().filter(|&c| c == '\'').count();
            prop_assert!(single_quotes % 2 == 0, "Unbalanced single quotes in: {}", shell_code);

            // Should contain proper shell structure
            prop_assert!(shell_code.contains("#!/bin/sh"));
            prop_assert!(shell_code.contains("set -euf"));
        }
    }

    /// Property: Let statements should create valid variable assignments
    #[test]
    fn prop_let_statements_valid(
        var_name in "[a-zA-Z_][a-zA-Z0-9_]{0,30}",
        value in "[a-zA-Z0-9 _.-]{0,100}"
    ) {
        let config = Config::default();
        let emitter = PosixEmitter::new(config);

        let ir = ShellIR::Let {
            name: var_name.clone(),
            value: ShellValue::String(value),
            effects: EffectSet::pure(),
        };

        let result = emitter.emit(&ir);
        prop_assert!(result.is_ok(), "Failed to emit let statement for: {}", var_name);

        if let Ok(shell_code) = result {
            // Should contain readonly assignment
            prop_assert!(shell_code.contains("readonly"));

            // Variable name should be properly escaped
            let escaped_name = super::escape::escape_variable_name(&var_name);
            prop_assert!(shell_code.contains(&escaped_name));

            // Should be valid shell syntax (basic check)
            prop_assert!(!shell_code.contains("readonly ="), "Invalid assignment syntax");
        }
    }

    /// Property: If statements should have balanced if/fi
    #[test]
    fn prop_if_statements_balanced(condition in prop::bool::ANY) {
        let config = Config::default();
        let emitter = PosixEmitter::new(config);

        let ir = ShellIR::If {
            test: ShellValue::Bool(condition),
            then_branch: Box::new(ShellIR::Noop),
            else_branch: Some(Box::new(ShellIR::Noop)),
        };

        let result = emitter.emit(&ir);
        prop_assert!(result.is_ok(), "Failed to emit if statement");

        if let Ok(shell_code) = result {
            // Focus on the main function content only
            if let Some(main_start) = shell_code.find("main() {") {
                if let Some(main_end) = shell_code[main_start..].find("# Cleanup") {
                    let main_content = &shell_code[main_start..main_start + main_end];
                    let if_count = main_content.matches("if ").count();
                    let fi_count = main_content.matches("fi").count();
                    prop_assert_eq!(if_count, fi_count, "Unbalanced if/fi in main function");

                    // Should contain then and else in main function
                    prop_assert!(main_content.contains("then"));
                    prop_assert!(main_content.contains("else"));
                }
            }
        }
    }

    /// Property: Concatenation should preserve order
    #[test]
    fn prop_concatenation_preserves_order(
        parts in prop::collection::vec("[a-zA-Z0-9]{1,10}", 1..5)
    ) {
        let config = Config::default();
        let emitter = PosixEmitter::new(config);

        let shell_values: Vec<ShellValue> = parts.iter()
            .map(|s| ShellValue::String(s.clone()))
            .collect();

        let concat_value = ShellValue::Concat(shell_values);
        let result = emitter.emit_shell_value(&concat_value);

        prop_assert!(result.is_ok(), "Failed to emit concatenation");

        if let Ok(shell_code) = result {
            // All parts should appear in order
            let mut last_pos = 0;
            for part in &parts {
                if let Some(pos) = shell_code[last_pos..].find(part) {
                    last_pos += pos + part.len();
                } else {
                    prop_assert!(false, "Part '{}' not found in order in: {}", part, shell_code);
                }
            }
        }
    }

    /// Property: Generated shell should be deterministic
    #[test]
    fn prop_emission_deterministic(var_name in "[a-zA-Z_][a-zA-Z0-9_]{0,20}") {
        let config = Config::default();
        let emitter1 = PosixEmitter::new(config.clone());
        let emitter2 = PosixEmitter::new(config);

        let ir = ShellIR::Let {
            name: var_name,
            value: ShellValue::String("test".to_string()),
            effects: EffectSet::pure(),
        };

        let result1 = emitter1.emit(&ir);
        let result2 = emitter2.emit(&ir);

        match (&result1, &result2) {
            (Ok(code1), Ok(code2)) => prop_assert_eq!(code1, code2, "Non-deterministic emission detected"),
            (Err(_), Err(_)) => {}, // Both failing is acceptable
            _ => prop_assert!(false, "Inconsistent success/failure between runs"),
        }
    }

    /// Property: Special characters should be properly escaped
    #[test]
    fn prop_special_chars_escaped(s in r#"['"$`\\;&|()<> \t\n]*"#) {
        use super::escape::*;

        let escaped = escape_shell_string(&s);

        // If the string contains special characters, it should be quoted
        if s.chars().any(|c| "'\"$`\\;&|()<> \t\n".contains(c)) && !s.is_empty() {
            prop_assert!(
                escaped.starts_with('\'') || escaped.starts_with('"'),
                "Special characters not properly escaped in: '{}' -> '{}'", s, escaped
            );
        }
    }

    /// Property: Exit codes should be valid
    #[test]
    fn prop_exit_codes_valid(code in 0i32..256i32) {
        let config = Config::default();
        let emitter = PosixEmitter::new(config);

        let ir = ShellIR::Exit {
            code: code as u8,
            message: Some("test message".to_string()),
        };

        let result = emitter.emit(&ir);
        prop_assert!(result.is_ok(), "Failed to emit exit statement with code: {}", code);

        if let Ok(shell_code) = result {
            let exit_string = format!("exit {code}");
            prop_assert!(shell_code.contains(&exit_string));
        }
    }
}

#[rstest]
#[case(ShellValue::String("test".to_string()), "test")]
#[case(ShellValue::Bool(true), "true")]
#[case(ShellValue::Bool(false), "false")]
#[case(ShellValue::Variable("var".to_string()), "\"$var\"")]
fn test_shell_value_emission_cases(#[case] value: ShellValue, #[case] expected: &str) {
    let config = Config::default();
    let emitter = PosixEmitter::new(config);

    let result = emitter.emit_shell_value(&value).unwrap();
    assert_eq!(result, expected);
}

#[test]
fn test_complex_nested_emission() {
    let config = Config::default();
    let emitter = PosixEmitter::new(config);

    let ir = ShellIR::Sequence(vec![
        ShellIR::Let {
            name: "prefix".to_string(),
            value: ShellValue::String("/usr/local".to_string()),
            effects: EffectSet::pure(),
        },
        ShellIR::If {
            test: ShellValue::Variable("install_mode".to_string()),
            then_branch: Box::new(ShellIR::Sequence(vec![
                ShellIR::Exec {
                    cmd: Command {
                        program: "mkdir".to_string(),
                        args: vec![ShellValue::Variable("prefix".to_string())],
                    },
                    effects: EffectSet::default(),
                },
                ShellIR::Exec {
                    cmd: Command {
                        program: "echo".to_string(),
                        args: vec![ShellValue::Concat(vec![
                            ShellValue::String("Installing to ".to_string()),
                            ShellValue::Variable("prefix".to_string()),
                        ])],
                    },
                    effects: EffectSet::pure(),
                },
            ])),
            else_branch: Some(Box::new(ShellIR::Exit {
                code: 1,
                message: Some("Installation cancelled".to_string()),
            })),
        },
    ]);

    let result = emitter.emit(&ir).unwrap();

    // Verify structure
    assert!(result.contains("readonly prefix=/usr/local"));
    assert!(result.contains("if test -n \"$install_mode\"; then"));
    assert!(result.contains("mkdir \"$prefix\""));
    assert!(result.contains("echo \"Installing to ${prefix}\""));
    assert!(result.contains("else"));
    assert!(result.contains("echo 'Installation cancelled' >&2"));
    assert!(result.contains("exit 1"));
    assert!(result.contains("fi"));
}

#[test]
fn test_emit_public_api() {
    let config = Config::default();

    let ir = ShellIR::Let {
        name: "test".to_string(),
        value: ShellValue::String("value".to_string()),
        effects: EffectSet::pure(),
    };

    // Test the public emit function
    let result = emit(&ir, &config).unwrap();
    assert!(result.contains("readonly test=value"));
}

#[test]
fn test_different_shell_dialects() {
    let mut config = Config::default();

    let ir = ShellIR::Noop;

    // Test POSIX (default)
    config.target = crate::models::ShellDialect::Posix;
    let result = emit(&ir, &config).unwrap();
    assert!(result.contains("#!/bin/sh"));

    // Test Bash (should still emit POSIX for now)
    config.target = crate::models::ShellDialect::Bash;
    let result = emit(&ir, &config).unwrap();
    assert!(result.contains("#!/bin/sh"));
}

#[test]
fn test_indentation_consistency() {
    let config = Config::default();
    let emitter = PosixEmitter::new(config);

    let ir = ShellIR::If {
        test: ShellValue::Bool(true),
        then_branch: Box::new(ShellIR::If {
            test: ShellValue::Bool(false),
            then_branch: Box::new(ShellIR::Let {
                name: "nested".to_string(),
                value: ShellValue::String("deep".to_string()),
                effects: EffectSet::pure(),
            }),
            else_branch: None,
        }),
        else_branch: None,
    };

    let result = emitter.emit(&ir).unwrap();

    // Check that nested structures have proper indentation
    let lines: Vec<&str> = result.lines().collect();
    let main_function_start = lines
        .iter()
        .position(|&line| line.contains("main() {"))
        .unwrap();

    // Lines inside main() should be indented
    for line in &lines[main_function_start + 1..] {
        if line.trim().is_empty() || line.starts_with('#') || line.starts_with('}') {
            continue;
        }
        if line.contains("trap") || line.contains("main \"$@\"") {
            break;
        }
        // Should start with spaces (indentation)
        assert!(
            line.starts_with("    ") || line.starts_with("        "),
            "Line not properly indented: '{line}'"
        );
    }
}
