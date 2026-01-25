use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::domain::types::Flag;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(rename_all = "snake_case")]
pub enum Signal {
    // Network
    Vpn,
    Proxy,
    Tor,
    Relay,
    PublicVpn,
    Hosting,
    TimezoneMismatch,

    // Device
    Rooted,
    Jailbroken,
    Emulator,
    VirtualMachine,
    Tampering,
    ClonedApp,
    FridaDetected,

    // Env
    Incognito,
    DevtoolsOpen,
    RemoteControlSuspected,

    // Behavior
    BotDetected,
    SuspiciousMouse,
    SuspiciousKeypress,
    PasteUsed,
    AutofillUsed,

    // Identity
    EmailDisposable,
    EmailBreached,
    PhoneVoip,
    PhoneRecentPort,
    HasSocialProfiles,
}

#[derive(Default, Debug, Clone, Serialize)]
pub struct Signals {
    pub flags: BTreeMap<Signal, Flag>,
}

impl Signals {
    pub fn has_yes(&self, s: Signal) -> bool {
        matches!(
            self.flags.get(&s).copied().unwrap_or(Flag::Unknown),
            Flag::Yes
        )
    }

    pub fn contains(&self, s: Signal) -> bool {
        self.flags.contains_key(&s)
    }
}
