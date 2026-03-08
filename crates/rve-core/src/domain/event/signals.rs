use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::domain::common::Flag;

/// Discrete risk signals attached to an event.
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

/// Signal map keyed by [`Signal`], holding tri-state [`Flag`] values.
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct Signals {
  pub flags: BTreeMap<Signal, Flag>,
}

impl Signals {
  /// Returns `true` when `s` exists and its flag is `yes`.
  pub fn has_yes(&self, s: Signal) -> bool {
    matches!(self.flags.get(&s).copied().unwrap_or(Flag::Unknown), Flag::Yes)
  }

  /// Returns `true` if `s` exists in the map.
  pub fn contains(&self, s: Signal) -> bool {
    self.flags.contains_key(&s)
  }
}
