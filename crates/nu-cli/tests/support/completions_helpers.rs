use std::path::PathBuf;

use nu_command::create_default_context;
use nu_engine::eval_block;
use nu_parser::parse;
use nu_protocol::{
    engine::{EngineState, Stack, StateDelta, StateWorkingSet},
    PipelineData, ShellError, Span, Value,
};
use nu_test_support::fs;
use reedline::Suggestion;
const SEP: char = std::path::MAIN_SEPARATOR;

// creates a new engine with the current path into the completions fixtures folder
pub fn new_engine() -> (PathBuf, String, EngineState, Stack) {
    // Target folder inside assets
    let dir = fs::fixtures().join("completions");
    let mut dir_str = dir
        .clone()
        .into_os_string()
        .into_string()
        .unwrap_or_default();
    dir_str.push(SEP);

    // Create a new engine with default context
    let mut engine_state = create_default_context(&dir);

    // New stack
    let mut stack = Stack::new();

    // New delta state
    let delta = StateDelta::new(&engine_state);

    // Add pwd as env var
    stack.add_env_var(
        "PWD".to_string(),
        Value::String {
            val: dir_str.clone(),
            span: nu_protocol::Span {
                start: 0,
                end: dir_str.len(),
            },
        },
    );

    // Merge delta
    let merge_result = engine_state.merge_delta(delta, Some(&mut stack), &dir);
    assert!(merge_result.is_ok());

    (dir, dir_str, engine_state, stack)
}

// match a list of suggestions with the expected values
pub fn match_suggestions(expected: Vec<String>, suggestions: Vec<Suggestion>) {
    expected.iter().zip(suggestions).for_each(|it| {
        assert_eq!(it.0, &it.1.value);
    });
}

// append the separator to the converted path
pub fn folder(path: PathBuf) -> String {
    let mut converted_path = file(path);
    converted_path.push(SEP);

    converted_path
}

// convert a given path to string
pub fn file(path: PathBuf) -> String {
    path.into_os_string().into_string().unwrap_or_default()
}

// merge_input executes the given input into the engine
// and merges the state
pub fn merge_input(
    input: &[u8],
    engine_state: &mut EngineState,
    stack: &mut Stack,
    dir: PathBuf,
) -> Result<(), ShellError> {
    let (block, delta) = {
        let mut working_set = StateWorkingSet::new(&engine_state);

        let (block, err) = parse(&mut working_set, None, input, false, &[]);

        assert!(err.is_none());

        (block, working_set.render())
    };
    assert!(eval_block(
        &engine_state,
        stack,
        &block,
        PipelineData::Value(
            Value::Nothing {
                span: Span { start: 0, end: 0 },
            },
            None
        ),
        false,
        false
    )
    .is_ok());

    // Merge delta
    engine_state.merge_delta(delta, Some(stack), &dir)
}
