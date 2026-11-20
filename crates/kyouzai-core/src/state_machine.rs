pub const FALLING_STATE: u32 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompanionState {
    Idle,
    Walk,
    Sleep,
    Grabbed,
    Custom(u32),
}

impl CompanionState {
    pub fn animation_key(&self) -> std::borrow::Cow<'static, str> {
        match self {
            Self::Idle => std::borrow::Cow::Borrowed("idle"),
            Self::Walk => std::borrow::Cow::Borrowed("walk"),
            Self::Sleep => std::borrow::Cow::Borrowed("sleep"),
            Self::Grabbed => std::borrow::Cow::Borrowed("grabbed"),
            Self::Custom(id) if *id == FALLING_STATE => std::borrow::Cow::Borrowed("falling"),
            Self::Custom(id) => std::borrow::Cow::Owned(format!("custom_{id}")),
        }
    }
}

#[derive(Debug, Clone)]
pub struct CompanionStateMachine {
    current: CompanionState,
}

impl Default for CompanionStateMachine {
    fn default() -> Self {
        Self {
            current: CompanionState::Idle,
        }
    }
}

impl CompanionStateMachine {
    pub fn current(&self) -> CompanionState {
        self.current
    }

    pub fn can_transition(&self, next: CompanionState) -> bool {
        if next == self.current {
            return false;
        }

        match (self.current, next) {
            (CompanionState::Sleep, CompanionState::Idle | CompanionState::Grabbed) => true,
            (CompanionState::Sleep, _) => false,

            (CompanionState::Grabbed, CompanionState::Custom(id)) if id == FALLING_STATE => true,
            (CompanionState::Grabbed, _) => false,

            (CompanionState::Custom(id), CompanionState::Grabbed) if id == FALLING_STATE => true,
            (CompanionState::Custom(id), CompanionState::Idle) if id == FALLING_STATE => true,
            (CompanionState::Custom(id), _) if id == FALLING_STATE => false,

            _ => true,
        }
    }

    pub fn transition(&mut self, next: CompanionState) {
        self.current = next;
    }
}
