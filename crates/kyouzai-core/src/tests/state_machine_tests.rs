use crate::state_machine::*;

#[test]
fn cannot_go_directly_from_grabbed_to_idle() {
    let mut machine = CompanionStateMachine::default();
    machine.transition(CompanionState::Grabbed);

    assert!(!machine.can_transition(CompanionState::Idle));
    assert!(machine.can_transition(CompanionState::Custom(FALLING_STATE)));
}

#[test]
fn falling_can_land_on_idle_or_be_grabbed() {
    let mut machine = CompanionStateMachine::default();
    machine.transition(CompanionState::Custom(FALLING_STATE));

    assert!(machine.can_transition(CompanionState::Idle));
    assert!(machine.can_transition(CompanionState::Grabbed));
    assert!(!machine.can_transition(CompanionState::Walk));
}

#[test]
fn sleep_can_wake_to_idle_or_be_grabbed_but_not_walk() {
    let mut machine = CompanionStateMachine::default();
    machine.transition(CompanionState::Sleep);

    assert!(machine.can_transition(CompanionState::Idle));
    assert!(machine.can_transition(CompanionState::Grabbed));
    assert!(!machine.can_transition(CompanionState::Walk));
}

#[test]
fn transitioning_to_the_same_state_is_not_a_valid_transition() {
    let machine = CompanionStateMachine::default();
    assert!(!machine.can_transition(CompanionState::Idle));
}
