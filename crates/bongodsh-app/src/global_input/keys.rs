use bongodsh_core::{InputEvent, KeyId};

/// Tracks physical keys without layout translation or character conversion.
pub(super) struct KeyFilter {
    held: [bool; 512],
    blocked: [bool; 512],
}

impl Default for KeyFilter {
    fn default() -> Self {
        Self {
            held: [false; 512],
            blocked: [false; 512],
        }
    }
}

impl KeyFilter {
    pub(super) fn event(
        &mut self,
        code: u16,
        pressed: bool,
        repeat: bool,
        typing: bool,
        shortcut: bool,
    ) -> Option<InputEvent> {
        let index = usize::from(code);
        let held = self.held.get_mut(index)?;
        let blocked = &mut self.blocked[index];
        let key = KeyId::new(u32::from(code));
        if !pressed {
            *held = false;
            *blocked = false;
            return Some(InputEvent::Release(key));
        }
        if !typing || shortcut {
            *blocked = true;
            return None;
        }
        if repeat || *held || *blocked {
            return None;
        }
        *held = true;
        Some(InputEvent::Press { key, repeat: false })
    }

    pub(super) fn reset(&mut self) {
        for (held, blocked) in self.held.iter_mut().zip(&mut self.blocked) {
            *blocked |= *held;
            *held = false;
        }
    }

    pub(super) fn tracked(&self) -> impl Iterator<Item = u16> + '_ {
        self.held
            .iter()
            .zip(&self.blocked)
            .enumerate()
            .filter(|(_, (held, blocked))| **held || **blocked)
            .map(|(code, _)| code as u16)
    }
}

#[cfg(target_os = "macos")]
pub(super) fn mac_typing_key(code: u16) -> bool {
    matches!(code, 0..=51 | 65 | 67 | 69 | 75 | 76 | 78 | 81..=89 | 91..=95)
}

#[cfg(target_os = "windows")]
pub(super) fn windows_typing_key(code: u16) -> bool {
    matches!(code,
        0x02..=0x1c | 0x1e..=0x29 | 0x2b..=0x35 | 0x37 | 0x39 |
        0x47..=0x53 | 0x56 | 0x73 | 0x7d | 0x11c | 0x135
    )
}
