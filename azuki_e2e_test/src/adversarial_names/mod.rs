// These legal spellings deliberately resemble generated constants and collide
// after conversion to snake case.
#![allow(non_camel_case_types)]

mod azuki_bridge;

include!(concat!(env!("OUT_DIR"), "/adversarial_names.rs"));

use parser::non_terminals::{
    A_b, AB, ACTION_TABLE, Action, EoiOrTerminal, EoiOrTerminalKind, GOTO_TABLE, Node, Node2,
    NonTerminalKind, State,
};
use parser::{S, parse};

#[test]
fn adversarial_names() {
    for (action_tokens, action) in [
        (
            vec![S::Shift("shift nodes"), S::Reduce("shift states")],
            Action::Shift {
                nodes: "shift nodes",
                states: "shift states",
            },
        ),
        (
            vec![
                S::Accept("reduce first"),
                S::Err("discard tuple field"),
                S::Terminal("reduce last"),
            ],
            Action::Reduce("reduce first", "reduce last"),
        ),
        (vec![], Action::Accept),
        (vec![S::NonTerminalNode("discard named field")], Action::Err),
    ] {
        // Exercise the empty and repeated tails of the manual non-terminal.
        let tail = ["tail first", "tail second", "tail third"];
        for tail_len in [0, 1, 3] {
            let mut tokens = vec![
                S::Eoi("src"),
                S::Eoi2("input"),
                S::Eoi3("states"),
                S::State2("state first"),
                S::State3("state last"),
                S::Node2("discard struct field"),
                S::Node("nodes"),
            ];
            tokens.extend(action_tokens.clone());
            tokens.extend([
                S::RuleKind2("manual first"),
                S::RuleKind3("discard manual argument"),
                S::RuleKind("manual last"),
            ]);
            for &text in &tail[..tail_len] {
                tokens.extend([S::R1("discard separator"), S::S0(text)]);
            }
            tokens.extend([
                S::ACTION_TABLE2("new_state"),
                S::GOTO_TABLE2("new_node"),
                S::AB("new_kind"),
                S::A_b("value"),
                S::AB("terminal_value"),
                S::A_b("t0"),
                S::S2("t1"),
                S::S3("t2"),
                S::EoiOrTerminal2("reduce"),
                S::EoiOrTerminal3("reduce_r1"),
                S::EoiOrTerminalKind2("get_action"),
                S::EoiOrTerminalKind3("get_goto"),
                S::NonTerminalKind2("pop_and_reduce"),
                S::NonTerminalKind3("from_terminal"),
                S::Node3("try_into_terminal"),
                S::Action2("azuki_bridge"),
                S::Action3("std"),
                S::ACTION_TABLE3("non_terminals"),
                S::GOTO_TABLE3("states_0"),
            ]);

            let actual = parse(tokens.clone()).unwrap();
            let mut manual = vec!["manual first", "manual last"];
            manual.extend_from_slice(&tail[..tail_len]);
            let expected = Node {
                src: EoiOrTerminal("src"),
                input: EoiOrTerminalKind("input"),
                states: NonTerminalKind("states"),
                nodes: State("state first", "state last"),
                rule: Node2("nodes"),
                top_state: action.clone(),
                next_kind: manual,
                new_state: ACTION_TABLE("new_state"),
                new_node: GOTO_TABLE("new_node"),
                new_kind: AB("new_kind"),
                value: A_b("value"),
                terminal_value: "terminal_value",
                t0: "t0",
                t1: "t1",
                t2: "t2",
                reduce: "reduce",
                reduce_r1: "reduce_r1",
                get_action: "get_action",
                get_goto: "get_goto",
                pop_and_reduce: "pop_and_reduce",
                from_terminal: "from_terminal",
                try_into_terminal: "try_into_terminal",
                azuki_bridge: "azuki_bridge",
                std: "std",
                non_terminals: "non_terminals",
                states_0: "states_0",
            };
            assert_eq!(expected, actual);

            let mut truncated = tokens.clone();
            truncated.pop();
            assert_eq!(Err(None), parse(truncated));

            let mut unexpected = tokens.clone();
            unexpected[0] = S::S3("unexpected");
            assert_eq!(Err(Some(S::S3("unexpected"))), parse(unexpected));

            // These are ordinary input tokens, even though their names occupy
            // the preferred end-of-input name and its numeric alternatives.
            for extra in [S::Eoi("extra"), S::Eoi2("extra"), S::Eoi3("extra")] {
                let mut trailing = tokens.clone();
                trailing.push(extra.clone());
                assert_eq!(Err(Some(extra)), parse(trailing));
            }
        }
    }
    assert_eq!(Err(None), parse([]));
}
